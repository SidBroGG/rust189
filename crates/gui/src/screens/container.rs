use crate::hud::get_or_load_item_texture;
use crate::theme::{
    COLOR_AQUA, COLOR_BLACK, COLOR_DARK_GRAY, COLOR_GOLD, COLOR_GRAY,
    COLOR_GREEN, COLOR_RED, COLOR_WHITE, COLOR_YELLOW,
};
use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Id, Key, LayerId, Order, Pos2,
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

/// Global dragging state across all container slots.
#[derive(Clone, Default)]
pub struct ActiveDragState {
    pub is_dragging: bool,
    pub drag_button: u8, // 0 = LMB (even split), 1 = RMB (1 per slot)
    pub drag_slots: Vec<usize>,
    pub shift_drag_visited: Vec<usize>,
    pub rmb_drag_visited: Vec<usize>,
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
    pub villager_trades: Vec<world::MerchantTradeRecipe>,

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
            villager_trades: Vec::new(),
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
            ContainerKind::Player => "Inventory".to_string(),
            ContainerKind::Chest => "Chest".to_string(),
            ContainerKind::DoubleChest => "Large Chest".to_string(),
            ContainerKind::EnderChest => "Ender Chest".to_string(),
            ContainerKind::Workbench => "Crafting".to_string(),
            ContainerKind::Furnace => "Furnace".to_string(),
            ContainerKind::Anvil => "Repair & Name".to_string(),
            ContainerKind::EnchantingTable => "Enchant".to_string(),
            ContainerKind::BrewingStand => "Brewing Stand".to_string(),
            ContainerKind::Dispenser => "Dispenser".to_string(),
            ContainerKind::Dropper => "Dropper".to_string(),
            ContainerKind::Hopper => "Hopper".to_string(),
            ContainerKind::Villager => "Villager".to_string(),
            ContainerKind::Beacon => "Beacon".to_string(),
            ContainerKind::Horse => "Horse".to_string(),
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
                    let clean_title = if window_title.starts_with('{') {
                        let parsed = world::chat::parse_chat_message(&window_title);
                        world::chat::strip_formatting(&parsed)
                    } else {
                        window_title
                    };
                    ui.add_sized(
                        Vec2::new(260.0, 20.0),
                        egui::Label::new(
                            RichText::new(clean_title)
                                .size(16.0)
                                .color(COLOR_GOLD)
                                .strong(),
                        )
                        .truncate(),
                    );
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
            if carried.is_enchanted() {
                let now_ms = (ctx.input(|i| i.time) * 1000.0) as u64;
                crate::hud::item_textures::render_enchantment_glint(&painter, ctx, icon_rect, now_ms);
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

    let mut drag_state: ActiveDragState = ui.data_mut(|d| {
        d.get_temp_mut_or_insert_with(egui::Id::new("active_drag_state_v1"), ActiveDragState::default)
            .clone()
    });

    let is_lmb_down = ui.input(|i| i.pointer.primary_down());
    let is_rmb_down = ui.input(|i| i.pointer.secondary_down());
    let is_shift = ui.input(|i| i.modifiers.shift);

    // Slot background & inset border
    let bg_color = if is_hovered {
        Color32::from_rgb(46, 50, 62)
    } else {
        Color32::from_rgb(28, 30, 36)
    };
    painter.rect_filled(rect, CornerRadius::same(3), bg_color);
    painter.rect_stroke(
        rect,
        CornerRadius::same(3),
        Stroke::new(1.0_f32, Color32::from_rgb(50, 54, 66)),
        StrokeKind::Inside,
    );

    // Highlight slot border if actively dragged over
    if drag_state.is_dragging && drag_state.drag_slots.contains(&slot_idx) {
        painter.rect_stroke(
            rect,
            CornerRadius::same(3),
            Stroke::new(2.0_f32, Color32::from_rgb(255, 215, 0)),
            StrokeKind::Inside,
        );
    }

    let item_opt = container.slots.get(slot_idx).cloned().flatten();

    if let Some(ref item) = item_opt {
        let name = get_item_name(item.item_id, item.item_damage);
        let max_damage = get_item_max_damage(item.item_id);

        // 1. Draw 2D texture icon
        let icon_rect = rect.shrink(3.0);
        if let Some(tex) = get_or_load_item_texture(ui.ctx(), item.item_id, item.item_damage, max_damage, &name) {
            painter.image(tex, icon_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        }

        // Enchanted glint animation (strictly clipped to item bounds)
        if item.is_enchanted() {
            let now_ms = (ui.input(|i| i.time) * 1000.0) as u64;
            crate::hud::item_textures::render_enchantment_glint(painter, ui.ctx(), icon_rect, now_ms);
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

    // Hotbar Swap: 1..9 keys swap hovered slot with hotbar
    if is_hovered && container.carried_item.is_none() {
        let num_keys = [
            (Key::Num1, 0),
            (Key::Num2, 1),
            (Key::Num3, 2),
            (Key::Num4, 3),
            (Key::Num5, 4),
            (Key::Num6, 5),
            (Key::Num7, 6),
            (Key::Num8, 7),
            (Key::Num9, 8),
        ];
        for (k, hb_idx) in num_keys {
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, k)) {
                let clicked_item = container.slots.get(slot_idx).cloned().flatten();
                let action_num = container.next_action_number();
                container.handle_hotbar_swap(slot_idx as i16, hb_idx);
                *action = ContainerAction::Click {
                    window_id: container.window_id as i8,
                    slot: slot_idx as i16,
                    button: hb_idx as i8,
                    action_number: action_num,
                    mode: 2,
                    item: clicked_item,
                };
                break;
            }
        }
    }

    // Drop Key: Q drops 1 item, Ctrl+Q drops entire stack
    if is_hovered && ui.input(|i| i.key_pressed(Key::Q)) {
        let is_ctrl = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
        let clicked_item = container.slots.get(slot_idx).cloned().flatten();
        let action_num = container.next_action_number();
        let _ = container.handle_drop(slot_idx as i16, is_ctrl);
        *action = ContainerAction::Click {
            window_id: container.window_id as i8,
            slot: slot_idx as i16,
            button: if is_ctrl { 1 } else { 0 },
            action_number: action_num,
            mode: 4,
            item: clicked_item,
        };
    }

    // Double Click: collects all matching items into cursor stack
    if response.double_clicked() && container.carried_item.is_some() {
        let clicked_item = container.slots.get(slot_idx).cloned().flatten();
        let action_num = container.next_action_number();
        container.handle_double_click(slot_idx as i16);
        *action = ContainerAction::Click {
            window_id: container.window_id as i8,
            slot: slot_idx as i16,
            button: 0,
            action_number: action_num,
            mode: 6,
            item: clicked_item,
        };
    }

    // Mouse Tweaks: Scroll wheel on slot (Up = pull 1 item, Down = push 1 item)
    if is_hovered {
        let scroll_y = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_y.abs() > 0.5 {
            let is_up = scroll_y > 0.0;
            if container.handle_wheel_scroll(slot_idx, is_up) {
                *action = ContainerAction::Click {
                    window_id: container.window_id as i8,
                    slot: slot_idx as i16,
                    button: if is_up { 0 } else { 1 },
                    action_number: container.next_action_number(),
                    mode: 0,
                    item: container.slots.get(slot_idx).cloned().flatten(),
                };
            }
        }
    }

    let primary_released = ui.input(|i| i.pointer.primary_released());
    let secondary_released = ui.input(|i| i.pointer.secondary_released());

    // Mouse Tweaks & Vanilla Dragging
    if !is_lmb_down && !is_rmb_down {
        if drag_state.is_dragging {
            if drag_state.drag_slots.len() > 1 {
                container.handle_drag_finish(drag_state.drag_button, &drag_state.drag_slots);
                *action = ContainerAction::Click {
                    window_id: container.window_id as i8,
                    slot: -999,
                    button: if drag_state.drag_button == 0 { 2 } else { 6 },
                    action_number: container.next_action_number(),
                    mode: 5,
                    item: None,
                };
            }
            drag_state.is_dragging = false;
            drag_state.drag_slots.clear();
        }
        drag_state.shift_drag_visited.clear();
        drag_state.rmb_drag_visited.clear();
    } else if is_hovered {
        // Mouse Tweaks: Shift + LMB Drag (instant quick-move sweep across slots)
        if is_shift && is_lmb_down {
            if !drag_state.shift_drag_visited.contains(&slot_idx) {
                drag_state.shift_drag_visited.push(slot_idx);
                if container.slots.get(slot_idx).and_then(|s| s.as_ref()).is_some() {
                    let clicked_item = container.slots.get(slot_idx).cloned().flatten();
                    container.handle_shift_click(slot_idx as i16);
                    *action = ContainerAction::Click {
                        window_id: container.window_id as i8,
                        slot: slot_idx as i16,
                        button: 0,
                        action_number: container.next_action_number(),
                        mode: 1,
                        item: clicked_item,
                    };
                }
            }
        } else if container.carried_item.is_some() {
            // Vanilla LMB Drag (even split) or RMB Drag
            if is_lmb_down {
                drag_state.is_dragging = true;
                drag_state.drag_button = 0;
                if !drag_state.drag_slots.contains(&slot_idx) {
                    drag_state.drag_slots.push(slot_idx);
                }
            } else if is_rmb_down {
                drag_state.is_dragging = true;
                drag_state.drag_button = 1;
                if !drag_state.drag_slots.contains(&slot_idx) {
                    drag_state.drag_slots.push(slot_idx);
                }
            }
        }
    }

    let mut click_processed = false;

    // Release drop: if carrying an item and releasing mouse over hovered slot (and not multi-slot drag)
    if is_hovered && (primary_released || secondary_released) && container.carried_item.is_some() {
        if !drag_state.is_dragging || drag_state.drag_slots.len() <= 1 {
            let is_right = secondary_released;
            let clicked_item = container.slots.get(slot_idx).cloned().flatten();
            let action_num = container.next_action_number();
            let _ = container.handle_normal_click(slot_idx as i16, is_right);
            *action = ContainerAction::Click {
                window_id: container.window_id as i8,
                slot: slot_idx as i16,
                button: if is_right { 1 } else { 0 },
                action_number: action_num,
                mode: 0,
                item: clicked_item,
            };
            click_processed = true;
        }
    }

    // Normal Clicks (executed if not already processed on release)
    if !click_processed && (response.clicked() || response.secondary_clicked()) {
        if !drag_state.is_dragging || drag_state.drag_slots.len() <= 1 {
            let clicked_item = container.slots.get(slot_idx).cloned().flatten();
            let action_num = container.next_action_number();

            if is_shift {
                container.handle_shift_click(slot_idx as i16);
                *action = ContainerAction::Click {
                    window_id: container.window_id as i8,
                    slot: slot_idx as i16,
                    button: if response.secondary_clicked() { 1 } else { 0 },
                    action_number: action_num,
                    mode: 1,
                    item: clicked_item,
                };
            } else {
                let is_right = response.secondary_clicked();
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

    ui.data_mut(|d| d.insert_temp(egui::Id::new("active_drag_state_v1"), drag_state));
}

/// Renders rich Minecraft tooltip with custom name, enchantments, lore, durability, and stats.
fn render_item_tooltip(ui: &mut egui::Ui, slot: &Slot, name: &str, max_damage: i16) {
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 1.5);
    ui.set_max_width(220.0);

    // Title: check custom name from NBT display.Name or fallback to default item name
    let custom_name = slot
        .nbt
        .as_ref()
        .and_then(|n| n.get_compound("display"))
        .and_then(|d| d.get_string("Name"));

    let def = get_item_def(slot.item_id);
    let title_color = if slot.nbt.is_some() {
        COLOR_AQUA
    } else {
        COLOR_WHITE
    };

    if let Some(cname) = custom_name {
        if cname.contains('§') || cname.contains('&') {
            let job = crate::theme::build_layout_job_styled(cname, 13.5, false, None);
            ui.label(job);
        } else {
            ui.label(
                RichText::new(cname)
                    .size(13.5)
                    .color(title_color)
                    .italics()
                    .strong(),
            );
        }
    } else {
        ui.label(RichText::new(name).size(13.5).color(title_color).strong());
    }

    // Attack damage / Armor stats
    if let Some(d) = def {
        if d.attack_damage > 1.0 {
            ui.label(
                RichText::new(format!("When in main hand: +{:.0} Attack Damage", d.attack_damage))
                    .size(11.0)
                    .color(COLOR_GRAY),
            );
        }
        if d.armor_defense > 0 {
            ui.label(
                RichText::new(format!("+{} Armor", d.armor_defense))
                    .size(11.0)
                    .color(COLOR_GRAY),
            );
        }
    }

    // Durability count
    if max_damage > 0 {
        let left = (max_damage - slot.item_damage).max(0);
        ui.label(
            RichText::new(format!("Durability: {} / {}", left, max_damage))
                .size(11.0)
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
                        let job = crate::theme::build_layout_job_styled(s, 11.0, false, None);
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
                            .size(11.0)
                            .color(COLOR_GRAY),
                    );
                }
            }
        }
    }

    // ID tag: "#0276/0"
    ui.label(
        RichText::new(format!("#{:04}/{}", slot.item_id, slot.item_damage))
            .size(9.5)
            .color(COLOR_DARK_GRAY),
    );
}

fn get_enchantment_name(id: i16) -> &'static str {
    match id {
        0 => "Protection",
        1 => "Fire Protection",
        2 => "Feather Falling",
        3 => "Blast Protection",
        4 => "Projectile Protection",
        5 => "Respiration",
        6 => "Aqua Affinity",
        7 => "Thorns",
        8 => "Depth Strider",
        16 => "Sharpness",
        17 => "Smite",
        18 => "Bane of Arthropods",
        19 => "Knockback",
        20 => "Fire Aspect",
        21 => "Looting",
        32 => "Efficiency",
        33 => "Silk Touch",
        34 => "Unbreaking",
        35 => "Fortune",
        48 => "Power",
        49 => "Punch",
        50 => "Flame",
        51 => "Infinity",
        61 => "Luck of the Sea",
        62 => "Lure",
        _ => "Enchantment",
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
    ui.label(RichText::new("Inventory").size(13.0).color(COLOR_GRAY));
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
            ui.label(RichText::new("Crafting").size(12.0).color(COLOR_GRAY));
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
            let custom_name = item
                .nbt
                .as_ref()
                .and_then(|n| n.get_compound("display"))
                .and_then(|d| d.get_string("Name"));
            ui_state.anvil_rename_text = custom_name
                .map(|s| s.to_string())
                .unwrap_or_else(|| get_item_name(item.item_id, item.item_damage));
        }
    } else {
        ui_state.anvil_last_item_id = 0;
    }

    // Rename input box
    ui.horizontal(|ui| {
        ui.label(RichText::new("Name:").size(13.0).color(COLOR_GRAY));
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
            RichText::new(format!("Enchantment Cost: {} Levels", cost))
                .size(13.0)
                .color(col)
                .strong(),
        );
    }

    render_player_inventory_slots(ui, container, 3, action);
}

