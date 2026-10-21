use crate::block::BlockState;
use glam::Vec3;
use std::f32::consts::PI;

/// Type of chest block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestType {
    Normal,
    Trapped,
    Ender,
}

/// Double chest half type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoubleChestType {
    Single,
    Left,
    Right,
}

/// Block entity state for single, double, and ender chests.
#[derive(Debug, Clone)]
pub struct ChestBlockEntity {
    pub chest_type: ChestType,
    pub facing: u8, // 2=North, 3=South, 4=West, 5=East
    pub double_type: DoubleChestType,
    pub lid_angle: f32,
    pub prev_lid_angle: f32,
    pub num_players_using: i32,
}

impl ChestBlockEntity {
    pub fn new(chest_type: ChestType, facing: u8, double_type: DoubleChestType) -> Self {
        Self {
            chest_type,
            facing,
            double_type,
            lid_angle: 0.0,
            prev_lid_angle: 0.0,
            num_players_using: 0,
        }
    }

    /// 20 TPS physics tick for lid opening/closing.
    pub fn tick(&mut self) {
        self.prev_lid_angle = self.lid_angle;
        let delta = 0.1f32;
        if self.num_players_using > 0 {
            self.lid_angle = (self.lid_angle + delta).min(1.0);
        } else {
            self.lid_angle = (self.lid_angle - delta).max(0.0);
        }
    }

    /// Computes authentic MCP 1.8.9 cubic eased lid angle [0.0..1.0].
    /// `f = 1.0 - (1.0 - f)^3`
    #[inline]
    pub fn interpolated_lid_angle(&self, partial_ticks: f32) -> f32 {
        let raw = self.prev_lid_angle + (self.lid_angle - self.prev_lid_angle) * partial_ticks;
        let f = 1.0 - raw.clamp(0.0, 1.0);
        1.0 - f * f * f
    }

    /// Converts interpolated lid angle to rotation in radians.
    #[inline]
    pub fn lid_rotation_radians(&self, partial_ticks: f32) -> f32 {
        -(self.interpolated_lid_angle(partial_ticks) * (PI / 2.0))
    }

    /// True if the chest is completely closed and idle.
    #[inline]
    pub fn is_static_closed(&self) -> bool {
        self.num_players_using <= 0 && self.lid_angle <= 0.0 && self.prev_lid_angle <= 0.0
    }
}

/// Block entity for standing and wall signs.
#[derive(Debug, Clone)]
pub struct SignBlockEntity {
    pub lines: [String; 4],
    pub is_wall: bool,
    pub rotation: u8, // 0..15 for floor standing, 2..5 for wall
}

impl SignBlockEntity {
    pub fn new(is_wall: bool, rotation: u8) -> Self {
        Self {
            lines: [String::new(), String::new(), String::new(), String::new()],
            is_wall,
            rotation,
        }
    }
}

/// Block entity for decorative banners with pattern layers.
#[derive(Debug, Clone)]
pub struct BannerBlockEntity {
    pub base_color: u8, // 0..15 EnumDyeColor
    pub patterns: Vec<(String, u8)>, // (pattern_code, dye_color)
    pub is_wall: bool,
    pub rotation: u8,
}

impl BannerBlockEntity {
    pub fn new(base_color: u8, is_wall: bool, rotation: u8) -> Self {
        Self {
            base_color,
            patterns: Vec::new(),
            is_wall,
            rotation,
        }
    }

    /// Maps two-character pattern codes to vanilla asset texture filenames.
    pub fn pattern_texture_name(code: &str) -> &'static str {
        match code {
            "b" => "base",
            "bs" => "stripe_bottom",
            "ts" => "stripe_top",
            "ls" => "stripe_left",
            "rs" => "stripe_right",
            "cs" => "stripe_center",
            "ms" => "stripe_middle",
            "dls" => "stripe_downleft",
            "drs" => "stripe_downright",
            "cr" => "cross",
            "sc" => "straight_cross",
            "ld" => "diagonal_left",
            "rud" => "diagonal_right",
            "lud" => "diagonal_up_left",
            "rd" => "diagonal_up_right",
            "vh" => "half_vertical",
            "vhr" => "half_vertical_right",
            "hh" => "half_horizontal",
            "hhr" => "half_horizontal_bottom",
            "bo" => "border",
            "cbo" => "curly_border",
            "bt" => "triangle_bottom",
            "tt" => "triangle_top",
            "bts" => "triangles_bottom",
            "tts" => "triangles_top",
            "mc" => "circle",
            "mr" => "rhombus",
            "bl" => "square_bottom_left",
            "br" => "square_bottom_right",
            "tl" => "square_top_left",
            "tr" => "square_top_right",
            "cre" => "creeper",
            "sku" => "skull",
            "flo" => "flower",
            "moj" => "mojang",
            "gra" => "gradient",
            "gru" => "gradient_up",
            "bri" => "bricks",
            "ss" => "small_stripes",
            _ => "base",
        }
    }

    /// Computes wind wave flapping angle in radians for banner slate.
    pub fn slate_rotation_radians(pos: (i32, i32, i32), world_time: i64, partial_ticks: f32) -> f32 {
        let f3 = (pos.0 * 7 + pos.1 * 9 + pos.2 * 13) as f32 + world_time as f32 + partial_ticks;
        (-0.0125 + 0.01 * (f3 * PI * 0.02).cos()) * PI
    }
}

