use crate::block::BlockState;
use crate::chunk::{ChunkColumn, WorldError};
use crate::entity::EntityTracker;
use crate::map::MapData;
use hashbrown::HashMap;

/// Manages the voxel world state, loaded chunks, and global dimensions.
#[derive(Debug, Default)]
pub struct World {
    pub chunks: HashMap<(i32, i32), ChunkColumn>,
    pub entity_tracker: EntityTracker,
    pub maps: HashMap<i32, MapData>,
    pub dimension: i8, // 0 = Overworld, -1 = Nether, 1 = End
    pub time: i64,
    pub age: i64,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            entity_tracker: EntityTracker::new(),
            maps: HashMap::new(),
            dimension: 0,
            time: 0,
            age: 0,
        }
    }

    #[inline]
    pub fn has_sky_light(&self) -> bool {
        self.dimension == 0
    }

    #[inline]
    pub fn chunk_coords(x: i32, z: i32) -> (i32, i32) {
        (x >> 4, z >> 4)
    }

    #[inline]
    pub fn local_coords(x: i32, z: i32) -> (usize, usize) {
        ((x.rem_euclid(16)) as usize, (z.rem_euclid(16)) as usize)
    }

    #[inline]
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> BlockState {
        if !(0..=255).contains(&y) {
            return BlockState::AIR;
        }
        let chunk_pos = Self::chunk_coords(x, z);
        let (rel_x, rel_z) = Self::local_coords(x, z);
        match self.chunks.get(&chunk_pos) {
            Some(col) => col.block_at(rel_x, y as usize, rel_z),
            None => BlockState::AIR,
        }
    }

    #[inline]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockState {
        self.block_at(x, y, z)
    }

    #[inline]
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, state: BlockState) {
        if !(0..=255).contains(&y) {
            return;
        }
        let chunk_pos = Self::chunk_coords(x, z);
        let (rel_x, rel_z) = Self::local_coords(x, z);
        let has_sky = self.has_sky_light();
        let chunk = self
            .chunks
            .entry(chunk_pos)
            .or_insert_with(|| ChunkColumn::new(chunk_pos.0, chunk_pos.1, has_sky));
        chunk.set_block(rel_x, y as usize, rel_z, state);
    }

    #[inline]
    pub fn block_light_at(&self, x: i32, y: i32, z: i32) -> u8 {
        if !(0..=255).contains(&y) {
            return 0;
        }
        let chunk_pos = Self::chunk_coords(x, z);
        let (rel_x, rel_z) = Self::local_coords(x, z);
        match self.chunks.get(&chunk_pos) {
            Some(col) => col.block_light_at(rel_x, y as usize, rel_z),
            None => 0,
        }
    }

    #[inline]
    pub fn sky_light_at(&self, x: i32, y: i32, z: i32) -> u8 {
        if !(0..=255).contains(&y) {
            return 15;
        }
        let chunk_pos = Self::chunk_coords(x, z);
        let (rel_x, rel_z) = Self::local_coords(x, z);
        match self.chunks.get(&chunk_pos) {
            Some(col) => col.sky_light_at(rel_x, y as usize, rel_z),
            None => 15,
        }
    }

    /// Handles single chunk data updates (0x21 ChunkData packet).
    /// If ground_up is true and bit_mask is 0, this unloads the chunk column.
    pub fn load_chunk_data(
        &mut self,
        x: i32,
        z: i32,
        ground_up: bool,
        bit_mask: u16,
        data: &[u8],
    ) -> Result<(), WorldError> {
        if ground_up && bit_mask == 0 {
            self.unload_chunk(x, z);
            return Ok(());
        }

        let has_sky = self.has_sky_light();
        let chunk = self
            .chunks
            .entry((x, z))
            .or_insert_with(|| ChunkColumn::new(x, z, has_sky));

        chunk.load_from_raw_payload(data, bit_mask, ground_up, has_sky)?;
        Ok(())
    }

    #[inline]
    pub fn unload_chunk(&mut self, x: i32, z: i32) -> Option<ChunkColumn> {
        self.chunks.remove(&(x, z))
    }

    #[inline]
    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&ChunkColumn> {
        self.chunks.get(&(x, z))
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, x: i32, z: i32) -> Option<&mut ChunkColumn> {
        self.chunks.get_mut(&(x, z))
    }

    #[inline]
    pub fn has_chunk(&self, x: i32, z: i32) -> bool {
        self.chunks.contains_key(&(x, z))
    }

    #[inline]
    pub fn is_chunk_loaded(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.has_chunk(chunk_x, chunk_z)
    }

    #[inline]
    pub fn is_block_loaded(&self, x: i32, z: i32) -> bool {
        let (cx, cz) = Self::chunk_coords(x, z);
        self.is_chunk_loaded(cx, cz)
    }

    #[inline]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.entity_tracker.clear();
        self.time = 0;
        self.age = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_block_placement() {
        let mut world = World::new();
        assert_eq!(world.block_at(10, 64, -5), BlockState::AIR);

        world.set_block(10, 64, -5, BlockState::new(3, 0)); // Dirt
        assert_eq!(world.block_at(10, 64, -5), BlockState::new(3, 0));
        assert_eq!(world.chunk_count(), 1);

        // Check unload
        let chunk_pos = World::chunk_coords(10, -5);
        world.unload_chunk(chunk_pos.0, chunk_pos.1);
        assert_eq!(world.chunk_count(), 0);
        assert_eq!(world.block_at(10, 64, -5), BlockState::AIR);
    }
}
