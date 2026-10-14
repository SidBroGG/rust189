use egui::{Color32, FontId, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};

/// Authentic Minecraft 1.8.9 styled button widget.
pub struct MinecraftButton<'a> {
    text: &'a str,
    enabled: bool,
    size: Vec2,
}

impl<'a> MinecraftButton<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            enabled: true,
            size: Vec2::new(200.0, 20.0),
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.size.x = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.size.y = height;
        self
    }
}

impl<'a> Widget for MinecraftButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(
            self.size,
            if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let is_hovered = response.hovered() && self.enabled;

            // Palette matching Minecraft 1.8.9 GUI button
            let (fill_col, top_left_col, bot_right_col, text_col, shadow_col) = if !self.enabled {
                (
                    Color32::from_rgb(58, 58, 58),
                    Color32::from_rgb(78, 78, 78),
                    Color32::from_rgb(32, 32, 32),
                    Color32::from_rgb(160, 160, 160),
                    Color32::from_rgb(40, 40, 40),
                )
            } else if is_hovered {
                (
                    Color32::from_rgb(142, 142, 142),
                    Color32::from_rgb(180, 180, 220),
                    Color32::from_rgb(70, 70, 90),
                    Color32::from_rgb(255, 255, 160),
                    Color32::from_rgb(63, 63, 40),
                )
            } else {
                (
                    Color32::from_rgb(114, 114, 114),
                    Color32::from_rgb(150, 150, 150),
                    Color32::from_rgb(55, 55, 55),
                    Color32::from_rgb(224, 224, 224),
                    Color32::from_rgb(56, 56, 56),
                )
            };

            // 1. Black outer outline (1px)
            painter.rect_filled(rect, 0.0, Color32::from_rgb(0, 0, 0));

            // 2. Button body (inset 1px)
            let inner_rect = rect.shrink(1.0);
            painter.rect_filled(inner_rect, 0.0, fill_col);

            // 3. Beveled edges
            // Top highlight
            painter.rect_filled(
                Rect::from_min_size(inner_rect.min, Vec2::new(inner_rect.width(), 1.0)),
                0.0,
                top_left_col,
            );
            // Left highlight
            painter.rect_filled(
                Rect::from_min_size(inner_rect.min, Vec2::new(1.0, inner_rect.height())),
                0.0,
                top_left_col,
            );
            // Bottom shadow
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(inner_rect.min.x, inner_rect.max.y - 1.0),
                    Vec2::new(inner_rect.width(), 1.0),
                ),
                0.0,
                bot_right_col,
            );
            // Right shadow
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(inner_rect.max.x - 1.0, inner_rect.min.y),
                    Vec2::new(1.0, inner_rect.height()),
                ),
                0.0,
                bot_right_col,
            );

            // 4. Centered text with authentic Minecraft 1px drop shadow
            let font_id = FontId::new(14.0, egui::FontFamily::Proportional);
            let center = rect.center();

            // Drop shadow
            painter.text(
                Pos2::new(center.x + 1.0, center.y + 1.0),
                egui::Align2::CENTER_CENTER,
                self.text,
                font_id.clone(),
                shadow_col,
            );
            // Main text
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                self.text,
                font_id,
                text_col,
            );
        }

        response
    }
}