/// Renders Enchanting table with horizontal compact layout (356px width), authentic clues, and clickable choices.
fn render_enchanting_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    player_level: i32,
    action: &mut ContainerAction,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(12.0, 0.0);

        // Left sub-panel: Item slot & Lapis slot side-by-side with clear labels (84px width)
        ui.vertical(|ui| {
            ui.set_width(84.0);
            ui.label(RichText::new("Item & Lapis").size(11.0).color(COLOR_GRAY));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(6.0);
                // Slot 0 (Item)
                render_slot(ui, container, 0, action);
                // Slot 1 (Lapis)
                render_slot(ui, container, 1, action);
            });
            ui.add_space(4.0);
            let lapis_count = container
                .slots
                .get(1)
                .and_then(|s| s.as_ref())
                .map(|s| {
                    if s.item_id == 351 && s.item_damage == 4 {
                        s.item_count as i32
                    } else {
                        0
                    }
                })
                .unwrap_or(0);
            let lapis_col = if lapis_count > 0 {
                COLOR_AQUA
            } else {
                COLOR_DARK_GRAY
            };
            ui.label(
                RichText::new(format!("Lapis: {}", lapis_count))
                    .size(10.5)
                    .color(lapis_col),
            );
        });

        // Right sub-panel: 3 Enchantment rows matching full width (260px width)
        ui.vertical(|ui| {
            ui.set_width(260.0);
            ui.spacing_mut().item_spacing = Vec2::splat(4.0);

            let lapis_count = container
                .slots
                .get(1)
                .and_then(|s| s.as_ref())
                .map(|s| {
                    if s.item_id == 351 && s.item_damage == 4 {
                        s.item_count as i32
                    } else {
                        0
                    }
                })
                .unwrap_or(0);

            for i in 0..3 {
                let cost = *container.properties.get(&(i as i16)).unwrap_or(&0);
                let clue_raw = *container.properties.get(&(4 + i as i16)).unwrap_or(&-1);
                let lapis_needed = (i + 1) as i32;
                let has_cost = cost > 0;
                let can_afford =
                    has_cost && player_level >= cost as i32 && lapis_count >= lapis_needed;

                let row_size = Vec2::new(260.0, 26.0);
                let (rect, resp) = ui.allocate_exact_size(
                    row_size,
                    if has_cost {
                        egui::Sense::click()
                    } else {
                        egui::Sense::hover()
                    },
                );

                let is_hovered = resp.hovered();
                let bg_col = if has_cost && can_afford {
                    if is_hovered {
                        Color32::from_rgb(46, 68, 52)
                    } else {
                        Color32::from_rgb(34, 50, 38)
                    }
                } else if has_cost {
                    if is_hovered {
                        Color32::from_rgb(44, 48, 58)
                    } else {
                        Color32::from_rgb(32, 34, 42)
                    }
                } else {
                    Color32::from_rgb(24, 26, 32)
                };

                let stroke_col = if has_cost && can_afford {
                    Color32::from_rgb(70, 145, 85)
                } else if has_cost {
                    Color32::from_rgb(55, 60, 72)
                } else {
                    Color32::from_rgb(38, 40, 48)
                };

                let painter = ui.painter();
                painter.rect_filled(rect, CornerRadius::same(3), bg_col);
                painter.rect_stroke(
                    rect,
                    CornerRadius::same(3),
                    Stroke::new(1.0_f32, stroke_col),
                    StrokeKind::Inside,
                );

                if has_cost {
                    // Option index badge: "1", "2", "3"
                    let num_text = format!("{}", i + 1);
                    painter.text(
                        rect.left_top() + Vec2::new(8.0, 6.0),
                        Align2::LEFT_TOP,
                        &num_text,
                        FontId::proportional(12.0),
                        COLOR_GRAY,
                    );

                    // Clue or SGA runes
                    let (clue_label, clue_full) = if clue_raw >= 0 {
                        let ench_id = (clue_raw & 0xFF) as i16;
                        let ench_lvl = ((clue_raw >> 8) & 0xFF) as i32;
                        let ench_name = get_enchantment_name(ench_id);
                        let lvl_str = roman_numeral(ench_lvl);
                        (
                            format!("{} {} ... ?", ench_name, lvl_str),
                            format!("{} {}", ench_name, lvl_str),
                        )
                    } else {
                        let runes = format!(
                            "{} {} {}",
                            GALACTIC_RUNES[(i * 7) % GALACTIC_RUNES.len()],
                            GALACTIC_RUNES[(i * 11 + 3) % GALACTIC_RUNES.len()],
                            GALACTIC_RUNES[(i * 5 + 1) % GALACTIC_RUNES.len()]
                        );
                        (runes.clone(), runes)
                    };

                    let text_col = if can_afford {
                        COLOR_YELLOW
                    } else {
                        Color32::from_rgb(170, 175, 185)
                    };
                    painter.text(
                        rect.left_top() + Vec2::new(24.0, 6.0),
                        Align2::LEFT_TOP,
                        &clue_label,
                        FontId::proportional(12.0),
                        text_col,
                    );

                    // Required level badge on right: "[ 30 ]"
                    let cost_str = format!("{}", cost);
                    let cost_col = if can_afford {
                        COLOR_GREEN
                    } else {
                        COLOR_RED
                    };
                    painter.text(
                        rect.right_top() + Vec2::new(-8.0, 6.0),
                        Align2::RIGHT_TOP,
                        &cost_str,
                        FontId::proportional(12.5),
                        cost_col,
                    );

                    // Click sends Enchant action
                    if resp.clicked() {
                        *action = ContainerAction::Enchant {
                            window_id: container.window_id as i8,
                            button: i as i8,
                        };
                    }

                    // Tooltip
                    if is_hovered {
                        resp.on_hover_ui(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                            if clue_raw >= 0 {
                                ui.label(
                                    RichText::new(format!("Enchantment: {} ... ?", clue_full))
                                        .size(13.0)
                                        .color(COLOR_GOLD)
                                        .strong(),
                                );
                            }
                            ui.label(
                                RichText::new(format!("Level Requirement: {}", cost))
                                    .size(12.0)
                                    .color(COLOR_WHITE),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "Cost: {} Level{} & {} Lapis Lazuli",
                                    lapis_needed,
                                    if lapis_needed == 1 { "" } else { "s" },
                                    lapis_needed
                                ))
                                .size(11.5)
                                .color(COLOR_AQUA),
                            );
                            if can_afford {
                                ui.label(
                                    RichText::new("✔ Click to Enchant")
                                        .size(11.0)
                                        .color(COLOR_GREEN),
                                );
                            } else {
                                ui.label(
                                    RichText::new(
                                        "✘ Insufficient levels or lapis (Clicking still sends packet)",
                                    )
                                    .size(10.5)
                                    .color(COLOR_RED),
                                );
                            }
                        });
                    }
                } else {
                    // Empty row
                    painter.text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        "---",
                        FontId::proportional(12.0),
                        COLOR_DARK_GRAY,
                    );
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

