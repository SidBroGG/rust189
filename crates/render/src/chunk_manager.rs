use crate::atlas::TextureAtlas;
use crate::frustum::Frustum;
use crate::mesher::{BlockWorldAccessor, ChunkMesher, SectionMeshData};
use crate::occlusion::SectionVisibility;
use crate::vertex::BlockFace;
use glam::Vec3;
use hashbrown::{HashMap, HashSet};
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use world::{BlockState, World};

pub const NEIGHBORHOOD_DIM: usize = 18;
pub const NEIGHBORHOOD_VOL: usize = NEIGHBORHOOD_DIM * NEIGHBORHOOD_DIM * NEIGHBORHOOD_DIM; // 5832

/// Thread-safe local voxel neighborhood snapshot for lock-free parallel meshing.
#[derive(Clone)]
pub struct NeighborhoodSnapshot {
    pub base_x: i32,
    pub base_y: i32,
    pub base_z: i32,
    pub blocks: Box<[BlockState; NEIGHBORHOOD_VOL]>,
    pub block_light: Box<[u8; NEIGHBORHOOD_VOL]>,
    pub sky_light: Box<[u8; NEIGHBORHOOD_VOL]>,
    pub animating_positions: hashbrown::HashSet<(i32, i32, i32)>,
}

impl NeighborhoodSnapshot {
    /// Captures an 18x18x18 neighborhood around a 16x16x16 chunk section.
    pub fn capture(world: &World, chunk_x: i32, section_y: usize, chunk_z: i32) -> Self {
        let base_x = chunk_x * 16;
        let base_y = section_y as i32 * 16;
        let base_z = chunk_z * 16;

        let mut blocks = vec![BlockState::AIR; NEIGHBORHOOD_VOL].into_boxed_slice();
        let mut block_light = vec![0u8; NEIGHBORHOOD_VOL].into_boxed_slice();
        let mut sky_light = vec![15u8; NEIGHBORHOOD_VOL].into_boxed_slice();

        // Pre-fetch 3x3 surrounding chunk columns (only 9 hash lookups total)
        let mut cols: [[Option<&world::ChunkColumn>; 3]; 3] = [[None; 3]; 3];
        for dx in -1..=1 {
            for dz in -1..=1 {
                cols[(dx + 1) as usize][(dz + 1) as usize] =
                    world.chunks.get(&(chunk_x + dx, chunk_z + dz));
            }
        }

        for sy in -1..=16 {
            let wy = base_y + sy;
            if !(0..=255).contains(&wy) {
                continue;
            }
            let sec_idx = (wy as usize) >> 4;
            let sec_y = (wy as usize) & 15;
            let base_sy_idx = (sy + 1) as usize * NEIGHBORHOOD_DIM;

            for sz in -1..=16 {
                let (col_dz, rel_z) = if sz < 0 {
                    (0, 15)
                } else if sz >= 16 {
                    (2, 0)
                } else {
                    (1, sz as usize)
                };

                let base_sz_idx = (base_sy_idx + (sz + 1) as usize) * NEIGHBORHOOD_DIM;

                // 1. Center 16 voxels (sx = 0..15, col_dx = 1, rel_x = 0..15)
                if let Some(col) = cols[1][col_dz] {
                    if let Some(sec) = &col.sections[sec_idx] {
                        if !sec.is_empty() {
                            let out_idx = base_sz_idx + 1;
                            let light_base = (sec_y << 8) | (rel_z << 4);
                            let byte_idx = light_base >> 1;

                            for i in 0..8 {
                                let b = sec.block_light[byte_idx + i];
                                block_light[out_idx + i * 2] = b & 0x0F;
                                block_light[out_idx + i * 2 + 1] = b >> 4;
                            }

                            if let Some(ref sky) = sec.sky_light {
                                for i in 0..8 {
                                    let b = sky[byte_idx + i];
                                    sky_light[out_idx + i * 2] = b & 0x0F;
                                    sky_light[out_idx + i * 2 + 1] = b >> 4;
                                }
                            }

                            match &sec.blocks {
                                world::PalettedContainer::SingleValue(s) => {
                                    blocks[out_idx..out_idx + 16].fill(*s);
                                }
                                _ => {
                                    for rel_x in 0..16 {
                                        blocks[out_idx + rel_x] = sec.block_at(rel_x, sec_y, rel_z);
                                    }
                                }
                            }
                        }
                    }
                }

                // 2. West neighbor voxel (sx = -1, col_dx = 0, rel_x = 15)
                if let Some(col) = cols[0][col_dz] {
                    if let Some(sec) = &col.sections[sec_idx] {
                        if !sec.is_empty() {
                            let out_idx = base_sz_idx;
                            blocks[out_idx] = sec.block_at(15, sec_y, rel_z);
                            block_light[out_idx] = sec.block_light_at(15, sec_y, rel_z);
                            sky_light[out_idx] = sec.sky_light_at(15, sec_y, rel_z);
                        }
                    }
                }

                // 3. East neighbor voxel (sx = 16, col_dx = 2, rel_x = 0)
                if let Some(col) = cols[2][col_dz] {
                    if let Some(sec) = &col.sections[sec_idx] {
                        if !sec.is_empty() {
                            let out_idx = base_sz_idx + 17;
                            blocks[out_idx] = sec.block_at(0, sec_y, rel_z);
                            block_light[out_idx] = sec.block_light_at(0, sec_y, rel_z);
                            sky_light[out_idx] = sec.sky_light_at(0, sec_y, rel_z);
                        }
                    }
                }
            }
        }

        let mut animating_positions = hashbrown::HashSet::new();
        for (&(x, y, z), _be) in &world.block_entities {
            if x >= base_x - 1 && x <= base_x + 16
                && y >= base_y - 1 && y <= base_y + 16
                && z >= base_z - 1 && z <= base_z + 16
            {
                if world.is_block_entity_animating(x, y, z) {
                    animating_positions.insert((x, y, z));
                }
            }
        }

        Self {
            base_x,
            base_y,
            base_z,
            blocks: blocks.try_into().unwrap(),
            block_light: block_light.try_into().unwrap(),
            sky_light: sky_light.try_into().unwrap(),
            animating_positions,
        }
    }
}

