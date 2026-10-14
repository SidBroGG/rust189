use glam::{Mat4, Vec3};

/// First-person and free-flight 3D perspective camera.
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // Degrees
    pub pitch: f32, // Degrees
    pub fov: f32,   // Degrees (vertical FOV)
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 64.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 70.0,
            aspect: 16.0 / 9.0,
            near: 0.05,
            far: 512.0,
        }
    }
}

impl Camera {
    pub fn new(position: Vec3, yaw: f32, pitch: f32, fov: f32, aspect: f32) -> Self {
        Self {
            position,
            yaw,
            pitch,
            fov,
            aspect,
            near: 0.05,
            far: 512.0,
        }
    }

    /// Computes the forward directional vector from yaw and pitch.
    pub fn forward(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        let x = -yaw_rad.sin() * pitch_rad.cos();
        let y = -pitch_rad.sin();
        let z = yaw_rad.cos() * pitch_rad.cos();

        Vec3::new(x, y, z).normalize()
    }

    /// Computes the right directional vector perpendicular to forward and world Up.
    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    /// Computes the right-handed view matrix.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.forward(), Vec3::Y)
    }

    /// Computes the right-handed perspective projection matrix for WebGPU depth range [0, 1].
    pub fn proj_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), self.aspect.max(0.001), self.near, self.far)
    }

    /// Computes combined View-Projection matrix.
    pub fn view_proj_matrix(&self) -> Mat4 {
        self.proj_matrix() * self.view_matrix()
    }

    /// Updates camera rotation by mouse delta.
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw = (self.yaw + delta_yaw).rem_euclid(360.0);
        self.pitch = (self.pitch + delta_pitch).clamp(-89.9, 89.9);
    }

    /// Translates camera in forward/backward direction on the horizontal plane.
    pub fn move_horizontal(&mut self, forward: f32, strafe: f32) {
        let yaw_rad = self.yaw.to_radians();
        let forward_dir = Vec3::new(-yaw_rad.sin(), 0.0, yaw_rad.cos()).normalize();
        let right_dir = forward_dir.cross(Vec3::Y).normalize();

        self.position += forward_dir * forward + right_dir * strafe;
    }

    /// Translates camera along global Y axis.
    pub fn move_vertical(&mut self, up: f32) {
        self.position.y += up;
    }
}
