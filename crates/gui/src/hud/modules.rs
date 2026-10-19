use super::data::{HudData, HudItemInfo};
use egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Stroke, Vec2,
};

/// Content dimensions for each HUD widget at scale 1.0.
pub fn get_module_content_size(id: &str, data: &HudData) -> Vec2 {
    match id {
        "crosshair" => Vec2::new(16.0, 16.0),
        "hotbar" => Vec2::new(186.0, 54.0),
        "vitals" => Vec2::new(186.0, 28.0),
        "exp_bar" => Vec2::new(186.0, 16.0),
        "boss_bar" => {
            let count = data.boss_bars.len();
            if count == 0 {
                Vec2::new(186.0, 20.0)
            } else {
                Vec2::new(186.0, count as f32 * 20.0)
            }
        }
        "scoreboard" => Vec2::new(140.0, 120.0),
        "chat" => Vec2::new(320.0, 140.0),
        "keystrokes" => Vec2::new(84.0, 92.0),
        "fps_ping" => Vec2::new(180.0, 48.0),
        "armor_status" => {
            let count = data.armor_items.iter().filter(|i| i.is_some()).count()
                + if data.held_item.is_some() { 1 } else { 0 };
            Vec2::new(110.0, (count.max(1) as f32 * 18.0 + 8.0).max(28.0))
        }
        "potion_effects" => {
            let count = data.potion_effects.len().max(1);
            Vec2::new(110.0, (count as f32 * 20.0 + 6.0).max(26.0))
        }
        "coords_direction" => Vec2::new(170.0, 52.0),
        _ => Vec2::new(100.0, 40.0),
    }
}

/// 1. Crosshair: 15x15 crosshair with contrast black/white lines in screen center.
pub fn render_crosshair(painter: &Painter, rect: Rect) {
    let center = rect.center();
    let arm_len = 4.5;
    let gap = 2.0;

    let arms = [
        // Top
        [Pos2::new(center.x, center.y - gap - arm_len), Pos2::new(center.x, center.y - gap)],
        // Bottom
        [Pos2::new(center.x, center.y + gap), Pos2::new(center.x, center.y + gap + arm_len)],
        // Left
        [Pos2::new(center.x - gap - arm_len, center.y), Pos2::new(center.x - gap, center.y)],
        // Right
        [Pos2::new(center.x + gap, center.y), Pos2::new(center.x + gap + arm_len, center.y)],
    ];

    // High-contrast outer stroke (black)
    for [p0, p1] in &arms {
        painter.line_segment([*p0, *p1], Stroke::new(2.5_f32, Color32::from_black_alpha(220)));
    }

    // Crisp inner fill (white)
    for [p0, p1] in &arms {
        painter.line_segment([*p0, *p1], Stroke::new(1.2_f32, Color32::WHITE));
    }

    // Center tiny dot
    painter.circle_filled(center, 0.8, Color32::WHITE);
}

fn render_item_icon(
    painter: &Painter,
    slot_rect: Rect,
    item: &HudItemInfo,
    scale: f32,
    tick_counter: u64,
) {
    let center = slot_rect.center();
    let icon_size = Vec2::splat(16.0 * scale);
    let icon_rect = Rect::from_center_size(center, icon_size);

    let ctx = painter.ctx();
    if let Some(tex_id) = crate::hud::item_textures::get_or_load_item_texture(
        ctx,
        item.item_id,
        item.damage,
        item.max_damage,
        &item.name,
    ) {
        painter.image(
            tex_id,
            icon_rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        // Fallback placeholder if asset file is missing from disk
        let bg_col = Color32::from_rgba_unmultiplied(60, 65, 80, 200);
        painter.rect_filled(icon_rect, CornerRadius::same(2), bg_col);
        let initial = item.name.chars().next().unwrap_or('?').to_string();
        painter.text(
            center,
            Align2::CENTER_CENTER,
            initial,
            FontId::monospace((8.0 * scale).max(5.0)),
            Color32::WHITE,
        );
    }

    // Enchanted Item Glint: authentic Minecraft 1.8.9 dual-layer glint strictly clipped to icon bounds
    if item.is_enchanted {
        let time_ms = tick_counter.saturating_mul(50);
        crate::hud::item_textures::render_enchantment_glint(painter, painter.ctx(), icon_rect, time_ms);
    }
}

fn render_heart_icon(
    painter: &Painter,
    pos: Pos2,
    scale: f32,
    is_full: bool,
    is_half: bool,
    is_golden: bool,
    shake: f32,
) {
    let size = Vec2::new(7.0 * scale, 7.0 * scale);
    let rect = Rect::from_min_size(Pos2::new(pos.x, pos.y + shake), size);
    let cr = CornerRadius::same(1);

    // Empty container background
    let bg_col = Color32::from_rgba_unmultiplied(35, 10, 10, 220);
    let border_col = Color32::from_rgba_unmultiplied(20, 5, 5, 240);
    painter.rect_filled(rect, cr, bg_col);
    painter.rect_stroke(rect, cr, Stroke::new((1.0 * scale).max(0.5), border_col), egui::StrokeKind::Inside);

    let (fill_col, glint_col) = if is_golden {
        (Color32::from_rgb(255, 215, 0), Color32::from_rgb(255, 248, 180))
    } else {
        (Color32::from_rgb(228, 24, 24), Color32::WHITE)
    };

    if is_full {
        painter.rect_filled(rect, cr, fill_col);
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(rect.left() + 1.2 * scale, rect.top() + 1.2 * scale), Vec2::new(1.5 * scale, 1.5 * scale)),
            CornerRadius::ZERO,
            glint_col,
        );
    } else if is_half {
        let half_w = 3.5 * scale;
        let half_rect = Rect::from_min_size(rect.min, Vec2::new(half_w, size.y));
        painter.rect_filled(half_rect, cr, fill_col);
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(rect.left() + 1.0 * scale, rect.top() + 1.2 * scale), Vec2::new(1.2 * scale, 1.5 * scale)),
            CornerRadius::ZERO,
            glint_col,
        );
    }
}

