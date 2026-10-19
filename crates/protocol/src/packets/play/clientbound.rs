use crate::codec::buf_ext::{BufExt, BufMutExt};
use crate::codec::metadata::MetadataEntry;
use crate::codec::position::Position;
use crate::codec::slot::{read_slot, write_slot, Slot};
use crate::error::ProtocolError;
use bytes::{Buf, Bytes, BytesMut};
use uuid::Uuid;

/// Clientbound KeepAlive packet (0x00).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundKeepAlivePacket {
    pub keep_alive_id: i32,
}

impl ClientboundKeepAlivePacket {
    pub const PACKET_ID: i32 = 0x00;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let keep_alive_id = buf.read_varint()?;
        Ok(Self { keep_alive_id })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.keep_alive_id);
        Ok(())
    }
}

/// Clientbound JoinGame packet (0x01).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundJoinGamePacket {
    pub entity_id: i32,
    pub game_mode: u8,
    pub dimension: i8,
    pub difficulty: u8,
    pub max_players: u8,
    pub level_type: String,
    pub reduced_debug_info: bool,
}

impl ClientboundJoinGamePacket {
    pub const PACKET_ID: i32 = 0x01;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 8 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 8,
                available: buf.remaining(),
            });
        }
        let entity_id = buf.get_i32();
        let game_mode = buf.get_u8();
        let dimension = buf.get_i8();
        let difficulty = buf.get_u8();
        let max_players = buf.get_u8();
        let level_type = buf.read_string(16)?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let reduced_debug_info = buf.get_u8() != 0;

        Ok(Self {
            entity_id,
            game_mode,
            dimension,
            difficulty,
            max_players,
            level_type,
            reduced_debug_info,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.entity_id.to_be_bytes());
        buf.extend_from_slice(&[
            self.game_mode,
            self.dimension as u8,
            self.difficulty,
            self.max_players,
        ]);
        buf.write_string(&self.level_type)?;
        buf.extend_from_slice(&[if self.reduced_debug_info { 1 } else { 0 }]);
        Ok(())
    }
}

/// Clientbound Chat Message packet (0x02).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundChatPacket {
    pub message: String,
    pub position: i8,
}

impl ClientboundChatPacket {
    pub const PACKET_ID: i32 = 0x02;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let message = buf.read_string(32767)?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let position = buf.get_i8();
        Ok(Self { message, position })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_string(&self.message)?;
        buf.extend_from_slice(&[self.position as u8]);
        Ok(())
    }
}

/// Clientbound Time Update packet (0x03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTimeUpdatePacket {
    pub world_age: i64,
    pub time_of_day: i64,
}

impl ClientboundTimeUpdatePacket {
    pub const PACKET_ID: i32 = 0x03;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 16 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 16,
                available: buf.remaining(),
            });
        }
        let world_age = buf.get_i64();
        let time_of_day = buf.get_i64();
        Ok(Self {
            world_age,
            time_of_day,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.world_age.to_be_bytes());
        buf.extend_from_slice(&self.time_of_day.to_be_bytes());
        Ok(())
    }
}

/// Clientbound Respawn packet (0x07).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRespawnPacket {
    pub dimension: i32,
    pub difficulty: u8,
    pub game_mode: u8,
    pub level_type: String,
}

impl ClientboundRespawnPacket {
    pub const PACKET_ID: i32 = 0x07;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 6 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 6,
                available: buf.remaining(),
            });
        }
        let dimension = buf.get_i32();
        let difficulty = buf.get_u8();
        let game_mode = buf.get_u8();
        let level_type = buf.read_string(16)?;
        Ok(Self {
            dimension,
            difficulty,
            game_mode,
            level_type,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.dimension.to_be_bytes());
        buf.extend_from_slice(&[self.difficulty, self.game_mode]);
        buf.write_string(&self.level_type)?;
        Ok(())
    }
}

/// Clientbound Player Position and Look packet (0x08).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundPlayerPosLookPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
}

impl ClientboundPlayerPosLookPacket {
    pub const PACKET_ID: i32 = 0x08;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 33 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 33,
                available: buf.remaining(),
            });
        }
        let x = buf.get_f64();
        let y = buf.get_f64();
        let z = buf.get_f64();
        let yaw = buf.get_f32();
        let pitch = buf.get_f32();
        let flags = buf.get_u8();
        Ok(Self {
            x,
            y,
            z,
            yaw,
            pitch,
            flags,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf.extend_from_slice(&self.z.to_be_bytes());
        buf.extend_from_slice(&self.yaw.to_be_bytes());
        buf.extend_from_slice(&self.pitch.to_be_bytes());
        buf.extend_from_slice(&[self.flags]);
        Ok(())
    }
}

/// Clientbound Chunk Data packet (0x21).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundChunkDataPacket {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub ground_up: bool,
    pub bit_mask: u16,
    pub data: Vec<u8>,
}

impl ClientboundChunkDataPacket {
    pub const PACKET_ID: i32 = 0x21;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 11 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 11,
                available: buf.remaining(),
            });
        }
        let chunk_x = buf.get_i32();
        let chunk_z = buf.get_i32();
        let ground_up = buf.get_u8() != 0;
        let bit_mask = buf.get_u16();

        let data_len = buf.read_varint()? as usize;
        if buf.remaining() < data_len {
            return Err(ProtocolError::BufferUnderflow {
                needed: data_len,
                available: buf.remaining(),
            });
        }
        let mut data = vec![0u8; data_len];
        buf.copy_to_slice(&mut data);

        Ok(Self {
            chunk_x,
            chunk_z,
            ground_up,
            bit_mask,
            data,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.chunk_x.to_be_bytes());
        buf.extend_from_slice(&self.chunk_z.to_be_bytes());
        buf.extend_from_slice(&[if self.ground_up { 1 } else { 0 }]);
        buf.extend_from_slice(&self.bit_mask.to_be_bytes());
        buf.write_varint(self.data.len() as i32);
        buf.extend_from_slice(&self.data);
        Ok(())
    }
}

