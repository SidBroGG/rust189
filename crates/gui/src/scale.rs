/// Calculates resolution scaling matching Minecraft 1.8.9's `ScaledResolution.java`
/// with support for fine-grained fractional scaling factors (e.g. 1.25x, 1.5x).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaledResolution {
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f32,
    pub scaled_width: f32,
    pub scaled_height: f32,
}

impl ScaledResolution {
    /// Creates a new `ScaledResolution`.
    ///
    /// * `gui_scale`: 0.0 = Auto (standard Minecraft formula based on width/height),
    ///                otherwise exact custom scale factor (e.g. 0.75, 1.0, 1.25, 1.5, 2.0).
    pub fn new(physical_width: u32, physical_height: u32, gui_scale: f32) -> Self {
        let width = physical_width.max(1);
        let height = physical_height.max(1);

        let scale_factor = if gui_scale <= 0.05 {
            // Auto scale mode: standard Minecraft 1.8.9 logic
            let mut auto_scale = 1;
            while (width / (auto_scale + 1)) >= 320 && (height / (auto_scale + 1)) >= 240 {
                auto_scale += 1;
            }
            auto_scale as f32
        } else {
            // Precise custom scale clamped between 0.50x and 4.00x
            gui_scale.clamp(0.5, 4.0)
        };

        let scaled_width = width as f32 / scale_factor;
        let scaled_height = height as f32 / scale_factor;

        Self {
            physical_width: width,
            physical_height: height,
            scale_factor,
            scaled_width,
            scaled_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaled_resolution_auto() {
        // At 1920x1080, auto scale factor should be 4.0
        let res = ScaledResolution::new(1920, 1080, 0.0);
        assert_eq!(res.scale_factor, 4.0);
        assert_eq!(res.scaled_width, 480.0);
        assert_eq!(res.scaled_height, 270.0);
    }

    #[test]
    fn test_scaled_resolution_explicit_scale() {
        // Explicit 2.0x scale
        let res = ScaledResolution::new(1920, 1080, 2.0);
        assert_eq!(res.scale_factor, 2.0);
        assert_eq!(res.scaled_width, 960.0);
        assert_eq!(res.scaled_height, 540.0);

        // Explicit 1.25x precise scale
        let res_frac = ScaledResolution::new(1920, 1080, 1.25);
        assert_eq!(res_frac.scale_factor, 1.25);
        assert_eq!(res_frac.scaled_width, 1920.0 / 1.25);
        assert_eq!(res_frac.scaled_height, 1080.0 / 1.25);

        // Explicit 1.0x scale
        let res1 = ScaledResolution::new(1920, 1080, 1.0);
        assert_eq!(res1.scale_factor, 1.0);
        assert_eq!(res1.scaled_width, 1920.0);
    }

    #[test]
    fn test_scaled_resolution_small_window() {
        // Small 640x480 window: auto scale factor can be at most 2.0
        let res = ScaledResolution::new(640, 480, 0.0);
        assert_eq!(res.scale_factor, 2.0);

        // Window smaller than 640x480 (e.g. 320x240) forces auto scale factor 1.0
        let res_tiny = ScaledResolution::new(320, 240, 0.0);
        assert_eq!(res_tiny.scale_factor, 1.0);
    }
}
