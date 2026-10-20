pub mod boss;
pub mod hostile;
pub mod layers;
pub mod peaceful;
pub mod vehicles;

pub use boss::*;
pub use hostile::*;
pub use layers::*;
pub use peaceful::*;
pub use vehicles::*;

use crate::entity_renderer::{add_box_geom, EntityVertex};
use glam::{Mat4, Vec3};
use world::entity::TrackedEntity;

/// Represents a single cuboid box with textures inside a ModelPart (MCP ModelBox).
#[derive(Clone, Debug)]
pub struct ModelBox {
    pub origin: Vec3,
    pub size: Vec3,
    pub tex_u: f32,
    pub tex_v: f32,
    pub expand: f32,
    pub mirror: bool,
}

/// Represents a hierarchical skeletal model part / bone (MCP ModelRenderer).
#[derive(Clone, Debug)]
pub struct ModelPart {
    pub rotation_point: Vec3,
    pub rotation: Vec3, // (rx, ry, rz) in radians
    pub offset: Vec3,
    pub boxes: Vec<ModelBox>,
    pub children: Vec<ModelPart>,
    pub texture_size: (f32, f32),
    pub mirror: bool,
    pub hidden: bool,
}

impl ModelPart {
    pub fn new(_u: f32, _v: f32, tex_w: f32, tex_h: f32) -> Self {
        Self {
            rotation_point: Vec3::ZERO,
            rotation: Vec3::ZERO,
            offset: Vec3::ZERO,
            boxes: Vec::new(),
            children: Vec::new(),
            texture_size: (tex_w, tex_h),
            mirror: false,
            hidden: false,
        }
    }

    pub fn set_rotation_point(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.rotation_point = Vec3::new(x, y, z);
        self
    }

    pub fn set_rotation(&mut self, rx: f32, ry: f32, rz: f32) -> &mut Self {
        self.rotation = Vec3::new(rx, ry, rz);
        self
    }

    pub fn add_box(
        &mut self,
        tex_u: f32,
        tex_v: f32,
        ox: f32,
        oy: f32,
        oz: f32,
        w: f32,
        h: f32,
        d: f32,
        expand: f32,
    ) -> &mut Self {
        self.boxes.push(ModelBox {
            origin: Vec3::new(ox, oy, oz),
            size: Vec3::new(w, h, d),
            tex_u,
            tex_v,
            expand,
            mirror: self.mirror,
        });
        self
    }

    pub fn add_child(&mut self, child: ModelPart) -> &mut Self {
        self.children.push(child);
        self
    }

    /// Recursively computes transformation and appends geometry to vertex/index buffers.
    pub fn render(
        &self,
        parent_matrix: Mat4,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        color: [f32; 4],
    ) {
        if self.hidden {
            return;
        }

        let local_matrix = parent_matrix
            * Mat4::from_translation(self.rotation_point * (1.0 / 16.0))
            * Mat4::from_rotation_z(self.rotation.z)
            * Mat4::from_rotation_y(self.rotation.y)
            * Mat4::from_rotation_x(self.rotation.x)
            * Mat4::from_translation(self.offset * (1.0 / 16.0));

        let (tw, th) = self.texture_size;
        for b in &self.boxes {
            add_box_geom(
                vertices,
                indices,
                local_matrix,
                b.tex_u,
                b.tex_v,
                b.origin,
                b.size,
                b.expand,
                b.mirror,
                tw,
                th,
                color,
            );
        }

        for child in &self.children {
            child.render(local_matrix, vertices, indices, color);
        }
    }
}

/// Vanilla root transformation for entities with preRenderCallback scale:
/// Translate(pos) * RotateY(180 - body_yaw) * Scale(-scale.x, -scale.y, scale.z) * Translate(0, -1.5078125, 0)
pub fn root_entity_matrix_scaled(entity: &TrackedEntity, partial_ticks: f32, scale: Vec3) -> Mat4 {
    let pos = entity.interpolated_pos(partial_ticks);
    let body_yaw = entity.interpolated_render_yaw_offset(partial_ticks);

    let mut m = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - body_yaw).to_radians())
        * Mat4::from_scale(Vec3::new(-scale.x, -scale.y, scale.z))
        * Mat4::from_translation(Vec3::new(0.0, -1.5078125, 0.0));

    if entity.death_time > 0 {
        let f = (((entity.death_time as f32 + partial_ticks - 1.0) / 20.0 * 1.6).sqrt()).min(1.0);
        m *= Mat4::from_rotation_z(f * (std::f32::consts::PI / 2.0));
    }

    if entity.is_sneaking {
        m *= Mat4::from_translation(Vec3::new(0.0, 0.2, 0.0));
    }

    m
}

/// Standard vanilla root transformation for entities:
/// Translate(interpolated_pos) * RotateY(180 - body_yaw) * Scale(-1, -1, 1) * Translate(0, -1.5078125, 0)
pub fn root_entity_matrix(entity: &TrackedEntity, partial_ticks: f32) -> Mat4 {
    root_entity_matrix_scaled(entity, partial_ticks, Vec3::ONE)
}

