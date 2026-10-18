use crate::atlas::{TextureAtlas, TextureRegion};
use crate::camera::Camera;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use world::physics::aabb::AABB;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl LineVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] = [
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x4,
            },
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TexturedVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl TexturedVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = [
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
            wgpu::VertexAttribute {
                offset: 20,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
            },
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TexturedVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

/// Particle types: Digging (block fragment), Crit (star), MagicCrit (cyan star).
#[derive(Clone, Debug)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub uv: TextureRegion,
    pub color: [f32; 4],
    pub scale: f32,
    pub age: f32,
    pub max_age: f32,
    pub gravity: f32,
}

pub struct InteractionRenderer {
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    atlas_bind_group: wgpu::BindGroup,
    line_pipeline: wgpu::RenderPipeline,
    textured_pipeline: wgpu::RenderPipeline,
    line_vertex_buffer: Option<wgpu::Buffer>,
    line_vertex_count: u32,
    crack_vertex_buffer: Option<wgpu::Buffer>,
    crack_vertex_count: u32,
    particle_vertex_buffer: Option<wgpu::Buffer>,
    particle_vertex_count: u32,
    pub particles: Vec<Particle>,
}

impl InteractionRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        atlas_texture_view: &wgpu::TextureView,
        atlas_sampler: &wgpu::Sampler,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("interaction-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("interaction.wgsl"))),
        });

        // 1. Camera uniform buffer
        let camera_uniform = CameraUniform {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("interaction-camera-buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("interaction-camera-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("interaction-camera-bind-group"),
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // 2. Atlas Bind Group Layout
        let atlas_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("interaction-atlas-bgl"),
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
            label: Some("interaction-atlas-bind-group"),
            layout: &atlas_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(atlas_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(atlas_sampler),
                },
            ],
        });

        // 3. Line Pipeline for Selection Bounding Box
        let line_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("interaction-line-layout"),
            bind_group_layouts: &[&camera_bgl],
            push_constant_ranges: &[],
        });

        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("selection-box-line-pipeline"),
            layout: Some(&line_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_line"),
                buffers: &[LineVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_line"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: -2,
                    slope_scale: -2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // 4. Textured Pipeline for Break Cracks & Particles
        let textured_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("interaction-textured-layout"),
            bind_group_layouts: &[&camera_bgl, &atlas_bgl],
            push_constant_ranges: &[],
        });

        let textured_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("interaction-textured-pipeline"),
            layout: Some(&textured_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_textured"),
                buffers: &[TexturedVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_textured"),
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
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: -4,
                    slope_scale: -4.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            camera_buffer,
            camera_bind_group,
            atlas_bind_group,
            line_pipeline,
            textured_pipeline,
            line_vertex_buffer: None,
            line_vertex_count: 0,
            crack_vertex_buffer: None,
            crack_vertex_count: 0,
            particle_vertex_buffer: None,
            particle_vertex_count: 0,
            particles: Vec::with_capacity(256),
        }
    }

    /// Updates the camera view-projection matrix uniform.
    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera) {
        let uniform = CameraUniform {
            view_proj: camera.view_proj_matrix().to_cols_array_2d(),
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Prepares line geometry for the selection bounding box (vanilla dark outline).
    pub fn update_selection_box(&mut self, device: &wgpu::Device, aabb: &AABB) {
        // Expand slightly (0.002 blocks) to prevent surface z-fighting
        let pad = 0.002;
        let min_x = (aabb.min_x - pad) as f32;
        let min_y = (aabb.min_y - pad) as f32;
        let min_z = (aabb.min_z - pad) as f32;
        let max_x = (aabb.max_x + pad) as f32;
        let max_y = (aabb.max_y + pad) as f32;
        let max_z = (aabb.max_z + pad) as f32;

        let col = [0.0f32, 0.0, 0.0, 0.4]; // Vanilla 1.8.9 black outline with 0.4 alpha

        let mut lines = Vec::with_capacity(24);
        let p = |x: f32, y: f32, z: f32| LineVertex {
            position: [x, y, z],
            color: col,
        };

        // 12 edges of the box
        // Bottom 4
        lines.push(p(min_x, min_y, min_z)); lines.push(p(max_x, min_y, min_z));
        lines.push(p(max_x, min_y, min_z)); lines.push(p(max_x, min_y, max_z));
        lines.push(p(max_x, min_y, max_z)); lines.push(p(min_x, min_y, max_z));
        lines.push(p(min_x, min_y, max_z)); lines.push(p(min_x, min_y, min_z));

        // Top 4
        lines.push(p(min_x, max_y, min_z)); lines.push(p(max_x, max_y, min_z));
        lines.push(p(max_x, max_y, min_z)); lines.push(p(max_x, max_y, max_z));
        lines.push(p(max_x, max_y, max_z)); lines.push(p(min_x, max_y, max_z));
        lines.push(p(min_x, max_y, max_z)); lines.push(p(min_x, max_y, min_z));

        // Vertical 4
        lines.push(p(min_x, min_y, min_z)); lines.push(p(min_x, max_y, min_z));
        lines.push(p(max_x, min_y, min_z)); lines.push(p(max_x, max_y, min_z));
        lines.push(p(max_x, min_y, max_z)); lines.push(p(max_x, max_y, max_z));
        lines.push(p(min_x, min_y, max_z)); lines.push(p(min_x, max_y, max_z));

        self.line_vertex_count = lines.len() as u32;
        self.line_vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("selection-box-line-buf"),
            contents: bytemuck::cast_slice(&lines),
            usage: wgpu::BufferUsages::VERTEX,
        }));
    }

    /// Clears active selection box.
    pub fn clear_selection_box(&mut self) {
        self.line_vertex_count = 0;
        self.line_vertex_buffer = None;
    }

    /// Prepares textured quad overlay for destroy stage crack animation (0..9).
    pub fn update_block_cracks(
        &mut self,
        device: &wgpu::Device,
        pos: (i32, i32, i32),
        progress_stage: usize,
        atlas: &TextureAtlas,
    ) {
        let stage_name = format!("destroy_stage_{}", progress_stage.min(9));
        let region = atlas.regions.get(&stage_name).copied().unwrap_or_default();

        let x = pos.0 as f32;
        let y = pos.1 as f32;
        let z = pos.2 as f32;

        let pad = 0.001f32;
        let x0 = x - pad;
        let y0 = y - pad;
        let z0 = z - pad;
        let x1 = x + 1.0 + pad;
        let y1 = y + 1.0 + pad;
        let z1 = z + 1.0 + pad;

        let col = [1.0f32, 1.0, 1.0, 0.8];
        let mut v = Vec::with_capacity(36);

        let u0 = region.u_min;
        let v0 = region.v_min;
        let u1 = region.u_max;
        let v1 = region.v_max;

        let mut add_quad = |p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3]| {
            v.push(TexturedVertex { position: p0, uv: [u0, v0], color: col });
            v.push(TexturedVertex { position: p1, uv: [u0, v1], color: col });
            v.push(TexturedVertex { position: p2, uv: [u1, v1], color: col });

            v.push(TexturedVertex { position: p0, uv: [u0, v0], color: col });
            v.push(TexturedVertex { position: p2, uv: [u1, v1], color: col });
            v.push(TexturedVertex { position: p3, uv: [u1, v0], color: col });
        };

        // Down
        add_quad([x0, y0, z1], [x0, y0, z0], [x1, y0, z0], [x1, y0, z1]);
        // Up
        add_quad([x0, y1, z0], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0]);
        // North
        add_quad([x1, y1, z0], [x1, y0, z0], [x0, y0, z0], [x0, y1, z0]);
        // South
        add_quad([x0, y1, z1], [x0, y0, z1], [x1, y0, z1], [x1, y1, z1]);
        // West
        add_quad([x0, y1, z0], [x0, y0, z0], [x0, y0, z1], [x0, y1, z1]);
        // East
        add_quad([x1, y1, z1], [x1, y0, z1], [x1, y0, z0], [x1, y1, z0]);

        self.crack_vertex_count = v.len() as u32;
        self.crack_vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("block-crack-vertex-buf"),
            contents: bytemuck::cast_slice(&v),
            usage: wgpu::BufferUsages::VERTEX,
        }));
    }

    /// Clears active block crack overlay.
    pub fn clear_block_cracks(&mut self) {
        self.crack_vertex_count = 0;
        self.crack_vertex_buffer = None;
    }

    /// Spawns block digging particles (EntityDiggingFX).
    pub fn spawn_digging_particles(&mut self, pos: Vec3, region: TextureRegion, count: usize) {
        let u_span = (region.u_max - region.u_min) * 0.25;
        let v_span = (region.v_max - region.v_min) * 0.25;

        for i in 0..count {
            let offset_x = ((i * 17) % 10) as f32 * 0.08 - 0.4;
            let offset_y = ((i * 23) % 10) as f32 * 0.08 - 0.4;
            let offset_z = ((i * 31) % 10) as f32 * 0.08 - 0.4;

            let vel_x = offset_x * 2.0;
            let vel_y = ((i * 13) % 5) as f32 * 0.5 + 1.2;
            let vel_z = offset_z * 2.0;

            let u_sub = region.u_min + ((i % 4) as f32) * u_span;
            let v_sub = region.v_min + (((i / 4) % 4) as f32) * v_span;

            self.particles.push(Particle {
                position: pos + Vec3::new(offset_x, offset_y, offset_z),
                velocity: Vec3::new(vel_x, vel_y, vel_z),
                uv: TextureRegion {
                    u_min: u_sub,
                    v_min: v_sub,
                    u_max: u_sub + u_span,
                    v_max: v_sub + v_span,
                },
                color: [1.0, 1.0, 1.0, 1.0],
                scale: 0.12,
                age: 0.0,
                max_age: 0.8,
                gravity: 9.8,
            });
        }
    }

    /// Spawns critical hit particles (EntityCritFX / EntityMagicCritFX).
    pub fn spawn_crit_particles(&mut self, pos: Vec3, is_magic: bool, count: usize) {
        let col = if is_magic {
            [0.2f32, 0.9, 1.0, 1.0] // Cyan magic crit
        } else {
            [1.0f32, 0.8, 0.2, 1.0] // Golden vanilla crit
        };

        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let speed = 1.8 + ((i * 7) % 5) as f32 * 0.3;
            let vel_x = angle.cos() * speed;
            let vel_z = angle.sin() * speed;
            let vel_y = 1.0 + ((i * 11) % 4) as f32 * 0.4;

            self.particles.push(Particle {
                position: pos + Vec3::new(0.0, 0.8, 0.0),
                velocity: Vec3::new(vel_x, vel_y, vel_z),
                uv: TextureRegion::default(),
                color: col,
                scale: 0.15,
                age: 0.0,
                max_age: 0.6,
                gravity: 4.0,
            });
        }
    }

    /// Advances particle simulation by dt seconds (physics tick).
    pub fn tick_particles(&mut self, dt: f32) {
        self.particles.retain_mut(|p| {
            p.age += dt;
            if p.age >= p.max_age {
                return false;
            }
            p.position += p.velocity * dt;
            p.velocity.y -= p.gravity * dt;
            p.velocity.x *= 0.98;
            p.velocity.z *= 0.98;
            true
        });
    }

    /// Updates particle vertex geometry for rendering.
    pub fn update_particle_mesh(&mut self, device: &wgpu::Device, camera: &Camera) {
        if self.particles.is_empty() {
            self.particle_vertex_count = 0;
            self.particle_vertex_buffer = None;
            return;
        }

        let right = camera.right();
        let up = right.cross(camera.forward()).normalize();

        let mut v = Vec::with_capacity(self.particles.len() * 6);

        for p in &self.particles {
            let half_s = p.scale * 0.5 * (1.0 - p.age / p.max_age);
            let rx = right * half_s;
            let uy = up * half_s;

            let p0 = p.position - rx - uy;
            let p1 = p.position - rx + uy;
            let p2 = p.position + rx + uy;
            let p3 = p.position + rx - uy;

            let u0 = p.uv.u_min;
            let v0 = p.uv.v_min;
            let u1 = p.uv.u_max;
            let v1 = p.uv.v_max;

            v.push(TexturedVertex { position: p0.into(), uv: [u0, v1], color: p.color });
            v.push(TexturedVertex { position: p1.into(), uv: [u0, v0], color: p.color });
            v.push(TexturedVertex { position: p2.into(), uv: [u1, v0], color: p.color });

            v.push(TexturedVertex { position: p0.into(), uv: [u0, v1], color: p.color });
            v.push(TexturedVertex { position: p2.into(), uv: [u1, v0], color: p.color });
            v.push(TexturedVertex { position: p3.into(), uv: [u1, v1], color: p.color });
        }

        self.particle_vertex_count = v.len() as u32;
        self.particle_vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("particle-vertex-buf"),
            contents: bytemuck::cast_slice(&v),
            usage: wgpu::BufferUsages::VERTEX,
        }));
    }

    /// Renders selection outline, break cracks, and particle passes.
    pub fn render<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        // 1. Draw Selection Box Outline
        if let Some(buf) = &self.line_vertex_buffer {
            if self.line_vertex_count > 0 {
                rpass.set_pipeline(&self.line_pipeline);
                rpass.set_bind_group(0, &self.camera_bind_group, &[]);
                rpass.set_vertex_buffer(0, buf.slice(..));
                rpass.draw(0..self.line_vertex_count, 0..1);
            }
        }

        // 2. Draw Block Damage Cracks
        if let Some(buf) = &self.crack_vertex_buffer {
            if self.crack_vertex_count > 0 {
                rpass.set_pipeline(&self.textured_pipeline);
                rpass.set_bind_group(0, &self.camera_bind_group, &[]);
                rpass.set_bind_group(1, &self.atlas_bind_group, &[]);
                rpass.set_vertex_buffer(0, buf.slice(..));
                rpass.draw(0..self.crack_vertex_count, 0..1);
            }
        }

        // 3. Draw Particles
        if let Some(buf) = &self.particle_vertex_buffer {
            if self.particle_vertex_count > 0 {
                rpass.set_pipeline(&self.textured_pipeline);
                rpass.set_bind_group(0, &self.camera_bind_group, &[]);
                rpass.set_bind_group(1, &self.atlas_bind_group, &[]);
                rpass.set_vertex_buffer(0, buf.slice(..));
                rpass.draw(0..self.particle_vertex_count, 0..1);
            }
        }
    }
}
