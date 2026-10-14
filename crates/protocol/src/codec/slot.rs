use crate::codec::nbt::{read_optional_compound, write_optional_compound, NbtTag};
use crate::error::ProtocolError;
use bytes::{Buf, BufMut};

/// Represents an item slot in Minecraft 1.8.9 inventories.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Slot {
    pub item_id: i16,
    pub item_count: i8,
    pub item_damage: i16,
    pub nbt: Option<NbtTag>,
}

impl Slot {
    pub fn new(item_id: i16, item_count: i8, item_damage: i16, nbt: Option<NbtTag>) -> Self {
        Self {
            item_id,
            item_count,
            item_damage,
            nbt,
        }
    }
}

/// Reads an inventory slot from the buffer.
/// Returns `Ok(None)` if `item_id == -1` (empty slot).
pub fn read_slot(buf: &mut impl Buf) -> Result<Option<Slot>, ProtocolError> {
    if buf.remaining() < 2 {
        return Err(ProtocolError::UnexpectedEof);
    }

    let item_id = buf.get_i16();
    if item_id <= -1 {
        return Ok(None);
    }

    if buf.remaining() < 3 {
        return Err(ProtocolError::UnexpectedEof);
    }

    let item_count = buf.get_i8();
    let item_damage = buf.get_i16();
    let nbt = read_optional_compound(buf)?;

    Ok(Some(Slot {
        item_id,
        item_count,
        item_damage,
        nbt,
    }))
}

/// Writes an inventory slot to the buffer.
/// An empty slot (`None`) is encoded as `-1i16`.
pub fn write_slot(slot: Option<&Slot>, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
    match slot {
        None => {
            buf.put_i16(-1);
            Ok(())
        }
        Some(s) => {
            buf.put_i16(s.item_id);
            buf.put_i8(s.item_count);
            buf.put_i16(s.item_damage);
            write_optional_compound(s.nbt.as_ref(), buf)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use std::collections::HashMap;

    #[test]
    fn test_slot_empty_roundtrip() {
        let mut buf = BytesMut::new();
        write_slot(None, &mut buf).unwrap();
        assert_eq!(buf.len(), 2);

        let mut reader = buf.freeze();
        let slot = read_slot(&mut reader).unwrap();
        assert_eq!(slot, None);
    }

    #[test]
    fn test_slot_with_item_and_nbt() {
        let mut nbt_map = HashMap::new();
        nbt_map.insert("Unbreakable".to_string(), NbtTag::Byte(1));
        let slot = Slot {
            item_id: 276, // Diamond Sword
            item_count: 1,
            item_damage: 0,
            nbt: Some(NbtTag::Compound(nbt_map)),
        };

        let mut buf = BytesMut::new();
        write_slot(Some(&slot), &mut buf).unwrap();

        let mut reader = buf.freeze();
        let decoded = read_slot(&mut reader).unwrap();
        assert_eq!(decoded, Some(slot));
    }
}
