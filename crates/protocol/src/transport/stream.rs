use crate::codec::varint::{read_varint, varint_size, write_varint};
use crate::crypto::Aes128Cfb8Cipher;
use crate::error::ProtocolError;
use crate::transport::compression::{compress_zlib, decompress_zlib};
use crate::transport::packet::RawPacket;
use bytes::{Buf, Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Attempts to parse a VarInt from a byte slice without consuming or advancing.
/// Returns `Ok(Some((value, bytes_consumed)))` if a complete VarInt is present,
/// `Ok(None)` if more bytes are needed, or `Err(VarIntTooBig)`.
pub fn try_read_varint(slice: &[u8]) -> Result<Option<(i32, usize)>, ProtocolError> {
    let mut value: i32 = 0;
    let mut shift: u32 = 0;
    for (idx, &byte) in slice.iter().enumerate() {
        value |= ((byte & 0x7F) as i32) << shift;
        if (byte & 0x80) == 0 {
            return Ok(Some((value, idx + 1)));
        }
        shift += 7;
        if shift >= 35 {
            return Err(ProtocolError::VarIntTooBig);
        }
    }
    Ok(None)
}

/// Asynchronous framed Minecraft 1.8.9 packet stream supporting
/// framing, streaming compression (Zlib), and symmetric encryption (AES-128/CFB8).
pub struct AsyncPacketStream<IO> {
    io: IO,
    read_buffer: BytesMut,
    compression_threshold: Option<i32>,
    cipher: Option<Aes128Cfb8Cipher>,
}

impl<IO: AsyncRead + AsyncWrite + Unpin> AsyncPacketStream<IO> {
    /// Creates a new packet stream from an underlying async reader/writer.
    pub fn new(io: IO) -> Self {
        Self {
            io,
            read_buffer: BytesMut::with_capacity(8192),
            compression_threshold: None,
            cipher: None,
        }
    }

    /// Sets the Zlib packet compression threshold.
    /// If threshold >= 0, compression is enabled. If < 0, compression is disabled.
    pub fn set_compression(&mut self, threshold: i32) {
        if threshold >= 0 {
            self.compression_threshold = Some(threshold);
        } else {
            self.compression_threshold = None;
        }
    }

    /// Enables AES-128/CFB8 symmetric encryption using the specified shared secret.
    pub fn enable_encryption(&mut self, shared_secret: &[u8; 16]) {
        self.cipher = Some(Aes128Cfb8Cipher::new(shared_secret));
    }

    /// Reads the next complete packet from the stream.
    pub async fn read_packet(&mut self) -> Result<RawPacket, ProtocolError> {
        loop {
            // Check if we can parse the packet length VarInt
            if let Some((packet_len, len_bytes)) = try_read_varint(&self.read_buffer)? {
                if packet_len < 0 {
                    return Err(ProtocolError::PacketTooLarge {
                        max: 2097152,
                        actual: packet_len as usize,
                    });
                }
                let packet_len = packet_len as usize;
                if packet_len > 2097152 {
                    return Err(ProtocolError::PacketTooLarge {
                        max: 2097152,
                        actual: packet_len,
                    });
                }

                let total_required = len_bytes + packet_len;
                if self.read_buffer.len() >= total_required {
                    self.read_buffer.advance(len_bytes);
                    let mut packet_body = self.read_buffer.split_to(packet_len);

                    match self.compression_threshold {
                        None => {
                            let id = read_varint(&mut packet_body)?;
                            let payload = packet_body.freeze();
                            return Ok(RawPacket::new(id, payload));
                        }
                        Some(_) => {
                            let data_len = read_varint(&mut packet_body)?;
                            if data_len == 0 {
                                let id = read_varint(&mut packet_body)?;
                                let payload = packet_body.freeze();
                                return Ok(RawPacket::new(id, payload));
                            } else {
                                if data_len < 0 || (data_len as usize) > 2097152 {
                                    return Err(ProtocolError::PacketTooLarge {
                                        max: 2097152,
                                        actual: data_len as usize,
                                    });
                                }
                                let decompressed =
                                    decompress_zlib(&packet_body, data_len as usize)?;
                                let mut decompressed_slice = &decompressed[..];
                                let id = read_varint(&mut decompressed_slice)?;
                                let payload = Bytes::copy_from_slice(decompressed_slice);
                                return Ok(RawPacket::new(id, payload));
                            }
                        }
                    }
                }
            }

            // Need more data from the network
            let mut temp = [0u8; 8192];
            let n = self.io.read(&mut temp).await?;
            if n == 0 {
                if self.read_buffer.is_empty() {
                    return Err(ProtocolError::ConnectionClosed);
                } else {
                    return Err(ProtocolError::UnexpectedEof);
                }
            }

            let chunk = &mut temp[..n];
            if let Some(ref mut cipher) = self.cipher {
                cipher.decrypt(chunk);
            }
            self.read_buffer.extend_from_slice(chunk);
        }
    }

    /// Writes a framed packet with ID and payload into the stream and flushes.
    pub async fn write_packet(&mut self, id: i32, payload: &[u8]) -> Result<(), ProtocolError> {
        let mut body = BytesMut::with_capacity(varint_size(id) + payload.len());
        write_varint(id, &mut body);
        body.extend_from_slice(payload);

        let mut frame = BytesMut::new();
        match self.compression_threshold {
            None => {
                write_varint(body.len() as i32, &mut frame);
                frame.extend_from_slice(&body);
            }
            Some(threshold) => {
                if body.len() < threshold as usize {
                    write_varint((varint_size(0) + body.len()) as i32, &mut frame);
                    write_varint(0, &mut frame);
                    frame.extend_from_slice(&body);
                } else {
                    let compressed = compress_zlib(&body)?;
                    let data_len = body.len() as i32;
                    write_varint((varint_size(data_len) + compressed.len()) as i32, &mut frame);
                    write_varint(data_len, &mut frame);
                    frame.extend_from_slice(&compressed);
                }
            }
        }

        if let Some(ref mut cipher) = self.cipher {
            cipher.encrypt(&mut frame);
        }

        self.io.write_all(&frame).await?;
        self.io.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn test_uncompressed_packet_roundtrip() {
        let (client_io, server_io) = duplex(4096);
        let mut client = AsyncPacketStream::new(client_io);
        let mut server = AsyncPacketStream::new(server_io);

        let payload = b"Hello uncompressed world!";
        tokio::spawn(async move {
            client.write_packet(0x02, payload).await.unwrap();
        });

        let packet = server.read_packet().await.unwrap();
        assert_eq!(packet.id, 0x02);
        assert_eq!(&packet.payload[..], payload);
    }

    #[tokio::test]
    async fn test_compressed_packet_roundtrip() {
        let (client_io, server_io) = duplex(4096);
        let mut client = AsyncPacketStream::new(client_io);
        let mut server = AsyncPacketStream::new(server_io);

        // Threshold of 128 bytes
        client.set_compression(128);
        server.set_compression(128);

        // Packet 1: smaller than threshold (uncompressed path)
        let small_payload = b"Small packet";
        // Packet 2: larger than threshold (compressed path)
        let large_payload = vec![0x42; 512];

        let small_clone = small_payload.to_vec();
        let large_clone = large_payload.clone();

        tokio::spawn(async move {
            client.write_packet(0x01, &small_clone).await.unwrap();
            client.write_packet(0x02, &large_clone).await.unwrap();
        });

        let p1 = server.read_packet().await.unwrap();
        assert_eq!(p1.id, 0x01);
        assert_eq!(&p1.payload[..], small_payload);

        let p2 = server.read_packet().await.unwrap();
        assert_eq!(p2.id, 0x02);
        assert_eq!(&p2.payload[..], &large_payload[..]);
    }

    #[tokio::test]
    async fn test_encrypted_packet_roundtrip() {
        let (client_io, server_io) = duplex(4096);
        let mut client = AsyncPacketStream::new(client_io);
        let mut server = AsyncPacketStream::new(server_io);

        let secret = [0x77u8; 16];
        client.enable_encryption(&secret);
        server.enable_encryption(&secret);

        let payload = b"Secret confidential encrypted packet!";
        tokio::spawn(async move {
            client.write_packet(0x05, payload).await.unwrap();
        });

        let packet = server.read_packet().await.unwrap();
        assert_eq!(packet.id, 0x05);
        assert_eq!(&packet.payload[..], payload);
    }
}
