use crate::block::BlockState;
use crate::section::{ChunkSection, NIBBLE_ARRAY_SIZE, SECTION_BLOCK_COUNT};
use thiserror::Error;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_DEPTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;
pub const SECTIONS_PER_CHUNK: usize = 16;
pub const BIOME_ARRAY_SIZE: usize = 256;

#[derive(Error, Debug)]
pub enum WorldError {
    #[error("Unexpected end of chunk buffer: required {expected} bytes, available {actual} bytes")]
    BufferUnderflow { expected: usize, actual: usize },
    #[error("Invalid coordinate: x={0}, y={1}, z={2}")]
    InvalidCoordinate(usize, usize, usize),
}

/// A 16x256x16 column of voxels in the world.
#[derive(Debug, Clone)]
pub struct ChunkColumn {
    pub x: i32,
    pub z: i32,
    pub sections: [Option<Box<ChunkSection>>; SECTIONS_PER_CHUNK],
    pub biomes: [u8; BIOME_ARRAY_SIZE],
    pub has_sky_light: bool,
}

impl ChunkColumn {
    pub fn new(x: i32, z: i32, has_sky_light: bool) -> Self {
        Self {
            x,
            z,
            sections: Default::default(),
            biomes: [0; BIOME_ARRAY_SIZE],
            has_sky_light,
        }
    }

    #[inline]
    pub fn block_at(&self, rel_x: usize, y: usize, rel_z: usize) -> BlockState {
        if rel_x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || rel_z >= CHUNK_DEPTH {
            return BlockState::AIR;
        }
        let sec_idx = y >> 4;
        let sec_y = y & 15;
        match &self.sections[sec_idx] {
            Some(sec) => sec.block_at(rel_x, sec_y, rel_z),
            None => BlockState::AIR,
        }
    }

    #[inline]
    pub fn set_block(&mut self, rel_x: usize, y: usize, rel_z: usize, state: BlockState) {
        if rel_x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || rel_z >= CHUNK_DEPTH {
            return;
        }
        let sec_idx = y >> 4;
        let sec_y = y & 15;
        if self.sections[sec_idx].is_none() {
            if state.is_air() {
                return;
            }
            self.sections[sec_idx] = Some(Box::new(ChunkSection::new(self.has_sky_light)));
        }
        if let Some(sec) = &mut self.sections[sec_idx] {
            sec.set_block(rel_x, sec_y, rel_z, state);
        }
    }

    #[inline]
    pub fn block_light_at(&self, rel_x: usize, y: usize, rel_z: usize) -> u8 {
        if rel_x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || rel_z >= CHUNK_DEPTH {
            return 0;
        }
        let sec_idx = y >> 4;
        let sec_y = y & 15;
        match &self.sections[sec_idx] {
            Some(sec) => sec.block_light_at(rel_x, sec_y, rel_z),
            None => 0,
        }
    }

    #[inline]
    pub fn set_block_light(&mut self, rel_x: usize, y: usize, rel_z: usize, light: u8) {
        if rel_x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || rel_z >= CHUNK_DEPTH {
            return;
        }
        let sec_idx = y >> 4;
        let sec_y = y & 15;
        if self.sections[sec_idx].is_none() {
            self.sections[sec_idx] = Some(Box::new(ChunkSection::new(self.has_sky_light)));
        }
        if let Some(sec) = &mut self.sections[sec_idx] {
            sec.set_block_light(rel_x, sec_y, rel_z, light);
        }
    }

    #[inline]
    pub fn sky_light_at(&self, rel_x: usize, y: usize, rel_z: usize) -> u8 {
        if rel_x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || rel_z >= CHUNK_DEPTH {
            return 15;
        }
        let sec_idx = y >> 4;
        let sec_y = y & 15;
        match &self.sections[sec_idx] {
            Some(sec) => sec.sky_light_at(rel_x, sec_y, rel_z),
            None => 15,
        }
    }

    #[inline]
    pub fn set_sky_light(&mut self, rel_x: usize, y: usize, rel_z: usize, light: u8) {
        if rel_x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || rel_z >= CHUNK_DEPTH {
            return;
        }
        let sec_idx = y >> 4;
        let sec_y = y & 15;
        if self.sections[sec_idx].is_none() {
            self.sections[sec_idx] = Some(Box::new(ChunkSection::new(self.has_sky_light)));
        }
        if let Some(sec) = &mut self.sections[sec_idx] {
            sec.set_sky_light(rel_x, sec_y, rel_z, light);
        }
    }

    #[inline]
    pub fn biome_at(&self, rel_x: usize, rel_z: usize) -> u8 {
        if rel_x >= CHUNK_WIDTH || rel_z >= CHUNK_DEPTH {
            return 0;
        }
        self.biomes[(rel_z << 4) | rel_x]
    }

    #[inline]
    pub fn set_biome(&mut self, rel_x: usize, rel_z: usize, biome: u8) {
        if rel_x >= CHUNK_WIDTH || rel_z >= CHUNK_DEPTH {
            return;
        }
        self.biomes[(rel_z << 4) | rel_x] = biome;
    }

