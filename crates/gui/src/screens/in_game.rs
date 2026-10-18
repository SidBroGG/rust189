use crate::hud::{
    get_module_content_size, render_hud_editor, render_module_by_id, HudData, HudEditorState,
    HudLayoutConfig,
};
use crate::i18n::{t, Language};
use crate::keybinds::standard_token_to_egui_key;
use crate::theme::parse_minecraft_text;
use egui::{
    Align, Align2, Color32, CornerRadius, Frame, Id, Key, LayerId, Layout, Order, Pos2, Rect,
    RichText, ScrollArea, Stroke, TextEdit, Vec2, Window,
};
use std::path::PathBuf;
use std::time::Instant;
use world::{BenchmarkSnapshot, Scoreboard, TabList};

#[derive(Debug, Clone)]
pub struct NametagInfo {
    pub screen_pos: (f32, f32),
    pub text: String,
    pub distance: f32,
    pub is_sneaking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InGameAction {
    None,
    Disconnect,
    OpenSettings,
    SendChat(String),
}

/// A timestamped chat line for smooth alpha fading.
#[derive(Debug, Clone)]
pub struct ChatMessageEntry {
    pub received_at: Instant,
    pub text: String,
}

/// Represents the in-game HUD overlay screen during the Play state.
pub struct InGameScreen {
    pub server_ip: String,
    pub player_name: String,
    pub pos: (f64, f64, f64),
    pub look: (f32, f32),
    pub dimension: i8,
    pub chat_messages: Vec<ChatMessageEntry>,
    pub chat_input: String,
    pub chat_opened: bool,
    pub escape_menu_opened: bool,
    pub hud_editor_opened: bool,
    pub show_fps: bool,
    pub toggle_fps_key: String,
    pub language: Language,
    pub show_tab_list: bool,
    pub tab_list: TabList,
    pub scoreboard: Scoreboard,
    pub benchmark: Option<BenchmarkSnapshot>,
    pub is_modal_open: bool,
    pub nametags: Vec<NametagInfo>,

    // Modular HUD Layout and Editor
    pub hud_layout: HudLayoutConfig,
    pub hud_editor: HudEditorState,
    pub hud_data: HudData,
    pub layout_path: PathBuf,
    pub tick_counter: u64,

    request_chat_focus: bool,
}

impl InGameScreen {
    pub fn new(server_ip: String, player_name: String) -> Self {
        let layout_path = PathBuf::from("hud_layout.toml");
        let hud_layout = HudLayoutConfig::load_or_default(&layout_path);

        Self {
            server_ip,
            player_name,
            pos: (0.0, 64.0, 0.0),
            look: (0.0, 0.0),
            dimension: 0,
            chat_messages: Vec::new(),
            chat_input: String::new(),
            chat_opened: false,
            escape_menu_opened: false,
            hud_editor_opened: false,
            show_fps: false,
            toggle_fps_key: "F3".to_string(),
            language: Language::Russian,
            show_tab_list: false,
            tab_list: TabList::new(),
            scoreboard: Scoreboard::new(),
            benchmark: None,
            is_modal_open: false,
            nametags: Vec::new(),
            hud_layout,
            hud_editor: HudEditorState::default(),
            hud_data: HudData::default(),
            layout_path,
            tick_counter: 0,
            request_chat_focus: false,
        }
    }

    pub fn add_chat_message(&mut self, msg: String) {
        if self.chat_messages.len() >= 100 {
            self.chat_messages.remove(0);
        }
        self.chat_messages.push(ChatMessageEntry {
            received_at: Instant::now(),
            text: msg,
        });
    }

    /// Synchronizes spatial & benchmark properties into the HUD data struct.
    pub fn sync_hud_data(&mut self) {
        self.hud_data.pos = self.pos;
        self.hud_data.look = self.look;
        self.hud_data.dimension = self.dimension;
        self.hud_data.benchmark = self.benchmark.clone();
    }

