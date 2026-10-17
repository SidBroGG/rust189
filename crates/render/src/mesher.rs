use crate::atlas::TextureAtlas;
use crate::block_model::BakedModelRegistry;
use crate::occlusion::{SectionVisibility, VisGraph};
use crate::vertex::{BlockFace, BlockVertex};
use world::{BlockState, ChunkSection, SECTION_DEPTH, SECTION_HEIGHT, SECTION_WIDTH};

/// Raw vertex and index buffers for an individual chunk section.
#[derive(Debug, Default, Clone)]
pub struct SectionMeshData {
    pub opaque_vertices: Vec<BlockVertex>,
    pub opaque_indices: Vec<u32>,
    pub translucent_vertices: Vec<BlockVertex>,
    pub translucent_indices: Vec<u32>,
    pub visibility: SectionVisibility,
}

impl SectionMeshData {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.opaque_indices.is_empty() && self.translucent_indices.is_empty()
    }

    #[inline]
    pub fn total_quads(&self) -> usize {
        (self.opaque_indices.len() + self.translucent_indices.len()) / 6
    }
}

/// Provides neighborhood block and lighting queries for a section being meshed.
pub trait BlockWorldAccessor {
    fn block_at(&self, x: i32, y: i32, z: i32) -> BlockState;
    fn block_light_at(&self, x: i32, y: i32, z: i32) -> u8;
    fn sky_light_at(&self, x: i32, y: i32, z: i32) -> u8;
}

/// High-performance chunk mesher with assets-based models, blockstates, textures,
/// Smooth Lighting, and Ambient Occlusion.
pub struct ChunkMesher;