/// Record of a changed block in MultiBlockChange.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockChangeRecord {
    pub horizontal_pos: u8,
    pub y: u8,
    pub block_state: i32,
}

impl BlockChangeRecord {
    #[inline]
    pub fn rel_x(&self) -> usize {
        ((self.horizontal_pos >> 4) & 15) as usize
    }

    #[inline]
    pub fn rel_z(&self) -> usize {
        (self.horizontal_pos & 15) as usize
    }
}

/// Clientbound MultiBlockChange packet (0x22).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundMultiBlockChangePacket {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub records: Vec<BlockChangeRecord>,
}

impl ClientboundMultiBlockChangePacket {
    pub const PACKET_ID: i32 = 0x22;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 8 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 8,
                available: buf.remaining(),
            });
        }
        let chunk_x = buf.get_i32();
        let chunk_z = buf.get_i32();
        let record_count = buf.read_varint()? as usize;
        let mut records = Vec::with_capacity(record_count);

        for _ in 0..record_count {
            if buf.remaining() < 2 {
                return Err(ProtocolError::BufferUnderflow {
                    needed: 2,
                    available: buf.remaining(),
                });
            }
            let horizontal_pos = buf.get_u8();
            let y = buf.get_u8();
            let block_state = buf.read_varint()?;
            records.push(BlockChangeRecord {
                horizontal_pos,
                y,
                block_state,
            });
        }

        Ok(Self {
            chunk_x,
            chunk_z,
            records,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.chunk_x.to_be_bytes());
        buf.extend_from_slice(&self.chunk_z.to_be_bytes());
        buf.write_varint(self.records.len() as i32);
        for rec in &self.records {
            buf.extend_from_slice(&[rec.horizontal_pos, rec.y]);
            buf.write_varint(rec.block_state);
        }
        Ok(())
    }
}

/// Clientbound BlockChange packet (0x23).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundBlockChangePacket {
    pub position: Position,
    pub block_state: i32,
}

impl ClientboundBlockChangePacket {
    pub const PACKET_ID: i32 = 0x23;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let position = buf.read_position()?;
        let block_state = buf.read_varint()?;
        Ok(Self {
            position,
            block_state,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_position(&self.position);
        buf.write_varint(self.block_state);
        Ok(())
    }
}

/// Meta entry for ChunkBulk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkBulkMeta {
    pub x: i32,
    pub z: i32,
    pub bit_mask: u16,
}

/// Clientbound MapChunkBulk packet (0x26).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundMapChunkBulkPacket {
    pub sky_light_sent: bool,
    pub meta: Vec<ChunkBulkMeta>,
    pub data: Vec<u8>,
}

impl ClientboundMapChunkBulkPacket {
    pub const PACKET_ID: i32 = 0x26;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let sky_light_sent = buf.get_u8() != 0;
        let count = buf.read_varint()? as usize;
        let mut meta = Vec::with_capacity(count);

        for _ in 0..count {
            if buf.remaining() < 10 {
                return Err(ProtocolError::BufferUnderflow {
                    needed: 10,
                    available: buf.remaining(),
                });
            }
            let x = buf.get_i32();
            let z = buf.get_i32();
            let bit_mask = buf.get_u16();
            meta.push(ChunkBulkMeta { x, z, bit_mask });
        }

        // Remaining data
        let mut data = vec![0u8; buf.remaining()];
        buf.copy_to_slice(&mut data);

        Ok(Self {
            sky_light_sent,
            meta,
            data,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[if self.sky_light_sent { 1 } else { 0 }]);
        buf.write_varint(self.meta.len() as i32);
        for m in &self.meta {
            buf.extend_from_slice(&m.x.to_be_bytes());
            buf.extend_from_slice(&m.z.to_be_bytes());
            buf.extend_from_slice(&m.bit_mask.to_be_bytes());
        }
        buf.extend_from_slice(&self.data);
        Ok(())
    }
}

/// Entry in PlayerListItem packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListItemEntry {
    pub uuid: Uuid,
    pub name: String,
    pub game_mode: i32,
    pub ping: i32,
    pub display_name: Option<String>,
}

/// Clientbound PlayerListItem (Tab List) packet (0x38).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerListItemPacket {
    pub action: i32, // 0: AddPlayer, 1: UpdateGameMode, 2: UpdateLatency, 3: UpdateDisplayName, 4: RemovePlayer
    pub entries: Vec<PlayerListItemEntry>,
}