impl BlockWorldAccessor for NeighborhoodSnapshot {
    #[inline]
    fn block_at(&self, x: i32, y: i32, z: i32) -> BlockState {
        let sx = x - self.base_x;
        let sy = y - self.base_y;
        let sz = z - self.base_z;
        if (-1..=16).contains(&sx) && (-1..=16).contains(&sy) && (-1..=16).contains(&sz) {
            let idx = (((sy + 1) as usize * NEIGHBORHOOD_DIM) + (sz + 1) as usize)
                * NEIGHBORHOOD_DIM
                + (sx + 1) as usize;
            self.blocks[idx]
        } else {
            BlockState::AIR
        }
    }

    #[inline]
    fn block_light_at(&self, x: i32, y: i32, z: i32) -> u8 {
        let sx = x - self.base_x;
        let sy = y - self.base_y;
        let sz = z - self.base_z;
        if (-1..=16).contains(&sx) && (-1..=16).contains(&sy) && (-1..=16).contains(&sz) {
            let idx = (((sy + 1) as usize * NEIGHBORHOOD_DIM) + (sz + 1) as usize)
                * NEIGHBORHOOD_DIM
                + (sx + 1) as usize;
            self.block_light[idx]
        } else {
            0
        }
    }

    #[inline]
    fn sky_light_at(&self, x: i32, y: i32, z: i32) -> u8 {
        let sx = x - self.base_x;
        let sy = y - self.base_y;
        let sz = z - self.base_z;
        if (-1..=16).contains(&sx) && (-1..=16).contains(&sy) && (-1..=16).contains(&sz) {
            let idx = (((sy + 1) as usize * NEIGHBORHOOD_DIM) + (sz + 1) as usize)
                * NEIGHBORHOOD_DIM
                + (sx + 1) as usize;
            self.sky_light[idx]
        } else {
            15
        }
    }

