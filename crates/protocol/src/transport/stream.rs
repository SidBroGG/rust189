use crate::codec::varint::{decode_varint_slice, read_varint, varint_size, write_varint};
use crate::crypto::Aes128Cfb8Cipher;
use crate::error::ProtocolError;
use crate::transport::compression::{compress_zlib, decompress_zlib};
use crate::transport::packet::RawPacket;
use crate::transport::ring_buffer::ZeroCopyRingBuffer;
use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Attempts to parse a VarInt from a byte slice using the hardware/fallback VarInt decoder.
/// Returns `Ok(Some((value, bytes_consumed)))` if a complete VarInt is present,
/// `Ok(None)` if more bytes are needed, or `Err(VarIntTooBig)`.
pub fn try_read_varint(slice: &[u8]) -> Result<Option<(i32, usize)>, ProtocolError> {
    match decode_varint_slice(slice) {
        Ok((val, len)) => Ok(Some((val, len))),
        Err(ProtocolError::UnexpectedEof) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Asynchronous framed Minecraft 1.8.9 packet stream supporting
/// zero-copy ring buffering, streaming compression (Zlib), and symmetric encryption (AES-128/CFB8).
pub struct AsyncPacketStream<IO> {
    io: IO,
    ring_buffer: ZeroCopyRingBuffer,
    write_buffer: BytesMut,
    compression_threshold: Option<i32>,
    cipher: Option<Aes128Cfb8Cipher>,
}

impl<IO: AsyncRead + AsyncWrite + Unpin> AsyncPacketStream<IO> {
    /// Creates a new packet stream from an underlying async reader/writer.
    pub fn new(io: IO) -> Self {
        Self {
            io,
            ring_buffer: ZeroCopyRingBuffer::new(),
            write_buffer: BytesMut::with_capacity(8192),
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
            // Check if we can parse the packet length VarInt from the contiguous readable slice
            let readable = self.ring_buffer.readable_slice();
            if let Some((packet_len, len_bytes)) = try_read_varint(readable)? {
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
                if readable.len() >= total_required {
                    let packet_data = &readable[len_bytes..total_required];
                    let raw_packet = match self.compression_threshold {
                        None => {
                            let mut slice = packet_data;
                            let id = read_varint(&mut slice)?;
                            let payload = Bytes::copy_from_slice(slice);
                            RawPacket::new(id, payload)
                        }
                        Some(_) => {
                            let mut slice = packet_data;
                            let data_len = read_varint(&mut slice)?;
                            if data_len == 0 {
                                let id = read_varint(&mut slice)?;
                                let payload = Bytes::copy_from_slice(slice);
                                RawPacket::new(id, payload)
                            } else {
                                if data_len < 0 || (data_len as usize) > 2097152 {
                                    return Err(ProtocolError::PacketTooLarge {
                                        max: 2097152,
                                        actual: data_len as usize,
                                    });
                                }
                                let decompressed = decompress_zlib(slice, data_len as usize)?;
                                let mut decompressed_slice = &decompressed[..];
                                let id = read_varint(&mut decompressed_slice)?;
                                let payload = Bytes::copy_from_slice(decompressed_slice);
                                RawPacket::new(id, payload)
                            }
                        }
                    };

                    self.ring_buffer.advance_read(total_required);
                    return Ok(raw_packet);
                }
            }

            // Direct zero-copy read from OS network socket into pre-allocated ring buffer
            let write_slice = self.ring_buffer.prepare_write_slice(8192);
            let n = self.io.read(write_slice).await?;
            if n == 0 {
                if self.ring_buffer.is_empty() {
                    return Err(ProtocolError::ConnectionClosed);
                } else {
                    return Err(ProtocolError::UnexpectedEof);
                }
            }

            // In-place decryption directly within the ring buffer slice without allocations
            if let Some(ref mut cipher) = self.cipher {
                cipher.decrypt(&mut write_slice[..n]);
            }
            self.ring_buffer.commit_write(n);
        }
    }

    /// Writes a framed packet with ID and payload into the stream and flushes.
    pub async fn write_packet(&mut self, id: i32, payload: &[u8]) -> Result<(), ProtocolError> {
        self.write_buffer.clear();
        let id_size = varint_size(id);
        let uncompressed_len = id_size + payload.len();

        match self.compression_threshold {
            None => {
                write_varint(uncompressed_len as i32, &mut self.write_buffer);
                write_varint(id, &mut self.write_buffer);
                self.write_buffer.extend_from_slice(payload);
            }
            Some(threshold) => {
                if uncompressed_len < threshold as usize {
                    let data_len_size = varint_size(0);
                    let packet_len = data_len_size + uncompressed_len;
                    write_varint(packet_len as i32, &mut self.write_buffer);
                    write_varint(0, &mut self.write_buffer);
                    write_varint(id, &mut self.write_buffer);
                    self.write_buffer.extend_from_slice(payload);
                } else {
                    let mut body = Vec::with_capacity(uncompressed_len);
                    write_varint(id, &mut body);
                    body.extend_from_slice(payload);
                    let compressed = compress_zlib(&body)?;

                    let data_len = uncompressed_len as i32;
                    let data_len_size = varint_size(data_len);
                    let packet_len = data_len_size + compressed.len();
                    write_varint(packet_len as i32, &mut self.write_buffer);
                    write_varint(data_len, &mut self.write_buffer);
                    self.write_buffer.extend_from_slice(&compressed);
                }
            }
        }

        if let Some(ref mut cipher) = self.cipher {
            cipher.encrypt(&mut self.write_buffer);
        }

        self.io.write_all(&self.write_buffer).await?;
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
