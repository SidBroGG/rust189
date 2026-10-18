pub mod in_game;
pub mod main_menu;
pub mod multiplayer;
pub mod server_dialog;
pub mod settings;

pub use in_game::{InGameAction, InGameScreen, NametagInfo};
pub use main_menu::{MainMenuAction, MainMenuScreen};
pub use multiplayer::{MultiplayerAction, MultiplayerScreen};
pub use settings::{AudioSettings, KeybindsSettings, SettingsAction, SettingsScreen, VideoSettings};

use crate::i18n::Language;
use std::path::PathBuf;

pub enum GuiOutput {
    ConnectToServer(String),
    DisconnectFromServer,
    SendChatMessage(String),
    Quit,
    NicknameUpdated(String),
    SettingsUpdated {
        language: Language,
        video: VideoSettings,
        audio: AudioSettings,
        keybinds: KeybindsSettings,
    },
    None,
}

pub struct ScreenManager {
    pub main_menu: MainMenuScreen,
    pub in_game: Option<InGameScreen>,
    last_reported_nickname: String,
}

impl ScreenManager {
    pub fn new(
        nickname: String,
        servers_dat_path: PathBuf,
        language: Language,
        video: VideoSettings,
        audio: AudioSettings,
        keybinds: KeybindsSettings,
    ) -> Self {
        let last_reported_nickname = nickname.clone();
        Self {
            main_menu: MainMenuScreen::new(
                nickname,
                servers_dat_path,
                language,
                video,
                audio,
                keybinds,
            ),
            in_game: None,
            last_reported_nickname,
        }
    }

    pub fn start_in_game(&mut self, server_ip: String, player_name: String) {
        let mut screen = InGameScreen::new(server_ip, player_name);
        screen.language = self.main_menu.language;
        screen.toggle_fps_key = self.main_menu.settings_screen.keybinds.toggle_fps.clone();
        self.in_game = Some(screen);
    }

    pub fn exit_in_game(&mut self) {
        self.in_game = None;
        self.main_menu.modal_mode = main_menu::ModalMode::None;
    }

    /// Syncs fullscreen state with window if the settings modal is not open.
    pub fn sync_fullscreen(&mut self, fullscreen: bool) {
        if self.main_menu.modal_mode != main_menu::ModalMode::Settings {
            self.main_menu.settings_screen.video.fullscreen = fullscreen;
        }
    }

    /// Forcibly updates the fullscreen state in settings screen (e.g. on F11 keybind).
    pub fn force_sync_fullscreen(&mut self, fullscreen: bool) {
        self.main_menu.settings_screen.video.fullscreen = fullscreen;
    }


    pub fn is_cursor_needed(&self) -> bool {
        if self.main_menu.modal_mode != main_menu::ModalMode::None {
            return true;
        }
        if let Some(ig) = &self.in_game {
            return ig.escape_menu_opened || ig.chat_opened || ig.hud_editor_opened;
        }
        true
    }

    pub fn render(&mut self, ctx: &egui::Context) -> GuiOutput {
        if let Some(in_game) = &mut self.in_game {
            // Render settings modal if opened from in-game Escape menu
            if self.main_menu.modal_mode == main_menu::ModalMode::Settings {
                in_game.is_modal_open = true;
                let _ = in_game.render(ctx);

                let escape_pressed = self.main_menu.settings_screen.rebinding_key.is_none()
                    && ctx.input(|i| i.key_pressed(egui::Key::Escape));

                let action = self.main_menu.settings_screen.render_window(ctx);
                if escape_pressed || matches!(action, SettingsAction::Done) {
                    self.main_menu.language = self.main_menu.settings_screen.language;
                    let language = self.main_menu.settings_screen.language;
                    let video = self.main_menu.settings_screen.video.clone();
                    let audio = self.main_menu.settings_screen.audio.clone();
                    let keybinds = self.main_menu.settings_screen.keybinds.clone();
                    in_game.language = language;
                    in_game.toggle_fps_key = keybinds.toggle_fps.clone();
                    in_game.escape_menu_opened = true;
                    in_game.is_modal_open = false;
                    self.main_menu.modal_mode = main_menu::ModalMode::None;
                    return GuiOutput::SettingsUpdated {
                        language,
                        video,
                        audio,
                        keybinds,
                    };
                }

                return GuiOutput::None;
            }

            in_game.is_modal_open = false;
            match in_game.render(ctx) {
                InGameAction::Disconnect => {
                    self.in_game = None;
                    self.main_menu.modal_mode = main_menu::ModalMode::None;
                    return GuiOutput::DisconnectFromServer;
                }
                InGameAction::OpenSettings => {
                    self.main_menu.modal_mode = main_menu::ModalMode::Settings;
                }
                InGameAction::SendChat(msg) => {
                    return GuiOutput::SendChatMessage(msg);
                }
                InGameAction::None => {}
            }

            return GuiOutput::None;
        }

        let mut out_settings = None;
        let action = self.main_menu.render(ctx, &mut out_settings);

        // Check if settings were updated in the settings modal
        if let Some((language, video, audio, keybinds)) = out_settings {
            return GuiOutput::SettingsUpdated {
                language,
                video,
                audio,
                keybinds,
            };
        }

        // Check if nickname changed in top panel
        if self.main_menu.nickname != self.last_reported_nickname {
            self.last_reported_nickname = self.main_menu.nickname.clone();
            return GuiOutput::NicknameUpdated(self.main_menu.nickname.clone());
        }

        match action {
            MainMenuAction::Connect(ip) => GuiOutput::ConnectToServer(ip),
            MainMenuAction::Quit => GuiOutput::Quit,
            MainMenuAction::None => GuiOutput::None,
        }
    }

    pub fn is_rebinding(&self) -> bool {
        self.main_menu.is_rebinding()
    }

    pub fn handle_rebind_token(&mut self, token: &str) {
        self.main_menu.handle_rebind_token(token);
    }

    pub fn cancel_rebinding(&mut self) {
        self.main_menu.cancel_rebinding();
    }
}
