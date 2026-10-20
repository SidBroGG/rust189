use crate::clipboard::text_input_with_paste;
use crate::i18n::{t, Language};
use crate::screens::server_dialog::{
    render_server_dialog, ServerDialogResult, ServerDialogState,
};
use crate::screens::settings::{AudioSettings, ChatSettings, KeybindsSettings, ScoreboardSettings, SettingsAction, SettingsScreen, VideoSettings};
use crate::servers::{load_servers_dat, ping_server_async, save_servers_dat, ServerEntry, ServerPingResult};
use crate::theme::parse_minecraft_text;
use egui::{
    Button, Color32, Context, CornerRadius, Frame, Margin, RichText, ScrollArea, Sense,
    Stroke, TopBottomPanel, Vec2,
};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalMode {
    None,
    Add,
    Edit(usize),
    DirectConnect,
    Settings,
}

pub enum MainMenuAction {
    Connect(String),
    Quit,
    None,
}

pub struct MainMenuScreen {
    pub nickname: String,
    pub language: Language,
    pub servers: Vec<ServerEntry>,
    pub selected_index: Option<usize>,
    pub servers_dat_path: PathBuf,
    pub modal_mode: ModalMode,
    pub dialog_state: ServerDialogState,
    pub settings_screen: SettingsScreen,
    ping_tx: Sender<(usize, Result<ServerPingResult, String>)>,
    ping_rx: Receiver<(usize, Result<ServerPingResult, String>)>,
}

impl MainMenuScreen {
    pub fn new(
        nickname: String,
        servers_dat_path: PathBuf,
        language: Language,
        video: VideoSettings,
        audio: AudioSettings,
        keybinds: KeybindsSettings,
        chat: ChatSettings,
        scoreboard: ScoreboardSettings,
    ) -> Self {
        let servers = load_servers_dat(&servers_dat_path);
        let (ping_tx, ping_rx) = channel();

        let mut screen = Self {
            nickname,
            language,
            servers,
            selected_index: None,
            servers_dat_path,
            modal_mode: ModalMode::None,
            dialog_state: ServerDialogState::default(),
            settings_screen: SettingsScreen::new(language, video, audio, keybinds, chat, scoreboard),
            ping_tx,
            ping_rx,
        };

        screen.refresh_all();
        screen
    }

    /// Triggers background ping for all servers in list.
    pub fn refresh_all(&mut self) {
        let ping_msg = t(self.language, "menu.pinging").to_string();
        for (idx, server) in self.servers.iter_mut().enumerate() {
            server.is_pinging = true;
            server.error = None;
            server.motd = ping_msg.clone();

            let target = server.ip.clone();
            let tx = self.ping_tx.clone();

            tokio::spawn(async move {
                let res = ping_server_async(&target).await;
                let _ = tx.send((idx, res));
            });
        }
    }

    /// Checks for any completed async server pings.
    fn poll_pings(&mut self) {
        while let Ok((idx, result)) = self.ping_rx.try_recv() {
            if let Some(server) = self.servers.get_mut(idx) {
                server.is_pinging = false;
                match result {
                    Ok(ping) => {
                        server.motd = ping.motd;
                        server.players_online = ping.players_online;
                        server.players_max = ping.players_max;
                        server.ping_ms = Some(ping.ping_ms);
                        server.version_name = ping.version_name;
                        server.protocol_version = ping.protocol_version;
                        server.icon = ping.icon;
                        server.error = None;
                    }
                    Err(err) => {
                        server.error = Some(err);
                        server.motd = t(self.language, "menu.connection_failed").to_string();
                        server.ping_ms = None;
                    }
                }
            }
        }
    }

