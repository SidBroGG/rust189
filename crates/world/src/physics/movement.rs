use crate::physics::aabb::AABB;
use crate::physics::collision::{
    get_block_slipperiness, is_ladder_block, is_lava_block, is_water_block, is_web_block,
};
use crate::world::World;
use glam::DVec3;
use std::f64::consts::PI;

/// User movement input state for one tick.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerInput {
    pub move_forward: f32, // -1.0 .. 1.0 (W/S)
    pub move_strafe: f32,  // -1.0 .. 1.0 (A/D: positive is left in MCP)
    pub jump: bool,
    pub sneak: bool,
    pub sprint: bool,
}

/// Authentic Minecraft 1.8.9 player physics and movement controller (MCP `EntityPlayerSP`).
#[derive(Debug, Clone)]
pub struct PlayerMovement {
    pub pos: DVec3,
    pub prev_pos: DVec3,
    pub motion: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub prev_yaw: f32,
    pub prev_pitch: f32,

    pub on_ground: bool,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub is_in_web: bool,
    pub is_in_water: bool,
    pub is_in_lava: bool,
    pub is_on_ladder: bool,
    pub collided_horizontally: bool,
    pub collided_vertically: bool,

    pub step_height: f32,
    pub jump_boost: u8,
    pub fall_distance: f32,
    pub food_level: f32,
    pub is_using_item: bool,

    // Camera and view bobbing state
    pub distance_walked_modified: f32,
    pub prev_distance_walked_modified: f32,
    pub camera_yaw: f32,
    pub prev_camera_yaw: f32,
    pub camera_pitch: f32,
    pub prev_camera_pitch: f32,
}

