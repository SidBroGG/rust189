use crate::i18n::{t, Language};
use crate::keybinds::{egui_key_to_standard, standardize_key, KeyCategory, ALL_KEYBINDS};
use egui::{Button, Color32, Context, ScrollArea, Slider, Ui, Vec2, Window};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoSettings {
    pub fov: f32,
    pub render_distance: u32,
    pub max_fps: u32,
    pub vsync: bool,
    /// GUI Scale: 0.0 = Auto, or precise custom scale factor (e.g. 1.0, 1.25, 1.5, 2.0).
    #[serde(default = "default_gui_scale")]
    pub gui_scale: f32,
    pub fullscreen: bool,
    pub view_bobbing: bool,
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

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            fov: 70.0,
            render_distance: 12,
            max_fps: 144,
            vsync: false,
            gui_scale: 0.0,
            fullscreen: false,
            view_bobbing: true,
            gamma: 1.0,
            antialiasing: default_antialiasing(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioSettings {
    pub master: f32,
    pub music: f32,
    pub blocks: f32,
    pub players: f32,
    pub hostile: f32,
    pub ambient: f32,
}

impl Default for AudioSettings {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeybindsSettings {
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

impl Default for KeybindsSettings {
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
        }
    }
}

impl KeybindsSettings {
    /// Normalizes all configured keys to canonical uppercase standard tokens.
    pub fn standardize_all(&mut self) {
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
    }

    pub fn get_key_for_id(&self, id: &str) -> &str {
        match id {
            "forward" => &self.forward,
            "back" => &self.back,
            "left" => &self.left,
            "right" => &self.right,
            "jump" => &self.jump,
            "sneak" => &self.sneak,
            "sprint" => &self.sprint,
            "drop_item" => &self.drop_item,
            "inventory" => &self.inventory,
            "chat" => &self.chat,
            "player_list" => &self.player_list,
            "toggle_perspective" => &self.toggle_perspective,
            "zoom" => &self.zoom,
            "fullscreen_toggle" => &self.fullscreen_toggle,
            "toggle_fps" => &self.toggle_fps,
            _ => "",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Video,
    Controls,
    Audio,
    Language,
}

pub enum SettingsAction {
    Done,
    None,
}

pub struct SettingsScreen {
    pub current_tab: SettingsTab,
    pub language: Language,
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub keybinds: KeybindsSettings,
    pub rebinding_key: Option<String>,
}

impl SettingsScreen {
    pub fn new(
        language: Language,
        video: VideoSettings,
        audio: AudioSettings,
        mut keybinds: KeybindsSettings,
    ) -> Self {
        keybinds.standardize_all();
        Self {
            current_tab: SettingsTab::Video,
            language,
            video,
            audio,
            keybinds,
            rebinding_key: None,
        }
    }

    /// Renders settings in a standard egui modal Window.
    pub fn render_window(&mut self, ctx: &Context) -> SettingsAction {
        let mut action = SettingsAction::None;
        let mut is_open = true;

        Window::new(t(self.language, "settings.title"))
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.current_tab, SettingsTab::Video, t(self.language, "settings.tab_video"));
                    ui.selectable_value(&mut self.current_tab, SettingsTab::Controls, t(self.language, "settings.tab_controls"));
                    ui.selectable_value(&mut self.current_tab, SettingsTab::Audio, t(self.language, "settings.tab_audio"));
                    ui.selectable_value(&mut self.current_tab, SettingsTab::Language, t(self.language, "settings.tab_language"));
                });
                ui.separator();

                match self.current_tab {
                    SettingsTab::Video => self.render_video_tab(ui),
                    SettingsTab::Controls => self.render_controls_tab(ui),
                    SettingsTab::Audio => self.render_audio_tab(ui),
                    SettingsTab::Language => self.render_language_tab(ui),
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(Button::new(t(self.language, "settings.done")).min_size(Vec2::new(90.0, 26.0))).clicked() {
                            action = SettingsAction::Done;
                        }
                    });
                });
            });

        if !is_open || (self.rebinding_key.is_none() && ctx.input(|i| i.key_pressed(egui::Key::Escape))) {
            action = SettingsAction::Done;
        }

        action
    }

    fn render_video_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.add_space(6.0);
            ui.add(Slider::new(&mut self.video.fov, 30.0..=110.0).text(t(self.language, "settings.video.fov")));
            ui.add(Slider::new(&mut self.video.render_distance, 2..=32).text(t(self.language, "settings.video.render_distance")));

            let fps_text = if self.video.max_fps == 0 || self.video.max_fps >= 260 {
                t(self.language, "settings.video.unlimited").to_string()
            } else {
                format!("{} FPS", self.video.max_fps)
            };
            ui.horizontal(|ui| {
                let mut fps_val = if self.video.max_fps == 0 { 260 } else { self.video.max_fps };
                if ui.add(Slider::new(&mut fps_val, 10..=260).text(t(self.language, "settings.video.max_fps"))).changed() {
                    self.video.max_fps = if fps_val >= 260 { 0 } else { fps_val };
                }
                ui.label(fps_text);
            });

            ui.add_space(4.0);
            ui.checkbox(&mut self.video.vsync, t(self.language, "settings.video.vsync"));
            ui.checkbox(&mut self.video.fullscreen, t(self.language, "settings.video.fullscreen"));
            ui.checkbox(&mut self.video.view_bobbing, t(self.language, "settings.video.view_bobbing"));
            ui.add(Slider::new(&mut self.video.gamma, 0.0..=1.0).text(t(self.language, "settings.video.gamma")));

            ui.add_space(4.0);
            // Fine-grained GUI scale setting
            ui.horizontal(|ui| {
                ui.label(t(self.language, "settings.video.gui_scale"));
                let mut is_auto = self.video.gui_scale <= 0.05;
                if ui.checkbox(&mut is_auto, t(self.language, "settings.video.gui_scale_auto")).changed() {
                    if is_auto {
                        self.video.gui_scale = 0.0;
                    } else {
                        self.video.gui_scale = 1.0;
                    }
                }
            });

            if self.video.gui_scale > 0.05 {
                ui.horizontal(|ui| {
                    ui.label(t(self.language, "settings.video.gui_scale_custom"));
                    let scale_val = self.video.gui_scale;
                    let percent = (scale_val * 100.0).round() as u32;
                    let text = format!("{}% ({:.2}x)", percent, scale_val);
                    ui.add(
                        Slider::new(&mut self.video.gui_scale, 0.5..=3.0)
                            .step_by(0.05)
                            .text(text),
                    );
                });
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            // Anti-Aliasing (Сглаживание)
            ui.horizontal(|ui| {
                ui.label(t(self.language, "settings.video.antialiasing"));
                let aa_text = match self.video.antialiasing {
                    0 => t(self.language, "settings.video.antialiasing_off"),
                    1 => t(self.language, "settings.video.antialiasing_fxaa"),
                    _ => t(self.language, "settings.video.antialiasing_high"),
                };
                if ui.button(aa_text).clicked() {
                    self.video.antialiasing = (self.video.antialiasing + 1) % 3;
                }
            });

            ui.add_space(6.0);
        });
    }

    fn render_controls_tab(&mut self, ui: &mut Ui) {
        // Handle keypress during rebinding
        let mut pressed_action = None;
        if let Some(ref target) = self.rebinding_key {
            let target_clone = target.clone();
            ui.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key { key, pressed: true, .. } = event {
                        if *key == egui::Key::Escape {
                            // Escape cancels rebinding
                            pressed_action = Some((target_clone.clone(), None));
                        } else if let Some(std_name) = egui_key_to_standard(*key) {
                            pressed_action = Some((target_clone.clone(), Some(std_name)));
                        }
                        break;
                    }
                }
            });
        }
        if let Some((target, new_key_opt)) = pressed_action {
            if let Some(new_key) = new_key_opt {
                self.apply_keybind(&target, new_key);
            }
            self.rebinding_key = None;
        }

        // Count key usage for conflict detection
        let mut key_usage_count = HashMap::new();
        for def in ALL_KEYBINDS {
            let k = self.keybinds.get_key_for_id(def.id);
            if !k.is_empty() {
                *key_usage_count.entry(k.to_string()).or_insert(0usize) += 1;
            }
        }

        ui.vertical(|ui| {
            ui.add_space(4.0);
            ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                let categories = [
                    KeyCategory::Movement,
                    KeyCategory::Gameplay,
                    KeyCategory::Multiplayer,
                    KeyCategory::Display,
                ];

                for cat in categories {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(t(self.language, cat.title_key())).strong());

                    for def in ALL_KEYBINDS.iter().filter(|d| d.category == cat) {
                        let current_key = self.keybinds.get_key_for_id(def.id);
                        let is_conflicting = key_usage_count.get(current_key).copied().unwrap_or(0) > 1;

                        ui.horizontal(|ui| {
                            ui.label(t(self.language, def.name_key));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let is_rebinding = self.rebinding_key.as_deref() == Some(def.id);
                                let btn_text = if is_rebinding {
                                    egui::RichText::new(t(self.language, "settings.controls.press_key"))
                                        .color(Color32::from_rgb(255, 215, 0))
                                } else if is_conflicting {
                                    egui::RichText::new(current_key)
                                        .color(Color32::from_rgb(255, 85, 85))
                                } else {
                                    egui::RichText::new(current_key)
                                };

                                let btn = Button::new(btn_text).min_size(Vec2::new(110.0, 22.0));
                                if ui.add(btn).clicked() {
                                    self.rebinding_key = Some(def.id.to_string());
                                }
                            });
                        });
                    }
                }
            });

            ui.add_space(6.0);
            if ui.button(t(self.language, "settings.controls.reset_defaults")).clicked() {
                self.keybinds = KeybindsSettings::default();
            }
        });
    }

    fn render_audio_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.add_space(6.0);
            ui.add(Slider::new(&mut self.audio.master, 0.0..=1.0).text(t(self.language, "settings.audio.master")));
            ui.add(Slider::new(&mut self.audio.music, 0.0..=1.0).text(t(self.language, "settings.audio.music")));
            ui.add(Slider::new(&mut self.audio.blocks, 0.0..=1.0).text(t(self.language, "settings.audio.blocks")));
            ui.add(Slider::new(&mut self.audio.players, 0.0..=1.0).text(t(self.language, "settings.audio.players")));
            ui.add(Slider::new(&mut self.audio.hostile, 0.0..=1.0).text(t(self.language, "settings.audio.hostile")));
            ui.add(Slider::new(&mut self.audio.ambient, 0.0..=1.0).text(t(self.language, "settings.audio.ambient")));
            ui.add_space(6.0);
        });
    }

    fn render_language_tab(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(t(self.language, "settings.lang.select_prompt")).strong());
            ui.add_space(6.0);

            ui.radio_value(&mut self.language, Language::Russian, Language::Russian.display_name());
            ui.add_space(4.0);
            ui.radio_value(&mut self.language, Language::English, Language::English.display_name());

            ui.add_space(8.0);
            ui.label(format!("{} {}", t(self.language, "settings.lang.current"), self.language.display_name()));
            ui.add_space(6.0);
        });
    }

    pub fn is_rebinding(&self) -> bool {
        self.rebinding_key.is_some()
    }

    pub fn handle_rebind_token(&mut self, token: &str) {
        if token.eq_ignore_ascii_case("ESCAPE") {
            self.rebinding_key = None;
        } else if let Some(target) = self.rebinding_key.take() {
            self.apply_keybind(&target, token.to_string());
        }
    }

    pub fn cancel_rebinding(&mut self) {
        self.rebinding_key = None;
    }

    pub fn apply_keybind(&mut self, id: &str, new_key: String) {
        let std_key = standardize_key(&new_key);
        match id {
            "forward" => self.keybinds.forward = std_key,
            "back" => self.keybinds.back = std_key,
            "left" => self.keybinds.left = std_key,
            "right" => self.keybinds.right = std_key,
            "jump" => self.keybinds.jump = std_key,
            "sneak" => self.keybinds.sneak = std_key,
            "sprint" => self.keybinds.sprint = std_key,
            "drop_item" => self.keybinds.drop_item = std_key,
            "inventory" => self.keybinds.inventory = std_key,
            "chat" => self.keybinds.chat = std_key,
            "player_list" => self.keybinds.player_list = std_key,
            "toggle_perspective" => self.keybinds.toggle_perspective = std_key,
            "zoom" => self.keybinds.zoom = std_key,
            "fullscreen_toggle" => self.keybinds.fullscreen_toggle = std_key,
            "toggle_fps" => self.keybinds.toggle_fps = std_key,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_serialization_roundtrip() {
        let video = VideoSettings::default();
        let json = serde_json::to_string(&video).expect("Serialization failed");
        let decoded: VideoSettings = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(video, decoded);

        let audio = AudioSettings::default();
        let json_a = serde_json::to_string(&audio).expect("Serialization failed");
        let decoded_a: AudioSettings = serde_json::from_str(&json_a).expect("Deserialization failed");
        assert_eq!(audio, decoded_a);

        let keybinds = KeybindsSettings::default();
        let json_k = serde_json::to_string(&keybinds).expect("Serialization failed");
        let decoded_k: KeybindsSettings = serde_json::from_str(&json_k).expect("Deserialization failed");
        assert_eq!(keybinds, decoded_k);
    }

    #[test]
    fn test_apply_keybind() {
        let mut screen = SettingsScreen::new(
            Language::Russian,
            VideoSettings::default(),
            AudioSettings::default(),
            KeybindsSettings::default(),
        );

        assert_eq!(screen.keybinds.forward, "W");
        screen.apply_keybind("forward", "KeyZ".to_string());
        assert_eq!(screen.keybinds.forward, "Z");

        screen.apply_keybind("fullscreen_toggle", "F12".to_string());
        assert_eq!(screen.keybinds.fullscreen_toggle, "F12");
    }
}
