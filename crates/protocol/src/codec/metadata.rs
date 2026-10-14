use crate::codec::slot::{read_slot, write_slot, Slot};
use crate::codec::string::{read_string, write_string};
use crate::error::ProtocolError;
use bytes::{Buf, BufMut};

/// Entity metadata value variants in Minecraft 1.8.9 (Protocol 47).
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    Slot(Option<Slot>),
    Position { x: i32, y: i32, z: i32 },
    Rotations { pitch: f32, yaw: f32, roll: f32 },
}

impl MetadataValue {
    pub fn type_id(&self) -> u8 {
        match self {
            MetadataValue::Byte(_) => 0,
            MetadataValue::Short(_) => 1,
            MetadataValue::Int(_) => 2,
            MetadataValue::Float(_) => 3,
            MetadataValue::String(_) => 4,
            MetadataValue::Slot(_) => 5,
            MetadataValue::Position { .. } => 6,
            MetadataValue::Rotations { .. } => 7,
        }
    }
}

/// A single entry in an entity's metadata dictionary.
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataEntry {
    pub index: u8,
    pub value: MetadataValue,
}

impl MetadataEntry {
    pub fn new(index: u8, value: MetadataValue) -> Self {
        Self { index, value }
    }
}

/// Reads entity metadata entries until the 127 (0x7F) terminator byte.
///
/// Follows Minecraft 1.8.9 MCP `DataWatcher.readWatchedListFromPacketBuffer`.
pub fn read_entity_metadata(buf: &mut impl Buf) -> Result<Vec<MetadataEntry>, ProtocolError> {
    let mut entries = Vec::new();

    loop {
        if !buf.has_remaining() {
            return Err(ProtocolError::UnexpectedEof);
        }

        let header = buf.get_u8();
        if header == 127 {
            break;
        }

        let type_id = (header & 0xE0) >> 5;
        let index = header & 0x1F;

        let value = match type_id {
            0 => {
                if !buf.has_remaining() {
                    return Err(ProtocolError::UnexpectedEof);
                }
                MetadataValue::Byte(buf.get_i8())
            }
            1 => {
                if buf.remaining() < 2 {
                    return Err(ProtocolError::UnexpectedEof);
                }
                MetadataValue::Short(buf.get_i16())
            }
            2 => {
                if buf.remaining() < 4 {
                    return Err(ProtocolError::UnexpectedEof);
                }
                MetadataValue::Int(buf.get_i32())
            }
            3 => {
                if buf.remaining() < 4 {
                    return Err(ProtocolError::UnexpectedEof);
                }
                MetadataValue::Float(buf.get_f32())
            }
            4 => MetadataValue::String(read_string(buf, 32767)?),
            5 => MetadataValue::Slot(read_slot(buf)?),
            6 => {
                if buf.remaining() < 12 {
                    return Err(ProtocolError::UnexpectedEof);
                }
                MetadataValue::Position {
                    x: buf.get_i32(),
                    y: buf.get_i32(),
                    z: buf.get_i32(),
                }
            }
            7 => {
                if buf.remaining() < 12 {
                    return Err(ProtocolError::UnexpectedEof);
                }
                MetadataValue::Rotations {
                    pitch: buf.get_f32(),
                    yaw: buf.get_f32(),
                    roll: buf.get_f32(),
                }
            }
            other => {
                return Err(ProtocolError::InvalidMetadata(format!(
                    "Unknown metadata type ID: {}",
                    other
                )))
            }
        };

        entries.push(MetadataEntry { index, value });
    }

    Ok(entries)
}

/// Writes entity metadata entries followed by the 127 terminator byte.
///
/// Follows Minecraft 1.8.9 MCP `DataWatcher.writeWatchableObjectToPacketBuffer`.
pub fn write_entity_metadata(
    entries: &[MetadataEntry],
    buf: &mut impl BufMut,
) -> Result<(), ProtocolError> {
    for entry in entries {
        let type_id = entry.value.type_id();
        let header = ((type_id << 5) | (entry.index & 0x1F)) as u8;
        buf.put_u8(header);

        match &entry.value {
            MetadataValue::Byte(v) => buf.put_i8(*v),
            MetadataValue::Short(v) => buf.put_i16(*v),
            MetadataValue::Int(v) => buf.put_i32(*v),
            MetadataValue::Float(v) => buf.put_f32(*v),
            MetadataValue::String(s) => write_string(s, buf)?,
            MetadataValue::Slot(s) => write_slot(s.as_ref(), buf)?,
            MetadataValue::Position { x, y, z } => {
                buf.put_i32(*x);
                buf.put_i32(*y);
                buf.put_i32(*z);
            }
            MetadataValue::Rotations { pitch, yaw, roll } => {
                buf.put_f32(*pitch);
                buf.put_f32(*yaw);
                buf.put_f32(*roll);
            }
        }
    }

    // Terminator
    buf.put_u8(127);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_entity_metadata_roundtrip() {
        let entries = vec![
            MetadataEntry::new(0, MetadataValue::Byte(0x20)), // Sneaking flag
            MetadataEntry::new(1, MetadataValue::Short(300)), // Air
            MetadataEntry::new(2, MetadataValue::String("CustomName".to_string())),
            MetadataEntry::new(3, MetadataValue::Byte(1)), // Always show name tag
            MetadataEntry::new(
                10,
                MetadataValue::Position {
                    x: 100,
                    y: 64,
                    z: -200,
                },
            ),
            MetadataEntry::new(
                11,
                MetadataValue::Rotations {
                    pitch: 10.5,
                    yaw: 20.25,
                    roll: 0.0,
                },
            ),
            MetadataEntry::new(
                8,
                MetadataValue::Slot(Some(Slot::new(276, 1, 0, None))),
            ),
        ];

        let mut buf = BytesMut::new();
        write_entity_metadata(&entries, &mut buf).unwrap();

        let mut reader = buf.freeze();
        let decoded = read_entity_metadata(&mut reader).unwrap();
        assert_eq!(decoded, entries);
    }
}
