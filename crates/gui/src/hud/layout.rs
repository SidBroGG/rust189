use super::element::HudElementConfig;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub const DEFAULT_PROFILE_NAME: &str = "Default";
pub const COMPETITIVE_PROFILE_NAME: &str = "Competitive PvP";
pub const MINIMAL_PROFILE_NAME: &str = "Minimal";

fn default_global_opacity() -> f32 {
    0.85
}

/// Configuration storing HUD element relative grid positions and styling across multiple profiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudLayoutConfig {
    pub active_profile: String,
    pub grid_snap_size: u32, // 0 = Free, 8, 16, 24, 32
    #[serde(default = "default_global_opacity")]
    pub global_opacity: f32, // 0.0 .. 1.0, master opacity slider for all HUD backgrounds & borders
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
            grid_snap_size: 16,
            global_opacity: default_global_opacity(),
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
                    if !config.profiles.contains_key(&config.active_profile) {
                        config.active_profile = DEFAULT_PROFILE_NAME.to_string();
                    }
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

    /// Resets the current active profile to default relative grid positions.
    pub fn reset_active_profile(&mut self) {
        let defaults = create_default_elements();
        self.profiles.insert(self.active_profile.clone(), defaults);
    }
}

/// Creates default elements mapping with authentic Minecraft relative grid layout.
pub fn create_default_elements() -> BTreeMap<String, HudElementConfig> {
    let mut m = BTreeMap::new();

    // 1. Crosshair
    m.insert(
        "crosshair".to_string(),
        HudElementConfig::new("crosshair", "Crosshair", 0.49, 0.49, 0.02, 0.02),
    );

    // 2. Hotbar & Status (Unified Hotbar + Health hearts + Hunger + Armor + Air + EXP)
    m.insert(
        "hotbar".to_string(),
        HudElementConfig::new("hotbar", "Hotbar & Status", 0.32, 0.88, 0.36, 0.11),
    );

    // 3. Boss Bar
    m.insert(
        "boss_bar".to_string(),
        HudElementConfig::new("boss_bar", "Boss Health Bar", 0.32, 0.03, 0.36, 0.05),
    );

    // 4. Scoreboard Sidebar
    m.insert(
        "scoreboard".to_string(),
        HudElementConfig::new("scoreboard", "Scoreboard", 0.80, 0.32, 0.18, 0.28),
    );

    // 5. Chat Box
    m.insert(
        "chat".to_string(),
        HudElementConfig::new("chat", "Chat Box", 0.02, 0.68, 0.32, 0.26),
    );

    // 6. Keystrokes & CPS
    m.insert(
        "keystrokes".to_string(),
        HudElementConfig::new("keystrokes", "Keystrokes & CPS", 0.02, 0.42, 0.11, 0.18),
    );

    // 7. FPS & Telemetry
    m.insert(
        "fps_ping".to_string(),
        HudElementConfig::new("fps_ping", "FPS & Telemetry", 0.02, 0.02, 0.18, 0.10),
    );

    // 8. Armor Status
    m.insert(
        "armor_status".to_string(),
        HudElementConfig::new("armor_status", "Armor Status", 0.85, 0.82, 0.13, 0.15),
    );

    // 9. Potion Effects
    m.insert(
        "potion_effects".to_string(),
        HudElementConfig::new("potion_effects", "Potion Effects", 0.84, 0.02, 0.14, 0.18),
    );

    // 10. Coordinates & Compass Direction
    m.insert(
        "coords_direction".to_string(),
        HudElementConfig::new("coords_direction", "Coordinates & Compass", 0.02, 0.13, 0.18, 0.10),
    );

    m
}

/// Creates competitive PvP profile with keystrokes, CPS and armor status prominently placed.
pub fn create_competitive_elements() -> BTreeMap<String, HudElementConfig> {
    let mut m = create_default_elements();

    if let Some(k) = m.get_mut("keystrokes") {
        k.rel_x = 0.02;
        k.rel_y = 0.45;
        k.rel_w = 0.13;
        k.rel_h = 0.20;
    }

    if let Some(a) = m.get_mut("armor_status") {
        a.rel_x = 0.84;
        a.rel_y = 0.80;
        a.rel_w = 0.14;
        a.rel_h = 0.17;
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
