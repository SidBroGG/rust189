use crate::hud::get_or_load_item_texture;
use crate::theme::{
    parse_minecraft_text, COLOR_AQUA, COLOR_BLACK, COLOR_DARK_GRAY, COLOR_GOLD, COLOR_GRAY,
    COLOR_GREEN, COLOR_RED, COLOR_WHITE, COLOR_YELLOW,
};
use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Id, Key, LayerId, Layout, Order, Pos2,
    Rect, RichText, ScrollArea, Stroke, StrokeKind, TextEdit, Vec2, Window,
};
use protocol::{Position, Slot};
use world::{
    get_creative_tab_items, get_item_def, get_item_max_damage, get_item_max_stack,
    get_item_name, search_items, ContainerKind, ContainerState, CreativeTab,
};

/// High-level action produced by container UI interactions to be sent to the network.
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerAction {
    None,
    Click {
        window_id: i8,
        slot: i16,
        button: i8,
        action_number: i16,
        mode: i8,
        item: Option<Slot>,
    },
    Close(i8),
    CreativeSetSlot {
        slot: i16,
        item: Option<Slot>,
    },
    Enchant {
        window_id: i8,
        button: i8,
    },
    UpdateSign {
        position: Position,
        lines: [String; 4],
    },
    CustomPayload {
        channel: String,
        data: Vec<u8>,
    },
}

/// Extra state for interactive screens (Anvil, Creative tabs, Sign, Book, Beacon, Villager).
#[derive(Debug, Clone)]
pub struct ContainerUiState {
    pub creative_tab: CreativeTab,
    pub creative_search: String,
    pub anvil_rename_text: String,
    pub anvil_last_item_id: i16,

    // Sign editor
    pub sign_pos: Option<Position>,
    pub sign_lines: [String; 4],

    // Book & Quill
    pub book_open: bool,
    pub book_pages: Vec<String>,
    pub book_page_idx: usize,
    pub book_title: String,
    pub book_signing: bool,

    // Beacon
    pub beacon_primary: i32,
    pub beacon_secondary: i32,

    // Villager
    pub villager_trade_idx: usize,

    // Drag & drop state
    pub drag_active: bool,
    pub drag_mode: u8, // 0 = split evenly, 1 = one per slot
    pub drag_slots: Vec<i16>,
}

impl Default for ContainerUiState {
    fn default() -> Self {
        Self {
            creative_tab: CreativeTab::BuildingBlocks,
            creative_search: String::new(),
            anvil_rename_text: String::new(),
            anvil_last_item_id: 0,
            sign_pos: None,
            sign_lines: [String::new(), String::new(), String::new(), String::new()],
            book_open: false,
            book_pages: vec![String::new()],
            book_page_idx: 0,
            book_title: String::new(),
            book_signing: false,
            beacon_primary: 0,
            beacon_secondary: 0,
            villager_trade_idx: 0,
            drag_active: false,
            drag_mode: 0,
            drag_slots: Vec::new(),
        }
    }
}

/// Standard Galactic Alphabet runes for Enchanting table.
const GALACTIC_RUNES: &[&str] = &[
    "ᔑ", "ʖ", "ᓵ", "↸", "ᒷ", "⎓", "⊣", "⍑", "╎", "⋮", "ꖌ", "ꖎ", "ᒲ", "リ", "𝙹", "!", "¡",
    "ᑑ", "∷", "ᓭ", "ℸ", "⚍", "⍊", "∴", "·", "⊣", "||",
];

fn roman_numeral(level: i32) -> &'static str {
    match level {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        6 => "VI",
        7 => "VII",
        8 => "VIII",
        9 => "IX",
        10 => "X",
        _ => "",
    }
}

