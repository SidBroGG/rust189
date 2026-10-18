use super::data::{HudBossBarInfo, HudData};
use egui::{
    Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Stroke, Vec2,
};

/// Content dimensions for each HUD widget at scale 1.0.
pub fn get_module_content_size(id: &str, data: &HudData) -> Vec2 {
    match id {
        "crosshair" => Vec2::new(16.0, 16.0),
        "hotbar" => Vec2::new(186.0, 26.0),
        "vitals" => Vec2::new(186.0, 28.0),
        "exp_bar" => Vec2::new(186.0, 16.0),
        "boss_bar" => {
            let count = data.boss_bars.len().max(1);
            Vec2::new(186.0, (count as f32 * 20.0).max(20.0))
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

/// 2. Hotbar: 9 slots with active slot highlight border and item badges.
pub fn render_hotbar(painter: &Painter, rect: Rect, data: &HudData) {
    let slot_w = 20.0 * (rect.width() / 186.0);
    let slot_h = rect.height() - 4.0;
    let start_x = rect.left() + 3.0;
    let start_y = rect.top() + 2.0;

    for i in 0..9 {
        let x = start_x + i as f32 * slot_w;
        let slot_rect = Rect::from_min_size(Pos2::new(x, start_y), Vec2::new(slot_w - 1.0, slot_h));

        let is_selected = i == data.selected_hotbar_slot;

        // Slot interior
        let slot_bg = if is_selected {
            Color32::from_rgba_unmultiplied(45, 50, 65, 230)
        } else {
            Color32::from_rgba_unmultiplied(20, 22, 28, 200)
        };
        painter.rect_filled(slot_rect, CornerRadius::same(2), slot_bg);

        // Slot border
        let border_stroke = if is_selected {
            Stroke::new(2.0_f32, Color32::from_rgb(255, 255, 255))
        } else {
            Stroke::new(1.0_f32, Color32::from_rgb(60, 65, 75))
        };
        painter.rect_stroke(slot_rect, CornerRadius::same(2), border_stroke, egui::StrokeKind::Inside);

        // Slot number indicator (1..9)
        painter.text(
            Pos2::new(slot_rect.left() + 2.0, slot_rect.top() + 1.0),
            Align2::LEFT_TOP,
            format!("{}", i + 1),
            FontId::monospace(8.0),
            Color32::from_rgb(140, 145, 160),
        );

        // Item rendering
        if let Some(item) = &data.hotbar_items[i] {
            // Item name shorthand or ID in center
            let short_name: String = item.name.chars().take(3).collect();
            painter.text(
                Pos2::new(slot_rect.center().x, slot_rect.center().y - 1.0),
                Align2::CENTER_CENTER,
                short_name,
                FontId::proportional(9.0),
                Color32::WHITE,
            );

            // Stack count in bottom-right if > 1
            if item.count > 1 {
                painter.text(
                    Pos2::new(slot_rect.right() - 2.0, slot_rect.bottom() - 1.0),
                    Align2::RIGHT_BOTTOM,
                    format!("{}", item.count),
                    FontId::monospace(8.5),
                    Color32::from_rgb(255, 255, 85),
                );
            }

            // Durability bar at bottom of slot
            if item.max_damage > 0 {
                let ratio = item.durability_ratio();
                let bar_w = (slot_rect.width() - 4.0) * ratio;
                let bar_y = slot_rect.bottom() - 2.5;
                let bar_color = if ratio > 0.5 {
                    Color32::from_rgb(85, 255, 85)
                } else if ratio > 0.2 {
                    Color32::from_rgb(255, 170, 0)
                } else {
                    Color32::from_rgb(255, 85, 85)
                };
                painter.line_segment(
                    [Pos2::new(slot_rect.left() + 2.0, bar_y), Pos2::new(slot_rect.left() + 2.0 + bar_w, bar_y)],
                    Stroke::new(1.5_f32, bar_color),
                );
            }
        }
    }
}

/// 3. Vitals: Health (with low-HP shake), Armor shields, Food shanks, Air bubbles.
pub fn render_vitals(painter: &Painter, rect: Rect, data: &HudData, tick_counter: u64) {
    let left_start = rect.left() + 4.0;
    let right_end = rect.right() - 4.0;
    let bottom_y = rect.bottom() - 6.0;
    let top_y = rect.top() + 3.0;

    // --- HEALTH ROW (Bottom Left) ---
    let is_low_hp = data.health <= 4.0;
    let full_hearts = (data.health / 2.0).floor() as usize;
    let half_heart = (data.health % 2.0) >= 1.0;

    for i in 0..10 {
        let shake = if is_low_hp {
            let s = ((tick_counter + i as u64 * 7) % 4) as f32;
            if s == 1.0 { -1.5 } else if s == 2.0 { 1.5 } else { 0.0 }
        } else {
            0.0
        };

        let x = left_start + i as f32 * 8.5;
        let y = bottom_y + shake;

        // Heart background container
        painter.circle_filled(Pos2::new(x + 2.0, y - 2.0), 3.0, Color32::from_rgb(40, 10, 10));
        painter.circle_filled(Pos2::new(x + 5.0, y - 2.0), 3.0, Color32::from_rgb(40, 10, 10));

        // Heart fill
        let heart_color = if i < full_hearts {
            Color32::from_rgb(255, 34, 34)
        } else if i == full_hearts && half_heart {
            Color32::from_rgb(255, 85, 85)
        } else {
            Color32::from_rgba_unmultiplied(30, 0, 0, 150)
        };

        let radius = if i < full_hearts || (i == full_hearts && half_heart) { 2.4 } else { 1.4 };
        painter.circle_filled(Pos2::new(x + 2.0, y - 2.0), radius, heart_color);
        painter.circle_filled(Pos2::new(x + 5.0, y - 2.0), radius, heart_color);
    }

    // Absorption golden hearts overlay (if any)
    if data.absorption > 0.0 {
        let abs_full = (data.absorption / 2.0).floor() as usize;
        for i in 0..abs_full.min(10) {
            let x = left_start + i as f32 * 8.5;
            painter.circle_filled(Pos2::new(x + 3.5, bottom_y - 2.0), 2.2, Color32::from_rgb(255, 215, 0));
        }
    }

    // --- ARMOR ROW (Top Left, directly above health) ---
    if data.armor_value > 0 {
        let full_shields = (data.armor_value / 2) as usize;
        let half_shield = (data.armor_value % 2) != 0;

        for i in 0..10 {
            let x = left_start + i as f32 * 8.5;
            let shield_rect = Rect::from_min_size(Pos2::new(x, top_y), Vec2::new(7.0, 7.0));

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
        let x = right_end - (i as f32 + 1.0) * 8.5;
        let y = bottom_y;

        // Food container
        let shank_color = if i < full_shanks {
            Color32::from_rgb(185, 105, 50)
        } else if i == full_shanks && half_shank {
            Color32::from_rgb(215, 140, 80)
        } else {
            Color32::from_rgba_unmultiplied(40, 25, 15, 140)
        };

        painter.rect_filled(
            Rect::from_min_size(Pos2::new(x, y - 5.0), Vec2::new(7.0, 6.0)),
            CornerRadius::same(2),
            shank_color,
        );
    }

    // --- AIR BUBBLES ROW (Top Right, above food if in water or air < 300) ---
    if data.is_in_water || data.air < 300 {
        let bubbles = ((data.air as f32 / 300.0) * 10.0).ceil().clamp(0.0, 10.0) as usize;
        for i in 0..bubbles {
            let x = right_end - (i as f32 + 1.0) * 8.5;
            painter.circle_stroke(
                Pos2::new(x + 3.5, top_y + 3.5),
                2.8,
                Stroke::new(1.0_f32, Color32::from_rgb(60, 180, 255)),
            );
            painter.circle_filled(
                Pos2::new(x + 2.5, top_y + 2.5),
                1.0,
                Color32::WHITE,
            );
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
    let items = if data.boss_bars.is_empty() {
        vec![HudBossBarInfo::new("Wither Boss", 1.0)]
    } else {
        data.boss_bars.clone()
    };

    let mut y = rect.top() + 2.0;
    for boss in items {
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
    let kw = 24.0;
    let kh = 22.0;
    let gap = 3.0;

    let center_x = rect.center().x;
    let top_y = rect.top() + 4.0;

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

        p.rect_filled(r, CornerRadius::same(3), bg);
        p.rect_stroke(r, CornerRadius::same(3), Stroke::new(1.0_f32, border_col), egui::StrokeKind::Inside);

        if let Some(subtext) = sub {
            p.text(
                Pos2::new(r.center().x, r.top() + 3.0),
                Align2::CENTER_TOP,
                label,
                FontId::proportional(9.0),
                text_col,
            );
            p.text(
                Pos2::new(r.center().x, r.bottom() - 2.0),
                Align2::CENTER_BOTTOM,
                subtext,
                FontId::monospace(8.0),
                text_col,
            );
        } else {
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0),
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
    let space_rect = Rect::from_min_size(Pos2::new(center_x - space_w * 0.5, r4_y), Vec2::new(space_w, 11.0));
    draw_key(painter, space_rect, "——", None, data.key_space);
}

/// 7. FPS & Telemetry Sparkline: FPS, 1% Low, RTT and mini frametime graph.
pub fn render_fps_ping(painter: &Painter, rect: Rect, data: &HudData) {
    let top_y = rect.top() + 4.0;
    let left_x = rect.left() + 6.0;

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
        FontId::monospace(11.0),
        Color32::from_rgb(85, 255, 85),
    );

    // 1% Low
    painter.text(
        Pos2::new(left_x + 68.0, top_y),
        Align2::LEFT_TOP,
        format!("1%: {:.0}", low1),
        FontId::monospace(10.0),
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
        Pos2::new(rect.right() - 6.0, top_y),
        Align2::RIGHT_TOP,
        format!("{} ms", rtt),
        FontId::monospace(10.0),
        ping_color,
    );

    // Mini Frametime Graph
    let graph_rect = Rect::from_min_size(
        Pos2::new(left_x, top_y + 16.0),
        Vec2::new(rect.width() - 12.0, rect.height() - 22.0),
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
        items.push(held);
    }

    if items.is_empty() {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "No Armor",
            FontId::proportional(10.0),
            Color32::from_rgb(140, 145, 160),
        );
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
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "No Active Effects",
            FontId::proportional(10.0),
            Color32::from_rgb(140, 145, 160),
        );
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
    let top_y = rect.top() + 4.0;
    let left_x = rect.left() + 6.0;

    // XYZ
    painter.text(
        Pos2::new(left_x, top_y),
        Align2::LEFT_TOP,
        format!("XYZ: {:.1} / {:.1} / {:.1}", data.pos.0, data.pos.1, data.pos.2),
        FontId::monospace(10.0),
        Color32::WHITE,
    );

    // Compass Direction based on yaw
    let (dir_name, axis_name) = compute_facing_direction(data.look.0);
    painter.text(
        Pos2::new(left_x, top_y + 16.0),
        Align2::LEFT_TOP,
        format!("Facing: {} ({})", dir_name, axis_name),
        FontId::proportional(10.0),
        Color32::from_rgb(85, 255, 255),
    );

    // Yaw / Pitch / Biome
    painter.text(
        Pos2::new(left_x, top_y + 32.0),
        Align2::LEFT_TOP,
        format!("Biome: {} [Y:{:.0} P:{:.0}]", data.biome, data.look.0, data.look.1),
        FontId::proportional(9.0),
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
