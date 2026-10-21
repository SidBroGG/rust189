use crate::camera::Camera;
use crate::entity_renderer::{add_box_geom, EntityUniforms, EntityVertex, GpuSkin};
use glam::{Mat4, Vec3};
use hashbrown::HashMap;
use std::f32::consts::PI;
use world::block_entity::{
    dye_color_rgb, BannerBlockEntity, BeaconBlockEntity, BlockEntity, ChestBlockEntity, ChestType,
    DoubleChestType, EnchantmentTableBlockEntity, MobSpawnerBlockEntity, PistonBlockEntity,
    SignBlockEntity, SkullBlockEntity,
};
use world::World;

/// Batch of vertices and indices to draw with a specific bound texture.
struct RenderBatch {
    texture_key: String,
    #[allow(dead_code)]
    vertex_offset: u32,
    index_offset: u32,
    index_count: u32,
}

/// Specialized renderer for all 1.8.9 dynamic block entities (TESRs).
pub struct BlockEntityRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    texture_cache: HashMap<String, GpuSkin>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    pub enable_static_baking: bool,
}

impl BlockEntityRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("block-entity-uniforms"),
            size: std::mem::size_of::<EntityUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("block-entity-uniform-bgl"),
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
            label: Some("block-entity-uniform-bg"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("block-entity-texture-bgl"),
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
            label: Some("block-entity-sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("block-entity-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("entity.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("block-entity-pipeline-layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("block-entity-pipeline"),
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
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_capacity = 32768;
        let index_capacity = 65536;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("block-entity-vbo"),
            size: (vertex_capacity * std::mem::size_of::<EntityVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("block-entity-ibo"),
            size: (index_capacity * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group_layout,
            sampler,
            texture_cache: HashMap::new(),
            vertex_buffer,
            index_buffer,
            vertex_capacity,
            index_capacity,
            enable_static_baking: true,
        }
    }

    /// Uploads or caches texture by resource key.
    pub fn ensure_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: &str,
    ) {
        if !self.texture_cache.contains_key(key) {
            let img = crate::entity_renderer::load_entity_texture(key);
            let gpu_skin = crate::entity_renderer::EntityRenderer::create_gpu_skin(
                device,
                queue,
                &self.texture_bind_group_layout,
                &self.sampler,
                &img,
                key,
            );
            self.texture_cache.insert(key.to_string(), gpu_skin);
        }
    }

    /// Main rendering entry point for all block entities in the world.
    pub fn render<'a>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
        world: &World,
        camera: &Camera,
        partial_ticks: f32,
    ) {
        if world.block_entities.is_empty() {
            return;
        }

        // 1. Upload camera uniforms
        let uniforms = EntityUniforms {
            view_proj: camera.view_proj_matrix().to_cols_array_2d(),
            camera_pos: [camera.position.x, camera.position.y, camera.position.z, 1.0],
            fog_color: [0.7, 0.8, 1.0, 1.0],
            fog_params: [64.0, 256.0, 0.0, 0.0],
            sun_direction: [0.3, 0.8, 0.5, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // 2. Build geometry sorted into texture batches
        let mut batches: Vec<RenderBatch> = Vec::new();
        let mut vertices: Vec<EntityVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut current_texture = String::new();
        let mut batch_v_offset = 0u32;
        let mut batch_i_offset = 0u32;

        let start_batch = |tex: &str, cur_tex: &mut String, batches: &mut Vec<RenderBatch>, v_len: usize, i_len: usize, b_v_off: &mut u32, b_i_off: &mut u32| {
            if *cur_tex != tex {
                if !cur_tex.is_empty() {
                    let count = (i_len as u32).saturating_sub(*b_i_off);
                    if count > 0 {
                        batches.push(RenderBatch {
                            texture_key: cur_tex.clone(),
                            vertex_offset: *b_v_off,
                            index_offset: *b_i_off,
                            index_count: count,
                        });
                    }
                }
                *cur_tex = tex.to_string();
                *b_v_off = v_len as u32;
                *b_i_off = i_len as u32;
            }
        };

        for (&(x, y, z), be) in &world.block_entities {
            let pos = Vec3::new(x as f32, y as f32, z as f32);

            // Distance culling (max 64 blocks for block entities)
            if pos.distance_squared(camera.position) > 64.0 * 64.0 {
                continue;
            }

            match be {
                BlockEntity::Chest(chest) => {
                    // Static closed chests are baked into chunk meshes for performance
                    if self.enable_static_baking && chest.is_static_closed() {
                        continue;
                    }

                    let tex = match chest.chest_type {
                        ChestType::Ender => "entity/chest/ender",
                        ChestType::Trapped => {
                            if chest.double_type != DoubleChestType::Single {
                                "entity/chest/trapped_double"
                            } else {
                                "entity/chest/trapped"
                            }
                        }
                        ChestType::Normal => {
                            if chest.double_type != DoubleChestType::Single {
                                "entity/chest/normal_double"
                            } else {
                                "entity/chest/normal"
                            }
                        }
                    };
                    start_batch(tex, &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                    self.build_chest_geom(&mut vertices, &mut indices, chest, pos, partial_ticks);
                }
                BlockEntity::Sign(sign) => {
                    start_batch("entity/sign", &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                    self.build_sign_geom(&mut vertices, &mut indices, sign, pos);

                    // Sign text rendered with ascii font
                    let has_text = sign.lines.iter().any(|l| !l.trim().is_empty());
                    if has_text {
                        start_batch("font/ascii", &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                        self.build_sign_text_geom(&mut vertices, &mut indices, sign, pos);
                    }
                }
                BlockEntity::Banner(banner) => {
                    start_batch("entity/banner_base", &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                    self.build_banner_base_geom(&mut vertices, &mut indices, banner, (x, y, z), world.time, partial_ticks);

                    // Render banner patterns
                    for (pattern_code, dye_col) in &banner.patterns {
                        let pat_name = BannerBlockEntity::pattern_texture_name(pattern_code);
                        let pat_tex = format!("entity/banner/{}", pat_name);
                        start_batch(&pat_tex, &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                        self.build_banner_pattern_geom(&mut vertices, &mut indices, banner, (x, y, z), *dye_col, world.time, partial_ticks);
                    }
                }
                BlockEntity::Skull(skull) => {
                    // Static skulls are baked into chunk meshes for performance
                    if self.enable_static_baking {
                        continue;
                    }

                    let tex = match skull.skull_type {
                        1 => "entity/skeleton/wither_skeleton",
                        2 => "entity/zombie/zombie",
                        3 => "entity/steve",
                        4 => "entity/creeper/creeper",
                        _ => "entity/skeleton/skeleton",
                    };
                    start_batch(tex, &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                    self.build_skull_geom(&mut vertices, &mut indices, skull, pos);
                }
                BlockEntity::EnchantmentTable(ench) => {
                    start_batch("entity/enchanting_table_book", &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                    self.build_enchantment_book_geom(&mut vertices, &mut indices, ench, pos, partial_ticks);
                }
                BlockEntity::Beacon(beacon) => {
                    if beacon.is_active && !beacon.segments.is_empty() {
                        start_batch("entity/beacon_beam", &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                        self.build_beacon_beam_geom(&mut vertices, &mut indices, beacon, pos, world.time, partial_ticks);
                    }
                }
                BlockEntity::EndPortal(portal) => {
                    if portal.active {
                        start_batch("environment/end_sky", &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                        self.build_end_portal_geom(&mut vertices, &mut indices, pos, world.time, camera);
                    }
                }
                BlockEntity::MobSpawner(spawner) => {
                    let tex = match spawner.mob_type.as_str() {
                        "Zombie" => "entity/zombie/zombie",
                        "Skeleton" => "entity/skeleton/skeleton",
                        "Creeper" => "entity/creeper/creeper",
                        "Spider" => "entity/spider/spider",
                        "Blaze" => "entity/blaze",
                        _ => "entity/pig/pig",
                    };
                    start_batch(tex, &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                    self.build_spawner_mob_geom(&mut vertices, &mut indices, spawner, pos, partial_ticks);
                }
                BlockEntity::Piston(piston) => {
                    start_batch("blocks/piston_top_normal", &mut current_texture, &mut batches, vertices.len(), indices.len(), &mut batch_v_offset, &mut batch_i_offset);
                    self.build_piston_geom(&mut vertices, &mut indices, piston, pos, partial_ticks);
                }
            }
        }

        // Close last batch
        if !current_texture.is_empty() {
            let count = (indices.len() as u32).saturating_sub(batch_i_offset);
            if count > 0 {
                batches.push(RenderBatch {
                    texture_key: current_texture,
                    vertex_offset: batch_v_offset,
                    index_offset: batch_i_offset,
                    index_count: count,
                });
            }
        }

        if vertices.is_empty() || indices.is_empty() {
            return;
        }

        // 3. Resize GPU buffers if necessary
        if vertices.len() > self.vertex_capacity {
            self.vertex_capacity = (vertices.len() * 3) / 2;
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("block-entity-vbo"),
                size: (self.vertex_capacity * std::mem::size_of::<EntityVertex>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if indices.len() > self.index_capacity {
            self.index_capacity = (indices.len() * 3) / 2;
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("block-entity-ibo"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        // 4. Upload vertex & index data
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

        // 5. Draw batches
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        for batch in &batches {
            self.ensure_texture(device, queue, &batch.texture_key);
            if let Some(gpu_skin) = self.texture_cache.get(&batch.texture_key) {
                render_pass.set_bind_group(1, &gpu_skin.bind_group, &[]);
                let start_idx = batch.index_offset;
                let end_idx = batch.index_offset + batch.index_count;
                render_pass.draw_indexed(start_idx..end_idx, 0, 0..1);
            }
        }
    }

    /// Builds single or double chest geometry (ModelChest / ModelLargeChest).
    fn build_chest_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        chest: &ChestBlockEntity,
        pos: Vec3,
        partial_ticks: f32,
    ) {
        let rot_y = match chest.facing {
            2 => 180.0f32.to_radians(),
            4 => 90.0f32.to_radians(),
            5 => (-90.0f32).to_radians(),
            _ => 0.0f32,
        };

        let is_double = chest.double_type != DoubleChestType::Single;
        let (tw, th) = if is_double { (128.0, 64.0) } else { (64.0, 64.0) };
        let box_w = if is_double { 30.0 } else { 14.0 };
        let knob_x = if is_double { 16.0 } else { 8.0 };

        let root_mat = Mat4::from_translation(pos + Vec3::new(0.5, 1.0, 0.5))
            * Mat4::from_scale(Vec3::new(1.0, -1.0, -1.0))
            * Mat4::from_rotation_y(rot_y)
            * Mat4::from_translation(Vec3::new(-0.5, -0.5, -0.5));

        // Base box: (ox, oy, oz) = (1, 6, 1), (w, h, d) = (box_w, 10, 14)
        add_box_geom(
            vertices,
            indices,
            root_mat * Mat4::from_scale(Vec3::splat(1.0 / 16.0)),
            0.0,
            19.0,
            Vec3::new(1.0, 6.0, 1.0),
            Vec3::new(box_w, 10.0, 14.0),
            0.0,
            false,
            tw,
            th,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Animated lid & knob
        let lid_angle = chest.lid_rotation_radians(partial_ticks);
        let lid_mat = root_mat
            * Mat4::from_translation(Vec3::new(0.0, 7.0 / 16.0, 15.0 / 16.0))
            * Mat4::from_rotation_x(lid_angle)
            * Mat4::from_translation(Vec3::new(0.0, -7.0 / 16.0, -15.0 / 16.0))
            * Mat4::from_scale(Vec3::splat(1.0 / 16.0));

        // Lid box: addBox(0, -5, -14, box_w, 5, 14) at (1, 7, 15)
        add_box_geom(
            vertices,
            indices,
            lid_mat,
            0.0,
            0.0,
            Vec3::new(1.0, 2.0, 1.0),
            Vec3::new(box_w, 5.0, 14.0),
            0.0,
            false,
            tw,
            th,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Knob box: addBox(-1, -2, -15, 2, 4, 1) at (knob_x, 7, 15)
        add_box_geom(
            vertices,
            indices,
            lid_mat,
            0.0,
            0.0,
            Vec3::new(knob_x - 1.0, 5.0, 0.0),
            Vec3::new(2.0, 4.0, 1.0),
            0.0,
            false,
            tw,
            th,
            [1.0, 1.0, 1.0, 1.0],
        );
    }

    /// Builds sign geometry (ModelSign).
    fn build_sign_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        sign: &SignBlockEntity,
        pos: Vec3,
    ) {
        let f = 0.6666667f32;
        let (root_mat, show_stick) = if !sign.is_wall {
            let rot = -(sign.rotation as f32 * 360.0 / 16.0).to_radians();
            let m = Mat4::from_translation(pos + Vec3::new(0.5, 0.75 * f, 0.5))
                * Mat4::from_rotation_y(rot)
                * Mat4::from_scale(Vec3::new(f, -f, -f));
            (m, true)
        } else {
            let rot = match sign.rotation {
                2 => 180.0f32.to_radians(),
                4 => 90.0f32.to_radians(),
                5 => (-90.0f32).to_radians(),
                _ => 0.0f32,
            };
            let m = Mat4::from_translation(pos + Vec3::new(0.5, 0.75 * f, 0.5))
                * Mat4::from_rotation_y(-rot)
                * Mat4::from_translation(Vec3::new(0.0, -0.3125, -0.4375))
                * Mat4::from_scale(Vec3::new(f, -f, -f));
            (m, false)
        };

        // Sign board: addBox(-12, -14, -1, 24, 12, 2)
        add_box_geom(
            vertices,
            indices,
            root_mat * Mat4::from_scale(Vec3::splat(1.0 / 16.0)),
            0.0,
            0.0,
            Vec3::new(-12.0, -14.0, -1.0),
            Vec3::new(24.0, 12.0, 2.0),
            0.0,
            false,
            64.0,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Sign stick: addBox(-1, -2, -1, 2, 14, 2)
        if show_stick {
            add_box_geom(
                vertices,
                indices,
                root_mat * Mat4::from_scale(Vec3::splat(1.0 / 16.0)),
                0.0,
                14.0,
                Vec3::new(-1.0, -2.0, -1.0),
                Vec3::new(2.0, 14.0, 2.0),
                0.0,
                false,
                64.0,
                32.0,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    }

    /// Renders 4 lines of sign text using 8x8 bitmap glyphs from ascii.png.
    fn build_sign_text_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        sign: &SignBlockEntity,
        pos: Vec3,
    ) {
        let f = 0.6666667f32;
        let f3 = 0.015625f32 * f;

        let root_mat = if !sign.is_wall {
            let rot = -(sign.rotation as f32 * 360.0 / 16.0).to_radians();
            Mat4::from_translation(pos + Vec3::new(0.5, 0.75 * f, 0.5))
                * Mat4::from_rotation_y(rot)
                * Mat4::from_translation(Vec3::new(0.0, 0.5 * f, -0.07 * f))
                * Mat4::from_scale(Vec3::new(f3, -f3, f3))
        } else {
            let rot = match sign.rotation {
                2 => 180.0f32.to_radians(),
                4 => 90.0f32.to_radians(),
                5 => (-90.0f32).to_radians(),
                _ => 0.0f32,
            };
            Mat4::from_translation(pos + Vec3::new(0.5, 0.75 * f, 0.5))
                * Mat4::from_rotation_y(-rot)
                * Mat4::from_translation(Vec3::new(0.0, -0.3125, -0.4375))
                * Mat4::from_translation(Vec3::new(0.0, 0.5 * f, -0.07 * f))
                * Mat4::from_scale(Vec3::new(f3, -f3, f3))
        };

        for (line_idx, line) in sign.lines.iter().enumerate() {
            let text = line.trim();
            if text.is_empty() {
                continue;
            }

            let char_width = 6.0f32;
            let total_width = text.len() as f32 * char_width;
            let start_x = -total_width / 2.0;
            let y_pos = (line_idx as f32) * 10.0 - 20.0;

            for (i, ch) in text.chars().enumerate() {
                let code = (ch as u32).min(255);
                let col = (code % 16) as f32;
                let row = (code / 16) as f32;
                let u0 = col / 16.0;
                let v0 = row / 16.0;
                let u1 = (col + 1.0) / 16.0;
                let v1 = (row + 1.0) / 16.0;

                let cx = start_x + (i as f32) * char_width;
                let cy = y_pos;
                let cw = 8.0f32;
                let ch_h = 8.0f32;

                let p0 = root_mat.transform_point3(Vec3::new(cx, cy, 0.0));
                let p1 = root_mat.transform_point3(Vec3::new(cx + cw, cy, 0.0));
                let p2 = root_mat.transform_point3(Vec3::new(cx + cw, cy + ch_h, 0.0));
                let p3 = root_mat.transform_point3(Vec3::new(cx, cy + ch_h, 0.0));

                let normal = [0.0, 0.0, -1.0];
                let color = [0.0, 0.0, 0.0, 1.0]; // Authentic sign black text

                let base_idx = vertices.len() as u32;
                vertices.push(EntityVertex { position: p0.to_array(), uv: [u0, v0], normal, color });
                vertices.push(EntityVertex { position: p1.to_array(), uv: [u1, v0], normal, color });
                vertices.push(EntityVertex { position: p2.to_array(), uv: [u1, v1], normal, color });
                vertices.push(EntityVertex { position: p3.to_array(), uv: [u0, v1], normal, color });

                indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2, base_idx, base_idx + 2, base_idx + 3]);
            }
        }
    }

    /// Builds banner stand and base cloth geometry (ModelBanner).
    fn build_banner_base_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        banner: &BannerBlockEntity,
        pos: (i32, i32, i32),
        world_time: i64,
        partial_ticks: f32,
    ) {
        let f = 0.6666667f32;
        let pos_v = Vec3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32);

        let (root_mat, show_stand) = if !banner.is_wall {
            let rot = -(banner.rotation as f32 * 360.0 / 16.0).to_radians();
            let m = Mat4::from_translation(pos_v + Vec3::new(0.5, 0.75 * f, 0.5))
                * Mat4::from_rotation_y(rot)
                * Mat4::from_scale(Vec3::new(f, -f, -f));
            (m, true)
        } else {
            let rot = match banner.rotation {
                2 => 180.0f32.to_radians(),
                4 => 90.0f32.to_radians(),
                5 => (-90.0f32).to_radians(),
                _ => 0.0f32,
            };
            let m = Mat4::from_translation(pos_v + Vec3::new(0.5, -0.25 * f, 0.5))
                * Mat4::from_rotation_y(-rot)
                * Mat4::from_translation(Vec3::new(0.0, -0.3125, -0.4375))
                * Mat4::from_scale(Vec3::new(f, -f, -f));
            (m, false)
        };

        // Stand: addBox(-1, -30, -1, 2, 42, 2)
        if show_stand {
            add_box_geom(
                vertices,
                indices,
                root_mat * Mat4::from_scale(Vec3::splat(1.0 / 16.0)),
                44.0,
                0.0,
                Vec3::new(-1.0, -30.0, -1.0),
                Vec3::new(2.0, 42.0, 2.0),
                0.0,
                false,
                64.0,
                64.0,
                [1.0, 1.0, 1.0, 1.0],
            );
        }

        // Top bar: addBox(-10, -32, -1, 20, 2, 2)
        add_box_geom(
            vertices,
            indices,
            root_mat * Mat4::from_scale(Vec3::splat(1.0 / 16.0)),
            0.0,
            42.0,
            Vec3::new(-10.0, -32.0, -1.0),
            Vec3::new(20.0, 2.0, 2.0),
            0.0,
            false,
            64.0,
            64.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Flapping banner slate cloth: addBox(-10, 0, -2, 20, 40, 1) at y = -32
        let slate_rot = BannerBlockEntity::slate_rotation_radians(pos, world_time, partial_ticks);
        let slate_mat = root_mat
            * Mat4::from_translation(Vec3::new(0.0, -32.0 / 16.0, 0.0))
            * Mat4::from_rotation_x(slate_rot)
            * Mat4::from_scale(Vec3::splat(1.0 / 16.0));

        let [r, g, b] = dye_color_rgb(banner.base_color);
        add_box_geom(
            vertices,
            indices,
            slate_mat,
            0.0,
            0.0,
            Vec3::new(-10.0, 0.0, -2.0),
            Vec3::new(20.0, 40.0, 1.0),
            0.0,
            false,
            64.0,
            64.0,
            [r, g, b, 1.0],
        );
    }

    /// Builds a single banner pattern layer on top of the slate cloth.
    fn build_banner_pattern_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        banner: &BannerBlockEntity,
        pos: (i32, i32, i32),
        dye_col: u8,
        world_time: i64,
        partial_ticks: f32,
    ) {
        let f = 0.6666667f32;
        let pos_v = Vec3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32);

        let root_mat = if !banner.is_wall {
            let rot = -(banner.rotation as f32 * 360.0 / 16.0).to_radians();
            Mat4::from_translation(pos_v + Vec3::new(0.5, 0.75 * f, 0.5))
                * Mat4::from_rotation_y(rot)
                * Mat4::from_scale(Vec3::new(f, -f, -f))
        } else {
            let rot = match banner.rotation {
                2 => 180.0f32.to_radians(),
                4 => 90.0f32.to_radians(),
                5 => (-90.0f32).to_radians(),
                _ => 0.0f32,
            };
            Mat4::from_translation(pos_v + Vec3::new(0.5, -0.25 * f, 0.5))
                * Mat4::from_rotation_y(-rot)
                * Mat4::from_translation(Vec3::new(0.0, -0.3125, -0.4375))
                * Mat4::from_scale(Vec3::new(f, -f, -f))
        };

        let slate_rot = BannerBlockEntity::slate_rotation_radians(pos, world_time, partial_ticks);
        let slate_mat = root_mat
            * Mat4::from_translation(Vec3::new(0.0, -32.0 / 16.0, 0.0))
            * Mat4::from_rotation_x(slate_rot)
            * Mat4::from_scale(Vec3::splat(1.0 / 16.0));

        let [r, g, b] = dye_color_rgb(dye_col);
        add_box_geom(
            vertices,
            indices,
            slate_mat,
            0.0,
            0.0,
            Vec3::new(-10.0, 0.0, -2.01),
            Vec3::new(20.0, 40.0, 1.02),
            0.001,
            false,
            64.0,
            64.0,
            [r, g, b, 1.0],
        );
    }

    /// Builds skull geometry (ModelSkeletonHead / ModelHumanoidHead).
    fn build_skull_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        skull: &SkullBlockEntity,
        pos: Vec3,
    ) {
        let rot_deg = skull.rotation_degrees();
        let (root_pos, rot_y) = if skull.facing == 1 {
            (pos + Vec3::new(0.5, 0.0, 0.5), rot_deg.to_radians())
        } else {
            let offset = match skull.facing {
                2 => Vec3::new(0.5, 0.25, 0.74),
                3 => Vec3::new(0.5, 0.25, 0.26),
                4 => Vec3::new(0.74, 0.25, 0.5),
                _ => Vec3::new(0.26, 0.25, 0.5),
            };
            (pos + offset, rot_deg.to_radians())
        };

        let root_mat = Mat4::from_translation(root_pos)
            * Mat4::from_rotation_y(rot_y)
            * Mat4::from_scale(Vec3::new(-1.0 / 16.0, -1.0 / 16.0, 1.0 / 16.0));

        // 8x8x8 head box: addBox(-4, -8, -4, 8, 8, 8)
        add_box_geom(
            vertices,
            indices,
            root_mat,
            0.0,
            0.0,
            Vec3::new(-4.0, -8.0, -4.0),
            Vec3::new(8.0, 8.0, 8.0),
            0.0,
            false,
            64.0,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Outer hat layer for player/zombie
        if matches!(skull.skull_type, 2 | 3) {
            add_box_geom(
                vertices,
                indices,
                root_mat,
                32.0,
                0.0,
                Vec3::new(-4.0, -8.0, -4.0),
                Vec3::new(8.0, 8.0, 8.0),
                0.25,
                false,
                64.0,
                32.0,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    }

    /// Builds Enchantment Table floating ModelBook geometry.
    fn build_enchantment_book_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        ench: &EnchantmentTableBlockEntity,
        pos: Vec3,
        partial_ticks: f32,
    ) {
        let f = ench.tick_count as f32 + partial_ticks;
        let bob = 0.1 + (f * 0.1).sin() * 0.01;
        let book_rot = ench.interpolated_rotation(partial_ticks);
        let book_spread = ench.interpolated_spread(partial_ticks);

        let root_mat = Mat4::from_translation(pos + Vec3::new(0.5, 0.75 + bob, 0.5))
            * Mat4::from_rotation_y(-book_rot)
            * Mat4::from_rotation_z(80.0f32.to_radians())
            * Mat4::from_scale(Vec3::splat(1.0 / 16.0));

        let open_angle = (1.25 + (f * 0.02).sin() * 0.1) * book_spread;

        // Right cover: addBox(-6, -5, 0, 6, 10, 0)
        let right_cover_mat = root_mat
            * Mat4::from_translation(Vec3::new(0.0, 0.0, -1.0))
            * Mat4::from_rotation_y(PI + open_angle);
        add_box_geom(
            vertices,
            indices,
            right_cover_mat,
            0.0,
            0.0,
            Vec3::new(-6.0, -5.0, 0.0),
            Vec3::new(6.0, 10.0, 0.01),
            0.0,
            false,
            64.0,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Left cover: addBox(0, -5, 0, 6, 10, 0)
        let left_cover_mat = root_mat
            * Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0))
            * Mat4::from_rotation_y(-open_angle);
        add_box_geom(
            vertices,
            indices,
            left_cover_mat,
            16.0,
            0.0,
            Vec3::new(0.0, -5.0, 0.0),
            Vec3::new(6.0, 10.0, 0.01),
            0.0,
            false,
            64.0,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Book spine: addBox(-1, -5, 0, 2, 10, 0) rotated 90 deg around Y
        let spine_mat = root_mat * Mat4::from_rotation_y(PI / 2.0);
        add_box_geom(
            vertices,
            indices,
            spine_mat,
            12.0,
            0.0,
            Vec3::new(-1.0, -5.0, 0.0),
            Vec3::new(2.0, 10.0, 0.01),
            0.0,
            false,
            64.0,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Right pages: addBox(0, -4, -0.99, 5, 8, 1)
        let right_pages_mat = root_mat
            * Mat4::from_translation(Vec3::new(open_angle.sin(), 0.0, 0.0))
            * Mat4::from_rotation_y(open_angle);
        add_box_geom(
            vertices,
            indices,
            right_pages_mat,
            0.0,
            10.0,
            Vec3::new(0.0, -4.0, -0.99),
            Vec3::new(5.0, 8.0, 1.0),
            0.0,
            false,
            64.0,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Left pages: addBox(0, -4, -0.01, 5, 8, 1)
        let left_pages_mat = root_mat
            * Mat4::from_translation(Vec3::new(open_angle.sin(), 0.0, 0.0))
            * Mat4::from_rotation_y(-open_angle);
        add_box_geom(
            vertices,
            indices,
            left_pages_mat,
            12.0,
            10.0,
            Vec3::new(0.0, -4.0, -0.01),
            Vec3::new(5.0, 8.0, 1.0),
            0.0,
            false,
            64.0,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );
    }

    /// Builds vertical Beacon beam geometry (inner rotating core + outer translucent sheath).
    fn build_beacon_beam_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        beacon: &BeaconBlockEntity,
        pos: Vec3,
        world_time: i64,
        partial_ticks: f32,
    ) {
        let t = world_time as f64 + partial_ticks as f64;
        let v_scroll = (-t * 0.2 - (-t * 0.1).floor()) as f32;
        let rot_y = (t * 0.025 * -1.5) as f32;

        let mut current_y = pos.y + 1.0;

        for seg in &beacon.segments {
            let seg_h = seg.height as f32;
            let top_y = current_y + seg_h;
            let [r, g, b] = seg.color;

            // 1. Inner core beam (radius 0.2, rotating)
            let core_mat = Mat4::from_translation(Vec3::new(pos.x + 0.5, 0.0, pos.z + 0.5))
                * Mat4::from_rotation_y(rot_y);

            let r_inner = 0.2f32;
            let v_top = seg_h * 2.5 + v_scroll;
            let corners = [
                core_mat.transform_point3(Vec3::new(-r_inner, current_y, -r_inner)),
                core_mat.transform_point3(Vec3::new(r_inner, current_y, -r_inner)),
                core_mat.transform_point3(Vec3::new(r_inner, current_y, r_inner)),
                core_mat.transform_point3(Vec3::new(-r_inner, current_y, r_inner)),
                core_mat.transform_point3(Vec3::new(-r_inner, top_y, -r_inner)),
                core_mat.transform_point3(Vec3::new(r_inner, top_y, -r_inner)),
                core_mat.transform_point3(Vec3::new(r_inner, top_y, r_inner)),
                core_mat.transform_point3(Vec3::new(-r_inner, top_y, r_inner)),
            ];

            let quads = [
                (corners[0], corners[1], corners[5], corners[4]),
                (corners[1], corners[2], corners[6], corners[5]),
                (corners[2], corners[3], corners[7], corners[6]),
                (corners[3], corners[0], corners[4], corners[7]),
            ];

            for (p0, p1, p2, p3) in quads {
                let base = vertices.len() as u32;
                let col = [r, g, b, 1.0];
                let norm = [0.0, 1.0, 0.0];
                vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0, v_scroll], normal: norm, color: col });
                vertices.push(EntityVertex { position: p1.to_array(), uv: [1.0, v_scroll], normal: norm, color: col });
                vertices.push(EntityVertex { position: p2.to_array(), uv: [1.0, v_top], normal: norm, color: col });
                vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0, v_top], normal: norm, color: col });
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }

            // 2. Outer sheath beam (radius 0.3, alpha 0.125)
            let r_outer = 0.3f32;
            let o_corners = [
                Vec3::new(pos.x + 0.5 - r_outer, current_y, pos.z + 0.5 - r_outer),
                Vec3::new(pos.x + 0.5 + r_outer, current_y, pos.z + 0.5 - r_outer),
                Vec3::new(pos.x + 0.5 + r_outer, current_y, pos.z + 0.5 + r_outer),
                Vec3::new(pos.x + 0.5 - r_outer, current_y, pos.z + 0.5 + r_outer),
                Vec3::new(pos.x + 0.5 - r_outer, top_y, pos.z + 0.5 - r_outer),
                Vec3::new(pos.x + 0.5 + r_outer, top_y, pos.z + 0.5 - r_outer),
                Vec3::new(pos.x + 0.5 + r_outer, top_y, pos.z + 0.5 + r_outer),
                Vec3::new(pos.x + 0.5 - r_outer, top_y, pos.z + 0.5 + r_outer),
            ];

            let o_quads = [
                (o_corners[0], o_corners[1], o_corners[5], o_corners[4]),
                (o_corners[1], o_corners[2], o_corners[6], o_corners[5]),
                (o_corners[2], o_corners[3], o_corners[7], o_corners[6]),
                (o_corners[3], o_corners[0], o_corners[4], o_corners[7]),
            ];

            for (p0, p1, p2, p3) in o_quads {
                let base = vertices.len() as u32;
                let col = [r, g, b, 0.125];
                let norm = [0.0, 1.0, 0.0];
                vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0, v_scroll], normal: norm, color: col });
                vertices.push(EntityVertex { position: p1.to_array(), uv: [1.0, v_scroll], normal: norm, color: col });
                vertices.push(EntityVertex { position: p2.to_array(), uv: [1.0, v_top], normal: norm, color: col });
                vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0, v_top], normal: norm, color: col });
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }

            current_y = top_y;
        }
    }

    /// Builds End Portal planar parallax geometry.
    fn build_end_portal_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        pos: Vec3,
        world_time: i64,
        camera: &Camera,
    ) {
        let y = pos.y + 0.75;
        let time = (world_time % 700000) as f32 / 700000.0;
        let u_shift = camera.position.x * 0.05 + time;
        let v_shift = camera.position.z * 0.05;

        let p0 = Vec3::new(pos.x, y, pos.z);
        let p1 = Vec3::new(pos.x + 1.0, y, pos.z);
        let p2 = Vec3::new(pos.x + 1.0, y, pos.z + 1.0);
        let p3 = Vec3::new(pos.x, y, pos.z + 1.0);

        let base = vertices.len() as u32;
        let norm = [0.0, 1.0, 0.0];
        let color = [0.9, 0.7, 1.0, 1.0];

        vertices.push(EntityVertex { position: p0.to_array(), uv: [u_shift, v_shift], normal: norm, color });
        vertices.push(EntityVertex { position: p1.to_array(), uv: [u_shift + 1.0, v_shift], normal: norm, color });
        vertices.push(EntityVertex { position: p2.to_array(), uv: [u_shift + 1.0, v_shift + 1.0], normal: norm, color });
        vertices.push(EntityVertex { position: p3.to_array(), uv: [u_shift, v_shift + 1.0], normal: norm, color });

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Builds miniature spinning mob inside spawner cage.
    fn build_spawner_mob_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        spawner: &MobSpawnerBlockEntity,
        pos: Vec3,
        partial_ticks: f32,
    ) {
        let rot_y = spawner.interpolated_rotation_degrees(partial_ticks).to_radians();
        let scale = 0.4375f32 * (1.0 / 16.0);

        let root_mat = Mat4::from_translation(pos + Vec3::new(0.5, 0.4, 0.5))
            * Mat4::from_rotation_y(rot_y)
            * Mat4::from_rotation_x((-30.0f32).to_radians())
            * Mat4::from_scale(Vec3::splat(scale));

        // Miniature mob: Head + Body (generic mob model)
        // Head: (8, 8, 8)
        add_box_geom(
            vertices,
            indices,
            root_mat,
            0.0,
            0.0,
            Vec3::new(-4.0, 0.0, -4.0),
            Vec3::new(8.0, 8.0, 8.0),
            0.0,
            false,
            64.0,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        // Body: (6, 8, 6)
        add_box_geom(
            vertices,
            indices,
            root_mat,
            16.0,
            16.0,
            Vec3::new(-3.0, -8.0, -3.0),
            Vec3::new(6.0, 8.0, 6.0),
            0.0,
            false,
            64.0,
            32.0,
            [1.0, 1.0, 1.0, 1.0],
        );
    }

    /// Builds extending/retracting piston head geometry.
    fn build_piston_geom(
        &self,
        vertices: &mut Vec<EntityVertex>,
        indices: &mut Vec<u32>,
        piston: &PistonBlockEntity,
        pos: Vec3,
        partial_ticks: f32,
    ) {
        let offset = piston.get_offset(partial_ticks);
        let head_mat = Mat4::from_translation(pos + offset) * Mat4::from_scale(Vec3::splat(1.0 / 16.0));

        // Piston head top plate: (16, 4, 16)
        add_box_geom(
            vertices,
            indices,
            head_mat,
            0.0,
            0.0,
            Vec3::new(0.0, 12.0, 0.0),
            Vec3::new(16.0, 4.0, 16.0),
            0.0,
            false,
            64.0,
            64.0,
            [1.0, 1.0, 1.0, 1.0],
        );
        // Piston rod: (4, 12, 4)
        add_box_geom(
            vertices,
            indices,
            head_mat,
            16.0,
            16.0,
            Vec3::new(6.0, 0.0, 6.0),
            Vec3::new(4.0, 12.0, 4.0),
            0.0,
            false,
            64.0,
            64.0,
            [1.0, 1.0, 1.0, 1.0],
        );
    }
}