    /// Renders the complete main menu with top bar, server list, and bottom actions.
    pub fn render(
        &mut self,
        ctx: &Context,
        out_settings: &mut Option<(
            Language,
            VideoSettings,
            AudioSettings,
            KeybindsSettings,
            ChatSettings,
            ScoreboardSettings,
        )>,
    ) -> MainMenuAction {
        self.poll_pings();
        let mut action = MainMenuAction::None;

        // 1. TOP PANEL: Nickname input and Settings button
        TopBottomPanel::top("main_top_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                ui.heading("⚡ Rust189");
                ui.label(
                    RichText::new("v1.8.9 (Protocol 47)")
                        .color(Color32::from_rgb(140, 140, 140))
                        .size(12.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(Button::new(t(self.language, "top.settings")).min_size(Vec2::new(100.0, 24.0))).clicked() {
                        self.settings_screen.language = self.language;
                        self.modal_mode = ModalMode::Settings;
                    }

                    ui.add_space(8.0);
                    text_input_with_paste(ui, &mut self.nickname, t(self.language, "top.nickname"), Some(130.0));
                    ui.label(t(self.language, "top.nickname"));
                });
            });
            ui.add_space(3.0);
        });

        // 2. BOTTOM PANEL: Server management actions
        TopBottomPanel::bottom("main_bottom_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // Left group
                if ui.button(t(self.language, "menu.add_server")).clicked() {
                    self.modal_mode = ModalMode::Add;
                    self.dialog_state = ServerDialogState {
                        name: "Server".to_string(),
                        ip: String::new(),
                        edit_index: None,
                    };
                }

                if ui.button(t(self.language, "menu.direct_connect")).clicked() {
                    self.modal_mode = ModalMode::DirectConnect;
                    self.dialog_state = ServerDialogState {
                        name: String::new(),
                        ip: "localhost".to_string(),
                        edit_index: None,
                    };
                }

                if ui.button(t(self.language, "menu.refresh")).clicked() {
                    self.refresh_all();
                }

                // Right group
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t(self.language, "menu.quit")).clicked() {
                        action = MainMenuAction::Quit;
                    }

                    let has_selection = self.selected_index.is_some();

                    if ui
                        .add_enabled(has_selection, Button::new(t(self.language, "menu.delete_server")))
                        .clicked()
                    {
                        if let Some(idx) = self.selected_index {
                            self.servers.remove(idx);
                            self.selected_index = None;
                            let _ = save_servers_dat(&self.servers, &self.servers_dat_path);
                        }
                    }

                    if ui
                        .add_enabled(has_selection, Button::new(t(self.language, "menu.edit_server")))
                        .clicked()
                    {
                        if let Some(idx) = self.selected_index {
                            if let Some(srv) = self.servers.get(idx) {
                                self.modal_mode = ModalMode::Edit(idx);
                                self.dialog_state = ServerDialogState {
                                    name: srv.name.clone(),
                                    ip: srv.ip.clone(),
                                    edit_index: Some(idx),
                                };
                            }
                        }
                    }

                    if ui
                        .add_enabled(
                            has_selection,
                            Button::new(RichText::new(format!("▶ {}", t(self.language, "menu.join_server"))).strong()),
                        )
                        .clicked()
                    {
                        if let Some(idx) = self.selected_index {
                            if let Some(srv) = self.servers.get(idx) {
                                action = MainMenuAction::Connect(srv.ip.clone());
                            }
                        }
                    }
                });
            });
            ui.add_space(4.0);
        });

        // 3. CENTRAL PANEL: Server List
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.servers.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(t(self.language, "menu.empty_servers"));
                });
                return;
            }

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut double_clicked_idx = None;

                    for (idx, server) in self.servers.iter().enumerate() {
                        let is_selected = self.selected_index == Some(idx);

                        let fill = if is_selected {
                            Color32::from_rgb(40, 48, 64)
                        } else {
                            Color32::from_rgb(28, 30, 34)
                        };

                        let stroke = if is_selected {
                            Stroke::new(1.5_f32, Color32::from_rgb(88, 140, 255))
                        } else {
                            Stroke::new(1.0_f32, Color32::from_rgb(44, 46, 52))
                        };

                        let card = Frame::NONE
                            .fill(fill)
                            .stroke(stroke)
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::symmetric(10, 8));

                        let card_response = card.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Server icon avatar
                                let (avatar_rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
                                ui.painter().rect_filled(
                                    avatar_rect,
                                    CornerRadius::same(4),
                                    Color32::from_rgb(48, 52, 60),
                                );
                                let initial = server.name.chars().next().unwrap_or('S').to_string();
                                ui.painter().text(
                                    avatar_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    initial,
                                    egui::FontId::proportional(18.0),
                                    Color32::from_rgb(255, 215, 0),
                                );

                                ui.add_space(8.0);

                                // Title and MOTD
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&server.name).strong().size(14.0));
                                        ui.label(
                                            RichText::new(&server.ip)
                                                .color(Color32::from_rgb(130, 130, 130))
                                                .size(12.0),
                                        );
                                    });

                                    ui.add_space(2.0);

                                    // MOTD
                                    if server.is_pinging {
                                        ui.label(
                                            RichText::new(t(self.language, "menu.pinging"))
                                                .color(Color32::from_rgb(120, 120, 120))
                                                .size(12.0),
                                        );
                                    } else if let Some(ref err) = server.error {
                                        ui.label(
                                            RichText::new(err)
                                                .color(Color32::from_rgb(255, 90, 90))
                                                .size(12.0),
                                        );
                                    } else {
                                        let segments = parse_minecraft_text(&server.motd);
                                        ui.horizontal(|ui| {
                                            for (col, text) in segments.iter().take(4) {
                                                ui.label(RichText::new(text).color(*col).size(12.0));
                                            }
                                        });
                                    }
                                });

                                // Right stats: Players count & Latency
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if !server.is_pinging && server.error.is_none() {
                                        if let Some(ping) = server.ping_ms {
                                            let ping_color = if ping < 60 {
                                                Color32::from_rgb(80, 220, 100)
                                            } else if ping < 140 {
                                                Color32::from_rgb(240, 200, 60)
                                            } else {
                                                Color32::from_rgb(240, 80, 80)
                                            };
                                            ui.label(
                                                RichText::new(format!("{} {}", ping, t(self.language, "menu.ping_ms")))
                                                    .color(ping_color)
                                                    .size(12.0),
                                            );
                                        }

                                        let count = format!("{}/{}", server.players_online, server.players_max);
                                        ui.label(
                                            RichText::new(count)
                                                .color(Color32::from_rgb(160, 160, 160))
                                                .size(12.0),
                                        );
                                    }
                                });
                            });
                        });

                        let interact = ui.interact(card_response.response.rect, card_response.response.id, Sense::click());
                        if interact.clicked() {
                            self.selected_index = Some(idx);
                        }
                        if interact.double_clicked() {
                            double_clicked_idx = Some(idx);
                        }

                        ui.add_space(4.0);
                    }

                    if let Some(idx) = double_clicked_idx {
                        if let Some(srv) = self.servers.get(idx) {
                            action = MainMenuAction::Connect(srv.ip.clone());
                        }
                    }
                });
        });

        // 4. Modals rendering
        match self.modal_mode {
            ModalMode::Add => {
                match render_server_dialog(ctx, &mut self.dialog_state, false, self.language) {
                    ServerDialogResult::Save { name, ip, .. } => {
                        let new_entry = ServerEntry::new(name, ip);
                        self.servers.push(new_entry);
                        let _ = save_servers_dat(&self.servers, &self.servers_dat_path);
                        self.refresh_all();
                        self.modal_mode = ModalMode::None;
                    }
                    ServerDialogResult::Cancel => {
                        self.modal_mode = ModalMode::None;
                    }
                    ServerDialogResult::None => {}
                }
            }
            ModalMode::Edit(idx) => {
                match render_server_dialog(ctx, &mut self.dialog_state, false, self.language) {
                    ServerDialogResult::Save { name, ip, .. } => {
                        if let Some(entry) = self.servers.get_mut(idx) {
                            entry.name = name;
                            entry.ip = ip;
                            let _ = save_servers_dat(&self.servers, &self.servers_dat_path);
                            self.refresh_all();
                        }
                        self.modal_mode = ModalMode::None;
                    }
                    ServerDialogResult::Cancel => {
                        self.modal_mode = ModalMode::None;
                    }
                    ServerDialogResult::None => {}
                }
            }
            ModalMode::DirectConnect => {
                match render_server_dialog(ctx, &mut self.dialog_state, true, self.language) {
                    ServerDialogResult::Save { ip, .. } => {
                        self.modal_mode = ModalMode::None;
                        action = MainMenuAction::Connect(ip);
                    }
                    ServerDialogResult::Cancel => {
                        self.modal_mode = ModalMode::None;
                    }
                    ServerDialogResult::None => {}
                }
            }
            ModalMode::Settings => {
                if let SettingsAction::Done = self.settings_screen.render_window(ctx) {
                    self.language = self.settings_screen.language;
                    *out_settings = Some((
                        self.settings_screen.language,
                        self.settings_screen.video.clone(),
                        self.settings_screen.audio.clone(),
                        self.settings_screen.keybinds.clone(),
                        self.settings_screen.chat.clone(),
                        self.settings_screen.scoreboard.clone(),
                    ));
                    self.modal_mode = ModalMode::None;
                }
            }
            ModalMode::None => {}
        }

        action
    }

    pub fn is_rebinding(&self) -> bool {
        self.modal_mode == ModalMode::Settings && self.settings_screen.is_rebinding()
    }

    pub fn handle_rebind_token(&mut self, token: &str) {
        if self.modal_mode == ModalMode::Settings {
            self.settings_screen.handle_rebind_token(token);
        }
    }

    pub fn cancel_rebinding(&mut self) {
        if self.modal_mode == ModalMode::Settings {
            self.settings_screen.cancel_rebinding();
        }
    }
}