impl PlayerMovement {
    pub const WIDTH: f64 = 0.6;
    pub const HEIGHT: f64 = 1.8;
    pub const SNEAK_HEIGHT: f64 = 1.65;
    pub const EYE_HEIGHT: f32 = 1.62;
    pub const SNEAK_EYE_HEIGHT: f32 = 1.54;

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let pos = DVec3::new(x, y, z);
        Self {
            pos,
            prev_pos: pos,
            motion: DVec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            prev_yaw: 0.0,
            prev_pitch: 0.0,

            on_ground: false,
            is_sneaking: false,
            is_sprinting: false,
            is_in_web: false,
            is_in_water: false,
            is_in_lava: false,
            is_on_ladder: false,
            collided_horizontally: false,
            collided_vertically: false,

            step_height: 0.6,
            jump_boost: 0,
            fall_distance: 0.0,
            food_level: 20.0,
            is_using_item: false,

            distance_walked_modified: 0.0,
            prev_distance_walked_modified: 0.0,
            camera_yaw: 0.0,
            prev_camera_yaw: 0.0,
            camera_pitch: 0.0,
            prev_camera_pitch: 0.0,
        }
    }

    /// Returns the current bounding box of the player.
    #[inline]
    pub fn get_aabb(&self) -> AABB {
        let h = if self.is_sneaking {
            Self::SNEAK_HEIGHT
        } else {
            Self::HEIGHT
        };
        AABB::from_point_size(self.pos.x, self.pos.y, self.pos.z, Self::WIDTH, h)
    }

    /// Returns eye height above player feet based on sneaking state.
    #[inline]
    pub fn get_eye_height(&self) -> f32 {
        if self.is_sneaking {
            Self::SNEAK_EYE_HEIGHT
        } else {
            Self::EYE_HEIGHT
        }
    }

    /// Returns interpolated eye position for smooth camera rendering.
    #[inline]
    pub fn get_interpolated_eye_pos(&self, partial_ticks: f32) -> DVec3 {
        let pt = partial_ticks as f64;
        let interp_pos = self.prev_pos + (self.pos - self.prev_pos) * pt;
        let eye_h = self.get_eye_height() as f64;
        DVec3::new(interp_pos.x, interp_pos.y + eye_h, interp_pos.z)
    }

    /// Applies external velocity (e.g. from server packet 0x12 EntityVelocity).
    pub fn apply_velocity(&mut self, vx: f64, vy: f64, vz: f64) {
        self.motion.x = vx;
        self.motion.y = vy;
        self.motion.z = vz;
    }

    /// Applies explosion impulse (from server packet 0x27 Explosion).
    pub fn apply_explosion(&mut self, mx: f64, my: f64, mz: f64) {
        self.motion.x += mx;
        self.motion.y += my;
        self.motion.z += mz;
    }

    /// Checks environment at current player position.
    fn check_environment(&mut self, world: &World) {
        let bb = self.get_aabb().contract(0.001, 0.001, 0.001);
        let min_x = bb.min_x.floor() as i32;
        let max_x = (bb.max_x + 1.0).floor() as i32;
        let min_y = (bb.min_y.floor() as i32).max(0);
        let max_y = ((bb.max_y + 1.0).floor() as i32).min(256);
        let min_z = bb.min_z.floor() as i32;
        let max_z = (bb.max_z + 1.0).floor() as i32;

        self.is_in_web = false;
        self.is_in_water = false;
        self.is_in_lava = false;
        self.is_on_ladder = false;

        for bx in min_x..max_x {
            for bz in min_z..max_z {
                for by in min_y..max_y {
                    let id = world.block_at(bx, by, bz).id();
                    if is_web_block(id) {
                        self.is_in_web = true;
                    }
                    if is_water_block(id) {
                        self.is_in_water = true;
                    }
                    if is_lava_block(id) {
                        self.is_in_lava = true;
                    }
                    if is_ladder_block(id) {
                        self.is_on_ladder = true;
                    }
                }
            }
        }
    }

    /// Single 20 TPS physics tick port of MCP `EntityPlayerSP.onLivingUpdate` and `moveEntityWithHeading`.
    pub fn tick(&mut self, world: &World, input: &PlayerInput) {
        self.prev_pos = self.pos;
        self.prev_yaw = self.yaw;
        self.prev_pitch = self.pitch;
        self.prev_distance_walked_modified = self.distance_walked_modified;
        self.prev_camera_yaw = self.camera_yaw;
        self.prev_camera_pitch = self.camera_pitch;

        self.check_environment(world);

        let mut strafe = input.move_strafe;
        let mut forward = input.move_forward;

        if self.is_using_item {
            strafe *= 0.2;
            forward *= 0.2;
            self.is_sprinting = false;
        }

        if input.sneak {
            strafe = (strafe as f64 * 0.3) as f32;
            forward = (forward as f64 * 0.3) as f32;
            self.is_sprinting = false;
            self.is_sneaking = true;
        } else {
            self.is_sneaking = false;
        }

        // Sprinting toggle and state validation
        if input.sprint
            && forward >= 0.8
            && self.food_level > 6.0
            && !self.is_sneaking
            && !self.is_using_item
            && !self.collided_horizontally
        {
            self.is_sprinting = true;
        } else if forward < 0.8 || self.collided_horizontally || self.is_sneaking || self.is_using_item {
            self.is_sprinting = false;
        }

        // Jump processing
        if input.jump {
            if self.is_in_water || self.is_in_lava {
                self.motion.y += 0.03999999910593033;
            } else if self.on_ground {
                let mut jump_vel = 0.42;
                if self.jump_boost > 0 {
                    jump_vel += (self.jump_boost as f64) * 0.1;
                }
                self.motion.y = jump_vel;

                // Sprint-jump 0.2 forward impulse (MCP `EntityLivingBase.jump`)
                if self.is_sprinting {
                    let rad = (self.yaw as f64) * (PI / 180.0);
                    self.motion.x -= rad.sin() * 0.2;
                    self.motion.z += rad.cos() * 0.2;
                }
            }
        }

        // Move with heading
        if self.is_in_water {
            self.move_flying(strafe, forward, 0.02);
            self.move_entity(world, self.motion.x, self.motion.y, self.motion.z);
            self.motion.x *= 0.8;
            self.motion.y *= 0.8;
            self.motion.z *= 0.8;
            self.motion.y -= 0.02;
            if self.collided_horizontally {
                self.motion.y = 0.3;
            }
        } else if self.is_in_lava {
            self.move_flying(strafe, forward, 0.02);
            self.move_entity(world, self.motion.x, self.motion.y, self.motion.z);
            self.motion.x *= 0.5;
            self.motion.y *= 0.5;
            self.motion.z *= 0.5;
            self.motion.y -= 0.02;
            if self.collided_horizontally {
                self.motion.y = 0.3;
            }
        } else {
            // Air or Ground physics
            let mut slipperiness = 0.6;
            let mut f4 = 0.91;
            if self.on_ground {
                let block_x = self.pos.x.floor() as i32;
                let block_y = (self.get_aabb().min_y - 0.2).floor() as i32;
                let block_z = self.pos.z.floor() as i32;
                let block_under = world.block_at(block_x, block_y, block_z).id();
                slipperiness = get_block_slipperiness(block_under) as f64;
                f4 = slipperiness * 0.91;
            }

            let f = 0.16277136 / (f4 * f4 * f4);
            let ai_move_speed = if self.is_sprinting {
                0.1 * 1.300000011920929
            } else {
                0.1
            };
            let f5 = if self.on_ground {
                (ai_move_speed * f) as f32
            } else if self.is_sprinting {
                0.026
            } else {
                0.02
            };

            self.move_flying(strafe, forward, f5);

            f4 = 0.91;
            if self.on_ground {
                f4 = slipperiness * 0.91;
            }

            if self.is_on_ladder {
                let f6 = 0.15;
                self.motion.x = self.motion.x.clamp(-f6, f6);
                self.motion.z = self.motion.z.clamp(-f6, f6);
                self.fall_distance = 0.0;
                if self.motion.y < -0.15 {
                    self.motion.y = -0.15;
                }
                if self.is_sneaking && self.motion.y < 0.0 {
                    self.motion.y = 0.0;
                }
            }

            self.move_entity(world, self.motion.x, self.motion.y, self.motion.z);

            if self.collided_horizontally && self.is_on_ladder {
                self.motion.y = 0.2;
            }

            // Vanilla gravity (0.08 blocks/tick) and drag (0.98)
            self.motion.y -= 0.08;
            self.motion.y *= 0.9800000190734863;
            self.motion.x *= f4;
            self.motion.z *= f4;
        }

        // View bobbing calculation (MCP `EntityPlayer.onLivingUpdate`)
        let dx = self.pos.x - self.prev_pos.x;
        let dz = self.pos.z - self.prev_pos.z;
        let d_horiz = (dx * dx + dz * dz).sqrt();
        if self.on_ground {
            self.distance_walked_modified += (d_horiz * 0.6) as f32;
        }

        let mut f_bob = d_horiz as f32;
        if f_bob > 0.1 {
            f_bob = 0.1;
        }
        if !self.on_ground {
            f_bob = 0.0;
        }
        self.camera_yaw += (f_bob - self.camera_yaw) * 0.4;

        let mut f1_pitch = (-self.motion.y * 0.20000000298023224).atan() as f32 * 15.0;
        if self.on_ground {
            f1_pitch = 0.0;
        }
        self.camera_pitch += (f1_pitch - self.camera_pitch) * 0.8;
    }

    /// Minecraft `Entity.moveFlying` angle acceleration with 45-degree normalization.
    pub fn move_flying(&mut self, mut strafe: f32, mut forward: f32, friction: f32) {
        let mut f = strafe * strafe + forward * forward;
        if f >= 1.0e-4 {
            f = f.sqrt();
            if f < 1.0 {
                f = 1.0;
            }
            f = friction / f;
            strafe *= f;
            forward *= f;

            let yaw_rad = (self.yaw as f64) * (PI / 180.0);
            let sin_yaw = yaw_rad.sin();
            let cos_yaw = yaw_rad.cos();

            self.motion.x += (strafe as f64) * cos_yaw - (forward as f64) * sin_yaw;
            self.motion.z += (forward as f64) * cos_yaw + (strafe as f64) * sin_yaw;
        }
    }

    /// Moves the player displacement matching MCP 1.8.9 `Entity.moveEntity`.
    pub fn move_entity(&mut self, world: &World, mut x: f64, mut y: f64, mut z: f64) {
        if self.is_in_web {
            self.is_in_web = false;
            x *= 0.25;
            y *= 0.05000000074505806;
            z *= 0.25;
            self.motion = DVec3::ZERO;
        }

        let mut aabb = self.get_aabb();
        let mut d3 = x;
        let d4 = y;
        let mut d5 = z;

        // Sneaking ledge protection (MCP `Entity.moveEntity`)
        let is_sneaking_on_ground = self.on_ground && self.is_sneaking;
        if is_sneaking_on_ground {
            let step = 0.05;
            while x != 0.0 && world.get_colliding_bounding_boxes(&aabb.offset(x, -1.0, 0.0)).is_empty() {
                if x < step && x >= -step {
                    x = 0.0;
                } else if x > 0.0 {
                    x -= step;
                } else {
                    x += step;
                }
                d3 = x;
            }

            while z != 0.0 && world.get_colliding_bounding_boxes(&aabb.offset(0.0, -1.0, z)).is_empty() {
                if z < step && z >= -step {
                    z = 0.0;
                } else if z > 0.0 {
                    z -= step;
                } else {
                    z += step;
                }
                d5 = z;
            }

            while x != 0.0
                && z != 0.0
                && world.get_colliding_bounding_boxes(&aabb.offset(x, -1.0, z)).is_empty()
            {
                if x < step && x >= -step {
                    x = 0.0;
                } else if x > 0.0 {
                    x -= step;
                } else {
                    x += step;
                }
                d3 = x;

                if z < step && z >= -step {
                    z = 0.0;
                } else if z > 0.0 {
                    z -= step;
                } else {
                    z += step;
                }
                d5 = z;
            }
        }

        let colliding = world.get_colliding_bounding_boxes(&aabb.add_coord(x, y, z));
        let orig_aabb = aabb;

        // Y collision offset
        for b in &colliding {
            y = b.calculate_y_offset(&aabb, y);
        }
        aabb = aabb.offset(0.0, y, 0.0);
        let flag1 = self.on_ground || (d4 != y && d4 < 0.0);

        // X collision offset
        for b in &colliding {
            x = b.calculate_x_offset(&aabb, x);
        }
        aabb = aabb.offset(x, 0.0, 0.0);

        // Z collision offset
        for b in &colliding {
            z = b.calculate_z_offset(&aabb, z);
        }
        aabb = aabb.offset(0.0, 0.0, z);

        // Step-height climbing (stepHeight 0.6)
        if self.step_height > 0.0 && flag1 && (d3 != x || d5 != z) {
            let d11 = x;
            let _d7 = y;
            let d8 = z;
            let step_y = self.step_height as f64;
            let step_colliding = world.get_colliding_bounding_boxes(&orig_aabb.add_coord(d3, step_y, d5));

            let mut bb4 = orig_aabb;
            let bb5 = bb4.add_coord(d3, 0.0, d5);
            let mut d9 = step_y;
            for b in &step_colliding {
                d9 = b.calculate_y_offset(&bb5, d9);
            }
            bb4 = bb4.offset(0.0, d9, 0.0);
            let mut d15 = d3;
            for b in &step_colliding {
                d15 = b.calculate_x_offset(&bb4, d15);
            }
            bb4 = bb4.offset(d15, 0.0, 0.0);
            let mut d16 = d5;
            for b in &step_colliding {
                d16 = b.calculate_z_offset(&bb4, d16);
            }
            bb4 = bb4.offset(0.0, 0.0, d16);

            let mut bb14 = orig_aabb;
            let mut d17 = step_y;
            for b in &step_colliding {
                d17 = b.calculate_y_offset(&bb14, d17);
            }
            bb14 = bb14.offset(0.0, d17, 0.0);
            let mut d18 = d3;
            for b in &step_colliding {
                d18 = b.calculate_x_offset(&bb14, d18);
            }
            bb14 = bb14.offset(d18, 0.0, 0.0);
            let mut d19 = d5;
            for b in &step_colliding {
                d19 = b.calculate_z_offset(&bb14, d19);
            }
            bb14 = bb14.offset(0.0, 0.0, d19);

            let d20 = d15 * d15 + d16 * d16;
            let d10 = d18 * d18 + d19 * d19;
            let (step_res_x, step_res_y, step_res_z, mut best_bb) = if d20 > d10 {
                (d15, -d9, d16, bb4)
            } else {
                (d18, -d17, d19, bb14)
            };

            let mut down_y = step_res_y;
            for b in &step_colliding {
                down_y = b.calculate_y_offset(&best_bb, down_y);
            }
            best_bb = best_bb.offset(0.0, down_y, 0.0);

            if d11 * d11 + d8 * d8 < step_res_x * step_res_x + step_res_z * step_res_z {
                x = step_res_x;
                y = down_y;
                z = step_res_z;
                aabb = best_bb;
            }
        }

        // Apply updated position
        self.pos.x = (aabb.min_x + aabb.max_x) * 0.5;
        self.pos.y = aabb.min_y;
        self.pos.z = (aabb.min_z + aabb.max_z) * 0.5;

        self.collided_horizontally = d3 != x || d5 != z;
        self.collided_vertically = d4 != y;
        self.on_ground = self.collided_vertically && d4 < 0.0;

        if d3 != x {
            self.motion.x = 0.0;
        }
        if d5 != z {
            self.motion.z = 0.0;
        }
        if d4 != y {
            let block_x = self.pos.x.floor() as i32;
            let block_y = (self.pos.y - 0.20000000298023224).floor() as i32;
            let block_z = self.pos.z.floor() as i32;
            let mut block_under = world.block_at(block_x, block_y, block_z).id();
            if block_under == 0 {
                let below = world.block_at(block_x, block_y - 1, block_z).id();
                if crate::physics::collision::is_fence_block(below)
                    || below == 139
                    || below == 107
                    || (183..=187).contains(&below)
                {
                    block_under = below;
                }
            }

            if block_under == 165 && !self.is_sneaking && self.motion.y < 0.0 {
                // MCP BlockSlime.onLanded
                self.motion.y = -self.motion.y;
            } else {
                // MCP Block.onLanded
                self.motion.y = 0.0;
            }
        }
    }
}
