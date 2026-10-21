use crate::hud::{
    render_hud_editor, render_module_by_id, HudData, HudEditorState, HudLayoutConfig,
};
use crate::i18n::{t, Language};
use crate::keybinds::standard_token_to_egui_key;
use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Id, Key, LayerId, Order, Pos2, Rect,
    RichText, ScrollArea, Stroke, StrokeKind, TextEdit, Vec2, Window,
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

#[derive(Debug, Clone, PartialEq)]
pub enum InGameAction {
    None,
    Disconnect,
    OpenSettings,
    SendChat(String),
    Respawn,
    DropItem { drop_all: bool },
    Container(crate::screens::container::ContainerAction),
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
    pub chat_history: Vec<String>,
    pub chat_history_index: Option<usize>,
    pub chat_uncommitted: String,
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
    pub gamemode: u8,
    pub is_creative: bool,

    // Container & Inventory state
    pub player_inventory: world::ContainerState,
    pub active_container: Option<world::ContainerState>,
    pub container_ui_state: crate::screens::container::ContainerUiState,
    pub inventory_opened: bool,
    pub inventory_key: String,

    pub tab_list_scroll: f32,
    pub credits_screen: Option<crate::screens::credits::CreditsScreen>,

    // Modular HUD Layout and Editor
    pub hud_layout: HudLayoutConfig,
    pub hud_editor: HudEditorState,
    pub hud_data: HudData,
    pub layout_path: PathBuf,
    pub tick_counter: u64,
    pub death_ticks: u32,
    pub show_scoreboard_numbers: bool,

    request_chat_focus: bool,
}

impl InGameScreen {
    pub fn open_credits(&mut self) {
        self.credits_screen = Some(crate::screens::credits::CreditsScreen::new(&self.player_name));
        self.escape_menu_opened = false;
        self.chat_opened = false;
        self.hud_editor_opened = false;
        self.active_container = None;
        self.inventory_opened = false;
    }