/// Helper to render an item icon inside a trade offer row.
fn render_trade_item_icon(ui: &mut egui::Ui, slot: &Slot) {
    let size = Vec2::splat(26.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(2), Color32::from_rgb(20, 22, 28));
    painter.rect_stroke(rect, CornerRadius::same(2), Stroke::new(1.0_f32, Color32::from_rgb(40, 44, 54)), StrokeKind::Inside);

    let name = get_item_name(slot.item_id, slot.item_damage);
    let max_damage = get_item_max_damage(slot.item_id);
    if let Some(tex) = get_or_load_item_texture(ui.ctx(), slot.item_id, slot.item_damage, max_damage, &name) {
        let icon_rect = rect.shrink(2.0);
        painter.image(tex, icon_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
    }
    if slot.item_count > 1 {
        let count_str = format!("{}", slot.item_count);
        let text_pos = rect.max + Vec2::new(-2.0, -2.0);
        painter.text(text_pos + Vec2::new(1.0, 1.0), Align2::RIGHT_BOTTOM, &count_str, FontId::proportional(11.0), Color32::from_rgb(30, 30, 30));
        painter.text(text_pos, Align2::RIGHT_BOTTOM, &count_str, FontId::proportional(11.0), COLOR_WHITE);
    }
    if resp.hovered() {
        resp.on_hover_ui(|ui| {
            render_item_tooltip(ui, slot, &name, max_damage);
        });
    }
}

/// Renders Villager / Merchant trading interface with a right sidebar of trade offers.
fn render_villager_body(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    ui_state: &mut ContainerUiState,
    action: &mut ContainerAction,
) {
    ui.horizontal_top(|ui| {
        // Left section: Active trade + player inventory
        ui.vertical(|ui| {
            ui.label(RichText::new("Trade").size(12.0).color(COLOR_GRAY));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);
                render_slot(ui, container, 0, action); // Buy 1
                render_slot(ui, container, 1, action); // Buy 2
                ui.label(RichText::new("➔").size(20.0).color(COLOR_GOLD));
                render_slot(ui, container, 2, action); // Sell
            });

            render_player_inventory_slots(ui, container, 3, action);
        });

        // Vertical separator line
        ui.add_space(8.0);
        let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 240.0), egui::Sense::hover());
        ui.painter().rect_filled(sep_rect, CornerRadius::ZERO, Color32::from_rgb(45, 50, 60));
        ui.add_space(8.0);

        // Right section: Trade offers sidebar
        ui.vertical(|ui| {
            ui.set_width(210.0);
            ui.label(RichText::new("Villager Trades").size(13.0).color(COLOR_GOLD).strong());
            ui.add_space(4.0);

            if ui_state.villager_trades.is_empty() {
                ui.label(RichText::new("No trades available").size(12.0).color(COLOR_DARK_GRAY));
            } else {
                ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 4.0);
                        for (idx, recipe) in ui_state.villager_trades.iter().enumerate() {
                            let is_selected = ui_state.villager_trade_idx == idx;
                            let bg_color = if recipe.disabled {
                                Color32::from_rgb(38, 25, 25)
                            } else if is_selected {
                                Color32::from_rgb(45, 55, 75)
                            } else {
                                Color32::from_rgb(26, 28, 34)
                            };

                            let border_color = if is_selected {
                                Color32::from_rgb(90, 140, 220)
                            } else if recipe.disabled {
                                Color32::from_rgb(90, 35, 35)
                            } else {
                                Color32::from_rgb(46, 50, 60)
                            };

                            let frame = Frame::NONE
                                .fill(bg_color)
                                .stroke(Stroke::new(1.0_f32, border_color))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(egui::Margin::symmetric(6, 4));

                            let response = frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

                                    if let Some(ref b1) = recipe.buy1 {
                                        render_trade_item_icon(ui, b1);
                                    }
                                    if let Some(ref b2) = recipe.buy2 {
                                        ui.label(RichText::new("+").size(11.0).color(COLOR_GRAY));
                                        render_trade_item_icon(ui, b2);
                                    }
                                    let arrow_col = if recipe.disabled { COLOR_RED } else { COLOR_YELLOW };
                                    ui.label(RichText::new("➔").size(12.0).color(arrow_col));
                                    if let Some(ref s) = recipe.sell {
                                        render_trade_item_icon(ui, s);
                                    }
                                    if recipe.disabled {
                                        ui.label(RichText::new("✖").size(11.0).color(COLOR_RED));
                                    }
                                });
                            }).response;

                            let clicked = response.interact(egui::Sense::click()).clicked();
                            if clicked && !recipe.disabled {
                                ui_state.villager_trade_idx = idx;
                                let mut buf = bytes::BytesMut::new();
                                buf.extend_from_slice(&(idx as i32).to_be_bytes());
                                *action = ContainerAction::CustomPayload {
                                    channel: "MC|TrSel".into(),
                                    data: buf.to_vec(),
                                };
                            }
                        }
                    });
            }
        });
    });
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
            ui.label(RichText::new("Beacon Effects").size(13.0).color(COLOR_GOLD));
            ui.horizontal(|ui| {
                let powers = [("Speed", 1), ("Haste", 3), ("Resistance", 11), ("Jump Boost", 8), ("Strength", 5)];
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
            ui.label(RichText::new("Payment").size(11.0).color(COLOR_GRAY));
            render_slot(ui, container, 0, action);
            if ui.button(RichText::new("✔ Done").size(12.0).color(COLOR_GREEN)).clicked() {
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
            ui.label(RichText::new("Saddle").size(11.0).color(COLOR_GRAY));
            render_slot(ui, container, 0, action);
            ui.label(RichText::new("Armor").size(11.0).color(COLOR_GRAY));
            render_slot(ui, container, 1, action);
        });

        // 15 Chest slots (3 rows of 5)
        ui.vertical(|ui| {
            ui.label(RichText::new("Horse Chest").size(11.0).color(COLOR_GRAY));
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
                ui.label(RichText::new("Edit Sign Message").size(16.0).color(COLOR_BLACK).strong());
                ui.add_space(10.0);

                for (i, line) in ui_state.sign_lines.iter_mut().enumerate() {
                    let edit = TextEdit::singleline(line)
                        .desired_width(220.0)
                        .font(FontId::proportional(14.0))
                        .text_color(COLOR_BLACK)
                        .hint_text(format!("Line {}", i + 1));
                    ui.add(edit);
                }

                ui.add_space(14.0);

                if ui.button(RichText::new("Done").size(14.0).color(COLOR_WHITE)).clicked() {
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
                    ui.label(RichText::new("Enter book title:").size(15.0).color(COLOR_BLACK).strong());
                    ui.add_space(8.0);
                    ui.add(TextEdit::singleline(&mut ui_state.book_title).desired_width(240.0));
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Sign and Close").size(13.0).color(COLOR_WHITE)).clicked() {
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
                        if ui.button(RichText::new("Cancel").size(13.0).color(COLOR_GRAY)).clicked() {
                            ui_state.book_signing = false;
                        }
                    });
                } else {
                    let total_pages = ui_state.book_pages.len();
                    let cur = ui_state.book_page_idx;

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("Page {} of {}", cur + 1, total_pages)).size(12.0).color(COLOR_DARK_GRAY));
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
                        if ui.button(RichText::new("◀ Prev").size(12.0)).clicked() && cur > 0 {
                            ui_state.book_page_idx -= 1;
                        }
                        if ui.button(RichText::new("Next ▶").size(12.0)).clicked() {
                            if cur + 1 < total_pages {
                                ui_state.book_page_idx += 1;
                            } else if total_pages < 50 {
                                ui_state.book_pages.push(String::new());
                                ui_state.book_page_idx += 1;
                            }
                        }

                        ui.add_space(16.0);

                        if ui.button(RichText::new("Sign").size(12.0)).clicked() {
                            ui_state.book_signing = true;
                        }
                        if ui.button(RichText::new("Done").size(12.0)).clicked() {
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

/// Renders a hotbar slot in creative inventory mode and updates both client state and emits CreativeSetSlot.
fn render_creative_hotbar_slot(
    ui: &mut egui::Ui,
    container: &mut ContainerState,
    slot_idx: usize,
    action: &mut ContainerAction,
) {
    let slot_size = Vec2::splat(36.0);
    let (rect, mut response) = ui.allocate_exact_size(slot_size, egui::Sense::click());
    let painter = ui.painter();
    let is_hovered = response.hovered();

    let bg_color = if is_hovered {
        Color32::from_rgb(46, 50, 62)
    } else {
        Color32::from_rgb(28, 30, 36)
    };
    painter.rect_filled(rect, CornerRadius::same(3), bg_color);
    painter.rect_stroke(
        rect,
        CornerRadius::same(3),
        Stroke::new(1.0_f32, Color32::from_rgb(50, 54, 66)),
        StrokeKind::Inside,
    );

    let item_opt = container.slots.get(slot_idx).cloned().flatten();
    if let Some(ref item) = item_opt {
        let name = get_item_name(item.item_id, item.item_damage);
        let max_damage = get_item_max_damage(item.item_id);
        if let Some(tex) = get_or_load_item_texture(
            ui.ctx(),
            item.item_id,
            item.item_damage,
            max_damage,
            &name,
        ) {
            let icon_rect = rect.shrink(3.0);
            painter.image(
                tex,
                icon_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        if item.item_count > 1 {
            let count_str = format!("{}", item.item_count);
            let text_pos = rect.max + Vec2::new(-2.0, -2.0);
            painter.text(
                text_pos + Vec2::new(1.0, 1.0),
                Align2::RIGHT_BOTTOM,
                &count_str,
                FontId::proportional(13.0),
                Color32::from_rgb(30, 30, 30),
            );
            painter.text(
                text_pos,
                Align2::RIGHT_BOTTOM,
                &count_str,
                FontId::proportional(13.0),
                COLOR_WHITE,
            );
        }
        if is_hovered {
            response = response.on_hover_ui(|ui| {
                render_item_tooltip(ui, item, &name, max_damage);
            });
        }
    }

    // Hotbar Swap: 1..9 keys swap hovered hotbar slot with target hotbar slot
    let num_keys = [
        (Key::Num1, 0),
        (Key::Num2, 1),
        (Key::Num3, 2),
        (Key::Num4, 3),
        (Key::Num5, 4),
        (Key::Num6, 5),
        (Key::Num7, 6),
        (Key::Num8, 7),
        (Key::Num9, 8),
    ];
    for (k, hb_idx) in num_keys {
        if is_hovered
            && container.carried_item.is_none()
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, k))
        {
            let target_slot = 36 + hb_idx;
            if slot_idx != target_slot {
                let a = container.slots[slot_idx].take();
                let b = container.slots[target_slot].take();
                container.slots[slot_idx] = b;
                container.slots[target_slot] = a.clone();
                *action = ContainerAction::CreativeSetSlot {
                    slot: target_slot as i16,
                    item: a,
                };
            }
            break;
        }
    }

    let is_shift = ui.input(|i| i.modifiers.shift);
    let is_left = response.clicked_by(egui::PointerButton::Primary);
    let is_right = response.clicked_by(egui::PointerButton::Secondary);

    if is_left || is_right {
        if is_shift {
            container.slots[slot_idx] = None;
            *action = ContainerAction::CreativeSetSlot {
                slot: slot_idx as i16,
                item: None,
            };
        } else if let Some(carried) = container.carried_item.take() {
            if is_left {
                if let Some(existing) = container.slots[slot_idx].take() {
                    if existing.item_id == carried.item_id
                        && existing.item_damage == carried.item_damage
                    {
                        let max_st = get_item_max_stack(carried.item_id) as i8;
                        let space = (max_st - existing.item_count).max(0);
                        let transfer = carried.item_count.min(space);
                        let mut new_slot = existing;
                        new_slot.item_count += transfer;
                        let remaining = carried.item_count - transfer;
                        container.slots[slot_idx] = Some(new_slot.clone());
                        if remaining > 0 {
                            let mut leftover = carried;
                            leftover.item_count = remaining;
                            container.carried_item = Some(leftover);
                        }
                        *action = ContainerAction::CreativeSetSlot {
                            slot: slot_idx as i16,
                            item: Some(new_slot),
                        };
                    } else {
                        container.slots[slot_idx] = Some(carried);
                        container.carried_item = Some(existing);
                        *action = ContainerAction::CreativeSetSlot {
                            slot: slot_idx as i16,
                            item: container.slots[slot_idx].clone(),
                        };
                    }
                } else {
                    container.slots[slot_idx] = Some(carried);
                    *action = ContainerAction::CreativeSetSlot {
                        slot: slot_idx as i16,
                        item: container.slots[slot_idx].clone(),
                    };
                }
            } else {
                let max_st = get_item_max_stack(carried.item_id) as i8;
                if let Some(mut existing) = container.slots[slot_idx].take() {
                    if existing.item_id == carried.item_id
                        && existing.item_damage == carried.item_damage
                        && existing.item_count < max_st
                    {
                        existing.item_count += 1;
                        container.slots[slot_idx] = Some(existing.clone());
                        let rem = carried.item_count - 1;
                        if rem > 0 {
                            let mut left = carried;
                            left.item_count = rem;
                            container.carried_item = Some(left);
                        }
                        *action = ContainerAction::CreativeSetSlot {
                            slot: slot_idx as i16,
                            item: Some(existing),
                        };
                    } else {
                        container.slots[slot_idx] = Some(existing);
                        container.carried_item = Some(carried);
                    }
                } else {
                    let mut placed = carried.clone();
                    placed.item_count = 1;
                    container.slots[slot_idx] = Some(placed.clone());
                    let rem = carried.item_count - 1;
                    if rem > 0 {
                        let mut left = carried;
                        left.item_count = rem;
                        container.carried_item = Some(left);
                    }
                    *action = ContainerAction::CreativeSetSlot {
                        slot: slot_idx as i16,
                        item: Some(placed),
                    };
                }
            }
        } else if let Some(existing) = container.slots[slot_idx].take() {
            if is_left {
                container.carried_item = Some(existing);
                *action = ContainerAction::CreativeSetSlot {
                    slot: slot_idx as i16,
                    item: None,
                };
            } else {
                let half = (existing.item_count + 1) / 2;
                let rem = existing.item_count - half;
                let mut picked = existing.clone();
                picked.item_count = half;
                container.carried_item = Some(picked);
                if rem > 0 {
                    let mut left = existing;
                    left.item_count = rem;
                    container.slots[slot_idx] = Some(left.clone());
                    *action = ContainerAction::CreativeSetSlot {
                        slot: slot_idx as i16,
                        item: Some(left),
                    };
                } else {
                    *action = ContainerAction::CreativeSetSlot {
                        slot: slot_idx as i16,
                        item: None,
                    };
                }
            }
        }
    }
}

/// Renders Creative Inventory screen (2 Rows of Tabs, 9-column Item Grid, Search, Snug Width, Survival Hotbar).
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
        .default_width(386.0)
        .max_width(386.0)
        .frame(frame)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Creative tabs in 2 rows of 5 buttons
                let row1 = [
                    (CreativeTab::BuildingBlocks, "Building Blocks"),
                    (CreativeTab::DecorationBlocks, "Decoration"),
                    (CreativeTab::Redstone, "Redstone"),
                    (CreativeTab::Transportation, "Transportation"),
                    (CreativeTab::Miscellaneous, "Miscellaneous"),
                ];
                let row2 = [
                    (CreativeTab::Food, "Foodstuffs"),
                    (CreativeTab::Tools, "Tools"),
                    (CreativeTab::Combat, "Combat"),
                    (CreativeTab::Brewing, "Brewing"),
                    (CreativeTab::Materials, "Materials"),
                ];

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(3.0);
                    for (tab, label) in row1 {
                        let is_active = ui_state.creative_tab == tab;
                        let btn = egui::Button::new(
                            RichText::new(label)
                                .size(11.0)
                                .color(if is_active { COLOR_YELLOW } else { COLOR_WHITE }),
                        )
                        .fill(if is_active {
                            Color32::from_rgb(50, 56, 70)
                        } else {
                            Color32::from_rgb(30, 32, 40)
                        })
                        .min_size(Vec2::new(69.0, 22.0));

                        if ui.add(btn).clicked() {
                            ui_state.creative_tab = tab;
                        }
                    }
                });

                ui.add_space(3.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(3.0);
                    for (tab, label) in row2 {
                        let is_active = ui_state.creative_tab == tab;
                        let btn = egui::Button::new(
                            RichText::new(label)
                                .size(11.0)
                                .color(if is_active { COLOR_YELLOW } else { COLOR_WHITE }),
                        )
                        .fill(if is_active {
                            Color32::from_rgb(50, 56, 70)
                        } else {
                            Color32::from_rgb(30, 32, 40)
                        })
                        .min_size(Vec2::new(69.0, 22.0));

                        if ui.add(btn).clicked() {
                            ui_state.creative_tab = tab;
                        }
                    }
                });

                ui.add_space(8.0);

                // Search field and trash can
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
                    ui.label(RichText::new("🔍").size(14.0));
                    let edit = TextEdit::singleline(&mut ui_state.creative_search)
                        .desired_width(290.0)
                        .hint_text("Search Items...");
                    ui.add(edit);

                    // Trash Can button
                    let (t_rect, t_resp) =
                        ui.allocate_exact_size(Vec2::splat(26.0), egui::Sense::click());
                    let painter = ui.painter();
                    let is_t_hov = t_resp.hovered();
                    let t_bg = if is_t_hov {
                        Color32::from_rgb(85, 30, 30)
                    } else {
                        Color32::from_rgb(55, 22, 22)
                    };
                    painter.rect_filled(t_rect, CornerRadius::same(3), t_bg);
                    painter.rect_stroke(
                        t_rect,
                        CornerRadius::same(3),
                        Stroke::new(1.0_f32, Color32::from_rgb(100, 40, 40)),
                        StrokeKind::Inside,
                    );
                    painter.text(
                        t_rect.center(),
                        Align2::CENTER_CENTER,
                        "🗑",
                        FontId::proportional(14.0),
                        COLOR_WHITE,
                    );
                    let is_t_clicked = t_resp.clicked();
                    if is_t_hov {
                        t_resp.on_hover_text("Delete item on cursor (Shift: clear hotbar)");
                    }
                    if is_t_clicked {
                        if ui.input(|i| i.modifiers.shift) {
                            container.carried_item = None;
                            for s in 36..=44 {
                                if container.slots[s].is_some() {
                                    container.slots[s] = None;
                                    action = ContainerAction::CreativeSetSlot {
                                        slot: s as i16,
                                        item: None,
                                    };
                                }
                            }
                        } else {
                            container.carried_item = None;
                        }
                    }
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
                                    let (rect, resp) =
                                        ui.allocate_exact_size(slot_size, egui::Sense::click());
                                    let painter = ui.painter();
                                    let is_hovered = resp.hovered();

                                    let bg_col = if is_hovered {
                                        Color32::from_rgb(46, 50, 62)
                                    } else {
                                        Color32::from_rgb(28, 30, 36)
                                    };
                                    painter.rect_filled(rect, CornerRadius::same(3), bg_col);
                                    painter.rect_stroke(
                                        rect,
                                        CornerRadius::same(3),
                                        Stroke::new(1.0_f32, Color32::from_rgb(50, 54, 66)),
                                        StrokeKind::Inside,
                                    );

                                    let name = get_item_name(item_id, damage);
                                    let max_damage = get_item_max_damage(item_id);
                                    if let Some(tex) = get_or_load_item_texture(
                                        ui.ctx(),
                                        item_id,
                                        damage,
                                        max_damage,
                                        &name,
                                    ) {
                                        let icon_rect = rect.shrink(3.0);
                                        painter.image(
                                            tex,
                                            icon_rect,
                                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                            Color32::WHITE,
                                        );
                                    }

                                    let is_item_clicked = resp.clicked();
                                    if is_hovered {
                                        resp.on_hover_ui(|ui| {
                                            let temp_slot = Slot::new(item_id, 1, damage, None);
                                            render_item_tooltip(ui, &temp_slot, &name, max_damage);
                                        });
                                    }

                                    let num_keys = [
                                        (Key::Num1, 0),
                                        (Key::Num2, 1),
                                        (Key::Num3, 2),
                                        (Key::Num4, 3),
                                        (Key::Num5, 4),
                                        (Key::Num6, 5),
                                        (Key::Num7, 6),
                                        (Key::Num8, 7),
                                        (Key::Num9, 8),
                                    ];
                                    for (k, hb_idx) in num_keys {
                                        if is_hovered
                                            && container.carried_item.is_none()
                                            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, k))
                                        {
                                            let max_stack = get_item_max_stack(item_id) as i8;
                                            let target_slot = 36 + hb_idx;
                                            let new_slot = Slot::new(item_id, max_stack, damage, None);
                                            container.slots[target_slot] = Some(new_slot.clone());
                                            action = ContainerAction::CreativeSetSlot {
                                                slot: target_slot as i16,
                                                item: Some(new_slot),
                                            };
                                            break;
                                        }
                                    }

                                    if is_item_clicked {
                                        let stack_size = get_item_max_stack(item_id) as i8;
                                        if ui.input(|i| i.modifiers.shift) {
                                            let mut target_slot = None;
                                            for s in 36..=44 {
                                                if container.slots[s].is_none() {
                                                    target_slot = Some(s);
                                                    break;
                                                }
                                            }
                                            let slot_idx = target_slot.unwrap_or(36);
                                            let new_slot =
                                                Slot::new(item_id, stack_size, damage, None);
                                            container.slots[slot_idx] = Some(new_slot.clone());
                                            action = ContainerAction::CreativeSetSlot {
                                                slot: slot_idx as i16,
                                                item: Some(new_slot),
                                            };
                                        } else {
                                            container.carried_item =
                                                Some(Slot::new(item_id, stack_size, damage, None));
                                        }
                                    }
                                }
                            });
                        }
                    });

                // Hotbar section (slots 36..45)
                ui.add_space(8.0);
                ui.label(RichText::new("Hotbar").size(12.0).color(COLOR_GRAY));
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(4.0);
                    for col in 0..9 {
                        let idx = 36 + col;
                        render_creative_hotbar_slot(ui, container, idx, &mut action);
                    }
                });
            });
        });

    // Render carried item floating on cursor
    if let Some(ref carried) = container.carried_item {
        let painter = ctx.layer_painter(LayerId::new(
            Order::Tooltip,
            Id::new("carried_item_creative_layer"),
        ));
        if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
            let icon_rect = Rect::from_min_size(pos + Vec2::new(-16.0, -16.0), Vec2::splat(32.0));
            let name = get_item_name(carried.item_id, carried.item_damage);
            let max_damage = get_item_max_damage(carried.item_id);
            if let Some(tex) = get_or_load_item_texture(
                ctx,
                carried.item_id,
                carried.item_damage,
                max_damage,
                &name,
            ) {
                painter.image(
                    tex,
                    icon_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            if carried.item_count > 1 {
                let count_str = format!("{}", carried.item_count);
                let text_pos = icon_rect.max + Vec2::new(-2.0, -2.0);
                painter.text(
                    text_pos + Vec2::new(1.0, 1.0),
                    Align2::RIGHT_BOTTOM,
                    &count_str,
                    FontId::proportional(14.0),
                    Color32::from_rgb(30, 30, 30),
                );
                painter.text(
                    text_pos,
                    Align2::RIGHT_BOTTOM,
                    &count_str,
                    FontId::proportional(14.0),
                    COLOR_WHITE,
                );
            }
        }
    }

    action
}