/// Block entity for mob and player skulls.
#[derive(Debug, Clone)]
pub struct SkullBlockEntity {
    pub skull_type: u8, // 0=Skeleton, 1=Wither, 2=Zombie, 3=Player, 4=Creeper
    pub rotation: u8,   // 0..15 on ground
    pub facing: u8,     // 1=Up, 2=North, 3=South, 4=West, 5=East
    pub owner_name: Option<String>,
}

impl SkullBlockEntity {
    pub fn new(skull_type: u8, rotation: u8, facing: u8) -> Self {
        Self {
            skull_type,
            rotation,
            facing,
            owner_name: None,
        }
    }

    #[inline]
    pub fn rotation_degrees(&self) -> f32 {
        if self.facing == 1 {
            (self.rotation as f32 * 360.0) / 16.0
        } else {
            match self.facing {
                2 => 0.0,
                3 => 180.0,
                4 => 270.0,
                _ => 90.0,
            }
        }
    }
}

/// Block entity for enchantment tables with floating, animating book.
#[derive(Debug, Clone)]
pub struct EnchantmentTableBlockEntity {
    pub tick_count: u32,
    pub page_flip: f32,
    pub page_flip_prev: f32,
    pub flip_turn: f32,
    pub flip_turn_prev: f32,
    pub book_spread: f32,
    pub book_spread_prev: f32,
    pub book_rotation: f32,
    pub book_rotation_prev: f32,
    pub target_rotation: f32,
}

impl Default for EnchantmentTableBlockEntity {
    fn default() -> Self {
        Self::new()
    }
}

impl EnchantmentTableBlockEntity {
    pub fn new() -> Self {
        Self {
            tick_count: 0,
            page_flip: 0.0,
            page_flip_prev: 0.0,
            flip_turn: 0.0,
            flip_turn_prev: 0.0,
            book_spread: 0.0,
            book_spread_prev: 0.0,
            book_rotation: 0.0,
            book_rotation_prev: 0.0,
            target_rotation: 0.0,
        }
    }

    /// 20 TPS tick based on MCP TileEntityEnchantmentTable.
    pub fn tick(&mut self, closest_player: Option<Vec3>, pos: Vec3) {
        self.book_spread_tick(closest_player, pos);
    }

    fn book_spread_tick(&mut self, closest_player: Option<Vec3>, pos: Vec3) {
        self.book_spread_prev = self.book_spread;
        self.book_rotation_prev = self.book_rotation;

        let is_near = if let Some(player_pos) = closest_player {
            let dx = (player_pos.x - (pos.x + 0.5)) as f64;
            let dy = (player_pos.y - (pos.y + 0.5)) as f64;
            let dz = (player_pos.z - (pos.z + 0.5)) as f64;
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq <= 9.0 {
                self.target_rotation = dz.atan2(dx) as f32;
                true
            } else {
                false
            }
        } else {
            false
        };

        if is_near {
            self.book_spread += 0.1;
            if self.book_spread < 0.5 || (self.tick_count % 40 == 0) {
                let f1 = self.flip_turn;
                let mut seed = self.tick_count.wrapping_mul(1103515245).wrapping_add(12345);
                loop {
                    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                    let delta = (((seed >> 16) & 3) as i32) - (((seed >> 18) & 3) as i32);
                    self.flip_turn += delta as f32;
                    if (self.flip_turn - f1).abs() > 0.01 {
                        break;
                    }
                }
            }
        } else {
            self.target_rotation += 0.02;
            self.book_spread -= 0.1;
        }

        // Normalize angle deltas
        while self.book_rotation >= PI {
            self.book_rotation -= PI * 2.0;
        }
        while self.book_rotation < -PI {
            self.book_rotation += PI * 2.0;
        }
        while self.target_rotation >= PI {
            self.target_rotation -= PI * 2.0;
        }
        while self.target_rotation < -PI {
            self.target_rotation += PI * 2.0;
        }

        let mut diff = self.target_rotation - self.book_rotation;
        while diff >= PI {
            diff -= PI * 2.0;
        }
        while diff < -PI {
            diff += PI * 2.0;
        }

        self.book_rotation += diff * 0.4;
        self.book_spread = self.book_spread.clamp(0.0, 1.0);
        self.tick_count = self.tick_count.wrapping_add(1);

        self.page_flip_prev = self.page_flip;
        let mut f = (self.flip_turn - self.page_flip) * 0.4;
        f = f.clamp(-0.2, 0.2);
        self.flip_turn_prev += (f - self.flip_turn_prev) * 0.9;
        self.page_flip += self.flip_turn_prev;
    }