impl ClientboundPlayerListItemPacket {
    pub const PACKET_ID: i32 = 0x38;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let action = buf.read_varint()?;
        let count = buf.read_varint()? as usize;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            let uuid = buf.read_uuid()?;
            match action {
                0 => {
                    // AddPlayer
                    let name = buf.read_string(16)?;
                    let prop_count = buf.read_varint()? as usize;
                    for _ in 0..prop_count {
                        let _prop_name = buf.read_string(32767)?;
                        let _prop_value = buf.read_string(32767)?;
                        if buf.remaining() < 1 {
                            return Err(ProtocolError::BufferUnderflow {
                                needed: 1,
                                available: buf.remaining(),
                            });
                        }
                        let has_sig = buf.get_u8() != 0;
                        if has_sig {
                            let _sig = buf.read_string(32767)?;
                        }
                    }
                    let game_mode = buf.read_varint()?;
                    let ping = buf.read_varint()?;
                    if buf.remaining() < 1 {
                        return Err(ProtocolError::BufferUnderflow {
                            needed: 1,
                            available: buf.remaining(),
                        });
                    }
                    let has_display_name = buf.get_u8() != 0;
                    let display_name = if has_display_name {
                        Some(buf.read_string(32767)?)
                    } else {
                        None
                    };

                    entries.push(PlayerListItemEntry {
                        uuid,
                        name,
                        game_mode,
                        ping,
                        display_name,
                    });
                }
                1 => {
                    // UpdateGameMode
                    let game_mode = buf.read_varint()?;
                    entries.push(PlayerListItemEntry {
                        uuid,
                        name: String::new(),
                        game_mode,
                        ping: 0,
                        display_name: None,
                    });
                }
                2 => {
                    // UpdateLatency
                    let ping = buf.read_varint()?;
                    entries.push(PlayerListItemEntry {
                        uuid,
                        name: String::new(),
                        game_mode: 0,
                        ping,
                        display_name: None,
                    });
                }
                3 => {
                    // UpdateDisplayName
                    if buf.remaining() < 1 {
                        return Err(ProtocolError::BufferUnderflow {
                            needed: 1,
                            available: buf.remaining(),
                        });
                    }
                    let has_display_name = buf.get_u8() != 0;
                    let display_name = if has_display_name {
                        Some(buf.read_string(32767)?)
                    } else {
                        None
                    };
                    entries.push(PlayerListItemEntry {
                        uuid,
                        name: String::new(),
                        game_mode: 0,
                        ping: 0,
                        display_name,
                    });
                }
                4 => {
                    // RemovePlayer
                    entries.push(PlayerListItemEntry {
                        uuid,
                        name: String::new(),
                        game_mode: 0,
                        ping: 0,
                        display_name: None,
                    });
                }
                _ => {}
            }
        }

        Ok(Self { action, entries })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.action);
        buf.write_varint(self.entries.len() as i32);
        for entry in &self.entries {
            buf.write_uuid(&entry.uuid);
            match self.action {
                0 => {
                    buf.write_string(&entry.name)?;
                    buf.write_varint(0); // 0 properties
                    buf.write_varint(entry.game_mode);
                    buf.write_varint(entry.ping);
                    if let Some(dn) = &entry.display_name {
                        buf.extend_from_slice(&[1]);
                        buf.write_string(dn)?;
                    } else {
                        buf.extend_from_slice(&[0]);
                    }
                }
                1 => {
                    buf.write_varint(entry.game_mode);
                }
                2 => {
                    buf.write_varint(entry.ping);
                }
                3 => {
                    if let Some(dn) = &entry.display_name {
                        buf.extend_from_slice(&[1]);
                        buf.write_string(dn)?;
                    } else {
                        buf.extend_from_slice(&[0]);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// Clientbound ScoreboardObjective packet (0x3B).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundScoreboardObjectivePacket {
    pub name: String,
    pub action: i8, // 0 = create, 1 = remove, 2 = update text
    pub display_text: Option<String>,
    pub objective_type: Option<String>,
}

impl ClientboundScoreboardObjectivePacket {
    pub const PACKET_ID: i32 = 0x3B;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let name = buf.read_string(16)?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let action = buf.get_i8();
        let (display_text, objective_type) = if action == 0 || action == 2 {
            let dt = buf.read_string(32)?;
            let ot = buf.read_string(16)?;
            (Some(dt), Some(ot))
        } else {
            (None, None)
        };

        Ok(Self {
            name,
            action,
            display_text,
            objective_type,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_string(&self.name)?;
        buf.extend_from_slice(&[self.action as u8]);
        if self.action == 0 || self.action == 2 {
            buf.write_string(self.display_text.as_deref().unwrap_or(""))?;
            buf.write_string(self.objective_type.as_deref().unwrap_or("integer"))?;
        }
        Ok(())
    }
}

/// Clientbound UpdateScore packet (0x3C).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateScorePacket {
    pub score_holder: String,
    pub action: i8, // 0 = create/update, 1 = remove
    pub objective_name: String,
    pub value: Option<i32>,
}

impl ClientboundUpdateScorePacket {
    pub const PACKET_ID: i32 = 0x3C;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let score_holder = buf.read_string(40)?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let action = buf.get_i8();
        let objective_name = buf.read_string(16)?;
        let value = if action != 1 {
            Some(buf.read_varint()?)
        } else {
            None
        };

        Ok(Self {
            score_holder,
            action,
            objective_name,
            value,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_string(&self.score_holder)?;
        buf.extend_from_slice(&[self.action as u8]);
        buf.write_string(&self.objective_name)?;
        if self.action != 1 {
            buf.write_varint(self.value.unwrap_or(0));
        }
        Ok(())
    }
}

/// Clientbound DisplayScoreboard packet (0x3D).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDisplayScoreboardPacket {
    pub position: i8, // 0 = List, 1 = Sidebar, 2 = BelowName
    pub score_name: String,
}

impl ClientboundDisplayScoreboardPacket {
    pub const PACKET_ID: i32 = 0x3D;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let position = buf.get_i8();
        let score_name = buf.read_string(16)?;
        Ok(Self {
            position,
            score_name,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[self.position as u8]);
        buf.write_string(&self.score_name)?;
        Ok(())
    }
}

/// Clientbound Disconnect packet (0x40).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDisconnectPlayPacket {
    pub reason: String,
}

impl ClientboundDisconnectPlayPacket {
    pub const PACKET_ID: i32 = 0x40;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let reason = buf.read_string(32767)?;
        Ok(Self { reason })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_string(&self.reason)?;
        Ok(())
    }
}

/// Clientbound SpawnPlayer packet (0x0C).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSpawnPlayerPacket {
    pub entity_id: i32,
    pub player_id: Uuid,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: i8,
    pub pitch: i8,
    pub current_item: i16,
    pub metadata: Vec<MetadataEntry>,
}