/// Main container rendering function.
pub fn render_container(
    ctx: &egui::Context,
    container: &mut ContainerState,
    ui_state: &mut ContainerUiState,
    player_level: i32,
) -> ContainerAction {
    let mut action = ContainerAction::None;

    // Check Sign editor
    if ui_state.sign_pos.is_some() {
        return render_sign_editor(ctx, ui_state);
    }

    // Check Book & Quill
    if ui_state.book_open {
        return render_book_screen(ctx, ui_state);
    }

    // If Escape key pressed, close window
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        let win_id = container.window_id as i8;
        action = ContainerAction::Close(win_id);
        return action;
    }

    // Modern Minecraft dark translucent modal styling
    let frame = Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(22, 24, 30, 248))
        .stroke(Stroke::new(2.0_f32, Color32::from_rgb(58, 64, 78)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(14, 12));

    let window_title = if container.title.is_empty() {
        match container.kind {
            ContainerKind::Player => "Инвентарь".to_string(),
            ContainerKind::Chest => "Одинарный сундук".to_string(),
            ContainerKind::DoubleChest => "Двойной сундук".to_string(),
            ContainerKind::EnderChest => "Сундук Края".to_string(),
            ContainerKind::Workbench => "Верстак".to_string(),
            ContainerKind::Furnace => "Печь".to_string(),
            ContainerKind::Anvil => "Наковальня".to_string(),
            ContainerKind::EnchantingTable => "Зачаровывание".to_string(),
            ContainerKind::BrewingStand => "Зельеварение".to_string(),
            ContainerKind::Dispenser => "Раздатчик".to_string(),
            ContainerKind::Dropper => "Выбрасыватель".to_string(),
            ContainerKind::Hopper => "Воронка".to_string(),
            ContainerKind::Villager => "Торговля".to_string(),
            ContainerKind::Beacon => "Маяк".to_string(),
            ContainerKind::Horse => "Инвентарь лошади".to_string(),
        }
    } else {
        container.title.clone()
    };

    let title_id = format!("container_modal_{}", container.window_id);

    Window::new(title_id)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(frame)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Header bar
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(window_title)
                            .size(16.0)
                            .color(COLOR_GOLD)
                            .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(RichText::new("✕").size(14.0).color(COLOR_GRAY)).clicked() {
                            action = ContainerAction::Close(container.window_id as i8);
                        }
                    });
                });
                ui.add_space(8.0);

                // Container body
                match container.kind {
                    ContainerKind::Player => {
                        render_player_inventory_body(ui, container, &mut action);
                    }
                    ContainerKind::Chest | ContainerKind::EnderChest => {
                        render_chest_body(ui, container, 3, &mut action);
                    }
                    ContainerKind::DoubleChest => {
                        render_chest_body(ui, container, 6, &mut action);
                    }
                    ContainerKind::Workbench => {
                        render_workbench_body(ui, container, &mut action);
                    }
                    ContainerKind::Furnace => {
                        render_furnace_body(ui, container, &mut action);
                    }
                    ContainerKind::Anvil => {
                        render_anvil_body(ui, container, ui_state, player_level, &mut action);
                    }
                    ContainerKind::EnchantingTable => {
                        render_enchanting_body(ui, container, player_level, &mut action);
                    }
                    ContainerKind::BrewingStand => {
                        render_brewing_body(ui, container, &mut action);
                    }
                    ContainerKind::Dispenser | ContainerKind::Dropper => {
                        render_dispenser_body(ui, container, &mut action);
                    }
                    ContainerKind::Hopper => {
                        render_hopper_body(ui, container, &mut action);
                    }
                    ContainerKind::Villager => {
                        render_villager_body(ui, container, ui_state, &mut action);
                    }
                    ContainerKind::Beacon => {
                        render_beacon_body(ui, container, ui_state, &mut action);
                    }
                    ContainerKind::Horse => {
                        render_horse_body(ui, container, &mut action);
                    }
                }
            });
        });

    // Render carried item floating on cursor
    if let Some(ref carried) = container.carried_item {
        let painter = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("carried_item_layer")));
        if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
            let icon_rect = Rect::from_min_size(pos + Vec2::new(-16.0, -16.0), Vec2::splat(32.0));
            let name = get_item_name(carried.item_id, carried.item_damage);
            let max_damage = get_item_max_damage(carried.item_id);
            if let Some(tex) = get_or_load_item_texture(ctx, carried.item_id, carried.item_damage, max_damage, &name) {
                painter.image(tex, icon_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
            }
            if carried.item_count > 1 {
                let count_str = format!("{}", carried.item_count);
                let text_pos = icon_rect.max + Vec2::new(-2.0, -2.0);
                // Shadow
                painter.text(text_pos + Vec2::new(1.0, 1.0), Align2::RIGHT_BOTTOM, &count_str, FontId::proportional(14.0), Color32::from_rgb(30, 30, 30));
                painter.text(text_pos, Align2::RIGHT_BOTTOM, &count_str, FontId::proportional(14.0), COLOR_WHITE);
            }
        }
    }

    action
}