    #[inline]
    pub fn interpolated_spread(&self, partial_ticks: f32) -> f32 {
        (self.book_spread_prev + (self.book_spread - self.book_spread_prev) * partial_ticks)
            .clamp(0.0, 1.0)
    }

    #[inline]
    pub fn interpolated_rotation(&self, partial_ticks: f32) -> f32 {
        let mut diff = self.book_rotation - self.book_rotation_prev;
        while diff >= PI {
            diff -= PI * 2.0;
        }
        while diff < -PI {
            diff += PI * 2.0;
        }
        self.book_rotation_prev + diff * partial_ticks
    }
}

/// Segment of beacon vertical beam of light.
#[derive(Debug, Clone, PartialEq)]
pub struct BeaconBeamSegment {
    pub height: i32,
    pub color: [f32; 3],
}

/// Block entity for beacon pyramid and vertical beam.
#[derive(Debug, Clone)]
pub struct BeaconBlockEntity {
    pub is_active: bool,
    pub levels: i32,
    pub segments: Vec<BeaconBeamSegment>,
}

impl Default for BeaconBlockEntity {
    fn default() -> Self {
        Self::new()
    }
}

impl BeaconBlockEntity {
    pub fn new() -> Self {
        Self {
            is_active: false,
            levels: 0,
            segments: Vec::new(),
        }
    }

    /// Recalculates vertical beam segments and stained glass color mixing.
    pub fn recalculate_beam<F>(&mut self, start_y: i32, mut get_block: F)
    where
        F: FnMut(i32) -> (u16, u8),
    {
        self.segments.clear();
        let mut current_segment = BeaconBeamSegment {
            height: 0,
            color: [1.0, 1.0, 1.0], // Default white
        };
        let mut is_first_glass = true;
        let mut active = true;

        for y in (start_y + 1)..256 {
            let (block_id, meta) = get_block(y);
            if block_id == 0 {
                // Air
                current_segment.height += 1;
                continue;
            }

            // Stained glass (95) or stained glass pane (160)
            if block_id == 95 || block_id == 160 {
                let glass_rgb = dye_color_rgb(meta);
                let new_rgb = if is_first_glass {
                    glass_rgb
                } else {
                    [
                        (current_segment.color[0] + glass_rgb[0]) * 0.5,
                        (current_segment.color[1] + glass_rgb[1]) * 0.5,
                        (current_segment.color[2] + glass_rgb[2]) * 0.5,
                    ]
                };
                is_first_glass = false;

                if current_segment.height > 0 {
                    if (new_rgb[0] - current_segment.color[0]).abs() < 0.001
                        && (new_rgb[1] - current_segment.color[1]).abs() < 0.001
                        && (new_rgb[2] - current_segment.color[2]).abs() < 0.001
                    {
                        current_segment.height += 1;
                    } else {
                        self.segments.push(current_segment);
                        current_segment = BeaconBeamSegment {
                            height: 1,
                            color: new_rgb,
                        };
                    }
                } else {
                    current_segment.color = new_rgb;
                    current_segment.height = 1;
                }
                continue;
            }

            // Transparent blocks like torches, signs, glass don't block beam
            if matches!(block_id, 20 | 50 | 63 | 68 | 8 | 9 | 102) {
                current_segment.height += 1;
                continue;
            }

            // Opaque block blocks beam
            active = false;
            break;
        }

        if active && current_segment.height > 0 {
            self.segments.push(current_segment);
            self.is_active = true;
        } else {
            self.is_active = false;
            self.segments.clear();
        }
    }
}

/// Block entity for End Portal blocks (119) with parallax starfield shader.
#[derive(Debug, Clone)]
pub struct EndPortalBlockEntity {
    pub active: bool,
}

impl Default for EndPortalBlockEntity {
    fn default() -> Self {
        Self { active: true }
    }
}

/// Block entity for monster spawner cage with spinning mob.
#[derive(Debug, Clone)]
pub struct MobSpawnerBlockEntity {
    pub mob_type: String,
    pub mob_rotation: f64,
    pub prev_mob_rotation: f64,
    pub spawn_delay: i32,
}

