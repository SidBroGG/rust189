use crate::block::BlockState;
use crate::entity::EntityTracker;
use crate::physics::aabb::AABB;
use crate::physics::collision::get_block_collision_boxes;
use crate::world::World;
use glam::DVec3;
use hashbrown::HashMap;
use protocol::BlockFace;

/// Result of a 3D voxel DDA raycast against blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockRayHit {
    pub block_pos: (i32, i32, i32),
    pub face: BlockFace,
    pub hit_pos: DVec3,
    pub distance: f64,
    /// Offset within the block [0.0, 1.0]
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_z: f32,
}

/// Result of raycasting against entities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityRayHit {
    pub entity_id: i32,
    pub hit_pos: DVec3,
    pub distance: f64,
}

/// Combined target of player crosshair.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetHit {
    Block(BlockRayHit),
    Entity(EntityRayHit),
    Miss,
}

/// Default block reach distances (MCP 1.8.9 PlayerControllerMP).
pub const SURVIVAL_BLOCK_REACH: f64 = 4.5;
pub const CREATIVE_BLOCK_REACH: f64 = 5.0;

/// Default entity reach distances (MCP 1.8.9 EntityRenderer.getMouseOver).
pub const SURVIVAL_ENTITY_REACH: f64 = 3.0;
pub const CREATIVE_ENTITY_REACH: f64 = 6.0;

/// Performs the Amanatides-Woo 3D Voxel DDA traversal through the voxel grid.
/// Never skips any block along the ray.
pub fn ray_trace_voxels(
    world: &World,
    start: DVec3,
    dir: DVec3,
    max_distance: f64,
) -> Option<BlockRayHit> {
    if max_distance <= 0.0 || dir.length_squared() < 1e-12 {
        return None;
    }
    let unit_dir = dir.normalize();

    let mut current_x = start.x.floor() as i32;
    let mut current_y = start.y.floor() as i32;
    let mut current_z = start.z.floor() as i32;

    let step_x = if unit_dir.x > 0.0 { 1 } else if unit_dir.x < 0.0 { -1 } else { 0 };
    let step_y = if unit_dir.y > 0.0 { 1 } else if unit_dir.y < 0.0 { -1 } else { 0 };
    let step_z = if unit_dir.z > 0.0 { 1 } else if unit_dir.z < 0.0 { -1 } else { 0 };

    let t_delta_x = if unit_dir.x != 0.0 { (1.0 / unit_dir.x).abs() } else { f64::INFINITY };
    let t_delta_y = if unit_dir.y != 0.0 { (1.0 / unit_dir.y).abs() } else { f64::INFINITY };
    let t_delta_z = if unit_dir.z != 0.0 { (1.0 / unit_dir.z).abs() } else { f64::INFINITY };

    let mut t_max_x = if unit_dir.x > 0.0 {
        ((current_x as f64 + 1.0) - start.x) * t_delta_x
    } else if unit_dir.x < 0.0 {
        (start.x - current_x as f64) * t_delta_x
    } else {
        f64::INFINITY
    };

    let mut t_max_y = if unit_dir.y > 0.0 {
        ((current_y as f64 + 1.0) - start.y) * t_delta_y
    } else if unit_dir.y < 0.0 {
        (start.y - current_y as f64) * t_delta_y
    } else {
        f64::INFINITY
    };

    let mut t_max_z = if unit_dir.z > 0.0 {
        ((current_z as f64 + 1.0) - start.z) * t_delta_z
    } else if unit_dir.z < 0.0 {
        (start.z - current_z as f64) * t_delta_z
    } else {
        f64::INFINITY
    };

    let mut block_boxes = Vec::with_capacity(6);
    let mut last_face = if unit_dir.y < 0.0 { BlockFace::Up } else { BlockFace::Down };

    let mut steps = 0;
    let max_steps = (max_distance * 3.0).ceil() as i32 + 8;

    while steps < max_steps {
        steps += 1;

        if current_y >= 0 && current_y < 256 {
            let state = world.block_at(current_x, current_y, current_z);
            if !state.is_air() {
                block_boxes.clear();
                let block_mask = AABB::new(
                    current_x as f64 - 0.01,
                    current_y as f64 - 0.01,
                    current_z as f64 - 0.01,
                    current_x as f64 + 1.01,
                    current_y as f64 + 1.01,
                    current_z as f64 + 1.01,
                );
                get_block_collision_boxes(world, current_x, current_y, current_z, state, &block_mask, &mut block_boxes);

                // If block has no collision boxes (e.g. torch, redstone, flower), use selection box 0..1
                if block_boxes.is_empty() && can_select_block(state.id()) {
                    block_boxes.push(get_block_selection_box(current_x, current_y, current_z, state));
                }

                let mut closest_box_hit: Option<(f64, DVec3)> = None;
                for bb in &block_boxes {
                    if let Some((t, normal)) = bb.ray_trace(start, unit_dir, max_distance) {
                        if t <= max_distance {
                            if let Some((prev_t, _)) = closest_box_hit {
                                if t < prev_t {
                                    closest_box_hit = Some((t, normal));
                                }
                            } else {
                                closest_box_hit = Some((t, normal));
                            }
                        }
                    }
                }

                if let Some((t, normal)) = closest_box_hit {
                    let hit_pos = start + unit_dir * t;
                    let face = normal_to_face(normal).unwrap_or(last_face);
                    let cursor_x = ((hit_pos.x - current_x as f64) as f32).clamp(0.0, 1.0);
                    let cursor_y = ((hit_pos.y - current_y as f64) as f32).clamp(0.0, 1.0);
                    let cursor_z = ((hit_pos.z - current_z as f64) as f32).clamp(0.0, 1.0);

                    return Some(BlockRayHit {
                        block_pos: (current_x, current_y, current_z),
                        face,
                        hit_pos,
                        distance: t,
                        cursor_x,
                        cursor_y,
                        cursor_z,
                    });
                }
            }
        }

        // Advance Amanatides-Woo DDA step
        if t_max_x < t_max_y && t_max_x < t_max_z {
            if t_max_x > max_distance {
                break;
            }
            current_x += step_x;
            last_face = if step_x > 0 { BlockFace::West } else { BlockFace::East };
            t_max_x += t_delta_x;
        } else if t_max_y < t_max_z {
            if t_max_y > max_distance {
                break;
            }
            current_y += step_y;
            last_face = if step_y > 0 { BlockFace::Down } else { BlockFace::Up };
            t_max_y += t_delta_y;
        } else {
            if t_max_z > max_distance {
                break;
            }
            current_z += step_z;
            last_face = if step_z > 0 { BlockFace::North } else { BlockFace::South };
            t_max_z += t_delta_z;
        }
    }

    None
}

