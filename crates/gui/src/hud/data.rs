use egui::Color32;
use world::BenchmarkSnapshot;

/// Item representation for Hotbar and Armor Status HUD widgets.
#[derive(Debug, Clone, PartialEq)]
pub struct HudItemInfo {
    pub item_id: i16,
    pub count: i8,
    pub damage: i16,
    pub max_damage: i16,
    pub name: String,
    pub is_enchanted: bool,
}

impl HudItemInfo {
    pub fn new(item_id: i16, count: i8, damage: i16, max_damage: i16, name: impl Into<String>) -> Self {
        Self {
            item_id,
            count,
            damage,
            max_damage,
            name: name.into(),
            is_enchanted: false,
        }
    }

    pub fn with_enchanted(mut self, enchanted: bool) -> Self {
        self.is_enchanted = enchanted;
        self
    }

    /// Remaining durability percentage (1.0 = undamaged, 0.0 = broken).
    #[inline]
    pub fn durability_ratio(&self) -> f32 {
        if self.max_damage <= 0 {
            1.0
        } else {
            let left = (self.max_damage - self.damage).max(0);
            (left as f32 / self.max_damage as f32).clamp(0.0, 1.0)
        }
    }

    /// Remaining durability count (e.g. 520).
    #[inline]
    pub fn durability_remaining(&self) -> i16 {
        if self.max_damage <= 0 {
            0
        } else {
            (self.max_damage - self.damage).max(0)
        }
    }
}

/// Active Potion effect info displayed in HUD.
#[derive(Debug, Clone, PartialEq)]
pub struct HudPotionInfo {
    pub id: u8,
    pub name: String,
    pub amplifier: u8,
    pub duration_ticks: i32,
    pub color: Color32,
}

impl HudPotionInfo {
    pub fn new(id: u8, name: impl Into<String>, amplifier: u8, duration_ticks: i32, color: Color32) -> Self {
        Self {
            id,
            name: name.into(),
            amplifier,
            duration_ticks,
            color,
        }
    }

    /// Formats remaining duration into standard MM:SS string.
    pub fn format_duration(&self) -> String {
        let total_sec = (self.duration_ticks / 20).max(0);
        let mins = total_sec / 60;
        let secs = total_sec % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Formats Roman numeral level (e.g. Level I, II, III).
    pub fn format_level(&self) -> &'static str {
        match self.amplifier {
            0 => "I",
            1 => "II",
            2 => "III",
            3 => "IV",
            4 => "V",
            _ => "+",
        }
    }
}

/// Boss bar entry displayed at top of screen.
#[derive(Debug, Clone, PartialEq)]
pub struct HudBossBarInfo {
    pub name: String,
    pub health_percent: f32, // 0.0 .. 1.0
}

impl HudBossBarInfo {
    pub fn new(name: impl Into<String>, health_percent: f32) -> Self {
        Self {
            name: name.into(),
            health_percent: health_percent.clamp(0.0, 1.0),
        }
    }
}

/// Consolidated live player & telemetry data fed into the HUD renderer every frame.
#[derive(Debug, Clone)]
pub struct HudData {
    // Vitals
    pub health: f32,
    pub max_health: f32,
    pub absorption: f32,
    pub food: u8,
    pub saturation: f32,
    pub air: i16,
    pub is_in_water: bool,
    pub armor_value: u8,
    pub exp_level: i32,
    pub exp_progress: f32,

    // Hotbar
    pub selected_hotbar_slot: usize,
    pub hotbar_items: [Option<HudItemInfo>; 9],

    // Armor & Held item
    pub armor_items: [Option<HudItemInfo>; 4], // 0: helmet, 1: chest, 2: legs, 3: boots
    pub held_item: Option<HudItemInfo>,

    // Keystrokes & CPS
    pub key_w: bool,
    pub key_a: bool,
    pub key_s: bool,
    pub key_d: bool,
    pub key_space: bool,
    pub key_lmb: bool,
    pub key_rmb: bool,
    pub lmb_cps: usize,
    pub rmb_cps: usize,

    // Potions & Boss bars
    pub potion_effects: Vec<HudPotionInfo>,
    pub boss_bars: Vec<HudBossBarInfo>,

