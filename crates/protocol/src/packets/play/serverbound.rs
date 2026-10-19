use crate::codec::buf_ext::{BufExt, BufMutExt};
use crate::codec::position::Position;
use crate::codec::slot::{read_slot, write_slot, Slot};
use crate::error::ProtocolError;
use bytes::{Buf, Bytes, BytesMut};

/// Serverbound KeepAlive packet (0x00).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundKeepAlivePacket {
    pub keep_alive_id: i32,
}

impl ServerboundKeepAlivePacket {
    pub const PACKET_ID: i32 = 0x00;

    pub fn new(keep_alive_id: i32) -> Self {
        Self { keep_alive_id }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.keep_alive_id);
        Ok(())
    }
}

/// Serverbound Chat Message packet (0x01).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatPacket {
    pub message: String,
}

impl ServerboundChatPacket {
    pub const PACKET_ID: i32 = 0x01;

    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_string(&self.message)?;
        Ok(())
    }
}

/// Serverbound Player Movement (Flying / OnGround) packet (0x03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundPlayerMovementPacket {
    pub on_ground: bool,
}

impl ServerboundPlayerMovementPacket {
    pub const PACKET_ID: i32 = 0x03;

    pub fn new(on_ground: bool) -> Self {
        Self { on_ground }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[if self.on_ground { 1 } else { 0 }]);
        Ok(())
    }
}

/// Serverbound Player Position packet (0x04).
#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundPlayerPositionPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}

impl ServerboundPlayerPositionPacket {
    pub const PACKET_ID: i32 = 0x04;

    pub fn new(x: f64, y: f64, z: f64, on_ground: bool) -> Self {
        Self { x, y, z, on_ground }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf.extend_from_slice(&self.z.to_be_bytes());
        buf.extend_from_slice(&[if self.on_ground { 1 } else { 0 }]);
        Ok(())
    }
}

/// Serverbound Player Look packet (0x05).
#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundPlayerLookPacket {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl ServerboundPlayerLookPacket {
    pub const PACKET_ID: i32 = 0x05;

    pub fn new(yaw: f32, pitch: f32, on_ground: bool) -> Self {
        Self { yaw, pitch, on_ground }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.yaw.to_be_bytes());
        buf.extend_from_slice(&self.pitch.to_be_bytes());
        buf.extend_from_slice(&[if self.on_ground { 1 } else { 0 }]);
        Ok(())
    }
}

/// Serverbound Player Position and Look packet (0x06).
#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundPlayerPositionAndLookPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl ServerboundPlayerPositionAndLookPacket {
    pub const PACKET_ID: i32 = 0x06;

    pub fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32, on_ground: bool) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            pitch,
            on_ground,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf.extend_from_slice(&self.z.to_be_bytes());
        buf.extend_from_slice(&self.yaw.to_be_bytes());
        buf.extend_from_slice(&self.pitch.to_be_bytes());
        buf.extend_from_slice(&[if self.on_ground { 1 } else { 0 }]);
        Ok(())
    }
}

/// Serverbound Client Settings packet (0x15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundClientSettingsPacket {
    pub locale: String,
    pub view_distance: i8,
    pub chat_flags: i8,
    pub chat_colors: bool,
    pub skin_parts: u8,
}

impl ServerboundClientSettingsPacket {
    pub const PACKET_ID: i32 = 0x15;

    pub fn new(
        locale: impl Into<String>,
        view_distance: i8,
        chat_flags: i8,
        chat_colors: bool,
        skin_parts: u8,
    ) -> Self {
        Self {
            locale: locale.into(),
            view_distance,
            chat_flags,
            chat_colors,
            skin_parts,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_string(&self.locale)?;
        buf.extend_from_slice(&[
            self.view_distance as u8,
            self.chat_flags as u8,
            if self.chat_colors { 1 } else { 0 },
            self.skin_parts,
        ]);
        Ok(())
    }
}

/// Actions for Serverbound EntityAction packet (0x0B).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAction {
    StartSneaking = 0,
    StopSneaking = 1,
    LeaveBed = 2,
    StartSprinting = 3,
    StopSprinting = 4,
    RidingJump = 5,
    OpenInventory = 6,
}