/// Normal to BlockFace mapping.
#[inline]
pub fn normal_to_face(normal: DVec3) -> Option<BlockFace> {
    if normal.y > 0.5 {
        Some(BlockFace::Up)
    } else if normal.y < -0.5 {
        Some(BlockFace::Down)
    } else if normal.z > 0.5 {
        Some(BlockFace::South)
    } else if normal.z < -0.5 {
        Some(BlockFace::North)
    } else if normal.x > 0.5 {
        Some(BlockFace::East)
    } else if normal.x < -0.5 {
        Some(BlockFace::West)
    } else {
        None
    }
}

/// Checks if a non-solid block is selectable/clickable by the player.
#[inline]
pub fn can_select_block(block_id: u16) -> bool {
    // Air, liquids (normally) are not clicked unless interacting with bucket
    !matches!(block_id, 0 | 8 | 9 | 10 | 11)
}

/// Gets the visual selection box of a block for raycasting and outline rendering.
pub fn get_block_selection_box(x: i32, y: i32, z: i32, state: BlockState) -> AABB {
    let bx = x as f64;
    let by = y as f64;
    let bz = z as f64;
    match state.id() {
        // Slabs (half block)
        44 | 126 => {
            let is_top = (state.meta() & 0x8) != 0;
            if is_top {
                AABB::new(bx, by + 0.5, bz, bx + 1.0, by + 1.0, bz + 1.0)
            } else {
                AABB::new(bx, by, bz, bx + 1.0, by + 0.5, bz + 1.0)
            }
        }
        // Carpet (1/16 block)
        171 => AABB::new(bx, by, bz, bx + 1.0, by + 0.0625, bz + 1.0),
        // Snow layer
        78 => {
            let layers = ((state.meta() & 0x7) + 1) as f64;
            AABB::new(bx, by, bz, bx + 1.0, by + layers * 0.125, bz + 1.0)
        }
        // Trapdoors
        96 | 167 => {
            let is_top = (state.meta() & 0x4) != 0;
            let is_open = (state.meta() & 0x8) != 0;
            if is_open {
                AABB::new(bx, by, bz, bx + 1.0, by + 1.0, bz + 1.0)
            } else if is_top {
                AABB::new(bx, by + 0.8125, bz, bx + 1.0, by + 1.0, bz + 1.0)
            } else {
                AABB::new(bx, by, bz, bx + 1.0, by + 0.1875, bz + 1.0)
            }
        }
        // Standard full cube selection
        _ => AABB::new(bx, by, bz, bx + 1.0, by + 1.0, bz + 1.0),
    }
}

/// Spatial Hash Grid for accelerating entity raycasts and hitbox queries.
pub struct EntitySpatialHash {
    cell_size: f64,
    cells: HashMap<(i32, i32, i32), Vec<i32>>,
}

