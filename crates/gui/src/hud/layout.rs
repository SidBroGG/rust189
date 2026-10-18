use super::element::{HudAnchor, HudElementConfig};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub const DEFAULT_PROFILE_NAME: &str = "Default";
pub const COMPETITIVE_PROFILE_NAME: &str = "Competitive PvP";
pub const MINIMAL_PROFILE_NAME: &str = "Minimal";

/// Configuration storing HUD element positions and styling across multiple profiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudLayoutConfig {
    pub active_profile: String,
    pub grid_snap_size: u32, // 0 = Free, 4, 8, 16, 32
    pub profiles: BTreeMap<String, BTreeMap<String, HudElementConfig>>,
}

impl Default for HudLayoutConfig {
    fn default() -> Self {
        let mut profiles = BTreeMap::new();
        profiles.insert(DEFAULT_PROFILE_NAME.to_string(), create_default_elements());
        profiles.insert(
            COMPETITIVE_PROFILE_NAME.to_string(),
            create_competitive_elements(),
        );
        profiles.insert(MINIMAL_PROFILE_NAME.to_string(), create_minimal_elements());

        Self {
            active_profile: DEFAULT_PROFILE_NAME.to_string(),
            grid_snap_size: 8,
            profiles,
        }
    }
}

impl HudLayoutConfig {
    /// Loads layout from a TOML file. Falls back to default if file does not exist or fails to parse.
    pub fn load_or_default(path: &Path) -> Self {
        if !path.exists() {
            let def = Self::default();
            let _ = def.save_to_file(path);
            return def;
        }

        match std::fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<Self>(&content) {
                Ok(mut config) => {
                    // Ensure active profile exists
                    if !config.profiles.contains_key(&config.active_profile) {
                        config.active_profile = DEFAULT_PROFILE_NAME.to_string();
                    }
                    // Ensure all elements exist in active profile
                    let defaults = create_default_elements();
                    if let Some(active_map) = config.profiles.get_mut(&config.active_profile) {
                        for (k, v) in defaults {
                            active_map.entry(k).or_insert(v);
                        }
                    }
                    config
                }
                Err(err) => {
                    tracing::warn!(error = %err, "Failed to parse hud_layout.toml, using defaults");
                    Self::default()
                }
            },
            Err(err) => {
                tracing::warn!(error = %err, "Failed to read hud_layout.toml, using defaults");
                Self::default()
            }
        }
    }

    /// Saves the current configuration to the given TOML file path.
    pub fn save_to_file(&self, path: &Path) -> Result<(), anyhow::Error> {
        let toml_str = toml::to_string_pretty(self)
            .context("Failed to serialize HudLayoutConfig to TOML")?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        std::fs::write(path, toml_str).context("Failed to write hud_layout.toml")?;
        Ok(())
    }

    /// Returns a reference to the active profile's elements map.
    pub fn active_elements(&self) -> &BTreeMap<String, HudElementConfig> {
        self.profiles
            .get(&self.active_profile)
            .or_else(|| self.profiles.get(DEFAULT_PROFILE_NAME))
            .expect("Default profile must exist")
    }

    /// Returns a mutable reference to the active profile's elements map.
    pub fn active_elements_mut(&mut self) -> &mut BTreeMap<String, HudElementConfig> {
        if !self.profiles.contains_key(&self.active_profile) {
            self.active_profile = DEFAULT_PROFILE_NAME.to_string();
        }
        self.profiles
            .get_mut(&self.active_profile)
            .expect("Active profile must exist")
    }

    /// Returns an element config by ID in the active profile.
    pub fn get_element(&self, id: &str) -> Option<&HudElementConfig> {
        self.active_elements().get(id)
    }

    /// Returns a mutable element config by ID in the active profile.
    pub fn get_element_mut(&mut self, id: &str) -> Option<&mut HudElementConfig> {
        self.active_elements_mut().get_mut(id)
    }

    /// Switches the active profile. If the profile does not exist, creates it from default.
    pub fn switch_profile(&mut self, name: &str) {
        if !self.profiles.contains_key(name) {
            self.profiles
                .insert(name.to_string(), create_default_elements());
        }
        self.active_profile = name.to_string();
    }

    /// Creates a new profile by cloning an existing one or using defaults.
    pub fn create_profile(&mut self, name: &str, clone_from: Option<&str>) {
        let base = if let Some(from) = clone_from {
            self.profiles
                .get(from)
                .cloned()
                .unwrap_or_else(create_default_elements)
        } else {
            create_default_elements()
        };
        self.profiles.insert(name.to_string(), base);
        self.active_profile = name.to_string();
    }

    /// Deletes a profile, preventing deletion of the "Default" profile.
    pub fn delete_profile(&mut self, name: &str) -> bool {
        if name == DEFAULT_PROFILE_NAME || self.profiles.len() <= 1 {
            return false;
        }
        self.profiles.remove(name);
        if self.active_profile == name {
            self.active_profile = DEFAULT_PROFILE_NAME.to_string();
        }
        true
    }

    /// Resets the current active profile to default positions.
    pub fn reset_active_profile(&mut self) {
        let defaults = create_default_elements();
        self.profiles.insert(self.active_profile.clone(), defaults);
    }
}

