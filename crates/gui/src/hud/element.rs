use egui::{Color32, Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};

/// Screen anchor position for calculating element base coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl HudAnchor {
    /// Computes the anchor position in screen coordinates given the viewport size.
    #[inline]
    pub fn compute_base_pos(&self, screen_size: Vec2) -> Pos2 {
        match self {
            Self::TopLeft => Pos2::new(0.0, 0.0),
            Self::TopCenter => Pos2::new(screen_size.x * 0.5, 0.0),
            Self::TopRight => Pos2::new(screen_size.x, 0.0),
            Self::CenterLeft => Pos2::new(0.0, screen_size.y * 0.5),
            Self::Center => Pos2::new(screen_size.x * 0.5, screen_size.y * 0.5),
            Self::CenterRight => Pos2::new(screen_size.x, screen_size.y * 0.5),
            Self::BottomLeft => Pos2::new(0.0, screen_size.y),
            Self::BottomCenter => Pos2::new(screen_size.x * 0.5, screen_size.y),
            Self::BottomRight => Pos2::new(screen_size.x, screen_size.y),
        }
    }
}

/// Configuration and layout parameters for an individual HUD element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudElementConfig {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub anchor: HudAnchor,
    pub offset: [f32; 2],
    pub scale: f32,
    pub bg_color: [u8; 4],
    pub border_color: [u8; 4],
    pub border_width: f32,
    pub corner_radius: f32,
}

impl HudElementConfig {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        anchor: HudAnchor,
        offset: [f32; 2],
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            visible: true,
            anchor,
            offset,
            scale: 1.0,
            bg_color: [12, 14, 18, 190],
            border_color: [65, 70, 80, 180],
            border_width: 1.0,
            corner_radius: 4.0,
        }
    }

    /// Computes the element bounding box on screen based on anchor, offset, and content size.
    pub fn screen_rect(&self, screen_size: Vec2, content_size: Vec2) -> Rect {
        let base = self.anchor.compute_base_pos(screen_size);
        let scaled_size = content_size * self.scale;

        // Position offset depends on anchor alignment
        let x = match self.anchor {
            HudAnchor::TopLeft | HudAnchor::CenterLeft | HudAnchor::BottomLeft => {
                base.x + self.offset[0]
            }
            HudAnchor::TopCenter | HudAnchor::Center | HudAnchor::BottomCenter => {
                base.x + self.offset[0] - scaled_size.x * 0.5
            }
            HudAnchor::TopRight | HudAnchor::CenterRight | HudAnchor::BottomRight => {
                base.x + self.offset[0] - scaled_size.x
            }
        };

        let y = match self.anchor {
            HudAnchor::TopLeft | HudAnchor::TopCenter | HudAnchor::TopRight => {
                base.y + self.offset[1]
            }
            HudAnchor::CenterLeft | HudAnchor::Center | HudAnchor::CenterRight => {
                base.y + self.offset[1] - scaled_size.y * 0.5
            }
            HudAnchor::BottomLeft | HudAnchor::BottomCenter | HudAnchor::BottomRight => {
                base.y + self.offset[1] - scaled_size.y
            }
        };

        Rect::from_min_size(Pos2::new(x, y), scaled_size)
    }

    /// Calculates new offset from a dragged screen position, applying grid snapping.
    pub fn set_from_screen_pos(
        &mut self,
        target_min: Pos2,
        screen_size: Vec2,
        content_size: Vec2,
        grid_step: u32,
    ) {
        let base = self.anchor.compute_base_pos(screen_size);
        let scaled_size = content_size * self.scale;

        // Invert the alignment calculation to get raw offset
        let raw_ox = match self.anchor {
            HudAnchor::TopLeft | HudAnchor::CenterLeft | HudAnchor::BottomLeft => {
                target_min.x - base.x
            }
            HudAnchor::TopCenter | HudAnchor::Center | HudAnchor::BottomCenter => {
                target_min.x - base.x + scaled_size.x * 0.5
            }
            HudAnchor::TopRight | HudAnchor::CenterRight | HudAnchor::BottomRight => {
                target_min.x - base.x + scaled_size.x
            }
        };

        let raw_oy = match self.anchor {
            HudAnchor::TopLeft | HudAnchor::TopCenter | HudAnchor::TopRight => {
                target_min.y - base.y
            }
            HudAnchor::CenterLeft | HudAnchor::Center | HudAnchor::CenterRight => {
                target_min.y - base.y + scaled_size.y * 0.5
            }
            HudAnchor::BottomLeft | HudAnchor::BottomCenter | HudAnchor::BottomRight => {
                target_min.y - base.y + scaled_size.y
            }
        };

        self.offset = [
            snap_val(raw_ox, grid_step),
            snap_val(raw_oy, grid_step),
        ];
    }

    #[inline]
    pub fn bg_color32(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.bg_color[0],
            self.bg_color[1],
            self.bg_color[2],
            self.bg_color[3],
        )
    }

    #[inline]
    pub fn border_color32(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(
            self.border_color[0],
            self.border_color[1],
            self.border_color[2],
            self.border_color[3],
        )
    }
}

/// Snaps a scalar value to the nearest multiple of `step`.
/// If `step == 0`, snapping is disabled (Free movement).
#[inline]
pub fn snap_val(val: f32, step: u32) -> f32 {
    if step == 0 {
        val
    } else {
        (val / step as f32).round() * step as f32
    }
}
