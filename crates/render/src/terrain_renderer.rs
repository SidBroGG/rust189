use crate::atlas::TextureAtlas;
use crate::camera::Camera;
use crate::chunk_manager::GpuSectionMesh;
use crate::vertex::BlockVertex;
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

/// Uniforms buffer uploaded to GPU for terrain rendering (144 bytes, 16-byte aligned).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TerrainUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
    pub fog_color: [f32; 4],
    pub fog_params: [f32; 4],
    pub sun_direction: [f32; 4],
    pub atlas_tile_size: [f32; 4],
}

/// Renders voxel terrain meshes with dual-pass (Opaque/Cutout + Translucent) architecture.
pub struct TerrainRenderer {
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub atlas_texture: wgpu::Texture,
    pub atlas_texture_view: wgpu::TextureView,
    pub atlas_sampler: wgpu::Sampler,
    pub atlas_bgl: wgpu::BindGroupLayout,
    pub atlas_bind_group: wgpu::BindGroup,
    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,
    pub opaque_pipeline: wgpu::RenderPipeline,
    pub translucent_pipeline: wgpu::RenderPipeline,
    pub animations: Vec<crate::atlas::AnimatedTexture>,
    pub atlas_tile_size: [f32; 4],
}

impl TerrainRenderer {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        atlas: &TextureAtlas,
    ) -> Self {
        let tile_w = (16.0 - 0.10) / atlas.width.max(1) as f32;
        let tile_h = (16.0 - 0.10) / atlas.height.max(1) as f32;
        let atlas_tile_size = [tile_w, tile_h, 0.0, 0.0];

        // 1. Create Terrain Uniform Buffer & Bind Group Layout
        let uniforms = TerrainUniforms {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0, 64.0, 0.0, 0.0],
            fog_color: [0.7, 0.8, 1.0, 1.0],
            fog_params: [120.0, 180.0, 0.0, 0.0],
            sun_direction: [0.3, 0.8, 0.5, 0.0],
            atlas_tile_size,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("terrain-uniforms-buf"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain-uniforms-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain-uniforms-bg"),
            layout: &uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // 2. Create Atlas Texture with 5 Mip Levels (0..=4), Sampler & Bind Group
        let mip_levels = Self::generate_mip_levels(atlas.width, atlas.height, &atlas.image_data, 5);

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("terrain-atlas-texture"),
            size: wgpu::Extent3d {
                width: atlas.width,
                height: atlas.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_levels.len() as u32,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for (level, data) in mip_levels.iter().enumerate() {
            let mip_w = (atlas.width >> level).max(1);
            let mip_h = (atlas.height >> level).max(1);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &atlas_texture,
                    mip_level: level as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(mip_w * 4),
                    rows_per_image: Some(mip_h),
                },
                wgpu::Extent3d {
                    width: mip_w,
                    height: mip_h,
                    depth_or_array_layers: 1,
                },
            );
        }

        let atlas_texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("terrain-atlas-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 4.0,
            anisotropy_clamp: 1,
            ..Default::default()
        });

        let atlas_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain-atlas-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain-atlas-bg"),
            layout: &atlas_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        // 3. Create Depth Texture
        let (depth_texture, depth_texture_view) =
            Self::create_depth_texture(device, width.max(1), height.max(1));

        // 4. Create Shader & Pipeline Layout
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain-wgsl-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("terrain.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain-pipeline-layout"),
            bind_group_layouts: &[&uniform_bgl, &atlas_bgl],
            push_constant_ranges: &[],
        });

        // 5. Create Opaque Pipeline (Depth Write = True, Blend = None)
        let opaque_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain-opaque-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[BlockVertex::desc(), crate::vertex::SectionInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Self::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // 6. Create Translucent Pipeline (Depth Write = False, Blend = AlphaBlending)
        let translucent_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain-translucent-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[BlockVertex::desc(), crate::vertex::SectionInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Self::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            uniform_buffer,
            uniform_bind_group,
            atlas_texture,
            atlas_texture_view,
            atlas_sampler,
            atlas_bgl,
            atlas_bind_group,
            depth_texture,
            depth_texture_view,
            opaque_pipeline,
            translucent_pipeline,
            animations: atlas.animations.clone(),
            atlas_tile_size,
        }
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("terrain-depth-texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Resizes the depth texture when the application window is resized.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let (depth_texture, depth_texture_view) =
            Self::create_depth_texture(device, width, height);
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
    }

    /// Updates uniform buffer values for camera and lighting.
    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        camera: &Camera,
        render_distance_blocks: f32,
        time_secs: f32,
        antialiasing: u32,
    ) {
        let view_proj = camera.view_proj_matrix().to_cols_array_2d();
        let sun_dir = Vec3::new(0.3, 0.8, 0.5).normalize();
        let uniforms = TerrainUniforms {
            view_proj,
            camera_pos: [camera.position.x, camera.position.y, camera.position.z, time_secs],
            fog_color: [0.7, 0.8, 1.0, 1.0],
            fog_params: [
                (render_distance_blocks * 0.7).max(16.0),
                render_distance_blocks.max(32.0),
                antialiasing as f32,
                0.0,
            ],
            sun_direction: [sun_dir.x, sun_dir.y, sun_dir.z, 0.0],
            atlas_tile_size: self.atlas_tile_size,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Generates pre-filtered mipmap levels using alpha-weighted box filtering.
    pub fn generate_mip_levels(
        base_width: u32,
        base_height: u32,
        base_data: &[u8],
        max_levels: u32,
    ) -> Vec<Vec<u8>> {
        let mut levels = Vec::with_capacity(max_levels as usize);
        levels.push(base_data.to_vec());

        let mut prev_w = base_width;
        let mut prev_h = base_height;

        for _ in 1..max_levels {
            let curr_w = (prev_w / 2).max(1);
            let curr_h = (prev_h / 2).max(1);
            let prev_data = levels.last().unwrap();
            let mut curr_data = vec![0u8; (curr_w * curr_h * 4) as usize];

            for y in 0..curr_h {
                for x in 0..curr_w {
                    let src_x = (x * 2).min(prev_w - 1);
                    let src_y = (y * 2).min(prev_h - 1);
                    let src_x2 = (src_x + 1).min(prev_w - 1);
                    let src_y2 = (src_y + 1).min(prev_h - 1);

                    let idx00 = ((src_y * prev_w + src_x) * 4) as usize;
                    let idx10 = ((src_y * prev_w + src_x2) * 4) as usize;
                    let idx01 = ((src_y2 * prev_w + src_x) * 4) as usize;
                    let idx11 = ((src_y2 * prev_w + src_x2) * 4) as usize;

                    let p00 = &prev_data[idx00..idx00 + 4];
                    let p10 = &prev_data[idx10..idx10 + 4];
                    let p01 = &prev_data[idx01..idx01 + 4];
                    let p11 = &prev_data[idx11..idx11 + 4];

                    let a00 = p00[3] as u32;
                    let a10 = p10[3] as u32;
                    let a01 = p01[3] as u32;
                    let a11 = p11[3] as u32;
                    let total_a = a00 + a10 + a01 + a11;

                    let dst_idx = ((y * curr_w + x) * 4) as usize;
                    if total_a == 0 {
                        curr_data[dst_idx..dst_idx + 4].copy_from_slice(&[0, 0, 0, 0]);
                    } else {
                        let r = ((p00[0] as u32 * a00
                            + p10[0] as u32 * a10
                            + p01[0] as u32 * a01
                            + p11[0] as u32 * a11)
                            / total_a) as u8;
                        let g = ((p00[1] as u32 * a00
                            + p10[1] as u32 * a10
                            + p01[1] as u32 * a01
                            + p11[1] as u32 * a11)
                            / total_a) as u8;
                        let b = ((p00[2] as u32 * a00
                            + p10[2] as u32 * a10
                            + p01[2] as u32 * a01
                            + p11[2] as u32 * a11)
                            / total_a) as u8;
                        let a = (total_a / 4) as u8;
                        curr_data[dst_idx..dst_idx + 4].copy_from_slice(&[r, g, b, a]);
                    }
                }
            }

            levels.push(curr_data);
            prev_w = curr_w;
            prev_h = curr_h;
        }

        levels
    }

    /// Steps texture animations and writes updated frames (across mip levels) into the GPU texture atlas.
    pub fn update_animations(&mut self, queue: &wgpu::Queue, dt_ms: u64) {
        for anim in &mut self.animations {
            let x = anim.x;
            let y = anim.y;
            let width = anim.width;
            let height = anim.height;
            if let Some(frame_bytes) = anim.update(dt_ms) {
                let tile_mips = Self::generate_mip_levels(width, height, frame_bytes, 5);
                for (level, mdata) in tile_mips.iter().enumerate() {
                    let mip_w = (width >> level).max(1);
                    let mip_h = (height >> level).max(1);
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &self.atlas_texture,
                            mip_level: level as u32,
                            origin: wgpu::Origin3d {
                                x: x >> level,
                                y: y >> level,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        mdata,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(mip_w * 4),
                            rows_per_image: Some(mip_h),
                        },
                        wgpu::Extent3d {
                            width: mip_w,
                            height: mip_h,
                            depth_or_array_layers: 1,
                        },
                    );
                }
            }
        }
    }

    /// Draws all visible opaque section meshes into the active render pass.
    pub fn render_opaque<'a>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'a>,
        visible_opaque: &[&'a GpuSectionMesh],
    ) {
        if visible_opaque.is_empty() {
            return;
        }
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.set_bind_group(1, &self.atlas_bind_group, &[]);
        rpass.set_pipeline(&self.opaque_pipeline);
        for mesh in visible_opaque {
            if let (Some(vbuf), Some(ibuf), Some(inst_buf)) = (&mesh.opaque_vbuf, &mesh.opaque_ibuf, &mesh.instance_buf) {
                rpass.set_vertex_buffer(0, vbuf.slice(..));
                rpass.set_vertex_buffer(1, inst_buf.slice(..));
                rpass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..mesh.opaque_index_count, 0, 0..1);
            }
        }
    }

    /// Draws all visible translucent section meshes into the active render pass.
    pub fn render_translucent<'a>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'a>,
        visible_translucent: &[&'a GpuSectionMesh],
    ) {
        if visible_translucent.is_empty() {
            return;
        }
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.set_bind_group(1, &self.atlas_bind_group, &[]);
        rpass.set_pipeline(&self.translucent_pipeline);
        for mesh in visible_translucent {
            if let (Some(vbuf), Some(ibuf), Some(inst_buf)) = (&mesh.translucent_vbuf, &mesh.translucent_ibuf, &mesh.instance_buf) {
                rpass.set_vertex_buffer(0, vbuf.slice(..));
                rpass.set_vertex_buffer(1, inst_buf.slice(..));
                rpass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..mesh.translucent_index_count, 0, 0..1);
            }
        }
    }

    /// Draws all visible opaque and translucent section meshes into the active render pass.
    pub fn render<'a>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'a>,
        visible_opaque: &[&'a GpuSectionMesh],
        visible_translucent: &[&'a GpuSectionMesh],
    ) {
        self.render_opaque(rpass, visible_opaque);
        self.render_translucent(rpass, visible_translucent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mip_levels_dimensions() {
        let width = 16u32;
        let height = 16u32;
        let mut data = vec![0u8; (width * height * 4) as usize];
        for chunk in data.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[255, 0, 0, 255]);
        }

        let mips = TerrainRenderer::generate_mip_levels(width, height, &data, 5);
        assert_eq!(mips.len(), 5);

        assert_eq!(mips[0].len(), 16 * 16 * 4);
        assert_eq!(mips[1].len(), 8 * 8 * 4);
        assert_eq!(mips[2].len(), 4 * 4 * 4);
        assert_eq!(mips[3].len(), 2 * 2 * 4);
        assert_eq!(mips[4].len(), 1 * 1 * 4);

        assert_eq!(mips[4], vec![255, 0, 0, 255]);
    }

    #[test]
    fn test_generate_mip_levels_alpha_weighting() {
        let width = 2u32;
        let height = 2u32;
        let data = vec![
            255, 0, 0, 255,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];

        let mips = TerrainRenderer::generate_mip_levels(width, height, &data, 2);
        assert_eq!(mips.len(), 2);
        assert_eq!(mips[1][0], 255);
        assert_eq!(mips[1][1], 0);
        assert_eq!(mips[1][2], 0);
        assert_eq!(mips[1][3], 63);
    }
}
