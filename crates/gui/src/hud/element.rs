use egui::{Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};

/// Screen anchor position for reference and compatibility.
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

/// Configuration and relative grid layout parameters for an individual HUD element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudElementConfig {
    pub id: String,
    pub name: String,
    pub visible: bool,
    /// Relative grid X position [0.0 .. 1.0] across viewport width.
    #[serde(default)]
    pub rel_x: f32,
    /// Relative grid Y position [0.0 .. 1.0] across viewport height.
    #[serde(default)]
    pub rel_y: f32,
    /// Relative grid width [0.0 .. 1.0] across viewport width.
    #[serde(default = "default_rel_dim")]
    pub rel_w: f32,
    /// Relative grid height [0.0 .. 1.0] across viewport height.
    #[serde(default = "default_rel_dim")]
    pub rel_h: f32,
}

fn default_rel_dim() -> f32 {
    0.1
}

impl Default for HudElementConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            visible: true,
            rel_x: 0.0,
            rel_y: 0.0,
            rel_w: 0.2,
            rel_h: 0.1,
        }
    }
}

impl HudElementConfig {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        rel_x: f32,
        rel_y: f32,
        rel_w: f32,
        rel_h: f32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            visible: true,
            rel_x: rel_x.clamp(0.0, 1.0),
            rel_y: rel_y.clamp(0.0, 1.0),
            rel_w: rel_w.clamp(0.01, 1.0),
            rel_h: rel_h.clamp(0.01, 1.0),
        }
    }

    /// Computes the element bounding box on screen based on relative coordinates and viewport size.
    #[inline]
    pub fn screen_rect(&self, screen_size: Vec2) -> Rect {
        let x = self.rel_x * screen_size.x;
        let y = self.rel_y * screen_size.y;
        let w = (self.rel_w * screen_size.x).max(12.0);
        let h = (self.rel_h * screen_size.y).max(12.0);
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h))
    }

    /// Updates position from dragged screen position, snapping to relative grid units.
    pub fn set_from_screen_pos(
        &mut self,
        target_min: Pos2,
        screen_size: Vec2,
        grid_step: f32,
    ) {
        if screen_size.x <= 0.0 || screen_size.y <= 0.0 {
            return;
        }

        let snapped_x = snap_val_f32(target_min.x, grid_step);
        let snapped_y = snap_val_f32(target_min.y, grid_step);

        let max_x = (screen_size.x - self.rel_w * screen_size.x).max(0.0);
        let max_y = (screen_size.y - self.rel_h * screen_size.y).max(0.0);

        let clamped_x = snapped_x.clamp(0.0, max_x);
        let clamped_y = snapped_y.clamp(0.0, max_y);

        self.rel_x = (clamped_x / screen_size.x).clamp(0.0, 1.0);
        self.rel_y = (clamped_y / screen_size.y).clamp(0.0, 1.0);
    }

    /// Updates dimensions from resized screen bounds, snapping to relative grid units.
    pub fn set_from_screen_size(
        &mut self,
        target_size: Vec2,
        screen_size: Vec2,
        grid_step: f32,
    ) {
        if screen_size.x <= 0.0 || screen_size.y <= 0.0 {
            return;
        }

        let min_size = grid_step.max(16.0);
        let snapped_w = snap_val_f32(target_size.x, grid_step).max(min_size);
        let snapped_h = snap_val_f32(target_size.y, grid_step).max(min_size);

        let max_w = (screen_size.x - self.rel_x * screen_size.x).max(min_size);
        let max_h = (screen_size.y - self.rel_y * screen_size.y).max(min_size);

        let clamped_w = snapped_w.clamp(min_size, max_w);
        let clamped_h = snapped_h.clamp(min_size, max_h);

        self.rel_w = (clamped_w / screen_size.x).clamp(0.01, 1.0);
        self.rel_h = (clamped_h / screen_size.y).clamp(0.01, 1.0);
    }
}

/// Snaps a floating-point scalar value to the nearest multiple of `step`.
#[inline]
pub fn snap_val_f32(val: f32, step: f32) -> f32 {
    if step <= 0.0 {
        val
    } else {
        (val / step).round() * step
    }
}

/// Snaps a scalar value to the nearest integer multiple of `step`.
/// If `step == 0`, snapping is disabled.
#[inline]
pub fn snap_val(val: f32, step: u32) -> f32 {
    snap_val_f32(val, step as f32)
}
