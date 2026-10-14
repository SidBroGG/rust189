use crate::codec::string::{read_string, write_string};
use crate::codec::varint::{read_varint, write_varint};
use crate::error::ProtocolError;
use bytes::{Buf, BufMut};

/// The next protocol state indicated in the Handshake packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextState {
    Status = 1,
    Login = 2,
}

impl NextState {
    pub fn from_i32(val: i32) -> Result<Self, ProtocolError> {
        match val {
            1 => Ok(NextState::Status),
            2 => Ok(NextState::Login),
            other => Err(ProtocolError::UnexpectedPacket {
                expected: "NextState 1 (Status) or 2 (Login)",
                actual: other,
            }),
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

/// Client Handshake packet (Packet ID: 0x00 in Handshake state).
#[derive(Debug, Clone, PartialEq)]
pub struct HandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: NextState,
}

impl HandshakePacket {
    pub const PACKET_ID: i32 = 0x00;

    /// Creates a handshake packet for Minecraft 1.8.9 (Protocol 47).
    pub fn new_v47(server_address: impl Into<String>, server_port: u16, next_state: NextState) -> Self {
        Self {
            protocol_version: 47,
            server_address: server_address.into(),
            server_port,
            next_state,
        }
    }

    /// Encodes the packet payload into a buffer.
    pub fn encode(&self, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
        write_varint(self.protocol_version, buf);
        write_string(&self.server_address, buf)?;
        buf.put_u16(self.server_port);
        write_varint(self.next_state.to_i32(), buf);
        Ok(())
    }

    /// Decodes the packet payload from a buffer.
    pub fn decode(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        let protocol_version = read_varint(buf)?;
        let server_address = read_string(buf, 255)?;
        if buf.remaining() < 2 {
            return Err(ProtocolError::UnexpectedEof);
        }
        let server_port = buf.get_u16();
        let next_state_id = read_varint(buf)?;
        let next_state = NextState::from_i32(next_state_id)?;

        Ok(Self {
            protocol_version,
            server_address,
            server_port,
            next_state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_handshake_roundtrip() {
        let pkt = HandshakePacket::new_v47("127.0.0.1", 25565, NextState::Login);
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut reader = buf.freeze();
        let decoded = HandshakePacket::decode(&mut reader).unwrap();
        assert_eq!(decoded, pkt);
    }
}
