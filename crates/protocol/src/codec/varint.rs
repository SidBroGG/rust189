use crate::error::ProtocolError;
use bytes::{Buf, BufMut};

/// Checks whether the host CPU supports the BMI2 instruction set.
pub fn has_bmi2() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        use std::sync::atomic::{AtomicU8, Ordering};
        static BMI2_STATE: AtomicU8 = AtomicU8::new(0);
        match BMI2_STATE.load(Ordering::Relaxed) {
            1 => false,
            2 => true,
            _ => {
                let detected = std::is_x86_feature_detected!("bmi2");
                BMI2_STATE.store(if detected { 2 } else { 1 }, Ordering::Relaxed);
                detected
            }
        }
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

/// Decodes a VarInt from a slice using x86 BMI2 instructions (`_pext_u64`).
///
/// In Minecraft Protocol 47, VarInts are 7-bit chunks with the MSB as continuation bit.
/// PEXT extracts all 7-bit payloads simultaneously into contiguous low-order bits,
/// reducing multi-branch decoding loops into a handful of CPU cycles.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "bmi2")]
pub unsafe fn decode_varint_bmi2(slice: &[u8]) -> Result<(i32, usize), ProtocolError> {
    if slice.is_empty() {
        return Err(ProtocolError::UnexpectedEof);
    }

    let word = if slice.len() >= 8 {
        u64::from_le_bytes(slice[..8].try_into().unwrap())
    } else {
        let mut tmp = [0u8; 8];
        tmp[..slice.len()].copy_from_slice(slice);
        u64::from_le_bytes(tmp)
    };

    // Bitwise check for the first byte where MSB (bit 7) is 0 (terminating byte)
    let term = (!word) & 0x8080808080808080u64;
    if term == 0 {
        if slice.len() < 5 {
            return Err(ProtocolError::UnexpectedEof);
        }
        return Err(ProtocolError::VarIntTooBig);
    }

    let trailing = term.trailing_zeros();
    let byte_len = ((trailing as usize) >> 3) + 1;
    if byte_len > slice.len() {
        return Err(ProtocolError::UnexpectedEof);
    }
    if byte_len > 5 {
        return Err(ProtocolError::VarIntTooBig);
    }

    let raw = core::arch::x86_64::_pext_u64(word, 0x0000007F_7F7F7F7F);
    let mask = if byte_len < 5 {
        (1u64 << (byte_len * 7)) - 1
    } else {
        0xFFFFFFFFu64
    };
    let val = (raw & mask) as i32;
    Ok((val, byte_len))
}

/// Decodes a VarInt from a slice using a branch-predictable scalar loop.
pub fn decode_varint_fallback(slice: &[u8]) -> Result<(i32, usize), ProtocolError> {
    let mut value: i32 = 0;
    let mut shift: u32 = 0;
    for (idx, &byte) in slice.iter().enumerate() {
        value |= ((byte & 0x7F) as i32) << shift;
        if (byte & 0x80) == 0 {
            return Ok((value, idx + 1));
        }
        shift += 7;
        if shift >= 35 {
            return Err(ProtocolError::VarIntTooBig);
        }
    }
    Err(ProtocolError::UnexpectedEof)
}

/// Automatically selects hardware BMI2 or fallback path to decode a VarInt from a slice.
#[inline]
pub fn decode_varint_slice(slice: &[u8]) -> Result<(i32, usize), ProtocolError> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if has_bmi2() {
            return unsafe { decode_varint_bmi2(slice) };
        }
    }
    decode_varint_fallback(slice)
}

/// Decodes a VarLong from a slice using x86 BMI2 instructions (`_pext_u64`).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "bmi2")]
pub unsafe fn decode_varlong_bmi2(slice: &[u8]) -> Result<(i64, usize), ProtocolError> {
    if slice.is_empty() {
        return Err(ProtocolError::UnexpectedEof);
    }

    let word = if slice.len() >= 8 {
        u64::from_le_bytes(slice[..8].try_into().unwrap())
    } else {
        let mut tmp = [0u8; 8];
        tmp[..slice.len()].copy_from_slice(slice);
        u64::from_le_bytes(tmp)
    };

    let term = (!word) & 0x8080808080808080u64;
    if term != 0 {
        let trailing = term.trailing_zeros();
        let byte_len = ((trailing as usize) >> 3) + 1;
        if byte_len <= 8 {
            if byte_len > slice.len() {
                return Err(ProtocolError::UnexpectedEof);
            }
            let raw = core::arch::x86_64::_pext_u64(word, 0x7F7F7F7F_7F7F7F7F);
            let mask = if byte_len < 8 {
                (1u64 << (byte_len * 7)) - 1
            } else {
                0x00FFFFFFFFFFFFFFu64
            };
            let val = (raw & mask) as i64;
            return Ok((val, byte_len));
        }
    }

    // VarLongs that exceed 8 bytes are handled via the scalar fallback
    decode_varlong_fallback(slice)
}

