use crate::error::ProtocolError;
use bytes::{Buf, BufMut, BytesMut};
use std::collections::HashMap;
use std::io::{Read, Write};

/// An NBT (Named Binary Tag) value representing data in Minecraft 1.8.9.
#[derive(Debug, Clone, PartialEq)]
pub enum NbtTag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List {
        element_type: u8,
        elements: Vec<NbtTag>,
    },
    Compound(HashMap<String, NbtTag>),
    IntArray(Vec<i32>),
}

impl NbtTag {
    pub fn tag_id(&self) -> u8 {
        match self {
            NbtTag::End => 0,
            NbtTag::Byte(_) => 1,
            NbtTag::Short(_) => 2,
            NbtTag::Int(_) => 3,
            NbtTag::Long(_) => 4,
            NbtTag::Float(_) => 5,
            NbtTag::Double(_) => 6,
            NbtTag::ByteArray(_) => 7,
            NbtTag::String(_) => 8,
            NbtTag::List { .. } => 9,
            NbtTag::Compound(_) => 10,
            NbtTag::IntArray(_) => 11,
        }
    }

    pub fn as_compound(&self) -> Option<&HashMap<String, NbtTag>> {
        if let NbtTag::Compound(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_compound_mut(&mut self) -> Option<&mut HashMap<String, NbtTag>> {
        if let NbtTag::Compound(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let NbtTag::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            NbtTag::Byte(v) => Some(*v as i32),
            NbtTag::Short(v) => Some(*v as i32),
            NbtTag::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            NbtTag::Byte(v) => Some(*v as i64),
            NbtTag::Short(v) => Some(*v as i64),
            NbtTag::Int(v) => Some(*v as i64),
            NbtTag::Long(v) => Some(*v),
            _ => None,
        }
    }

    pub fn get_compound(&self, key: &str) -> Option<&NbtTag> {
        self.as_compound().and_then(|m| m.get(key)).filter(|t| matches!(t, NbtTag::Compound(_)))
    }

    pub fn get_list(&self, key: &str) -> Option<&[NbtTag]> {
        if let Some(NbtTag::List { elements, .. }) = self.as_compound().and_then(|m| m.get(key)) {
            Some(elements.as_slice())
        } else {
            None
        }
    }

    pub fn get_short(&self, key: &str) -> Option<i16> {
        match self.as_compound().and_then(|m| m.get(key)) {
            Some(NbtTag::Short(v)) => Some(*v),
            Some(NbtTag::Byte(v)) => Some(*v as i16),
            Some(NbtTag::Int(v)) => Some(*v as i16),
            _ => None,
        }
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.as_compound().and_then(|m| m.get(key)).and_then(|t| t.as_string())
    }

    pub fn insert(&mut self, key: impl Into<String>, tag: NbtTag) {
        if let NbtTag::Compound(map) = self {
            map.insert(key.into(), tag);
        }
    }

    pub fn set_string(&mut self, key: impl Into<String>, val: impl Into<String>) {
        self.insert(key, NbtTag::String(val.into()));
    }

    pub fn set_list(&mut self, key: impl Into<String>, list: Vec<NbtTag>) {
        let element_type = list.first().map(|t| t.tag_id()).unwrap_or(0);
        self.insert(key, NbtTag::List { element_type, elements: list });
    }
}

const MAX_NBT_DEPTH: usize = 512;

/// Reads an uncompressed root compound tag from the buffer.
pub fn read_named_compound(buf: &mut impl Buf) -> Result<(String, NbtTag), ProtocolError> {
    if !buf.has_remaining() {
        return Err(ProtocolError::UnexpectedEof);
    }

    let type_id = buf.get_u8();
    if type_id == 0 {
        return Ok((String::new(), NbtTag::End));
    }
    if type_id != 10 {
        return Err(ProtocolError::InvalidNbt(format!(
            "Root NBT tag must be Compound (10), got {}",
            type_id
        )));
    }

    let name = read_nbt_string(buf)?;
    let compound = read_compound_payload(buf, 0)?;
    Ok((name, compound))
}

/// Reads an optional NBT tag as used in Minecraft 1.8.9 slots:
/// If the first byte is 0 (TAG_End), returns `Ok(None)`.
/// Otherwise reads a root compound and returns `Ok(Some(tag))`.
pub fn read_optional_compound(buf: &mut impl Buf) -> Result<Option<NbtTag>, ProtocolError> {
    if !buf.has_remaining() {
        return Err(ProtocolError::UnexpectedEof);
    }

    // Inspect first byte
    let peek_byte = buf.chunk()[0];
    if peek_byte == 0 {
        buf.advance(1);
        Ok(None)
    } else {
        let (_name, tag) = read_named_compound(buf)?;
        Ok(Some(tag))
    }
}

/// Writes an optional NBT root compound.
/// If `None`, writes a single byte 0.
pub fn write_optional_compound(tag: Option<&NbtTag>, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
    match tag {
        None => {
            buf.put_u8(0);
            Ok(())
        }
        Some(t) => write_named_compound("", t, buf),
    }
}

/// Writes a named root compound tag to the buffer.
pub fn write_named_compound(name: &str, tag: &NbtTag, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
    match tag {
        NbtTag::Compound(map) => {
            buf.put_u8(10); // TAG_Compound
            write_nbt_string(name, buf)?;
            write_compound_payload(map, buf)?;
            Ok(())
        }
        NbtTag::End => {
            buf.put_u8(0);
            Ok(())
        }
        other => Err(ProtocolError::InvalidNbt(format!(
            "Cannot serialize non-compound root tag: {:?}",
            other.tag_id()
        ))),
    }
}

fn read_nbt_string(buf: &mut impl Buf) -> Result<String, ProtocolError> {
    if buf.remaining() < 2 {
        return Err(ProtocolError::UnexpectedEof);
    }
    let len = buf.get_u16() as usize;
    if buf.remaining() < len {
        return Err(ProtocolError::UnexpectedEof);
    }
    let mut bytes = vec![0u8; len];
    buf.copy_to_slice(&mut bytes);
    Ok(String::from_utf8(bytes)?)
}

fn write_nbt_string(s: &str, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
    let bytes = s.as_bytes();
    if bytes.len() > u16::MAX as usize {
        return Err(ProtocolError::InvalidNbt("NBT string length exceeds u16::MAX".into()));
    }
    buf.put_u16(bytes.len() as u16);
    buf.put_slice(bytes);
    Ok(())
}

fn read_tag_payload(type_id: u8, buf: &mut impl Buf, depth: usize) -> Result<NbtTag, ProtocolError> {
    if depth > MAX_NBT_DEPTH {
        return Err(ProtocolError::InvalidNbt("Exceeded max NBT recursion depth".into()));
    }

    match type_id {
        0 => Ok(NbtTag::End),
        1 => {
            if buf.remaining() < 1 {
                return Err(ProtocolError::UnexpectedEof);
            }
            Ok(NbtTag::Byte(buf.get_i8()))
        }
        2 => {
            if buf.remaining() < 2 {
                return Err(ProtocolError::UnexpectedEof);
            }
            Ok(NbtTag::Short(buf.get_i16()))
        }
        3 => {
            if buf.remaining() < 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            Ok(NbtTag::Int(buf.get_i32()))
        }
        4 => {
            if buf.remaining() < 8 {
                return Err(ProtocolError::UnexpectedEof);
            }
            Ok(NbtTag::Long(buf.get_i64()))
        }
        5 => {
            if buf.remaining() < 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            Ok(NbtTag::Float(buf.get_f32()))
        }
        6 => {
            if buf.remaining() < 8 {
                return Err(ProtocolError::UnexpectedEof);
            }
            Ok(NbtTag::Double(buf.get_f64()))
        }
        7 => {
            if buf.remaining() < 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let len = buf.get_i32();
            if len < 0 {
                return Err(ProtocolError::InvalidNbt("Negative ByteArray length".into()));
            }
            let len = len as usize;
            if buf.remaining() < len {
                return Err(ProtocolError::UnexpectedEof);
            }
            let mut data = vec![0u8; len];
            buf.copy_to_slice(&mut data);
            Ok(NbtTag::ByteArray(data))
        }
        8 => {
            let s = read_nbt_string(buf)?;
            Ok(NbtTag::String(s))
        }
        9 => {
            if buf.remaining() < 5 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let element_type = buf.get_u8();
            let len = buf.get_i32();
            if len < 0 {
                return Err(ProtocolError::InvalidNbt("Negative List length".into()));
            }
            let len = len as usize;
            let mut elements = Vec::with_capacity(len);
            for _ in 0..len {
                elements.push(read_tag_payload(element_type, buf, depth + 1)?);
            }
            Ok(NbtTag::List { element_type, elements })
        }
        10 => read_compound_payload(buf, depth + 1),
        11 => {
            if buf.remaining() < 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let len = buf.get_i32();
            if len < 0 {
                return Err(ProtocolError::InvalidNbt("Negative IntArray length".into()));
            }
            let len = len as usize;
            if buf.remaining() < len * 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let mut data = Vec::with_capacity(len);
            for _ in 0..len {
                data.push(buf.get_i32());
            }
            Ok(NbtTag::IntArray(data))
        }
        other => Err(ProtocolError::InvalidNbt(format!("Unknown NBT tag ID: {}", other))),
    }
}

fn read_compound_payload(buf: &mut impl Buf, depth: usize) -> Result<NbtTag, ProtocolError> {
    let mut map = HashMap::new();
    loop {
        if !buf.has_remaining() {
            return Err(ProtocolError::UnexpectedEof);
        }
        let tag_id = buf.get_u8();
        if tag_id == 0 {
            break; // TAG_End
        }
        let name = read_nbt_string(buf)?;
        let tag = read_tag_payload(tag_id, buf, depth)?;
        map.insert(name, tag);
    }
    Ok(NbtTag::Compound(map))
}

fn write_tag_payload(tag: &NbtTag, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
    match tag {
        NbtTag::End => Ok(()),
        NbtTag::Byte(v) => {
            buf.put_i8(*v);
            Ok(())
        }
        NbtTag::Short(v) => {
            buf.put_i16(*v);
            Ok(())
        }
        NbtTag::Int(v) => {
            buf.put_i32(*v);
            Ok(())
        }
        NbtTag::Long(v) => {
            buf.put_i64(*v);
            Ok(())
        }
        NbtTag::Float(v) => {
            buf.put_f32(*v);
            Ok(())
        }
        NbtTag::Double(v) => {
            buf.put_f64(*v);
            Ok(())
        }
        NbtTag::ByteArray(v) => {
            buf.put_i32(v.len() as i32);
            buf.put_slice(v);
            Ok(())
        }
        NbtTag::String(v) => write_nbt_string(v, buf),
        NbtTag::List { element_type, elements } => {
            buf.put_u8(*element_type);
            buf.put_i32(elements.len() as i32);
            for el in elements {
                write_tag_payload(el, buf)?;
            }
            Ok(())
        }
        NbtTag::Compound(map) => write_compound_payload(map, buf),
        NbtTag::IntArray(v) => {
            buf.put_i32(v.len() as i32);
            for &num in v {
                buf.put_i32(num);
            }
            Ok(())
        }
    }
}

fn write_compound_payload(map: &HashMap<String, NbtTag>, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
    for (k, v) in map {
        buf.put_u8(v.tag_id());
        write_nbt_string(k, buf)?;
        write_tag_payload(v, buf)?;
    }
    buf.put_u8(0); // TAG_End
    Ok(())
}

/// Reads Zlib-compressed NBT data from a reader using libdeflate.
pub fn read_zlib(mut reader: impl Read) -> Result<(String, NbtTag), ProtocolError> {
    let mut compressed = Vec::new();
    reader.read_to_end(&mut compressed)?;
    let decompressed = crate::transport::compression::decompress_zlib_dynamic(&compressed)?;
    let mut slice = &decompressed[..];
    read_named_compound(&mut slice)
}

/// Writes Zlib-compressed NBT data to a writer using libdeflate.
pub fn write_zlib(mut writer: impl Write, name: &str, tag: &NbtTag) -> Result<(), ProtocolError> {
    let mut buf = BytesMut::new();
    write_named_compound(name, tag, &mut buf)?;
    let compressed = crate::transport::compression::compress_zlib(&buf)?;
    writer.write_all(&compressed)?;
    Ok(())
}

/// Reads Gzip-compressed NBT data from a reader using libdeflate.
pub fn read_gzip(mut reader: impl Read) -> Result<(String, NbtTag), ProtocolError> {
    let mut compressed = Vec::new();
    reader.read_to_end(&mut compressed)?;
    let decompressed = crate::transport::compression::decompress_gzip_dynamic(&compressed)?;
    let mut slice = &decompressed[..];
    read_named_compound(&mut slice)
}

/// Writes Gzip-compressed NBT data to a writer using libdeflate.
pub fn write_gzip(mut writer: impl Write, name: &str, tag: &NbtTag) -> Result<(), ProtocolError> {
    let mut buf = BytesMut::new();
    write_named_compound(name, tag, &mut buf)?;
    let compressed = crate::transport::compression::compress_gzip(&buf)?;
    writer.write_all(&compressed)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nbt_roundtrip() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), NbtTag::String("Alex".to_string()));
        map.insert("health".to_string(), NbtTag::Float(20.0));
        map.insert("level".to_string(), NbtTag::Int(42));
        map.insert("is_pro".to_string(), NbtTag::Byte(1));
        map.insert("score".to_string(), NbtTag::Long(99999999999));
        map.insert("byte_data".to_string(), NbtTag::ByteArray(vec![1, 2, 3, 4, 5]));
        map.insert("int_array".to_string(), NbtTag::IntArray(vec![100, 200, 300]));