    /// Unpacks raw voxel data matching Protocol 47 specification.
    /// Returns the number of bytes consumed from `data`.
    pub fn load_from_raw_payload(
        &mut self,
        data: &[u8],
        bit_mask: u16,
        ground_up: bool,
        has_sky_light: bool,
    ) -> Result<usize, WorldError> {
        let mut offset = 0;
        self.has_sky_light = has_sky_light;

        // 1. Block IDs and Metadata (4096 u16 per section = 8192 bytes)
        for i in 0..SECTIONS_PER_CHUNK {
            if (bit_mask & (1 << i)) != 0 {
                if self.sections[i].is_none() {
                    self.sections[i] = Some(Box::new(ChunkSection::new(has_sky_light)));
                }

                let sec = self.sections[i].as_mut().unwrap();
                let block_bytes = SECTION_BLOCK_COUNT * 2;
                if offset + block_bytes > data.len() {
                    return Err(WorldError::BufferUnderflow {
                        expected: offset + block_bytes,
                        actual: data.len(),
                    });
                }

                let slice = &data[offset..offset + block_bytes];
                for k in 0..SECTION_BLOCK_COUNT {
                    let low = slice[k * 2] as u16;
                    let high = slice[k * 2 + 1] as u16;
                    sec.blocks[k] = (high << 8) | low;
                }
                sec.recalculate_non_air_count();
                offset += block_bytes;
            } else if ground_up {
                self.sections[i] = None;
            }
        }

        // 2. Block light (2048 bytes per section)
        for i in 0..SECTIONS_PER_CHUNK {
            if (bit_mask & (1 << i)) != 0 {
                if let Some(sec) = &mut self.sections[i] {
                    if offset + NIBBLE_ARRAY_SIZE > data.len() {
                        return Err(WorldError::BufferUnderflow {
                            expected: offset + NIBBLE_ARRAY_SIZE,
                            actual: data.len(),
                        });
                    }
                    sec.block_light.copy_from_slice(&data[offset..offset + NIBBLE_ARRAY_SIZE]);
                    offset += NIBBLE_ARRAY_SIZE;
                }
            }
        }

        // 3. Sky light (2048 bytes per section, if present)
        if has_sky_light {
            for i in 0..SECTIONS_PER_CHUNK {
                if (bit_mask & (1 << i)) != 0 {
                    if let Some(sec) = &mut self.sections[i] {
                        if offset + NIBBLE_ARRAY_SIZE > data.len() {
                            return Err(WorldError::BufferUnderflow {
                                expected: offset + NIBBLE_ARRAY_SIZE,
                                actual: data.len(),
                            });
                        }
                        if sec.sky_light.is_none() {
                            sec.sky_light = Some(vec![0; NIBBLE_ARRAY_SIZE].into_boxed_slice().try_into().unwrap());
                        }
                        if let Some(sky) = &mut sec.sky_light {
                            sky.copy_from_slice(&data[offset..offset + NIBBLE_ARRAY_SIZE]);
                        }
                        offset += NIBBLE_ARRAY_SIZE;
                    }
                }
            }
        }

        // 4. Biomes (256 bytes)
        if ground_up {
            if offset + BIOME_ARRAY_SIZE > data.len() {
                return Err(WorldError::BufferUnderflow {
                    expected: offset + BIOME_ARRAY_SIZE,
                    actual: data.len(),
                });
            }
            self.biomes.copy_from_slice(&data[offset..offset + BIOME_ARRAY_SIZE]);
            offset += BIOME_ARRAY_SIZE;
        }

        Ok(offset)
    }

    /// Serializes chunk data into raw binary format matching Protocol 47.
    /// Useful for unit testing, round-trips, and cache verification.
    pub fn serialize_to_raw_payload(
        &self,
        bit_mask: u16,
        ground_up: bool,
        has_sky_light: bool,
    ) -> Vec<u8> {
        let mut buf = Vec::new();

        // 1. Blocks
        for i in 0..SECTIONS_PER_CHUNK {
            if (bit_mask & (1 << i)) != 0 {
                if let Some(sec) = &self.sections[i] {
                    for &b in sec.blocks.iter() {
                        buf.push((b & 0xFF) as u8);
                        buf.push((b >> 8) as u8);
                    }
                } else {
                    buf.resize(buf.len() + SECTION_BLOCK_COUNT * 2, 0);
                }
            }
        }

        // 2. Block light
        for i in 0..SECTIONS_PER_CHUNK {
            if (bit_mask & (1 << i)) != 0 {
                if let Some(sec) = &self.sections[i] {
                    buf.extend_from_slice(&*sec.block_light);
                } else {
                    buf.resize(buf.len() + NIBBLE_ARRAY_SIZE, 0);
                }
            }
        }

        // 3. Sky light
        if has_sky_light {
            for i in 0..SECTIONS_PER_CHUNK {
                if (bit_mask & (1 << i)) != 0 {
                    if let Some(sec) = &self.sections[i] {
                        if let Some(sky) = &sec.sky_light {
                            buf.extend_from_slice(&**sky);
                        } else {
                            buf.resize(buf.len() + NIBBLE_ARRAY_SIZE, 0xFF);
                        }
                    } else {
                        buf.resize(buf.len() + NIBBLE_ARRAY_SIZE, 0xFF);
                    }
                }
            }
        }

        // 4. Biomes
        if ground_up {
            buf.extend_from_slice(&self.biomes);
        }

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_payload_roundtrip() {
        let mut chunk = ChunkColumn::new(0, 0, true);
        chunk.set_block(0, 0, 0, BlockState::new(1, 0)); // Stone at (0, 0, 0)
        chunk.set_block(15, 15, 15, BlockState::new(2, 0)); // Grass at (15, 15, 15)
        chunk.set_biome(0, 0, 4); // Forest biome

        let bit_mask = 1; // Section 0 only
        let raw = chunk.serialize_to_raw_payload(bit_mask, true, true);

        let mut loaded = ChunkColumn::new(0, 0, true);
        let consumed = loaded
            .load_from_raw_payload(&raw, bit_mask, true, true)
            .expect("Unpacking failed");

        assert_eq!(consumed, raw.len());
        assert_eq!(loaded.block_at(0, 0, 0), BlockState::new(1, 0));
        assert_eq!(loaded.block_at(15, 15, 15), BlockState::new(2, 0));
        assert_eq!(loaded.biome_at(0, 0), 4);
    }
}