fn render_hunger_icon(
    painter: &Painter,
    pos: Pos2,
    scale: f32,
    is_full: bool,
    is_half: bool,
) {
    let size = Vec2::new(7.0 * scale, 7.0 * scale);
    let rect = Rect::from_min_size(pos, size);
    let cr = CornerRadius::same(1);

    // Empty container background
    let bg_col = Color32::from_rgba_unmultiplied(40, 22, 12, 180);
    let border_col = Color32::from_rgba_unmultiplied(20, 10, 5, 240);
    painter.rect_filled(rect, cr, bg_col);
    painter.rect_stroke(rect, cr, Stroke::new((1.0 * scale).max(0.5), border_col), egui::StrokeKind::Inside);

    let meat_col = Color32::from_rgb(188, 86, 26);
    let hi_col = Color32::from_rgb(232, 150, 61);

    if is_full {
        painter.rect_filled(rect, cr, meat_col);
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(rect.left() + 1.2 * scale, rect.top() + 1.2 * scale), Vec2::new(1.8 * scale, 1.5 * scale)),
            CornerRadius::ZERO,
            hi_col,
        );
    } else if is_half {
        let half_w = 3.5 * scale;
        let half_rect = Rect::from_min_size(Pos2::new(rect.left() + half_w, rect.top()), Vec2::new(half_w, size.y));
        painter.rect_filled(half_rect, cr, meat_col);
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(rect.left() + half_w + 0.8 * scale, rect.top() + 1.2 * scale), Vec2::new(1.2 * scale, 1.5 * scale)),
            CornerRadius::ZERO,
            hi_col,
        );
    }
}

fn render_bubble_icon(painter: &Painter, center: Pos2, scale: f32, is_full: bool, is_popping: bool) {
    let size = Vec2::new(7.0 * scale, 7.0 * scale);
    let rect = Rect::from_center_size(center, size);
    let cr = CornerRadius::same(2);

    let bg_col = Color32::from_rgba_unmultiplied(15, 45, 75, 120);
    let border_col = Color32::from_rgba_unmultiplied(40, 140, 200, 90);
    painter.rect_filled(rect, cr, bg_col);
    painter.rect_stroke(rect, cr, Stroke::new((1.0 * scale).max(0.5), border_col), egui::StrokeKind::Inside);

    if is_popping {
        let pop_col = Color32::from_rgb(120, 210, 255);
        painter.rect_stroke(rect, cr, Stroke::new((1.0 * scale).max(0.5), pop_col), egui::StrokeKind::Inside);
        painter.circle_filled(Pos2::new(center.x - 2.5 * scale, center.y - 2.5 * scale), 0.8 * scale, pop_col);
        painter.circle_filled(Pos2::new(center.x + 2.5 * scale, center.y + 2.5 * scale), 0.8 * scale, pop_col);
    } else if is_full {
        painter.rect_filled(rect, cr, Color32::from_rgba_unmultiplied(35, 125, 215, 210));
        painter.rect_stroke(
            rect,
            cr,
            Stroke::new((1.0 * scale).max(0.5), Color32::from_rgb(90, 195, 255)),
            egui::StrokeKind::Inside,
        );
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(rect.left() + 1.2 * scale, rect.top() + 1.2 * scale), Vec2::new(1.5 * scale, 1.5 * scale)),
            CornerRadius::ZERO,
            Color32::WHITE,
        );
    }
}