impl ChunkMesher {
    /// Meshes a single 16x16x16 chunk section using Binary Greedy Meshing.
    pub fn mesh_section<W: BlockWorldAccessor>(
        section: &ChunkSection,
        chunk_x: i32,
        section_y: usize,
        chunk_z: i32,
        world: &W,
        atlas: &TextureAtlas,
    ) -> SectionMeshData {
        let mut mesh = SectionMeshData::default();

        let base_x = chunk_x * 16;
        let base_y = (section_y as i32) * 16;
        let base_z = chunk_z * 16;

        // Bitmask tracking which faces (0..5) on each voxel in [0..4096] were already greedy-meshed
        let mut meshed_faces = [0u8; SECTION_HEIGHT * SECTION_DEPTH * SECTION_WIDTH];

        // -------------------------------------------------------------
        // PASS 1: Binary Greedy Meshing for coplanar standard cube faces
        // -------------------------------------------------------------
        for face in BlockFace::ALL {
            let (dx, dy, dz) = face.offset();

            for d in 0..16usize {
                let mut keys = [0u64; 256];

                for v in 0..16usize {
                    for u in 0..16usize {
                        let (lx, ly, lz) = match face {
                            BlockFace::Up | BlockFace::Down => (u, d, v),
                            BlockFace::North | BlockFace::South => (u, v, d),
                            BlockFace::West | BlockFace::East => (d, v, u),
                        };

                        let block = section.block_at(lx, ly, lz);
                        if block.is_air() {
                            continue;
                        }

                        let wx = base_x + lx as i32;
                        let wy = base_y + ly as i32;
                        let wz = base_z + lz as i32;

                        let model = match atlas.models.get_model(block, wx, wy, wz, world) {
                            Some(m) => m,
                            None => continue,
                        };

                        if !model.is_standard_cube() {
                            continue;
                        }

                        // Face culling
                        let nx = wx + dx;
                        let ny = wy + dy;
                        let nz = wz + dz;
                        let neighbor = world.block_at(nx, ny, nz);
                        if neighbor.is_opaque() {
                            continue;
                        }

                        // Compute corner AO and lighting
                        let (s_left, s_right, s_down, s_up) = face.tangents();
                        let (ao0, light0) = Self::compute_corner_ao_light(nx, ny, nz, s_left, s_down, world);
                        let (ao1, light1) = Self::compute_corner_ao_light(nx, ny, nz, s_right, s_down, world);
                        let (ao2, light2) = Self::compute_corner_ao_light(nx, ny, nz, s_right, s_up, world);
                        let (ao3, light3) = Self::compute_corner_ao_light(nx, ny, nz, s_left, s_up, world);

                        // Only merge if AO and lighting are uniform across the quad
                        if (ao0 == ao1 && ao1 == ao2 && ao2 == ao3) && (light0 == light1 && light1 == light2 && light2 == light3) {
                            let quad = match model.get_face_quad(face) {
                                Some(q) => q,
                                None => continue,
                            };
                            let tint = if quad.tint_index >= 0 {
                                BakedModelRegistry::get_block_tint(block)
                            } else {
                                [1.0, 1.0, 1.0]
                            };

                            let cr = (tint[0].clamp(0.0, 1.0) * 255.0).round() as u64;
                            let cg = (tint[1].clamp(0.0, 1.0) * 255.0).round() as u64;
                            let cb = (tint[2].clamp(0.0, 1.0) * 255.0).round() as u64;

                            let key = (block.raw() as u64)
                                | ((ao0 as u64) << 16)
                                | ((light0[0] as u64) << 18)
                                | ((light0[1] as u64) << 22)
                                | (cr << 26)
                                | (cg << 34)
                                | (cb << 42)
                                | (1u64 << 63);

                            keys[v * 16 + u] = key;
                        }
                    }
                }

                // 2D Binary Greedy Meshing on 16x16 slice with row masks
                for v in 0..16usize {
                    for u in 0..16usize {
                        let key = keys[v * 16 + u];
                        if key == 0 {
                            continue;
                        }

                        // Expand along u
                        let mut w = 1usize;
                        while u + w < 16 && keys[v * 16 + (u + w)] == key {
                            w += 1;
                        }

                        // Expand along v using bitmask matching
                        let mut h = 1usize;
                        while v + h < 16 {
                            let mut matches = true;
                            for col in u..(u + w) {
                                if keys[(v + h) * 16 + col] != key {
                                    matches = false;
                                    break;
                                }
                            }
                            if matches {
                                h += 1;
                            } else {
                                break;
                            }
                        }

                        // Clear merged cells and mark as meshed
                        for r in v..(v + h) {
                            for col in u..(u + w) {
                                keys[r * 16 + col] = 0;
                                let (ml_x, ml_y, ml_z) = match face {
                                    BlockFace::Up | BlockFace::Down => (col, d, r),
                                    BlockFace::North | BlockFace::South => (col, r, d),
                                    BlockFace::West | BlockFace::East => (d, r, col),
                                };
                                let b_idx = (ml_y * 16 + ml_z) * 16 + ml_x;
                                meshed_faces[b_idx] |= 1 << face.index();
                            }
                        }

                        // Emit merged quad
                        Self::emit_greedy_quad(
                            &mut mesh.opaque_vertices,
                            &mut mesh.opaque_indices,
                            face,
                            d,
                            u,
                            v,
                            w,
                            h,
                            key,
                            atlas,
                            base_x,
                            base_y,
                            base_z,
                            world,
                        );
                    }
                }
            }
        }

        // -------------------------------------------------------------
        // PASS 2: Custom geometry, translucent blocks, and non-uniform AO faces
        // -------------------------------------------------------------
        for ly in 0..SECTION_HEIGHT {
            for lz in 0..SECTION_DEPTH {
                for lx in 0..SECTION_WIDTH {
                    let b_idx = (ly * 16 + lz) * 16 + lx;
                    let block = section.block_at(lx, ly, lz);
                    if block.is_air() {
                        continue;
                    }

                    let wx = base_x + lx as i32;
                    let wy = base_y + ly as i32;
                    let wz = base_z + lz as i32;

                    let model = match atlas.models.get_model(block, wx, wy, wz, world) {
                        Some(m) => m,
                        None => continue,
                    };

                    let is_translucent = model.is_translucent;
                    let block_id = block.id();
                    let is_leaf = matches!(block_id, 18 | 161);

                    let (target_vertices, target_indices) = if is_translucent {
                        (
                            &mut mesh.translucent_vertices,
                            &mut mesh.translucent_indices,
                        )
                    } else {
                        (&mut mesh.opaque_vertices, &mut mesh.opaque_indices)
                    };

                    let tint_color = BakedModelRegistry::get_block_tint(block);

                    for quad in &model.quads {
                        // Skip if already greedily meshed in Pass 1
                        if quad.cull_face == Some(quad.face) && (meshed_faces[b_idx] & (1 << quad.face.index())) != 0 {
                            continue;
                        }

                        // Face culling check
                        let cull_face = quad.cull_face.or_else(|| if is_leaf { Some(quad.face) } else { None });
                        if let Some(cull_face) = cull_face {
                            let (dx, dy, dz) = cull_face.offset();
                            let nx = wx + dx;
                            let ny = wy + dy;
                            let nz = wz + dz;
                            let neighbor = world.block_at(nx, ny, nz);

                            if neighbor.is_opaque() {
                                continue;
                            }
                            if is_translucent && neighbor.id() == block_id {
                                continue;
                            }
                            if is_leaf && matches!(neighbor.id(), 18 | 161) {
                                continue;
                            }
                        }

                        let quad_tint = if quad.tint_index >= 0 {
                            tint_color
                        } else {
                            [1.0, 1.0, 1.0]
                        };

                        let base_idx = target_vertices.len() as u32;

                        if !quad.shade || !model.ambient_occlusion {
                            // Flat / unshaded lighting
                            let bl = world.block_light_at(wx, wy, wz);
                            let sl = world.sky_light_at(wx, wy, wz);
                            let light = [bl as f32 / 15.0, sl as f32 / 15.0];
                            let ao = 3u8;
                            let normal_index = quad.face.index();

                            for i in 0..4 {
                                let local_pos = [
                                    lx as f32 + quad.positions[i][0],
                                    ly as f32 + quad.positions[i][1],
                                    lz as f32 + quad.positions[i][2],
                                ];
                                target_vertices.push(BlockVertex::pack(
                                    local_pos,
                                    quad.uvs[i],
                                    light,
                                    ao,
                                    quad_tint,
                                    normal_index,
                                    (0, 0),
                                    false,
                                ));
                            }

                            target_indices.extend_from_slice(&[
                                base_idx,
                                base_idx + 1,
                                base_idx + 2,
                                base_idx,
                                base_idx + 2,
                                base_idx + 3,
                            ]);
                        } else {
                            // Smooth natural lighting & AO
                            let (dx, dy, dz) = quad.face.offset();
                            let nx = wx + dx;
                            let ny = wy + dy;
                            let nz = wz + dz;
                            let (s_left, s_right, s_down, s_up) = quad.face.tangents();

                            let (ao0, light0) = Self::compute_corner_ao_light(nx, ny, nz, s_left, s_down, world);
                            let (ao1, light1) = Self::compute_corner_ao_light(nx, ny, nz, s_right, s_down, world);
                            let (ao2, light2) = Self::compute_corner_ao_light(nx, ny, nz, s_right, s_up, world);
                            let (ao3, light3) = Self::compute_corner_ao_light(nx, ny, nz, s_left, s_up, world);

                            let normal_index = quad.face.index();

                            let aos = [ao0, ao1, ao2, ao3];
                            let lights = [
                                [light0[0] as f32 / 15.0, light0[1] as f32 / 15.0],
                                [light1[0] as f32 / 15.0, light1[1] as f32 / 15.0],
                                [light2[0] as f32 / 15.0, light2[1] as f32 / 15.0],
                                [light3[0] as f32 / 15.0, light3[1] as f32 / 15.0],
                            ];

                            for i in 0..4 {
                                let local_pos = [
                                    lx as f32 + quad.positions[i][0],
                                    ly as f32 + quad.positions[i][1],
                                    lz as f32 + quad.positions[i][2],
                                ];
                                target_vertices.push(BlockVertex::pack(
                                    local_pos,
                                    quad.uvs[i],
                                    lights[i],
                                    aos[i],
                                    quad_tint,
                                    normal_index,
                                    (0, 0),
                                    false,
                                ));
                            }

                            // Diagonal flip for anisotropic AO artifact elimination
                            if (ao0 as u32) + (ao2 as u32) > (ao1 as u32) + (ao3 as u32) {
                                target_indices.extend_from_slice(&[
                                    base_idx,
                                    base_idx + 1,
                                    base_idx + 2,
                                    base_idx,
                                    base_idx + 2,
                                    base_idx + 3,
                                ]);
                            } else {
                                target_indices.extend_from_slice(&[
                                    base_idx + 1,
                                    base_idx + 2,
                                    base_idx + 3,
                                    base_idx + 1,
                                    base_idx + 3,
                                    base_idx,
                                ]);
                            }
                        }
                    }
                }
            }
        }

        mesh.visibility = VisGraph::compute_visibility(section);
        mesh
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_greedy_quad<W: BlockWorldAccessor>(
        target_vertices: &mut Vec<BlockVertex>,
        target_indices: &mut Vec<u32>,
        face: BlockFace,
        d: usize,
        u: usize,
        v: usize,
        w: usize,
        h: usize,
        key: u64,
        atlas: &TextureAtlas,
        base_x: i32,
        base_y: i32,
        base_z: i32,
        world: &W,
    ) {
        let raw_block = (key & 0xFFFF) as u16;
        let block = BlockState::from_raw(raw_block);
        let ao = ((key >> 16) & 3) as u8;
        let bl = ((key >> 18) & 15) as u8;
        let sl = ((key >> 22) & 15) as u8;
        let cr = ((key >> 26) & 255) as u8;
        let cg = ((key >> 34) & 255) as u8;
        let cb = ((key >> 42) & 255) as u8;

        let tint = [cr as f32 / 255.0, cg as f32 / 255.0, cb as f32 / 255.0];
        let light = [bl as f32 / 15.0, sl as f32 / 15.0];

        let (wx, wy, wz) = match face {
            BlockFace::Up | BlockFace::Down => (base_x + u as i32, base_y + d as i32, base_z + v as i32),
            BlockFace::North | BlockFace::South => (base_x + u as i32, base_y + v as i32, base_z + d as i32),
            BlockFace::West | BlockFace::East => (base_x + d as i32, base_y + v as i32, base_z + u as i32),
        };

        let model = match atlas.models.get_model(block, wx, wy, wz, world) {
            Some(m) => m,
            None => return,
        };

        let quad = match model.get_face_quad(face) {
            Some(q) => q,
            None => return,
        };

        // If unmerged 1x1 standard quad: use exact canonical UVs directly without shader tiling!
        if w == 1 && h == 1 {
            let (v0, v1, v2, v3) = match face {
                BlockFace::Up => {
                    let y = (d + 1) as f32;
                    (
                        [u as f32, y, (v + 1) as f32],
                        [(u + 1) as f32, y, (v + 1) as f32],
                        [(u + 1) as f32, y, v as f32],
                        [u as f32, y, v as f32],
                    )
                }
                BlockFace::Down => {
                    let y = d as f32;
                    (
                        [u as f32, y, v as f32],
                        [(u + 1) as f32, y, v as f32],
                        [(u + 1) as f32, y, (v + 1) as f32],
                        [u as f32, y, (v + 1) as f32],
                    )
                }
                BlockFace::North => {
                    let z = d as f32;
                    (
                        [(u + 1) as f32, v as f32, z],
                        [u as f32, v as f32, z],
                        [u as f32, (v + 1) as f32, z],
                        [(u + 1) as f32, (v + 1) as f32, z],
                    )
                }
                BlockFace::South => {
                    let z = (d + 1) as f32;
                    (
                        [u as f32, v as f32, z],
                        [(u + 1) as f32, v as f32, z],
                        [(u + 1) as f32, (v + 1) as f32, z],
                        [u as f32, (v + 1) as f32, z],
                    )
                }
                BlockFace::West => {
                    let x = d as f32;
                    (
                        [x, v as f32, u as f32],
                        [x, v as f32, (u + 1) as f32],
                        [x, (v + 1) as f32, (u + 1) as f32],
                        [x, (v + 1) as f32, u as f32],
                    )
                }
                BlockFace::East => {
                    let x = (d + 1) as f32;
                    (
                        [(x), v as f32, (u + 1) as f32],
                        [(x), v as f32, u as f32],
                        [(x), (v + 1) as f32, u as f32],
                        [(x), (v + 1) as f32, (u + 1) as f32],
                    )
                }
            };
            let normal_idx = face.index();
            let base_idx = target_vertices.len() as u32;

            target_vertices.push(BlockVertex::pack(v0, quad.uvs[0], light, ao, tint, normal_idx, (0, 0), false));
            target_vertices.push(BlockVertex::pack(v1, quad.uvs[1], light, ao, tint, normal_idx, (0, 0), false));
            target_vertices.push(BlockVertex::pack(v2, quad.uvs[2], light, ao, tint, normal_idx, (0, 0), false));
            target_vertices.push(BlockVertex::pack(v3, quad.uvs[3], light, ao, tint, normal_idx, (0, 0), false));

            target_indices.extend_from_slice(&[
                base_idx,
                base_idx + 1,
                base_idx + 2,
                base_idx,
                base_idx + 2,
                base_idx + 3,
            ]);
            return;
        }

        // True minimum UV coordinates in the atlas for merged quads
        let u_min = quad.uvs.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
        let v_min = quad.uvs.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
        let uv_min = [u_min, v_min];

        // 4 canonical vertices in counter-clockwise order matching block_model.rs
        let (v0, v1, v2, v3, uv0, uv1, uv2, uv3) = match face {
            BlockFace::Up => {
                let y = (d + 1) as f32;
                (
                    [u as f32, y, (v + h) as f32],
                    [(u + w) as f32, y, (v + h) as f32],
                    [(u + w) as f32, y, v as f32],
                    [u as f32, y, v as f32],
                    (0, h as u8),
                    (w as u8, h as u8),
                    (w as u8, 0),
                    (0, 0),
                )
            }
            BlockFace::Down => {
                let y = d as f32;
                (
                    [u as f32, y, v as f32],
                    [(u + w) as f32, y, v as f32],
                    [(u + w) as f32, y, (v + h) as f32],
                    [u as f32, y, (v + h) as f32],
                    (0, 0),
                    (w as u8, 0),
                    (w as u8, h as u8),
                    (0, h as u8),
                )
            }
            BlockFace::North => {
                let z = d as f32;
                (
                    [(u + w) as f32, v as f32, z],
                    [u as f32, v as f32, z],
                    [u as f32, (v + h) as f32, z],
                    [(u + w) as f32, (v + h) as f32, z],
                    (w as u8, h as u8),
                    (0, h as u8),
                    (0, 0),
                    (w as u8, 0),
                )
            }
            BlockFace::South => {
                let z = (d + 1) as f32;
                (
                    [u as f32, v as f32, z],
                    [(u + w) as f32, v as f32, z],
                    [(u + w) as f32, (v + h) as f32, z],
                    [u as f32, (v + h) as f32, z],
                    (0, h as u8),
                    (w as u8, h as u8),
                    (w as u8, 0),
                    (0, 0),
                )
            }
            BlockFace::West => {
                let x = d as f32;
                (
                    [x, v as f32, u as f32],
                    [x, v as f32, (u + w) as f32],
                    [x, (v + h) as f32, (u + w) as f32],
                    [x, (v + h) as f32, u as f32],
                    (0, h as u8),
                    (w as u8, h as u8),
                    (w as u8, 0),
                    (0, 0),
                )
            }
            BlockFace::East => {
                let x = (d + 1) as f32;
                (
                    [(x), v as f32, (u + w) as f32],
                    [(x), v as f32, u as f32],
                    [(x), (v + h) as f32, u as f32],
                    [(x), (v + h) as f32, (u + w) as f32],
                    (w as u8, h as u8),
                    (0, h as u8),
                    (0, 0),
                    (w as u8, 0),
                )
            }
        };

        let normal_idx = face.index();
        let base_idx = target_vertices.len() as u32;

        target_vertices.push(BlockVertex::pack(v0, uv_min, light, ao, tint, normal_idx, uv0, true));
        target_vertices.push(BlockVertex::pack(v1, uv_min, light, ao, tint, normal_idx, uv1, true));
        target_vertices.push(BlockVertex::pack(v2, uv_min, light, ao, tint, normal_idx, uv2, true));
        target_vertices.push(BlockVertex::pack(v3, uv_min, light, ao, tint, normal_idx, uv3, true));

        target_indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }

    #[inline]
    fn compute_corner_ao_light<W: BlockWorldAccessor>(
        nx: i32,
        ny: i32,
        nz: i32,
        side1: (i32, i32, i32),
        side2: (i32, i32, i32),
        world: &W,
    ) -> (u8, [u8; 2]) {
        let b1 = world
            .block_at(nx + side1.0, ny + side1.1, nz + side1.2)
            .is_opaque();
        let b2 = world
            .block_at(nx + side2.0, ny + side2.1, nz + side2.2)
            .is_opaque();
        let corner = world
            .block_at(
                nx + side1.0 + side2.0,
                ny + side1.1 + side2.1,
                nz + side1.2 + side2.2,
            )
            .is_opaque();

        let ao: u8 = if b1 && b2 {
            0
        } else {
            3 - (b1 as u8 + b2 as u8 + corner as u8)
        };

        let center_bl = world.block_light_at(nx, ny, nz) as u32;
        let center_sl = world.sky_light_at(nx, ny, nz) as u32;

        let s1_bl = world.block_light_at(nx + side1.0, ny + side1.1, nz + side1.2) as u32;
        let s1_sl = world.sky_light_at(nx + side1.0, ny + side1.1, nz + side1.2) as u32;

        let s2_bl = world.block_light_at(nx + side2.0, ny + side2.1, nz + side2.2) as u32;
        let s2_sl = world.sky_light_at(nx + side2.0, ny + side2.1, nz + side2.2) as u32;

        let c_bl = world.block_light_at(
            nx + side1.0 + side2.0,
            ny + side1.1 + side2.1,
            nz + side1.2 + side2.2,
        ) as u32;
        let c_sl = world.sky_light_at(
            nx + side1.0 + side2.0,
            ny + side1.1 + side2.1,
            nz + side1.2 + side2.2,
        ) as u32;

        let avg_bl = (((center_bl + s1_bl + s2_bl + c_bl) + 2) / 4).min(15) as u8;
        let avg_sl = (((center_sl + s1_sl + s2_sl + c_sl) + 2) / 4).min(15) as u8;

        (ao, [avg_bl, avg_sl])
    }
}

impl BlockWorldAccessor for world::World {
    #[inline]
    fn block_at(&self, x: i32, y: i32, z: i32) -> BlockState {
        self.block_at(x, y, z)
    }