    pub fn render(&mut self, ctx: &egui::Context) -> InGameAction {
        let mut action = InGameAction::None;
        self.tick_counter = self.tick_counter.wrapping_add(1);
        self.sync_hud_data();

        let screen_rect = ctx.screen_rect();
        let screen_size = screen_rect.size();

        // If HUD editor is open, render editor exclusively
        if self.hud_editor_opened {
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                self.hud_editor_opened = false;
                let _ = self.hud_layout.save_to_file(&self.layout_path);
                return InGameAction::None;
            }

            let closed = render_hud_editor(
                ctx,
                &mut self.hud_editor,
                &mut self.hud_layout,
                &self.hud_data,
                self.tick_counter,
            );
            if closed {
                self.hud_editor_opened = false;
                let _ = self.hud_layout.save_to_file(&self.layout_path);
            }
            return InGameAction::None;
        }

        // 2. Check Escape key: closes chat if open, otherwise toggles Escape menu
        if !self.is_modal_open && ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.chat_opened {
                self.chat_opened = false;
                self.chat_input.clear();
            } else {
                self.escape_menu_opened = !self.escape_menu_opened;
            }
        }

        // 3. Check Tab key for Tab List overlay
        if !self.escape_menu_opened && ctx.input(|i| i.key_down(Key::Tab)) {
            self.show_tab_list = true;
        } else {
            self.show_tab_list = false;
        }

        // 4. Check Toggle FPS / Telemetry key (e.g. F3)
        if !self.chat_opened {
            let toggle_key = standard_token_to_egui_key(&self.toggle_fps_key).unwrap_or(Key::F3);
            if ctx.input(|i| i.key_pressed(toggle_key)) {
                self.show_fps = !self.show_fps;
            }
        }

        // 5. Check T or Enter or Slash (/) to open chat
        if !self.chat_opened && !self.escape_menu_opened {
            if ctx.input(|i| i.key_pressed(Key::T) || i.key_pressed(Key::Enter)) {
                self.chat_opened = true;
                self.request_chat_focus = true;
            } else if ctx.input(|i| i.key_pressed(Key::Slash)) {
                self.chat_opened = true;
                self.chat_input = "/".to_string();
                self.request_chat_focus = true;
            }
        }

        // 6. RENDER MODULAR HUD ELEMENTS (Driven by HudLayoutConfig)
        let hud_painter = ctx.layer_painter(LayerId::new(Order::Middle, Id::new("hud_modular_layer")));