/// Unified Hotbar & Status: Combines continuous Hotbar slots, Experience bar, and Vitals.
pub fn render_unified_hotbar(painter: &Painter, rect: Rect, data: &HudData, tick_counter: u64) {
    if data.gamemode == 3 {
        // In Spectator mode, hotbar and player stats are not displayed
        return;
    }

    let scale = (rect.width() / 186.0).clamp(0.4, 3.0);
    let center_x = rect.center().x;

    let is_survival = data.gamemode == 0 || data.gamemode == 2;
    let show_vitals = is_survival && !data.is_creative;

    // 1. Hotbar Frame & Slots (bottom)
    let bar_w = 182.0 * scale;
    let bar_h = 22.0 * scale;
    let start_x = center_x - bar_w * 0.5;
    let start_y = rect.bottom() - bar_h - 2.0 * scale;

    let hotbar_rect = Rect::from_min_size(Pos2::new(start_x, start_y), Vec2::new(bar_w, bar_h));

    // Hotbar continuous metallic/dark background container
    painter.rect_filled(
        hotbar_rect,
        CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(18, 20, 26, 220),
    );
    // Dark outer border
    painter.rect_stroke(
        hotbar_rect,
        CornerRadius::ZERO,
        Stroke::new((1.0 * scale).max(0.5), Color32::from_rgb(40, 44, 52)),
        egui::StrokeKind::Inside,
    );

    // 9 Contiguous Slots
    for i in 0..9 {
        let slot_left = start_x + (i as f32 * 20.0 + 1.0) * scale;
        let slot_rect = Rect::from_min_size(
            Pos2::new(slot_left, start_y + 1.0 * scale),
            Vec2::new(20.0 * scale, 20.0 * scale),
        );

        // Slot vertical separator (between slots)
        if i > 0 {
            let div_x = start_x + (i as f32 * 20.0) * scale;
            painter.line_segment(
                [Pos2::new(div_x, start_y + 1.0 * scale), Pos2::new(div_x, start_y + 21.0 * scale)],
                Stroke::new((1.0 * scale).max(0.5), Color32::from_rgb(45, 48, 58)),
            );
        }

        // Inner slot inset shadows for classic 3D beveled pocket look
        let inset_left = slot_left + 1.0 * scale;
        let inset_top = start_y + 2.0 * scale;
        let inset_right = slot_left + 19.0 * scale;
        let inset_bottom = start_y + 20.0 * scale;

        let shadow_col = Color32::from_rgba_unmultiplied(10, 12, 16, 160);
        let highlight_col = Color32::from_rgba_unmultiplied(90, 95, 110, 90);
        painter.line_segment([Pos2::new(inset_left, inset_top), Pos2::new(inset_right, inset_top)], Stroke::new(1.0 * scale, shadow_col));
        painter.line_segment([Pos2::new(inset_left, inset_top), Pos2::new(inset_left, inset_bottom)], Stroke::new(1.0 * scale, shadow_col));
        painter.line_segment([Pos2::new(inset_left, inset_bottom), Pos2::new(inset_right, inset_bottom)], Stroke::new(1.0 * scale, highlight_col));
        painter.line_segment([Pos2::new(inset_right, inset_top), Pos2::new(inset_right, inset_bottom)], Stroke::new(1.0 * scale, highlight_col));

        // Render item icon & stack count if present (NO slot numbers 1..9!)
        if let Some(item) = &data.hotbar_items[i] {
            render_item_icon(painter, slot_rect, item, scale, tick_counter);

            // Stack count in bottom-right corner if count > 1
            if item.count > 1 {
                let count_str = format!("{}", item.count);
                let font_size = (7.5 * scale).clamp(5.5, 9.5);
                let font = FontId::monospace(font_size);
                let num_pos = Pos2::new(slot_rect.right() - 2.0 * scale, slot_rect.bottom() - 1.5 * scale);

                // Dark Minecraft drop shadow
                painter.text(
                    Pos2::new(num_pos.x + 1.0, num_pos.y + 1.0),
                    Align2::RIGHT_BOTTOM,
                    &count_str,
                    font.clone(),
                    Color32::from_rgb(63, 63, 63),
                );
                // Crisp white count text
                painter.text(
                    num_pos,
                    Align2::RIGHT_BOTTOM,
                    count_str,
                    font,
                    Color32::WHITE,
                );
            }

            // Durability bar at bottom of slot
            if item.max_damage > 0 {
                let ratio = item.durability_ratio();
                let bar_w = (15.0 * scale) * ratio;
                let bar_y = slot_rect.bottom() - 2.5 * scale;
                let bar_x = slot_rect.left() + 2.5 * scale;
                let bar_color = if ratio > 0.5 {
                    Color32::from_rgb(85, 255, 85)
                } else if ratio > 0.2 {
                    Color32::from_rgb(255, 170, 0)
                } else {
                    Color32::from_rgb(255, 85, 85)
                };
                painter.line_segment(
                    [Pos2::new(bar_x, bar_y), Pos2::new(bar_x + 15.0 * scale, bar_y)],
                    Stroke::new(1.5 * scale, Color32::from_rgb(10, 10, 15)),
                );
                painter.line_segment(
                    [Pos2::new(bar_x, bar_y), Pos2::new(bar_x + bar_w, bar_y)],
                    Stroke::new(1.5 * scale, bar_color),
                );
            }
        }
    }

    // Active Slot Selector Box (protruding authentic white frame)
    let sel_idx = data.selected_hotbar_slot.min(8);
    let sel_x = start_x + (sel_idx as f32 * 20.0 - 1.0) * scale;
    let sel_y = start_y - 1.0 * scale;
    let sel_rect = Rect::from_min_size(Pos2::new(sel_x, sel_y), Vec2::new(22.0 * scale, 24.0 * scale));

    painter.rect_stroke(
        sel_rect,
        CornerRadius::ZERO,
        Stroke::new((2.0 * scale).max(1.5), Color32::WHITE),
        egui::StrokeKind::Outside,
    );
    painter.rect_stroke(
        sel_rect,
        CornerRadius::ZERO,
        Stroke::new((0.8 * scale).max(0.5), Color32::from_rgb(60, 65, 75)),
        egui::StrokeKind::Inside,
    );
    painter.rect_filled(
        sel_rect,
        CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(255, 255, 255, 15),
    );

    // 2. Experience Bar (directly above hotbar)
    let show_exp = show_vitals || data.exp_level > 0 || data.exp_progress > 0.0;
    let exp_y = start_y - 6.0 * scale;
    if show_exp {
        let exp_bar_rect = Rect::from_min_size(
            Pos2::new(start_x, exp_y),
            Vec2::new(bar_w, (4.0 * scale).max(2.0)),
        );
        painter.rect_filled(exp_bar_rect, CornerRadius::same(1), Color32::from_rgb(25, 35, 20));
        painter.rect_stroke(
            exp_bar_rect,
            CornerRadius::same(1),
            Stroke::new(1.0_f32, Color32::from_rgb(10, 15, 10)),
            egui::StrokeKind::Inside,
        );

        let progress = data.exp_progress.clamp(0.0, 1.0);
        if progress > 0.0 {
            let fill_w = exp_bar_rect.width() * progress;
            let fill_rect = Rect::from_min_size(exp_bar_rect.min, Vec2::new(fill_w, exp_bar_rect.height()));
            painter.rect_filled(fill_rect, CornerRadius::same(1), Color32::from_rgb(128, 255, 32));
        }

        if data.exp_level > 0 {
            let lvl_str = format!("{}", data.exp_level);
            let lvl_center = Pos2::new(center_x, exp_y - 4.0 * scale);
            let font = FontId::monospace((9.0 * scale).max(7.0));
            for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
                painter.text(
                    Pos2::new(lvl_center.x + dx, lvl_center.y + dy),
                    Align2::CENTER_CENTER,
                    &lvl_str,
                    font.clone(),
                    Color32::BLACK,
                );
            }
            painter.text(
                lvl_center,
                Align2::CENTER_CENTER,
                lvl_str,
                font,
                Color32::from_rgb(128, 255, 32),
            );
        }
    }

    // 3. Vitals: Row 1 (Health & Food), Row 2 (Armor & Air Bubbles)
    if show_vitals {
        let vitals_bottom_y = if show_exp { exp_y - 4.0 * scale } else { start_y - 4.0 * scale };
        let row1_y = vitals_bottom_y - 8.0 * scale;
        let row2_y = vitals_bottom_y - 18.0 * scale; // Strictly 10 units above Row 1!

        // --- ROW 1 (LEFT): HEALTH (10 Hearts) ---
        let is_low_hp = data.health <= 4.0;
        let full_hearts = (data.health / 2.0).floor() as usize;
        let half_heart = (data.health % 2.0) >= 1.0;

        for i in 0..10 {
            let shake = if is_low_hp {
                let s = (tick_counter as f32 * 0.35 + i as f32 * 0.7).sin();
                s * 0.5 * scale
            } else {
                0.0
            };

            let h_pos = Pos2::new(start_x + i as f32 * 8.0 * scale, row1_y);
            let is_full = i < full_hearts;
            let is_half = i == full_hearts && half_heart;
            render_heart_icon(painter, h_pos, scale, is_full, is_half, false, shake);
        }

        // Absorption Golden Hearts Overlay (Row 1 left)
        if data.absorption > 0.0 {
            let abs_full = (data.absorption / 2.0).floor() as usize;
            let abs_half = (data.absorption % 2.0) >= 1.0;
            for i in 0..abs_full.min(10) {
                let h_pos = Pos2::new(start_x + i as f32 * 8.0 * scale, row1_y);
                render_heart_icon(painter, h_pos, scale, true, false, true, 0.0);
            }
            if abs_full < 10 && abs_half {
                let h_pos = Pos2::new(start_x + abs_full as f32 * 8.0 * scale, row1_y);
                render_heart_icon(painter, h_pos, scale, false, true, true, 0.0);
            }
        }

        // --- ROW 1 (RIGHT): FOOD / HUNGER (10 Drumsticks) ---
        let full_shanks = (data.food / 2) as usize;
        let half_shank = (data.food % 2) != 0;
        let right_end = start_x + bar_w;

        for i in 0..10 {
            let f_pos = Pos2::new(right_end - (i as f32 + 1.0) * 8.0 * scale, row1_y);
            let is_full = i < full_shanks;
            let is_half = i == full_shanks && half_shank;
            render_hunger_icon(painter, f_pos, scale, is_full, is_half);
        }

        // --- ROW 2 (LEFT): ARMOR (10 Shields, above Health) ---
        if data.armor_value > 0 {
            let full_shields = (data.armor_value / 2) as usize;
            let half_shield = (data.armor_value % 2) != 0;

            for i in 0..10 {
                let x = start_x + i as f32 * 8.0 * scale;
                let shield_rect = Rect::from_min_size(Pos2::new(x, row2_y), Vec2::new(7.0 * scale, 7.0 * scale));
                let col = if i < full_shields {
                    Color32::from_rgb(180, 185, 200)
                } else if i == full_shields && half_shield {
                    Color32::from_rgb(130, 135, 150)
                } else {
                    Color32::from_rgba_unmultiplied(40, 45, 55, 120)
                };
                painter.rect_filled(shield_rect, CornerRadius::same(1), col);
            }
        }

        // --- ROW 2 (RIGHT): AIR BUBBLES (10 Bubbles, above Food) ---
        // Displayed ONLY when submerged or air < 300; completely hidden on dry land!
        if data.is_in_water || data.air < 300 {
            let full_bubbles = ((data.air.max(0) as f32 / 30.0).floor() as usize).min(10);
            let partial_bubble = (data.air.max(0) % 30) > 0 && full_bubbles < 10;

            for i in 0..10 {
                let center = Pos2::new(
                    right_end - (i as f32 + 0.5) * 8.0 * scale,
                    row2_y + 3.5 * scale,
                );
                if i < full_bubbles {
                    render_bubble_icon(painter, center, scale, true, false);
                } else if i == full_bubbles && partial_bubble {
                    render_bubble_icon(painter, center, scale, false, true);
                }
            }
        }
    }
}