        let mut sub = HashMap::new();
        sub.insert("x".to_string(), NbtTag::Double(12.34));
        sub.insert("y".to_string(), NbtTag::Double(56.78));
        map.insert("pos".to_string(), NbtTag::Compound(sub));

        let list = vec![
            NbtTag::String("item1".to_string()),
            NbtTag::String("item2".to_string()),
        ];
        map.insert(
            "inventory".to_string(),
            NbtTag::List {
                element_type: 8,
                elements: list,
            },
        );

        let root = NbtTag::Compound(map);
        let mut buf = BytesMut::new();
        write_named_compound("Player", &root, &mut buf).unwrap();

        let mut reader = buf.freeze();
        let (name, decoded) = read_named_compound(&mut reader).unwrap();
        assert_eq!(name, "Player");
        assert_eq!(decoded, root);
    }

    #[test]
    fn test_nbt_optional_compound() {
        let mut buf = BytesMut::new();
        write_optional_compound(None, &mut buf).unwrap();
        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0], 0);

        let mut reader = buf.freeze();
        let decoded = read_optional_compound(&mut reader).unwrap();
        assert_eq!(decoded, None);

        let mut map = HashMap::new();
        map.insert("Damage".to_string(), NbtTag::Short(15));
        let tag = NbtTag::Compound(map);

        let mut buf2 = BytesMut::new();
        write_optional_compound(Some(&tag), &mut buf2).unwrap();
        let mut reader2 = buf2.freeze();
        let decoded2 = read_optional_compound(&mut reader2).unwrap();
        assert_eq!(decoded2, Some(tag));
    }

    #[test]
    fn test_nbt_zlib_and_gzip_compression() {
        let mut map = HashMap::new();
        map.insert("server".to_string(), NbtTag::String("ServerName".to_string()));
        let root = NbtTag::Compound(map);

        // Zlib
        let mut compressed_zlib = Vec::new();
        write_zlib(&mut compressed_zlib, "ServerData", &root).unwrap();
        let (name, decoded) = read_zlib(&compressed_zlib[..]).unwrap();
        assert_eq!(name, "ServerData");
        assert_eq!(decoded, root);

        // Gzip
        let mut compressed_gzip = Vec::new();
        write_gzip(&mut compressed_gzip, "ServerData", &root).unwrap();
        let (name2, decoded2) = read_gzip(&compressed_gzip[..]).unwrap();
        assert_eq!(name2, "ServerData");
        assert_eq!(decoded2, root);
    }
}