impl ClientboundSpawnPlayerPacket {
    pub const PACKET_ID: i32 = 0x0C;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        let player_id = buf.read_uuid()?;
        if buf.remaining() < 16 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 16,
                available: buf.remaining(),
            });
        }
        let x = buf.get_i32();
        let y = buf.get_i32();
        let z = buf.get_i32();
        let yaw = buf.get_i8();
        let pitch = buf.get_i8();
        let current_item = buf.get_i16();
        let metadata = buf.read_metadata()?;

        Ok(Self {
            entity_id,
            player_id,
            x,
            y,
            z,
            yaw,
            pitch,
            current_item,
            metadata,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.write_uuid(&self.player_id);
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf.extend_from_slice(&self.z.to_be_bytes());
        buf.extend_from_slice(&[self.yaw as u8, self.pitch as u8]);
        buf.extend_from_slice(&self.current_item.to_be_bytes());
        buf.write_metadata(&self.metadata)?;
        Ok(())
    }
}

/// Clientbound SpawnMob packet (0x0F).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSpawnMobPacket {
    pub entity_id: i32,
    pub mob_type: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: i8,
    pub pitch: i8,
    pub head_pitch: i8,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
    pub metadata: Vec<MetadataEntry>,
}

impl ClientboundSpawnMobPacket {
    pub const PACKET_ID: i32 = 0x0F;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 22 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 22,
                available: buf.remaining(),
            });
        }
        let mob_type = buf.get_u8();
        let x = buf.get_i32();
        let y = buf.get_i32();
        let z = buf.get_i32();
        let yaw = buf.get_i8();
        let pitch = buf.get_i8();
        let head_pitch = buf.get_i8();
        let velocity_x = buf.get_i16();
        let velocity_y = buf.get_i16();
        let velocity_z = buf.get_i16();
        let metadata = buf.read_metadata()?;

        Ok(Self {
            entity_id,
            mob_type,
            x,
            y,
            z,
            yaw,
            pitch,
            head_pitch,
            velocity_x,
            velocity_y,
            velocity_z,
            metadata,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&[self.mob_type]);
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf.extend_from_slice(&self.z.to_be_bytes());
        buf.extend_from_slice(&[self.yaw as u8, self.pitch as u8, self.head_pitch as u8]);
        buf.extend_from_slice(&self.velocity_x.to_be_bytes());
        buf.extend_from_slice(&self.velocity_y.to_be_bytes());
        buf.extend_from_slice(&self.velocity_z.to_be_bytes());
        buf.write_metadata(&self.metadata)?;
        Ok(())
    }
}

/// Clientbound SpawnObject packet (0x0E).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSpawnObjectPacket {
    pub entity_id: i32,
    pub object_type: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub pitch: i8,
    pub yaw: i8,
    pub data: i32,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}

impl ClientboundSpawnObjectPacket {
    pub const PACKET_ID: i32 = 0x0E;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 19 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 19,
                available: buf.remaining(),
            });
        }
        let object_type = buf.get_u8();
        let x = buf.get_i32();
        let y = buf.get_i32();
        let z = buf.get_i32();
        let pitch = buf.get_i8();
        let yaw = buf.get_i8();
        let data = buf.get_i32();

        let (velocity_x, velocity_y, velocity_z) = if data > 0 {
            if buf.remaining() < 6 {
                return Err(ProtocolError::BufferUnderflow {
                    needed: 6,
                    available: buf.remaining(),
                });
            }
            (buf.get_i16(), buf.get_i16(), buf.get_i16())
        } else {
            (0, 0, 0)
        };

        Ok(Self {
            entity_id,
            object_type,
            x,
            y,
            z,
            pitch,
            yaw,
            data,
            velocity_x,
            velocity_y,
            velocity_z,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&[self.object_type]);
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf.extend_from_slice(&self.z.to_be_bytes());
        buf.extend_from_slice(&[self.pitch as u8, self.yaw as u8]);
        buf.extend_from_slice(&self.data.to_be_bytes());
        if self.data > 0 {
            buf.extend_from_slice(&self.velocity_x.to_be_bytes());
            buf.extend_from_slice(&self.velocity_y.to_be_bytes());
            buf.extend_from_slice(&self.velocity_z.to_be_bytes());
        }
        Ok(())
    }
}

/// Clientbound DestroyEntities packet (0x13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDestroyEntitiesPacket {
    pub entity_ids: Vec<i32>,
}

impl ClientboundDestroyEntitiesPacket {
    pub const PACKET_ID: i32 = 0x13;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let count = buf.read_varint()?;
        let mut entity_ids = Vec::with_capacity((count as usize).min(4096));
        for _ in 0..count {
            entity_ids.push(buf.read_varint()?);
        }
        Ok(Self { entity_ids })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_ids.len() as i32);
        for &id in &self.entity_ids {
            buf.write_varint(id);
        }
        Ok(())
    }
}

/// Clientbound Entity packet (0x14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundEntityPacket {
    pub entity_id: i32,
}