/// Renders a single slot with item icon, count badge, durability bar, and hover tooltip.
pub fn render_slot(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    slot_idx: usize,
    action: &mut ContainerAction,
) {
    let slot_size = Vec2::splat(36.0);
    let (rect, mut response) = ui.allocate_exact_size(slot_size, egui::Sense::click());
    let painter = ui.painter();

    let is_hovered = response.hovered();

    // Slot background & inset border
    let bg_color = if is_hovered {
        Color32::from_rgb(46, 50, 62)
    } else {
        Color32::from_rgb(28, 30, 36)
    };
    painter.rect_filled(rect, CornerRadius::same(3), bg_color);
    painter.rect_stroke(rect, CornerRadius::same(3), Stroke::new(1.0_f32, Color32::from_rgb(50, 54, 66)), StrokeKind::Inside);

    let item_opt = container.slots.get(slot_idx).cloned().flatten();

    if let Some(ref item) = item_opt {
        let name = get_item_name(item.item_id, item.item_damage);
        let max_damage = get_item_max_damage(item.item_id);

        // 1. Draw 2D texture icon
        if let Some(tex) = get_or_load_item_texture(ui.ctx(), item.item_id, item.item_damage, max_damage, &name) {
            let icon_rect = rect.shrink(3.0);
            painter.image(tex, icon_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        }

        // 2. Draw Stack Count badge (if count > 1)
        if item.item_count > 1 {
            let count_str = format!("{}", item.item_count);
            let text_pos = rect.max - Vec2::new(3.0, 3.0);
            // Drop shadow
            painter.text(text_pos + Vec2::new(1.0, 1.0), Align2::RIGHT_BOTTOM, &count_str, FontId::proportional(12.0), Color32::from_rgb(20, 20, 20));
            painter.text(text_pos, Align2::RIGHT_BOTTOM, &count_str, FontId::proportional(12.0), COLOR_WHITE);
        }

        // 3. Durability Bar (if damageable)
        if max_damage > 0 && item.item_damage > 0 {
            let left = (max_damage - item.item_damage).max(0) as f32;
            let ratio = (left / max_damage as f32).clamp(0.0, 1.0);
            let bar_w = (rect.width() - 8.0) * ratio;
            let bar_rect = Rect::from_min_size(rect.left_bottom() + Vec2::new(4.0, -5.0), Vec2::new(bar_w, 2.0));
            let bg_bar_rect = Rect::from_min_size(rect.left_bottom() + Vec2::new(4.0, -5.0), Vec2::new(rect.width() - 8.0, 2.0));

            let bar_col = if ratio > 0.5 {
                COLOR_GREEN
            } else if ratio > 0.25 {
                COLOR_YELLOW
            } else {
                COLOR_RED
            };

            painter.rect_filled(bg_bar_rect, CornerRadius::ZERO, Color32::BLACK);
            painter.rect_filled(bar_rect, CornerRadius::ZERO, bar_col);
        }

        // 4. Hover Tooltip (Name, Lore, Stats, Damage)
        if is_hovered && container.carried_item.is_none() {
            response = response.on_hover_ui(|ui| {
                render_item_tooltip(ui, item, &name, max_damage);
            });
        }
    }

    // Handle clicks
    if response.clicked() || response.secondary_clicked() {
        let is_shift = ui.input(|i| i.modifiers.shift);
        let is_right = response.secondary_clicked();

        let clicked_item = container.slots.get(slot_idx).cloned().flatten();
        let action_num = container.next_action_number();

        if is_shift {
            // Mode 1: Shift click
            container.handle_shift_click(slot_idx as i16);
            *action = ContainerAction::Click {
                window_id: container.window_id as i8,
                slot: slot_idx as i16,
                button: if is_right { 1 } else { 0 },
                action_number: action_num,
                mode: 1,
                item: clicked_item,
            };
        } else {
            // Mode 0: Normal click
            let _ = container.handle_normal_click(slot_idx as i16, is_right);
            *action = ContainerAction::Click {
                window_id: container.window_id as i8,
                slot: slot_idx as i16,
                button: if is_right { 1 } else { 0 },
                action_number: action_num,
                mode: 0,
                item: clicked_item,
            };
        }
    }
}

/// Renders rich Minecraft tooltip with name, enchantments, lore, durability, and stats.
fn render_item_tooltip(ui: &mut egui::Ui, slot: &Slot, name: &str, max_damage: i16) {
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);

    // Title: colored by rarity
    let def = get_item_def(slot.item_id);
    let title_color = if slot.nbt.is_some() {
        COLOR_AQUA
    } else {
        COLOR_WHITE
    };

    ui.label(RichText::new(name).size(15.0).color(title_color).strong());

    // Attack damage / Armor stats
    if let Some(d) = def {
        if d.attack_damage > 1.0 {
            ui.label(
                RichText::new(format!("В руке: +{:.0} урона", d.attack_damage))
                    .size(12.0)
                    .color(COLOR_GRAY),
            );
        }
        if d.armor_defense > 0 {
            ui.label(
                RichText::new(format!("Броня: +{} защиты", d.armor_defense))
                    .size(12.0)
                    .color(COLOR_GRAY),
            );
        }
    }

    // Durability count
    if max_damage > 0 {
        let left = (max_damage - slot.item_damage).max(0);
        ui.label(
            RichText::new(format!("Прочность: {} / {}", left, max_damage))
                .size(12.0)
                .color(COLOR_GRAY),
        );
    }

    // NBT display.Lore parsing
    if let Some(ref nbt) = slot.nbt {
        // Inspect lore lines if tag compound contains display -> Lore
        if let Some(display) = nbt.get_compound("display") {
            if let Some(lore_list) = display.get_list("Lore") {
                for tag in lore_list {
                    if let Some(s) = tag.as_string() {
                        let parsed = parse_minecraft_text(s);
                        let mut job = egui::text::LayoutJob::default();
                        for (color, text) in parsed {
                            job.append(
                                &text,
                                0.0,
                                egui::TextFormat {
                                    font_id: FontId::proportional(12.0),
                                    color,
                                    ..Default::default()
                                },
                            );
                        }
                        ui.label(job);
                    }
                }
            }
        }

        // Enchantment tags
        if let Some(ench_list) = nbt.get_list("ench") {
            for tag in ench_list {
                if let (Some(id), Some(lvl)) = (tag.get_short("id"), tag.get_short("lvl")) {
                    let ench_name = get_enchantment_name(id);
                    let lvl_str = roman_numeral(lvl as i32);
                    ui.label(
                        RichText::new(format!("{} {}", ench_name, lvl_str))
                            .size(12.0)
                            .color(COLOR_GRAY),
                    );
                }
            }
        }
    }

    // ID tag: "#0276/0"
    ui.label(
        RichText::new(format!("#{:04}/{}", slot.item_id, slot.item_damage))
            .size(10.0)
            .color(COLOR_DARK_GRAY),
    );
}

fn get_enchantment_name(id: i16) -> &'static str {
    match id {
        0 => "Защита",
        1 => "Огнеупорность",
        2 => "Невесомость",
        3 => "Взрывоустойчивость",
        4 => "Снарядостойкость",
        5 => "Подводное дыхание",
        6 => "Подводник",
        7 => "Шипы",
        8 => "Подводная ходьба",
        16 => "Острота",
        17 => "Небесная кара",
        18 => "Гибель членистоногих",
        19 => "Отдача",
        20 => "Заговор огня",
        21 => "Добыча",
        32 => "Эффективность",
        33 => "Шёлковое касание",
        34 => "Прочность",
        35 => "Удача",
        48 => "Сила",
        49 => "Опрокидывание",
        50 => "Горящая стрела",
        51 => "Бесконечность",
        61 => "Морская удача",
        62 => "Приманка",
        _ => "Зачарование",
    }
}

