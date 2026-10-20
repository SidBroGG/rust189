use crate::camera::Camera;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use hashbrown::HashMap;
use image::RgbaImage;
use std::f32::consts::PI;
use uuid::Uuid;
use world::entity::{EntityKind, EntityTracker, TrackedEntity};

/// Represents a single vertex in entity 3D meshes.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct EntityVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl EntityVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] = [
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
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<EntityVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

/// Uniforms buffer passed to `entity.wgsl`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct EntityUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
    pub fog_color: [f32; 4],
    pub fog_params: [f32; 4],
    pub sun_direction: [f32; 4],
}

/// GPU Texture and BindGroup handle for skin/entity textures.
pub struct GpuSkin {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

/// High-performance entity renderer for players, mobs, and projectiles.
pub struct EntityRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    default_skin: GpuSkin,
    arrow_skin: GpuSkin,
    skin_cache: HashMap<Uuid, GpuSkin>,
    entity_texture_cache: HashMap<String, GpuSkin>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
}

impl EntityRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        default_skin_img: &RgbaImage,
        arrow_skin_img: Option<&RgbaImage>,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("entity-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("entity.wgsl").into()),
        });

        // 1. Uniform bind group layout
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("entity-uniform-bgl"),
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

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("entity-uniform-buffer"),
            size: std::mem::size_of::<EntityUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("entity-uniform-bg"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // 2. Texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("entity-texture-bgl"),
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("entity-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let default_skin = Self::create_gpu_skin(
            device,
            queue,
            &texture_bind_group_layout,
            &sampler,
            default_skin_img,
            "default-steve",
        );

        let fallback_arrow = image::ImageBuffer::from_pixel(32, 32, image::Rgba([180, 160, 130, 255]));
        let arrow_img = arrow_skin_img.unwrap_or(&fallback_arrow);
        let arrow_skin = Self::create_gpu_skin(
            device,
            queue,
            &texture_bind_group_layout,
            &sampler,
            arrow_img,
            "arrow-texture",
        );

        // 3. Pipeline layout & pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("entity-pipeline-layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("entity-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[EntityVertex::desc()],
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
                cull_mode: None, // Double sided for model outerwear & small parts
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_capacity = 8192;
        let index_capacity = 16384;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("entity-vertex-buffer"),
            size: (vertex_capacity * std::mem::size_of::<EntityVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("entity-index-buffer"),
            size: (index_capacity * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group_layout,
            sampler,
            default_skin,
            arrow_skin,
            skin_cache: HashMap::new(),
            entity_texture_cache: HashMap::new(),
            vertex_buffer,
            index_buffer,
            vertex_capacity,
            index_capacity,
        }
    }

    fn create_gpu_skin(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        img: &RgbaImage,
        label: &str,
    ) -> GpuSkin {
        let width = img.width().max(1);
        let height = img.height().max(1);

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            img.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{}-bg", label)),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        GpuSkin {
            texture,
            view,
            bind_group,
        }
    }

    /// Uploads or caches a skin texture for a player UUID.
    pub fn ensure_skin(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uuid: Uuid,
        skin_img: &RgbaImage,
    ) {
        if !self.skin_cache.contains_key(&uuid) {
            let gpu_skin = Self::create_gpu_skin(
                device,
                queue,
                &self.texture_bind_group_layout,
                &self.sampler,
                skin_img,
                &format!("skin-{}", uuid),
            );
            self.skin_cache.insert(uuid, gpu_skin);
        }
    }

    /// Uploads or caches an entity/mob texture by resource key.
    pub fn ensure_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: &str,
    ) {
        if !self.entity_texture_cache.contains_key(key) {
            let img = load_entity_texture(key);
            let gpu_skin = Self::create_gpu_skin(
                device,
                queue,
                &self.texture_bind_group_layout,
                &self.sampler,
                &img,
                key,
            );
            self.entity_texture_cache.insert(key.to_string(), gpu_skin);
        }
    }

    /// Updates uniform buffer (matrices, fog, camera pos).
    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        camera: &Camera,
        render_distance_blocks: f32,
    ) {
        let view_proj = camera.view_proj_matrix();
        let uniforms = EntityUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: [camera.position.x, camera.position.y, camera.position.z, 1.0],
            fog_color: [0.70, 0.80, 1.00, 1.0],
            fog_params: [render_distance_blocks * 0.75, render_distance_blocks, 0.0, 0.0],
            sun_direction: [0.4, 0.9, 0.2, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Renders all entities from the entity tracker plus optional local player in third-person.
    pub fn render<'a>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rpass: &mut wgpu::RenderPass<'a>,
        tracker: &EntityTracker,
        partial_ticks: f32,
        local_player: Option<&TrackedEntity>,
    ) {
        if tracker.count() == 0 && local_player.is_none() {
            return;
        }

        // Group geometry by texture key for hardware-instanced batching (one draw call per texture)
        let mut collector: HashMap<String, (Vec<EntityVertex>, Vec<u32>)> = HashMap::new();

        let all_entities = tracker.entities.values().chain(local_player);
        for entity in all_entities {
            let alpha = if entity.is_invisible { 0.15 } else { 1.0 };
            let color = if entity.hurt_time > 0 {
                [1.0, 0.35, 0.35, alpha]
            } else {
                [1.0, 1.0, 1.0, alpha]
            };

            match &entity.kind {
                EntityKind::Player { uuid, .. } => {
                    let key = format!("player_{}", uuid);
                    let (verts, inds) = collector.entry(key).or_default();
                    build_player_model(verts, inds, entity, partial_ticks, color);
                }
                EntityKind::Mob { mob_type } => {
                    match *mob_type {
                        50 => {
                            let (verts, inds) = collector.entry("entity/creeper/creeper".to_string()).or_default();
                            crate::models::hostile::build_creeper_model(verts, inds, entity, partial_ticks, color);
                            let is_powered = entity.metadata.iter().any(|m| m.index == 17 && matches!(m.value, world::entity::MetadataValue::Byte(1)));
                            if is_powered {
                                let (cverts, cinds) = collector.entry("entity/creeper/creeper_armor".to_string()).or_default();
                                crate::models::layers::render_creeper_charge_layer(cverts, cinds, entity, partial_ticks);
                            }
                        }
                        51 => {
                            let is_wither = entity.metadata.iter().any(|m| m.index == 13 && matches!(m.value, world::entity::MetadataValue::Byte(1)));
                            let key = if is_wither { "entity/skeleton/wither_skeleton" } else { "entity/skeleton/skeleton" };
                            let (verts, inds) = collector.entry(key.to_string()).or_default();
                            crate::models::hostile::build_skeleton_model(verts, inds, entity, partial_ticks, is_wither, color);
                            if !is_wither {
                                let (bverts, binds) = collector.entry("items/bow_standby".to_string()).or_default();
                                crate::models::hostile::build_skeleton_bow(bverts, binds, entity, partial_ticks);
                            }
                        }
                        52 => {
                            let (verts, inds) = collector.entry("entity/spider/spider".to_string()).or_default();
                            crate::models::hostile::build_spider_model(verts, inds, entity, partial_ticks, false, color);
                            let (everts, einds) = collector.entry("entity/spider_eyes".to_string()).or_default();
                            crate::models::layers::render_spider_eyes_layer(everts, einds, entity, partial_ticks, false);
                        }
                        53 => {
                            let (verts, inds) = collector.entry("entity/zombie/zombie".to_string()).or_default();
                            crate::models::hostile::build_giant_model(verts, inds, entity, partial_ticks, color);
                        }
                        54 => {
                            let (verts, inds) = collector.entry("entity/zombie/zombie".to_string()).or_default();
                            crate::models::hostile::build_zombie_model(verts, inds, entity, partial_ticks, color);
                        }
                        55 => {
                            let (verts, inds) = collector.entry("entity/slime/slime".to_string()).or_default();
                            crate::models::hostile::build_slime_model(verts, inds, entity, partial_ticks, color);
                            crate::models::layers::render_slime_gel_layer(verts, inds, entity, partial_ticks, color);
                        }
                        56 => {
                            let (verts, inds) = collector.entry("entity/ghast/ghast".to_string()).or_default();
                            crate::models::hostile::build_ghast_model(verts, inds, entity, partial_ticks, color);
                        }
                        57 => {
                            let (verts, inds) = collector.entry("entity/zombie_pigman".to_string()).or_default();
                            crate::models::hostile::build_zombie_model(verts, inds, entity, partial_ticks, color);
                        }
                        58 => {
                            let (verts, inds) = collector.entry("entity/enderman/enderman".to_string()).or_default();
                            crate::models::hostile::build_enderman_model(verts, inds, entity, partial_ticks, color);
                            let (everts, einds) = collector.entry("entity/enderman/enderman_eyes".to_string()).or_default();
                            crate::models::layers::render_enderman_eyes_layer(everts, einds, entity, partial_ticks);
                        }
                        59 => {
                            let (verts, inds) = collector.entry("entity/spider/cave_spider".to_string()).or_default();
                            crate::models::hostile::build_spider_model(verts, inds, entity, partial_ticks, true, color);
                            let (everts, einds) = collector.entry("entity/spider_eyes".to_string()).or_default();
                            crate::models::layers::render_spider_eyes_layer(everts, einds, entity, partial_ticks, true);
                        }
                        60 => {
                            let (verts, inds) = collector.entry("entity/silverfish".to_string()).or_default();
                            crate::models::hostile::build_silverfish_model(verts, inds, entity, partial_ticks, color);
                        }
                        61 => {
                            let (verts, inds) = collector.entry("entity/blaze".to_string()).or_default();
                            crate::models::hostile::build_blaze_model(verts, inds, entity, partial_ticks, color);
                        }
                        62 => {
                            let (verts, inds) = collector.entry("entity/slime/magmacube".to_string()).or_default();
                            crate::models::hostile::build_magma_cube_model(verts, inds, entity, partial_ticks, color);
                        }
                        63 => {
                            let (verts, inds) = collector.entry("entity/enderdragon/dragon".to_string()).or_default();
                            crate::models::boss::build_dragon_model(verts, inds, entity, partial_ticks, color);
                            let (everts, einds) = collector.entry("entity/enderdragon/dragon_eyes".to_string()).or_default();
                            crate::models::layers::render_dragon_eyes_layer(everts, einds, entity, partial_ticks);
                        }
                        64 => {
                            let (verts, inds) = collector.entry("entity/wither/wither".to_string()).or_default();
                            crate::models::boss::build_wither_model(verts, inds, entity, partial_ticks, color);
                            let (averts, ainds) = collector.entry("entity/wither/wither_armor".to_string()).or_default();
                            crate::models::layers::render_wither_aura_layer(averts, ainds, entity, partial_ticks);
                        }
                        65 => {
                            let (verts, inds) = collector.entry("entity/bat".to_string()).or_default();
                            crate::models::peaceful::build_bat_model(verts, inds, entity, partial_ticks, color);
                        }
                        66 => {
                            let (verts, inds) = collector.entry("entity/witch".to_string()).or_default();
                            crate::models::hostile::build_witch_model(verts, inds, entity, partial_ticks, color);
                        }
                        67 => {
                            let (verts, inds) = collector.entry("entity/endermite".to_string()).or_default();
                            crate::models::hostile::build_endermite_model(verts, inds, entity, partial_ticks, color);
                        }
                        68 => {
                            let is_elder = entity.metadata.iter().any(|m| m.index == 16 && matches!(m.value, world::entity::MetadataValue::Int(v) if (v & 4) != 0));
                            let key = if is_elder { "entity/guardian_elder" } else { "entity/guardian" };
                            let (verts, inds) = collector.entry(key.to_string()).or_default();
                            crate::models::hostile::build_guardian_model(verts, inds, entity, partial_ticks, is_elder, color);
                        }
                        90 => {
                            let (verts, inds) = collector.entry("entity/pig/pig".to_string()).or_default();
                            crate::models::peaceful::build_pig_model(verts, inds, entity, partial_ticks, color);
                            let has_saddle = entity.metadata.iter().any(|m| m.index == 16 && matches!(m.value, world::entity::MetadataValue::Byte(1)));
                            if has_saddle {
                                let (sverts, sinds) = collector.entry("entity/pig/pig_saddle".to_string()).or_default();
                                crate::models::layers::render_pig_saddle_layer(sverts, sinds, entity, partial_ticks, color);
                            }
                        }
                        91 => {
                            let (verts, inds) = collector.entry("entity/sheep/sheep".to_string()).or_default();
                            crate::models::peaceful::build_sheep_model(verts, inds, entity, partial_ticks, color);
                            let dye = entity.metadata.iter().find(|m| m.index == 16).and_then(|m| match m.value {
                                world::entity::MetadataValue::Byte(b) => Some((b & 0x0F) as u8),
                                _ => None,
                            }).unwrap_or(0);
                            let is_sheared = entity.metadata.iter().any(|m| m.index == 16 && matches!(m.value, world::entity::MetadataValue::Byte(b) if (b & 0x10) != 0));
                            let (wverts, winds) = collector.entry("entity/sheep/sheep_fur".to_string()).or_default();
                            crate::models::layers::render_sheep_wool_layer(wverts, winds, entity, partial_ticks, dye, is_sheared);
                        }
                        92 => {
                            let (verts, inds) = collector.entry("entity/cow/cow".to_string()).or_default();
                            crate::models::peaceful::build_cow_model(verts, inds, entity, partial_ticks, color);
                        }
                        93 => {
                            let (verts, inds) = collector.entry("entity/chicken".to_string()).or_default();
                            crate::models::peaceful::build_chicken_model(verts, inds, entity, partial_ticks, color);
                        }
                        94 => {
                            let (verts, inds) = collector.entry("entity/squid".to_string()).or_default();
                            crate::models::peaceful::build_squid_model(verts, inds, entity, partial_ticks, color);
                        }
                        95 => {
                            let (verts, inds) = collector.entry("entity/wolf/wolf".to_string()).or_default();
                            crate::models::peaceful::build_wolf_model(verts, inds, entity, partial_ticks, color);
                            let is_tamed = entity.metadata.iter().any(|m| m.index == 16 && matches!(m.value, world::entity::MetadataValue::Byte(b) if (b & 0x04) != 0));
                            if is_tamed {
                                let collar_dye = entity.metadata.iter().find(|m| m.index == 20).and_then(|m| match m.value {
                                    world::entity::MetadataValue::Byte(b) => Some((b & 0x0F) as u8),
                                    _ => None,
                                }).unwrap_or(14);
                                let (cverts, cinds) = collector.entry("entity/wolf/wolf_collar".to_string()).or_default();
                                crate::models::layers::render_wolf_collar_layer(cverts, cinds, entity, partial_ticks, collar_dye);
                            }
                        }
                        96 => {
                            let (verts, inds) = collector.entry("entity/cow/mooshroom".to_string()).or_default();
                            crate::models::peaceful::build_cow_model(verts, inds, entity, partial_ticks, color);
                            let (mverts, minds) = collector.entry("blocks/mushroom_red".to_string()).or_default();
                            crate::models::layers::render_mooshroom_mushroom_layer(mverts, minds, entity, partial_ticks, color);
                        }
                        97 => {
                            let (verts, inds) = collector.entry("entity/snowman".to_string()).or_default();
                            crate::models::peaceful::build_snowman_model(verts, inds, entity, partial_ticks, color);
                            crate::models::layers::render_snowman_head_layer(verts, inds, entity, partial_ticks, color);
                        }
                        98 => {
                            let (verts, inds) = collector.entry("entity/cat/ocelot".to_string()).or_default();
                            crate::models::peaceful::build_ocelot_model(verts, inds, entity, partial_ticks, color);
                        }
                        99 => {
                            let (verts, inds) = collector.entry("entity/iron_golem".to_string()).or_default();
                            crate::models::peaceful::build_iron_golem_model(verts, inds, entity, partial_ticks, color);
                            crate::models::layers::render_iron_golem_flower_layer(verts, inds, entity, partial_ticks, color);
                        }
                        100 => {
                            let (verts, inds) = collector.entry("entity/horse/horse_white".to_string()).or_default();
                            crate::models::peaceful::build_horse_model(verts, inds, entity, partial_ticks, false, color);
                        }
                        101 => {
                            let (verts, inds) = collector.entry("entity/rabbit/brown".to_string()).or_default();
                            crate::models::peaceful::build_rabbit_model(verts, inds, entity, partial_ticks, color);
                        }
                        120 => {
                            let (verts, inds) = collector.entry("entity/villager/villager".to_string()).or_default();
                            crate::models::peaceful::build_villager_model(verts, inds, entity, partial_ticks, color);
                        }
                        _ => {
                            let (verts, inds) = collector.entry("entity/zombie/zombie".to_string()).or_default();
                            crate::models::hostile::build_zombie_model(verts, inds, entity, partial_ticks, color);
                        }
                    }
                }
                EntityKind::Object { object_type, .. } => {
                    match *object_type {
                        1 => {
                            let (verts, inds) = collector.entry("entity/boat".to_string()).or_default();
                            crate::models::vehicles::build_boat_model(verts, inds, entity, partial_ticks, color);
                        }
                        2 => {
                            // Dropped item (EntityItem)
                            let slot = entity.metadata.iter().find(|m| m.index == 10).and_then(|m| match &m.value {
                                world::entity::MetadataValue::Slot(Some(s)) => Some((s.item_id, s.item_count, s.item_damage)),
                                _ => None,
                            });
                            let (item_id, dmg) = slot.map_or((260, 0), |(id, _, d)| (id, d));
                            let (tex_key, is_block) = get_item_render_texture_key(item_id, dmg);
                            let (verts, inds) = collector.entry(tex_key).or_default();
                            crate::models::vehicles::build_dropped_item_model(verts, inds, entity, is_block, partial_ticks, color);
                        }
                        10 => {
                            let (verts, inds) = collector.entry("entity/minecart".to_string()).or_default();
                            crate::models::vehicles::build_minecart_model(verts, inds, entity, partial_ticks, color);
                        }
                        51 => {
                            let (verts, inds) = collector.entry("entity/endercrystal/endercrystal".to_string()).or_default();
                            crate::models::vehicles::build_ender_crystal_model(verts, inds, entity, partial_ticks, color);
                        }
                        60 => {
                            let (verts, inds) = collector.entry("entity/arrow".to_string()).or_default();
                            crate::models::vehicles::build_arrow_model(verts, inds, entity, partial_ticks, color);
                        }
                        61 => {
                            let (verts, inds) = collector.entry("items/snowball".to_string()).or_default();
                            crate::models::vehicles::build_throwable_model(verts, inds, entity, 0.35, partial_ticks, color);
                        }
                        62 => {
                            let (verts, inds) = collector.entry("items/egg".to_string()).or_default();
                            crate::models::vehicles::build_throwable_model(verts, inds, entity, 0.35, partial_ticks, color);
                        }
                        63 => {
                            let (verts, inds) = collector.entry("items/fireball".to_string()).or_default();
                            crate::models::vehicles::build_throwable_model(verts, inds, entity, 0.8, partial_ticks, color);
                        }
                        64 => {
                            let (verts, inds) = collector.entry("items/fireball".to_string()).or_default();
                            crate::models::vehicles::build_throwable_model(verts, inds, entity, 0.4, partial_ticks, color);
                        }
                        65 => {
                            let (verts, inds) = collector.entry("items/ender_pearl".to_string()).or_default();
                            crate::models::vehicles::build_throwable_model(verts, inds, entity, 0.35, partial_ticks, color);
                        }
                        66 => {
                            let (verts, inds) = collector.entry("entity/wither/wither".to_string()).or_default();
                            crate::models::vehicles::build_wither_skull_model(verts, inds, entity, partial_ticks, color);
                        }
                        71 => {
                            // Item Frame: wooden borders + birch background plate + optional item inside
                            let (f_verts, f_inds) = collector.entry("blocks/planks_birch".to_string()).or_default();
                            crate::models::vehicles::build_item_frame_borders(f_verts, f_inds, entity, partial_ticks, color);

                            let (bg_verts, bg_inds) = collector.entry("blocks/itemframe_background".to_string()).or_default();
                            crate::models::vehicles::build_item_frame_background(bg_verts, bg_inds, entity, partial_ticks, color);

                            let item_slot = entity.metadata.iter().find(|m| m.index == 8).and_then(|m| match &m.value {
                                world::entity::MetadataValue::Slot(Some(slot)) => Some((slot.item_id, slot.item_damage)),
                                _ => None,
                            });
                            if let Some((item_id, dmg)) = item_slot {
                                let rot_steps = entity.metadata.iter().find(|m| m.index == 9).and_then(|m| match m.value {
                                    world::entity::MetadataValue::Byte(b) => Some((b & 7) as u8),
                                    _ => None,
                                }).unwrap_or(0);
                                let (tex_key, is_block) = get_item_render_texture_key(item_id, dmg);
                                let (i_verts, i_inds) = collector.entry(tex_key).or_default();
                                crate::models::vehicles::build_item_frame_item(i_verts, i_inds, entity, rot_steps, is_block, partial_ticks, color);
                            }
                        }
                        72 => {
                            let (verts, inds) = collector.entry("items/ender_eye".to_string()).or_default();
                            crate::models::vehicles::build_throwable_model(verts, inds, entity, 0.35, partial_ticks, color);
                        }
                        73 => {
                            let (verts, inds) = collector.entry("items/potion_bottle_drinkable".to_string()).or_default();
                            crate::models::vehicles::build_throwable_model(verts, inds, entity, 0.4, partial_ticks, color);
                        }
                        75 => {
                            let (verts, inds) = collector.entry("items/experience_bottle".to_string()).or_default();
                            crate::models::vehicles::build_throwable_model(verts, inds, entity, 0.35, partial_ticks, color);
                        }
                        77 => {
                            let (verts, inds) = collector.entry("entity/lead_knot".to_string()).or_default();
                            crate::models::vehicles::build_leash_knot_model(verts, inds, entity, partial_ticks, color);
                        }
                        78 => {
                            let (verts, inds) = collector.entry("entity/armorstand/wood".to_string()).or_default();
                            crate::models::vehicles::build_armor_stand_model(verts, inds, entity, partial_ticks, color);
                        }
                        90 => {
                            let (verts, inds) = collector.entry("particle/particles".to_string()).or_default();
                            crate::models::vehicles::build_fishing_bobber_model(verts, inds, entity, partial_ticks);
                        }
                        _ => {
                            let (verts, inds) = collector.entry("entity/arrow".to_string()).or_default();
                            crate::models::vehicles::build_arrow_model(verts, inds, entity, partial_ticks, color);
                        }
                    }
                }
                EntityKind::Painting { title, facing } => {
                    let (verts, inds) = collector.entry("painting/paintings_kristoffer_zetterstrand".to_string()).or_default();
                    crate::models::vehicles::build_painting_model(verts, inds, entity, title, *facing, partial_ticks, color);
                }
            }
        }

        // Flatten collected per-texture batches into continuous vertex and index buffers
        let mut vertices: Vec<EntityVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        struct DrawBatch {
            index_start: u32,
            index_count: u32,
            texture_key: String,
        }

        let mut batches: Vec<DrawBatch> = Vec::new();

        for (tex_key, (b_verts, b_inds)) in collector {
            if b_verts.is_empty() || b_inds.is_empty() {
                continue;
            }
            let vertex_offset = vertices.len() as u32;
            let index_start = indices.len() as u32;

            vertices.extend_from_slice(&b_verts);
            for idx in b_inds {
                indices.push(idx + vertex_offset);
            }
            let index_count = (indices.len() as u32) - index_start;

            batches.push(DrawBatch {
                index_start,
                index_count,
                texture_key: tex_key,
            });
        }

        if vertices.is_empty() || indices.is_empty() {
            return;
        }

        // Ensure GPU textures are loaded
        for batch in &batches {
            if !batch.texture_key.starts_with("player_") && batch.texture_key != "arrow" {
                self.ensure_texture(device, queue, &batch.texture_key);
            }
        }

        // Expand buffers if necessary
        if vertices.len() > self.vertex_capacity {
            self.vertex_capacity = vertices.len() * 2;
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("entity-vertex-buffer-resized"),
                size: (self.vertex_capacity * std::mem::size_of::<EntityVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if indices.len() > self.index_capacity {
            self.index_capacity = indices.len() * 2;
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("entity-index-buffer-resized"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

        // Draw passes
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        for batch in batches {
            let bg = if batch.texture_key == "arrow" {
                &self.arrow_skin.bind_group
            } else if let Some(uuid_str) = batch.texture_key.strip_prefix("player_") {
                if let Ok(uuid) = Uuid::parse_str(uuid_str) {
                    if let Some(gpu_skin) = self.skin_cache.get(&uuid) {
                        &gpu_skin.bind_group
                    } else {
                        &self.default_skin.bind_group
                    }
                } else {
                    &self.default_skin.bind_group
                }
            } else if let Some(gpu_skin) = self.entity_texture_cache.get(&batch.texture_key) {
                &gpu_skin.bind_group
            } else {
                &self.default_skin.bind_group
            };

            rpass.set_bind_group(1, bg, &[]);
            rpass.draw_indexed(batch.index_start..(batch.index_start + batch.index_count), 0, 0..1);
        }
    }
}

/// Maps item ID and metadata to texture asset key and whether it renders as a 3D block or flat 3D slab.
pub fn get_item_render_texture_key(item_id: i16, damage: i16) -> (String, bool) {
    match item_id {
        // Blocks (1..255)
        1 => match damage {
            1 => ("blocks/stone_granite".to_string(), true),
            2 => ("blocks/stone_granite_smooth".to_string(), true),
            3 => ("blocks/stone_diorite".to_string(), true),
            4 => ("blocks/stone_diorite_smooth".to_string(), true),
            5 => ("blocks/stone_andesite".to_string(), true),
            6 => ("blocks/stone_andesite_smooth".to_string(), true),
            _ => ("blocks/stone".to_string(), true),
        },
        2 => ("blocks/grass_top".to_string(), true),
        3 => match damage {
            1 => ("blocks/coarse_dirt".to_string(), true),
            2 => ("blocks/dirt_podzol_top".to_string(), true),
            _ => ("blocks/dirt".to_string(), true),
        },
        4 => ("blocks/cobblestone".to_string(), true),
        5 => match damage {
            1 => ("blocks/planks_spruce".to_string(), true),
            2 => ("blocks/planks_birch".to_string(), true),
            3 => ("blocks/planks_jungle".to_string(), true),
            4 => ("blocks/planks_acacia".to_string(), true),
            5 => ("blocks/planks_big_oak".to_string(), true),
            _ => ("blocks/planks_oak".to_string(), true),
        },
        6 => match damage {
            1 => ("blocks/sapling_spruce".to_string(), true),
            2 => ("blocks/sapling_birch".to_string(), true),
            3 => ("blocks/sapling_jungle".to_string(), true),
            4 => ("blocks/sapling_acacia".to_string(), true),
            5 => ("blocks/sapling_roofed_oak".to_string(), true),
            _ => ("blocks/sapling_oak".to_string(), true),
        },
        12 => if damage == 1 { ("blocks/red_sand".to_string(), true) } else { ("blocks/sand".to_string(), true) },
        17 => match damage {
            1 => ("blocks/log_spruce".to_string(), true),
            2 => ("blocks/log_birch".to_string(), true),
            3 => ("blocks/log_jungle".to_string(), true),
            _ => ("blocks/log_oak".to_string(), true),
        },
        18 => match damage {
            1 => ("blocks/leaves_spruce".to_string(), true),
            2 => ("blocks/leaves_birch".to_string(), true),
            3 => ("blocks/leaves_jungle".to_string(), true),
            _ => ("blocks/leaves_oak".to_string(), true),
        },
        19 => if damage == 1 { ("blocks/sponge_wet".to_string(), true) } else { ("blocks/sponge".to_string(), true) },
        24 => match damage {
            1 => ("blocks/sandstone_carved".to_string(), true),
            2 => ("blocks/sandstone_smooth".to_string(), true),
            _ => ("blocks/sandstone_normal".to_string(), true),
        },
        31 => ("blocks/tallgrass".to_string(), true),
        35 => {
            let names = [
                "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
                "silver", "cyan", "purple", "blue", "brown", "green", "red", "black",
            ];
            let name = names.get((damage & 15) as usize).unwrap_or(&"white");
            (format!("blocks/wool_colored_{}", name), true)
        }
        44 => ("blocks/stone_slab_top".to_string(), true),
        53 => ("blocks/planks_oak".to_string(), true),
        54 => ("entity/chest/normal".to_string(), true),
        67 => ("blocks/cobblestone".to_string(), true),
        95 => {
            let names = [
                "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
                "silver", "cyan", "purple", "blue", "brown", "green", "red", "black",
            ];
            let name = names.get((damage & 15) as usize).unwrap_or(&"white");
            (format!("blocks/glass_{}", name), true)
        }
        125 | 126 => ("blocks/planks_oak".to_string(), true),
        134..=136 | 163 | 164 => ("blocks/planks_oak".to_string(), true),
        161 => ("blocks/leaves_acacia".to_string(), true),
        162 => ("blocks/log_acacia".to_string(), true),
        171 => {
            let names = [
                "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
                "silver", "cyan", "purple", "blue", "brown", "green", "red", "black",
            ];
            let name = names.get((damage & 15) as usize).unwrap_or(&"white");
            (format!("blocks/wool_colored_{}", name), true)
        }
        // Items
        261 => ("items/bow_standby".to_string(), false),
        324 => ("items/door_wood".to_string(), false),
        328 => ("items/minecart_normal".to_string(), false),
        330 => ("items/door_iron".to_string(), false),
        333 => ("items/boat".to_string(), false),
        342 => ("items/minecart_chest".to_string(), false),
        343 => ("items/minecart_furnace".to_string(), false),
        346 => ("items/fishing_rod_uncast".to_string(), false),
        373 => ("items/potion_bottle_drinkable".to_string(), false),
        383 => ("items/spawn_egg".to_string(), false),
        407 => ("items/minecart_tnt".to_string(), false),
        408 => ("items/minecart_hopper".to_string(), false),
        427 => ("items/door_spruce".to_string(), false),
        428 => ("items/door_birch".to_string(), false),
        429 => ("items/door_jungle".to_string(), false),
        430 => ("items/door_acacia".to_string(), false),
        431 => ("items/door_dark_oak".to_string(), false),
        _ => {
            if let Some(def) = world::item::get_item_def(item_id) {
                if def.is_block {
                    (format!("blocks/{}", def.name), true)
                } else {
                    (format!("items/{}", def.name), false)
                }
            } else {
                ("items/apple".to_string(), false)
            }
        }
    }
}

/// Helper function to load texture image from filesystem with safe procedural fallbacks.
fn load_entity_texture(key: &str) -> RgbaImage {
    let clean_key = key.trim_start_matches('/');
    let candidates = [
        format!("assets/minecraft/textures/{}.png", clean_key),
        format!("assets/minecraft/textures/{}", clean_key),
        format!("../assets/minecraft/textures/{}.png", clean_key),
        format!("../../assets/minecraft/textures/{}.png", clean_key),
        format!("textures/{}.png", clean_key),
    ];

    for path in &candidates {
        if let Ok(img) = image::open(path) {
            return img.to_rgba8();
        }
    }

    // Safe fallback to apple or stone before procedural checkerboard to prevent checkerboard textures
    let fallback_keys: &[&str] = if clean_key.starts_with("blocks/") {
        &["assets/minecraft/textures/blocks/stone.png", "../assets/minecraft/textures/blocks/stone.png", "../../assets/minecraft/textures/blocks/stone.png"]
    } else if clean_key.starts_with("items/") {
        &["assets/minecraft/textures/items/apple.png", "../assets/minecraft/textures/items/apple.png", "../../assets/minecraft/textures/items/apple.png"]
    } else {
        &[]
    };

    for path in fallback_keys {
        if let Ok(img) = image::open(path) {
            return img.to_rgba8();
        }
    }

    // Procedural fallback checkerboard with color derived from key hash
    let mut hash: u32 = 0x811c9dc5;
    for b in clean_key.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    let r = (((hash >> 16) & 0xFF) as u8).max(50);
    let g = (((hash >> 8) & 0xFF) as u8).max(50);
    let b = ((hash & 0xFF) as u8).max(50);

    let mut img = image::RgbaImage::new(64, 64);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let is_checker = ((x / 8) + (y / 8)) % 2 == 0;
        if is_checker {
            *pixel = image::Rgba([r, g, b, 255]);
        } else {
            *pixel = image::Rgba([r / 2, g / 2, b / 2, 255]);
        }
    }
    img
}

/// Helper to add a textured 3D cuboid box to the vertex and index lists following ModelBox specification.
pub fn add_box_geom(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    part_matrix: Mat4,
    tex_u: f32,
    tex_v: f32,
    origin: Vec3,
    size: Vec3,
    expand: f32,
    mirror: bool,
    tex_w: f32,
    tex_h: f32,
    color: [f32; 4],
) {
    let x1 = origin.x - expand;
    let y1 = origin.y - expand;
    let z1 = origin.z - expand;
    let mut x2 = origin.x + size.x + expand;
    let y2 = origin.y + size.y + expand;
    let z2 = origin.z + size.z + expand;

    let mut x1_val = x1;
    if mirror {
        let tmp = x2;
        x2 = x1_val;
        x1_val = tmp;
    }

    let v7 = Vec3::new(x1_val, y1, z1);
    let v0 = Vec3::new(x2, y1, z1);
    let v1 = Vec3::new(x2, y2, z1);
    let v2 = Vec3::new(x1_val, y2, z1);
    let v3 = Vec3::new(x1_val, y1, z2);
    let v4 = Vec3::new(x2, y1, z2);
    let v5 = Vec3::new(x2, y2, z2);
    let v6 = Vec3::new(x1_val, y2, z2);

    let w = size.x;
    let h = size.y;
    let d = size.z;

    // 6 quads matching ModelBox.java:
    let quads = [
        // Quad 0 (+X face): {v4, v0, v1, v5}
        ([v4, v0, v1, v5], tex_u + d + w, tex_v + d, tex_u + d + w + d, tex_v + d + h),
        // Quad 1 (-X face): {v7, v3, v6, v2}
        ([v7, v3, v6, v2], tex_u, tex_v + d, tex_u + d, tex_v + d + h),
        // Quad 2 (-Y face / top): {v4, v3, v7, v0}
        ([v4, v3, v7, v0], tex_u + d, tex_v, tex_u + d + w, tex_v + d),
        // Quad 3 (+Y face / bottom): {v1, v2, v6, v5}
        ([v1, v2, v6, v5], tex_u + d + w, tex_v + d, tex_u + d + w + w, tex_v),
        // Quad 4 (-Z face / front): {v0, v7, v2, v1}
        ([v0, v7, v2, v1], tex_u + d, tex_v + d, tex_u + d + w, tex_v + d + h),
        // Quad 5 (+Z face / back): {v3, v4, v5, v6}
        ([v3, v4, v5, v6], tex_u + d + w + d, tex_v + d, tex_u + d + w + d + w, tex_v + d + h),
    ];

    for (raw_verts, u1, v1, u2, v2) in quads {
        let mut uv = [
            [u2 / tex_w, v1 / tex_h],
            [u1 / tex_w, v1 / tex_h],
            [u1 / tex_w, v2 / tex_h],
            [u2 / tex_w, v2 / tex_h],
        ];
        let mut qverts = raw_verts;
        if mirror {
            qverts.reverse();
            uv.reverse();
        }

        let raw_normal = (qverts[2] - qverts[1]).cross(qverts[0] - qverts[1]).normalize();
        let world_normal = part_matrix.transform_vector3(raw_normal).normalize().to_array();

        let base_idx = vertices.len() as u32;
        for (mcp_pos, uv_coord) in qverts.iter().zip(uv.iter()) {
            let world_pos = part_matrix.transform_point3(*mcp_pos * (1.0 / 16.0));
            vertices.push(EntityVertex {
                position: world_pos.to_array(),
                uv: *uv_coord,
                normal: world_normal,
                color,
            });
        }

        indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }
}

/// Helper to add a simple axis-aligned 3D cuboid box in local units.
pub fn add_box(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    transform: Mat4,
    min: Vec3,
    max: Vec3,
    u0: f32,
    v0: f32,
    _w: f32,
    _h: f32,
    _d: f32,
    color: [f32; 4],
) {
    let size = max - min;
    add_box_geom(
        vertices,
        indices,
        transform,
        u0,
        v0,
        min * 16.0,
        size * 16.0,
        0.0,
        false,
        64.0,
        64.0,
        color,
    );
}

/// Builds the full hierarchical 3D player model with base layer, outerwear, animations & held item.
pub fn build_player_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let body_yaw = entity.interpolated_render_yaw_offset(partial_ticks);
    let head_yaw = entity.interpolated_head_yaw(partial_ticks);
    let pitch = entity.interpolated_pitch(partial_ticks);
    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);
    let swing_prog = entity.swing_progress;
    let age_in_ticks = entity.ticks_existed as f32 + partial_ticks;

    let is_sneak = entity.is_sneaking;

    // Vanilla entity root transformation:
    // Translate(pos) * RotateY(180 - bodyYaw) * Scale(-1, -1, 1) * Translate(0, -1.5078125, 0)
    let mut root_matrix = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - body_yaw).to_radians())
        * Mat4::from_scale(Vec3::new(-1.0, -1.0, 1.0))
        * Mat4::from_translation(Vec3::new(0.0, -1.5078125, 0.0));

    if is_sneak {
        root_matrix *= Mat4::from_translation(Vec3::new(0.0, 0.2, 0.0));
    }

    let get_part_matrix = |rot_point: Vec3, rot_angles: Vec3| -> Mat4 {
        root_matrix
            * Mat4::from_translation(rot_point * (1.0 / 16.0))
            * Mat4::from_rotation_z(rot_angles.z)
            * Mat4::from_rotation_y(rot_angles.y)
            * Mat4::from_rotation_x(rot_angles.x)
    };

    // 1. Head & Headwear (independent at root)
    let net_head_yaw = head_yaw - body_yaw;
    let head_rot_y = net_head_yaw.to_radians();
    let head_rot_x = pitch.to_radians();
    let head_pivot_y = if is_sneak { 1.0 } else { 0.0 };
    let head_matrix = get_part_matrix(
        Vec3::new(0.0, head_pivot_y, 0.0),
        Vec3::new(head_rot_x, head_rot_y, 0.0),
    );

    // Head base layer (8x8x8 px)
    add_box_geom(
        vertices,
        indices,
        head_matrix,
        0.0,
        0.0,
        Vec3::new(-4.0, -8.0, -4.0),
        Vec3::new(8.0, 8.0, 8.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );
    // Headwear outer layer (+0.5 px inflation)
    add_box_geom(
        vertices,
        indices,
        head_matrix,
        32.0,
        0.0,
        Vec3::new(-4.0, -8.0, -4.0),
        Vec3::new(8.0, 8.0, 8.0),
        0.5,
        false,
        64.0,
        64.0,
        color,
    );

    // 2. Torso / Body & Bodywear
    let body_rot_x = if is_sneak { 0.5 } else { 0.0 };
    let mut body_rot_y = 0.0f32;
    if swing_prog > 0.0 {
        body_rot_y = ((swing_prog.sqrt() * PI * 2.0).sin()) * 0.2;
    }
    let body_matrix = get_part_matrix(Vec3::ZERO, Vec3::new(body_rot_x, body_rot_y, 0.0));

    // Torso base layer (8x12x4 px)
    add_box_geom(
        vertices,
        indices,
        body_matrix,
        16.0,
        16.0,
        Vec3::new(-4.0, 0.0, -2.0),
        Vec3::new(8.0, 12.0, 4.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );
    // Torso outerwear (+0.25 px inflation)
    add_box_geom(
        vertices,
        indices,
        body_matrix,
        16.0,
        32.0,
        Vec3::new(-4.0, 0.0, -2.0),
        Vec3::new(8.0, 12.0, 4.0),
        0.25,
        false,
        64.0,
        64.0,
        color,
    );

    // 3. Right Arm & Right Armwear
    let mut r_arm_pivot = Vec3::new(-5.0, 2.0, 0.0);
    let mut r_arm_rot_x = (limb_swing * 0.6662 + PI).cos() * 2.0 * limb_amount * 0.5;
    let mut r_arm_rot_y = 0.0f32;
    let mut r_arm_rot_z = 0.0f32;

    // Check held item
    let has_held_item = if let EntityKind::Player { held_item, .. } = &entity.kind {
        *held_item != 0
    } else {
        false
    };
    if has_held_item {
        r_arm_rot_x = r_arm_rot_x * 0.5 - (PI / 10.0);
    }

    if swing_prog > 0.0 {
        r_arm_pivot.z = body_rot_y.sin() * 5.0;
        r_arm_pivot.x = -body_rot_y.cos() * 5.0;
        r_arm_rot_y += body_rot_y * 2.0;
        r_arm_rot_z += (swing_prog * PI).sin() * -0.4;

        let f = 1.0 - swing_prog;
        let punch = 1.0 - f * f * f * f;
        let f1 = (punch * PI).sin();
        let f2 = (swing_prog * PI).sin() * -(head_rot_x - 0.7) * 0.75;
        r_arm_rot_x -= f1 * 1.2 + f2;
    }

    if is_sneak {
        r_arm_rot_x += 0.4;
    }

    // Idle breathing sway
    r_arm_rot_z += (age_in_ticks * 0.09).cos() * 0.05 + 0.05;
    r_arm_rot_x += (age_in_ticks * 0.067).sin() * 0.05;

    let r_arm_matrix = get_part_matrix(
        r_arm_pivot,
        Vec3::new(r_arm_rot_x, r_arm_rot_y, r_arm_rot_z),
    );

    // Right arm base layer (4x12x4 px)
    add_box_geom(
        vertices,
        indices,
        r_arm_matrix,
        40.0,
        16.0,
        Vec3::new(-3.0, -2.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );
    // Right armwear (+0.25 px inflation)
    add_box_geom(
        vertices,
        indices,
        r_arm_matrix,
        40.0,
        32.0,
        Vec3::new(-3.0, -2.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.25,
        false,
        64.0,
        64.0,
        color,
    );

    // Held 3D item attached to right arm
    if has_held_item {
        let item_local = Mat4::from_translation(Vec3::new(-1.0, 10.0, 0.0) * (1.0 / 16.0))
            * Mat4::from_rotation_x(0.8)
            * Mat4::from_rotation_z(0.3);
        let item_matrix = r_arm_matrix * item_local;
        add_box_geom(
            vertices,
            indices,
            item_matrix,
            0.0,
            0.0,
            Vec3::new(-0.5, -6.0, -0.5),
            Vec3::new(1.0, 14.0, 1.0),
            0.0,
            false,
            64.0,
            64.0,
            [0.85, 0.85, 1.0, 1.0],
        );
    }

    // 4. Left Arm & Left Armwear
    let mut l_arm_pivot = Vec3::new(5.0, 2.0, 0.0);
    let mut l_arm_rot_x = (limb_swing * 0.6662).cos() * 2.0 * limb_amount * 0.5;
    let mut l_arm_rot_y = 0.0f32;
    let mut l_arm_rot_z = 0.0f32;

    if swing_prog > 0.0 {
        l_arm_pivot.z = -body_rot_y.sin() * 5.0;
        l_arm_pivot.x = body_rot_y.cos() * 5.0;
        l_arm_rot_y += body_rot_y;
        l_arm_rot_x += body_rot_y;
    }

    if is_sneak {
        l_arm_rot_x += 0.4;
    }

    l_arm_rot_z -= (age_in_ticks * 0.09).cos() * 0.05 + 0.05;
    l_arm_rot_x -= (age_in_ticks * 0.067).sin() * 0.05;

    let l_arm_matrix = get_part_matrix(
        l_arm_pivot,
        Vec3::new(l_arm_rot_x, l_arm_rot_y, l_arm_rot_z),
    );

    // Left arm base layer (4x12x4 px)
    add_box_geom(
        vertices,
        indices,
        l_arm_matrix,
        32.0,
        48.0,
        Vec3::new(-1.0, -2.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );
    // Left armwear (+0.25 px inflation)
    add_box_geom(
        vertices,
        indices,
        l_arm_matrix,
        48.0,
        48.0,
        Vec3::new(-1.0, -2.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.25,
        false,
        64.0,
        64.0,
        color,
    );

    // 5. Right Leg & Right Legwear
    let r_leg_pivot = Vec3::new(
        -1.9,
        if is_sneak { 9.0 } else { 12.0 },
        if is_sneak { 4.0 } else { 0.1 },
    );
    let r_leg_rot_x = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let r_leg_matrix = get_part_matrix(r_leg_pivot, Vec3::new(r_leg_rot_x, 0.0, 0.0));

    // Right leg base layer (4x12x4 px)
    add_box_geom(
        vertices,
        indices,
        r_leg_matrix,
        0.0,
        16.0,
        Vec3::new(-2.0, 0.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );
    // Right legwear (+0.25 px inflation)
    add_box_geom(
        vertices,
        indices,
        r_leg_matrix,
        0.0,
        32.0,
        Vec3::new(-2.0, 0.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.25,
        false,
        64.0,
        64.0,
        color,
    );

    // 6. Left Leg & Left Legwear
    let l_leg_pivot = Vec3::new(
        1.9,
        if is_sneak { 9.0 } else { 12.0 },
        if is_sneak { 4.0 } else { 0.1 },
    );
    let l_leg_rot_x = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;
    let l_leg_matrix = get_part_matrix(l_leg_pivot, Vec3::new(l_leg_rot_x, 0.0, 0.0));

    // Left leg base layer (4x12x4 px)
    add_box_geom(
        vertices,
        indices,
        l_leg_matrix,
        16.0,
        48.0,
        Vec3::new(-2.0, 0.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );
    // Left legwear (+0.25 px inflation)
    add_box_geom(
        vertices,
        indices,
        l_leg_matrix,
        0.0,
        48.0,
        Vec3::new(-2.0, 0.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.25,
        false,
        64.0,
        64.0,
        color,
    );

    // 7. Cape
    let cape_pivot = Vec3::new(0.0, if is_sneak { 2.0 } else { 0.0 }, 2.0);
    let cape_rot_x = 0.1 + (limb_amount * 0.5);
    let cape_matrix = get_part_matrix(cape_pivot, Vec3::new(cape_rot_x, 0.0, 0.0));
    add_box_geom(
        vertices,
        indices,
        cape_matrix,
        0.0,
        0.0,
        Vec3::new(-5.0, 0.0, -0.5),
        Vec3::new(10.0, 16.0, 1.0),
        0.0,
        false,
        64.0,
        32.0,
        [0.9 * color[0], 0.2 * color[1], 0.2 * color[2], color[3]],
    );
}

/// Builds mob models (e.g. Zombie / Skeleton humanoid with forward arms, Creeper).
pub fn build_mob_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    _mob_type: u8,
    partial_ticks: f32,
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let body_yaw = entity.interpolated_render_yaw_offset(partial_ticks);
    let head_yaw = entity.interpolated_head_yaw(partial_ticks);
    let pitch = entity.interpolated_pitch(partial_ticks);
    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);

    let color = [0.4, 0.7, 0.4, 1.0]; // Mob tint

    let root_matrix = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - body_yaw).to_radians())
        * Mat4::from_scale(Vec3::new(-1.0, -1.0, 1.0))
        * Mat4::from_translation(Vec3::new(0.0, -1.5078125, 0.0));

    let get_part_matrix = |rot_point: Vec3, rot_angles: Vec3| -> Mat4 {
        root_matrix
            * Mat4::from_translation(rot_point * (1.0 / 16.0))
            * Mat4::from_rotation_z(rot_angles.z)
            * Mat4::from_rotation_y(rot_angles.y)
            * Mat4::from_rotation_x(rot_angles.x)
    };

    // Head
    let net_head_yaw = head_yaw - body_yaw;
    let head_matrix = get_part_matrix(
        Vec3::ZERO,
        Vec3::new(pitch.to_radians(), net_head_yaw.to_radians(), 0.0),
    );
    add_box_geom(
        vertices,
        indices,
        head_matrix,
        0.0,
        0.0,
        Vec3::new(-4.0, -8.0, -4.0),
        Vec3::new(8.0, 8.0, 8.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );

    // Torso
    let torso_matrix = get_part_matrix(Vec3::ZERO, Vec3::ZERO);
    add_box_geom(
        vertices,
        indices,
        torso_matrix,
        16.0,
        16.0,
        Vec3::new(-4.0, 0.0, -2.0),
        Vec3::new(8.0, 12.0, 4.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );

    // Zombie arms outstretched forward (-90 deg X rotation)
    let arm_rot_x = -PI / 2.0;
    let r_arm_matrix = get_part_matrix(
        Vec3::new(-5.0, 2.0, 0.0),
        Vec3::new(arm_rot_x, 0.0, 0.0),
    );
    add_box_geom(
        vertices,
        indices,
        r_arm_matrix,
        40.0,
        16.0,
        Vec3::new(-3.0, -2.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );

    let l_arm_matrix = get_part_matrix(
        Vec3::new(5.0, 2.0, 0.0),
        Vec3::new(arm_rot_x, 0.0, 0.0),
    );
    add_box_geom(
        vertices,
        indices,
        l_arm_matrix,
        32.0,
        48.0,
        Vec3::new(-1.0, -2.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );

    // Legs
    let r_leg_rot_x = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let r_leg_matrix = get_part_matrix(
        Vec3::new(-1.9, 12.0, 0.1),
        Vec3::new(r_leg_rot_x, 0.0, 0.0),
    );
    add_box_geom(
        vertices,
        indices,
        r_leg_matrix,
        0.0,
        16.0,
        Vec3::new(-2.0, 0.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );

    let l_leg_rot_x = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;
    let l_leg_matrix = get_part_matrix(
        Vec3::new(1.9, 12.0, 0.1),
        Vec3::new(l_leg_rot_x, 0.0, 0.0),
    );
    add_box_geom(
        vertices,
        indices,
        l_leg_matrix,
        16.0,
        48.0,
        Vec3::new(-2.0, 0.0, -2.0),
        Vec3::new(4.0, 12.0, 4.0),
        0.0,
        false,
        64.0,
        64.0,
        color,
    );
}

/// Builds 3D projectile and item models (Arrows, Ender Pearls, Potions).
pub fn build_object_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    object_type: u8,
    partial_ticks: f32,
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let yaw = entity.interpolated_yaw(partial_ticks);
    let pitch = entity.interpolated_pitch(partial_ticks);

    let root_pos = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);

    if object_type == 60 {
        // 3D Arrow projectile
        let rot = Quat::from_rotation_y(-yaw.to_radians()) * Quat::from_rotation_x(-pitch.to_radians());
        let transform = Mat4::from_rotation_translation(rot, root_pos);

        // Arrow Shaft: thin 3D rod (0.05 x 0.05 x 0.8 blocks)
        add_box(
            vertices, indices, transform,
            Vec3::new(-0.025, -0.025, -0.4), Vec3::new(0.025, 0.025, 0.4),
            0.0, 0.0, 2.0, 2.0, 16.0,
            [0.85, 0.75, 0.6, 1.0],
        );

        // Arrow Tip: arrowhead box
        add_box(
            vertices, indices, transform,
            Vec3::new(-0.05, -0.05, 0.35), Vec3::new(0.05, 0.05, 0.45),
            0.0, 0.0, 4.0, 4.0, 4.0,
            [0.3, 0.3, 0.35, 1.0],
        );

        // Arrow Fins / Fletching
        add_box(
            vertices, indices, transform,
            Vec3::new(-0.08, -0.08, -0.4), Vec3::new(0.08, 0.08, -0.3),
            0.0, 0.0, 4.0, 4.0, 4.0,
            [0.9, 0.9, 0.9, 1.0],
        );
    } else {
        // Floating Item / Ender Pearl / Potion: small 3D glowing jewel / sphere
        let color = if object_type == 65 {
            [0.1, 0.5, 0.4, 1.0] // Ender Pearl cyan/green
        } else if object_type == 73 {
            [0.8, 0.2, 0.7, 1.0] // Potion purple
        } else {
            [0.9, 0.85, 0.3, 1.0] // Item gold/yellow
        };

        // Spinning 3D orb
        let spin = Quat::from_rotation_y((pos.x + pos.z) as f32 * 2.0);
        let transform = Mat4::from_rotation_translation(spin, root_pos);

        add_box(
            vertices, indices, transform,
            Vec3::splat(-0.125), Vec3::splat(0.125),
            0.0, 0.0, 8.0, 8.0, 8.0,
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec3;
    use uuid::Uuid;

    #[test]
    fn test_add_box_geom_quad_count_and_uvs() {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Head box: origin=(-4, -8, -4), size=(8, 8, 8), tex=(0, 0)
        add_box_geom(
            &mut vertices,
            &mut indices,
            Mat4::IDENTITY,
            0.0,
            0.0,
            Vec3::new(-4.0, -8.0, -4.0),
            Vec3::new(8.0, 8.0, 8.0),
            0.0,
            false,
            64.0,
            64.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        assert_eq!(vertices.len(), 24);
        assert_eq!(indices.len(), 36);

        // Quad 4 is front face (indices 16..20)
        let front_quad = &vertices[16..20];
        // Check U coordinates are within [8/64, 16/64]
        for v in front_quad {
            assert!(v.uv[0] >= 8.0 / 64.0 - 0.001 && v.uv[0] <= 16.0 / 64.0 + 0.001);
            assert!(v.uv[1] >= 8.0 / 64.0 - 0.001 && v.uv[1] <= 16.0 / 64.0 + 0.001);
        }
    }

    #[test]
    fn test_build_player_model_produces_valid_geometry() {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let entity = TrackedEntity::new(
            1,
            EntityKind::Player {
                uuid: Uuid::nil(),
                name: "Steve".to_string(),
                held_item: 276,
            },
            DVec3::new(0.0, 64.0, 0.0),
            0.0,
            0.0,
        );

        build_player_model(&mut vertices, &mut indices, &entity, 0.5, [1.0, 1.0, 1.0, 1.0]);

        assert!(!vertices.is_empty());
        assert!(!indices.is_empty());
        for v in &vertices {
            assert!(!v.position[0].is_nan());
            assert!(!v.position[1].is_nan());
            assert!(!v.position[2].is_nan());
        }
    }
}
