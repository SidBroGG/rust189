pub mod clipboard;
pub mod i18n;
pub mod integration;
pub mod keybinds;
pub mod scale;
pub mod screens;
pub mod servers;
pub mod theme;
pub mod widgets;

pub use clipboard::{get_clipboard_text, set_clipboard_text, text_input_with_paste};
pub use i18n::{t, Language};
pub use integration::GuiIntegration;
pub use keybinds::{
    matches_winit_key, matches_winit_mouse, standard_token_to_egui_key, standardize_key,
    winit_key_to_standard, winit_mouse_to_standard, KeyBindDef, KeyCategory, ALL_KEYBINDS,
};
pub use scale::ScaledResolution;
pub use screens::{
    AudioSettings, GuiOutput, InGameAction, InGameScreen, KeybindsSettings, MainMenuScreen,
    MultiplayerScreen, NametagInfo, ScreenManager, SettingsScreen, VideoSettings,
};
pub use servers::{load_servers_dat, ping_server_async, save_servers_dat, ServerEntry, ServerPingResult};