/// Renders player inventory bottom section (27 main slots + 9 hotbar slots).
pub fn render_player_inventory_slots(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    start_idx: usize,
    action: &mut ContainerAction,
) {
    ui.add_space(8.0);
    ui.label(RichText::new("Инвентарь").size(13.0).color(COLOR_GRAY));
    ui.add_space(4.0);

    // 27 Main inventory slots (3 rows of 9)
    for row in 0..3 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(4.0);
            for col in 0..9 {
                let idx = start_idx + row * 9 + col;
                render_slot(ui, container, idx, action);
            }
        });
    }

    ui.add_space(6.0);

    // 9 Hotbar slots (1 row of 9)
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(4.0);
        for col in 0..9 {
            let idx = start_idx + 27 + col;
            render_slot(ui, container, idx, action);
        }
    });
}

/// Renders survival player inventory (2x2 craft, armor, main, hotbar).
fn render_player_inventory_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    action: &mut ContainerAction,
) {
    // Upper section: Armor (left), Player doll (center), 2x2 Crafting (right)
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(12.0, 0.0);

        // Armor slots (5..=8: helmet, chestplate, leggings, boots)
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(4.0);
            for slot_idx in 5..=8 {
                render_slot(ui, container, slot_idx, action);
            }
        });

        // Player Preview Doll
        let doll_size = Vec2::new(80.0, 156.0);
        let (rect, _) = ui.allocate_exact_size(doll_size, egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(18, 20, 26));
        painter.rect_stroke(rect, CornerRadius::same(4), Stroke::new(1.0_f32, Color32::from_rgb(45, 50, 62)), StrokeKind::Inside);

        // Silhouette head & body
        let center = rect.center();
        painter.circle_filled(center - Vec2::new(0.0, 32.0), 18.0, Color32::from_rgb(45, 50, 62));
        painter.rect_filled(
            Rect::from_center_size(center + Vec2::new(0.0, 14.0), Vec2::new(38.0, 56.0)),
            CornerRadius::same(2),
            Color32::from_rgb(45, 50, 62),
        );

        // 2x2 Crafting Grid
        ui.vertical(|ui| {
            ui.label(RichText::new("Создание").size(12.0).color(COLOR_GRAY));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(4.0);
                // 2x2 Matrix: 1, 2 top; 3, 4 bottom
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(4.0);
                    ui.horizontal(|ui| {
                        render_slot(ui, container, 1, action);
                        render_slot(ui, container, 2, action);
                    });
                    ui.horizontal(|ui| {
                        render_slot(ui, container, 3, action);
                        render_slot(ui, container, 4, action);
                    });
                });

                // Arrow
                ui.label(RichText::new(" ➔ ").size(18.0).color(COLOR_GRAY));

                // Result slot (0)
                render_slot(ui, container, 0, action);
            });
        });
    });

    // Lower section: 36 Player slots
    render_player_inventory_slots(ui, container, 9, action);
}

/// Renders single or double chest.
fn render_chest_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    rows: usize,
    action: &mut ContainerAction,
) {
    // Chest slots
    for row in 0..rows {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(4.0);
            for col in 0..9 {
                let idx = row * 9 + col;
                render_slot(ui, container, idx, action);
            }
        });
    }

    render_player_inventory_slots(ui, container, rows * 9, action);
}

/// Renders 3x3 workbench / crafting table.
fn render_workbench_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    action: &mut ContainerAction,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(12.0, 0.0);
        // 3x3 grid (slots 1..=9)
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(4.0);
            for row in 0..3 {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(4.0);
                    for col in 0..3 {
                        let idx = 1 + row * 3 + col;
                        render_slot(ui, container, idx, action);
                    }
                });
            }
        });

        // Arrow
        ui.label(RichText::new(" ➔ ").size(22.0).color(COLOR_GRAY));

        // Result slot (0)
        ui.vertical(|ui| {
            ui.add_space(38.0);
            render_slot(ui, container, 0, action);
        });
    });

    render_player_inventory_slots(ui, container, 10, action);
}

