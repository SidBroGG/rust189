use crate::block::BlockState;
use crate::physics::aabb::AABB;
use crate::world::World;
use glam::DVec3;

/// Result of a voxel block raycast.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayTraceResult {
    pub block_pos: (i32, i32, i32),
    pub hit_normal: DVec3,
    pub hit_pos: DVec3,
    pub distance: f64,
}

/// Returns the block's slipperiness factor (MCP `block.slipperiness`).
/// Default is 0.6, ice/packed ice is 0.98, slime block is 0.8.
#[inline]
pub fn get_block_slipperiness(block_id: u16) -> f32 {
    match block_id {
        79 | 174 => 0.98, // Ice, Packed Ice
        165 => 0.8,      // Slime Block
        _ => 0.6,
    }
}

/// Checks whether a block is a fluid (water or lava).
#[inline]
pub fn is_fluid_block(block_id: u16) -> bool {
    matches!(block_id, 8 | 9 | 10 | 11)
}

/// Checks whether a block is water.
#[inline]
pub fn is_water_block(block_id: u16) -> bool {
    matches!(block_id, 8 | 9)
}

/// Checks whether a block is lava.
#[inline]
pub fn is_lava_block(block_id: u16) -> bool {
    matches!(block_id, 10 | 11)
}

/// Checks whether a block is a climbable ladder or vine.
#[inline]
pub fn is_ladder_block(block_id: u16) -> bool {
    matches!(block_id, 65 | 106) // Ladder, Vine
}

/// Checks whether a block is cobweb.
#[inline]
pub fn is_web_block(block_id: u16) -> bool {
    block_id == 30
}

/// Checks if a block is stairs.
#[inline]
pub fn is_stairs_block(id: u16) -> bool {
    matches!(
        id,
        53 | 67 | 108 | 109 | 114 | 128 | 134 | 135 | 136 | 156 | 163 | 164 | 180
    )
}

/// Checks if a block is a fence.
#[inline]
pub fn is_fence_block(id: u16) -> bool {
    matches!(id, 85 | 188..=192)
}

/// Checks if a block connects to fences (solid full block or same fence).
#[inline]
pub fn can_fence_connect_to(world: &World, x: i32, y: i32, z: i32) -> bool {
    let state = world.block_at(x, y, z);
    let id = state.id();
    if is_fence_block(id) || id == 107 || (183..=187).contains(&id) {
        return true;
    }
    state.is_opaque()
}