impl EntitySpatialHash {
    pub fn new(cell_size: f64) -> Self {
        Self {
            cell_size: cell_size.max(1.0),
            cells: HashMap::new(),
        }
    }

    #[inline]
    fn to_cell_coord(&self, val: f64) -> i32 {
        (val / self.cell_size).floor() as i32
    }

    /// Clears and populates the spatial hash from an EntityTracker.
    pub fn rebuild(&mut self, tracker: &EntityTracker) {
        self.cells.clear();
        for entity in tracker.entities.values() {
            let aabb = entity.bounding_box();
            let min_cx = self.to_cell_coord(aabb.min_x);
            let max_cx = self.to_cell_coord(aabb.max_x);
            let min_cy = self.to_cell_coord(aabb.min_y);
            let max_cy = self.to_cell_coord(aabb.max_y);
            let min_cz = self.to_cell_coord(aabb.min_z);
            let max_cz = self.to_cell_coord(aabb.max_z);

            for cx in min_cx..=max_cx {
                for cy in min_cy..=max_cy {
                    for cz in min_cz..=max_cz {
                        self.cells.entry((cx, cy, cz)).or_default().push(entity.id);
                    }
                }
            }
        }
    }

    /// Traces a ray against entities using exact bounding box intersection and collision filtering.
    pub fn ray_trace_entities(
        &self,
        tracker: &EntityTracker,
        start: DVec3,
        dir: DVec3,
        max_distance: f64,
        ignore_id: Option<i32>,
        partial_ticks: f32,
    ) -> Option<EntityRayHit> {
        if max_distance <= 0.0 || dir.length_squared() < 1e-12 {
            return None;
        }
        let unit_dir = dir.normalize();
        let border_size = 0.1;

        let mut closest_hit: Option<EntityRayHit> = None;
        let mut min_dist = max_distance;

        // Iterate tracker entities directly for guaranteed consistency, zero quantization error,
        // and exact visual hitbox alignment via partial_ticks interpolation.
        for entity in tracker.entities.values() {
            if Some(entity.id) == ignore_id || !entity.can_be_collided_with() {
                continue;
            }

            let interp_pos = entity.interpolated_pos(partial_ticks);

            // Quick radial reject: if entity center is farther than max_distance + 3.0 blocks, skip.
            let dist_sq = (interp_pos - start).length_squared();
            let max_r = max_distance + 3.0;
            if dist_sq > max_r * max_r {
                continue;
            }

            let mut bb = entity.bounding_box();
            let offset = interp_pos - entity.pos;
            bb = bb.offset(offset.x, offset.y, offset.z);
            bb.min_x -= border_size;
            bb.min_y -= border_size;
            bb.min_z -= border_size;
            bb.max_x += border_size;
            bb.max_y += border_size;
            bb.max_z += border_size;

            if let Some((t, _)) = bb.ray_trace(start, unit_dir, min_dist) {
                if t < min_dist {
                    min_dist = t;
                    closest_hit = Some(EntityRayHit {
                        entity_id: entity.id,
                        hit_pos: start + unit_dir * t,
                        distance: t,
                    });
                }
            }
        }

        closest_hit
    }
}

/// Unified target picking function (MCP 1.8.9 EntityRenderer.getMouseOver).
pub fn pick_target(
    world: &World,
    spatial_hash: &EntitySpatialHash,
    eye_pos: DVec3,
    look_dir: DVec3,
    is_creative: bool,
    ignore_entity_id: Option<i32>,
    partial_ticks: f32,
) -> TargetHit {
    let block_reach = if is_creative { CREATIVE_BLOCK_REACH } else { SURVIVAL_BLOCK_REACH };
    let entity_reach = if is_creative { CREATIVE_BLOCK_REACH } else { 3.0 };

    let block_hit = ray_trace_voxels(world, eye_pos, look_dir, block_reach);
    let mut search_distance = block_reach;
    if let Some(bh) = &block_hit {
        search_distance = bh.distance;
    }

    let entity_hit = spatial_hash.ray_trace_entities(
        &world.entity_tracker,
        eye_pos,
        look_dir,
        search_distance,
        ignore_entity_id,
        partial_ticks,
    );

    if let Some(eh) = entity_hit {
        // MCP 1.8.9 EntityRenderer.java:470 parity:
        // Entity attack reach is capped at 3.0 blocks in survival.
        // If entity is within reach and closer than the targeted block, select entity.
        if eh.distance <= entity_reach {
            let closer_than_block = block_hit.as_ref().map_or(true, |b| eh.distance <= b.distance);
            if closer_than_block {
                return TargetHit::Entity(eh);
            }
        }
    }

    if let Some(bh) = block_hit {
        TargetHit::Block(bh)
    } else {
        TargetHit::Miss
    }
}
