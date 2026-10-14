use crate::i18n::{t, Language};
use crate::keybinds::standard_token_to_egui_key;
use crate::theme::parse_minecraft_text;
use egui::{
    Align, Align2, Color32, CornerRadius, Frame, Key, Layout, Rect, RichText, ScrollArea, Stroke,
    TextEdit, Vec2, Window,
};
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

/// Represents the in-game HUD overlay screen during the Play state.
pub struct InGameScreen {
    pub server_ip: String,
    pub player_name: String,
    pub pos: (f64, f64, f64),
    pub look: (f32, f32),
    pub dimension: i8,
    pub chat_messages: Vec<String>,
    pub chat_input: String,
    pub chat_opened: bool,
    pub escape_menu_opened: bool,
    pub show_fps: bool,
    pub toggle_fps_key: String,
    pub language: Language,
    pub show_tab_list: bool,
    pub tab_list: TabList,
    pub scoreboard: Scoreboard,
    pub benchmark: Option<BenchmarkSnapshot>,
    pub is_modal_open: bool,
    pub nametags: Vec<NametagInfo>,
    request_chat_focus: bool,
}

impl InGameScreen {
    pub fn new(server_ip: String, player_name: String) -> Self {
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
            show_fps: false,
            toggle_fps_key: "F3".to_string(),
            language: Language::Russian,
            show_tab_list: false,
            tab_list: TabList::new(),
            scoreboard: Scoreboard::new(),
            benchmark: None,
            is_modal_open: false,
            nametags: Vec::new(),
            request_chat_focus: false,
        }
    }

    pub fn add_chat_message(&mut self, msg: String) {
        if self.chat_messages.len() >= 100 {
            self.chat_messages.remove(0);
        }
        self.chat_messages.push(msg);
    }

    pub fn render(&mut self, ctx: &egui::Context) -> InGameAction {
        let mut action = InGameAction::None;

        // 1. Check Escape key: closes chat if open, otherwise toggles Escape menu (only when no modal is open)
        if !self.is_modal_open && ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.chat_opened {
                self.chat_opened = false;
                self.chat_input.clear();
            } else {
                self.escape_menu_opened = !self.escape_menu_opened;
            }
        }

        // 2. Check Tab key for Tab List (only when escape menu is closed)
        if !self.escape_menu_opened && ctx.input(|i| i.key_down(Key::Tab)) {
            self.show_tab_list = true;
        } else {
            self.show_tab_list = false;
        }

        // 3. Check Toggle FPS / Telemetry key (e.g. F3) when not typing in chat
        if !self.chat_opened {
            let toggle_key = standard_token_to_egui_key(&self.toggle_fps_key).unwrap_or(Key::F3);
            if ctx.input(|i| i.key_pressed(toggle_key)) {
                self.show_fps = !self.show_fps;
            }
        }

        // 4. Check T or Enter to open chat if not already opened and escape menu not open
        if !self.chat_opened && !self.escape_menu_opened {
            if ctx.input(|i| i.key_pressed(Key::T) || i.key_pressed(Key::Enter)) {
                self.chat_opened = true;
                self.request_chat_focus = true;
            }
        }

        // 5. Top-Left Telemetry HUD (Coordinates, FPS, 1% Low, Latency) - toggleable via key
        if self.show_fps {
            egui::Area::new(egui::Id::new("telemetry_hud_area"))
                .anchor(Align2::LEFT_TOP, Vec2::new(14.0, 14.0))
                .show(ctx, |ui| {
                    Frame::NONE
                        .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 160))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                if let Some(b) = &self.benchmark {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("FPS: {:.0}", b.current_fps))
                                                .color(Color32::from_rgb(85, 255, 85))
                                                .strong(),
                                        );
                                        ui.label(
                                            RichText::new(format!("1% Low: {:.0}", b.low_1_percent_fps))
                                                .color(Color32::from_rgb(255, 255, 85)),
                                        );
                                        ui.label(
                                            RichText::new(format!(
                                                "0.1% Low: {:.0}",
                                                b.low_0_1_percent_fps
                                            ))
                                            .color(Color32::from_rgb(255, 170, 0)),
                                        );
                                    });

                                    // Frame Stage Timings Breakdown
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("Stages:")
                                                .color(Color32::from_rgb(175, 180, 190))
                                                .strong(),
                                        );
                                        ui.label(
                                            RichText::new(format!("Net: {:.2}ms", b.stages.network_ms))
                                                .color(Color32::from_rgb(85, 255, 255)),
                                        );
                                        ui.label(
                                            RichText::new(format!("UI: {:.2}ms", b.stages.gui_ms))
                                                .color(Color32::from_rgb(255, 215, 85)),
                                        );
                                        ui.label(
                                            RichText::new(format!("GPU: {:.2}ms", b.stages.render_ms))
                                                .color(Color32::from_rgb(130, 255, 130)),
                                        );
                                    });

                                    // Interactive Frametime Sparkline / Graph
                                    if !b.recent_frametimes.is_empty() {
                                        ui.add_space(3.0);
                                        let graph_size = Vec2::new(260.0, 50.0);
                                        let (response, painter) = ui.allocate_painter(graph_size, egui::Sense::hover());
                                        let rect = response.rect;

                                        // Dark sleek graph background
                                        painter.rect_filled(
                                            rect,
                                            CornerRadius::same(3),
                                            Color32::from_rgba_unmultiplied(16, 18, 24, 230),
                                        );
                                        painter.rect_stroke(
                                            rect,
                                            CornerRadius::same(3),
                                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(65, 70, 82, 180)),
                                            egui::StrokeKind::Inside,
                                        );

                                        // Vertical ceiling (at least 33.33ms for 30 FPS guide)
                                        let max_sample = b.recent_frametimes.iter().copied().fold(33.33_f64, f64::max);
                                        let max_ms = (max_sample * 1.15).min(100.0);

                                        // 16.67ms (60 FPS) guideline
                                        let y_60 = rect.bottom() - ((16.67 / max_ms) as f32 * rect.height());
                                        if y_60 >= rect.top() {
                                            painter.line_segment(
                                                [egui::pos2(rect.left(), y_60), egui::pos2(rect.right(), y_60)],
                                                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 170, 0, 70)),
                                            );
                                            painter.text(
                                                egui::pos2(rect.right() - 3.0, y_60 - 2.0),
                                                Align2::RIGHT_BOTTOM,
                                                "16.7ms (60)",
                                                egui::FontId::monospace(9.0),
                                                Color32::from_rgba_unmultiplied(255, 170, 0, 150),
                                            );
                                        }

                                        // 8.33ms (120 FPS) guideline
                                        let y_120 = rect.bottom() - ((8.33 / max_ms) as f32 * rect.height());
                                        if y_120 >= rect.top() {
                                            painter.line_segment(
                                                [egui::pos2(rect.left(), y_120), egui::pos2(rect.right(), y_120)],
                                                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(85, 255, 85, 60)),
                                            );
                                            painter.text(
                                                egui::pos2(rect.right() - 3.0, y_120 - 2.0),
                                                Align2::RIGHT_BOTTOM,
                                                "8.3ms (120)",
                                                egui::FontId::monospace(9.0),
                                                Color32::from_rgba_unmultiplied(85, 255, 85, 140),
                                            );
                                        }

                                        // Render individual frame bars
                                        let count = b.recent_frametimes.len();
                                        let bar_width = rect.width() / count as f32;

                                        let hovered_idx = response.hover_pos().and_then(|hp| {
                                            if rect.contains(hp) {
                                                let rel_x = hp.x - rect.left();
                                                let idx = ((rel_x / rect.width()) * count as f32).floor() as usize;
                                                Some(idx.min(count - 1))
                                            } else {
                                                None
                                            }
                                        });

                                        for (i, &ms) in b.recent_frametimes.iter().enumerate() {
                                            let x0 = rect.left() + i as f32 * bar_width;
                                            let x1 = x0 + (bar_width - 0.5).max(1.0);
                                            let h = ((ms / max_ms) as f32 * rect.height()).clamp(1.0, rect.height());
                                            let y0 = rect.bottom() - h;

                                            let color = if ms <= 8.33 {
                                                Color32::from_rgb(85, 255, 85) // >120 FPS
                                            } else if ms <= 16.67 {
                                                Color32::from_rgb(140, 230, 80) // 60-120 FPS
                                            } else if ms <= 33.33 {
                                                Color32::from_rgb(255, 180, 50) // 30-60 FPS
                                            } else {
                                                Color32::from_rgb(255, 75, 75) // <30 FPS
                                            };

                                            let bar_rect = Rect::from_min_max(
                                                egui::pos2(x0, y0),
                                                egui::pos2(x1, rect.bottom()),
                                            );
                                            painter.rect_filled(bar_rect, CornerRadius::ZERO, color);
                                        }

                                        // Hover interaction: indicator line & tooltip
                                        if let Some(idx) = hovered_idx {
                                            let hovered_ms = b.recent_frametimes[idx];
                                            let hovered_fps = if hovered_ms > 0.0 { 1000.0 / hovered_ms } else { 0.0 };
                                            let hx = rect.left() + (idx as f32 + 0.5) * bar_width;

                                            painter.line_segment(
                                                [egui::pos2(hx, rect.top()), egui::pos2(hx, rect.bottom())],
                                                Stroke::new(1.5_f32, Color32::WHITE),
                                            );

                                            let bar_h = ((hovered_ms / max_ms) as f32 * rect.height()).clamp(1.0, rect.height());
                                            let bar_top_y = rect.bottom() - bar_h;
                                            painter.circle_filled(egui::pos2(hx, bar_top_y), 2.5, Color32::WHITE);

                                            response.on_hover_ui(|ui| {
                                                ui.label(
                                                    RichText::new(format!(
                                                        "Frame -{}: {:.2} ms ({:.0} FPS)",
                                                        count - 1 - idx,
                                                        hovered_ms,
                                                        hovered_fps
                                                    ))
                                                    .color(Color32::WHITE)
                                                    .monospace()
                                                    .strong(),
                                                );
                                            });
                                        }
                                        ui.add_space(3.0);
                                    }

                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("RTT: {:.1} ms", b.rtt_ms))
                                                .color(Color32::from_rgb(85, 255, 255)),
                                        );
                                        ui.label(
                                            RichText::new(format!("Jitter: {:.1} ms", b.jitter_ms))
                                                .color(Color32::from_rgb(170, 170, 170)),
                                        );
                                        ui.label(
                                            RichText::new(format!(
                                                "Down: {:.1} KB/s",
                                                b.bytes_in_per_sec / 1024.0
                                            ))
                                            .color(Color32::from_rgb(170, 170, 170)),
                                        );
                                    });

                                    ui.label(
                                        RichText::new(format!("Chunks: {}", b.loaded_chunks))
                                            .color(Color32::from_rgb(200, 205, 215)),
                                    );
                                }

                                ui.separator();

                                ui.label(
                                    RichText::new(format!(
                                        "XYZ: {:.2} / {:.2} / {:.2}",
                                        self.pos.0, self.pos.1, self.pos.2
                                    ))
                                    .color(Color32::WHITE),
                                );

                                ui.label(
                                    RichText::new(format!(
                                        "Facing: Yaw {:.1} / Pitch {:.1}",
                                        self.look.0, self.look.1
                                    ))
                                    .color(Color32::from_rgb(180, 185, 195)),
                                );
                            });
                        });
                });
        }

        // 6. Right Sidebar Scoreboard (Protocol 47 Sidebar)
        if let Some((title, lines)) = self.scoreboard.get_sidebar_lines() {
            egui::Area::new(egui::Id::new("scoreboard_sidebar_area"))
                .anchor(Align2::RIGHT_CENTER, Vec2::new(-14.0, 0.0))
                .show(ctx, |ui| {
                    Frame::NONE
                        .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 170))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
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

        // 7. Bottom-Left Chat Window & Input Box
        egui::Area::new(egui::Id::new("chat_area"))
            .anchor(Align2::LEFT_BOTTOM, Vec2::new(14.0, -14.0))
            .show(ctx, |ui| {
                let bg_alpha = if self.chat_opened { 200 } else { 120 };
                Frame::NONE
                    .fill(Color32::from_rgba_unmultiplied(0, 0, 0, bg_alpha))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(egui::Margin::symmetric(8, 8))
                    .show(ui, |ui| {
                        ui.set_width(450.0);
                        ScrollArea::vertical()
                            .max_height(200.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for msg in &self.chat_messages {
                                    ui.horizontal_wrapped(|ui| {
                                        for seg in crate::theme::parse_formatted_segments(msg) {
                                            let mut rt = RichText::new(seg.text).color(seg.style.color);
                                            if seg.style.bold {
                                                rt = rt.strong();
                                            }
                                            if seg.style.italic {
                                                rt = rt.italics();
                                            }
                                            if seg.style.underline {
                                                rt = rt.underline();
                                            }
                                            if seg.style.strikethrough {
                                                rt = rt.strikethrough();
                                            }
                                            ui.label(rt);
                                        }
                                    });
                                }
                            });

                        if self.chat_opened {
                            ui.separator();
                            ui.horizontal(|ui| {
                                let edit = TextEdit::singleline(&mut self.chat_input)
                                    .hint_text("Press Enter to send chat message...")
                                    .desired_width(380.0);

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

        // 8. Tab List Overlay (when holding Tab key)
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

        // 8.5. Render Nametags (only when escape menu is not open)
        if !self.escape_menu_opened {
            let nametag_painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("player_nametags")));
            for nametag in &self.nametags {
                let max_dist = if nametag.is_sneaking { 32.0 } else { 64.0 };
                if nametag.distance > max_dist {
                    continue;
                }

                // Perspective projection: text size on screen is inversely proportional to distance (1 / distance).
                // At 8 blocks distance, standard legible size is ~11.0 px.
                // Clamped to [3.5, 18.0] so it shrinks naturally into the distance alongside the 3D entity model.
                let font_size = (88.0 / nametag.distance.max(3.0)).clamp(3.5, 18.0);

                // Alpha fades out as the player reaches the maximum render distance
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
                let center = egui::pos2(nametag.screen_pos.0, nametag.screen_pos.1);

                let pad_x = (font_size * 0.3).clamp(2.0, 6.0);
                let pad_y = (font_size * 0.15).clamp(1.0, 3.0);
                let bg_rect = Rect::from_center_size(center, Vec2::new(text_w + pad_x * 2.0, text_h + pad_y * 2.0));

                nametag_painter.rect_filled(bg_rect, CornerRadius::same(2), Color32::from_black_alpha(alpha));
                nametag_painter.galley(
                    egui::pos2(center.x - text_w * 0.5, center.y - text_h * 0.5),
                    galley,
                    Color32::WHITE,
                );
            }
        }

        // 9. Escape / Game Menu Modal Window (toggled via Esc key)
        if self.escape_menu_opened {
            // Semi-transparent backdrop overlay across viewport
            let screen_rect = ctx.screen_rect();
            ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("esc_backdrop")))
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

                        let btn_size = Vec2::new(240.0, 34.0);

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

                        ui.add_space(8.0);

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