/// Creates default elements mapping with authentic Minecraft layout.
pub fn create_default_elements() -> BTreeMap<String, HudElementConfig> {
    let mut m = BTreeMap::new();

    // 1. Crosshair
    let mut crosshair = HudElementConfig::new(
        "crosshair",
        "Crosshair",
        HudAnchor::Center,
        [0.0, 0.0],
    );
    crosshair.bg_color = [0, 0, 0, 0];
    crosshair.border_color = [0, 0, 0, 0];
    m.insert("crosshair".to_string(), crosshair);

    // 2. Hotbar
    let mut hotbar = HudElementConfig::new(
        "hotbar",
        "Hotbar",
        HudAnchor::BottomCenter,
        [0.0, -22.0],
    );
    hotbar.bg_color = [18, 20, 24, 210];
    hotbar.border_color = [70, 75, 88, 190];
    m.insert("hotbar".to_string(), hotbar);

    // 3. Vitals (Health, Armor, Food, Air)
    let mut vitals = HudElementConfig::new(
        "vitals",
        "Health & Hunger",
        HudAnchor::BottomCenter,
        [0.0, -48.0],
    );
    vitals.bg_color = [0, 0, 0, 0];
    vitals.border_color = [0, 0, 0, 0];
    m.insert("vitals".to_string(), vitals);

    // 4. Experience Bar
    let mut exp_bar = HudElementConfig::new(
        "exp_bar",
        "Experience Bar",
        HudAnchor::BottomCenter,
        [0.0, -26.0],
    );
    exp_bar.bg_color = [0, 0, 0, 0];
    exp_bar.border_color = [0, 0, 0, 0];
    m.insert("exp_bar".to_string(), exp_bar);

    // 5. Boss Bar
    let mut boss_bar = HudElementConfig::new(
        "boss_bar",
        "Boss Health Bar",
        HudAnchor::TopCenter,
        [0.0, 16.0],
    );
    boss_bar.bg_color = [12, 14, 18, 180];
    boss_bar.border_color = [65, 70, 80, 180];
    m.insert("boss_bar".to_string(), boss_bar);

    // 6. Scoreboard Sidebar
    let mut scoreboard = HudElementConfig::new(
        "scoreboard",
        "Scoreboard",
        HudAnchor::CenterRight,
        [-12.0, 0.0],
    );
    scoreboard.bg_color = [0, 0, 0, 160];
    scoreboard.border_color = [55, 60, 70, 150];
    m.insert("scoreboard".to_string(), scoreboard);

    // 7. Chat Box
    let mut chat = HudElementConfig::new(
        "chat",
        "Chat Box",
        HudAnchor::BottomLeft,
        [14.0, -48.0],
    );
    chat.bg_color = [0, 0, 0, 130];
    chat.border_color = [50, 55, 65, 140];
    m.insert("chat".to_string(), chat);

    // 8. Keystrokes & CPS
    let mut keystrokes = HudElementConfig::new(
        "keystrokes",
        "Keystrokes & CPS",
        HudAnchor::CenterLeft,
        [16.0, -50.0],
    );
    keystrokes.bg_color = [14, 16, 22, 210];
    keystrokes.border_color = [60, 68, 82, 190];
    m.insert("keystrokes".to_string(), keystrokes);

    // 9. FPS & Frametime Sparkline
    let mut fps_ping = HudElementConfig::new(
        "fps_ping",
        "FPS & Telemetry",
        HudAnchor::TopLeft,
        [14.0, 14.0],
    );
    fps_ping.bg_color = [14, 16, 22, 210];
    fps_ping.border_color = [60, 68, 82, 190];
    m.insert("fps_ping".to_string(), fps_ping);

    // 10. Armor Status
    let mut armor_status = HudElementConfig::new(
        "armor_status",
        "Armor Status",
        HudAnchor::BottomRight,
        [-16.0, -40.0],
    );
    armor_status.bg_color = [14, 16, 22, 210];
    armor_status.border_color = [60, 68, 82, 190];
    m.insert("armor_status".to_string(), armor_status);

    // 11. Potion Effects
    let mut potion_effects = HudElementConfig::new(
        "potion_effects",
        "Potion Effects",
        HudAnchor::TopRight,
        [-16.0, 16.0],
    );
    potion_effects.bg_color = [14, 16, 22, 210];
    potion_effects.border_color = [60, 68, 82, 190];
    m.insert("potion_effects".to_string(), potion_effects);

    // 12. Coordinates & Compass Direction
    let mut coords = HudElementConfig::new(
        "coords_direction",
        "Coordinates & Compass",
        HudAnchor::TopLeft,
        [14.0, 160.0],
    );
    coords.bg_color = [14, 16, 22, 210];
    coords.border_color = [60, 68, 82, 190];
    m.insert("coords_direction".to_string(), coords);

    m
}

/// Creates competitive PvP profile with keystrokes, CPS and armor status prominently placed.
pub fn create_competitive_elements() -> BTreeMap<String, HudElementConfig> {
    let mut m = create_default_elements();

    if let Some(k) = m.get_mut("keystrokes") {
        k.offset = [16.0, 20.0];
        k.anchor = HudAnchor::CenterLeft;
        k.scale = 1.05;
    }

    if let Some(a) = m.get_mut("armor_status") {
        a.offset = [-16.0, -20.0];
        a.scale = 1.0;
    }

    if let Some(p) = m.get_mut("potion_effects") {
        p.offset = [-16.0, 20.0];
    }

    m
}

/// Creates minimal profile with only essential vanilla elements visible.
pub fn create_minimal_elements() -> BTreeMap<String, HudElementConfig> {
    let mut m = create_default_elements();

    if let Some(k) = m.get_mut("keystrokes") {
        k.visible = false;
    }
    if let Some(a) = m.get_mut("armor_status") {
        a.visible = false;
    }
    if let Some(c) = m.get_mut("coords_direction") {
        c.visible = false;
    }

    m
}
