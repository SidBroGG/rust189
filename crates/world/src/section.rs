use crate::block::BlockState;
use crate::palette::PalettedContainer;

pub const SECTION_WIDTH: usize = 16;
pub const SECTION_HEIGHT: usize = 16;
pub const SECTION_DEPTH: usize = 16;
pub const SECTION_BLOCK_COUNT: usize = SECTION_WIDTH * SECTION_HEIGHT * SECTION_DEPTH; // 4096
pub const NIBBLE_ARRAY_SIZE: usize = SECTION_BLOCK_COUNT / 2; // 2048

/// Represents a 16x16x16 sub-chunk section backed by an adaptive PalettedContainer.
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkSection {
    pub blocks: PalettedContainer,
    pub block_light: Box<[u8; NIBBLE_ARRAY_SIZE]>,
    pub sky_light: Option<Box<[u8; NIBBLE_ARRAY_SIZE]>>,
    non_air_count: u16,
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self::new(true)
    }
}

impl ChunkSection {
    pub fn new(has_sky_light: bool) -> Self {
        let sky = if has_sky_light {
            Some(vec![0xFF; NIBBLE_ARRAY_SIZE].into_boxed_slice().try_into().unwrap())
        } else {
            None
        };

        Self {
            blocks: PalettedContainer::new_empty(),
            block_light: vec![0; NIBBLE_ARRAY_SIZE].into_boxed_slice().try_into().unwrap(),
            sky_light: sky,
            non_air_count: 0,
        }
    }

    #[inline]
    pub const fn coord_index(x: usize, y: usize, z: usize) -> usize {
        (y << 8) | (z << 4) | x
    }

    #[inline]
    pub fn block_at(&self, x: usize, y: usize, z: usize) -> BlockState {
        if x >= SECTION_WIDTH || y >= SECTION_HEIGHT || z >= SECTION_DEPTH {
            return BlockState::AIR;
        }
        self.blocks.get(x, y, z)
    }

    #[inline]
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, state: BlockState) {
        if x >= SECTION_WIDTH || y >= SECTION_HEIGHT || z >= SECTION_DEPTH {
            return;
        }
        let old = self.blocks.get(x, y, z);
        if old == state {
            return;
        }
        if old.is_air() && !state.is_air() {
            self.non_air_count += 1;
        } else if !old.is_air() && state.is_air() && self.non_air_count > 0 {
            self.non_air_count -= 1;
        }
        self.blocks.set(x, y, z, state);
    }

    #[inline]
    pub fn block_light_at(&self, x: usize, y: usize, z: usize) -> u8 {
        if x >= SECTION_WIDTH || y >= SECTION_HEIGHT || z >= SECTION_DEPTH {
            return 0;
        }
        let idx = Self::coord_index(x, y, z);
        get_nibble(&self.block_light, idx)
    }

    #[inline]
    pub fn set_block_light(&mut self, x: usize, y: usize, z: usize, light: u8) {
        if x >= SECTION_WIDTH || y >= SECTION_HEIGHT || z >= SECTION_DEPTH {
            return;
        }
        let idx = Self::coord_index(x, y, z);
        set_nibble(&mut self.block_light, idx, light);
    }

    #[inline]
    pub fn sky_light_at(&self, x: usize, y: usize, z: usize) -> u8 {
        if x >= SECTION_WIDTH || y >= SECTION_HEIGHT || z >= SECTION_DEPTH {
            return 15;
        }
        match &self.sky_light {
            Some(sky) => get_nibble(sky, Self::coord_index(x, y, z)),
            None => 0,
        }
    }

    #[inline]
    pub fn set_sky_light(&mut self, x: usize, y: usize, z: usize, light: u8) {
        if x >= SECTION_WIDTH || y >= SECTION_HEIGHT || z >= SECTION_DEPTH {
            return;
        }
        let idx = Self::coord_index(x, y, z);
        if let Some(sky) = &mut self.sky_light {
            set_nibble(sky, idx, light);
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.non_air_count == 0
    }

    #[inline]
    pub fn non_air_count(&self) -> usize {
        self.non_air_count as usize
    }

    pub fn recalculate_non_air_count(&mut self) {
        self.non_air_count = self.blocks.count_non_air() as u16;
    }

    /// Loads 4096 blocks from little-endian byte slice into adaptive paletted container.
    pub fn load_blocks_from_raw_slice(&mut self, slice: &[u8]) {
        let (blocks, non_air) = PalettedContainer::from_raw_slice_counted(slice);
        self.blocks = blocks;
        self.non_air_count = non_air as u16;
    }

    /// Writes blocks to raw little-endian byte buffer.
    pub fn write_blocks_to_raw(&self, buf: &mut Vec<u8>) {
        self.blocks.write_linear_bytes(buf);
    }

    /// Fills 64-word bitmask of opaque blocks for BFS occlusion culling.
    #[inline]
    pub fn fill_opaque_bits(&self, opaque_bits: &mut [u64; 64]) -> usize {
        self.blocks.fill_opaque_bits(opaque_bits)
    }

    /// Compacts palette storage to free unused memory.
    pub fn compact(&mut self) {
        self.blocks.compact();
    }

    /// Total heap memory usage in bytes for this section.
    pub fn heap_memory_usage(&self) -> usize {
        let mut total = self.blocks.heap_memory_usage();
        total += NIBBLE_ARRAY_SIZE; // block_light
        if self.sky_light.is_some() {
            total += NIBBLE_ARRAY_SIZE;
        }
        total
    }
}

#[inline]
pub fn get_nibble(slice: &[u8; NIBBLE_ARRAY_SIZE], index: usize) -> u8 {
    let byte = slice[index >> 1];
    if (index & 1) == 0 {
        byte & 0x0F
    } else {
        (byte >> 4) & 0x0F
    }
}

#[inline]
pub fn set_nibble(slice: &mut [u8; NIBBLE_ARRAY_SIZE], index: usize, value: u8) {
    let byte_idx = index >> 1;
    let val = value & 0x0F;
    if (index & 1) == 0 {
        slice[byte_idx] = (slice[byte_idx] & 0xF0) | val;
    } else {
        slice[byte_idx] = (slice[byte_idx] & 0x0F) | (val << 4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nibble_packing() {
        let mut nibbles = Box::new([0u8; NIBBLE_ARRAY_SIZE]);
        set_nibble(&mut nibbles, 0, 7);
        set_nibble(&mut nibbles, 1, 14);
        assert_eq!(get_nibble(&nibbles, 0), 7);
        assert_eq!(get_nibble(&nibbles, 1), 14);
        assert_eq!(nibbles[0], (14 << 4) | 7);
    }

    #[test]
    fn test_section_block_access() {
        let mut section = ChunkSection::new(true);
        assert!(section.is_empty());
        section.set_block(3, 4, 5, BlockState::new(1, 0));
        assert_eq!(section.block_at(3, 4, 5), BlockState::new(1, 0));
        assert_eq!(section.non_air_count(), 1);
        assert!(!section.is_empty());

        section.set_block(3, 4, 5, BlockState::AIR);
        assert_eq!(section.non_air_count(), 0);
        assert!(section.is_empty());
    }
}