/// Serverbound EntityAction packet (0x0B).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundEntityActionPacket {
    pub entity_id: i32,
    pub action: EntityAction,
    pub jump_boost: i32,
}

impl ServerboundEntityActionPacket {
    pub const PACKET_ID: i32 = 0x0B;

    pub fn new(entity_id: i32, action: EntityAction, jump_boost: i32) -> Self {
        Self {
            entity_id,
            action,
            jump_boost,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.write_varint(self.action as i32);
        buf.write_varint(self.jump_boost);
        Ok(())
    }
}

/// Serverbound Player Abilities packet (0x13).
#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundPlayerAbilitiesPacket {
    pub invulnerable: bool,
    pub is_flying: bool,
    pub allow_flying: bool,
    pub creative_mode: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
}

impl ServerboundPlayerAbilitiesPacket {
    pub const PACKET_ID: i32 = 0x13;

    pub fn new(is_flying: bool) -> Self {
        Self {
            invulnerable: false,
            is_flying,
            allow_flying: false,
            creative_mode: false,
            fly_speed: 0.05,
            walk_speed: 0.1,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        let mut flags = 0u8;
        if self.invulnerable {
            flags |= 0x01;
        }
        if self.is_flying {
            flags |= 0x02;
        }
        if self.allow_flying {
            flags |= 0x04;
        }
        if self.creative_mode {
            flags |= 0x08;
        }
        buf.extend_from_slice(&[flags]);
        buf.extend_from_slice(&self.fly_speed.to_be_bytes());
        buf.extend_from_slice(&self.walk_speed.to_be_bytes());
        Ok(())
    }
}

/// Serverbound ConfirmTransaction packet (0x0F).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundConfirmTransactionPacket {
    pub window_id: u8,
    pub action_number: i16,
    pub accepted: bool,
}

impl ServerboundConfirmTransactionPacket {
    pub const PACKET_ID: i32 = 0x0F;

    pub fn new(window_id: u8, action_number: i16, accepted: bool) -> Self {
        Self {
            window_id,
            action_number,
            accepted,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[self.window_id]);
        buf.extend_from_slice(&self.action_number.to_be_bytes());
        buf.extend_from_slice(&[if self.accepted { 1 } else { 0 }]);
        Ok(())
    }

    pub fn decode(buf: &mut bytes::Bytes) -> Result<Self, ProtocolError> {
        use bytes::Buf;
        if buf.remaining() < 4 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 4,
                available: buf.remaining(),
            });
        }
        let window_id = buf.get_u8();
        let action_number = buf.get_i16();
        let accepted = buf.get_u8() != 0;
        Ok(Self {
            window_id,
            action_number,
            accepted,
        })
    }
}

/// Actions for Serverbound Player Digging packet (0x07).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiggingAction {
    StartDestroyBlock = 0,
    AbortDestroyBlock = 1,
    StopDestroyBlock = 2,
    DropAllItems = 3,
    DropItem = 4,
    ReleaseUseItem = 5,
}

impl DiggingAction {
    pub fn from_i32(val: i32) -> Result<Self, ProtocolError> {
        match val {
            0 => Ok(Self::StartDestroyBlock),
            1 => Ok(Self::AbortDestroyBlock),
            2 => Ok(Self::StopDestroyBlock),
            3 => Ok(Self::DropAllItems),
            4 => Ok(Self::DropItem),
            5 => Ok(Self::ReleaseUseItem),
            _ => Err(ProtocolError::InvalidEnumValue {
                enum_name: "DiggingAction",
                value: val,
            }),
        }
    }
}

/// Block faces for digging and block placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockFace {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

impl BlockFace {
    pub fn from_u8(val: u8) -> Result<Self, ProtocolError> {
        match val {
            0 => Ok(Self::Down),
            1 => Ok(Self::Up),
            2 => Ok(Self::North),
            3 => Ok(Self::South),
            4 => Ok(Self::West),
            5 => Ok(Self::East),
            _ => Err(ProtocolError::InvalidEnumValue {
                enum_name: "BlockFace",
                value: val as i32,
            }),
        }
    }