impl Default for MobSpawnerBlockEntity {
    fn default() -> Self {
        Self::new("Pig")
    }
}

impl MobSpawnerBlockEntity {
    pub fn new(mob_type: &str) -> Self {
        Self {
            mob_type: mob_type.to_string(),
            mob_rotation: 0.0,
            prev_mob_rotation: 0.0,
            spawn_delay: 200,
        }
    }

    pub fn tick(&mut self) {
        self.prev_mob_rotation = self.mob_rotation;
        let speed = 1000.0 / (self.spawn_delay as f64 + 200.0);
        self.mob_rotation = (self.mob_rotation + speed) % 360.0;
    }

    #[inline]
    pub fn interpolated_rotation_degrees(&self, partial_ticks: f32) -> f32 {
        let r = self.prev_mob_rotation
            + (self.mob_rotation - self.prev_mob_rotation) * (partial_ticks as f64);
        (r * 10.0) as f32
    }
}

/// Block entity for moving piston blocks (36).
#[derive(Debug, Clone)]
pub struct PistonBlockEntity {
    pub piston_state: BlockState,
    pub facing: u8, // 0=Down, 1=Up, 2=North, 3=South, 4=West, 5=East
    pub extending: bool,
    pub progress: f32,
    pub last_progress: f32,
}

impl PistonBlockEntity {
    pub fn new(piston_state: BlockState, facing: u8, extending: bool) -> Self {
        Self {
            piston_state,
            facing,
            extending,
            progress: 0.0,
            last_progress: 0.0,
        }
    }

    pub fn tick(&mut self) {
        self.last_progress = self.progress;
        if self.progress < 1.0 {
            // Pistons in 1.8.9 complete movement in 2 ticks (progress 0.5 per tick)
            self.progress = (self.progress + 0.5).min(1.0);
        }
    }

    #[inline]
    pub fn get_progress(&self, partial_ticks: f32) -> f32 {
        (self.last_progress + (self.progress - self.last_progress) * partial_ticks).clamp(0.0, 1.0)
    }

    /// Computes translation offset vector for moving block and piston head.
    pub fn get_offset(&self, partial_ticks: f32) -> Vec3 {
        let p = self.get_progress(partial_ticks);
        let factor = if self.extending { p - 1.0 } else { 1.0 - p };
        let dir = match self.facing {
            0 => Vec3::new(0.0, -1.0, 0.0),
            1 => Vec3::new(0.0, 1.0, 0.0),
            2 => Vec3::new(0.0, 0.0, -1.0),
            3 => Vec3::new(0.0, 0.0, 1.0),
            4 => Vec3::new(-1.0, 0.0, 0.0),
            _ => Vec3::new(1.0, 0.0, 0.0),
        };
        dir * factor
    }
}

/// Unified enum for all 1.8.9 block entities.
#[derive(Debug, Clone)]
pub enum BlockEntity {
    Chest(ChestBlockEntity),
    Sign(SignBlockEntity),
    Banner(BannerBlockEntity),
    Skull(SkullBlockEntity),
    EnchantmentTable(EnchantmentTableBlockEntity),
    Beacon(BeaconBlockEntity),
    EndPortal(EndPortalBlockEntity),
    MobSpawner(MobSpawnerBlockEntity),
    Piston(PistonBlockEntity),
}

/// Returns RGB color for vanilla 1.8.9 EnumDyeColor values 0..15.
pub fn dye_color_rgb(color: u8) -> [f32; 3] {
    match color & 15 {
        0 => [1.0, 1.0, 1.0],      // White
        1 => [0.85, 0.5, 0.2],     // Orange
        2 => [0.7, 0.3, 0.85],     // Magenta
        3 => [0.4, 0.6, 0.85],     // Light Blue
        4 => [0.9, 0.85, 0.2],     // Yellow
        5 => [0.5, 0.8, 0.1],      // Lime
        6 => [0.95, 0.5, 0.65],    // Pink
        7 => [0.3, 0.3, 0.3],      // Gray
        8 => [0.6, 0.6, 0.6],      // Silver
        9 => [0.3, 0.6, 0.7],      // Cyan
        10 => [0.5, 0.25, 0.7],    // Purple
        11 => [0.2, 0.3, 0.7],     // Blue
        12 => [0.4, 0.3, 0.2],     // Brown
        13 => [0.4, 0.5, 0.2],     // Green
        14 => [0.6, 0.2, 0.2],     // Red
        15 => [0.1, 0.1, 0.1],     // Black
        _ => [1.0, 1.0, 1.0],
    }
}