    // Player position & orientation
    pub pos: (f64, f64, f64),
    pub look: (f32, f32), // yaw, pitch
    pub biome: String,
    pub dimension: i8,

    // Gamemode & Capabilities
    pub gamemode: u8,
    pub is_creative: bool,

    // Telemetry
    pub benchmark: Option<BenchmarkSnapshot>,
    pub ping_ms: i32,
}

impl Default for HudData {
    fn default() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            absorption: 0.0,
            food: 20,
            saturation: 5.0,
            air: 300,
            is_in_water: false,
            armor_value: 0,
            exp_level: 0,
            exp_progress: 0.0,
            selected_hotbar_slot: 0,
            hotbar_items: Default::default(),
            armor_items: Default::default(),
            held_item: None,
            key_w: false,
            key_a: false,
            key_s: false,
            key_d: false,
            key_space: false,
            key_lmb: false,
            key_rmb: false,
            lmb_cps: 0,
            rmb_cps: 0,
            potion_effects: Vec::new(),
            boss_bars: Vec::new(),
            pos: (0.0, 64.0, 0.0),
            look: (0.0, 0.0),
            biome: "Plains".to_string(),
            dimension: 0,
            gamemode: 0,
            is_creative: false,
            benchmark: None,
            ping_ms: 0,
        }
    }
}

impl HudData {
    /// Generates realistic PvP mock data for testing and visual HUD previewing in editor.
    pub fn mock_pvp_data() -> Self {
        let hotbar_items = [
            Some(HudItemInfo::new(276, 1, 35, 1561, "Diamond Sword").with_enchanted(true)),
            Some(HudItemInfo::new(261, 1, 12, 384, "Bow").with_enchanted(true)),
            Some(HudItemInfo::new(278, 1, 120, 1561, "Diamond Pickaxe")),
            Some(HudItemInfo::new(279, 1, 80, 1561, "Diamond Axe")),
            Some(HudItemInfo::new(277, 1, 10, 1561, "Diamond Shovel")),
            Some(HudItemInfo::new(322, 64, 0, 0, "Golden Apple")),
            Some(HudItemInfo::new(1, 64, 0, 0, "Stone")),
            Some(HudItemInfo::new(5, 64, 0, 0, "Oak Wood Planks")),
            Some(HudItemInfo::new(368, 16, 0, 0, "Ender Pearl")),
        ];

        let armor_items = [
            Some(HudItemInfo::new(310, 1, 45, 363, "Diamond Helmet")),
            Some(HudItemInfo::new(311, 1, 80, 528, "Diamond Chestplate")),
            Some(HudItemInfo::new(312, 1, 60, 495, "Diamond Leggings")),
            Some(HudItemInfo::new(313, 1, 30, 429, "Diamond Boots")),
        ];

        let potion_effects = vec![
            HudPotionInfo::new(1, "Speed", 1, 1420, Color32::from_rgb(124, 175, 255)),
            HudPotionInfo::new(5, "Strength", 0, 960, Color32::from_rgb(255, 85, 85)),
            HudPotionInfo::new(10, "Regeneration", 0, 480, Color32::from_rgb(255, 105, 180)),
        ];

        let boss_bars = vec![
            HudBossBarInfo::new("Ender Dragon", 0.78),
        ];

        Self {
            health: 18.0,
            max_health: 20.0,
            absorption: 4.0,
            food: 19,
            saturation: 4.5,
            air: 280,
            is_in_water: false,
            armor_value: 20,
            exp_level: 42,
            exp_progress: 0.72,
            selected_hotbar_slot: 0,
            hotbar_items,
            armor_items,
            held_item: Some(HudItemInfo::new(276, 1, 35, 1561, "Diamond Sword")),
            key_w: true,
            key_a: false,
            key_s: false,
            key_d: false,
            key_space: false,
            key_lmb: true,
            key_rmb: false,
            lmb_cps: 14,
            rmb_cps: 8,
            potion_effects,
            boss_bars,
            pos: (124.5, 64.0, -89.2),
            look: (-45.0, 12.5),
            biome: "Plains".to_string(),
            dimension: 0,
            gamemode: 0,
            is_creative: false,
            benchmark: None,
            ping_ms: 28,
        }
    }
}