    #[inline]
    fn block_light_at(&self, x: i32, y: i32, z: i32) -> u8 {
        self.block_light_at(x, y, z)
    }

    #[inline]
    fn sky_light_at(&self, x: i32, y: i32, z: i32) -> u8 {
        self.sky_light_at(x, y, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::time::Instant;

    struct TestWorld {
        blocks: hashbrown::HashMap<(i32, i32, i32), BlockState>,
    }

    impl BlockWorldAccessor for TestWorld {
        fn block_at(&self, x: i32, y: i32, z: i32) -> BlockState {
            self.blocks
                .get(&(x, y, z))
                .copied()
                .unwrap_or(BlockState::AIR)
        }

        fn block_light_at(&self, _x: i32, _y: i32, _z: i32) -> u8 {
            0
        }

        fn sky_light_at(&self, _x: i32, _y: i32, _z: i32) -> u8 {
            15
        }
    }

    #[test]
    fn test_single_block_meshing_produces_6_faces() {
        let atlas = TextureAtlas::build(Path::new("assets/minecraft/textures/blocks"));
        let mut section = ChunkSection::new(true);
        section.set_block(8, 8, 8, BlockState::new(1, 0)); // Stone

        let mut world = TestWorld {
            blocks: hashbrown::HashMap::new(),
        };
        world.blocks.insert((8, 8, 8), BlockState::new(1, 0));

        let mesh = ChunkMesher::mesh_section(&section, 0, 0, 0, &world, &atlas);
        assert_eq!(mesh.opaque_vertices.len(), 24); // 6 faces * 4 vertices
        assert_eq!(mesh.opaque_indices.len(), 36); // 6 faces * 6 indices
        assert_eq!(mesh.total_quads(), 6);
    }

    #[test]
    fn test_face_culling_between_adjacent_blocks() {
        let atlas = TextureAtlas::build(Path::new("assets/minecraft/textures/blocks"));
        let mut section = ChunkSection::new(true);
        section.set_block(8, 8, 8, BlockState::new(1, 0)); // Stone
        section.set_block(9, 8, 8, BlockState::new(1, 0)); // Adjacent stone in +X

        let mut world = TestWorld {
            blocks: hashbrown::HashMap::new(),
        };
        world.blocks.insert((8, 8, 8), BlockState::new(1, 0));
        world.blocks.insert((9, 8, 8), BlockState::new(1, 0));

        let mesh = ChunkMesher::mesh_section(&section, 0, 0, 0, &world, &atlas);
        // Shared face (+X on (8,8,8) and -X on (9,8,8)) are both culled!
        // With Binary Greedy Meshing, the top (+Y), bottom (-Y), north (-Z), and south (+Z) faces
        // are merged into 2x1 quads (4 merged quads + 2 end 1x1 quads = 6 quads total!)
        assert_eq!(mesh.total_quads(), 6);
        assert_eq!(mesh.opaque_vertices.len(), 24);
        assert_eq!(mesh.opaque_indices.len(), 36);
    }

    #[test]
    fn test_binary_greedy_meshing_merges_large_plane() {
        let atlas = TextureAtlas::build(Path::new("assets/minecraft/textures/blocks"));
        let mut section = ChunkSection::new(true);
        let mut world = TestWorld {
            blocks: hashbrown::HashMap::new(),
        };

        // Create a 4x4 flat platform of stone
        for z in 4..8 {
            for x in 4..8 {
                section.set_block(x, 4, z, BlockState::new(1, 0));
                world.blocks.insert((x as i32, 4, z as i32), BlockState::new(1, 0));
            }
        }

        let mesh = ChunkMesher::mesh_section(&section, 0, 0, 0, &world, &atlas);
        // Without greedy meshing, 16 blocks would produce:
        // 16 top + 16 bottom + 4*4 sides = 48 quads (192 vertices).
        // With greedy meshing:
        // Top: exactly 1 merged quad (4x4)!
        // Bottom: exactly 1 merged quad (4x4)!
        // North, South, East, West sides: 1 merged quad each (4x1)!
        // Total: exactly 6 quads (24 vertices)! An 87.5% reduction!
        assert_eq!(mesh.total_quads(), 6);
        assert_eq!(mesh.opaque_vertices.len(), 24);
    }

    #[test]
    fn test_mesher_speed_benchmark() {
        let atlas = TextureAtlas::build(Path::new("assets/minecraft/textures/blocks"));
        let mut section = ChunkSection::new(true);

        // Fill a realistic terrain landscape: bottom 8 layers solid, top layer grass
        let mut world = TestWorld {
            blocks: hashbrown::HashMap::new(),
        };

        for y in 0..8 {
            for z in 0..16 {
                for x in 0..16 {
                    let block = if y == 7 {
                        BlockState::new(2, 0) // Grass
                    } else if y > 4 {
                        BlockState::new(3, 0) // Dirt
                    } else {
                        BlockState::new(1, 0) // Stone
                    };
                    section.set_block(x, y, z, block);
                    world.blocks.insert((x as i32, y as i32, z as i32), block);
                }
            }
        }

        // Measure time to mesh 10 iterations
        let start = Instant::now();
        let iters = 10;
        let mut last_mesh = SectionMeshData::default();
        for _ in 0..iters {
            last_mesh = ChunkMesher::mesh_section(&section, 0, 0, 0, &world, &atlas);
        }
        let total_us = start.elapsed().as_micros();
        let us_per_section = total_us / iters;

        println!(
            "Mesh benchmark: {} us/section ({} quads generated)",
            us_per_section,
            last_mesh.total_quads()
        );

        // Verification: greedy meshing a full realistic section takes < 20000 microseconds in debug mode
        assert!(
            us_per_section < 20000,
            "Mesher took too long: {} us",
            us_per_section
        );
        assert!(last_mesh.total_quads() > 0);
    }

    #[test]
    fn test_cross_model_tallgrass_meshing() {
        let atlas = TextureAtlas::build(Path::new("assets/minecraft/textures/blocks"));
        let mut section = ChunkSection::new(true);
        section.set_block(4, 4, 4, BlockState::new(31, 1)); // Tall grass

        let world = TestWorld {
            blocks: hashbrown::HashMap::new(),
        };

        let mesh = ChunkMesher::mesh_section(&section, 0, 0, 0, &world, &atlas);
        // Cross model (tall grass) has 4 quads, 16 vertices, 24 indices
        assert_eq!(mesh.total_quads(), 4);
        assert_eq!(mesh.opaque_vertices.len(), 16);
        assert_eq!(mesh.opaque_indices.len(), 24);
    }

    #[test]
    fn test_smart_leaves_face_culling() {
        let atlas = TextureAtlas::build(Path::new("assets/minecraft/textures/blocks"));
        let mut section = ChunkSection::new(true);
        section.set_block(5, 5, 5, BlockState::new(18, 0)); // Oak leaves
        section.set_block(6, 5, 5, BlockState::new(18, 0)); // Adjacent oak leaves

        let mut world = TestWorld {
            blocks: hashbrown::HashMap::new(),
        };
        world.blocks.insert((5, 5, 5), BlockState::new(18, 0));
        world.blocks.insert((6, 5, 5), BlockState::new(18, 0));

        let mesh = ChunkMesher::mesh_section(&section, 0, 0, 0, &world, &atlas);
        // Internal shared face between adjacent leaves must be culled!
        // 6 + 6 - 2 = 10 faces
        assert_eq!(mesh.total_quads(), 10);
    }

    #[test]
    fn test_custom_geometry_blocks_meshing() {
        let atlas = TextureAtlas::build(Path::new("assets/minecraft/textures/blocks"));

        // 1. Slab
        let mut section_slab = ChunkSection::new(true);
        section_slab.set_block(2, 2, 2, BlockState::new(44, 0)); // Bottom stone slab
        let world = TestWorld {
            blocks: hashbrown::HashMap::new(),
        };
        let mesh_slab = ChunkMesher::mesh_section(&section_slab, 0, 0, 0, &world, &atlas);
        assert_eq!(mesh_slab.total_quads(), 6);
        // Verify slab height in vertex positions: max Y must be 2.5
        let max_y = mesh_slab
            .opaque_vertices
            .iter()
            .map(|v| v.unpack().local_pos[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((max_y - 2.5).abs() < 1e-4);

        // 2. Carpet
        let mut section_carpet = ChunkSection::new(true);
        section_carpet.set_block(3, 3, 3, BlockState::new(171, 14)); // Red carpet
        let mesh_carpet = ChunkMesher::mesh_section(&section_carpet, 0, 0, 0, &world, &atlas);
        assert_eq!(mesh_carpet.total_quads(), 6);
        let max_y_carpet = mesh_carpet
            .opaque_vertices
            .iter()
            .map(|v| v.unpack().local_pos[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((max_y_carpet - 3.0625).abs() < 1e-4);

        // 3. Stairs
        let mut section_stairs = ChunkSection::new(true);
        section_stairs.set_block(4, 4, 4, BlockState::new(53, 0)); // Oak stairs facing East
        let mesh_stairs = ChunkMesher::mesh_section(&section_stairs, 0, 0, 0, &world, &atlas);
        // Base slab (6) + step (6) = 12 faces
        assert_eq!(mesh_stairs.total_quads(), 12);
    }

    #[test]
    fn test_glass_pane_and_iron_bars_connection() {
        let atlas = TextureAtlas::build(Path::new("assets/minecraft/textures/blocks"));

        // 1. Two adjacent glass panes along X: (5, 5, 5) and (6, 5, 5)
        let mut section = ChunkSection::new(true);
        section.set_block(5, 5, 5, BlockState::new(102, 0)); // Glass pane
        section.set_block(6, 5, 5, BlockState::new(102, 0)); // Glass pane
        let mut world = TestWorld {
            blocks: hashbrown::HashMap::new(),
        };
        world.blocks.insert((5, 5, 5), BlockState::new(102, 0));
        world.blocks.insert((6, 5, 5), BlockState::new(102, 0));

        let mesh = ChunkMesher::mesh_section(&section, 0, 0, 0, &world, &atlas);
        assert!(mesh.translucent_vertices.len() > 0);

        // In block 5, vertex extends all the way to 6.0 (East)
        let overall_max_x = mesh
            .translucent_vertices
            .iter()
            .map(|v| v.unpack().local_pos[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let overall_min_x = mesh
            .translucent_vertices
            .iter()
            .map(|v| v.unpack().local_pos[0])
            .fold(f32::INFINITY, f32::min);

        // The two panes span from 5.4375 to 6.5625 without disconnection
        assert!(overall_min_x <= 5.5);
        assert!(overall_max_x >= 6.5);

        // 2. Glass pane (8, 8, 8) connected to Iron bar (9, 8, 8)
        let mut section_mix = ChunkSection::new(true);
        section_mix.set_block(8, 8, 8, BlockState::new(102, 0)); // Glass pane
        section_mix.set_block(9, 8, 8, BlockState::new(101, 0)); // Iron bars
        let mut world_mix = TestWorld {
            blocks: hashbrown::HashMap::new(),
        };
        world_mix.blocks.insert((8, 8, 8), BlockState::new(102, 0));
        world_mix.blocks.insert((9, 8, 8), BlockState::new(101, 0));

        let mesh_mix = ChunkMesher::mesh_section(&section_mix, 0, 0, 0, &world_mix, &atlas);
        // Glass pane is in translucent vertices, reaching 9.0
        let max_x_pane = mesh_mix
            .translucent_vertices
            .iter()
            .map(|v| v.unpack().local_pos[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            (max_x_pane - 9.0).abs() < 1e-3,
            "Pane max X should meet iron bar at 9.0, got {}",
            max_x_pane
        );

        // Iron bar is in opaque vertices, reaching 9.0 on the west side
        let min_x_bars = mesh_mix
            .opaque_vertices
            .iter()
            .map(|v| v.unpack().local_pos[0])
            .fold(f32::INFINITY, f32::min);
        assert!(
            (min_x_bars - 9.0).abs() < 1e-3,
            "Iron bar min X should meet glass pane at 9.0, got {}",
            min_x_bars
        );

        // 3. Glass pane (1, 1, 1) connected to solid Glass block (2, 1, 1)
        let mut section_glass = ChunkSection::new(true);
        section_glass.set_block(1, 1, 1, BlockState::new(102, 0)); // Glass pane
        let mut world_glass = TestWorld {
            blocks: hashbrown::HashMap::new(),
        };
        world_glass
            .blocks
            .insert((1, 1, 1), BlockState::new(102, 0));
        world_glass.blocks.insert((2, 1, 1), BlockState::new(20, 0)); // Glass block (ID 20)

        let mesh_glass = ChunkMesher::mesh_section(&section_glass, 0, 0, 0, &world_glass, &atlas);
        let max_x_pane_to_glass = mesh_glass
            .translucent_vertices
            .iter()
            .map(|v| v.unpack().local_pos[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            (max_x_pane_to_glass - 2.0).abs() < 1e-3,
            "Pane max X should reach glass block at 2.0, got {}",
            max_x_pane_to_glass
        );
    }
}