    pub fn new(server_ip: String, player_name: String) -> Self {
        let layout_path = PathBuf::from("hud_layout.toml");
        let hud_layout = HudLayoutConfig::load_or_default(&layout_path);
        let show_scoreboard_numbers = hud_layout.scoreboard_numbers;

        Self {
            server_ip,
            player_name,
            pos: (0.0, 64.0, 0.0),
            look: (0.0, 0.0),
            dimension: 0,
            chat_messages: Vec::new(),
            chat_input: String::new(),
            chat_opened: false,
            chat_history: Vec::new(),
            chat_history_index: None,
            chat_uncommitted: String::new(),
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
            gamemode: 0,
            is_creative: false,
            player_inventory: world::ContainerState::new_player_inventory(),
            active_container: None,
            container_ui_state: crate::screens::container::ContainerUiState::default(),
            inventory_opened: false,
            inventory_key: "E".to_string(),
            tab_list_scroll: 0.0,
            credits_screen: None,
            hud_layout,
            hud_editor: HudEditorState::default(),
            hud_data: HudData::default(),
            layout_path,
            tick_counter: 0,
            death_ticks: 0,
            show_scoreboard_numbers,
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
        self.hud_data.gamemode = self.gamemode;
        self.hud_data.is_creative = self.is_creative || self.gamemode == 1;
        self.hud_data.benchmark = self.benchmark.clone();
    }

    pub fn render(&mut self, ctx: &egui::Context) -> InGameAction {
        let mut action = InGameAction::None;
        self.tick_counter = self.tick_counter.wrapping_add(1);
        self.sync_hud_data();

        // 0. End Poem & Credits Screen (GuiWinGame)
        if let Some(ref mut credits) = self.credits_screen {
            match credits.render(ctx) {
                crate::screens::credits::CreditsAction::CloseAndRespawn => {
                    self.credits_screen = None;
                    return InGameAction::Respawn;
                }
                crate::screens::credits::CreditsAction::None => {
                    return InGameAction::None;
                }
            }
        }

        let screen_rect = ctx.screen_rect();
        let screen_size = screen_rect.size();

        // 1. Death Screen (GuiGameOver) handling when player health <= 0.0
        if self.hud_data.health <= 0.0 {
            self.death_ticks = self.death_ticks.saturating_add(1);
            self.escape_menu_opened = false;
            self.chat_opened = false;
            self.hud_editor_opened = false;

            // Fullscreen classic red vignette wash (rendered in Background behind the modal window)
            let death_painter = ctx.layer_painter(LayerId::new(Order::Background, Id::new("death_overlay")));
            death_painter.rect_filled(
                screen_rect,
                CornerRadius::ZERO,
                Color32::from_rgba_unmultiplied(110, 15, 15, 175),
            );

            let can_respawn = self.death_ticks >= 20;

            Window::new("DeathScreenModal")
                .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, -10.0))
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .frame(
                    Frame::NONE
                        .fill(Color32::from_rgba_unmultiplied(20, 10, 12, 235))
                        .stroke(Stroke::new(1.5_f32, Color32::from_rgb(110, 25, 25)))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(36, 28)),
                )
                .show(ctx, |ui| {
                    ui.set_width(260.0);
                    ui.vertical_centered(|ui| {
                        // Title: "Вы погибли!" / "You died!"
                        ui.label(
                            RichText::new(t(self.language, "death.title"))
                                .size(28.0)
                                .color(Color32::from_rgb(255, 60, 60))
                                .strong(),
                        );
                        ui.add_space(8.0);

                        // Score: "Счёт: 0" / "Score: 0"
                        let score = self.hud_data.exp_level;
                        ui.label(
                            RichText::new(format!("{} {}", t(self.language, "death.score"), score))
                                .size(14.0)
                                .color(Color32::from_rgb(255, 255, 120)),
                        );

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(16.0);

                        let btn_size = Vec2::new(240.0, 34.0);

                        // Respawn button (disabled for first 20 ticks = 1 second)
                        let respawn_btn = egui::Button::new(
                            RichText::new(t(self.language, "death.respawn"))
                                .size(14.0)
                                .color(if can_respawn { Color32::WHITE } else { Color32::from_rgb(120, 120, 120) })
                                .strong(),
                        );

                        if ui.add_enabled(can_respawn, respawn_btn.min_size(btn_size)).clicked() {
                            action = InGameAction::Respawn;
                        }

                        ui.add_space(8.0);

                        // Return to title screen / Disconnect button
                        let title_btn = egui::Button::new(
                            RichText::new(t(self.language, "death.title_screen"))
                                .size(14.0)
                                .color(Color32::from_rgb(230, 230, 240)),
                        );

                        if ui.add_sized(btn_size, title_btn).clicked() {
                            action = InGameAction::Disconnect;
                        }
                    });
                });