    #[inline]
    pub fn normal(&self) -> (i32, i32, i32) {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }
}

/// Serverbound Player Digging packet (0x07).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundPlayerDiggingPacket {
    pub status: DiggingAction,
    pub position: Position,
    pub facing: BlockFace,
}

impl ServerboundPlayerDiggingPacket {
    pub const PACKET_ID: i32 = 0x07;

    pub fn new(status: DiggingAction, position: Position, facing: BlockFace) -> Self {
        Self {
            status,
            position,
            facing,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.status as i32);
        self.position.write(buf);
        buf.extend_from_slice(&[self.facing as u8]);
        Ok(())
    }

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let status = DiggingAction::from_i32(buf.read_varint()?)?;
        let position = Position::read(buf)?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let facing = BlockFace::from_u8(buf.get_u8())?;
        Ok(Self {
            status,
            position,
            facing,
        })
    }
}

/// Serverbound Player Block Placement packet (0x08).
#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundPlayerBlockPlacementPacket {
    pub position: Position,
    pub placed_block_direction: u8,
    pub stack: Option<Slot>,
    pub facing_x: u8,
    pub facing_y: u8,
    pub facing_z: u8,
}

impl ServerboundPlayerBlockPlacementPacket {
    pub const PACKET_ID: i32 = 0x08;

    pub fn new(
        position: Position,
        placed_block_direction: u8,
        stack: Option<Slot>,
        facing_x: u8,
        facing_y: u8,
        facing_z: u8,
    ) -> Self {
        Self {
            position,
            placed_block_direction,
            stack,
            facing_x,
            facing_y,
            facing_z,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        self.position.write(buf);
        buf.extend_from_slice(&[self.placed_block_direction]);
        write_slot(self.stack.as_ref(), buf)?;
        buf.extend_from_slice(&[self.facing_x, self.facing_y, self.facing_z]);
        Ok(())
    }

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let position = Position::read(buf)?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let placed_block_direction = buf.get_u8();
        let stack = read_slot(buf)?;
        if buf.remaining() < 3 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 3,
                available: buf.remaining(),
            });
        }
        let facing_x = buf.get_u8();
        let facing_y = buf.get_u8();
        let facing_z = buf.get_u8();
        Ok(Self {
            position,
            placed_block_direction,
            stack,
            facing_x,
            facing_y,
            facing_z,
        })
    }
}

/// Action types for Serverbound UseEntity packet (0x02).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseEntityAction {
    Interact = 0,
    Attack = 1,
    InteractAt = 2,
}

impl UseEntityAction {
    pub fn from_i32(val: i32) -> Result<Self, ProtocolError> {
        match val {
            0 => Ok(Self::Interact),
            1 => Ok(Self::Attack),
            2 => Ok(Self::InteractAt),
            _ => Err(ProtocolError::InvalidEnumValue {
                enum_name: "UseEntityAction",
                value: val,
            }),
        }
    }
}

/// Serverbound Use Entity packet (0x02).
#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundUseEntityPacket {
    pub entity_id: i32,
    pub action: UseEntityAction,
    pub target: Option<(f32, f32, f32)>,
}

impl ServerboundUseEntityPacket {
    pub const PACKET_ID: i32 = 0x02;

    pub fn new(entity_id: i32, action: UseEntityAction, target: Option<(f32, f32, f32)>) -> Self {
        Self {
            entity_id,
            action,
            target,
        }
    }