/// Appends collision bounding boxes for the block at (x, y, z) that intersect with `mask`.
pub fn get_block_collision_boxes(
    world: &World,
    x: i32,
    y: i32,
    z: i32,
    state: BlockState,
    mask: &AABB,
    boxes: &mut Vec<AABB>,
) {
    let id = state.id();
    let meta = state.meta();
    let (fx, fy, fz) = (x as f64, y as f64, z as f64);

    // Pass-through blocks with no solid collision
    match id {
        0 | 6 | 8 | 9 | 10 | 11 | 27 | 28 | 30 | 31 | 32 | 37 | 38 | 39 | 40 | 50 | 51 | 55
        | 59 | 65 | 66 | 68 | 69 | 75 | 76 | 77 | 83 | 90 | 104 | 105 | 106 | 115 | 119
        | 127 | 131 | 132 | 141 | 142 | 143 | 157 | 175 => return,
        _ => {}
    }

    // Fence Gate (107, 183..187) - open has no collision
    if (id == 107 || (183..=187).contains(&id)) && (meta & 4) != 0 {
        return;
    }

    // Carpets (171): 1/16 block height (0.0625)
    if id == 171 {
        let bb = AABB::new(fx, fy, fz, fx + 1.0, fy + 0.0625, fz + 1.0);
        if bb.intersects(mask) {
            boxes.push(bb);
        }
        return;
    }

    // Pressure Plates (70, 72, 147, 148): 1/16 height
    if matches!(id, 70 | 72 | 147 | 148) {
        let bb = AABB::new(fx + 0.0625, fy, fz + 0.0625, fx + 0.9375, fy + 0.0625, fz + 0.9375);
        if bb.intersects(mask) {
            boxes.push(bb);
        }
        return;
    }

    // Cactus (81): contracted by 1/16
    if id == 81 {
        let f = 0.0625;
        let bb = AABB::new(fx + f, fy, fz + f, fx + 1.0 - f, fy + 1.0 - f, fz + 1.0 - f);
        if bb.intersects(mask) {
            boxes.push(bb);
        }
        return;
    }

    // Chests (54, 130, 146): 0.875 height, 1/16 margin
    if matches!(id, 54 | 130 | 146) {
        let bb = AABB::new(fx + 0.0625, fy, fz + 0.0625, fx + 0.9375, fy + 0.875, fz + 0.9375);
        if bb.intersects(mask) {
            boxes.push(bb);
        }
        return;
    }

    // Half-Slabs (44, 126, 182)
    if matches!(id, 44 | 126 | 182) {
        let is_top = (meta & 8) != 0;
        let (y_min, y_max) = if is_top {
            (fy + 0.5, fy + 1.0)
        } else {
            (fy, fy + 0.5)
        };
        let bb = AABB::new(fx, y_min, fz, fx + 1.0, y_max, fz + 1.0);
        if bb.intersects(mask) {
            boxes.push(bb);
        }
        return;
    }

    // Fences (85, 188..192) & Cobblestone Walls (139): 1.5 blocks high
    if is_fence_block(id) || id == 139 {
        let north = can_fence_connect_to(world, x, y, z - 1);
        let south = can_fence_connect_to(world, x, y, z + 1);
        let west = can_fence_connect_to(world, x - 1, y, z);
        let east = can_fence_connect_to(world, x + 1, y, z);

        let z_min = if north { 0.0 } else { 0.375 };
        let z_max = if south { 1.0 } else { 0.625 };
        if north || south {
            let bb = AABB::new(fx + 0.375, fy, fz + z_min, fx + 0.625, fy + 1.5, fz + z_max);
            if bb.intersects(mask) {
                boxes.push(bb);
            }
        }

        let x_min = if west { 0.0 } else { 0.375 };
        let x_max = if east { 1.0 } else { 0.625 };
        if west || east || (!north && !south) {
            let bb = AABB::new(fx + x_min, fy, fz + 0.375, fx + x_max, fy + 1.5, fz + 0.625);
            if bb.intersects(mask) {
                boxes.push(bb);
            }
        }
        return;
    }

    // Fence Gate closed (107, 183..187): 1.5 blocks high
    if id == 107 || (183..=187).contains(&id) {
        let facing = meta & 3;
        let bb = if facing == 0 || facing == 2 {
            // South or North -> spans X
            AABB::new(fx, fy, fz + 0.375, fx + 1.0, fy + 1.5, fz + 0.625)
        } else {
            // East or West -> spans Z
            AABB::new(fx + 0.375, fy, fz, fx + 0.625, fy + 1.5, fz + 1.0)
        };
        if bb.intersects(mask) {
            boxes.push(bb);
        }
        return;
    }

    // Trapdoors (96, 167)
    if matches!(id, 96 | 167) {
        let is_open = (meta & 4) != 0;
        let is_top = (meta & 8) != 0;
        let bb = if is_open {
            let facing = meta & 3;
            match facing {
                0 => AABB::new(fx, fy, fz + 0.8125, fx + 1.0, fy + 1.0, fz + 1.0),
                1 => AABB::new(fx, fy, fz, fx + 1.0, fy + 1.0, fz + 0.1875),
                2 => AABB::new(fx + 0.8125, fy, fz, fx + 1.0, fy + 1.0, fz + 1.0),
                _ => AABB::new(fx, fy, fz, fx + 0.1875, fy + 1.0, fz + 1.0),
            }
        } else if is_top {
            AABB::new(fx, fy + 0.8125, fz, fx + 1.0, fy + 1.0, fz + 1.0)
        } else {
            AABB::new(fx, fy, fz, fx + 1.0, fy + 0.1875, fz + 1.0)
        };
        if bb.intersects(mask) {
            boxes.push(bb);
        }
        return;
    }

    // Doors (64, 71, 193..197)
    if matches!(id, 64 | 71 | 193..=197) {
        let is_top = (meta & 8) != 0;
        let (lower_meta, upper_meta) = if is_top {
            let lower_state = world.block_at(x, y - 1, z);
            (lower_state.meta(), meta)
        } else {
            let upper_state = world.block_at(x, y + 1, z);
            (meta, upper_state.meta())
        };
        let is_open = (lower_meta & 4) != 0;
        let facing = lower_meta & 3;
        let is_hinge_right = (upper_meta & 1) != 0;

        let bb = compute_door_aabb(fx, fy, fz, facing, is_open, is_hinge_right);
        if bb.intersects(mask) {
            boxes.push(bb);
        }
        return;
    }

    // Stairs: base slab + step pieces
    if is_stairs_block(id) {
        let is_top = (meta & 4) != 0;
        let facing = meta & 3; // 0=East(+X), 1=West(-X), 2=South(+Z), 3=North(-Z)
        let (base_y_min, base_y_max, step_y_min, step_y_max) = if is_top {
            (fy + 0.5, fy + 1.0, fy, fy + 0.5)
        } else {
            (fy, fy + 0.5, fy + 0.5, fy + 1.0)
        };

        // 1. Base slab
        let base_bb = AABB::new(fx, base_y_min, fz, fx + 1.0, base_y_max, fz + 1.0);
        if base_bb.intersects(mask) {
            boxes.push(base_bb);
        }

        // 2. Step piece
        let (step_x_min, step_x_max, step_z_min, step_z_max) = match facing {
            0 => (fx + 0.5, fx + 1.0, fz, fz + 1.0), // East (+X)
            1 => (fx, fx + 0.5, fz, fz + 1.0),       // West (-X)
            2 => (fx, fx + 1.0, fz + 0.5, fz + 1.0), // South (+Z)
            _ => (fx, fx + 1.0, fz, fz + 0.5),       // North (-Z)
        };
        let step_bb = AABB::new(
            step_x_min, step_y_min, step_z_min, step_x_max, step_y_max, step_z_max,
        );
        if step_bb.intersects(mask) {
            boxes.push(step_bb);
        }
        return;
    }

    // Default full solid block
    let bb = AABB::new(fx, fy, fz, fx + 1.0, fy + 1.0, fz + 1.0);
    if bb.intersects(mask) {
        boxes.push(bb);
    }
}

