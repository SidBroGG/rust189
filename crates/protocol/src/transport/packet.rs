use bytes::Bytes;

/// A raw protocol packet containing its packet ID and raw payload bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct RawPacket {
    pub id: i32,
    pub payload: Bytes,
}

impl RawPacket {
    pub fn new(id: i32, payload: Bytes) -> Self {
        Self { id, payload }
    }
}