/// Renders furnace with animated fuel flame and smelt arrow.
fn render_furnace_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    action: &mut ContainerAction,
) {
    let cook_time = *container.properties.get(&0).unwrap_or(&0) as f32;
    let total_cook = *container.properties.get(&1).unwrap_or(&200) as f32;
    let burn_time = *container.properties.get(&2).unwrap_or(&0) as f32;
    let total_burn = *container.properties.get(&3).unwrap_or(&200) as f32;

    let cook_ratio = (cook_time / total_cook.max(1.0)).clamp(0.0, 1.0);
    let burn_ratio = (burn_time / total_burn.max(1.0)).clamp(0.0, 1.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(16.0, 0.0);

        // Input & Fuel column
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(6.0);
            render_slot(ui, container, 0, action); // Input (0)

            // Flame indicator
            let flame_size = Vec2::new(36.0, 14.0);
            let (f_rect, _) = ui.allocate_exact_size(flame_size, egui::Sense::hover());
            let painter = ui.painter();
            let flame_col = if burn_ratio > 0.0 {
                COLOR_GOLD
            } else {
                Color32::from_rgb(60, 60, 60)
            };
            painter.text(f_rect.center(), Align2::CENTER_CENTER, "🔥", FontId::proportional(14.0), flame_col);

            render_slot(ui, container, 1, action); // Fuel (1)
        });

        // Progress Arrow
        ui.vertical(|ui| {
            ui.add_space(28.0);
            let arrow_size = Vec2::new(36.0, 20.0);
            let (a_rect, _) = ui.allocate_exact_size(arrow_size, egui::Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(a_rect, CornerRadius::same(2), Color32::from_rgb(35, 38, 46));
            let fill_w = a_rect.width() * cook_ratio;
            let fill_rect = Rect::from_min_size(a_rect.min, Vec2::new(fill_w, a_rect.height()));
            painter.rect_filled(fill_rect, CornerRadius::same(2), Color32::from_rgb(80, 200, 80));
            painter.text(a_rect.center(), Align2::CENTER_CENTER, "➔", FontId::proportional(16.0), COLOR_WHITE);
        });

        // Output slot (2)
        ui.vertical(|ui| {
            ui.add_space(20.0);
            render_slot(ui, container, 2, action);
        });
    });

    render_player_inventory_slots(ui, container, 3, action);
}

/// Renders Anvil with repair/rename text input and level cost.
fn render_anvil_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    ui_state: &mut ContainerUiState,
    player_level: i32,
    action: &mut ContainerAction,
) {
    // If input slot 0 changed, initialize rename text
    if let Some(ref item) = container.slots[0] {
        if item.item_id != ui_state.anvil_last_item_id {
            ui_state.anvil_last_item_id = item.item_id;
            ui_state.anvil_rename_text = get_item_name(item.item_id, item.item_damage);
        }
    } else {
        ui_state.anvil_last_item_id = 0;
    }

    // Rename input box
    ui.horizontal(|ui| {
        ui.label(RichText::new("Название:").size(13.0).color(COLOR_GRAY));
        let edit = TextEdit::singleline(&mut ui_state.anvil_rename_text)
            .desired_width(220.0);
        if ui.add(edit).changed() {
            // Send MC|ItemName packet
            let mut payload = bytes::BytesMut::new();
            let _ = protocol::BufMutExt::write_string(&mut payload, &ui_state.anvil_rename_text);
            *action = ContainerAction::CustomPayload {
                channel: "MC|ItemName".into(),
                data: payload.to_vec(),
            };
        }
    });

    ui.add_space(8.0);

    // Slots: Input 1 (0) + Input 2 (1) -> Output (2)
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(10.0, 0.0);
        render_slot(ui, container, 0, action);
        ui.label(RichText::new("+").size(16.0).color(COLOR_GRAY));
        render_slot(ui, container, 1, action);
        ui.label(RichText::new("➔").size(18.0).color(COLOR_GRAY));
        render_slot(ui, container, 2, action);
    });

    // Level Cost
    let cost = *container.properties.get(&0).unwrap_or(&0);
    if cost > 0 {
        ui.add_space(6.0);
        let col = if player_level >= cost as i32 {
            COLOR_GREEN
        } else {
            COLOR_RED
        };
        ui.label(
            RichText::new(format!("Стоимость зачарования: {} ур.", cost))
                .size(13.0)
                .color(col)
                .strong(),
        );
    }

    render_player_inventory_slots(ui, container, 3, action);
}

/// Renders Enchanting table with 3 enchantment rows and galactic runes.
fn render_enchanting_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    player_level: i32,
    action: &mut ContainerAction,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(14.0, 0.0);

        // Item slot (0) and Lapis slot (1)
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(6.0);
            ui.label(RichText::new("Предмет").size(11.0).color(COLOR_GRAY));
            render_slot(ui, container, 0, action);
            ui.label(RichText::new("Лазурит").size(11.0).color(COLOR_GRAY));
            render_slot(ui, container, 1, action);
        });

        // 3 Enchantment choices
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(4.0);
            for i in 0..3 {
                let cost = *container.properties.get(&(i as i16)).unwrap_or(&0);
                let can_afford = cost > 0 && player_level >= cost as i32;
                let bg_col = if can_afford {
                    Color32::from_rgb(38, 48, 40)
                } else {
                    Color32::from_rgb(32, 34, 40)
                };

                let runes = format!(
                    "{} {} {}",
                    GALACTIC_RUNES[(i * 7) % GALACTIC_RUNES.len()],
                    GALACTIC_RUNES[(i * 11 + 3) % GALACTIC_RUNES.len()],
                    GALACTIC_RUNES[(i * 5 + 1) % GALACTIC_RUNES.len()]
                );

                let btn_text = if cost > 0 {
                    format!("{} ... {}", runes, cost)
                } else {
                    "---".to_string()
                };

                let btn = egui::Button::new(
                    RichText::new(btn_text)
                        .size(13.0)
                        .color(if can_afford { COLOR_YELLOW } else { COLOR_DARK_GRAY }),
                )
                .fill(bg_col)
                .min_size(Vec2::new(180.0, 26.0));

                if ui.add_enabled(can_afford, btn).clicked() {
                    *action = ContainerAction::Enchant {
                        window_id: container.window_id as i8,
                        button: i as i8,
                    };
                }
            }
        });
    });

    render_player_inventory_slots(ui, container, 2, action);
}

