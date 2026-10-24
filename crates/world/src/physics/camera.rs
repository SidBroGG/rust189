use crate::physics::movement::PlayerMovement;
use crate::world::World;
use glam::{DVec3, Vec3};
use std::f64::consts::PI;

/// Smooths mouse movement for cinematic camera (MCP `net.minecraft.util.MouseFilter`).
#[derive(Debug, Default, Clone)]
pub struct MouseFilter {
    target: f32,
    remaining: f32,
    acc: f32,
}

impl MouseFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn smooth(&mut self, mut value: f32, weight: f32) -> f32 {
        self.target += value;
        value = (self.target - self.remaining) * weight;
        self.acc += (value - self.acc) * 0.5;
        if (value > 0.0 && value > self.acc) || (value < 0.0 && value < self.acc) {
            value = self.acc;
        }
        self.remaining += value;
        value
    }

    pub fn reset(&mut self) {
        self.target = 0.0;
        self.remaining = 0.0;
        self.acc = 0.0;
    }
}

/// Camera perspective modes corresponding to F5 key presses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraPerspective {
    #[default]
    FirstPerson = 0,
    ThirdPersonRear = 1,
    ThirdPersonFront = 2,
}

impl CameraPerspective {
    pub fn cycle(self) -> Self {
        match self {
            Self::FirstPerson => Self::ThirdPersonRear,
            Self::ThirdPersonRear => Self::ThirdPersonFront,
            Self::ThirdPersonFront => Self::FirstPerson,
        }
    }
}

/// Calculated camera pose for rendering.
#[derive(Debug, Clone, Copy)]
pub struct CameraPose {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub bob_x: f32,
    pub bob_y: f32,
    pub bob_pitch: f32,
}

/// Controller managing camera view bobbing, mouse look, cinematic smoothing,
/// and third-person perspective with wall-clipping collision detection.
#[derive(Debug, Clone)]
pub struct CameraController {
    pub perspective: CameraPerspective,
    pub cinematic_camera: bool,
    pub filter_x: MouseFilter,
    pub filter_y: MouseFilter,
    pub smooth_cam_yaw: f32,
    pub smooth_cam_pitch: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            perspective: CameraPerspective::FirstPerson,
            cinematic_camera: false,
            filter_x: MouseFilter::new(),
            filter_y: MouseFilter::new(),
            smooth_cam_yaw: 0.0,
            smooth_cam_pitch: 0.0,
        }
    }
}

impl CameraController {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rotates the player look angles based on raw mouse delta, sensitivity, and invert settings.
    pub fn handle_mouse_input(
        &mut self,
        player: &mut PlayerMovement,
        dx: f32,
        dy: f32,
        sensitivity: f32,
        invert_y: bool,
    ) {
        let f = sensitivity * 0.6 + 0.2;
        let f1 = f * f * f * 8.0;
        let mut delta_x = dx * f1;
        // In winit, mouse motion up gives dy < 0, whereas Minecraft (LWJGL) gives positive deltaY.
        // Inverting dy aligns it so moving mouse forward tilts pitch upward towards the sky.
        let mut delta_y = -dy * f1;

        if invert_y {
            delta_y = -delta_y;
        }

        if self.cinematic_camera {
            self.smooth_cam_yaw += delta_x;
            self.smooth_cam_pitch += delta_y;
            delta_x = self.filter_x.smooth(self.smooth_cam_yaw, 0.05 * f1);
            delta_y = self.filter_y.smooth(self.smooth_cam_pitch, 0.05 * f1);
            self.smooth_cam_yaw = 0.0;
            self.smooth_cam_pitch = 0.0;
        } else {
            self.smooth_cam_yaw = 0.0;
            self.smooth_cam_pitch = 0.0;
            self.filter_x.reset();
            self.filter_y.reset();
        }

        // Port of MCP `Entity.setAngles` (lines 389-398):
        // Advance both current and previous rotation angles by the mouse delta
        // to eliminate 20-Hz partial_ticks frame-rate interpolation jitter!
        let old_pitch = player.pitch;
        let old_yaw = player.yaw;
        player.yaw += delta_x * 0.15;
        player.pitch = (player.pitch - delta_y * 0.15).clamp(-90.0, 90.0);
        player.prev_pitch += player.pitch - old_pitch;
        player.prev_yaw += player.yaw - old_yaw;
    }

