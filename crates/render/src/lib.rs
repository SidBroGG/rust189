pub mod atlas;
pub mod block_model;
pub mod camera;
pub mod chunk_manager;
pub mod context;
pub mod entity_renderer;
pub mod frustum;
pub mod interaction_renderer;
pub mod mesher;
pub mod models;
pub mod occlusion;
pub mod terrain_renderer;
pub mod vertex;

pub use atlas::{AnimatedTexture, TextureAtlas, TextureRegion};
pub use block_model::{BakedModel, BakedModelRegistry, BakedQuad, ModelBakery};
pub use camera::Camera;
pub use chunk_manager::{ChunkMeshManager, GpuSectionMesh, NeighborhoodSnapshot, RawSectionMesh, SectionKey};
pub use context::GpuContext;
pub use entity_renderer::{EntityRenderer, EntityUniforms, EntityVertex};
pub use frustum::{Frustum, Plane};
pub use interaction_renderer::{InteractionRenderer, LineVertex, Particle, TexturedVertex};
pub use mesher::{BlockWorldAccessor, ChunkMesher, SectionMeshData};
pub use occlusion::{SectionVisibility, VisGraph};
pub use terrain_renderer::{TerrainRenderer, TerrainUniforms};
pub use vertex::{BlockFace, BlockVertex, SectionInstance, UnpackedBlockVertex};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_crate_types() {
        let desc = wgpu::InstanceDescriptor::default();
        assert_eq!(desc.flags, wgpu::InstanceFlags::default());
    }

    #[test]
    fn test_block_vertex_layout() {
        let layout = BlockVertex::desc();
        assert_eq!(layout.array_stride, 16);
        assert_eq!(layout.attributes.len(), 1);
    }

    #[test]
    fn test_camera_matrices() {
        let cam = Camera::default();
        let view = cam.view_matrix();
        let proj = cam.proj_matrix();
        let vp = cam.view_proj_matrix();
        assert_ne!(view, glam::Mat4::ZERO);
        assert_ne!(proj, glam::Mat4::ZERO);
        assert_eq!(vp, proj * view);
    }

    #[test]
    fn test_frustum_intersection() {
        let cam = Camera::default();
        let frustum = Frustum::from_view_proj(cam.view_proj_matrix());
        // Point in front of camera
        let forward_box_min = cam.position + cam.forward() * 10.0 - glam::Vec3::splat(1.0);
        let forward_box_max = cam.position + cam.forward() * 10.0 + glam::Vec3::splat(1.0);
        assert!(frustum.intersects_aabb(forward_box_min, forward_box_max));

        // Point far behind camera
        let behind_box_min = cam.position - cam.forward() * 100.0 - glam::Vec3::splat(1.0);
        let behind_box_max = cam.position - cam.forward() * 100.0 + glam::Vec3::splat(1.0);
        assert!(!frustum.intersects_aabb(behind_box_min, behind_box_max));
    }

    #[test]
    fn test_entity_vertex_layout() {
        let layout = EntityVertex::desc();
        assert_eq!(layout.array_stride, 48); // 3*4 + 2*4 + 3*4 + 4*4 = 12 + 8 + 12 + 16 = 48 bytes
        assert_eq!(layout.attributes.len(), 4);
    }
}
