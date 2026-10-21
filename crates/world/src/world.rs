use crate::block::BlockState;
use crate::block_entity::{
    BannerBlockEntity, BeaconBlockEntity, BlockEntity, ChestBlockEntity, ChestType, DoubleChestType,
    EnchantmentTableBlockEntity, EndPortalBlockEntity, MobSpawnerBlockEntity, PistonBlockEntity,
    SignBlockEntity, SkullBlockEntity,
};
use crate::chunk::{ChunkColumn, WorldError};
use crate::entity::EntityTracker;
use crate::map::MapData;
use hashbrown::HashMap;

/// Manages the voxel world state, loaded chunks, and global dimensions.
#[derive(Debug, Default)]
pub struct World {
    pub chunks: HashMap<(i32, i32), ChunkColumn>,
    pub entity_tracker: EntityTracker,
    pub block_entities: HashMap<(i32, i32, i32), BlockEntity>,
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
            block_entities: HashMap::new(),
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
        self.update_block_entity_on_set(x, y, z, state);
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

        // Scan newly loaded sections for block entities
        let mut new_bes = Vec::new();
        for sec_idx in 0..16 {
            if (bit_mask & (1 << sec_idx)) != 0 {
                if let Some(sec) = &chunk.sections[sec_idx] {
                    let sec_y_base = (sec_idx as i32) * 16;
                    for ly in 0..16 {
                        for lz in 0..16 {
                            for lx in 0..16 {
                                let block = sec.block_at(lx, ly, lz);
                                let id = block.id();
                                if matches!(id, 54 | 130 | 146 | 63 | 68 | 176 | 177 | 144 | 116 | 138 | 119 | 52 | 36) {
                                    let wx = x * 16 + lx as i32;
                                    let wy = sec_y_base + ly as i32;
                                    let wz = z * 16 + lz as i32;
                                    if !self.block_entities.contains_key(&(wx, wy, wz)) {
                                        new_bes.push((wx, wy, wz, block));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for (wx, wy, wz, block) in new_bes {
            self.update_block_entity_on_set(wx, wy, wz, block);
        }

        Ok(())
    }

    #[inline]
    pub fn unload_chunk(&mut self, x: i32, z: i32) -> Option<ChunkColumn> {
        self.block_entities
            .retain(|&(bx, _, bz), _| (bx >> 4) != x || (bz >> 4) != z);
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

    #[inline]
    pub fn get_block_entity(&self, x: i32, y: i32, z: i32) -> Option<&BlockEntity> {
        self.block_entities.get(&(x, y, z))
    }

    #[inline]
    pub fn get_block_entity_mut(&mut self, x: i32, y: i32, z: i32) -> Option<&mut BlockEntity> {
        self.block_entities.get_mut(&(x, y, z))
    }

    #[inline]
    pub fn set_block_entity(&mut self, x: i32, y: i32, z: i32, entity: BlockEntity) {
        self.block_entities.insert((x, y, z), entity);
    }

    #[inline]
    pub fn remove_block_entity(&mut self, x: i32, y: i32, z: i32) -> Option<BlockEntity> {
        self.block_entities.remove(&(x, y, z))
    }

    /// Automatically initialises or removes block entities when a block changes.
    pub fn update_block_entity_on_set(&mut self, x: i32, y: i32, z: i32, state: BlockState) {
        let id = state.id();
        let meta = state.meta();
        match id {
            54 | 146 | 130 => {
                let chest_type = match id {
                    130 => ChestType::Ender,
                    146 => ChestType::Trapped,
                    _ => ChestType::Normal,
                };
                let facing = if (2..=5).contains(&meta) { meta } else { 2 };
                let mut chest = ChestBlockEntity::new(chest_type, facing, DoubleChestType::Single);
                if chest_type != ChestType::Ender {
                    for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let nx = x + dx;
                        let nz = z + dz;
                        let n_state = self.block_at(nx, y, nz);
                        if n_state.id() == id {
                            match facing {
                                2 => {
                                    if dx < 0 {
                                        chest.double_type = DoubleChestType::Right;
                                    } else if dx > 0 {
                                        chest.double_type = DoubleChestType::Left;
                                    }
                                }
                                3 => {
                                    if dx < 0 {
                                        chest.double_type = DoubleChestType::Left;
                                    } else if dx > 0 {
                                        chest.double_type = DoubleChestType::Right;
                                    }
                                }
                                4 => {
                                    if dz < 0 {
                                        chest.double_type = DoubleChestType::Left;
                                    } else if dz > 0 {
                                        chest.double_type = DoubleChestType::Right;
                                    }
                                }
                                _ => {
                                    if dz < 0 {
                                        chest.double_type = DoubleChestType::Right;
                                    } else if dz > 0 {
                                        chest.double_type = DoubleChestType::Left;
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
                self.block_entities
                    .insert((x, y, z), BlockEntity::Chest(chest));
            }
            63 => {
                self.block_entities
                    .insert((x, y, z), BlockEntity::Sign(SignBlockEntity::new(false, meta)));
            }
            68 => {
                self.block_entities
                    .insert((x, y, z), BlockEntity::Sign(SignBlockEntity::new(true, meta)));
            }
            176 => {
                self.block_entities.insert(
                    (x, y, z),
                    BlockEntity::Banner(BannerBlockEntity::new(meta, false, meta)),
                );
            }
            177 => {
                self.block_entities.insert(
                    (x, y, z),
                    BlockEntity::Banner(BannerBlockEntity::new(meta, true, meta)),
                );
            }
            144 => {
                self.block_entities
                    .insert((x, y, z), BlockEntity::Skull(SkullBlockEntity::new(0, 0, meta)));
            }
            116 => {
                self.block_entities.insert(
                    (x, y, z),
                    BlockEntity::EnchantmentTable(EnchantmentTableBlockEntity::new()),
                );
            }
            138 => {
                let mut beacon = BeaconBlockEntity::new();
                beacon.recalculate_beam(y, |by| {
                    let b = self.block_at(x, by, z);
                    (b.id(), b.meta())
                });
                self.block_entities
                    .insert((x, y, z), BlockEntity::Beacon(beacon));
            }
            119 => {
                self.block_entities.insert(
                    (x, y, z),
                    BlockEntity::EndPortal(EndPortalBlockEntity::default()),
                );
            }
            52 => {
                self.block_entities.insert(
                    (x, y, z),
                    BlockEntity::MobSpawner(MobSpawnerBlockEntity::default()),
                );
            }
            36 => {
                self.block_entities.insert(
                    (x, y, z),
                    BlockEntity::Piston(PistonBlockEntity::new(
                        BlockState::AIR,
                        meta & 7,
                        (meta & 8) != 0,
                    )),
                );
            }
            0 => {
                self.block_entities.remove(&(x, y, z));
            }
            _ => {}
        }
    }

    /// 20 TPS physics tick for all dynamic block entities.
    pub fn tick_block_entities(&mut self, player_pos: glam::Vec3) {
        for (&pos, be) in self.block_entities.iter_mut() {
            match be {
                BlockEntity::Chest(c) => c.tick(),
                BlockEntity::EnchantmentTable(e) => {
                    e.tick(
                        Some(player_pos),
                        glam::Vec3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32),
                    );
                }
                BlockEntity::MobSpawner(s) => s.tick(),
                BlockEntity::Piston(p) => p.tick(),
                _ => {}
            }
        }
    }

    /// Checks whether the block entity at (x, y, z) is currently animating (e.g. chest opening/closing).
    pub fn is_block_entity_animating(&self, x: i32, y: i32, z: i32) -> bool {
        if let Some(be) = self.block_entities.get(&(x, y, z)) {
            match be {
                BlockEntity::Chest(c) => !c.is_static_closed(),
                BlockEntity::Skull(_) => false,
                _ => true,
            }
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.entity_tracker.clear();
        self.block_entities.clear();
        self.maps.clear();
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