impl ClientboundEntityPacket {
    pub const PACKET_ID: i32 = 0x14;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        Ok(Self { entity_id })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        Ok(())
    }
}

/// Clientbound EntityRelativeMove packet (0x15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundEntityRelMovePacket {
    pub entity_id: i32,
    pub delta_x: i8,
    pub delta_y: i8,
    pub delta_z: i8,
    pub on_ground: bool,
}

impl ClientboundEntityRelMovePacket {
    pub const PACKET_ID: i32 = 0x15;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 4 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 4,
                available: buf.remaining(),
            });
        }
        let delta_x = buf.get_i8();
        let delta_y = buf.get_i8();
        let delta_z = buf.get_i8();
        let on_ground = buf.get_u8() != 0;

        Ok(Self {
            entity_id,
            delta_x,
            delta_y,
            delta_z,
            on_ground,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&[
            self.delta_x as u8,
            self.delta_y as u8,
            self.delta_z as u8,
            self.on_ground as u8,
        ]);
        Ok(())
    }
}

/// Clientbound EntityLook packet (0x16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundEntityLookPacket {
    pub entity_id: i32,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool,
}

impl ClientboundEntityLookPacket {
    pub const PACKET_ID: i32 = 0x16;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 3 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 3,
                available: buf.remaining(),
            });
        }
        let yaw = buf.get_i8();
        let pitch = buf.get_i8();
        let on_ground = buf.get_u8() != 0;

        Ok(Self {
            entity_id,
            yaw,
            pitch,
            on_ground,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&[self.yaw as u8, self.pitch as u8, self.on_ground as u8]);
        Ok(())
    }
}

/// Clientbound EntityLookAndRelativeMove packet (0x17).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundEntityLookMovePacket {
    pub entity_id: i32,
    pub delta_x: i8,
    pub delta_y: i8,
    pub delta_z: i8,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool,
}

impl ClientboundEntityLookMovePacket {
    pub const PACKET_ID: i32 = 0x17;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 6 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 6,
                available: buf.remaining(),
            });
        }
        let delta_x = buf.get_i8();
        let delta_y = buf.get_i8();
        let delta_z = buf.get_i8();
        let yaw = buf.get_i8();
        let pitch = buf.get_i8();
        let on_ground = buf.get_u8() != 0;

        Ok(Self {
            entity_id,
            delta_x,
            delta_y,
            delta_z,
            yaw,
            pitch,
            on_ground,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&[
            self.delta_x as u8,
            self.delta_y as u8,
            self.delta_z as u8,
            self.yaw as u8,
            self.pitch as u8,
            self.on_ground as u8,
        ]);
        Ok(())
    }
}

/// Clientbound EntityTeleport packet (0x18).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundEntityTeleportPacket {
    pub entity_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool,
}

impl ClientboundEntityTeleportPacket {
    pub const PACKET_ID: i32 = 0x18;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 15 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 15,
                available: buf.remaining(),
            });
        }
        let x = buf.get_i32();
        let y = buf.get_i32();
        let z = buf.get_i32();
        let yaw = buf.get_i8();
        let pitch = buf.get_i8();
        let on_ground = buf.get_u8() != 0;

        Ok(Self {
            entity_id,
            x,
            y,
            z,
            yaw,
            pitch,
            on_ground,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf.extend_from_slice(&self.z.to_be_bytes());
        buf.extend_from_slice(&[self.yaw as u8, self.pitch as u8, self.on_ground as u8]);
        Ok(())
    }
}

/// Clientbound EntityHeadLook packet (0x19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundEntityHeadLookPacket {
    pub entity_id: i32,
    pub head_yaw: i8,
}

impl ClientboundEntityHeadLookPacket {
    pub const PACKET_ID: i32 = 0x19;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let head_yaw = buf.get_i8();
        Ok(Self {
            entity_id,
            head_yaw,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&[self.head_yaw as u8]);
        Ok(())
    }
}

/// Clientbound Entity Status packet (0x1A).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundEntityStatusPacket {
    pub entity_id: i32,
    pub status: i8,
}

impl ClientboundEntityStatusPacket {
    pub const PACKET_ID: i32 = 0x1A;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 5 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 5,
                available: buf.remaining(),
            });
        }
        let entity_id = buf.get_i32();
        let status = buf.get_i8();
        Ok(Self { entity_id, status })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.entity_id.to_be_bytes());
        buf.extend_from_slice(&[self.status as u8]);
        Ok(())
    }
}

/// Clientbound Animation packet (0x0B).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundAnimationPacket {
    pub entity_id: i32,
    pub animation: u8,
}

impl ClientboundAnimationPacket {
    pub const PACKET_ID: i32 = 0x0B;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let animation = buf.get_u8();
        Ok(Self {
            entity_id,
            animation,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&[self.animation]);
        Ok(())
    }
}

/// Clientbound EntityVelocity packet (0x12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundEntityVelocityPacket {
    pub entity_id: i32,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}

impl ClientboundEntityVelocityPacket {
    pub const PACKET_ID: i32 = 0x12;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 6 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 6,
                available: buf.remaining(),
            });
        }
        let velocity_x = buf.get_i16();
        let velocity_y = buf.get_i16();
        let velocity_z = buf.get_i16();
        Ok(Self {
            entity_id,
            velocity_x,
            velocity_y,
            velocity_z,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&self.velocity_x.to_be_bytes());
        buf.extend_from_slice(&self.velocity_y.to_be_bytes());
        buf.extend_from_slice(&self.velocity_z.to_be_bytes());
        Ok(())
    }
}