    /// Computes the final camera position and orientation including view bobbing and third-person raycasts.
    pub fn compute_camera_pose(
        &self,
        world: &World,
        player: &PlayerMovement,
        partial_ticks: f32,
        view_bobbing_enabled: bool,
    ) -> CameraPose {
        let eye_pos = player.get_interpolated_eye_pos(partial_ticks);

        // Interpolate yaw and pitch (wrapping yaw delta across 360-degree boundary)
        let pt = partial_ticks;
        let diff = crate::entity::wrap_degrees(player.yaw - player.prev_yaw);
        let yaw = player.prev_yaw + diff * pt;
        let pitch = player.prev_pitch + (player.pitch - player.prev_pitch) * pt;

        // View bobbing calculation (MCP `EntityRenderer.setupViewBobbing`)
        let mut bob_x = 0.0f32;
        let mut bob_y = 0.0f32;
        let mut bob_roll = 0.0f32;
        let mut bob_pitch = 0.0f32;

        if view_bobbing_enabled {
            let f = player.distance_walked_modified - player.prev_distance_walked_modified;
            let f1 = -(player.distance_walked_modified + f * partial_ticks);
            let f2 = player.prev_camera_yaw + (player.camera_yaw - player.prev_camera_yaw) * partial_ticks;
            let f3 = player.prev_camera_pitch + (player.camera_pitch - player.prev_camera_pitch) * partial_ticks;

            let pi_f = std::f32::consts::PI;
            bob_x = (f1 * pi_f).sin() * f2 * 0.5;
            bob_y = -((f1 * pi_f).cos().abs()) * f2;
            bob_roll = (f1 * pi_f).sin() * f2 * 3.0;
            bob_pitch = ((f1 * pi_f - 0.2).cos().abs()) * f2 * 5.0 + f3;
        }

        // Camera positioning based on perspective mode (MCP `EntityRenderer.orientCamera`)
        match self.perspective {
            CameraPerspective::FirstPerson => CameraPose {
                position: Vec3::new(
                    eye_pos.x as f32,
                    eye_pos.y as f32,
                    eye_pos.z as f32,
                ),
                yaw,
                pitch,
                roll: bob_roll,
                bob_x,
                bob_y,
                bob_pitch,
            },
            CameraPerspective::ThirdPersonRear | CameraPerspective::ThirdPersonFront => {
                let is_front = self.perspective == CameraPerspective::ThirdPersonFront;
                let target_pitch = if is_front { pitch + 180.0 } else { pitch };

                let max_third_person_dist = 4.0f64;
                let mut clamped_dist = max_third_person_dist;

                let rad_yaw = (yaw as f64) * (PI / 180.0);
                let rad_pitch = (target_pitch as f64) * (PI / 180.0);

                let d4 = -rad_yaw.sin() * rad_pitch.cos() * max_third_person_dist;
                let d5 = rad_yaw.cos() * rad_pitch.cos() * max_third_person_dist;
                let d6 = -rad_pitch.sin() * max_third_person_dist;

                // 8-corner raycasting for obstacle clipping against world blocks (MCP `EntityRenderer.orientCamera`)
                for i in 0..8 {
                    let ox = if (i & 1) != 0 { 0.1 } else { -0.1 };
                    let oy = if (i & 2) != 0 { 0.1 } else { -0.1 };
                    let oz = if (i & 4) != 0 { 0.1 } else { -0.1 };
                    let corner_start = eye_pos + DVec3::new(ox, oy, oz);
                    let corner_end = corner_start - DVec3::new(d4, d6, d5);

                    if let Some(hit) = world.ray_trace_blocks(corner_start, corner_end) {
                        let hit_dist = (hit.hit_pos - corner_start).length();
                        if hit_dist < clamped_dist {
                            clamped_dist = hit_dist;
                        }
                    }
                }

                clamped_dist = clamped_dist.max(0.2);

                let actual_d4 = -rad_yaw.sin() * rad_pitch.cos() * clamped_dist;
                let actual_d5 = rad_yaw.cos() * rad_pitch.cos() * clamped_dist;
                let actual_d6 = -rad_pitch.sin() * clamped_dist;

                let cam_pos = eye_pos - DVec3::new(actual_d4, actual_d6, actual_d5);

                CameraPose {
                    position: Vec3::new(cam_pos.x as f32, cam_pos.y as f32, cam_pos.z as f32),
                    yaw: if is_front { yaw + 180.0 } else { yaw },
                    pitch: if is_front { -pitch } else { pitch },
                    roll: 0.0,
                    bob_x: 0.0,
                    bob_y: 0.0,
                    bob_pitch: 0.0,
                }
            }
        }
    }
}
