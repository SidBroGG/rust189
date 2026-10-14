use glam::{Mat4, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub d: f32,
}

impl Plane {
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        let len = (x * x + y * y + z * z).sqrt();
        if len > 1e-6 {
            Self {
                normal: Vec3::new(x / len, y / len, z / len),
                d: w / len,
            }
        } else {
            Self {
                normal: Vec3::ZERO,
                d: 0.0,
            }
        }
    }

    #[inline]
    pub fn distance_to_point(&self, p: Vec3) -> f32 {
        self.normal.dot(p) + self.d
    }
}

/// Represents the camera view frustum formed by 6 clipping planes.
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Extracts the 6 frustum clipping planes from a view-projection matrix (WebGPU depth range [0, 1]).
    pub fn from_view_proj(vp: Mat4) -> Self {
        let m = vp.to_cols_array_2d();

        // Row vectors of the matrix
        let row0 = [m[0][0], m[1][0], m[2][0], m[3][0]];
        let row1 = [m[0][1], m[1][1], m[2][1], m[3][1]];
        let row2 = [m[0][2], m[1][2], m[2][2], m[3][2]];
        let row3 = [m[0][3], m[1][3], m[2][3], m[3][3]];

        // Left plane: row3 + row0
        let left = Plane::new(
            row3[0] + row0[0],
            row3[1] + row0[1],
            row3[2] + row0[2],
            row3[3] + row0[3],
        );

        // Right plane: row3 - row0
        let right = Plane::new(
            row3[0] - row0[0],
            row3[1] - row0[1],
            row3[2] - row0[2],
            row3[3] - row0[3],
        );

        // Bottom plane: row3 + row1
        let bottom = Plane::new(
            row3[0] + row1[0],
            row3[1] + row1[1],
            row3[2] + row1[2],
            row3[3] + row1[3],
        );

        // Top plane: row3 - row1
        let top = Plane::new(
            row3[0] - row1[0],
            row3[1] - row1[1],
            row3[2] - row1[2],
            row3[3] - row1[3],
        );

        // Near plane (WebGPU [0, 1] depth range): row2
        let near = Plane::new(row2[0], row2[1], row2[2], row2[3]);

        // Far plane: row3 - row2
        let far = Plane::new(
            row3[0] - row2[0],
            row3[1] - row2[1],
            row3[2] - row2[2],
            row3[3] - row2[3],
        );

        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }

    /// Checks whether an Axis-Aligned Bounding Box (AABB) is inside or intersects the frustum.
    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            // Find p-vertex (furthest along normal)
            let p = Vec3::new(
                if plane.normal.x >= 0.0 { max.x } else { min.x },
                if plane.normal.y >= 0.0 { max.y } else { min.y },
                if plane.normal.z >= 0.0 { max.z } else { min.z },
            );

            if plane.distance_to_point(p) < 0.0 {
                return false;
            }
        }
        true
    }
}
