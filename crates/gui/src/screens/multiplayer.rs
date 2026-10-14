use crate::i18n::Language;
use crate::screens::server_dialog::{
    render_server_dialog, ServerDialogResult, ServerDialogState,
};
use crate::servers::{load_servers_dat, ping_server_async, save_servers_dat, ServerEntry, ServerPingResult};
use crate::theme::parse_minecraft_text;
use crate::widgets::{draw_dirt_background, MinecraftButton};
use egui::{Align2, Color32, Pos2, Rect, ScrollArea, Sense, Ui, Vec2};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum MultiplayerAction {
    Connect(String),
    BackToMainMenu,
    None,
}

enum ModalMode {
    None,
    Add,
    Edit(usize),
    DirectConnect,
}

pub struct MultiplayerScreen {
    servers: Vec<ServerEntry>,
    selected_index: Option<usize>,
    servers_dat_path: PathBuf,
    modal_mode: ModalMode,
    dialog_state: ServerDialogState,
    ping_tx: Sender<(usize, Result<ServerPingResult, String>)>,
    ping_rx: Receiver<(usize, Result<ServerPingResult, String>)>,
}

impl MultiplayerScreen {
    pub fn new(servers_dat_path: PathBuf) -> Self {
        let servers = load_servers_dat(&servers_dat_path);
        let (ping_tx, ping_rx) = channel();

        let mut screen = Self {
            servers,
            selected_index: None,
            servers_dat_path,
            modal_mode: ModalMode::None,
            dialog_state: ServerDialogState::default(),
            ping_tx,
            ping_rx,
        };

        screen.refresh_all();
        screen
    }

    /// Triggers asynchronous pinging for all servers in the list.
    pub fn refresh_all(&mut self) {
        for (idx, server) in self.servers.iter_mut().enumerate() {
            server.is_pinging = true;
            server.error = None;
            server.motd = "Pinging...".to_string();

            let target = server.ip.clone();
            let tx = self.ping_tx.clone();

            tokio::spawn(async move {
                let res = ping_server_async(&target).await;
                let _ = tx.send((idx, res));
            });
        }
    }