        for (id, elem) in self.hud_layout.active_elements() {
            if !elem.visible {
                continue;
            }

            // Crosshair is hidden when escape menu is open
            if id == "crosshair" && self.escape_menu_opened {
                continue;
            }

            let content_size = get_module_content_size(id, &self.hud_data);
            let rect = elem.screen_rect(screen_size, content_size);

            // Draw element background & border if configured
            if elem.bg_color[3] > 0 {
                hud_painter.rect_filled(
                    rect,
                    CornerRadius::same(elem.corner_radius as u8),
                    elem.bg_color32(),
                );
            }
            if elem.border_width > 0.0 && elem.border_color[3] > 0 {
                hud_painter.rect_stroke(
                    rect,
                    CornerRadius::same(elem.corner_radius as u8),
                    Stroke::new(elem.border_width, elem.border_color32()),
                    egui::StrokeKind::Inside,
                );
            }

            // Special cases: Scoreboard & Chat are interactive or multi-line egui Areas
            if id == "scoreboard" {
                if let Some((title, lines)) = self.scoreboard.get_sidebar_lines() {
                    egui::Area::new(Id::new("scoreboard_sidebar_area"))
                        .fixed_pos(rect.min)
                        .show(ctx, |ui| {
                            Frame::NONE
                                .fill(elem.bg_color32())
                                .corner_radius(CornerRadius::same(elem.corner_radius as u8))
                                .inner_margin(egui::Margin::symmetric(8, 6))
                                .show(ui, |ui| {
                                    ui.set_width(rect.width());
                                    ui.vertical(|ui| {
                                        // Title
                                        ui.horizontal(|ui| {
                                            for (color, text) in parse_minecraft_text(&title) {
                                                ui.label(RichText::new(text).color(color).strong());
                                            }
                                        });

                                        ui.separator();

                                        for (entry, val) in lines {
                                            ui.horizontal(|ui| {
                                                for (color, text) in parse_minecraft_text(&entry) {
                                                    ui.label(RichText::new(text).color(color));
                                                }
                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Center),
                                                    |ui| {
                                                        ui.label(
                                                            RichText::new(format!("{}", val))
                                                                .color(Color32::from_rgb(255, 85, 85)),
                                                        );
                                                    },
                                                );
                                            });
                                        }
                                    });
                                });
                        });
                }
            } else if id == "chat" {
                let now = Instant::now();
                egui::Area::new(Id::new("chat_area"))
                    .fixed_pos(rect.min)
                    .show(ctx, |ui| {
                        let bg_alpha = if self.chat_opened { elem.bg_color[3] } else { (elem.bg_color[3] as f32 * 0.7) as u8 };
                        Frame::NONE
                            .fill(Color32::from_rgba_unmultiplied(
                                elem.bg_color[0],
                                elem.bg_color[1],
                                elem.bg_color[2],
                                bg_alpha,
                            ))
                            .corner_radius(CornerRadius::same(elem.corner_radius as u8))
                            .inner_margin(egui::Margin::symmetric(8, 8))
                            .show(ui, |ui| {
                                ui.set_width(rect.width());

                                ScrollArea::vertical()
                                    .max_height(rect.height() - if self.chat_opened { 36.0 } else { 0.0 })
                                    .stick_to_bottom(true)
                                    .show(ui, |ui| {
                                        for entry in &self.chat_messages {
                                            // Smooth 10s fading when chat is closed
                                            let alpha = if self.chat_opened {
                                                255
                                            } else {
                                                let age = now.duration_since(entry.received_at).as_secs_f32();
                                                if age > 14.0 {
                                                    continue;
                                                } else if age > 10.0 {
                                                    ((1.0 - (age - 10.0) / 4.0) * 255.0) as u8
                                                } else {
                                                    255
                                                }
                                            };

                                            ui.horizontal_wrapped(|ui| {
                                                for word in entry.text.split_whitespace() {
                                                    if word.starts_with("http://") || word.starts_with("https://") {
                                                        // Clickable URL Link
                                                        if ui.link(RichText::new(word).color(Color32::from_rgba_unmultiplied(85, 255, 255, alpha)).underline()).clicked() {
                                                            ctx.open_url(egui::OpenUrl::new_tab(word));
                                                        }
                                                    } else {
                                                        for seg in crate::theme::parse_formatted_segments(word) {
                                                            let mut col = seg.style.color;
                                                            col = Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), alpha);
                                                            let mut rt = RichText::new(seg.text).color(col);
                                                            if seg.style.bold { rt = rt.strong(); }
                                                            if seg.style.italic { rt = rt.italics(); }
                                                            ui.label(rt);
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    });

                                // Chat Input Box with Tab Autocompletion
                                if self.chat_opened {
                                    ui.separator();

                                    // Tab completion logic
                                    if ctx.input(|i| i.key_pressed(Key::Tab)) {
                                        if self.chat_input.starts_with('/') {
                                            let cmds = ["/help", "/ping", "/spawn", "/msg", "/gamemode", "/clear", "/party", "/guild", "/stats"];
                                            let cur = self.chat_input.to_lowercase();
                                            if let Some(&matched) = cmds.iter().find(|&&c| c.starts_with(&cur)) {
                                                self.chat_input = format!("{} ", matched);
                                            }
                                        } else if let Some(last_word) = self.chat_input.split_whitespace().last() {
                                            let last_lower = last_word.to_lowercase();
                                            for entry in self.tab_list.entries.values() {
                                                if entry.name.to_lowercase().starts_with(&last_lower) {
                                                    let prefix_len = self.chat_input.len() - last_word.len();
                                                    self.chat_input.truncate(prefix_len);
                                                    self.chat_input.push_str(&entry.name);
                                                    self.chat_input.push(' ');
                                                    break;
                                                }
                                            }
                                        }
                                    }

                                    ui.horizontal(|ui| {
                                        let edit = TextEdit::singleline(&mut self.chat_input)
                                            .hint_text("Press Enter to send, Tab to autocomplete...")
                                            .desired_width(rect.width() - 60.0);

                                        let response = ui.add(edit);
                                        if self.request_chat_focus {
                                            response.request_focus();
                                            self.request_chat_focus = false;
                                        }

                                        if (response.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)))
                                            || ui.button("Send").clicked()
                                        {
                                            let trimmed = self.chat_input.trim().to_string();
                                            if !trimmed.is_empty() {
                                                action = InGameAction::SendChat(trimmed);
                                            }
                                            self.chat_input.clear();
                                            self.chat_opened = false;
                                        }
                                    });
                                }
                            });
                    });
            } else {
                // All visual modules: Crosshair, Hotbar, Vitals, Exp Bar, Boss Bar, Keystrokes, FPS/Ping, Armor Status, Potions, Coords
                render_module_by_id(&hud_painter, id, rect, &self.hud_data, self.tick_counter);
            }
        }

        // 7. Tab List Overlay (when holding Tab key)
        if self.show_tab_list {
            let sorted_players = self.tab_list.get_sorted_entries();
            Window::new("TabListWindow")
                .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 16.0))
                .title_bar(false)
                .resizable(false)
                .frame(
                    Frame::NONE
                        .fill(Color32::from_rgba_unmultiplied(12, 14, 18, 225))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(65, 70, 80, 200)))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(egui::Margin::symmetric(8, 4)),
                )
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(format!("Players ({})", sorted_players.len()))
                                .size(11.0)
                                .color(Color32::from_rgb(180, 185, 195))
                                .strong(),
                        );
                        ui.add_space(2.0);

                        ScrollArea::vertical()
                            .max_height(320.0)
                            .auto_shrink([true, true])
                            .show(ui, |ui| {
                                egui::Grid::new("tab_list_grid")
                                    .num_columns(2)
                                    .spacing([12.0, 1.5])
                                    .show(ui, |ui| {
                                        for p in sorted_players {
                                            let raw_name = if let Some(dn) = &p.display_name {
                                                dn.clone()
                                            } else {
                                                self.scoreboard.format_player_name(&p.name)
                                            };

                                            let name_job = crate::theme::build_layout_job_styled(
                                                &raw_name,
                                                11.5,
                                                false,
                                                None,
                                            );
                                            ui.label(name_job);

                                            let ping_color = if p.ping < 80 {
                                                Color32::from_rgb(85, 255, 85)
                                            } else if p.ping < 160 {
                                                Color32::from_rgb(255, 255, 85)
                                            } else {
                                                Color32::from_rgb(255, 85, 85)
                                            };
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.label(
                                                        RichText::new(format!("{}ms", p.ping.max(0)))
                                                            .size(10.5)
                                                            .color(ping_color),
                                                    );
                                                },
                                            );
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
                });
        }

        // 8. Render Nametags (only when escape menu is not open)
        if !self.escape_menu_opened {
            let nametag_painter = ctx.layer_painter(LayerId::new(Order::Background, Id::new("player_nametags")));
            for nametag in &self.nametags {
                let max_dist = if nametag.is_sneaking { 32.0 } else { 64.0 };
                if nametag.distance > max_dist {
                    continue;
                }

                let font_size = (88.0 / nametag.distance.max(3.0)).clamp(3.5, 18.0);
                let fade = ((max_dist - nametag.distance) / (max_dist * 0.25)).clamp(0.0, 1.0);
                let base_alpha = if nametag.is_sneaking { 60 } else { 150 };
                let alpha = (base_alpha as f32 * fade) as u8;

                if alpha < 5 {
                    continue;
                }

                let text_alpha = if nametag.is_sneaking {
                    (140.0 * fade) as u8
                } else {
                    (255.0 * fade) as u8
                };

                let job = crate::theme::build_layout_job(&nametag.text, font_size, Some(text_alpha));
                let galley = nametag_painter.layout_job(job);
                let text_w = galley.size().x;
                let text_h = galley.size().y;
                let center = Pos2::new(nametag.screen_pos.0, nametag.screen_pos.1);

                let pad_x = (font_size * 0.3).clamp(2.0, 6.0);
                let pad_y = (font_size * 0.15).clamp(1.0, 3.0);
                let bg_rect = Rect::from_center_size(center, Vec2::new(text_w + pad_x * 2.0, text_h + pad_y * 2.0));

                nametag_painter.rect_filled(bg_rect, CornerRadius::same(2), Color32::from_black_alpha(alpha));
                nametag_painter.galley(
                    Pos2::new(center.x - text_w * 0.5, center.y - text_h * 0.5),
                    galley,
                    Color32::WHITE,
                );
            }
        }

        // 9. Escape / Game Menu Modal Window (toggled via Esc key)
        if self.escape_menu_opened {
            let screen_rect = ctx.screen_rect();
            ctx.layer_painter(LayerId::new(Order::Middle, Id::new("esc_backdrop")))
                .rect_filled(screen_rect, 0.0, Color32::from_black_alpha(150));

            Window::new("InGameEscapeMenu")
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .frame(
                    Frame::NONE
                        .fill(Color32::from_rgba_unmultiplied(18, 22, 28, 245))
                        .stroke(Stroke::new(1.5_f32, Color32::from_rgb(60, 68, 85)))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(32, 26)),
                )
                .show(ctx, |ui| {
                    ui.set_width(260.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(t(self.language, "ingame.menu_title"))
                                .size(20.0)
                                .color(Color32::WHITE)
                                .strong(),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("{} {}", t(self.language, "ingame.connected_to"), self.server_ip))
                                .size(12.0)
                                .color(Color32::from_rgb(85, 255, 85)),
                        );

                        let dim_str = match self.dimension {
                            0 => "Overworld",
                            -1 => "Nether",
                            1 => "The End",
                            _ => "Custom Dimension",
                        };
                        ui.label(
                            RichText::new(format!("Dimension: {}", dim_str))
                                .size(11.0)
                                .color(Color32::from_rgb(160, 165, 180)),
                        );

                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(14.0);

                        let btn_size = Vec2::new(240.0, 32.0);

                        if ui
                            .add_sized(
                                btn_size,
                                egui::Button::new(
                                    RichText::new(t(self.language, "ingame.back_to_game"))
                                        .size(14.0)
                                        .color(Color32::WHITE)
                                        .strong(),
                                ),
                            )
                            .clicked()
                        {
                            self.escape_menu_opened = false;
                        }

                        ui.add_space(6.0);

                        // HUD Editor button in game menu
                        if ui
                            .add_sized(
                                btn_size,
                                egui::Button::new(
                                    RichText::new("HUD Layout Editor (RShift)")
                                        .size(13.0)
                                        .color(Color32::from_rgb(85, 255, 255)),
                                ),
                            )
                            .clicked()
                        {
                            self.escape_menu_opened = false;
                            self.hud_editor_opened = true;
                        }

                        ui.add_space(6.0);

                        if ui
                            .add_sized(
                                btn_size,
                                egui::Button::new(
                                    RichText::new(t(self.language, "ingame.options"))
                                        .size(13.0)
                                        .color(Color32::from_rgb(220, 225, 235)),
                                ),
                            )
                            .clicked()
                        {
                            action = InGameAction::OpenSettings;
                        }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        if ui
                            .add_sized(
                                btn_size,
                                egui::Button::new(
                                    RichText::new(t(self.language, "ingame.disconnect"))
                                        .size(14.0)
                                        .color(Color32::from_rgb(255, 85, 85))
                                        .strong(),
                                ),
                            )
                            .clicked()
                        {
                            action = InGameAction::Disconnect;
                        }
                    });
                });
        }

        action
    }
}
