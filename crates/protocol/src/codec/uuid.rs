use crate::error::ProtocolError;
use bytes::{Buf, BufMut};
use uuid::Uuid;

/// Reads a 128-bit UUID (most significant bits + least significant bits) from buffer.
///
/// Follows Minecraft 1.8.9 MCP `PacketBuffer.readUuid`.
pub fn read_uuid(buf: &mut impl Buf) -> Result<Uuid, ProtocolError> {
    if buf.remaining() < 16 {
        return Err(ProtocolError::UnexpectedEof);
    }
    let most = buf.get_u64();
    let least = buf.get_u64();
    Ok(Uuid::from_u64_pair(most, least))
}

/// Writes a 128-bit UUID (most significant bits + least significant bits) to buffer.
///
/// Follows Minecraft 1.8.9 MCP `PacketBuffer.writeUuid`.
pub fn write_uuid(uuid: &Uuid, buf: &mut impl BufMut) {
    let (most, least) = uuid.as_u64_pair();
    buf.put_u64(most);
    buf.put_u64(least);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_uuid_roundtrip() {
        let original = Uuid::new_v4();
        let mut buf = BytesMut::new();
        write_uuid(&original, &mut buf);
        assert_eq!(buf.len(), 16);

        let mut reader = buf.freeze();
        let decoded = read_uuid(&mut reader).unwrap();
        assert_eq!(decoded, original);
    }
}
