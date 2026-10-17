use glam::DVec3;

/// 3D Axis-Aligned Bounding Box ported precisely from MCP 1.8.9 (`net.minecraft.util.AxisAlignedBB`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl AABB {
    /// Creates a new AABB from two corner points, ensuring mins <= maxs.
    #[inline]
    pub fn new(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> Self {
        Self {
            min_x: x1.min(x2),
            min_y: y1.min(y2),
            min_z: z1.min(z2),
            max_x: x1.max(x2),
            max_y: y1.max(y2),
            max_z: z1.max(z2),
        }
    }

    /// Creates an AABB centered at (x, z) on the bottom plane at y, with given width and height.
    #[inline]
    pub fn from_point_size(x: f64, y: f64, z: f64, width: f64, height: f64) -> Self {
        let half_w = width * 0.5;
        Self {
            min_x: x - half_w,
            min_y: y,
            min_z: z - half_w,
            max_x: x + half_w,
            max_y: y + height,
            max_z: z + half_w,
        }
    }

    /// Expands bounding box in the direction of vector (x, y, z), matching `AxisAlignedBB.addCoord`.
    #[inline]
    pub fn add_coord(&self, x: f64, y: f64, z: f64) -> Self {
        let mut min_x = self.min_x;
        let mut min_y = self.min_y;
        let mut min_z = self.min_z;
        let mut max_x = self.max_x;
        let mut max_y = self.max_y;
        let mut max_z = self.max_z;

        if x < 0.0 {
            min_x += x;
        } else if x > 0.0 {
            max_x += x;
        }

        if y < 0.0 {
            min_y += y;
        } else if y > 0.0 {
            max_y += y;
        }

        if z < 0.0 {
            min_z += z;
        } else if z > 0.0 {
            max_z += z;
        }

        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    /// Expands bounding box outwards on all sides by (x, y, z).
    #[inline]
    pub fn expand(&self, x: f64, y: f64, z: f64) -> Self {
        Self {
            min_x: self.min_x - x,
            min_y: self.min_y - y,
            min_z: self.min_z - z,
            max_x: self.max_x + x,
            max_y: self.max_y + y,
            max_z: self.max_z + z,
        }
    }

    /// Contracts bounding box inwards on all sides by (x, y, z).
    #[inline]
    pub fn contract(&self, x: f64, y: f64, z: f64) -> Self {
        Self {
            min_x: self.min_x + x,
            min_y: self.min_y + y,
            min_z: self.min_z + z,
            max_x: self.max_x - x,
            max_y: self.max_y - y,
            max_z: self.max_z - z,
        }
    }

    /// Offsets the bounding box by (x, y, z).
    #[inline]
    pub fn offset(&self, x: f64, y: f64, z: f64) -> Self {
        Self {
            min_x: self.min_x + x,
            min_y: self.min_y + y,
            min_z: self.min_z + z,
            max_x: self.max_x + x,
            max_y: self.max_y + y,
            max_z: self.max_z + z,
        }
    }

    /// Checks if this AABB intersects with another AABB.
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        other.max_x > self.min_x
            && other.min_x < self.max_x
            && other.max_y > self.min_y
            && other.min_y < self.max_y
            && other.max_z > self.min_z
            && other.min_z < self.max_z
    }

    /// Calculates X offset collision against `other`, matching `AxisAlignedBB.calculateXOffset`.
    #[inline]
    pub fn calculate_x_offset(&self, other: &Self, mut offset_x: f64) -> f64 {
        if other.max_y > self.min_y
            && other.min_y < self.max_y
            && other.max_z > self.min_z
            && other.min_z < self.max_z
        {
            if offset_x > 0.0 && other.max_x <= self.min_x {
                let d1 = self.min_x - other.max_x;
                if d1 < offset_x {
                    offset_x = d1;
                }
            } else if offset_x < 0.0 && other.min_x >= self.max_x {
                let d0 = self.max_x - other.min_x;
                if d0 > offset_x {
                    offset_x = d0;
                }
            }
        }
        offset_x
    }

    /// Calculates Y offset collision against `other`, matching `AxisAlignedBB.calculateYOffset`.
    #[inline]
    pub fn calculate_y_offset(&self, other: &Self, mut offset_y: f64) -> f64 {
        if other.max_x > self.min_x
            && other.min_x < self.max_x
            && other.max_z > self.min_z
            && other.min_z < self.max_z
        {
            if offset_y > 0.0 && other.max_y <= self.min_y {
                let d1 = self.min_y - other.max_y;
                if d1 < offset_y {
                    offset_y = d1;
                }
            } else if offset_y < 0.0 && other.min_y >= self.max_y {
                let d0 = self.max_y - other.min_y;
                if d0 > offset_y {
                    offset_y = d0;
                }
            }
        }
        offset_y
    }

    /// Calculates Z offset collision against `other`, matching `AxisAlignedBB.calculateZOffset`.
    #[inline]
    pub fn calculate_z_offset(&self, other: &Self, mut offset_z: f64) -> f64 {
        if other.max_x > self.min_x
            && other.min_x < self.max_x
            && other.max_y > self.min_y
            && other.min_y < self.max_y
        {
            if offset_z > 0.0 && other.max_z <= self.min_z {
                let d1 = self.min_z - other.max_z;
                if d1 < offset_z {
                    offset_z = d1;
                }
            } else if offset_z < 0.0 && other.min_z >= self.max_z {
                let d0 = self.max_z - other.min_z;
                if d0 > offset_z {
                    offset_z = d0;
                }
            }
        }
        offset_z
    }

    /// Tests ray intersection using slab method. Returns distance to entry and hit face normal.
    pub fn ray_trace(&self, origin: DVec3, dir: DVec3, max_dist: f64) -> Option<(f64, DVec3)> {
        let mut t_min = 0.0f64;
        let mut t_max = max_dist;
        let mut hit_normal = DVec3::ZERO;

        for i in 0..3 {
            let (min_val, max_val, orig, d, normal_neg, normal_pos) = match i {
                0 => (self.min_x, self.max_x, origin.x, dir.x, DVec3::NEG_X, DVec3::X),
                1 => (self.min_y, self.max_y, origin.y, dir.y, DVec3::NEG_Y, DVec3::Y),
                _ => (self.min_z, self.max_z, origin.z, dir.z, DVec3::NEG_Z, DVec3::Z),
            };

            if d.abs() < 1e-9 {
                if orig < min_val || orig > max_val {
                    return None;
                }
            } else {
                let inv_d = 1.0 / d;
                let mut t1 = (min_val - orig) * inv_d;
                let mut t2 = (max_val - orig) * inv_d;
                let mut n = normal_neg;

                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                    n = normal_pos;
                }

                if t1 > t_min {
                    t_min = t1;
                    hit_normal = n;
                }
                t_max = t_max.min(t2);

                if t_min > t_max {
                    return None;
                }
            }
        }

        if t_min <= max_dist && t_min >= 0.0 {
            Some((t_min, hit_normal))
        } else {
            None
        }
    }
}
