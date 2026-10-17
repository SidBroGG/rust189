use crate::error::ProtocolError;
use bytes::{Buf, BufMut, BytesMut};

/// RakNet 16-byte offline magic sequence.
pub const RAKNET_MAGIC: [u8; 16] = [
    0x00, 0xFF, 0xFF, 0x00, 0xFE, 0xFE, 0xFE, 0xFE,
    0xFD, 0xFD, 0xFD, 0xFD, 0x12, 0x34, 0x56, 0x78,
];

/// RakNet message and packet IDs.
pub mod packet_ids {
    pub const CONNECTED_PING: u8 = 0x00;
    pub const UNCONNECTED_PING: u8 = 0x01;
    pub const CONNECTED_PONG: u8 = 0x03;
    pub const OPEN_CONNECTION_REQUEST_1: u8 = 0x05;
    pub const OPEN_CONNECTION_REPLY_1: u8 = 0x06;
    pub const OPEN_CONNECTION_REQUEST_2: u8 = 0x07;
    pub const OPEN_CONNECTION_REPLY_2: u8 = 0x08;
    pub const CONNECTION_REQUEST: u8 = 0x09;
    pub const CONNECTION_REQUEST_ACCEPTED: u8 = 0x10;
    pub const DISCONNECT: u8 = 0x15;
    pub const UNCONNECTED_PONG: u8 = 0x1C;
    pub const ACK: u8 = 0xC0;
    pub const NACK: u8 = 0xA0;
}

/// RakNet frame reliability types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reliability {
    Unreliable = 0,
    UnreliableSequenced = 1,
    Reliable = 2,
    ReliableOrdered = 3,
    ReliableSequenced = 4,
    UnreliableWithAckReceipt = 5,
    ReliableWithAckReceipt = 6,
    ReliableOrderedWithAckReceipt = 7,
}

impl Reliability {
    pub fn from_u8(v: u8) -> Result<Self, ProtocolError> {
        match v {
            0 => Ok(Self::Unreliable),
            1 => Ok(Self::UnreliableSequenced),
            2 => Ok(Self::Reliable),
            3 => Ok(Self::ReliableOrdered),
            4 => Ok(Self::ReliableSequenced),
            5 => Ok(Self::UnreliableWithAckReceipt),
            6 => Ok(Self::ReliableWithAckReceipt),
            7 => Ok(Self::ReliableOrderedWithAckReceipt),
            other => Err(ProtocolError::InvalidNbt(format!("Unknown RakNet reliability: {}", other))),
        }
    }

    #[inline]
    pub fn is_reliable(&self) -> bool {
        matches!(
            self,
            Self::Reliable
                | Self::ReliableOrdered
                | Self::ReliableSequenced
                | Self::ReliableWithAckReceipt
                | Self::ReliableOrderedWithAckReceipt
        )
    }

    #[inline]
    pub fn is_ordered(&self) -> bool {
        matches!(
            self,
            Self::ReliableOrdered | Self::ReliableOrderedWithAckReceipt
        )
    }

    #[inline]
    pub fn is_sequenced(&self) -> bool {
        matches!(self, Self::UnreliableSequenced | Self::ReliableSequenced)
    }
}

/// Information for reassembling fragmented compound frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentInfo {
    pub compound_size: u32,
    pub compound_id: u16,
    pub fragment_index: u32,
}

/// An individual RakNet frame within a datagram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub reliability: Reliability,
    pub reliable_index: Option<u32>,
    pub sequence_index: Option<u32>,
    pub order_index: Option<u32>,
    pub order_channel: Option<u8>,
    pub fragment: Option<FragmentInfo>,
    pub body: Vec<u8>,
}

impl Frame {
    pub fn new_reliable_ordered(body: Vec<u8>, reliable_idx: u32, order_idx: u32, channel: u8) -> Self {
        Self {
            reliability: Reliability::ReliableOrdered,
            reliable_index: Some(reliable_idx),
            sequence_index: None,
            order_index: Some(order_idx),
            order_channel: Some(channel),
            fragment: None,
            body,
        }
    }

