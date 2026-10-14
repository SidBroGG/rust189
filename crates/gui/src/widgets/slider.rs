use egui::{Color32, FontId, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};

/// Authentic Minecraft 1.8.9 styled slider widget.
pub struct MinecraftSlider<'a> {
    value: &'a mut f32,
    min: f32,
    max: f32,
    label: String,
    size: Vec2,
}

impl<'a> MinecraftSlider<'a> {
    pub fn new(value: &'a mut f32, min: f32, max: f32, label: impl Into<String>) -> Self {
        Self {
            value,
            min,
            max,
            label: label.into(),
            size: Vec2::new(150.0, 20.0),
        }
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.size.x = width;
        self
    }
}

impl<'a> Widget for MinecraftSlider<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, mut response) = ui.allocate_exact_size(self.size, Sense::click_and_drag());

        let range = (self.max - self.min).max(0.0001);
        let normalized = ((*self.value - self.min) / range).clamp(0.0, 1.0);

        if response.clicked() || response.dragged() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let thumb_half_width = 4.0;
                let track_min = rect.min.x + thumb_half_width;
                let track_max = rect.max.x - thumb_half_width;
                let track_width = (track_max - track_min).max(1.0);

                let new_norm = ((pointer_pos.x - track_min) / track_width).clamp(0.0, 1.0);
                *self.value = self.min + new_norm * range;
                response.mark_changed();
            }
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // 1. Draw slider track (darker button inset)
            painter.rect_filled(rect, 0.0, Color32::from_rgb(0, 0, 0));
            let track_rect = rect.shrink(1.0);
            painter.rect_filled(track_rect, 0.0, Color32::from_rgb(45, 45, 45));

            // Top/left inner shadow
            painter.rect_filled(
                Rect::from_min_size(track_rect.min, Vec2::new(track_rect.width(), 1.0)),
                0.0,
                Color32::from_rgb(20, 20, 20),
            );
            painter.rect_filled(
                Rect::from_min_size(track_rect.min, Vec2::new(1.0, track_rect.height())),
                0.0,
                Color32::from_rgb(20, 20, 20),
            );

            // 2. Draw slider thumb handle (8px wide button)
            let thumb_width = 8.0;
            let thumb_x = rect.min.x + normalized * (rect.width() - thumb_width);
            let thumb_rect = Rect::from_min_size(
                Pos2::new(thumb_x, rect.min.y),
                Vec2::new(thumb_width, rect.height()),
            );

            // Thumb black outline
            painter.rect_filled(thumb_rect, 0.0, Color32::from_rgb(0, 0, 0));
            let inner_thumb = thumb_rect.shrink(1.0);
            let is_thumb_active = response.dragged() || response.hovered();

            let (thumb_fill, thumb_hl, thumb_sh) = if is_thumb_active {
                (
                    Color32::from_rgb(142, 142, 142),
                    Color32::from_rgb(180, 180, 220),
                    Color32::from_rgb(70, 70, 90),
                )
            } else {
                (
                    Color32::from_rgb(114, 114, 114),
                    Color32::from_rgb(150, 150, 150),
                    Color32::from_rgb(55, 55, 55),
                )
            };

            painter.rect_filled(inner_thumb, 0.0, thumb_fill);
            // Thumb highlight and shadow
            painter.rect_filled(
                Rect::from_min_size(inner_thumb.min, Vec2::new(inner_thumb.width(), 1.0)),
                0.0,
                thumb_hl,
            );
            painter.rect_filled(
                Rect::from_min_size(inner_thumb.min, Vec2::new(1.0, inner_thumb.height())),
                0.0,
                thumb_hl,
            );
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(inner_thumb.min.x, inner_thumb.max.y - 1.0),
                    Vec2::new(inner_thumb.width(), 1.0),
                ),
                0.0,
                thumb_sh,
            );
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(inner_thumb.max.x - 1.0, inner_thumb.min.y),
                    Vec2::new(1.0, inner_thumb.height()),
                ),
                0.0,
                thumb_sh,
            );

            // 3. Centered label text with drop shadow
            let font_id = FontId::new(14.0, egui::FontFamily::Proportional);
            let center = rect.center();

            let (text_col, shadow_col) = if is_thumb_active {
                (Color32::from_rgb(255, 255, 160), Color32::from_rgb(63, 63, 40))
            } else {
                (Color32::from_rgb(224, 224, 224), Color32::from_rgb(56, 56, 56))
            };

            painter.text(
                Pos2::new(center.x + 1.0, center.y + 1.0),
                egui::Align2::CENTER_CENTER,
                &self.label,
                font_id.clone(),
                shadow_col,
            );
            painter.text(center, egui::Align2::CENTER_CENTER, &self.label, font_id, text_col);
        }

        response
    }
}
