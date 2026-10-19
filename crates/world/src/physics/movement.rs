use crate::physics::aabb::AABB;
use crate::physics::collision::{
    get_block_slipperiness, get_water_flow_vector, is_any_liquid_in_aabb, is_ladder_block,
    is_lava_block, is_water_block, is_web_block, normalize_mcp,
};
use crate::world::World;
use glam::DVec3;
use std::f64::consts::PI;

/// Player capabilities ported from MCP `PlayerCapabilities`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerCapabilities {
    pub is_flying: bool,
    pub allow_flying: bool,
    pub is_creative: bool,
    pub is_invulnerable: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
}

impl Default for PlayerCapabilities {
    fn default() -> Self {
        Self {
            is_flying: false,
            allow_flying: false,
            is_creative: false,
            is_invulnerable: false,
            fly_speed: 0.05,
            walk_speed: 0.1,
        }
    }
}

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
    pub bounding_box: AABB,
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

    // Flight and abilities (MCP `EntityPlayerSP`)
    pub capabilities: PlayerCapabilities,
    pub jump_ticks: u32,
    pub fly_toggle_timer: u32,
    pub was_jump_pressed: bool,
    pub abilities_dirty: bool,
    pub is_spectator: bool,

    // Camera and view bobbing state
    pub distance_walked_modified: f32,
    pub prev_distance_walked_modified: f32,
    pub camera_yaw: f32,
    pub prev_camera_yaw: f32,
    pub camera_pitch: f32,
    pub prev_camera_pitch: f32,
}

impl PlayerMovement {
    pub const WIDTH: f32 = 0.6;
    pub const HEIGHT: f32 = 1.8;
    pub const SNEAK_HEIGHT: f32 = 1.65;
    pub const STEP_HEIGHT: f32 = 0.6;
    pub const EYE_HEIGHT: f32 = 1.62;
    pub const SNEAK_EYE_HEIGHT: f32 = 1.54;

    #[inline]
    pub fn half_width() -> f64 {
        (Self::WIDTH / 2.0) as f64
    }