/// Renders Brewing stand with 3 potion slots and ingredient.
fn render_brewing_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    action: &mut ContainerAction,
) {
    let brew_time = *container.properties.get(&0).unwrap_or(&0) as f32;
    let brew_ratio = (brew_time / 400.0).clamp(0.0, 1.0);

    ui.vertical_centered(|ui| {
        // Ingredient (slot 3)
        render_slot(ui, container, 3, action);

        // Brewing tube / progress
        let p_col = if brew_time > 0.0 { COLOR_YELLOW } else { COLOR_DARK_GRAY };
        ui.label(RichText::new(format!("⚗ {:.0}%", brew_ratio * 100.0)).size(12.0).color(p_col));

        // 3 Potion slots (0, 1, 2)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(8.0);
            render_slot(ui, container, 0, action);
            render_slot(ui, container, 1, action);
            render_slot(ui, container, 2, action);
        });
    });

    render_player_inventory_slots(ui, container, 4, action);
}

/// Renders Dispenser / Dropper 3x3.
fn render_dispenser_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    action: &mut ContainerAction,
) {
    for row in 0..3 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(4.0);
            for col in 0..3 {
                let idx = row * 3 + col;
                render_slot(ui, container, idx, action);
            }
        });
    }

    render_player_inventory_slots(ui, container, 9, action);
}

/// Renders Hopper 5 slots.
fn render_hopper_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    action: &mut ContainerAction,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(4.0);
        for i in 0..5 {
            render_slot(ui, container, i, action);
        }
    });

    render_player_inventory_slots(ui, container, 5, action);
}

/// Renders Villager / Merchant trading interface.
fn render_villager_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    _ui_state: &mut ContainerUiState,
    action: &mut ContainerAction,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(12.0, 0.0);
        render_slot(ui, container, 0, action); // Buy 1
        render_slot(ui, container, 1, action); // Buy 2
        ui.label(RichText::new("➔").size(18.0).color(COLOR_GRAY));
        render_slot(ui, container, 2, action); // Sell
    });

    render_player_inventory_slots(ui, container, 3, action);
}

/// Renders Beacon power selection interface.
fn render_beacon_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    ui_state: &mut ContainerUiState,
    action: &mut ContainerAction,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(16.0, 0.0);

        // Powers list
        ui.vertical(|ui| {
            ui.label(RichText::new("Эффекты маяка").size(13.0).color(COLOR_GOLD));
            ui.horizontal(|ui| {
                let powers = [("Скорость", 1), ("Спешка", 3), ("Сопротивление", 11), ("Прыгучесть", 8), ("Сила", 5)];
                for (name, id) in powers {
                    let sel = ui_state.beacon_primary == id;
                    let btn = egui::Button::new(RichText::new(name).size(11.0).color(if sel { COLOR_YELLOW } else { COLOR_WHITE }))
                        .fill(if sel { Color32::from_rgb(60, 50, 20) } else { Color32::from_rgb(35, 38, 46) });
                    if ui.add(btn).clicked() {
                        ui_state.beacon_primary = id;
                    }
                }
            });
        });

        // Payment slot (0) & Confirm button
        ui.vertical(|ui| {
            ui.label(RichText::new("Оплата").size(11.0).color(COLOR_GRAY));
            render_slot(ui, container, 0, action);
            if ui.button(RichText::new("✔ Применить").size(12.0).color(COLOR_GREEN)).clicked() {
                let mut buf = bytes::BytesMut::new();
                buf.extend_from_slice(&(ui_state.beacon_primary).to_be_bytes());
                buf.extend_from_slice(&(ui_state.beacon_secondary).to_be_bytes());
                *action = ContainerAction::CustomPayload {
                    channel: "MC|Beacon".into(),
                    data: buf.to_vec(),
                };
            }
        });
    });

    render_player_inventory_slots(ui, container, 1, action);
}

/// Renders Horse inventory (Saddle, Armor, optional Chest).
fn render_horse_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    action: &mut ContainerAction,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(14.0, 0.0);

        // Saddle & Armor
        ui.vertical(|ui| {
            ui.label(RichText::new("Седло").size(11.0).color(COLOR_GRAY));
            render_slot(ui, container, 0, action);
            ui.label(RichText::new("Броня").size(11.0).color(COLOR_GRAY));
            render_slot(ui, container, 1, action);
        });

        // 15 Chest slots (3 rows of 5)
        ui.vertical(|ui| {
            ui.label(RichText::new("Сундук лошади").size(11.0).color(COLOR_GRAY));
            for row in 0..3 {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(4.0);
                    for col in 0..5 {
                        let idx = 2 + row * 5 + col;
                        render_slot(ui, container, idx, action);
                    }
                });
            }
        });
    });

    render_player_inventory_slots(ui, container, 17, action);
}

