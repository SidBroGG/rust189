use crate::error::ProtocolError;
use bytes::{Buf, BufMut};

/// Reads a VarInt from a buffer without heap allocations.
///
/// Follows Minecraft 1.8.9 MCP `PacketBuffer.readVarIntFromBuffer`:
/// reads up to 5 bytes, where the 7 lower bits represent payload and
/// the 8th bit indicates if more bytes follow.
pub fn read_varint(buf: &mut impl Buf) -> Result<i32, ProtocolError> {
    let mut value: i32 = 0;
    let mut shift: u32 = 0;

    loop {
        if !buf.has_remaining() {
            return Err(ProtocolError::UnexpectedEof);
        }

        let byte = buf.get_u8();
        value |= ((byte & 0x7F) as i32) << shift;

        if (byte & 0x80) == 0 {
            return Ok(value);
        }

        shift += 7;
        if shift >= 35 {
            return Err(ProtocolError::VarIntTooBig);
        }
    }
}

/// Writes a VarInt into a buffer.
///
/// Follows Minecraft 1.8.9 MCP `PacketBuffer.writeVarIntToBuffer`.
pub fn write_varint(val: i32, buf: &mut impl BufMut) {
    let mut v = val as u32;
    while (v & !0x7F) != 0 {
        buf.put_u8(((v & 0x7F) | 0x80) as u8);
        v >>= 7;
    }
    buf.put_u8(v as u8);
}

/// Returns the size in bytes of a VarInt when encoded.
#[inline]
pub fn varint_size(val: i32) -> usize {
    let mut v = val as u32;
    let mut count = 0;
    loop {
        count += 1;
        v >>= 7;
        if v == 0 {
            break;
        }
    }
    count
}

/// Reads a VarLong from a buffer without heap allocations.
///
/// Follows Minecraft 1.8.9 MCP `PacketBuffer.readVarLong`:
/// reads up to 10 bytes.
pub fn read_varlong(buf: &mut impl Buf) -> Result<i64, ProtocolError> {
    let mut value: i64 = 0;
    let mut shift: u32 = 0;

    loop {
        if !buf.has_remaining() {
            return Err(ProtocolError::UnexpectedEof);
        }

        let byte = buf.get_u8();
        value |= ((byte & 0x7F) as i64) << shift;

        if (byte & 0x80) == 0 {
            return Ok(value);
        }

        shift += 7;
        if shift >= 70 {
            return Err(ProtocolError::VarLongTooBig);
        }
    }
}

/// Writes a VarLong into a buffer.
///
/// Follows Minecraft 1.8.9 MCP `PacketBuffer.writeVarLong`.
pub fn write_varlong(val: i64, buf: &mut impl BufMut) {
    let mut v = val as u64;
    while (v & !0x7F) != 0 {
        buf.put_u8(((v & 0x7F) | 0x80) as u8);
        v >>= 7;
    }
    buf.put_u8(v as u8);
}

/// Returns the size in bytes of a VarLong when encoded.
#[inline]
pub fn varlong_size(val: i64) -> usize {
    let mut v = val as u64;
    let mut count = 0;
    loop {
        count += 1;
        v >>= 7;
        if v == 0 {
            break;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_varint_roundtrip() {
        let test_cases = [
            0,
            1,
            2,
            127,
            128,
            255,
            256,
            2097151,
            2147483647,
            -1,
            -2147483648,
        ];

        for &val in &test_cases {
            let mut buf = BytesMut::new();
            write_varint(val, &mut buf);
            assert_eq!(buf.len(), varint_size(val), "varint_size mismatch for {}", val);

            let mut reader = buf.freeze();
            let decoded = read_varint(&mut reader).expect("failed to decode varint");
            assert_eq!(decoded, val, "roundtrip failed for {}", val);
            assert!(!reader.has_remaining(), "leftover bytes in buffer");
        }
    }

    #[test]
    fn test_varlong_roundtrip() {
        let test_cases = [
            0i64,
            1,
            127,
            128,
            2147483647,
            -1,
            9223372036854775807,
            -9223372036854775808,
        ];

        for &val in &test_cases {
            let mut buf = BytesMut::new();
            write_varlong(val, &mut buf);
            assert_eq!(buf.len(), varlong_size(val), "varlong_size mismatch for {}", val);

            let mut reader = buf.freeze();
            let decoded = read_varlong(&mut reader).expect("failed to decode varlong");
            assert_eq!(decoded, val, "roundtrip failed for {}", val);
            assert!(!reader.has_remaining(), "leftover bytes in buffer");
        }
    }

    #[test]
    fn test_varint_overflow() {
        let invalid = [0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        let mut slice = &invalid[..];
        assert!(matches!(read_varint(&mut slice), Err(ProtocolError::VarIntTooBig)));
    }
}