/// Decodes a VarLong from a slice using a branch-predictable scalar loop.
pub fn decode_varlong_fallback(slice: &[u8]) -> Result<(i64, usize), ProtocolError> {
    let mut value: i64 = 0;
    let mut shift: u32 = 0;
    for (idx, &byte) in slice.iter().enumerate() {
        value |= ((byte & 0x7F) as i64) << shift;
        if (byte & 0x80) == 0 {
            return Ok((value, idx + 1));
        }
        shift += 7;
        if shift >= 70 {
            return Err(ProtocolError::VarLongTooBig);
        }
    }
    Err(ProtocolError::UnexpectedEof)
}

/// Automatically selects hardware BMI2 or fallback path to decode a VarLong from a slice.
#[inline]
pub fn decode_varlong_slice(slice: &[u8]) -> Result<(i64, usize), ProtocolError> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if has_bmi2() {
            return unsafe { decode_varlong_bmi2(slice) };
        }
    }
    decode_varlong_fallback(slice)
}

/// Reads a VarInt from a buffer without heap allocations.
/// Accelerates contiguous buffer reads via BMI2 hardware decoding.
pub fn read_varint(buf: &mut impl Buf) -> Result<i32, ProtocolError> {
    let chunk = buf.chunk();
    if !chunk.is_empty() {
        match decode_varint_slice(chunk) {
            Ok((val, consumed)) => {
                buf.advance(consumed);
                return Ok(val);
            }
            Err(ProtocolError::UnexpectedEof) => {
                // Partial slice, fall through to multi-chunk byte-by-byte
            }
            Err(e) => return Err(e),
        }
    }

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
/// Accelerates contiguous buffer reads via BMI2 hardware decoding.
pub fn read_varlong(buf: &mut impl Buf) -> Result<i64, ProtocolError> {
    let chunk = buf.chunk();
    if !chunk.is_empty() {
        match decode_varlong_slice(chunk) {
            Ok((val, consumed)) => {
                buf.advance(consumed);
                return Ok(val);
            }
            Err(ProtocolError::UnexpectedEof) => {
                // Partial slice, fall through to multi-chunk byte-by-byte
            }
            Err(e) => return Err(e),
        }
    }

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

    #[test]
    fn test_varint_bmi2_fallback_parity() {
        let values = [
            0, 1, 2, 127, 128, 255, 256, 16384, 2097151, 2097152,
            2147483647, -1, -2, -128, -2147483648,
        ];
        for &val in &values {
            let mut buf = BytesMut::new();
            write_varint(val, &mut buf);
            let slice = &buf[..];

            let (decoded_slice, len_slice) = decode_varint_slice(slice).unwrap();
            let (decoded_fallback, len_fallback) = decode_varint_fallback(slice).unwrap();

            assert_eq!(decoded_slice, val);
            assert_eq!(decoded_fallback, val);
            assert_eq!(len_slice, len_fallback);

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            if has_bmi2() {
                let (decoded_bmi2, len_bmi2) = unsafe { decode_varint_bmi2(slice) }.unwrap();
                assert_eq!(decoded_bmi2, val);
                assert_eq!(len_bmi2, len_slice);
            }
        }
    }

    #[test]
    fn test_varlong_bmi2_fallback_parity() {
        let values = [
            0i64, 1, 127, 128, 2147483647, -1, 9223372036854775807, -9223372036854775808,
            (1i64 << 35) - 1, (1i64 << 50), -123456789012345,
        ];
        for &val in &values {
            let mut buf = BytesMut::new();
            write_varlong(val, &mut buf);
            let slice = &buf[..];

            let (decoded_slice, len_slice) = decode_varlong_slice(slice).unwrap();
            let (decoded_fallback, len_fallback) = decode_varlong_fallback(slice).unwrap();

            assert_eq!(decoded_slice, val);
            assert_eq!(decoded_fallback, val);
            assert_eq!(len_slice, len_fallback);

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            if has_bmi2() {
                let (decoded_bmi2, len_bmi2) = unsafe { decode_varlong_bmi2(slice) }.unwrap();
                assert_eq!(decoded_bmi2, val);
                assert_eq!(len_bmi2, len_slice);
            }
        }
    }
}

