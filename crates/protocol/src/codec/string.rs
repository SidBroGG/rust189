use crate::codec::varint::{read_varint, write_varint};
use crate::error::ProtocolError;
use bytes::{Buf, BufMut};

/// Reads a VarInt-length prefixed UTF-8 string from the buffer.
///
/// Follows Minecraft 1.8.9 MCP `PacketBuffer.readStringFromBuffer`:
/// checks that byte length <= max_chars * 4 and character count <= max_chars.
pub fn read_string(buf: &mut impl Buf, max_chars: usize) -> Result<String, ProtocolError> {
    let byte_len = read_varint(buf)? as usize;

    if byte_len > max_chars * 4 {
        return Err(ProtocolError::StringTooLong {
            max: max_chars * 4,
            actual: byte_len,
        });
    }

    if buf.remaining() < byte_len {
        return Err(ProtocolError::UnexpectedEof);
    }

    let mut bytes = vec![0u8; byte_len];
    buf.copy_to_slice(&mut bytes);

    let s = String::from_utf8(bytes)?;
    let char_count = s.chars().count();
    if char_count > max_chars {
        return Err(ProtocolError::StringTooLong {
            max: max_chars,
            actual: char_count,
        });
    }

    Ok(s)
}

/// Writes a VarInt-length prefixed UTF-8 string into the buffer.
///
/// Follows Minecraft 1.8.9 MCP `PacketBuffer.writeString`.
pub fn write_string(s: &str, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
    let bytes = s.as_bytes();
    if bytes.len() > 32767 {
        return Err(ProtocolError::StringTooLong {
            max: 32767,
            actual: bytes.len(),
        });
    }

    write_varint(bytes.len() as i32, buf);
    buf.put_slice(bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_string_roundtrip() {
        let text = "Minecraft 1.8.9 High-Performance Client! Привет, мир! 🚀";
        let mut buf = BytesMut::new();
        write_string(text, &mut buf).unwrap();

        let mut reader = buf.freeze();
        let decoded = read_string(&mut reader, 32767).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_string_limit() {
        let text = "abcdefghij";
        let mut buf = BytesMut::new();
        write_string(text, &mut buf).unwrap();

        let mut reader = buf.freeze();
        let res = read_string(&mut reader, 5);
        assert!(res.is_err());
    }
}
