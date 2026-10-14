use crate::codec::string::{read_string, write_string};
use crate::codec::varint::{read_varint, write_varint};
use crate::error::ProtocolError;
use bytes::{Buf, BufMut};

/// Client Login Start packet (Packet ID: 0x00 in Login state).
#[derive(Debug, Clone, PartialEq)]
pub struct LoginStartPacket {
    pub username: String,
}

impl LoginStartPacket {
    pub const PACKET_ID: i32 = 0x00;

    pub fn new(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
        }
    }

    pub fn encode(&self, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
        write_string(&self.username, buf)
    }

    pub fn decode(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        let username = read_string(buf, 16)?;
        Ok(Self { username })
    }
}

/// Client Encryption Response packet (Packet ID: 0x01 in Login state).
#[derive(Debug, Clone, PartialEq)]
pub struct EncryptionResponsePacket {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

impl EncryptionResponsePacket {
    pub const PACKET_ID: i32 = 0x01;

    pub fn new(shared_secret: Vec<u8>, verify_token: Vec<u8>) -> Self {
        Self {
            shared_secret,
            verify_token,
        }
    }

    pub fn encode(&self, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
        write_varint(self.shared_secret.len() as i32, buf);
        buf.put_slice(&self.shared_secret);

        write_varint(self.verify_token.len() as i32, buf);
        buf.put_slice(&self.verify_token);
        Ok(())
    }

    pub fn decode(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        let secret_len = read_varint(buf)? as usize;
        if buf.remaining() < secret_len {
            return Err(ProtocolError::UnexpectedEof);
        }
        let mut shared_secret = vec![0u8; secret_len];
        buf.copy_to_slice(&mut shared_secret);

        let token_len = read_varint(buf)? as usize;
        if buf.remaining() < token_len {
            return Err(ProtocolError::UnexpectedEof);
        }
        let mut verify_token = vec![0u8; token_len];
        buf.copy_to_slice(&mut verify_token);

        Ok(Self {
            shared_secret,
            verify_token,
        })
    }
}

/// Server Disconnect (Login) packet (Packet ID: 0x00 in Login state).
#[derive(Debug, Clone, PartialEq)]
pub struct DisconnectLoginPacket {
    pub reason: String,
}

impl DisconnectLoginPacket {
    pub const PACKET_ID: i32 = 0x00;

    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    pub fn encode(&self, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
        write_string(&self.reason, buf)
    }

    pub fn decode(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        let reason = read_string(buf, 32767)?;
        Ok(Self { reason })
    }
}

/// Server Encryption Request packet (Packet ID: 0x01 in Login state).
#[derive(Debug, Clone, PartialEq)]
pub struct EncryptionRequestPacket {
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub verify_token: Vec<u8>,
}

impl EncryptionRequestPacket {
    pub const PACKET_ID: i32 = 0x01;

    pub fn new(server_id: impl Into<String>, public_key: Vec<u8>, verify_token: Vec<u8>) -> Self {
        Self {
            server_id: server_id.into(),
            public_key,
            verify_token,
        }
    }

    pub fn encode(&self, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
        write_string(&self.server_id, buf)?;

        write_varint(self.public_key.len() as i32, buf);
        buf.put_slice(&self.public_key);

        write_varint(self.verify_token.len() as i32, buf);
        buf.put_slice(&self.verify_token);
        Ok(())
    }

    pub fn decode(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        let server_id = read_string(buf, 20)?;

        let key_len = read_varint(buf)? as usize;
        if buf.remaining() < key_len {
            return Err(ProtocolError::UnexpectedEof);
        }
        let mut public_key = vec![0u8; key_len];
        buf.copy_to_slice(&mut public_key);

        let token_len = read_varint(buf)? as usize;
        if buf.remaining() < token_len {
            return Err(ProtocolError::UnexpectedEof);
        }
        let mut verify_token = vec![0u8; token_len];
        buf.copy_to_slice(&mut verify_token);

        Ok(Self {
            server_id,
            public_key,
            verify_token,
        })
    }
}

/// Server Login Success packet (Packet ID: 0x02 in Login state).
#[derive(Debug, Clone, PartialEq)]
pub struct LoginSuccessPacket {
    pub uuid: String,
    pub username: String,
}

impl LoginSuccessPacket {
    pub const PACKET_ID: i32 = 0x02;

    pub fn new(uuid: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            uuid: uuid.into(),
            username: username.into(),
        }
    }

    pub fn encode(&self, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
        write_string(&self.uuid, buf)?;
        write_string(&self.username, buf)?;
        Ok(())
    }

    pub fn decode(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        let uuid = read_string(buf, 36)?;
        let username = read_string(buf, 16)?;
        Ok(Self { uuid, username })
    }
}

/// Server Set Compression packet (Packet ID: 0x03 in Login state).
#[derive(Debug, Clone, PartialEq)]
pub struct SetCompressionPacket {
    pub threshold: i32,
}

impl SetCompressionPacket {
    pub const PACKET_ID: i32 = 0x03;

    pub fn new(threshold: i32) -> Self {
        Self { threshold }
    }

    pub fn encode(&self, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
        write_varint(self.threshold, buf);
        Ok(())
    }

    pub fn decode(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        let threshold = read_varint(buf)?;
        Ok(Self { threshold })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_login_start_roundtrip() {
        let pkt = LoginStartPacket::new("Alex_Dev");
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut reader = buf.freeze();
        let decoded = LoginStartPacket::decode(&mut reader).unwrap();
        assert_eq!(decoded, pkt);
    }

    #[test]
    fn test_login_success_roundtrip() {
        let pkt = LoginSuccessPacket::new(
            "5627dd98-e6be-3c21-b8a8-e92344183641",
            "Steve",
        );
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut reader = buf.freeze();
        let decoded = LoginSuccessPacket::decode(&mut reader).unwrap();
        assert_eq!(decoded, pkt);
    }

    #[test]
    fn test_set_compression_roundtrip() {
        let pkt = SetCompressionPacket::new(256);
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut reader = buf.freeze();
        let decoded = SetCompressionPacket::decode(&mut reader).unwrap();
        assert_eq!(decoded, pkt);
    }
}