/// Clientbound EntityEquipment packet (0x04).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundEntityEquipmentPacket {
    pub entity_id: i32,
    pub slot: i16,
    pub item: Option<Slot>,
}

impl ClientboundEntityEquipmentPacket {
    pub const PACKET_ID: i32 = 0x04;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 2 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 2,
                available: buf.remaining(),
            });
        }
        let slot = buf.get_i16();
        let item = buf.read_slot()?;
        Ok(Self {
            entity_id,
            slot,
            item,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&self.slot.to_be_bytes());
        buf.write_slot(self.item.as_ref())?;
        Ok(())
    }
}

/// Clientbound EntityMetadata packet (0x1C).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundEntityMetadataPacket {
    pub entity_id: i32,
    pub metadata: Vec<MetadataEntry>,
}

impl ClientboundEntityMetadataPacket {
    pub const PACKET_ID: i32 = 0x1C;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        let metadata = buf.read_metadata()?;
        Ok(Self {
            entity_id,
            metadata,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.write_metadata(&self.metadata)?;
        Ok(())
    }
}

/// Clientbound Teams packet (0x3E).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTeamsPacket {
    pub team_name: String,
    pub action: i8, // 0 = create, 1 = remove, 2 = update, 3 = add players, 4 = remove players
    pub display_name: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub friendly_flags: Option<i8>,
    pub name_tag_visibility: Option<String>,
    pub color: Option<i8>,
    pub players: Vec<String>,
}

impl ClientboundTeamsPacket {
    pub const PACKET_ID: i32 = 0x3E;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let team_name = buf.read_string(16)?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let action = buf.get_i8();
        let (display_name, prefix, suffix, friendly_flags, name_tag_visibility, color) =
            if action == 0 || action == 2 {
                let dn = buf.read_string(32)?;
                let p = buf.read_string(16)?;
                let s = buf.read_string(16)?;
                if buf.remaining() < 1 {
                    return Err(ProtocolError::BufferUnderflow {
                        needed: 1,
                        available: buf.remaining(),
                    });
                }
                let ff = buf.get_i8();
                let ntv = buf.read_string(32)?;
                if buf.remaining() < 1 {
                    return Err(ProtocolError::BufferUnderflow {
                        needed: 1,
                        available: buf.remaining(),
                    });
                }
                let c = buf.get_i8();
                (Some(dn), Some(p), Some(s), Some(ff), Some(ntv), Some(c))
            } else {
                (None, None, None, None, None, None)
            };

        let players = if action == 0 || action == 3 || action == 4 {
            let count = buf.read_varint()? as usize;
            let mut list = Vec::with_capacity(count);
            for _ in 0..count {
                list.push(buf.read_string(40)?);
            }
            list
        } else {
            Vec::new()
        };

        Ok(Self {
            team_name,
            action,
            display_name,
            prefix,
            suffix,
            friendly_flags,
            name_tag_visibility,
            color,
            players,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_string(&self.team_name)?;
        buf.extend_from_slice(&[self.action as u8]);
        if self.action == 0 || self.action == 2 {
            buf.write_string(self.display_name.as_deref().unwrap_or(""))?;
            buf.write_string(self.prefix.as_deref().unwrap_or(""))?;
            buf.write_string(self.suffix.as_deref().unwrap_or(""))?;
            buf.extend_from_slice(&[self.friendly_flags.unwrap_or(0) as u8]);
            buf.write_string(self.name_tag_visibility.as_deref().unwrap_or("always"))?;
            buf.extend_from_slice(&[self.color.unwrap_or(-1) as u8]);
        }
        if self.action == 0 || self.action == 3 || self.action == 4 {
            buf.write_varint(self.players.len() as i32);
            for p in &self.players {
                buf.write_string(p)?;
            }
        }
        Ok(())
    }
}

/// Clientbound Explosion packet (0x27).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundExplosionPacket {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub radius: f32,
    pub records: Vec<(i8, i8, i8)>,
    pub player_motion_x: f32,
    pub player_motion_y: f32,
    pub player_motion_z: f32,
}

impl ClientboundExplosionPacket {
    pub const PACKET_ID: i32 = 0x27;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 16 + 4 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 20,
                available: buf.remaining(),
            });
        }
        let x = buf.get_f32();
        let y = buf.get_f32();
        let z = buf.get_f32();
        let radius = buf.get_f32();
        let record_count = buf.get_i32();
        if record_count < 0 || buf.remaining() < (record_count as usize) * 3 + 12 {
            return Err(ProtocolError::BufferUnderflow {
                needed: (record_count.max(0) as usize) * 3 + 12,
                available: buf.remaining(),
            });
        }
        let mut records = Vec::with_capacity(record_count as usize);
        for _ in 0..record_count {
            records.push((buf.get_i8(), buf.get_i8(), buf.get_i8()));
        }
        let player_motion_x = buf.get_f32();
        let player_motion_y = buf.get_f32();
        let player_motion_z = buf.get_f32();

        Ok(Self {
            x,
            y,
            z,
            radius,
            records,
            player_motion_x,
            player_motion_y,
            player_motion_z,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.x.to_be_bytes());
        buf.extend_from_slice(&self.y.to_be_bytes());
        buf.extend_from_slice(&self.z.to_be_bytes());
        buf.extend_from_slice(&self.radius.to_be_bytes());
        buf.extend_from_slice(&(self.records.len() as i32).to_be_bytes());
        for rec in &self.records {
            buf.extend_from_slice(&[rec.0 as u8, rec.1 as u8, rec.2 as u8]);
        }
        buf.extend_from_slice(&self.player_motion_x.to_be_bytes());
        buf.extend_from_slice(&self.player_motion_y.to_be_bytes());
        buf.extend_from_slice(&self.player_motion_z.to_be_bytes());
        Ok(())
    }
}