    /// Processes any completed background ping results without blocking.
    fn poll_ping_results(&mut self) {
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
                        server.motd = "Can't connect to server".to_string();
                        server.ping_ms = None;
                    }
                }
            }
        }
    }

    pub fn render(&mut self, ui: &mut Ui) -> MultiplayerAction {
        self.poll_ping_results();
        let mut action = MultiplayerAction::None;

        draw_dirt_background(ui, Some("Play Multiplayer"));

        let screen_rect = ui.max_rect();
        let header_height = 42.0;
        let footer_height = 54.0;
        let list_rect = Rect::from_min_max(
            Pos2::new(screen_rect.min.x, screen_rect.min.y + header_height),
            Pos2::new(screen_rect.max.x, screen_rect.max.y - footer_height),
        );

        // Server list area
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(list_rect), |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    let mut double_clicked_idx = None;

                    for (idx, server) in self.servers.iter().enumerate() {
                        let is_selected = self.selected_index == Some(idx);
                        let row_width = 380.0_f32.min(ui.available_width() - 20.0);
                        let row_height = 36.0;

                        ui.vertical_centered(|ui| {
                            let (rect, response) = ui.allocate_exact_size(
                                Vec2::new(row_width, row_height),
                                Sense::click(),
                            );

                            if response.clicked() {
                                self.selected_index = Some(idx);
                            }
                            if response.double_clicked() {
                                double_clicked_idx = Some(idx);
                            }

                            if ui.is_rect_visible(rect) {
                                let painter = ui.painter();

                                // Selection / hover highlight border
                                if is_selected {
                                    painter.rect_filled(
                                        rect,
                                        0.0,
                                        Color32::from_rgba_premultiplied(0, 0, 0, 140),
                                    );
                                    painter.rect_stroke(
                                        rect,
                                        0.0,
                                        egui::Stroke::new(1.0_f32, Color32::from_rgb(255, 255, 255)),
                                        egui::StrokeKind::Inside,
                                    );
                                } else if response.hovered() {
                                    painter.rect_filled(
                                        rect,
                                        0.0,
                                        Color32::from_rgba_premultiplied(255, 255, 255, 20),
                                    );
                                }

                                // Server Icon placeholder (32x32)
                                let icon_rect = Rect::from_min_size(
                                    Pos2::new(rect.min.x + 4.0, rect.min.y + 2.0),
                                    Vec2::splat(32.0),
                                );
                                painter.rect_filled(icon_rect, 0.0, Color32::from_rgb(45, 45, 45));
                                painter.rect_stroke(
                                    icon_rect,
                                    0.0,
                                    egui::Stroke::new(1.0_f32, Color32::from_rgb(80, 80, 80)),
                                    egui::StrokeKind::Inside,
                                );
                                let initial = server.name.chars().next().unwrap_or('S').to_string();
                                painter.text(
                                    icon_rect.center(),
                                    Align2::CENTER_CENTER,
                                    initial,
                                    egui::FontId::new(16.0, egui::FontFamily::Proportional),
                                    Color32::from_rgb(255, 215, 0),
                                );

                                // Server Title & MOTD text
                                let text_x = rect.min.x + 42.0;

                                // Title
                                painter.text(
                                    Pos2::new(text_x, rect.min.y + 4.0),
                                    Align2::LEFT_TOP,
                                    &server.name,
                                    egui::FontId::new(14.0, egui::FontFamily::Proportional),
                                    Color32::from_rgb(255, 255, 255),
                                );

                                // MOTD or Status
                                if server.is_pinging {
                                    painter.text(
                                        Pos2::new(text_x, rect.min.y + 19.0),
                                        Align2::LEFT_TOP,
                                        "Pinging...",
                                        egui::FontId::new(12.0, egui::FontFamily::Proportional),
                                        Color32::from_rgb(128, 128, 128),
                                    );
                                } else if let Some(ref err) = server.error {
                                    painter.text(
                                        Pos2::new(text_x, rect.min.y + 19.0),
                                        Align2::LEFT_TOP,
                                        err,
                                        egui::FontId::new(12.0, egui::FontFamily::Proportional),
                                        Color32::from_rgb(255, 85, 85),
                                    );
                                } else {
                                    // Parse color codes in MOTD
                                    let segments = parse_minecraft_text(&server.motd);
                                    let mut curr_x = text_x;
                                    for (col, text) in segments.iter().take(4) {
                                        let text_galley = painter.layout_no_wrap(
                                            text.clone(),
                                            egui::FontId::new(12.0, egui::FontFamily::Proportional),
                                            *col,
                                        );
                                        painter.galley(
                                            Pos2::new(curr_x, rect.min.y + 19.0),
                                            text_galley.clone(),
                                            *col,
                                        );
                                        curr_x += text_galley.size().x;
                                        if curr_x > rect.max.x - 70.0 {
                                            break;
                                        }
                                    }
                                }

                                // Player count and Ping on the right
                                if !server.is_pinging && server.error.is_none() {
                                    let count_str = format!("{}/{}", server.players_online, server.players_max);
                                    painter.text(
                                        Pos2::new(rect.max.x - 4.0, rect.min.y + 4.0),
                                        Align2::RIGHT_TOP,
                                        count_str,
                                        egui::FontId::new(12.0, egui::FontFamily::Proportional),
                                        Color32::from_rgb(170, 170, 170),
                                    );

                                    if let Some(ping) = server.ping_ms {
                                        let ping_str = format!("{} ms", ping);
                                        let ping_col = if ping < 60 {
                                            Color32::from_rgb(85, 255, 85)
                                        } else if ping < 150 {
                                            Color32::from_rgb(255, 255, 85)
                                        } else {
                                            Color32::from_rgb(255, 85, 85)
                                        };
                                        painter.text(
                                            Pos2::new(rect.max.x - 4.0, rect.min.y + 19.0),
                                            Align2::RIGHT_TOP,
                                            ping_str,
                                            egui::FontId::new(11.0, egui::FontFamily::Proportional),
                                            ping_col,
                                        );
                                    }
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }

                    if let Some(idx) = double_clicked_idx {
                        if let Some(srv) = self.servers.get(idx) {
                            action = MultiplayerAction::Connect(srv.ip.clone());
                        }
                    }
                });
        });

        // Footer buttons (two rows of buttons, matching vanilla Minecraft layout)
        let footer_area = Rect::from_min_max(
            Pos2::new(screen_rect.min.x, screen_rect.max.y - footer_height),
            screen_rect.max,
        );
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(footer_area), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);
                // Row 1: Join Server | Direct Connect | Add Server
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 380.0) / 2.0);
                    let can_join = self.selected_index.is_some();

                    if ui
                        .add(MinecraftButton::new("Join Server").width(120.0).enabled(can_join))
                        .clicked()
                    {
                        if let Some(idx) = self.selected_index {
                            if let Some(srv) = self.servers.get(idx) {
                                action = MultiplayerAction::Connect(srv.ip.clone());
                            }
                        }
                    }
                    ui.add_space(8.0);
                    if ui.add(MinecraftButton::new("Direct Connect").width(120.0)).clicked() {
                        self.modal_mode = ModalMode::DirectConnect;
                        self.dialog_state = ServerDialogState {
                            name: String::new(),
                            ip: "localhost".to_string(),
                            edit_index: None,
                        };
                    }
                    ui.add_space(8.0);
                    if ui.add(MinecraftButton::new("Add Server").width(120.0)).clicked() {
                        self.modal_mode = ModalMode::Add;
                        self.dialog_state = ServerDialogState {
                            name: "Server".to_string(),
                            ip: String::new(),
                            edit_index: None,
                        };
                    }
                });

                ui.add_space(4.0);
                // Row 2: Edit | Delete | Refresh | Cancel
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 380.0) / 2.0);
                    let has_selection = self.selected_index.is_some();

                    if ui.add(MinecraftButton::new("Edit").width(90.0).enabled(has_selection)).clicked() {
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
                    ui.add_space(6.0);
                    if ui.add(MinecraftButton::new("Delete").width(90.0).enabled(has_selection)).clicked() {
                        if let Some(idx) = self.selected_index {
                            self.servers.remove(idx);
                            self.selected_index = None;
                            let _ = save_servers_dat(&self.servers, &self.servers_dat_path);
                        }
                    }
                    ui.add_space(6.0);
                    if ui.add(MinecraftButton::new("Refresh").width(90.0)).clicked() {
                        self.refresh_all();
                    }
                    ui.add_space(6.0);
                    if ui.add(MinecraftButton::new("Cancel").width(90.0)).clicked() {
                        action = MultiplayerAction::BackToMainMenu;
                    }
                });
            });
        });

        // Render modal if open
        match self.modal_mode {
            ModalMode::Add => {
                match render_server_dialog(ui.ctx(), &mut self.dialog_state, false, Language::Russian) {
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
                match render_server_dialog(ui.ctx(), &mut self.dialog_state, false, Language::Russian) {
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
                match render_server_dialog(ui.ctx(), &mut self.dialog_state, true, Language::Russian) {
                    ServerDialogResult::Save { ip, .. } => {
                        self.modal_mode = ModalMode::None;
                        action = MultiplayerAction::Connect(ip);
                    }
                    ServerDialogResult::Cancel => {
                        self.modal_mode = ModalMode::None;
                    }
                    ServerDialogResult::None => {}
                }
            }
            ModalMode::None => {}
        }

        action
    }
}