/// Renders Sign text editing modal.
fn render_sign_editor(ctx: &egui::Context, ui_state: &mut ContainerUiState) -> ContainerAction {
    let mut action = ContainerAction::None;

    Window::new("SignEditor")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(
            Frame::NONE
                .fill(Color32::from_rgb(160, 115, 65)) // Oak wood color
                .stroke(Stroke::new(3.0_f32, Color32::from_rgb(95, 65, 35)))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(egui::Margin::symmetric(24, 20)),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Редактирование таблички").size(16.0).color(COLOR_BLACK).strong());
                ui.add_space(10.0);

                for (i, line) in ui_state.sign_lines.iter_mut().enumerate() {
                    let edit = TextEdit::singleline(line)
                        .desired_width(220.0)
                        .font(FontId::proportional(14.0))
                        .text_color(COLOR_BLACK)
                        .hint_text(format!("Строка {}", i + 1));
                    ui.add(edit);
                }

                ui.add_space(14.0);

                if ui.button(RichText::new("Готово").size(14.0).color(COLOR_WHITE)).clicked() {
                    if let Some(pos) = ui_state.sign_pos.take() {
                        action = ContainerAction::UpdateSign {
                            position: pos,
                            lines: ui_state.sign_lines.clone(),
                        };
                    }
                }
            });
        });

    action
}

/// Renders Book & Quill editing modal.
fn render_book_screen(ctx: &egui::Context, ui_state: &mut ContainerUiState) -> ContainerAction {
    let mut action = ContainerAction::None;

    Window::new("BookEditor")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(
            Frame::NONE
                .fill(Color32::from_rgb(240, 235, 215)) // Parchment color
                .stroke(Stroke::new(3.0_f32, Color32::from_rgb(110, 80, 50)))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(28, 22)),
        )
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                if ui_state.book_signing {
                    ui.label(RichText::new("Введите название книги:").size(15.0).color(COLOR_BLACK).strong());
                    ui.add_space(8.0);
                    ui.add(TextEdit::singleline(&mut ui_state.book_title).desired_width(240.0));
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Подписать и закрыть").size(13.0).color(COLOR_WHITE)).clicked() {
                            ui_state.book_open = false;
                            ui_state.book_signing = false;
                            // Send MC|BSign
                            let mut buf = bytes::BytesMut::new();
                            let mut nbt_tag = protocol::NbtTag::Compound(std::collections::HashMap::new());
                            nbt_tag.set_string("title", ui_state.book_title.clone());
                            nbt_tag.set_string("author", "Player");
                            let slot = Slot::new(387, 1, 0, Some(nbt_tag));
                            let _ = protocol::write_slot(Some(&slot), &mut buf);
                            action = ContainerAction::CustomPayload {
                                channel: "MC|BSign".into(),
                                data: buf.to_vec(),
                            };
                        }
                        if ui.button(RichText::new("Отмена").size(13.0).color(COLOR_GRAY)).clicked() {
                            ui_state.book_signing = false;
                        }
                    });
                } else {
                    let total_pages = ui_state.book_pages.len();
                    let cur = ui_state.book_page_idx;

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("Страница {} из {}", cur + 1, total_pages)).size(12.0).color(COLOR_DARK_GRAY));
                    });
                    ui.add_space(6.0);

                    // Page text area
                    if ui_state.book_pages.is_empty() {
                        ui_state.book_pages.push(String::new());
                    }
                    let edit = TextEdit::multiline(&mut ui_state.book_pages[cur])
                        .desired_rows(10)
                        .desired_width(260.0)
                        .font(FontId::proportional(13.0))
                        .text_color(COLOR_BLACK);
                    ui.add(edit);

                    ui.add_space(10.0);

                    // Navigation buttons
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("◀ Пред.").size(12.0)).clicked() && cur > 0 {
                            ui_state.book_page_idx -= 1;
                        }
                        if ui.button(RichText::new("След. ▶").size(12.0)).clicked() {
                            if cur + 1 < total_pages {
                                ui_state.book_page_idx += 1;
                            } else if total_pages < 50 {
                                ui_state.book_pages.push(String::new());
                                ui_state.book_page_idx += 1;
                            }
                        }

                        ui.add_space(16.0);

                        if ui.button(RichText::new("Подписать").size(12.0)).clicked() {
                            ui_state.book_signing = true;
                        }
                        if ui.button(RichText::new("Готово").size(12.0)).clicked() {
                            ui_state.book_open = false;
                            // Send MC|BEdit
                            let mut buf = bytes::BytesMut::new();
                            let mut nbt_tag = protocol::NbtTag::Compound(std::collections::HashMap::new());
                            let mut pages_list = Vec::new();
                            for page in &ui_state.book_pages {
                                pages_list.push(protocol::NbtTag::String(page.clone()));
                            }
                            nbt_tag.set_list("pages", pages_list);
                            let slot = Slot::new(386, 1, 0, Some(nbt_tag));
                            let _ = protocol::write_slot(Some(&slot), &mut buf);
                            action = ContainerAction::CustomPayload {
                                channel: "MC|BEdit".into(),
                                data: buf.to_vec(),
                            };
                        }
                    });
                }
            });
        });

    action
}