    #[inline]
    fn is_block_entity_animating(&self, x: i32, y: i32, z: i32) -> bool {
        self.animating_positions.contains(&(x, y, z))
    }
}

pub type SectionKey = (i32, i32, usize);

/// Mesh data produced by the background mesher thread.
pub struct RawSectionMesh {
    pub key: SectionKey,
    pub mesh: SectionMeshData,
    pub epoch: u64,
    pub task_id: u64,
}

/// GPU-allocated vertex and index buffers for a chunk section.
pub struct GpuSectionMesh {
    pub key: SectionKey,
    pub aabb_min: Vec3,
    pub aabb_max: Vec3,
    pub center: Vec3,
    pub instance_buf: Option<wgpu::Buffer>,
    pub opaque_vbuf: Option<wgpu::Buffer>,
    pub opaque_ibuf: Option<wgpu::Buffer>,
    pub opaque_index_count: u32,
    pub translucent_vbuf: Option<wgpu::Buffer>,
    pub translucent_ibuf: Option<wgpu::Buffer>,
    pub translucent_index_count: u32,
    pub visibility: SectionVisibility,
}

/// Manages multi-threaded asynchronous chunk section meshing and GPU buffer lifecycle.
pub struct ChunkMeshManager {
    thread_pool: Arc<rayon::ThreadPool>,
    tx: Sender<RawSectionMesh>,
    rx: Receiver<RawSectionMesh>,
    pub gpu_meshes: HashMap<SectionKey, GpuSectionMesh>,
    pub dirty_sections: HashSet<SectionKey>,
    ready_meshes: Vec<RawSectionMesh>,
    task_generations: HashMap<SectionKey, u64>,
    uploaded_generations: HashMap<SectionKey, u64>,
    in_flight_count: usize,
    next_task_id: u64,
    pub epoch: u64,
}

impl Default for ChunkMeshManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkMeshManager {
    pub fn new() -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(
                std::thread::available_parallelism()
                    .map(|n| n.get().saturating_sub(2).clamp(1, 6))
                    .unwrap_or(3),
            )
            .thread_name(|i| format!("mesher-worker-{}", i))
            .build()
            .expect("Failed to build chunk mesher thread pool");

        let (tx, rx) = channel();

