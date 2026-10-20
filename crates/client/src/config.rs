use anyhow::{Context, Result};
use gui::standardize_key;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Full client configuration stored in `config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub chat: ChatConfig,
    #[serde(default)]
    pub scoreboard: ScoreboardConfig,
    pub auth: AuthConfig,
    pub video: VideoConfig,
    pub audio: AudioConfig,
    pub keybinds: KeybindsConfig,
    #[serde(default)]
    pub mouse: MouseConfig,
    pub network: NetworkConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            chat: ChatConfig::default(),
            scoreboard: ScoreboardConfig::default(),
            auth: AuthConfig::default(),
            video: VideoConfig::default(),
            audio: AudioConfig::default(),
            keybinds: KeybindsConfig::default(),
            mouse: MouseConfig::default(),
            network: NetworkConfig::default(),
        }
    }
}

impl ClientConfig {
    /// Loads client configuration from the given path.
    /// If the configuration file does not exist, writes the default configuration and returns it.
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file at {:?}", path))?;
            let mut config: Self = toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML configuration from {:?}", path))?;
            config.keybinds.standardize();
            Ok(config)
        } else {
            let config = Self::default();
            config.save(path)?;
            Ok(config)
        }
    }

    /// Serializes and writes configuration to a TOML file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize client configuration to TOML")?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write configuration to {:?}", path))?;
        Ok(())
    }
}

/// General client settings (language, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralConfig {
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "en_US".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
        }
    }
}

/// In-game chat settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatConfig {
    #[serde(default = "default_chat_font_size")]
    pub font_size: f32,
}

fn default_chat_font_size() -> f32 {
    13.0
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            font_size: default_chat_font_size(),
        }
    }
}

/// Offline authorization profile settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConfig {
    /// Offline username (e.g. "Steve", "Alex", or competitive IGN).
    pub nickname: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            nickname: "Steve".to_string(),
        }
    }
}

/// Graphics and display settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoConfig {
    /// Field of view in degrees (30.0 - 110.0, default 70.0).
    pub fov: f32,
    /// Render distance in chunks (default 12 for high performance).
    pub render_distance: u32,
    /// Maximum framerate limiter (0 for unlimited).
    pub max_fps: u32,
    /// Vertical synchronization.
    pub vsync: bool,
    /// GUI Scale: 0.0 = Auto, or precise custom scale factor (e.g. 1.0, 1.25, 1.5, 2.0).
    #[serde(default = "default_gui_scale")]
    pub gui_scale: f32,
    /// Fullscreen mode.
    pub fullscreen: bool,
    /// View bobbing animation while walking.
    pub view_bobbing: bool,
    /// Brightness (0.0 = Moody, 1.0 = Bright).
    pub gamma: f32,
    #[serde(default = "default_antialiasing")]
    pub antialiasing: u32,
}

fn default_gui_scale() -> f32 {
    0.0
}

fn default_antialiasing() -> u32 {
    1
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            fov: 70.0,
            render_distance: 12,
            max_fps: 144,
            vsync: false,
            gui_scale: default_gui_scale(),
            fullscreen: false,
            view_bobbing: true,
            gamma: 1.0,
            antialiasing: default_antialiasing(),
        }
    }
}

/// Audio volume channel settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioConfig {
    pub master: f32,
    pub music: f32,
    pub blocks: f32,
    pub players: f32,
    pub hostile: f32,
    pub ambient: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.3,
            blocks: 1.0,
            players: 1.0,
            hostile: 1.0,
            ambient: 0.8,
        }
    }
}

/// Standardized configurable keybindings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeybindsConfig {
    #[serde(default = "default_forward")]
    pub forward: String,
    #[serde(default = "default_back")]
    pub back: String,
    #[serde(default = "default_left")]
    pub left: String,
    #[serde(default = "default_right")]
    pub right: String,
    #[serde(default = "default_jump")]
    pub jump: String,
    #[serde(default = "default_sneak")]
    pub sneak: String,
    #[serde(default = "default_sprint")]
    pub sprint: String,
    #[serde(default = "default_drop_item")]
    pub drop_item: String,
    #[serde(default = "default_inventory")]
    pub inventory: String,
    #[serde(default = "default_chat")]
    pub chat: String,
    #[serde(default = "default_player_list")]
    pub player_list: String,
    #[serde(default = "default_toggle_perspective")]
    pub toggle_perspective: String,
    #[serde(default = "default_zoom")]
    pub zoom: String,
    #[serde(default = "default_fullscreen_toggle")]
    pub fullscreen_toggle: String,
    #[serde(default = "default_toggle_fps")]
    pub toggle_fps: String,
    #[serde(default = "default_toggle_cinematic")]
    pub toggle_cinematic: String,
}

fn default_forward() -> String { "W".to_string() }
fn default_back() -> String { "S".to_string() }
fn default_left() -> String { "A".to_string() }
fn default_right() -> String { "D".to_string() }
fn default_jump() -> String { "SPACE".to_string() }
fn default_sneak() -> String { "LSHIFT".to_string() }
fn default_sprint() -> String { "LCONTROL".to_string() }
fn default_drop_item() -> String { "Q".to_string() }
fn default_inventory() -> String { "E".to_string() }
fn default_chat() -> String { "T".to_string() }
fn default_player_list() -> String { "TAB".to_string() }
fn default_toggle_perspective() -> String { "F5".to_string() }
fn default_zoom() -> String { "C".to_string() }
fn default_fullscreen_toggle() -> String { "F11".to_string() }
fn default_toggle_fps() -> String { "F3".to_string() }
fn default_toggle_cinematic() -> String { "F8".to_string() }

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            forward: default_forward(),
            back: default_back(),
            left: default_left(),
            right: default_right(),
            jump: default_jump(),
            sneak: default_sneak(),
            sprint: default_sprint(),
            drop_item: default_drop_item(),
            inventory: default_inventory(),
            chat: default_chat(),
            player_list: default_player_list(),
            toggle_perspective: default_toggle_perspective(),
            zoom: default_zoom(),
            fullscreen_toggle: default_fullscreen_toggle(),
            toggle_fps: default_toggle_fps(),
            toggle_cinematic: default_toggle_cinematic(),
        }
    }
}

impl KeybindsConfig {
    pub fn standardize(&mut self) {
        self.forward = standardize_key(&self.forward);
        self.back = standardize_key(&self.back);
        self.left = standardize_key(&self.left);
        self.right = standardize_key(&self.right);
        self.jump = standardize_key(&self.jump);
        self.sneak = standardize_key(&self.sneak);
        self.sprint = standardize_key(&self.sprint);
        self.drop_item = standardize_key(&self.drop_item);
        self.inventory = standardize_key(&self.inventory);
        self.chat = standardize_key(&self.chat);
        self.player_list = standardize_key(&self.player_list);
        self.toggle_perspective = standardize_key(&self.toggle_perspective);
        self.zoom = standardize_key(&self.zoom);
        self.fullscreen_toggle = standardize_key(&self.fullscreen_toggle);
        self.toggle_fps = standardize_key(&self.toggle_fps);
        self.toggle_cinematic = standardize_key(&self.toggle_cinematic);
    }
}

/// Mouse sensitivity and camera settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MouseConfig {
    #[serde(default = "default_mouse_sensitivity")]
    pub sensitivity: f32,
    #[serde(default)]
    pub invert_y: bool,
    #[serde(default)]
    pub smooth_camera: bool,
}

fn default_mouse_sensitivity() -> f32 {
    0.5
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            sensitivity: default_mouse_sensitivity(),
            invert_y: false,
            smooth_camera: false,
        }
    }
}

/// Network and multiplayer connection settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkConfig {
    pub tcp_nodelay: bool,
    pub default_server: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            tcp_nodelay: true,
            default_server: None,
        }
    }
}

impl From<&VideoConfig> for gui::VideoSettings {
    fn from(c: &VideoConfig) -> Self {
        Self {
            fov: c.fov,
            render_distance: c.render_distance,
            max_fps: c.max_fps,
            vsync: c.vsync,
            gui_scale: c.gui_scale,
            fullscreen: c.fullscreen,
            view_bobbing: c.view_bobbing,
            gamma: c.gamma,
            antialiasing: c.antialiasing,
        }
    }
}

impl From<&gui::VideoSettings> for VideoConfig {
    fn from(s: &gui::VideoSettings) -> Self {
        Self {
            fov: s.fov,
            render_distance: s.render_distance,
            max_fps: s.max_fps,
            vsync: s.vsync,
            gui_scale: s.gui_scale,
            fullscreen: s.fullscreen,
            view_bobbing: s.view_bobbing,
            gamma: s.gamma,
            antialiasing: s.antialiasing,
        }
    }
}

impl From<&AudioConfig> for gui::AudioSettings {
    fn from(c: &AudioConfig) -> Self {
        Self {
            master: c.master,
            music: c.music,
            blocks: c.blocks,
            players: c.players,
            hostile: c.hostile,
            ambient: c.ambient,
        }
    }
}

impl From<&gui::AudioSettings> for AudioConfig {
    fn from(s: &gui::AudioSettings) -> Self {
        Self {
            master: s.master,
            music: s.music,
            blocks: s.blocks,
            players: s.players,
            hostile: s.hostile,
            ambient: s.ambient,
        }
    }
}

impl From<&KeybindsConfig> for gui::KeybindsSettings {
    fn from(c: &KeybindsConfig) -> Self {
        Self {
            forward: c.forward.clone(),
            back: c.back.clone(),
            left: c.left.clone(),
            right: c.right.clone(),
            jump: c.jump.clone(),
            sneak: c.sneak.clone(),
            sprint: c.sprint.clone(),
            drop_item: c.drop_item.clone(),
            inventory: c.inventory.clone(),
            chat: c.chat.clone(),
            player_list: c.player_list.clone(),
            toggle_perspective: c.toggle_perspective.clone(),
            zoom: c.zoom.clone(),
            fullscreen_toggle: c.fullscreen_toggle.clone(),
            toggle_fps: c.toggle_fps.clone(),
        }
    }
}

impl From<&gui::KeybindsSettings> for KeybindsConfig {
    fn from(s: &gui::KeybindsSettings) -> Self {
        Self {
            forward: s.forward.clone(),
            back: s.back.clone(),
            left: s.left.clone(),
            right: s.right.clone(),
            jump: s.jump.clone(),
            sneak: s.sneak.clone(),
            sprint: s.sprint.clone(),
            drop_item: s.drop_item.clone(),
            inventory: s.inventory.clone(),
            chat: s.chat.clone(),
            player_list: s.player_list.clone(),
            toggle_perspective: s.toggle_perspective.clone(),
            zoom: s.zoom.clone(),
            fullscreen_toggle: s.fullscreen_toggle.clone(),
            toggle_fps: s.toggle_fps.clone(),
            toggle_cinematic: "F8".to_string(),
        }
    }
}

impl From<&ChatConfig> for gui::ChatSettings {
    fn from(c: &ChatConfig) -> Self {
        Self {
            font_size: c.font_size,
        }
    }
}

impl From<&gui::ChatSettings> for ChatConfig {
    fn from(s: &gui::ChatSettings) -> Self {
        Self {
            font_size: s.font_size,
        }
    }
}

/// Sidebar Scoreboard display settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreboardConfig {
    #[serde(default = "default_scoreboard_numbers")]
    pub show_numbers: bool,
}

fn default_scoreboard_numbers() -> bool {
    true
}

impl Default for ScoreboardConfig {
    fn default() -> Self {
        Self {
            show_numbers: default_scoreboard_numbers(),
        }
    }
}

impl From<&ScoreboardConfig> for gui::ScoreboardSettings {
    fn from(c: &ScoreboardConfig) -> Self {
        Self {
            show_numbers: c.show_numbers,
        }
    }
}

impl From<&gui::ScoreboardSettings> for ScoreboardConfig {
    fn from(s: &gui::ScoreboardSettings) -> Self {
        Self {
            show_numbers: s.show_numbers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_roundtrip() {
        let config = ClientConfig::default();
        let toml_str = toml::to_string_pretty(&config).expect("Serialization failed");
        let decoded: ClientConfig = toml::from_str(&toml_str).expect("Deserialization failed");
        assert_eq!(config, decoded);
    }

    #[test]
    fn test_custom_config_values() {
        let toml_str = r#"
            [general]
            language = "en_US"

            [auth]
            nickname = "PvP_Master"

            [video]
            fov = 90.0
            render_distance = 16
            max_fps = 240
            vsync = false
            gui_scale = 1.25
            fullscreen = true
            view_bobbing = false
            gamma = 1.0

            [audio]
            master = 0.8
            music = 0.0
            blocks = 1.0
            players = 1.0
            hostile = 1.0
            ambient = 0.5

            [keybinds]
            forward = "KeyW"
            back = "KeyS"
            left = "KeyA"
            right = "KeyD"
            jump = "Space"
            sneak = "ShiftLeft"
            sprint = "R"
            drop_item = "Q"
            inventory = "E"
            chat = "Enter"
            player_list = "Tab"
            toggle_perspective = "F5"
            zoom = "C"

            [network]
            tcp_nodelay = true
            default_server = "127.0.0.1:25565"
        "#;

        let mut config: ClientConfig = toml::from_str(toml_str).expect("Failed to parse custom config");
        config.keybinds.standardize();
        assert_eq!(config.general.language, "en_US");
        assert_eq!(config.auth.nickname, "PvP_Master");
        assert_eq!(config.video.fov, 90.0);
        assert_eq!(config.video.gui_scale, 1.25);
        assert_eq!(config.keybinds.forward, "W");
        assert_eq!(config.keybinds.sneak, "LSHIFT");
        assert_eq!(config.network.default_server.as_deref(), Some("127.0.0.1:25565"));
    }

    #[test]
    fn test_config_gui_conversions() {
        let config = ClientConfig::default();
        let gui_video: gui::VideoSettings = (&config.video).into();
        let back_video: VideoConfig = (&gui_video).into();
        assert_eq!(config.video, back_video);

        let gui_audio: gui::AudioSettings = (&config.audio).into();
        let back_audio: AudioConfig = (&gui_audio).into();
        assert_eq!(config.audio, back_audio);

        let gui_keybinds: gui::KeybindsSettings = (&config.keybinds).into();
        let back_keybinds: KeybindsConfig = (&gui_keybinds).into();
        assert_eq!(config.keybinds, back_keybinds);

        let gui_chat: gui::ChatSettings = (&config.chat).into();
        let back_chat: ChatConfig = (&gui_chat).into();
        assert_eq!(config.chat, back_chat);

        let gui_scoreboard: gui::ScoreboardSettings = (&config.scoreboard).into();
        let back_scoreboard: ScoreboardConfig = (&gui_scoreboard).into();
        assert_eq!(config.scoreboard, back_scoreboard);
    }
}