/// Renders Creative Inventory screen (Tabs, Item Grid, Search, Survival Hotbar).
pub fn render_creative_inventory(
    ctx: &egui::Context,
    ui_state: &mut ContainerUiState,
    container: &mut ContainerState,
) -> ContainerAction {
    let mut action = ContainerAction::None;

    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        return ContainerAction::Close(0);
    }

    let frame = Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(22, 24, 30, 248))
        .stroke(Stroke::new(2.0_f32, Color32::from_rgb(58, 64, 78)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(14, 12));

    Window::new("creative_inventory_modal")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(frame)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Creative tabs bar
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(2.0);
                    let tabs = [
                        (CreativeTab::BuildingBlocks, "Блоки"),
                        (CreativeTab::DecorationBlocks, "Декор"),
                        (CreativeTab::Redstone, "Редстоун"),
                        (CreativeTab::Transportation, "Транспорт"),
                        (CreativeTab::Miscellaneous, "Разное"),
                        (CreativeTab::Food, "Еда"),
                        (CreativeTab::Tools, "Инструменты"),
                        (CreativeTab::Combat, "Оружие"),
                        (CreativeTab::Brewing, "Зелья"),
                        (CreativeTab::Materials, "Материалы"),
                    ];

                    for (tab, label) in tabs {
                        let is_active = ui_state.creative_tab == tab;
                        let btn = egui::Button::new(
                            RichText::new(label).size(11.0).color(if is_active { COLOR_YELLOW } else { COLOR_WHITE }),
                        )
                        .fill(if is_active { Color32::from_rgb(50, 56, 70) } else { Color32::from_rgb(30, 32, 40) });

                        if ui.add(btn).clicked() {
                            ui_state.creative_tab = tab;
                        }
                    }
                });

                ui.add_space(8.0);

                // Search field
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍").size(14.0));
                    let edit = TextEdit::singleline(&mut ui_state.creative_search)
                        .desired_width(200.0)
                        .hint_text("Поиск предметов...");
                    ui.add(edit);
                });

                ui.add_space(8.0);

                // Item grid (scrollable 9 columns)
                let items = if ui_state.creative_search.trim().is_empty() {
                    get_creative_tab_items(ui_state.creative_tab)
                } else {
                    search_items(&ui_state.creative_search)
                };

                ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let chunk_size = 9;
                        for row in items.chunks(chunk_size) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::splat(4.0);
                                for &(item_id, damage) in row {
                                    let slot_size = Vec2::splat(36.0);
                                    let (rect, resp) = ui.allocate_exact_size(slot_size, egui::Sense::click());
                                    let painter = ui.painter();
                                    let is_hovered = resp.hovered();

                                    let bg_col = if is_hovered {
                                        Color32::from_rgb(46, 50, 62)
                                    } else {
                                        Color32::from_rgb(28, 30, 36)
                                    };
                                    painter.rect_filled(rect, CornerRadius::same(3), bg_col);
                                    painter.rect_stroke(rect, CornerRadius::same(3), Stroke::new(1.0_f32, Color32::from_rgb(50, 54, 66)), StrokeKind::Inside);

                                    let name = get_item_name(item_id, damage);
                                    let max_damage = get_item_max_damage(item_id);
                                    if let Some(tex) = get_or_load_item_texture(ui.ctx(), item_id, damage, max_damage, &name) {
                                        let icon_rect = rect.shrink(3.0);
                                        painter.image(tex, icon_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                                    }

                                    let resp = if is_hovered {
                                        resp.on_hover_ui(|ui| {
                                            let temp_slot = Slot::new(item_id, 1, damage, None);
                                            render_item_tooltip(ui, &temp_slot, &name, max_damage);
                                        })
                                    } else {
                                        resp
                                    };

                                    if resp.clicked() {
                                        let stack_size = get_item_max_stack(item_id) as i8;
                                        container.carried_item = Some(Slot::new(item_id, stack_size, damage, None));
                                    }
                                }
                            });
                        }
                    });

                // Hotbar section (slots 36..45)
                ui.add_space(8.0);
                ui.label(RichText::new("Панель быстрого доступа").size(12.0).color(COLOR_GRAY));
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(4.0);
                    for col in 0..9 {
                        let idx = 36 + col;
                        render_slot(ui, container, idx, &mut action);
                    }

                    ui.add_space(10.0);

                    // Trash Can slot (destroys carried item)
                    let (t_rect, t_resp) = ui.allocate_exact_size(Vec2::splat(36.0), egui::Sense::click());
                    let painter = ui.painter();
                    painter.rect_filled(t_rect, CornerRadius::same(3), Color32::from_rgb(60, 25, 25));
                    painter.rect_stroke(t_rect, CornerRadius::same(3), Stroke::new(1.0_f32, Color32::from_rgb(100, 40, 40)), StrokeKind::Inside);
                    painter.text(t_rect.center(), Align2::CENTER_CENTER, "🗑", FontId::proportional(16.0), COLOR_WHITE);
                    if t_resp.clicked() {
                        container.carried_item = None;
                    }
                    if t_resp.hovered() {
                        t_resp.on_hover_text("Удалить предмет на курсоре");
                    }
                });
            });
        });

    // Render carried item floating on cursor
    if let Some(ref carried) = container.carried_item {
        let painter = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("carried_item_creative_layer")));
        if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
            let icon_rect = Rect::from_min_size(pos + Vec2::new(-16.0, -16.0), Vec2::splat(32.0));
            let name = get_item_name(carried.item_id, carried.item_damage);
            let max_damage = get_item_max_damage(carried.item_id);
            if let Some(tex) = get_or_load_item_texture(ctx, carried.item_id, carried.item_damage, max_damage, &name) {
                painter.image(tex, icon_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
            }
            if carried.item_count > 1 {
                let count_str = format!("{}", carried.item_count);
                let text_pos = icon_rect.max + Vec2::new(-2.0, -2.0);
                painter.text(text_pos + Vec2::new(1.0, 1.0), Align2::RIGHT_BOTTOM, &count_str, FontId::proportional(14.0), Color32::from_rgb(30, 30, 30));
                painter.text(text_pos, Align2::RIGHT_BOTTOM, &count_str, FontId::proportional(14.0), COLOR_WHITE);
            }
        }
    }

    action
}