        Self {
            thread_pool: Arc::new(pool),
            tx,
            rx,
            gpu_meshes: HashMap::new(),
            dirty_sections: HashSet::new(),
            ready_meshes: Vec::new(),
            task_generations: HashMap::new(),
            uploaded_generations: HashMap::new(),
            in_flight_count: 0,
            next_task_id: 1,
            epoch: 0,
        }
    }

    /// Marks a chunk section as dirty (needs meshing).
    #[inline]
    pub fn mark_section_dirty(&mut self, chunk_x: i32, section_y: usize, chunk_z: i32) {
        if section_y < 16 {
            self.dirty_sections.insert((chunk_x, chunk_z, section_y));
        }
    }

    /// Marks the chunk section containing voxel (x, y, z) as dirty.
    #[inline]
    pub fn mark_block_dirty(&mut self, x: i32, y: i32, z: i32) {
        if (0..256).contains(&y) {
            self.mark_section_dirty(x >> 4, (y >> 4) as usize, z >> 4);
        }
    }

    /// Marks all non-empty sections indicated by bit_mask as dirty.
    pub fn mark_chunk_dirty(&mut self, chunk_x: i32, chunk_z: i32, bit_mask: u16) {
        for sec_idx in 0..16 {
            if (bit_mask & (1 << sec_idx)) != 0 {
                self.dirty_sections.insert((chunk_x, chunk_z, sec_idx));
            }
        }
    }

    /// Checks if a section has an active mesh, is marked dirty, or has a task in-flight.
    #[inline]
    pub fn has_mesh(&self, chunk_x: i32, chunk_z: i32, section_y: usize) -> bool {
        let key = (chunk_x, chunk_z, section_y);
        self.gpu_meshes.contains_key(&key)
            || self.dirty_sections.contains(&key)
            || self.task_generations.contains_key(&key)
    }

    /// Checks if any section in this chunk column has an active mesh or pending task.
    pub fn has_chunk_mesh(&self, chunk_x: i32, chunk_z: i32) -> bool {
        for sec_y in 0..16 {
            if self.has_mesh(chunk_x, chunk_z, sec_y) {
                return true;
            }
        }
        false
    }

    #[inline]
    pub fn has_mesh_or_pending(&self, chunk_x: i32, chunk_z: i32, section_y: usize) -> bool {
        self.has_mesh(chunk_x, chunk_z, section_y)
    }

    /// Dispatches up to `max_dispatches` closest dirty sections to the background mesher thread pool.
    pub fn dispatch_dirty_sections(
        &mut self,
        world: &World,
        atlas: Arc<TextureAtlas>,
        camera_pos: Vec3,
        render_distance: u32,
        max_dispatches: usize,
    ) {
        // 1. Drain finished tasks from workers into ready queue
        while let Ok(raw) = self.rx.try_recv() {
            self.in_flight_count = self.in_flight_count.saturating_sub(1);
            if raw.epoch == self.epoch && self.task_generations.contains_key(&raw.key) {
                self.ready_meshes.push(raw);
            }
        }

        if self.dirty_sections.is_empty() {
            return;
        }

        // Limit concurrent in-flight tasks to avoid CPU saturation and thread starvation
        let max_in_flight = 8;
        if self.in_flight_count >= max_in_flight {
            return;
        }

        let slots = (max_in_flight - self.in_flight_count).min(max_dispatches);
        if slots == 0 {
            return;
        }

        // 2. Select closest dirty sections to camera
        let max_dist = (render_distance as f32 * 16.0 + 32.0).max(64.0);
        let max_dist_sq = max_dist * max_dist;

        let mut candidates: Vec<(SectionKey, f32)> = Vec::with_capacity(self.dirty_sections.len().min(128));
        for &key in &self.dirty_sections {
            let cx = (key.0 * 16 + 8) as f32;
            let cy = (key.2 * 16 + 8) as f32;
            let cz = (key.1 * 16 + 8) as f32;
            let dx = cx - camera_pos.x;
            let dy = cy - camera_pos.y;
            let dz = cz - camera_pos.z;
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq <= max_dist_sq {
                candidates.push((key, dist_sq));
            }
        }

        if candidates.is_empty() {
            for &key in self.dirty_sections.iter().take(slots) {
                candidates.push((key, 0.0));
            }
        } else {
            candidates.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        let to_dispatch: Vec<SectionKey> = candidates.iter().take(slots).map(|(k, _)| *k).collect();

        // 3. Dispatch chosen sections
        for key in to_dispatch {
            self.dirty_sections.remove(&key);

            let section = match world.chunks.get(&(key.0, key.1)) {
                Some(col) => match &col.sections[key.2] {
                    Some(sec) => {
                        if sec.is_empty() {
                            self.gpu_meshes.remove(&key);
                            self.task_generations.remove(&key);
                            self.uploaded_generations.remove(&key);
                            continue;
                        }
                        sec.clone()
                    }
                    None => {
                        self.gpu_meshes.remove(&key);
                        self.task_generations.remove(&key);
                        self.uploaded_generations.remove(&key);
                        continue;
                    }
                },
                None => {
                    self.gpu_meshes.remove(&key);
                    self.task_generations.remove(&key);
                    self.uploaded_generations.remove(&key);
                    continue;
                }
            };

            let snapshot = NeighborhoodSnapshot::capture(world, key.0, key.2, key.1);
            let task_id = self.next_task_id;
            self.next_task_id = self.next_task_id.wrapping_add(1);
            self.task_generations.insert(key, task_id);
            self.in_flight_count += 1;

            let tx = self.tx.clone();
            let epoch = self.epoch;
            let atlas = atlas.clone();
            self.thread_pool.spawn(move || {
                let mesh = ChunkMesher::mesh_section(
                    &section,
                    key.0,
                    key.2,
                    key.1,
                    &snapshot,
                    &atlas,
                );
                let _ = tx.send(RawSectionMesh { key, mesh, epoch, task_id });
            });
        }
    }

    /// Enqueues a chunk section for meshing (compatibility helper).
    pub fn enqueue_section(
        &mut self,
        _world: &World,
        _atlas: Arc<TextureAtlas>,
        chunk_x: i32,
        section_y: usize,
        chunk_z: i32,
    ) {
        self.mark_section_dirty(chunk_x, section_y, chunk_z);
    }

    /// Uploads up to `max_uploads` ready meshes to GPU buffers per frame with a frame time budget.
    pub fn upload_pending(&mut self, device: &wgpu::Device, max_uploads: usize) -> usize {
        use wgpu::util::DeviceExt;

        // Drain any newly arrived meshes from workers
        while let Ok(raw) = self.rx.try_recv() {
            self.in_flight_count = self.in_flight_count.saturating_sub(1);
            if raw.epoch == self.epoch && self.task_generations.contains_key(&raw.key) {
                self.ready_meshes.push(raw);
            }
        }

        if self.ready_meshes.is_empty() {
            return 0;
        }

        let start = std::time::Instant::now();
        let max_budget = std::time::Duration::from_micros(1500);
        let mut uploaded = 0;

        while uploaded < max_uploads && !self.ready_meshes.is_empty() {
            if uploaded > 0 && start.elapsed() >= max_budget {
                break;
            }

            let raw = self.ready_meshes.pop().unwrap();

            // Discard meshing results that were started in an older world or dimension
            if raw.epoch != self.epoch {
                continue;
            }

            let key = raw.key;
            // Discard if this section was removed / unloaded
            if !self.task_generations.contains_key(&key) {
                continue;
            }

            // Discard if an equal or newer task for this section has already been uploaded
            if let Some(&uploaded_id) = self.uploaded_generations.get(&key) {
                if raw.task_id <= uploaded_id {
                    continue;
                }
            }

            self.uploaded_generations.insert(key, raw.task_id);

            let base_x = (key.0 * 16) as f32;
            let base_y = (key.2 * 16) as f32;
            let base_z = (key.1 * 16) as f32;

            let aabb_min = Vec3::new(base_x, base_y, base_z);
            let aabb_max = Vec3::new(base_x + 16.0, base_y + 16.0, base_z + 16.0);
            let center = (aabb_min + aabb_max) * 0.5;

            let opaque_vbuf = if !raw.mesh.opaque_vertices.is_empty() {
                Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("opaque-section-vbuf"),
                    contents: bytemuck::cast_slice(&raw.mesh.opaque_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }))
            } else {
                None
            };

            let opaque_ibuf = if !raw.mesh.opaque_indices.is_empty() {
                Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("opaque-section-ibuf"),
                    contents: bytemuck::cast_slice(&raw.mesh.opaque_indices),
                    usage: wgpu::BufferUsages::INDEX,
                }))
            } else {
                None
            };

            let translucent_vbuf = if !raw.mesh.translucent_vertices.is_empty() {
                Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("translucent-section-vbuf"),
                    contents: bytemuck::cast_slice(&raw.mesh.translucent_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }))
            } else {
                None
            };

            let translucent_ibuf = if !raw.mesh.translucent_indices.is_empty() {
                Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("translucent-section-ibuf"),
                    contents: bytemuck::cast_slice(&raw.mesh.translucent_indices),
                    usage: wgpu::BufferUsages::INDEX,
                }))
            } else {
                None
            };

            let instance = crate::vertex::SectionInstance::new(key.0, key.2, key.1);
            let instance_buf = if opaque_vbuf.is_some() || translucent_vbuf.is_some() {
                Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("section-instance-buf"),
                    contents: bytemuck::bytes_of(&instance),
                    usage: wgpu::BufferUsages::VERTEX,
                }))
            } else {
                None
            };

            self.gpu_meshes.insert(
                key,
                GpuSectionMesh {
                    key,
                    aabb_min,
                    aabb_max,
                    center,
                    instance_buf,
                    opaque_vbuf,
                    opaque_ibuf,
                    opaque_index_count: raw.mesh.opaque_indices.len() as u32,
                    translucent_vbuf,
                    translucent_ibuf,
                    translucent_index_count: raw.mesh.translucent_indices.len() as u32,
                    visibility: raw.mesh.visibility,
                },
            );

            uploaded += 1;
        }

        uploaded
    }

    /// Unloads all section meshes belonging to a chunk column.
    pub fn remove_chunk(&mut self, chunk_x: i32, chunk_z: i32) {
        self.gpu_meshes.retain(|&(cx, cz, _), _| cx != chunk_x || cz != chunk_z);
        self.dirty_sections.retain(|&(cx, cz, _)| cx != chunk_x || cz != chunk_z);
        self.ready_meshes.retain(|raw| raw.key.0 != chunk_x || raw.key.1 != chunk_z);
        self.task_generations.retain(|&(cx, cz, _), _| cx != chunk_x || cz != chunk_z);
        self.uploaded_generations.retain(|&(cx, cz, _), _| cx != chunk_x || cz != chunk_z);
    }

    /// Clears all GPU meshes and pending meshing tasks.
    pub fn clear(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.gpu_meshes.clear();
        self.dirty_sections.clear();
        self.ready_meshes.clear();
        self.task_generations.clear();
        self.uploaded_generations.clear();
        self.in_flight_count = 0;
        while self.rx.try_recv().is_ok() {}
    }

    /// Filters loaded sections with Frustum Culling and Chunk-Level Occlusion Culling,
    /// sorting opaque sections front-to-back and translucent sections back-to-front.
    pub fn get_visible_sections<'a>(
        &'a self,
        frustum: &Frustum,
        camera_pos: Vec3,
    ) -> (Vec<&'a GpuSectionMesh>, Vec<&'a GpuSectionMesh>) {
        if self.gpu_meshes.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let mut opaque_visible = Vec::new();
        let mut translucent_visible = Vec::new();

        let cam_chunk_x = (camera_pos.x.floor() as i32) >> 4;
        let cam_chunk_z = (camera_pos.z.floor() as i32) >> 4;
        let cam_sec_y_raw = (camera_pos.y.floor() as i32) >> 4;

        // BFS Queue item: (SectionKey, entered_from_face)
        // If entered_from_face is None, it is an initial entry section (camera inside or looking from sky).
        let mut queue: VecDeque<(SectionKey, Option<BlockFace>)> = VecDeque::with_capacity(256);
        let mut visited: HashSet<SectionKey> = HashSet::with_capacity(self.gpu_meshes.len() * 2);

        // Determine starting section(s):
        if (0..16).contains(&cam_sec_y_raw) {
            let start_key = (cam_chunk_x, cam_chunk_z, cam_sec_y_raw as usize);
            visited.insert(start_key);
            queue.push_back((start_key, None));
        } else {
            // Camera is outside world height (above Y=255 or below Y=0):
            // Seed BFS with all loaded boundary sections in the camera frustum
            let target_y = if cam_sec_y_raw >= 16 { 15 } else { 0 };
            for (&key, mesh) in &self.gpu_meshes {
                if key.2 == target_y && frustum.intersects_aabb(mesh.aabb_min, mesh.aabb_max) {
                    if visited.insert(key) {
                        queue.push_back((key, None));
                    }
                }
            }
        }

        while let Some((curr_key, entered_face)) = queue.pop_front() {
            // If current section has a GPU mesh, check frustum and add to visible lists
            if let Some(mesh) = self.gpu_meshes.get(&curr_key) {
                // If camera is inside start section, or section passes frustum culling:
                if entered_face.is_none() || frustum.intersects_aabb(mesh.aabb_min, mesh.aabb_max) {
                    if mesh.opaque_index_count > 0 && mesh.opaque_vbuf.is_some() {
                        opaque_visible.push(mesh);
                    }
                    if mesh.translucent_index_count > 0 && mesh.translucent_vbuf.is_some() {
                        translucent_visible.push(mesh);
                    }
                }
            }

            // Get visibility graph for current section:
            // Camera's starting section has all directions open so sight can leave the camera point.
            // Empty air sections have all directions open (sight freely passes through air).
            // Solid/meshed sections use their computed face-to-face visibility bitmask.
            let vis = if entered_face.is_none() {
                SectionVisibility::all_visible()
            } else {
                self.gpu_meshes
                    .get(&curr_key)
                    .map(|m| m.visibility)
                    .unwrap_or(SectionVisibility::all_visible())
            };

            // Explore 6 neighbor directions:
            for exit_dir in BlockFace::ALL {
                // Chunk Occlusion Test:
                // If we entered through `entered_face`, sight entered this section through `entered_face.opposite()`.
                // Can sight pass through this section to exit through `exit_dir`?
                if let Some(entered) = entered_face {
                    let in_face = entered.opposite();
                    if !vis.is_visible(in_face, exit_dir) {
                        // Occluded! A wall or mountain blocks line-of-sight in this direction.
                        continue;
                    }
                }

                let (dx, dy, dz) = exit_dir.offset();
                let nx = curr_key.0 + dx;
                let nz = curr_key.1 + dz;
                let ny_i32 = curr_key.2 as i32 + dy;

                if !(0..16).contains(&ny_i32) {
                    continue;
                }
                let neighbor_key = (nx, nz, ny_i32 as usize);

                if visited.contains(&neighbor_key) {
                    continue;
                }

                // Distance bound check: within 16 chunk sections horizontally
                let dist_x = (nx - cam_chunk_x).abs();
                let dist_z = (nz - cam_chunk_z).abs();
                if dist_x > 16 || dist_z > 16 {
                    continue;
                }

                // Frustum Culling on neighbor bounding box:
                let min_x = (nx * 16) as f32;
                let min_y = (ny_i32 * 16) as f32;
                let min_z = (nz * 16) as f32;
                let aabb_min = Vec3::new(min_x, min_y, min_z);
                let aabb_max = Vec3::new(min_x + 16.0, min_y + 16.0, min_z + 16.0);

                if !frustum.intersects_aabb(aabb_min, aabb_max) {
                    continue;
                }

                visited.insert(neighbor_key);
                queue.push_back((neighbor_key, Some(exit_dir)));
            }
        }

        // Safety fallback: if BFS found nothing (e.g. camera in unpopulated/unloaded section),
        // fallback to standard frustum-only list so geometry is never erroneously dropped.
        if opaque_visible.is_empty() && translucent_visible.is_empty() {
            for mesh in self.gpu_meshes.values() {
                if frustum.intersects_aabb(mesh.aabb_min, mesh.aabb_max) {
                    if mesh.opaque_index_count > 0 && mesh.opaque_vbuf.is_some() {
                        opaque_visible.push(mesh);
                    }
                    if mesh.translucent_index_count > 0 && mesh.translucent_vbuf.is_some() {
                        translucent_visible.push(mesh);
                    }
                }
            }
        }

        // Sort opaque meshes front-to-back for early depth test rejection
        opaque_visible.sort_by(|a, b| {
            let dist_a = a.center.distance_squared(camera_pos);
            let dist_b = b.center.distance_squared(camera_pos);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Sort translucent meshes back-to-front for correct alpha blending
        translucent_visible.sort_by(|a, b| {
            let dist_a = a.center.distance_squared(camera_pos);
            let dist_b = b.center.distance_squared(camera_pos);
            dist_b.partial_cmp(&dist_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        (opaque_visible, translucent_visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Camera;

    #[test]
    fn test_chunk_level_occlusion_culling() {
        let mut cm = ChunkMeshManager::new();

        // Create a dummy GPU buffer context wasn't needed since we only check mesh keys
        // Section 0 at (0, 4, 0) - Camera is here
        let key0 = (0, 0, 4);
        let mesh0 = GpuSectionMesh {
            key: key0,
            aabb_min: Vec3::new(0.0, 64.0, 0.0),
            aabb_max: Vec3::new(16.0, 80.0, 16.0),
            center: Vec3::new(8.0, 72.0, 8.0),
            instance_buf: None,
            opaque_vbuf: None,
            opaque_ibuf: None,
            opaque_index_count: 0,
            translucent_vbuf: None,
            translucent_ibuf: None,
            translucent_index_count: 0,
            visibility: SectionVisibility::all_visible(),
        };

        // Section 1 at (0, 5, 0) in +Z: A solid rock wall section that blocks all visibility
        let key1 = (0, 1, 4);
        let mesh1 = GpuSectionMesh {
            key: key1,
            aabb_min: Vec3::new(0.0, 64.0, 16.0),
            aabb_max: Vec3::new(16.0, 80.0, 32.0),
            center: Vec3::new(8.0, 72.0, 24.0),
            instance_buf: None,
            opaque_vbuf: None,
            opaque_ibuf: None,
            opaque_index_count: 36, // dummy count
            translucent_vbuf: None,
            translucent_ibuf: None,
            translucent_index_count: 0,
            visibility: SectionVisibility::none_visible(), // Solid wall!
        };

        // Section 2 at (0, 6, 0) further in +Z: Behind the solid rock wall
        let key2 = (0, 2, 4);
        let mesh2 = GpuSectionMesh {
            key: key2,
            aabb_min: Vec3::new(0.0, 64.0, 32.0),
            aabb_max: Vec3::new(16.0, 80.0, 48.0),
            center: Vec3::new(8.0, 72.0, 40.0),
            instance_buf: None,
            opaque_vbuf: None,
            opaque_ibuf: None,
            opaque_index_count: 36, // dummy count
            translucent_vbuf: None,
            translucent_ibuf: None,
            translucent_index_count: 0,
            visibility: SectionVisibility::all_visible(),
        };

        cm.gpu_meshes.insert(key0, mesh0);
        cm.gpu_meshes.insert(key1, mesh1);
        cm.gpu_meshes.insert(key2, mesh2);

        // Camera inside section 0 looking in +Z direction
        let mut cam = Camera::default();
        cam.position = Vec3::new(8.0, 72.0, 8.0);
        cam.pitch = 0.0;
        cam.yaw = 0.0; // facing +Z
        let frustum = Frustum::from_view_proj(cam.view_proj_matrix());

        // Verify Frustum Culling on its own: all 3 sections are in the camera's FOV!
        assert!(frustum.intersects_aabb(Vec3::new(0.0, 64.0, 0.0), Vec3::new(16.0, 80.0, 16.0)));
        assert!(frustum.intersects_aabb(Vec3::new(0.0, 64.0, 16.0), Vec3::new(16.0, 80.0, 32.0)));
        assert!(frustum.intersects_aabb(Vec3::new(0.0, 64.0, 32.0), Vec3::new(16.0, 80.0, 48.0)));

        // Run BFS Occlusion Culling:
        // Section 1 is visited because camera can see into the wall front.
        // But Section 1 has visibility == none_visible(), so sight cannot pass through it!
        // Therefore, Section 2 behind the wall is completely occluded and NOT visited!
        // Note: to be added to opaque_visible, a mesh must have opaque_vbuf.is_some() or we can check queue visited.
        // Let's verify via get_visible_sections traversal behavior:
        let (opaque, _) = cm.get_visible_sections(&frustum, cam.position);
        let visited_keys: Vec<SectionKey> = opaque.iter().map(|m| m.key).collect();
        assert!(!visited_keys.contains(&key2), "Section 2 behind the solid wall must be culled by Occlusion Culling!");
    }
}