/// Clientbound Player Abilities packet (0x39).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundPlayerAbilitiesPacket {
    pub invulnerable: bool,
    pub is_flying: bool,
    pub allow_flying: bool,
    pub creative_mode: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
}

impl ClientboundPlayerAbilitiesPacket {
    pub const PACKET_ID: i32 = 0x39;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 9 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 9,
                available: buf.remaining(),
            });
        }
        let flags = buf.get_u8();
        let fly_speed = buf.get_f32();
        let walk_speed = buf.get_f32();

        Ok(Self {
            invulnerable: (flags & 0x01) != 0,
            is_flying: (flags & 0x02) != 0,
            allow_flying: (flags & 0x04) != 0,
            creative_mode: (flags & 0x08) != 0,
            fly_speed,
            walk_speed,
        })
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

/// Clientbound ConfirmTransaction packet (0x32).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundConfirmTransactionPacket {
    pub window_id: u8,
    pub action_number: i16,
    pub accepted: bool,
}

impl ClientboundConfirmTransactionPacket {
    pub const PACKET_ID: i32 = 0x32;

    pub fn new(window_id: u8, action_number: i16, accepted: bool) -> Self {
        Self {
            window_id,
            action_number,
            accepted,
        }
    }

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
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

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[self.window_id]);
        buf.extend_from_slice(&self.action_number.to_be_bytes());
        buf.extend_from_slice(&[if self.accepted { 1 } else { 0 }]);
        Ok(())
    }
}

/// Clientbound Block Break Animation packet (0x25).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundBlockBreakAnimPacket {
    pub entity_id: i32,
    pub position: Position,
    pub progress: i8,
}

impl ClientboundBlockBreakAnimPacket {
    pub const PACKET_ID: i32 = 0x25;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        let position = Position::read(buf)?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let progress = buf.get_i8();
        Ok(Self {
            entity_id,
            position,
            progress,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        self.position.write(buf);
        buf.extend_from_slice(&[self.progress as u8]);
        Ok(())
    }
}

/// Clientbound Update Health packet (0x06).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundUpdateHealthPacket {
    pub health: f32,
    pub food: i32,
    pub food_saturation: f32,
}

impl ClientboundUpdateHealthPacket {
    pub const PACKET_ID: i32 = 0x06;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 4 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 4,
                available: buf.remaining(),
            });
        }
        let health = buf.get_f32();
        let food = buf.read_varint()?;
        if buf.remaining() < 4 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 4,
                available: buf.remaining(),
            });
        }
        let food_saturation = buf.get_f32();
        Ok(Self {
            health,
            food,
            food_saturation,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.health.to_be_bytes());
        buf.write_varint(self.food);
        buf.extend_from_slice(&self.food_saturation.to_be_bytes());
        Ok(())
    }
}

/// Clientbound Set Experience packet (0x1F).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetExperiencePacket {
    pub experience_bar: f32,
    pub level: i32,
    pub total_experience: i32,
}

impl ClientboundSetExperiencePacket {
    pub const PACKET_ID: i32 = 0x1F;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 4 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 4,
                available: buf.remaining(),
            });
        }
        let experience_bar = buf.get_f32();
        let level = buf.read_varint()?;
        let total_experience = buf.read_varint()?;
        Ok(Self {
            experience_bar,
            level,
            total_experience,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&self.experience_bar.to_be_bytes());
        buf.write_varint(self.level);
        buf.write_varint(self.total_experience);
        Ok(())
    }
}

/// Clientbound Held Item Change packet (0x09).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundHeldItemChangePacket {
    pub slot: i8,
}

impl ClientboundHeldItemChangePacket {
    pub const PACKET_ID: i32 = 0x09;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let slot = buf.get_i8();
        Ok(Self { slot })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[self.slot as u8]);
        Ok(())
    }
}

/// Clientbound Set Slot packet (0x2F).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetSlotPacket {
    pub window_id: i8,
    pub slot: i16,
    pub item: Option<Slot>,
}

impl ClientboundSetSlotPacket {
    pub const PACKET_ID: i32 = 0x2F;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 3 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 3,
                available: buf.remaining(),
            });
        }
        let window_id = buf.get_i8();
        let slot = buf.get_i16();
        let item = read_slot(buf)?;
        Ok(Self {
            window_id,
            slot,
            item,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[self.window_id as u8]);
        buf.extend_from_slice(&self.slot.to_be_bytes());
        write_slot(self.item.as_ref(), buf)?;
        Ok(())
    }
}

/// Clientbound Window Items packet (0x30).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundWindowItemsPacket {
    pub window_id: u8,
    pub items: Vec<Option<Slot>>,
}

impl ClientboundWindowItemsPacket {
    pub const PACKET_ID: i32 = 0x30;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 3 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 3,
                available: buf.remaining(),
            });
        }
        let window_id = buf.get_u8();
        let count = buf.get_i16();
        let mut items = Vec::with_capacity(count.max(0) as usize);
        for _ in 0..count {
            items.push(read_slot(buf)?);
        }
        Ok(Self { window_id, items })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[self.window_id]);
        buf.extend_from_slice(&(self.items.len() as i16).to_be_bytes());
        for item in &self.items {
            write_slot(item.as_ref(), buf)?;
        }
        Ok(())
    }
}

/// Clientbound Entity Effect packet (0x1D).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundEntityEffectPacket {
    pub entity_id: i32,
    pub effect_id: i8,
    pub amplifier: i8,
    pub duration: i32,
    pub hide_particles: bool,
}

