use crate::codec::buf_ext::BufMutExt;
use crate::error::ProtocolError;
use bytes::BytesMut;

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