/// 2. Standalone Hotbar: 9 continuous slots with active selector frame and item badges.
pub fn render_hotbar(painter: &Painter, rect: Rect, data: &HudData, tick_counter: u64) {
    let scale = (rect.width() / 186.0).clamp(0.4, 3.0);
    let bar_w = 182.0 * scale;
    let bar_h = 22.0 * scale;
    let start_x = rect.center().x - bar_w * 0.5;
    let start_y = rect.center().y - bar_h * 0.5;

    let hotbar_rect = Rect::from_min_size(Pos2::new(start_x, start_y), Vec2::new(bar_w, bar_h));
    painter.rect_filled(hotbar_rect, CornerRadius::ZERO, Color32::from_rgba_unmultiplied(18, 20, 26, 220));
    painter.rect_stroke(hotbar_rect, CornerRadius::ZERO, Stroke::new((1.0 * scale).max(0.5), Color32::from_rgb(40, 44, 52)), egui::StrokeKind::Inside);

    for i in 0..9 {
        let slot_left = start_x + (i as f32 * 20.0 + 1.0) * scale;
        let slot_rect = Rect::from_min_size(Pos2::new(slot_left, start_y + 1.0 * scale), Vec2::new(20.0 * scale, 20.0 * scale));

        if i > 0 {
            let div_x = start_x + (i as f32 * 20.0) * scale;
            painter.line_segment([Pos2::new(div_x, start_y + 1.0 * scale), Pos2::new(div_x, start_y + 21.0 * scale)], Stroke::new((1.0 * scale).max(0.5), Color32::from_rgb(45, 48, 58)));
        }

        if let Some(item) = &data.hotbar_items[i] {
            render_item_icon(painter, slot_rect, item, scale, tick_counter);

            if item.count > 1 {
                let count_str = format!("{}", item.count);
                let font = FontId::monospace((7.5 * scale).clamp(5.5, 9.5));
                let num_pos = Pos2::new(slot_rect.right() - 2.0 * scale, slot_rect.bottom() - 1.5 * scale);
                painter.text(Pos2::new(num_pos.x + 1.0, num_pos.y + 1.0), Align2::RIGHT_BOTTOM, &count_str, font.clone(), Color32::from_rgb(63, 63, 63));
                painter.text(num_pos, Align2::RIGHT_BOTTOM, count_str, font, Color32::WHITE);
            }

            if item.max_damage > 0 {
                let ratio = item.durability_ratio();
                let bar_w = (15.0 * scale) * ratio;
                let bar_y = slot_rect.bottom() - 2.5 * scale;
                let bar_x = slot_rect.left() + 2.5 * scale;
                let bar_color = if ratio > 0.5 {
                    Color32::from_rgb(85, 255, 85)
                } else if ratio > 0.2 {
                    Color32::from_rgb(255, 170, 0)
                } else {
                    Color32::from_rgb(255, 85, 85)
                };
                painter.line_segment([Pos2::new(bar_x, bar_y), Pos2::new(bar_x + 15.0 * scale, bar_y)], Stroke::new(1.5 * scale, Color32::from_rgb(10, 10, 15)));
                painter.line_segment([Pos2::new(bar_x, bar_y), Pos2::new(bar_x + bar_w, bar_y)], Stroke::new(1.5 * scale, bar_color));
            }
        }
    }

    let sel_idx = data.selected_hotbar_slot.min(8);
    let sel_x = start_x + (sel_idx as f32 * 20.0 - 1.0) * scale;
    let sel_y = start_y - 1.0 * scale;
    let sel_rect = Rect::from_min_size(Pos2::new(sel_x, sel_y), Vec2::new(22.0 * scale, 24.0 * scale));
    painter.rect_stroke(sel_rect, CornerRadius::ZERO, Stroke::new((2.0 * scale).max(1.5), Color32::WHITE), egui::StrokeKind::Outside);
    painter.rect_stroke(sel_rect, CornerRadius::ZERO, Stroke::new((0.8 * scale).max(0.5), Color32::from_rgb(60, 65, 75)), egui::StrokeKind::Inside);
    painter.rect_filled(sel_rect, CornerRadius::ZERO, Color32::from_rgba_unmultiplied(255, 255, 255, 15));
}

