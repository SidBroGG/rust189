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

/// Native baseline content dimensions (width, height) for each HUD module at 1.0x scale.
#[inline]
pub fn get_module_base_size(id: &str) -> Vec2 {
    match id {
        "hotbar" => Vec2::new(186.0, 52.0),
        "boss_bar" => Vec2::new(186.0, 22.0),
        "scoreboard" => Vec2::new(140.0, 120.0),
        "chat" => Vec2::new(320.0, 130.0),
        "keystrokes" => Vec2::new(84.0, 92.0),
        "fps_ping" => Vec2::new(180.0, 52.0),
        "armor_status" => Vec2::new(100.0, 75.0),
        "potion_effects" => Vec2::new(110.0, 65.0),
        "coords_direction" => Vec2::new(170.0, 48.0),
        _ => Vec2::new(100.0, 40.0),
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

    /// Computes the element bounding box on screen based on relative coordinates and viewport size,
    /// enforcing strict aspect-ratio locking so elements never stretch in only one dimension.
    #[inline]
    pub fn screen_rect(&self, screen_size: Vec2) -> Rect {
        self.screen_rect_with_grid(screen_size, 16.0)
    }

    /// Computes the element bounding box on screen snapped to a specific grid step.
    #[inline]
    pub fn screen_rect_with_grid(&self, screen_size: Vec2, grid_step: f32) -> Rect {
        let base = get_module_base_size(&self.id);
        let aspect = (base.x / base.y.max(1.0)).max(0.1);

        let raw_w = (self.rel_w * screen_size.x).clamp(base.x * 0.4, screen_size.x * 0.95);
        let w = if grid_step > 0.0 {
            snap_val_f32(raw_w, grid_step).max(base.x * 0.4)
        } else {
            raw_w
        };
        let h = w / aspect;

        let max_x = (screen_size.x - w).max(0.0);
        let max_y = (screen_size.y - h).max(0.0);

        let raw_x = (self.rel_x * screen_size.x).clamp(0.0, max_x);
        let raw_y = (self.rel_y * screen_size.y).clamp(0.0, max_y);

        let x = if grid_step > 0.0 {
            snap_val_f32(raw_x, grid_step).clamp(0.0, max_x)
        } else {
            raw_x
        };
        let y = if grid_step > 0.0 {
            snap_val_f32(raw_y, grid_step).clamp(0.0, max_y)
        } else {
            raw_y
        };

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

        let base = get_module_base_size(&self.id);
        let aspect = (base.x / base.y.max(1.0)).max(0.1);
        let w = (self.rel_w * screen_size.x).clamp(base.x * 0.4, screen_size.x * 0.95);
        let h = w / aspect;

        let snapped_x = snap_val_f32(target_min.x, grid_step);
        let snapped_y = snap_val_f32(target_min.y, grid_step);

        let max_x = (screen_size.x - w).max(0.0);
        let max_y = (screen_size.y - h).max(0.0);

        let clamped_x = snapped_x.clamp(0.0, max_x);
        let clamped_y = snapped_y.clamp(0.0, max_y);

        self.rel_x = (clamped_x / screen_size.x).clamp(0.0, 1.0);
        self.rel_y = (clamped_y / screen_size.y).clamp(0.0, 1.0);
    }

    /// Updates dimensions from resized screen bounds, scaling width and height together proportionally.
    pub fn set_from_screen_size(
        &mut self,
        target_size: Vec2,
        screen_size: Vec2,
        grid_step: f32,
    ) {
        if screen_size.x <= 0.0 || screen_size.y <= 0.0 {
            return;
        }

        let base = get_module_base_size(&self.id);
        let aspect = (base.x / base.y.max(1.0)).max(0.1);

        let min_w = base.x * 0.5;
        let max_w = (screen_size.x - self.rel_x * screen_size.x).max(min_w);
        let snapped_w = snap_val_f32(target_size.x, grid_step).clamp(min_w, max_w);

        self.rel_w = (snapped_w / screen_size.x).clamp(0.01, 1.0);
        self.rel_h = (snapped_w / aspect) / screen_size.y;
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
