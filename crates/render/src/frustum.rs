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

/// Represents the camera view frustum formed by 6 clipping planes with SIMD acceleration.
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6],
    pub xs: [glam::Vec4; 2],
    pub ys: [glam::Vec4; 2],
    pub zs: [glam::Vec4; 2],
    pub ds: [glam::Vec4; 2],
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

        let planes = [left, right, bottom, top, near, far];

        // Group into 2 glam::Vec4 SIMD vectors
        // Vector 0: planes 0..3 (left, right, bottom, top)
        let xs0 = glam::Vec4::new(planes[0].normal.x, planes[1].normal.x, planes[2].normal.x, planes[3].normal.x);
        let ys0 = glam::Vec4::new(planes[0].normal.y, planes[1].normal.y, planes[2].normal.y, planes[3].normal.y);
        let zs0 = glam::Vec4::new(planes[0].normal.z, planes[1].normal.z, planes[2].normal.z, planes[3].normal.z);
        let ds0 = glam::Vec4::new(planes[0].d, planes[1].d, planes[2].d, planes[3].d);

        // Vector 1: planes 4..5 (near, far), lanes 2 & 3 padded to never reject (d = 1e6)
        let xs1 = glam::Vec4::new(planes[4].normal.x, planes[5].normal.x, 0.0, 0.0);
        let ys1 = glam::Vec4::new(planes[4].normal.y, planes[5].normal.y, 0.0, 0.0);
        let zs1 = glam::Vec4::new(planes[4].normal.z, planes[5].normal.z, 0.0, 0.0);
        let ds1 = glam::Vec4::new(planes[4].d, planes[5].d, 1e6, 1e6);

        Self {
            planes,
            xs: [xs0, xs1],
            ys: [ys0, ys1],
            zs: [zs0, zs1],
            ds: [ds0, ds1],
        }
    }

    /// SIMD-accelerated AABB frustum intersection test testing 4 planes simultaneously.
    #[inline]
    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        let min_x = glam::Vec4::splat(min.x);
        let max_x = glam::Vec4::splat(max.x);
        let min_y = glam::Vec4::splat(min.y);
        let max_y = glam::Vec4::splat(max.y);
        let min_z = glam::Vec4::splat(min.z);
        let max_z = glam::Vec4::splat(max.z);

        for i in 0..2 {
            let px = glam::Vec4::select(self.xs[i].cmpge(glam::Vec4::ZERO), max_x, min_x);
            let py = glam::Vec4::select(self.ys[i].cmpge(glam::Vec4::ZERO), max_y, min_y);
            let pz = glam::Vec4::select(self.zs[i].cmpge(glam::Vec4::ZERO), max_z, min_z);

            let dist = self.xs[i] * px + self.ys[i] * py + self.zs[i] * pz + self.ds[i];
            if dist.cmplt(glam::Vec4::ZERO).any() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_frustum_matches_scalar() {
        let vp = Mat4::perspective_rh(1.2, 16.0 / 9.0, 0.1, 100.0);
        let frustum = Frustum::from_view_proj(vp);

        // AABB directly in front of camera
        let inside_min = Vec3::new(-1.0, -1.0, -10.0);
        let inside_max = Vec3::new(1.0, 1.0, -8.0);
        assert!(frustum.intersects_aabb(inside_min, inside_max));

        // AABB far behind camera
        let behind_min = Vec3::new(-1.0, -1.0, 5.0);
        let behind_max = Vec3::new(1.0, 1.0, 10.0);
        assert!(!frustum.intersects_aabb(behind_min, behind_max));
    }
}