impl ClientboundEntityEffectPacket {
    pub const PACKET_ID: i32 = 0x1D;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 2 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 2,
                available: buf.remaining(),
            });
        }
        let effect_id = buf.get_i8();
        let amplifier = buf.get_i8();
        let duration = buf.read_varint()?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let hide_particles = buf.get_u8() != 0;
        Ok(Self {
            entity_id,
            effect_id,
            amplifier,
            duration,
            hide_particles,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&[self.effect_id as u8, self.amplifier as u8]);
        buf.write_varint(self.duration);
        buf.extend_from_slice(&[if self.hide_particles { 1 } else { 0 }]);
        Ok(())
    }
}

/// Clientbound Remove Entity Effect packet (0x1E).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRemoveEntityEffectPacket {
    pub entity_id: i32,
    pub effect_id: i8,
}

impl ClientboundRemoveEntityEffectPacket {
    pub const PACKET_ID: i32 = 0x1E;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let entity_id = buf.read_varint()?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let effect_id = buf.get_i8();
        Ok(Self {
            entity_id,
            effect_id,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_varint(self.entity_id);
        buf.extend_from_slice(&[self.effect_id as u8]);
        Ok(())
    }
}

/// Clientbound Change Game State packet (0x2B).
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundChangeGameStatePacket {
    pub state: u8,
    pub value: f32,
}

impl ClientboundChangeGameStatePacket {
    pub const PACKET_ID: i32 = 0x2B;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 5 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 5,
                available: buf.remaining(),
            });
        }
        let state = buf.get_u8();
        let value = buf.get_f32();
        Ok(Self { state, value })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[self.state]);
        buf.extend_from_slice(&self.value.to_be_bytes());
        Ok(())
    }
}

/// Clientbound Custom Payload packet (0x3F).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomPayloadPacket {
    pub channel: String,
    pub data: Vec<u8>,
}

impl ClientboundCustomPayloadPacket {
    pub const PACKET_ID: i32 = 0x3F;

    pub fn new(channel: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            channel: channel.into(),
            data,
        }
    }

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let channel = buf.read_string(20)?;
        let mut data = vec![0u8; buf.remaining()];
        buf.copy_to_slice(&mut data);
        Ok(Self { channel, data })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.write_string(&self.channel)?;
        buf.extend_from_slice(&self.data);
        Ok(())
    }
}

/// Clientbound Open Window packet (0x2D).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundOpenWindowPacket {
    pub window_id: u8,
    pub inventory_type: String,
    pub window_title: String,
    pub slot_count: u8,
    pub entity_id: Option<i32>,
}

impl ClientboundOpenWindowPacket {
    pub const PACKET_ID: i32 = 0x2D;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 2 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 2,
                available: buf.remaining(),
            });
        }
        let window_id = buf.get_u8();
        let inventory_type = buf.read_string(32)?;
        let window_title = buf.read_string(32767)?;
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let slot_count = buf.get_u8();
        let entity_id = if inventory_type == "EntityHorse" {
            if buf.remaining() < 4 {
                return Err(ProtocolError::BufferUnderflow {
                    needed: 4,
                    available: buf.remaining(),
                });
            }
            Some(buf.get_i32())
        } else {
            None
        };
        Ok(Self {
            window_id,
            inventory_type,
            window_title,
            slot_count,
            entity_id,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[self.window_id]);
        buf.write_string(&self.inventory_type)?;
        buf.write_string(&self.window_title)?;
        buf.extend_from_slice(&[self.slot_count]);
        if self.inventory_type == "EntityHorse" {
            if let Some(eid) = self.entity_id {
                buf.extend_from_slice(&eid.to_be_bytes());
            } else {
                buf.extend_from_slice(&0i32.to_be_bytes());
            }
        }
        Ok(())
    }
}

/// Clientbound Close Window packet (0x2E).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCloseWindowPacket {
    pub window_id: u8,
}

impl ClientboundCloseWindowPacket {
    pub const PACKET_ID: i32 = 0x2E;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 1 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 1,
                available: buf.remaining(),
            });
        }
        let window_id = buf.get_u8();
        Ok(Self { window_id })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[self.window_id]);
        Ok(())
    }
}

/// Clientbound Window Property packet (0x31).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundWindowPropertyPacket {
    pub window_id: u8,
    pub property: i16,
    pub value: i16,
}

impl ClientboundWindowPropertyPacket {
    pub const PACKET_ID: i32 = 0x31;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        if buf.remaining() < 5 {
            return Err(ProtocolError::BufferUnderflow {
                needed: 5,
                available: buf.remaining(),
            });
        }
        let window_id = buf.get_u8();
        let property = buf.get_i16();
        let value = buf.get_i16();
        Ok(Self {
            window_id,
            property,
            value,
        })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        buf.extend_from_slice(&[self.window_id]);
        buf.extend_from_slice(&self.property.to_be_bytes());
        buf.extend_from_slice(&self.value.to_be_bytes());
        Ok(())
    }
}

/// Clientbound Sign Editor Open packet (0x36).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSignEditorOpenPacket {
    pub position: Position,
}

impl ClientboundSignEditorOpenPacket {
    pub const PACKET_ID: i32 = 0x36;

    pub fn decode(buf: &mut Bytes) -> Result<Self, ProtocolError> {
        let position = Position::read(buf)?;
        Ok(Self { position })
    }

    pub fn encode(&self, buf: &mut BytesMut) -> Result<(), ProtocolError> {
        self.position.write(buf);
        Ok(())
    }
}