/// 3. Standalone Vitals: Health, Armor shields, Food shanks, Air bubbles.
pub fn render_vitals(painter: &Painter, rect: Rect, data: &HudData, tick_counter: u64) {
    let scale = (rect.width() / 186.0).clamp(0.4, 3.0);
    let left_start = rect.left() + 4.0;
    let right_end = rect.right() - 4.0;
    let row1_y = rect.bottom() - 10.0 * scale;
    let row2_y = rect.bottom() - 20.0 * scale;

    // --- HEALTH ROW (Bottom Left) ---
    let is_low_hp = data.health <= 4.0;
    let full_hearts = (data.health / 2.0).floor() as usize;
    let half_heart = (data.health % 2.0) >= 1.0;

    for i in 0..10 {
        let shake = if is_low_hp {
            let s = (tick_counter as f32 * 0.35 + i as f32 * 0.7).sin();
            s * 0.5 * scale
        } else {
            0.0
        };

        let h_pos = Pos2::new(left_start + i as f32 * 8.0 * scale, row1_y);
        let is_full = i < full_hearts;
        let is_half = i == full_hearts && half_heart;
        render_heart_icon(painter, h_pos, scale, is_full, is_half, false, shake);
    }

    // Absorption golden hearts overlay
    if data.absorption > 0.0 {
        let abs_full = (data.absorption / 2.0).floor() as usize;
        let abs_half = (data.absorption % 2.0) >= 1.0;
        for i in 0..abs_full.min(10) {
            let h_pos = Pos2::new(left_start + i as f32 * 8.0 * scale, row1_y);
            render_heart_icon(painter, h_pos, scale, true, false, true, 0.0);
        }
        if abs_full < 10 && abs_half {
            let h_pos = Pos2::new(left_start + abs_full as f32 * 8.0 * scale, row1_y);
            render_heart_icon(painter, h_pos, scale, false, true, true, 0.0);
        }
    }

    // --- ARMOR ROW (Top Left, directly above health) ---
    if data.armor_value > 0 {
        let full_shields = (data.armor_value / 2) as usize;
        let half_shield = (data.armor_value % 2) != 0;

        for i in 0..10 {
            let x = left_start + i as f32 * 8.0 * scale;
            let shield_rect = Rect::from_min_size(Pos2::new(x, row2_y), Vec2::new(7.0 * scale, 7.0 * scale));
            let col = if i < full_shields {
                Color32::from_rgb(180, 185, 200)
            } else if i == full_shields && half_shield {
                Color32::from_rgb(130, 135, 150)
            } else {
                Color32::from_rgba_unmultiplied(40, 45, 55, 120)
            };
            painter.rect_filled(shield_rect, CornerRadius::same(1), col);
        }
    }

    // --- FOOD / HUNGER ROW (Bottom Right) ---
    let full_shanks = (data.food / 2) as usize;
    let half_shank = (data.food % 2) != 0;

    for i in 0..10 {
        let f_pos = Pos2::new(right_end - (i as f32 + 1.0) * 8.0 * scale, row1_y);
        let is_full = i < full_shanks;
        let is_half = i == full_shanks && half_shank;
        render_hunger_icon(painter, f_pos, scale, is_full, is_half);
    }

    // --- AIR BUBBLES ROW (Top Right, above food if submerged or air < 300) ---
    if data.is_in_water || data.air < 300 {
        let full_bubbles = ((data.air.max(0) as f32 / 30.0).floor() as usize).min(10);
        let partial_bubble = (data.air.max(0) % 30) > 0 && full_bubbles < 10;

        for i in 0..10 {
            let center = Pos2::new(
                right_end - (i as f32 + 0.5) * 8.0 * scale,
                row2_y + 3.5 * scale,
            );
            if i < full_bubbles {
                render_bubble_icon(painter, center, scale, true, false);
            } else if i == full_bubbles && partial_bubble {
                render_bubble_icon(painter, center, scale, false, true);
            }
        }
    }
}

