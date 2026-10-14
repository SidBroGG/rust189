use crate::error::ProtocolError;
use bytes::{Buf, BufMut};

/// Represents a voxel 3D block coordinate packed according to Minecraft 1.8.9.
///
/// Bit layout in Protocol 47:
/// - X: 26 bits signed (bits 38..63)
/// - Y: 12 bits signed (bits 26..37)
/// - Z: 26 bits signed (bits 0..25)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Packs the position into a 64-bit integer conforming to MCP `BlockPos.toLong()`.
    #[inline]
    pub fn to_u64(&self) -> u64 {
        let x_part = ((self.x as i64) & 0x03FF_FFFF) << 38;
        let y_part = ((self.y as i64) & 0x0FFF) << 26;
        let z_part = (self.z as i64) & 0x03FF_FFFF;
        (x_part | y_part | z_part) as u64
    }

    /// Unpacks a 64-bit integer into a Position conforming to MCP `BlockPos.fromLong()`.
    #[inline]
    pub fn from_u64(val: u64) -> Self {
        let s = val as i64;
        let x = (s >> 38) as i32;
        let y = ((s << 26) >> 52) as i32;
        let z = ((s << 38) >> 38) as i32;
        Self { x, y, z }
    }

    /// Reads a packed Position from buffer.
    pub fn read(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        if buf.remaining() < 8 {
            return Err(ProtocolError::UnexpectedEof);
        }
        Ok(Self::from_u64(buf.get_u64()))
    }

    /// Writes a packed Position to buffer.
    pub fn write(&self, buf: &mut impl BufMut) {
        buf.put_u64(self.to_u64());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_position_roundtrip() {
        let positions = [
            Position::new(0, 0, 0),
            Position::new(1, 64, -1),
            Position::new(-100, 255, 100),
            Position::new(-33554432, -2048, -33554432),
            Position::new(33554431, 2047, 33554431),
            Position::new(-1, -1, -1),
            Position::new(1234567, 128, -7654321),
        ];

        for &pos in &positions {
            let packed = pos.to_u64();
            let unpacked = Position::from_u64(packed);
            assert_eq!(unpacked, pos, "Position packing mismatch for {:?}", pos);

            let mut buf = BytesMut::new();
            pos.write(&mut buf);
            let mut reader = buf.freeze();
            let read_pos = Position::read(&mut reader).unwrap();
            assert_eq!(read_pos, pos, "Buffer read mismatch for {:?}", pos);
        }
    }
}