    pub fn attack(entity_id: i32) -> Self {
        Self {
            entity_id,
            action: UseEntityAction::Attack,
            target: None,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.write_varint(self.action as i32);
        if self.action == UseEntityAction::InteractAt {
            if let Some((x, y, z)) = self.target {
                buf.extend_from_slice(&x.to_be_bytes());
                buf.extend_from_slice(&y.to_be_bytes());
                buf.extend_from_slice(&z.to_be_bytes());
            } else {
                buf.extend_from_slice(&[0u8; 12]);
            }
        }
        Ok(())
    }

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        let action = UseEntityAction::from_i32(buf.read_varint()?)?;
        let target = if action == UseEntityAction::InteractAt {
            if buf.remaining() < 12 {
                return Err(ProtocolError::BufferUnderflow {
                    needed: 12,
                    available: buf.remaining(),
                });
            }
            Some((buf.get_f32(), buf.get_f32(), buf.get_f32()))
        } else {
            None
        };
        Ok(Self {
            entity_id,
            action,
            target,
        })
    }
}

/// Serverbound Animation packet (Arm Swing) (0x0A).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerboundAnimationPacket;

impl ServerboundAnimationPacket {
    pub const PACKET_ID: i32 = 0x0A;

    pub fn encode(&self, _buf: &mut BytesMut) -> Result<(), ProtocolError> {
        Ok(())
    }

    pub fn decode(_buf: &mut Bytes) -> Result<Self, ProtocolError> {
        Ok(Self)
    }
}

/// Serverbound Held Item Change packet (0x09).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundHeldItemChangePacket {
    pub slot: i16,
}

impl ServerboundHeldItemChangePacket {
    pub const PACKET_ID: i32 = 0x09;

    pub fn new(slot: i16) -> Self {
        Self { slot }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.slot.to_be_bytes());
        Ok(())
    }

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 2 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 2,
                available: buf.remaining(),
            });
        }
        let slot = buf.get_i16();
        Ok(Self { slot })
    }
}

/// Serverbound Client Status action enum (Protocol 47).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientStatusAction {
    PerformRespawn = 0,
    RequestStats = 1,
    OpenInventoryAchievement = 2,
}

/// Serverbound Client Status packet (0x16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundClientStatusPacket {
    pub action: ClientStatusAction,
}

impl ServerboundClientStatusPacket {
    pub const PACKET_ID: i32 = 0x16;

    pub fn new(action: ClientStatusAction) -> Self {
        Self { action }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.action as i32);
        Ok(())
    }

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let val = buf.read_varint()?;
        let action = match val {
            0 => ClientStatusAction::PerformRespawn,
            1 => ClientStatusAction::RequestStats,
            2 => ClientStatusAction::OpenInventoryAchievement,
            _ => ClientStatusAction::PerformRespawn,
        };
        Ok(Self { action })
    }
}

/// Serverbound Custom Payload packet (0x17).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCustomPayloadPacket {
    pub channel: String,
    pub data: Vec<u8>,
}

impl ServerboundCustomPayloadPacket {
    pub const PACKET_ID: i32 = 0x17;

    pub fn new(channel: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            channel: channel.into(),
            data,
        }
    }

    /// Helper to create an `MC|Brand` packet formatted according to vanilla MCP 1.8.9:
    /// channel = "MC|Brand", data = writeString(brand_name) (VarInt length + UTF-8 bytes).
    pub fn brand(brand_name: &str) -> Self {
        let mut buf = BytesMut::new();
        buf.write_string(brand_name).unwrap();
        Self {
            channel: "MC|Brand".to_string(),
            data: buf.to_vec(),
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_string(&self.channel)?;
        buf.extend_from_slice(&self.data);
        Ok(())
    }

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let channel = buf.read_string(20)?;
        let mut data = vec![0u8; buf.remaining()];
        buf.copy_to_slice(&mut data);
        Ok(Self { channel, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_status_packet_roundtrip() {
        let pkt = ServerboundClientStatusPacket::new(ClientStatusAction::PerformRespawn);
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ServerboundClientStatusPacket::decode(&mut bytes).unwrap();
        assert_eq!(decoded.action, ClientStatusAction::PerformRespawn);
    }

    #[test]
    fn test_custom_payload_packet_roundtrip() {
        let pkt = ServerboundCustomPayloadPacket::brand("vanilla");
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ServerboundCustomPayloadPacket::decode(&mut bytes).unwrap();
        assert_eq!(decoded.channel, "MC|Brand");
        assert_eq!(decoded.data, pkt.data);
    }
}
