use crate::codec::metadata::{read_entity_metadata, write_entity_metadata, MetadataEntry};
use crate::codec::nbt::{
    read_named_compound, read_optional_compound, write_named_compound, write_optional_compound,
    NbtTag,
};
use crate::codec::position::Position;
use crate::codec::slot::{read_slot, write_slot, Slot};
use crate::codec::string::{read_string, write_string};
use crate::codec::uuid::{read_uuid, write_uuid};
use crate::codec::varint::{read_varint, read_varlong, write_varint, write_varlong};
use crate::error::ProtocolError;
use bytes::{Buf, BufMut};
use uuid::Uuid;

/// Extension trait for `bytes::Buf` providing Minecraft 1.8.9 reading primitives.
pub trait BufExt: Buf + Sized {
    #[inline]
    fn read_varint(&mut self) -> Result<i32, ProtocolError> {
        read_varint(self)
    }

    #[inline]
    fn read_varlong(&mut self) -> Result<i64, ProtocolError> {
        read_varlong(self)
    }

    #[inline]
    fn read_string(&mut self, max_chars: usize) -> Result<String, ProtocolError> {
        read_string(self, max_chars)
    }

    #[inline]
    fn read_uuid(&mut self) -> Result<Uuid, ProtocolError> {
        read_uuid(self)
    }

    #[inline]
    fn read_position(&mut self) -> Result<Position, ProtocolError> {
        Position::read(self)
    }

    #[inline]
    fn read_slot(&mut self) -> Result<Option<Slot>, ProtocolError> {
        read_slot(self)
    }

    #[inline]
    fn read_metadata(&mut self) -> Result<Vec<MetadataEntry>, ProtocolError> {
        read_entity_metadata(self)
    }

    #[inline]
    fn read_nbt_named(&mut self) -> Result<(String, NbtTag), ProtocolError> {
        read_named_compound(self)
    }

    #[inline]
    fn read_optional_nbt(&mut self) -> Result<Option<NbtTag>, ProtocolError> {
        read_optional_compound(self)
    }
}

impl<T: Buf> BufExt for T {}

/// Extension trait for `bytes::BufMut` providing Minecraft 1.8.9 writing primitives.
pub trait BufMutExt: BufMut + Sized {
    #[inline]
    fn write_varint(&mut self, val: i32) {
        write_varint(val, self);
    }

    #[inline]
    fn write_varlong(&mut self, val: i64) {
        write_varlong(val, self);
    }

    #[inline]
    fn write_string(&mut self, s: &str) -> Result<(), ProtocolError> {
        write_string(s, self)
    }

    #[inline]
    fn write_uuid(&mut self, uuid: &Uuid) {
        write_uuid(uuid, self);
    }

    #[inline]
    fn write_position(&mut self, pos: &Position) {
        pos.write(self);
    }

    #[inline]
    fn write_slot(&mut self, slot: Option<&Slot>) -> Result<(), ProtocolError> {
        write_slot(slot, self)
    }

    #[inline]
    fn write_metadata(&mut self, entries: &[MetadataEntry]) -> Result<(), ProtocolError> {
        write_entity_metadata(entries, self)
    }

    #[inline]
    fn write_nbt_named(&mut self, name: &str, tag: &NbtTag) -> Result<(), ProtocolError> {
        write_named_compound(name, tag, self)
    }

    #[inline]
    fn write_optional_nbt(&mut self, tag: Option<&NbtTag>) -> Result<(), ProtocolError> {
        write_optional_compound(tag, self)
    }
}

impl<T: BufMut> BufMutExt for T {}