/// 4. Experience Bar: green progress bar with centered level counter.
pub fn render_exp_bar(painter: &Painter, rect: Rect, data: &HudData) {
    let bar_rect = Rect::from_min_size(
        Pos2::new(rect.left() + 2.0, rect.bottom() - 6.0),
        Vec2::new(rect.width() - 4.0, 4.0),
    );

    // Dark background groove
    painter.rect_filled(bar_rect, CornerRadius::same(1), Color32::from_rgb(25, 35, 20));
    painter.rect_stroke(
        bar_rect,
        CornerRadius::same(1),
        Stroke::new(1.0_f32, Color32::from_rgb(10, 15, 10)),
        egui::StrokeKind::Inside,
    );

    // Green filled portion
    let progress = data.exp_progress.clamp(0.0, 1.0);
    if progress > 0.0 {
        let fill_w = bar_rect.width() * progress;
        let fill_rect = Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_rect.height()));
        painter.rect_filled(fill_rect, CornerRadius::same(1), Color32::from_rgb(128, 255, 32));
    }

    // Centered XP Level number
    if data.exp_level > 0 {
        let lvl_str = format!("{}", data.exp_level);
        let center = Pos2::new(rect.center().x, rect.top() + 4.0);

        // 4-way black drop shadow
        let font = FontId::monospace(10.0);
        let offsets = [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)];
        for (dx, dy) in offsets {
            painter.text(
                Pos2::new(center.x + dx, center.y + dy),
                Align2::CENTER_CENTER,
                &lvl_str,
                font.clone(),
                Color32::BLACK,
            );
        }

        // Vibrant lime green text
        painter.text(
            center,
            Align2::CENTER_CENTER,
            lvl_str,
            font,
            Color32::from_rgb(128, 255, 32),
        );
    }
}

/// 5. Boss Bar: multiple boss health bars with notch marks and names.
pub fn render_boss_bar(painter: &Painter, rect: Rect, data: &HudData) {
    if data.boss_bars.is_empty() {
        return;
    }

    let mut y = rect.top() + 2.0;
    for boss in &data.boss_bars {
        // Boss Title
        painter.text(
            Pos2::new(rect.center().x, y),
            Align2::CENTER_TOP,
            &boss.name,
            FontId::proportional(10.5),
            Color32::WHITE,
        );

        y += 12.0;

        let bar_rect = Rect::from_min_size(
            Pos2::new(rect.left() + 4.0, y),
            Vec2::new(rect.width() - 8.0, 5.0),
        );

        // Background
        painter.rect_filled(bar_rect, CornerRadius::same(1), Color32::from_rgb(35, 10, 35));
        painter.rect_stroke(
            bar_rect,
            CornerRadius::same(1),
            Stroke::new(1.0_f32, Color32::from_rgb(15, 5, 15)),
            egui::StrokeKind::Inside,
        );

        // Filled bar (Purple / Magenta)
        let fill_w = bar_rect.width() * boss.health_percent;
        if fill_w > 0.0 {
            painter.rect_filled(
                Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_rect.height())),
                CornerRadius::same(1),
                Color32::from_rgb(205, 45, 205),
            );
        }

        // Notches at 20%, 40%, 60%, 80%
        for n in 1..5 {
            let notch_x = bar_rect.left() + (bar_rect.width() * n as f32 * 0.2);
            painter.line_segment(
                [Pos2::new(notch_x, bar_rect.top()), Pos2::new(notch_x, bar_rect.bottom())],
                Stroke::new(1.0_f32, Color32::from_rgb(15, 5, 15)),
            );
        }

        y += 8.0;
    }
}