    pub fn new_unreliable(body: Vec<u8>) -> Self {
        Self {
            reliability: Reliability::Unreliable,
            reliable_index: None,
            sequence_index: None,
            order_index: None,
            order_channel: None,
            fragment: None,
            body,
        }
    }

    /// Serializes the frame into a byte buffer.
    pub fn encode(&self, buf: &mut BytesMut) {
        let is_fragmented = self.fragment.is_some();
        let flags = ((self.reliability as u8) << 5) | (if is_fragmented { 0x10 } else { 0x00 });
        buf.put_u8(flags);

        let bit_len = (self.body.len() as u16) * 8;
        buf.put_u16(bit_len);

        if self.reliability.is_reliable() {
            let idx = self.reliable_index.unwrap_or(0);
            buf.put_u8((idx & 0xFF) as u8);
            buf.put_u8(((idx >> 8) & 0xFF) as u8);
            buf.put_u8(((idx >> 16) & 0xFF) as u8);
        }

        if self.reliability.is_sequenced() {
            let idx = self.sequence_index.unwrap_or(0);
            buf.put_u8((idx & 0xFF) as u8);
            buf.put_u8(((idx >> 8) & 0xFF) as u8);
            buf.put_u8(((idx >> 16) & 0xFF) as u8);
        }

        if self.reliability.is_ordered() {
            let idx = self.order_index.unwrap_or(0);
            buf.put_u8((idx & 0xFF) as u8);
            buf.put_u8(((idx >> 8) & 0xFF) as u8);
            buf.put_u8(((idx >> 16) & 0xFF) as u8);
            buf.put_u8(self.order_channel.unwrap_or(0));
        }

        if let Some(ref frag) = self.fragment {
            buf.put_u32(frag.compound_size);
            buf.put_u16(frag.compound_id);
            buf.put_u32(frag.fragment_index);
        }

        buf.put_slice(&self.body);
    }

    /// Deserializes a frame from a reader.
    pub fn decode(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        if buf.remaining() < 3 {
            return Err(ProtocolError::UnexpectedEof);
        }

        let flags = buf.get_u8();
        let reliability = Reliability::from_u8(flags >> 5)?;
        let is_fragmented = (flags & 0x10) != 0;

        let bit_len = buf.get_u16();
        let byte_len = ((bit_len + 7) / 8) as usize;

        let mut reliable_index = None;
        if reliability.is_reliable() {
            if buf.remaining() < 3 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let b0 = buf.get_u8() as u32;
            let b1 = buf.get_u8() as u32;
            let b2 = buf.get_u8() as u32;
            reliable_index = Some(b0 | (b1 << 8) | (b2 << 16));
        }

        let mut sequence_index = None;
        if reliability.is_sequenced() {
            if buf.remaining() < 3 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let b0 = buf.get_u8() as u32;
            let b1 = buf.get_u8() as u32;
            let b2 = buf.get_u8() as u32;
            sequence_index = Some(b0 | (b1 << 8) | (b2 << 16));
        }

        let mut order_index = None;
        let mut order_channel = None;
        if reliability.is_ordered() {
            if buf.remaining() < 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let b0 = buf.get_u8() as u32;
            let b1 = buf.get_u8() as u32;
            let b2 = buf.get_u8() as u32;
            order_index = Some(b0 | (b1 << 8) | (b2 << 16));
            order_channel = Some(buf.get_u8());
        }

        let mut fragment = None;
        if is_fragmented {
            if buf.remaining() < 10 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let compound_size = buf.get_u32();
            let compound_id = buf.get_u16();
            let fragment_index = buf.get_u32();
            fragment = Some(FragmentInfo {
                compound_size,
                compound_id,
                fragment_index,
            });
        }

        if buf.remaining() < byte_len {
            return Err(ProtocolError::UnexpectedEof);
        }

        let mut body = vec![0u8; byte_len];
        buf.copy_to_slice(&mut body);

        Ok(Self {
            reliability,
            reliable_index,
            sequence_index,
            order_index,
            order_channel,
            fragment,
            body,
        })
    }
}