fn compute_door_aabb(
    fx: f64,
    fy: f64,
    fz: f64,
    facing: u8,
    is_open: bool,
    is_hinge_right: bool,
) -> AABB {
    let t = 0.1875; // 3/16 thickness
    if !is_open {
        match facing {
            0 => AABB::new(fx, fy, fz, fx + t, fy + 1.0, fz + 1.0),
            1 => AABB::new(fx, fy, fz, fx + 1.0, fy + 1.0, fz + t),
            2 => AABB::new(fx + 1.0 - t, fy, fz, fx + 1.0, fy + 1.0, fz + 1.0),
            _ => AABB::new(fx, fy, fz + 1.0 - t, fx + 1.0, fy + 1.0, fz + 1.0),
        }
    } else {
        match (facing, is_hinge_right) {
            (0, false) => AABB::new(fx, fy, fz, fx + 1.0, fy + 1.0, fz + t),
            (0, true) => AABB::new(fx, fy, fz + 1.0 - t, fx + 1.0, fy + 1.0, fz + 1.0),
            (1, false) => AABB::new(fx + 1.0 - t, fy, fz, fx + 1.0, fy + 1.0, fz + 1.0),
            (1, true) => AABB::new(fx, fy, fz, fx + t, fy + 1.0, fz + 1.0),
            (2, false) => AABB::new(fx, fy, fz + 1.0 - t, fx + 1.0, fy + 1.0, fz + 1.0),
            (2, true) => AABB::new(fx, fy, fz, fx + 1.0, fy + 1.0, fz + t),
            (_, false) => AABB::new(fx, fy, fz, fx + t, fy + 1.0, fz + 1.0),
            (_, true) => AABB::new(fx + 1.0 - t, fy, fz, fx + 1.0, fy + 1.0, fz + 1.0),
        }
    }
}

impl World {
    /// Collects all colliding bounding boxes in the world that intersect `mask` (MCP `getCollidingBoundingBoxes`).
    pub fn get_colliding_bounding_boxes(&self, mask: &AABB) -> Vec<AABB> {
        let mut list = Vec::new();
        let min_x = mask.min_x.floor() as i32;
        let max_x = (mask.max_x + 1.0).floor() as i32;
        let min_y = (mask.min_y.floor() as i32).max(0);
        let max_y = ((mask.max_y + 1.0).floor() as i32).min(256);
        let min_z = mask.min_z.floor() as i32;
        let max_z = (mask.max_z + 1.0).floor() as i32;

        for bx in min_x..max_x {
            for bz in min_z..max_z {
                for by in min_y..max_y {
                    let state = self.block_at(bx, by, bz);
                    if !state.is_air() {
                        get_block_collision_boxes(self, bx, by, bz, state, mask, &mut list);
                    }
                }
            }
        }

        list
    }

    /// Traces a ray through the world blocks, returning the first block collision encountered (MCP `rayTraceBlocks`).
    pub fn ray_trace_blocks(&self, start: DVec3, end: DVec3) -> Option<RayTraceResult> {
        let dir = end - start;
        let max_dist = dir.length();
        if max_dist < 1e-6 {
            return None;
        }
        let unit_dir = dir / max_dist;

        let mask = AABB::new(
            start.x.min(end.x) - 0.5,
            start.y.min(end.y) - 0.5,
            start.z.min(end.z) - 0.5,
            start.x.max(end.x) + 0.5,
            start.y.max(end.y) + 0.5,
            start.z.max(end.z) + 0.5,
        );

        let mut closest_hit: Option<RayTraceResult> = None;
        let mut min_t = max_dist;

        let min_x = mask.min_x.floor() as i32;
        let max_x = (mask.max_x + 1.0).floor() as i32;
        let min_y = (mask.min_y.floor() as i32).max(0);
        let max_y = ((mask.max_y + 1.0).floor() as i32).min(256);
        let min_z = mask.min_z.floor() as i32;
        let max_z = (mask.max_z + 1.0).floor() as i32;

        let mut block_boxes = Vec::with_capacity(4);

        for bx in min_x..max_x {
            for bz in min_z..max_z {
                for by in min_y..max_y {
                    let state = self.block_at(bx, by, bz);
                    if state.is_air() {
                        continue;
                    }
                    block_boxes.clear();
                    get_block_collision_boxes(self, bx, by, bz, state, &mask, &mut block_boxes);

                    for bb in &block_boxes {
                        if let Some((t, normal)) = bb.ray_trace(start, unit_dir, min_t) {
                            if t < min_t {
                                min_t = t;
                                closest_hit = Some(RayTraceResult {
                                    block_pos: (bx, by, bz),
                                    hit_normal: normal,
                                    hit_pos: start + unit_dir * t,
                                    distance: t,
                                });
                            }
                        }
                    }
                }
            }
        }

        closest_hit
    }
}