/// 6. Keystrokes & CPS: W, A, S, D, LMB (CPS), RMB (CPS), Space keys.
pub fn render_keystrokes(painter: &Painter, rect: Rect, data: &HudData) {
    let scale = (rect.width() / 84.0).min(rect.height() / 92.0).clamp(0.4, 3.0);
    let kw = 24.0 * scale;
    let kh = 22.0 * scale;
    let gap = 3.0 * scale;
    let space_h = 11.0 * scale;

    let total_h = kh * 3.0 + gap * 3.0 + space_h;
    let center_x = rect.center().x;
    let top_y = rect.center().y - total_h * 0.5;

    // Helper closure to draw a single keystroke button
    let draw_key = |p: &Painter, r: Rect, label: &str, sub: Option<&str>, pressed: bool| {
        let (bg, text_col, border_col) = if pressed {
            (
                Color32::from_rgba_unmultiplied(225, 230, 240, 240),
                Color32::from_rgb(15, 18, 24),
                Color32::WHITE,
            )
        } else {
            (
                Color32::from_rgba_unmultiplied(18, 22, 28, 210),
                Color32::from_rgb(200, 205, 220),
                Color32::from_rgb(55, 62, 75),
            )
        };

        p.rect_filled(r, CornerRadius::same((3.0 * scale).max(1.0) as u8), bg);
        p.rect_stroke(
            r,
            CornerRadius::same((3.0 * scale).max(1.0) as u8),
            Stroke::new((1.0 * scale).max(0.5), border_col),
            egui::StrokeKind::Inside,
        );

        if let Some(subtext) = sub {
            p.text(
                Pos2::new(r.center().x, r.top() + 3.0 * scale),
                Align2::CENTER_TOP,
                label,
                FontId::proportional(9.0 * scale),
                text_col,
            );
            p.text(
                Pos2::new(r.center().x, r.bottom() - 2.0 * scale),
                Align2::CENTER_BOTTOM,
                subtext,
                FontId::monospace(8.0 * scale),
                text_col,
            );
        } else {
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0 * scale),
                text_col,
            );
        }
    };

    // Row 1: [ W ]
    let w_rect = Rect::from_min_size(Pos2::new(center_x - kw * 0.5, top_y), Vec2::new(kw, kh));
    draw_key(painter, w_rect, "W", None, data.key_w);

    // Row 2: [ A ] [ S ] [ D ]
    let r2_y = top_y + kh + gap;
    let a_rect = Rect::from_min_size(Pos2::new(center_x - kw * 1.5 - gap, r2_y), Vec2::new(kw, kh));
    let s_rect = Rect::from_min_size(Pos2::new(center_x - kw * 0.5, r2_y), Vec2::new(kw, kh));
    let d_rect = Rect::from_min_size(Pos2::new(center_x + kw * 0.5 + gap, r2_y), Vec2::new(kw, kh));
    draw_key(painter, a_rect, "A", None, data.key_a);
    draw_key(painter, s_rect, "S", None, data.key_s);
    draw_key(painter, d_rect, "D", None, data.key_d);

    // Row 3: [ LMB (CPS) ] [ RMB (CPS) ]
    let r3_y = r2_y + kh + gap;
    let mw = (kw * 3.0 + gap * 2.0 - gap) * 0.5;
    let lmb_rect = Rect::from_min_size(Pos2::new(center_x - mw - gap * 0.5, r3_y), Vec2::new(mw, kh));
    let rmb_rect = Rect::from_min_size(Pos2::new(center_x + gap * 0.5, r3_y), Vec2::new(mw, kh));
    let lmb_sub = format!("{} CPS", data.lmb_cps);
    let rmb_sub = format!("{} CPS", data.rmb_cps);
    draw_key(painter, lmb_rect, "LMB", Some(&lmb_sub), data.key_lmb);
    draw_key(painter, rmb_rect, "RMB", Some(&rmb_sub), data.key_rmb);

    // Row 4: [ ---- SPACE ---- ]
    let r4_y = r3_y + kh + gap;
    let space_w = kw * 3.0 + gap * 2.0;
    let space_rect = Rect::from_min_size(Pos2::new(center_x - space_w * 0.5, r4_y), Vec2::new(space_w, space_h));
    draw_key(painter, space_rect, "——", None, data.key_space);
}

/// 7. FPS & Telemetry Sparkline: FPS, 1% Low, RTT and mini frametime graph.
pub fn render_fps_ping(painter: &Painter, rect: Rect, data: &HudData) {
    let scale = (rect.width() / 180.0).min(rect.height() / 52.0).clamp(0.4, 3.0);
    let top_y = rect.top() + 4.0 * scale;
    let left_x = rect.left() + 6.0 * scale;

    let (fps, low1, rtt) = if let Some(b) = &data.benchmark {
        (b.current_fps, b.low_1_percent_fps, b.rtt_ms as i32)
    } else {
        (120.0, 95.0, data.ping_ms)
    };

    // FPS Label
    painter.text(
        Pos2::new(left_x, top_y),
        Align2::LEFT_TOP,
        format!("FPS: {:.0}", fps),
        FontId::monospace(11.0 * scale),
        Color32::from_rgb(85, 255, 85),
    );

    // 1% Low
    painter.text(
        Pos2::new(left_x + 68.0 * scale, top_y),
        Align2::LEFT_TOP,
        format!("1%: {:.0}", low1),
        FontId::monospace(10.0 * scale),
        Color32::from_rgb(255, 215, 85),
    );

    // Ping / RTT
    let ping_color = if rtt < 60 {
        Color32::from_rgb(85, 255, 85)
    } else if rtt < 130 {
        Color32::from_rgb(255, 255, 85)
    } else {
        Color32::from_rgb(255, 85, 85)
    };
    painter.text(
        Pos2::new(rect.right() - 6.0 * scale, top_y),
        Align2::RIGHT_TOP,
        format!("{} ms", rtt),
        FontId::monospace(10.0 * scale),
        ping_color,
    );

    // Mini Frametime Graph
    let graph_top = top_y + 16.0 * scale;
    let graph_h = (rect.bottom() - 4.0 * scale - graph_top).max(8.0 * scale);
    let graph_rect = Rect::from_min_size(
        Pos2::new(left_x, graph_top),
        Vec2::new(rect.width() - 12.0 * scale, graph_h),
    );

    painter.rect_filled(graph_rect, CornerRadius::same(2), Color32::from_rgb(14, 16, 22));
    painter.rect_stroke(
        graph_rect,
        CornerRadius::same(2),
        Stroke::new(1.0_f32, Color32::from_rgb(45, 50, 60)),
        egui::StrokeKind::Inside,
    );

    if let Some(b) = &data.benchmark {
        let count = b.recent_frametimes.len();
        if count > 0 {
            let bar_w = graph_rect.width() / count as f32;
            let max_ms = 33.33_f64; // up to 30 FPS ceiling

            // 60 FPS (16.7ms) line
            let y_60 = graph_rect.bottom() - (16.67 / max_ms as f32 * graph_rect.height());
            painter.line_segment(
                [Pos2::new(graph_rect.left(), y_60), Pos2::new(graph_rect.right(), y_60)],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 170, 0, 70)),
            );

            for (i, &ms) in b.recent_frametimes.iter().enumerate() {
                let x0 = graph_rect.left() + i as f32 * bar_w;
                let h = ((ms / max_ms) as f32 * graph_rect.height()).clamp(1.0, graph_rect.height());
                let y0 = graph_rect.bottom() - h;

                let color = if ms <= 8.33 {
                    Color32::from_rgb(85, 255, 85)
                } else if ms <= 16.67 {
                    Color32::from_rgb(140, 230, 80)
                } else {
                    Color32::from_rgb(255, 100, 100)
                };

                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x0 + (bar_w - 0.5).max(1.0), graph_rect.bottom())),
                    CornerRadius::ZERO,
                    color,
                );
            }
        }
    }
}

