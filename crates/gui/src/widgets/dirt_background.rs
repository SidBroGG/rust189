use egui::{Color32, FontId, Pos2, Rect, Ui, Vec2};

/// Paints the classic Minecraft dirt background with header and footer bars.
pub fn draw_dirt_background(ui: &mut Ui, screen_title: Option<&str>) {
    let screen_rect = ui.max_rect();
    let painter = ui.painter();

    // 1. Dark dirt background base fill
    painter.rect_filled(
        screen_rect,
        0.0,
        Color32::from_rgb(32, 26, 22),
    );

    // 2. Procedural pixel grid variation to mimic 16x16 tiled dirt
    let tile_size = 16.0;
    let cols = (screen_rect.width() / tile_size).ceil() as usize;
    let rows = (screen_rect.height() / tile_size).ceil() as usize;

    for y in 0..rows {
        for x in 0..cols {
            // Subtle pseudo-random variation per tile
            let hash = ((x * 374761393) ^ (y * 668265263)) & 0xFF;
            let shade_offset = (hash % 9) as i16 - 4;
            let r = (32 + shade_offset).clamp(0, 255) as u8;
            let g = (26 + shade_offset).clamp(0, 255) as u8;
            let b = (22 + shade_offset).clamp(0, 255) as u8;

            let tile_rect = Rect::from_min_size(
                Pos2::new(
                    screen_rect.min.x + x as f32 * tile_size,
                    screen_rect.min.y + y as f32 * tile_size,
                ),
                Vec2::splat(tile_size),
            );

            if screen_rect.intersects(tile_rect) {
                painter.rect_filled(tile_rect, 0.0, Color32::from_rgb(r, g, b));
            }
        }
    }

    // 3. Top Header Bar (dark overlay)
    let header_height = 42.0;
    let header_rect = Rect::from_min_size(
        screen_rect.min,
        Vec2::new(screen_rect.width(), header_height),
    );
    painter.rect_filled(
        header_rect,
        0.0,
        Color32::from_rgba_premultiplied(0, 0, 0, 160),
    );

    // 4. Bottom Footer Bar (dark overlay)
    let footer_height = 54.0;
    let footer_rect = Rect::from_min_size(
        Pos2::new(screen_rect.min.x, screen_rect.max.y - footer_height),
        Vec2::new(screen_rect.width(), footer_height),
    );
    painter.rect_filled(
        footer_rect,
        0.0,
        Color32::from_rgba_premultiplied(0, 0, 0, 160),
    );

    // 5. Title text with drop shadow if provided
    if let Some(title) = screen_title {
        let center = Pos2::new(screen_rect.center().x, screen_rect.min.y + header_height / 2.0);
        let font_id = FontId::new(18.0, egui::FontFamily::Proportional);

        // Shadow
        painter.text(
            Pos2::new(center.x + 1.0, center.y + 1.0),
            egui::Align2::CENTER_CENTER,
            title,
            font_id.clone(),
            Color32::from_rgb(56, 56, 56),
        );
        // White title
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            title,
            font_id,
            Color32::from_rgb(255, 255, 255),
        );
    }
}