            return action;
        } else {
            self.death_ticks = 0;
        }

        // If HUD editor is open, render editor exclusively
        if self.hud_editor_opened {
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                self.hud_editor_opened = false;
                self.show_scoreboard_numbers = self.hud_layout.scoreboard_numbers;
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
            self.show_scoreboard_numbers = self.hud_layout.scoreboard_numbers;
            if closed {
                self.hud_editor_opened = false;
                let _ = self.hud_layout.save_to_file(&self.layout_path);
            }
            return InGameAction::None;
        }

        // 2. Check Escape key: closes modal screens in hierarchical order
        if !self.is_modal_open && ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.container_ui_state.sign_pos.is_some() {
                self.container_ui_state.sign_pos = None;
                return InGameAction::None;
            } else if self.container_ui_state.book_open {
                self.container_ui_state.book_open = false;
                return InGameAction::None;
            } else if let Some(container) = self.active_container.take() {
                return InGameAction::Container(crate::screens::container::ContainerAction::Close(container.window_id as i8));
            } else if self.inventory_opened {
                self.inventory_opened = false;
                return InGameAction::Container(crate::screens::container::ContainerAction::Close(0));
            } else if self.chat_opened {
                self.chat_opened = false;
                self.chat_input.clear();
                self.chat_uncommitted.clear();
                self.chat_history_index = None;
            } else {
                self.escape_menu_opened = !self.escape_menu_opened;
            }
        }

        // 2.5 Check Inventory Key (e.g. E) to open/close player/creative inventory or active container
        let inv_key = standard_token_to_egui_key(&self.inventory_key).unwrap_or(Key::E);
        if !self.escape_menu_opened
            && !self.chat_opened
            && !ctx.wants_keyboard_input()
            && ctx.input(|i| i.key_pressed(inv_key))
        {
            if self.container_ui_state.sign_pos.is_none() && !self.container_ui_state.book_open {
                if let Some(container) = self.active_container.take() {
                    return InGameAction::Container(crate::screens::container::ContainerAction::Close(container.window_id as i8));
                } else if self.inventory_opened {
                    self.inventory_opened = false;
                    return InGameAction::Container(crate::screens::container::ContainerAction::Close(0));
                } else {
                    self.inventory_opened = true;
                    return InGameAction::Container(crate::screens::container::ContainerAction::OpenInventory);
                }
            }
        }

        // 3. Check Tab key for Tab List overlay (only when chat is closed)
        if !self.escape_menu_opened && !self.chat_opened && ctx.input(|i| i.key_down(Key::Tab)) {
            self.show_tab_list = true;
        } else {
            self.show_tab_list = false;
            self.tab_list_scroll = 0.0;
        }

        // 4. Check Toggle FPS / Telemetry key (e.g. F3)
        if !self.chat_opened {
            let toggle_key = standard_token_to_egui_key(&self.toggle_fps_key).unwrap_or(Key::F3);
            if ctx.input(|i| i.key_pressed(toggle_key)) {
                self.show_fps = !self.show_fps;
            }
        }

        // 5. Check T or Enter or Slash (/) to open chat
        if !self.chat_opened && !self.escape_menu_opened && !self.inventory_opened && self.active_container.is_none() {
            if ctx.input(|i| i.key_pressed(Key::T) || i.key_pressed(Key::Enter)) {
                self.chat_opened = true;
                self.chat_history_index = None;
                self.chat_uncommitted.clear();
                self.request_chat_focus = true;
            } else if ctx.input(|i| i.key_pressed(Key::Slash)) {
                self.chat_opened = true;
                self.chat_history_index = None;
                self.chat_uncommitted.clear();
                self.chat_input = "/".to_string();
                self.request_chat_focus = true;
            }
        }

        // 6. RENDER MODULAR HUD ELEMENTS (Driven by HudLayoutConfig)
        let hud_painter = ctx.layer_painter(LayerId::new(Order::Middle, Id::new("hud_modular_layer")));

        // Rigid center crosshair (always at exact screen center when menus/containers are not open)
        if !self.escape_menu_opened
            && !self.inventory_opened
            && self.active_container.is_none()
            && self.container_ui_state.sign_pos.is_none()
            && !self.container_ui_state.book_open
        {
            let center = Pos2::new(screen_size.x * 0.5, screen_size.y * 0.5);
            let crosshair_rect = Rect::from_center_size(center, Vec2::splat(16.0));
            crate::hud::render_crosshair(&hud_painter, crosshair_rect);
        }

        let bg_alpha = (255.0 * self.hud_layout.global_opacity).clamp(0.0, 255.0) as u8;
        let border_alpha = (220.0 * self.hud_layout.global_opacity).clamp(0.0, 255.0) as u8;
        let elem_bg_col = Color32::from_rgba_unmultiplied(16, 18, 24, bg_alpha);
        let elem_border_col = Color32::from_rgba_unmultiplied(65, 72, 88, border_alpha);

        let active_elements: Vec<(String, crate::hud::HudElementConfig)> = self
            .hud_layout
            .active_elements()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (id, elem) in &active_elements {
            if !elem.visible {
                continue;
            }

            // Crosshair is fixed and rendered separately, skip if present in layout
            if id == "crosshair" {
                continue;
            }

            // Skip legacy separate vitals and exp_bar (unified into hotbar)
            if id == "vitals" || id == "exp_bar" {
                continue;
            }

            // Boss health bar is hidden if there is no active boss
            if id == "boss_bar" && self.hud_data.boss_bars.is_empty() {
                continue;
            }

            // Potion effects hidden if no active effects
            if id == "potion_effects" && self.hud_data.potion_effects.is_empty() {
                continue;
            }

            // Armor status hidden if no armor equipped
            if id == "armor_status" {
                let has_armor = self.hud_data.armor_items.iter().any(|a| a.is_some())
                    || self.hud_data.held_item.as_ref().map_or(false, |h| h.max_damage > 0);
                if !has_armor {
                    continue;
                }
            }

            let rect = elem.screen_rect(screen_size);

            // Draw element background & border if configured (unless crosshair, hotbar, scoreboard or chat)
            if id != "crosshair" && id != "hotbar" && id != "scoreboard" && id != "chat" {
                if bg_alpha > 0 {
                    hud_painter.rect_filled(
                        rect,
                        CornerRadius::same(4),
                        elem_bg_col,
                    );
                }
                if border_alpha > 0 {
                    hud_painter.rect_stroke(
                        rect,
                        CornerRadius::same(4),
                        Stroke::new(1.0_f32, elem_border_col),
                        egui::StrokeKind::Inside,
                    );
                }
            }

            // Special cases: Scoreboard & Chat are interactive or multi-line egui Areas
            if id == "scoreboard" {
                if let Some((title, lines)) = self.scoreboard.get_sidebar_lines() {
                    let pad_x = 6.0_f32;
                    let pad_y = 5.0_f32;
                    let content_w = (rect.width() - pad_x * 2.0).max(10.0);
                    let line_count = lines.len();
                    let title_h = 16.0_f32;
                    let sep_h = 4.0_f32;
                    let remaining_h = (rect.height() - pad_y * 2.0 - title_h - sep_h).max(16.0);
                    let row_h = if line_count > 0 {
                        (remaining_h / line_count as f32).clamp(11.0, 18.0)
                    } else {
                        16.0
                    };
                    let base_font_size = (row_h * 0.85).clamp(8.0, 15.0);
                    let total_needed_h = pad_y * 2.0 + title_h + sep_h + line_count as f32 * row_h;
                    let total_h = rect.height().max(total_needed_h.min(rect.height()));
                    let scoreboard_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), total_h));

                    // Background and border strictly matching configured element dimensions
                    hud_painter.rect_filled(scoreboard_rect, CornerRadius::same(4), elem_bg_col);
                    hud_painter.rect_stroke(
                        scoreboard_rect,
                        CornerRadius::same(4),
                        Stroke::new(1.0_f32, elem_border_col),
                        StrokeKind::Inside,
                    );

                    // Render Title (centered, auto-compressing if longer than content_w)
                    let title_y = rect.top() + pad_y;
                    let title_font_size = (base_font_size * 1.08).clamp(9.0, 16.0);
                    let title_job = crate::theme::build_layout_job_styled(&title, title_font_size, false, None);
                    let title_galley = ctx.fonts(|f| f.layout_job(title_job));
                    let title_w = title_galley.size().x;

                    let final_title_galley = if title_w > content_w && content_w > 10.0 {
                        let scale = (content_w / title_w).clamp(0.35, 1.0);
                        let scaled_fs = (title_font_size * scale).max(5.5);
                        let job = crate::theme::build_layout_job_styled(&title, scaled_fs, false, None);
                        ctx.fonts(|f| f.layout_job(job))
                    } else {
                        title_galley
                    };

                    let title_x = rect.left() + pad_x + (content_w - final_title_galley.size().x).max(0.0) * 0.5;
                    hud_painter.galley(Pos2::new(title_x, title_y), final_title_galley, Color32::WHITE);

                    // Horizontal separator line underneath title
                    let sep_y = title_y + title_h;
                    hud_painter.line_segment(
                        [Pos2::new(rect.left() + pad_x, sep_y), Pos2::new(rect.right() - pad_x, sep_y)],
                        Stroke::new(1.0_f32, Color32::from_white_alpha(45)),
                    );

                    // Render Scoreboard lines with compressed text
                    let mut cur_y = sep_y + 4.0;
                    for (entry, val) in lines {
                        if cur_y + row_h > scoreboard_rect.bottom() - pad_y {
                            break;
                        }

                        // Score number on right side (optional based on setting)
                        let val_w = if self.show_scoreboard_numbers {
                            let val_str = format!("{}", val);
                            let val_job = crate::theme::build_layout_job_styled(&val_str, base_font_size, false, None);
                            let val_galley = ctx.fonts(|f| f.layout_job(val_job));
                            let val_w = val_galley.size().x;
                            let val_x = rect.right() - pad_x - val_w;

                            hud_painter.galley(
                                Pos2::new(val_x, cur_y),
                                val_galley,
                                Color32::from_rgb(255, 85, 85),
                            );
                            val_w
                        } else {
                            0.0
                        };

                        // Entry name on left side, compressed to fit within available width
                        let entry_max_w = if self.show_scoreboard_numbers {
                            (content_w - val_w - 6.0).max(10.0)
                        } else {
                            content_w.max(10.0)
                        };
                        let entry_job = crate::theme::build_layout_job_styled(&entry, base_font_size, false, None);
                        let entry_galley = ctx.fonts(|f| f.layout_job(entry_job));
                        let entry_w = entry_galley.size().x;

                        let final_entry_galley = if entry_w > entry_max_w && entry_max_w > 10.0 {
                            let scale = (entry_max_w / entry_w).clamp(0.25, 1.0);
                            let scaled_fs = (base_font_size * scale).max(5.0);
                            let job = crate::theme::build_layout_job_styled(&entry, scaled_fs, false, None);
                            ctx.fonts(|f| f.layout_job(job))
                        } else {
                            entry_galley
                        };

                        let entry_x = rect.left() + pad_x;
                        hud_painter.galley(Pos2::new(entry_x, cur_y), final_entry_galley, Color32::WHITE);

                        cur_y += row_h;
                    }
                }
            } else if id == "chat" {
                let now = Instant::now();
                let has_visible_messages = self.chat_messages.iter().any(|entry| {
                    now.duration_since(entry.received_at).as_secs_f32() <= 14.0
                });
                if !self.chat_opened && !has_visible_messages {
                    continue;
                }
                let chat_font_size = self.hud_layout.chat_font_size.clamp(8.0, 28.0);
                egui::Area::new(Id::new("chat_area"))
                    .fixed_pos(rect.min)
                    .interactable(self.chat_opened)
                    .show(ctx, |ui| {
                        let chat_bg_alpha = if self.chat_opened { bg_alpha } else { (bg_alpha as f32 * 0.7) as u8 };
                        let frame_stroke = if self.chat_opened {
                            Stroke::new(1.0_f32, elem_border_col)
                        } else {
                            Stroke::NONE
                        };
                        Frame::NONE
                            .fill(Color32::from_rgba_unmultiplied(16, 18, 24, chat_bg_alpha))
                            .stroke(frame_stroke)
                            .corner_radius(CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(2, 2))
                            .show(ui, |ui| {
                                ui.set_width(rect.width());

                                ScrollArea::vertical()
                                    .max_height((rect.height() - if self.chat_opened { 42.0 } else { 0.0 }).max(20.0))
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
                                                        if ui.link(RichText::new(word).color(Color32::from_rgba_unmultiplied(85, 255, 255, alpha)).size(chat_font_size).underline()).clicked() {
                                                            ctx.open_url(egui::OpenUrl::new_tab(word));
                                                        }
                                                    } else {
                                                        for seg in crate::theme::parse_formatted_segments(word) {
                                                            let mut col = seg.style.color;
                                                            col = Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), alpha);
                                                            let mut rt = RichText::new(seg.text).color(col).size(chat_font_size);
                                                            if seg.style.bold { rt = rt.strong(); }
                                                            if seg.style.italic { rt = rt.italics(); }
                                                            ui.label(rt);
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    });

                                // Chat Input Box with Tab Autocompletion, History Navigation, and Font Size Controls
                                if self.chat_opened {
                                    ui.separator();

                                    // Tab completion logic
                                    let tab_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Tab));
                                    if tab_pressed {
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
                                        self.request_chat_focus = true;
                                    }

                                    // Arrow Up / Down Chat History Navigation (MCP GuiChat.getSentHistory)
                                    let up_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowUp));
                                    let down_pressed = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::ArrowDown));

                                    if up_pressed && !self.chat_history.is_empty() {
                                        let next_idx = match self.chat_history_index {
                                            None => {
                                                self.chat_uncommitted = self.chat_input.clone();
                                                self.chat_history.len() - 1
                                            }
                                            Some(idx) => {
                                                if idx > 0 { idx - 1 } else { 0 }
                                            }
                                        };
                                        self.chat_history_index = Some(next_idx);
                                        self.chat_input = self.chat_history[next_idx].clone();
                                        self.request_chat_focus = true;
                                    } else if down_pressed {
                                        if let Some(idx) = self.chat_history_index {
                                            if idx + 1 < self.chat_history.len() {
                                                let next_idx = idx + 1;
                                                self.chat_history_index = Some(next_idx);
                                                self.chat_input = self.chat_history[next_idx].clone();
                                            } else {
                                                self.chat_history_index = None;
                                                self.chat_input = self.chat_uncommitted.clone();
                                            }
                                            self.request_chat_focus = true;
                                        }
                                    }

                                    ui.horizontal(|ui| {
                                        let edit = TextEdit::singleline(&mut self.chat_input)
                                            .hint_text("Press Enter to send, Tab to autocomplete...")
                                            .font(FontId::proportional(chat_font_size))
                                            .lock_focus(true)
                                            .desired_width((rect.width() - 8.0).max(60.0));

                                        let output = edit.show(ui);
                                        let response = output.response;
                                        if self.request_chat_focus || tab_pressed || up_pressed || down_pressed {
                                            response.request_focus();
                                            let mut state = TextEdit::load_state(ui.ctx(), response.id).unwrap_or_default();
                                            let char_len = self.chat_input.chars().count();
                                            state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(char_len))));
                                            state.store(ui.ctx(), response.id);
                                            self.request_chat_focus = false;
                                        }

                                        if (response.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)))
                                            || ctx.input(|i| i.key_pressed(Key::Enter))
                                        {
                                            let trimmed = self.chat_input.trim().to_string();
                                            if !trimmed.is_empty() {
                                                if self.chat_history.last().map_or(true, |last| last != &trimmed) {
                                                    if self.chat_history.len() >= 100 {
                                                        self.chat_history.remove(0);
                                                    }
                                                    self.chat_history.push(trimmed.clone());
                                                }
                                                action = InGameAction::SendChat(trimmed);
                                            }
                                            self.chat_input.clear();
                                            self.chat_uncommitted.clear();
                                            self.chat_history_index = None;
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

                        let scroll_out = ScrollArea::vertical()
                            .vertical_scroll_offset(self.tab_list_scroll)
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
                        self.tab_list_scroll = scroll_out.state.offset.y;
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

        // 8.5 CONTAINER & INVENTORY SCREENS
        if self.container_ui_state.sign_pos.is_some() || self.container_ui_state.book_open {
            ctx.layer_painter(LayerId::new(Order::Middle, Id::new("container_backdrop")))
                .rect_filled(screen_rect, 0.0, Color32::from_black_alpha(140));
            let mut dummy = world::ContainerState::new_player_inventory();
            let c_act = crate::screens::container::render_container(
                ctx,
                &mut dummy,
                &mut self.container_ui_state,
                self.hud_data.exp_level,
            );
            if c_act != crate::screens::container::ContainerAction::None {
                return InGameAction::Container(c_act);
            }
        } else if let Some(ref mut c) = self.active_container {
            ctx.layer_painter(LayerId::new(Order::Middle, Id::new("container_backdrop")))
                .rect_filled(screen_rect, 0.0, Color32::from_black_alpha(140));
            let c_act = crate::screens::container::render_container(
                ctx,
                c,
                &mut self.container_ui_state,
                self.hud_data.exp_level,
            );
            if let crate::screens::container::ContainerAction::Close(_) = c_act {
                self.active_container = None;
            }
            if c_act != crate::screens::container::ContainerAction::None {
                return InGameAction::Container(c_act);
            }
        } else if self.inventory_opened {
            ctx.layer_painter(LayerId::new(Order::Middle, Id::new("container_backdrop")))
                .rect_filled(screen_rect, 0.0, Color32::from_black_alpha(140));
            let c_act = if self.hud_data.is_creative {
                crate::screens::container::render_creative_inventory(
                    ctx,
                    &mut self.container_ui_state,
                    &mut self.player_inventory,
                )
            } else {
                crate::screens::container::render_container(
                    ctx,
                    &mut self.player_inventory,
                    &mut self.container_ui_state,
                    self.hud_data.exp_level,
                )
            };
            if let crate::screens::container::ContainerAction::Close(_) = c_act {
                self.inventory_opened = false;
            }
            if c_act != crate::screens::container::ContainerAction::None {
                return InGameAction::Container(c_act);
            }
        }

        // 9. Escape / Game Menu Modal Window (toggled via Esc key)
        if self.escape_menu_opened {
            let screen_rect = ctx.screen_rect();
            ctx.layer_painter(LayerId::new(Order::Middle, Id::new("esc_backdrop")))
                .rect_filled(screen_rect, 0.0, Color32::from_black_alpha(150));

            if !self.is_modal_open {
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
        }

        action
    }
}