/// 8. Armor Status: Helmet, Chestplate, Leggings, Boots, and Hand item durabilities.
pub fn render_armor_status(painter: &Painter, rect: Rect, data: &HudData) {
    let mut items = Vec::new();
    for armor in &data.armor_items {
        if let Some(a) = armor {
            items.push(a);
        }
    }
    if let Some(held) = &data.held_item {
        if held.max_damage > 0 {
            items.push(held);
        }
    }

    if items.is_empty() {
        return;
    }

    let mut y = rect.top() + 4.0;
    for item in items {
        // Item short title
        painter.text(
            Pos2::new(rect.left() + 6.0, y),
            Align2::LEFT_TOP,
            &item.name,
            FontId::proportional(9.5),
            Color32::WHITE,
        );

        // Durability text
        let dur_str = if item.max_damage > 0 {
            format!("{}", item.durability_remaining())
        } else {
            "∞".to_string()
        };

        painter.text(
            Pos2::new(rect.right() - 6.0, y),
            Align2::RIGHT_TOP,
            dur_str,
            FontId::monospace(9.0),
            Color32::from_rgb(255, 255, 120),
        );

        y += 11.0;

        // Mini durability bar
        if item.max_damage > 0 {
            let ratio = item.durability_ratio();
            let bar_rect = Rect::from_min_size(
                Pos2::new(rect.left() + 6.0, y),
                Vec2::new(rect.width() - 12.0, 2.5),
            );

            painter.rect_filled(bar_rect, CornerRadius::same(1), Color32::from_rgb(25, 28, 35));
            let fill_w = bar_rect.width() * ratio;
            let col = if ratio > 0.5 {
                Color32::from_rgb(85, 255, 85)
            } else if ratio > 0.2 {
                Color32::from_rgb(255, 170, 0)
            } else {
                Color32::from_rgb(255, 85, 85)
            };
            painter.rect_filled(
                Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_rect.height())),
                CornerRadius::same(1),
                col,
            );
        }

        y += 6.0;
    }
}

/// 9. Potion Effects: active potions list with icons, names, levels and timers.
pub fn render_potion_effects(painter: &Painter, rect: Rect, data: &HudData) {
    if data.potion_effects.is_empty() {
        return;
    }

    let mut y = rect.top() + 4.0;
    for pot in &data.potion_effects {
        // Colored dot
        painter.circle_filled(Pos2::new(rect.left() + 8.0, y + 5.0), 3.0, pot.color);

        // Name + Level
        let name_str = format!("{} {}", pot.name, pot.format_level());
        painter.text(
            Pos2::new(rect.left() + 16.0, y),
            Align2::LEFT_TOP,
            name_str,
            FontId::proportional(9.5),
            Color32::WHITE,
        );

        // Timer
        painter.text(
            Pos2::new(rect.right() - 6.0, y),
            Align2::RIGHT_TOP,
            pot.format_duration(),
            FontId::monospace(9.0),
            Color32::from_rgb(255, 215, 85),
        );

        y += 18.0;
    }
}

/// 10. Coordinates, Biome & Compass Direction.
pub fn render_coords_direction(painter: &Painter, rect: Rect, data: &HudData) {
    let scale = (rect.width() / 170.0).min(rect.height() / 48.0).clamp(0.4, 3.0);
    let top_y = rect.top() + 4.0 * scale;
    let left_x = rect.left() + 6.0 * scale;

    // XYZ
    painter.text(
        Pos2::new(left_x, top_y),
        Align2::LEFT_TOP,
        format!("XYZ: {:.1} / {:.1} / {:.1}", data.pos.0, data.pos.1, data.pos.2),
        FontId::monospace(10.0 * scale),
        Color32::WHITE,
    );

    // Compass Direction based on yaw
    let (dir_name, axis_name) = compute_facing_direction(data.look.0);
    painter.text(
        Pos2::new(left_x, top_y + 15.0 * scale),
        Align2::LEFT_TOP,
        format!("Facing: {} ({})", dir_name, axis_name),
        FontId::proportional(10.0 * scale),
        Color32::from_rgb(85, 255, 255),
    );

    // Yaw / Pitch / Biome
    painter.text(
        Pos2::new(left_x, top_y + 30.0 * scale),
        Align2::LEFT_TOP,
        format!("Biome: {} [Y:{:.0} P:{:.0}]", data.biome, data.look.0, data.look.1),
        FontId::proportional(9.0 * scale),
        Color32::from_rgb(170, 175, 190),
    );
}

/// Computes facing cardinal direction and primary axis from 1.8.9 yaw angle.
pub fn compute_facing_direction(yaw: f32) -> (&'static str, &'static str) {
    let normalized_yaw = (yaw % 360.0 + 360.0) % 360.0;
    // Minecraft: 0 = South (+Z), 90 = West (-X), 180 = North (-Z), 270 = East (+X)
    let octant = ((normalized_yaw + 22.5) / 45.0).floor() as usize % 8;
    match octant {
        0 => ("South", "+Z"),
        1 => ("South-West", "+Z/-X"),
        2 => ("West", "-X"),
        3 => ("North-West", "-Z/-X"),
        4 => ("North", "-Z"),
        5 => ("North-East", "-Z/+X"),
        6 => ("East", "+X"),
        7 => ("South-East", "+Z/+X"),
        _ => ("South", "+Z"),
    }
}