    #[inline]
    pub fn current_height(is_sneaking: bool) -> f64 {
        if is_sneaking {
            Self::SNEAK_HEIGHT as f64
        } else {
            Self::HEIGHT as f64
        }
    }

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let pos = DVec3::new(x, y, z);
        let half_w = Self::half_width();
        let bounding_box = AABB::new(
            x - half_w,
            y,
            z - half_w,
            x + half_w,
            y + Self::current_height(false),
            z + half_w,
        );
        Self {
            pos,
            prev_pos: pos,
            bounding_box,
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

            step_height: Self::STEP_HEIGHT,
            jump_boost: 0,
            fall_distance: 0.0,
            food_level: 20.0,
            is_using_item: false,

            capabilities: PlayerCapabilities::default(),
            jump_ticks: 0,
            fly_toggle_timer: 0,
            was_jump_pressed: false,
            abilities_dirty: false,
            is_spectator: false,

            distance_walked_modified: 0.0,
            prev_distance_walked_modified: 0.0,
            camera_yaw: 0.0,
            prev_camera_yaw: 0.0,
            camera_pitch: 0.0,
            prev_camera_pitch: 0.0,
        }
    }

    /// Returns the current bounding box of the player (MCP `Entity.getEntityBoundingBox`).
    #[inline]
    pub fn get_aabb(&self) -> AABB {
        self.bounding_box
    }

    /// Sets the bounding box of the player (MCP `Entity.setEntityBoundingBox`).
    #[inline]
    pub fn set_aabb(&mut self, aabb: AABB) {
        self.bounding_box = aabb;
    }

    /// Resets the entity's position coordinates from its current bounding box center and bottom (MCP `Entity.resetPositionToBB`).
    #[inline]
    pub fn reset_position_to_bb(&mut self) {
        self.pos.x = (self.bounding_box.min_x + self.bounding_box.max_x) * 0.5;
        self.pos.y = self.bounding_box.min_y;
        self.pos.z = (self.bounding_box.min_z + self.bounding_box.max_z) * 0.5;
    }

    /// Sets the entity's position and reconstructs bounding box (MCP `Entity.setPosition`).
    pub fn set_position(&mut self, x: f64, y: f64, z: f64) {
        self.pos = DVec3::new(x, y, z);
        let half_w = Self::half_width();
        let h = Self::current_height(self.is_sneaking);
        self.bounding_box = AABB::new(
            x - half_w,
            y,
            z - half_w,
            x + half_w,
            y + h,
            z + half_w,
        );
    }

    /// Sets position and rotation, updating bounding box and resetting prev angles (MCP `Entity.setPositionAndRotation`).
    pub fn set_position_and_rotation(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
    ) {
        self.prev_pos = DVec3::new(x, y, z);
        self.pos = self.prev_pos;
        self.prev_yaw = yaw;
        self.yaw = yaw;
        self.prev_pitch = pitch;
        self.pitch = pitch;

        let half_w = Self::half_width();
        let h = Self::current_height(self.is_sneaking);
        self.bounding_box = AABB::new(
            x - half_w,
            y,
            z - half_w,
            x + half_w,
            y + h,
            z + half_w,
        );
    }

    /// Pushes the player out of solid full cubes (MCP `EntityPlayerSP.pushOutOfBlocks`).
    pub fn push_out_of_blocks(&mut self, world: &World, x: f64, y: f64, z: f64) -> bool {
        let bx = x.floor() as i32;
        let by = y.floor() as i32;
        let bz = z.floor() as i32;
        let d0 = x - bx as f64;
        let d1 = z - bz as f64;

        let is_open_block_space = |cx: i32, cy: i32, cz: i32| -> bool {
            !world.block_at(cx, cy, cz).is_opaque() && !world.block_at(cx, cy + 1, cz).is_opaque()
        };

        if !is_open_block_space(bx, by, bz) {
            let mut i = -1;
            let mut d2 = 9999.0;

            if is_open_block_space(bx - 1, by, bz) && d0 < d2 {
                d2 = d0;
                i = 0;
            }
            if is_open_block_space(bx + 1, by, bz) && 1.0 - d0 < d2 {
                d2 = 1.0 - d0;
                i = 1;
            }
            if is_open_block_space(bx, by, bz - 1) && d1 < d2 {
                d2 = d1;
                i = 4;
            }
            if is_open_block_space(bx, by, bz + 1) && 1.0 - d1 < d2 {
                i = 5;
            }

            let f = 0.1;
            if i == 0 {
                self.motion.x = -f;
            }
            if i == 1 {
                self.motion.x = f;
            }
            if i == 4 {
                self.motion.z = -f;
            }
            if i == 5 {
                self.motion.z = f;
            }
            return true;
        }
        false
    }

    /// Checks if the player is resting on top of a solid block (MCP parity on teleport/spawn).
    #[inline]
    pub fn check_ground_contact(&self, world: &World) -> bool {
        let test_bb = self.bounding_box.offset(0.0, -0.05, 0.0);
        !world.get_colliding_bounding_boxes(&test_bb).is_empty()
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
        // Web check (contract by 0.001)
        let web_bb = self.get_aabb().contract(0.001, 0.001, 0.001);
        let min_x = web_bb.min_x.floor() as i32;
        let max_x = (web_bb.max_x + 1.0).floor() as i32;
        let min_y = (web_bb.min_y.floor() as i32).max(0);
        let max_y = ((web_bb.max_y + 1.0).floor() as i32).min(256);
        let min_z = web_bb.min_z.floor() as i32;
        let max_z = (web_bb.max_z + 1.0).floor() as i32;

        self.is_in_web = false;
        'web_loop: for bx in min_x..max_x {
            for bz in min_z..max_z {
                for by in min_y..max_y {
                    let id = world.block_at(bx, by, bz).id();
                    if is_web_block(id) {
                        self.is_in_web = true;
                        break 'web_loop;
                    }
                }
            }
        }

        // Lava check matching MCP `Entity.isInLava` (expand by -0.1, -0.4, -0.1)
        let lava_bb = self.get_aabb().expand(
            -0.10000000149011612,
            -0.4000000059604645,
            -0.10000000149011612,
        );
        let l_min_x = lava_bb.min_x.floor() as i32;
        let l_max_x = (lava_bb.max_x + 1.0).floor() as i32;
        let l_min_y = (lava_bb.min_y.floor() as i32).max(0);
        let l_max_y = ((lava_bb.max_y + 1.0).floor() as i32).min(256);
        let l_min_z = lava_bb.min_z.floor() as i32;
        let l_max_z = (lava_bb.max_z + 1.0).floor() as i32;

        self.is_in_lava = false;
        'lava_loop: for bx in l_min_x..l_max_x {
            for bz in l_min_z..l_max_z {
                for by in l_min_y..l_max_y {
                    let id = world.block_at(bx, by, bz).id();
                    if is_lava_block(id) {
                        self.is_in_lava = true;
                        break 'lava_loop;
                    }
                }
            }
        }

        // Water check matching MCP `handleWaterMovement` (expand Y by -0.4, contract by 0.001)
        let water_bb = self
            .get_aabb()
            .expand(0.0, -0.4000000059604645, 0.0)
            .contract(0.001, 0.001, 0.001);
        let w_min_x = water_bb.min_x.floor() as i32;
        let w_max_x = (water_bb.max_x + 1.0).floor() as i32;
        let w_min_y = (water_bb.min_y.floor() as i32).max(0);
        let w_max_y = ((water_bb.max_y + 1.0).floor() as i32).min(256);
        let w_min_z = water_bb.min_z.floor() as i32;
        let w_max_z = (water_bb.max_z + 1.0).floor() as i32;

        let mut flow_vec = DVec3::ZERO;
        self.is_in_water = false;

        for bx in w_min_x..w_max_x {
            for bz in w_min_z..w_max_z {
                for by in w_min_y..w_max_y {
                    let state = world.block_at(bx, by, bz);
                    if is_water_block(state.id()) {
                        let meta = (state.meta() & 15) as i32;
                        let decay = if meta >= 8 { 0 } else { meta };
                        let pct = (decay + 1) as f64 / 9.0;
                        let surface_y = (by + 1) as f64 - pct;
                        if (w_max_y as f64) >= surface_y {
                            self.is_in_water = true;
                            flow_vec += get_water_flow_vector(world, bx, by, bz);
                        }
                    }
                }
            }
        }

        if flow_vec.length_squared() > 0.0 && !self.capabilities.is_flying {
            let push = normalize_mcp(flow_vec) * 0.014;
            self.motion += push;
        }

        // Exact MCP `EntityLivingBase.isOnLadder` (lines 1134-1141):
        // Checked strictly at the entity's feet block position (floor(posX), floor(minY), floor(posZ))
        let ladder_x = self.pos.x.floor() as i32;
        let ladder_y = self.get_aabb().min_y.floor() as i32;
        let ladder_z = self.pos.z.floor() as i32;
        let ladder_id = world.block_at(ladder_x, ladder_y, ladder_z).id();
        self.is_on_ladder = is_ladder_block(ladder_id);
    }

    /// Single 20 TPS physics tick port of MCP `EntityPlayerSP.onLivingUpdate` and `moveEntityWithHeading`.
    pub fn tick(&mut self, world: &World, input: &PlayerInput) {
        self.prev_pos = self.pos;
        self.prev_yaw = self.yaw;
        self.prev_pitch = self.pitch;
        self.prev_distance_walked_modified = self.distance_walked_modified;
        self.prev_camera_yaw = self.camera_yaw;
        self.prev_camera_pitch = self.camera_pitch;

        if self.jump_ticks > 0 {
            self.jump_ticks -= 1;
        }
        if self.fly_toggle_timer > 0 {
            self.fly_toggle_timer -= 1;
        }

        // Flight toggle on double-tap jump (MCP `EntityPlayerSP` lines 823-846)
        let fresh_jump = input.jump && !self.was_jump_pressed;
        self.was_jump_pressed = input.jump;

        if self.capabilities.allow_flying {
            if self.is_spectator {
                if !self.capabilities.is_flying {
                    self.capabilities.is_flying = true;
                    self.abilities_dirty = true;
                }
            } else if fresh_jump {
                if self.fly_toggle_timer == 0 {
                    self.fly_toggle_timer = 7;
                } else {
                    self.capabilities.is_flying = !self.capabilities.is_flying;
                    self.abilities_dirty = true;
                    self.fly_toggle_timer = 0;
                }
            }
        }

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
            if !self.is_sneaking {
                self.is_sneaking = true;
                self.bounding_box.max_y = self.bounding_box.min_y + Self::current_height(true);
            }
        } else if self.is_sneaking {
            self.is_sneaking = false;
            self.bounding_box.max_y = self.bounding_box.min_y + Self::current_height(false);
        }

        // Push out of solid full cubes (MCP `EntityPlayerSP` lines 795-798)
        let offset_w = (Self::WIDTH as f64) * 0.35;
        self.push_out_of_blocks(world, self.pos.x - offset_w, self.bounding_box.min_y + 0.5, self.pos.z + offset_w);
        self.push_out_of_blocks(world, self.pos.x - offset_w, self.bounding_box.min_y + 0.5, self.pos.z - offset_w);
        self.push_out_of_blocks(world, self.pos.x + offset_w, self.bounding_box.min_y + 0.5, self.pos.z - offset_w);
        self.push_out_of_blocks(world, self.pos.x + offset_w, self.bounding_box.min_y + 0.5, self.pos.z + offset_w);

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

        // Jump processing (MCP `EntityLivingBase.onLivingUpdate` lines 2008-2027)
        if input.jump {
            if self.is_in_water || self.is_in_lava {
                self.motion.y += 0.03999999910593033;
            } else if self.on_ground && self.jump_ticks == 0 {
                let mut jump_vel = 0.41999998688697815;
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
                self.jump_ticks = 10;
            }
        } else {
            self.jump_ticks = 0;
        }

        // Flight vertical motion (MCP `EntityPlayerSP` lines 848-859)
        if self.capabilities.is_flying {
            if input.sneak {
                self.motion.y -= (self.capabilities.fly_speed * 3.0) as f64;
            }
            if input.jump {
                self.motion.y += (self.capabilities.fly_speed * 3.0) as f64;
            }
        }

        // Travel input scale matching MCP `EntityLivingBase.onLivingUpdate` lines 2031-2032
        strafe *= 0.98;
        forward *= 0.98;

        // Move with heading
        if self.capabilities.is_flying {
            let jump_factor = self.capabilities.fly_speed * if self.is_sprinting { 2.0 } else { 1.0 };
            self.move_flying(strafe, forward, jump_factor);
            self.move_entity(world, self.motion.x, self.motion.y, self.motion.z);
            self.motion.y *= 0.6;
            self.motion.x *= 0.91;
            self.motion.z *= 0.91;
        } else if self.is_in_water {
            self.move_flying(strafe, forward, 0.02);
            let prev_y = self.pos.y;
            self.move_entity(world, self.motion.x, self.motion.y, self.motion.z);
            self.motion.x *= 0.80000001192092896;
            self.motion.y *= 0.800000011920929;
            self.motion.z *= 0.80000001192092896;
            self.motion.y -= 0.02;

            // MCP `EntityLivingBase.java` line 1730:
            // Step out of water onto bank only when horizontal collision occurs AND target space is air (not liquid, no blocks)
            if self.collided_horizontally {
                let offset_bb = self.get_aabb().offset(
                    self.motion.x,
                    self.motion.y + 0.6000000238418579 - (self.pos.y - prev_y),
                    self.motion.z,
                );
                if world.get_colliding_bounding_boxes(&offset_bb).is_empty()
                    && !is_any_liquid_in_aabb(world, &offset_bb)
                {
                    self.motion.y = 0.30000001192092896;
                }
            }
        } else if self.is_in_lava {
            self.move_flying(strafe, forward, 0.02);
            let prev_y = self.pos.y;
            self.move_entity(world, self.motion.x, self.motion.y, self.motion.z);
            self.motion.x *= 0.5;
            self.motion.y *= 0.5;
            self.motion.z *= 0.5;
            self.motion.y -= 0.02;

            if self.collided_horizontally {
                let offset_bb = self.get_aabb().offset(
                    self.motion.x,
                    self.motion.y + 0.6000000238418579 - (self.pos.y - prev_y),
                    self.motion.z,
                );
                if world.get_colliding_bounding_boxes(&offset_bb).is_empty()
                    && !is_any_liquid_in_aabb(world, &offset_bb)
                {
                    self.motion.y = 0.30000001192092896;
                }
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

            // MCP `BlockSlime.onEntityCollidedWithBlock`:
            // Walking on slime slows horizontal speed down to ~0.4x
            if self.on_ground {
                let block_x = self.pos.x.floor() as i32;
                let block_y = (self.pos.y - 0.20000000298023224).floor() as i32;
                let block_z = self.pos.z.floor() as i32;
                let block_under = world.block_at(block_x, block_y, block_z).id();
                if block_under == 165 && !self.is_sneaking && self.motion.y.abs() < 0.1 {
                    let d0 = 0.4 + self.motion.y.abs() * 0.2;
                    self.motion.x *= d0;
                    self.motion.z *= d0;
                }
            }
        }

        // Distance walked on ground for view bobbing (MCP `Entity.java` line 872)
        let dx = self.pos.x - self.prev_pos.x;
        let dz = self.pos.z - self.prev_pos.z;
        let d_horiz = (dx * dx + dz * dz).sqrt();
        if self.on_ground {
            self.distance_walked_modified += (d_horiz * 0.6) as f32;
        }

        // View bobbing calculation (MCP `EntityPlayer.onLivingUpdate` lines 635-654)
        let f_motion = (self.motion.x * self.motion.x + self.motion.z * self.motion.z).sqrt() as f32;
        let mut f_bob = if f_motion > 0.1 { 0.1 } else { f_motion };
        if !self.on_ground {
            f_bob = 0.0;
        }
        self.camera_yaw += (f_bob - self.camera_yaw) * 0.4;

        let mut f1_pitch = (-self.motion.y * 0.20000000298023224).atan() as f32 * 15.0;
        if self.on_ground {
            f1_pitch = 0.0;
        }
        self.camera_pitch += (f1_pitch - self.camera_pitch) * 0.8;

        // Landing disables flying in creative/survival mode, but NEVER in spectator mode (MCP `EntityPlayerSP` line 904)
        if self.on_ground && self.capabilities.is_flying && !self.is_spectator {
            self.capabilities.is_flying = false;
            self.abilities_dirty = true;
        }
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

        let mut aabb = self.bounding_box;
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
            let d7 = y;
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
            } else {
                x = d11;
                y = d7;
                z = d8;
            }
        }

        // Apply updated bounding box and derived position (MCP `Entity.resetPositionToBB`)
        self.bounding_box = aabb;
        self.reset_position_to_bb();

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
