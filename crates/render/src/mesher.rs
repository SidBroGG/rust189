use crate::atlas::TextureAtlas;
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

/// High-performance Cull-Face chunk mesher with Smooth Lighting and Ambient Occlusion.
pub struct ChunkMesher;

impl ChunkMesher {
    /// Meshes a single 16x16x16 chunk section.
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

        for ly in 0..SECTION_HEIGHT {
            for lz in 0..SECTION_DEPTH {
                for lx in 0..SECTION_WIDTH {
                    let block = section.block_at(lx, ly, lz);
                    if block.is_air() {
                        continue;
                    }

                    let wx = base_x + lx as i32;
                    let wy = base_y + ly as i32;
                    let wz = base_z + lz as i32;

                    let is_translucent = TextureAtlas::is_translucent(block);
                    let _is_cutout = TextureAtlas::is_cutout(block);

                    // Cross-model blocks (tall grass, flowers, saplings, crops, reeds)
                    if TextureAtlas::is_cross(block) {
                        let (region, tint) = atlas.get_block_uv(block, BlockFace::North);
                        Self::generate_cross_quads(
                            wx as f32,
                            wy as f32,
                            wz as f32,
                            region.u_min,
                            region.v_min,
                            region.u_max,
                            region.v_max,
                            tint,
                            wx,
                            wy,
                            wz,
                            world,
                            is_translucent,
                            &mut mesh,
                        );
                        continue;
                    }

                    let block_id = block.id();
                    let block_meta = block.meta();

                    // Non-standard geometry blocks (slabs, stairs, trapdoors, carpets, snow layers, panes, fences)
                    match block_id {
                        // Slabs (half slabs; double slabs 43 & 125 are full cubes)
                        44 | 126 => {
                            let is_top = (block_meta & 8) != 0;
                            let (y0, y1) = if is_top { (0.5, 1.0) } else { (0.0, 0.5) };
                            Self::generate_custom_box(wx, wy, wz, [0.0, y0, 0.0], [1.0, y1, 1.0], block, atlas, world, is_translucent, &mut mesh);
                            continue;
                        }
                        // Stairs
                        53 | 67 | 108 | 109 | 114 | 128 | 134 | 135 | 136 | 156 | 163 | 164 => {
                            let is_top = (block_meta & 4) != 0;
                            let dir = block_meta & 3;
                            let (base_y0, base_y1) = if is_top { (0.5, 1.0) } else { (0.0, 0.5) };
                            Self::generate_custom_box(wx, wy, wz, [0.0, base_y0, 0.0], [1.0, base_y1, 1.0], block, atlas, world, is_translucent, &mut mesh);

                            let (step_y0, step_y1) = if is_top { (0.0, 0.5) } else { (0.5, 1.0) };
                            let (step_min, step_max) = match dir {
                                0 => ([0.5, step_y0, 0.0], [1.0, step_y1, 1.0]), // East
                                1 => ([0.0, step_y0, 0.0], [0.5, step_y1, 1.0]), // West
                                2 => ([0.0, step_y0, 0.5], [1.0, step_y1, 1.0]), // South
                                _ => ([0.0, step_y0, 0.0], [1.0, step_y1, 0.5]), // North
                            };
                            Self::generate_custom_box(wx, wy, wz, step_min, step_max, block, atlas, world, is_translucent, &mut mesh);
                            continue;
                        }
                        // Trapdoors
                        96 | 167 => {
                            let is_open = (block_meta & 4) != 0;
                            let is_top = (block_meta & 8) != 0;
                            let dir = block_meta & 3;
                            if !is_open {
                                let (y0, y1) = if is_top { (0.8125, 1.0) } else { (0.0, 0.1875) };
                                Self::generate_custom_box(wx, wy, wz, [0.0, y0, 0.0], [1.0, y1, 1.0], block, atlas, world, is_translucent, &mut mesh);
                            } else {
                                let (box_min, box_max) = match dir {
                                    0 => ([0.0, 0.0, 0.8125], [1.0, 1.0, 1.0]), // North (against South wall)
                                    1 => ([0.0, 0.0, 0.0], [1.0, 1.0, 0.1875]), // South (against North wall)
                                    2 => ([0.8125, 0.0, 0.0], [1.0, 1.0, 1.0]), // West (against East wall)
                                    _ => ([0.0, 0.0, 0.0], [0.1875, 1.0, 1.0]), // East (against West wall)
                                };
                                Self::generate_custom_box(wx, wy, wz, box_min, box_max, block, atlas, world, is_translucent, &mut mesh);
                            }
                            continue;
                        }
                        // Carpet
                        171 => {
                            Self::generate_custom_box(wx, wy, wz, [0.0, 0.0, 0.0], [1.0, 0.0625, 1.0], block, atlas, world, is_translucent, &mut mesh);
                            continue;
                        }
                        // Snow Layer
                        78 => {
                            let h = (((block_meta & 7) + 1) as f32 * 0.125).min(1.0);
                            Self::generate_custom_box(wx, wy, wz, [0.0, 0.0, 0.0], [1.0, h, 1.0], block, atlas, world, is_translucent, &mut mesh);
                            continue;
                        }
                        // Panes & Iron Bars
                        101 | 102 | 160 => {
                            Self::generate_custom_box(wx, wy, wz, [0.4375, 0.0, 0.4375], [0.5625, 1.0, 0.5625], block, atlas, world, is_translucent, &mut mesh);
                            let can_connect = |nx: i32, ny: i32, nz: i32| -> bool {
                                let b = world.block_at(nx, ny, nz);
                                b.is_opaque() || matches!(b.id(), 101 | 102 | 160)
                            };
                            let conn_n = can_connect(wx, wy, wz - 1);
                            let conn_s = can_connect(wx, wy, wz + 1);
                            let conn_w = can_connect(wx - 1, wy, wz);
                            let conn_e = can_connect(wx + 1, wy, wz);

                            if conn_n {
                                Self::generate_custom_box(wx, wy, wz, [0.4375, 0.0, 0.0], [0.5625, 1.0, 0.4375], block, atlas, world, is_translucent, &mut mesh);
                            }
                            if conn_s {
                                Self::generate_custom_box(wx, wy, wz, [0.4375, 0.0, 0.5625], [0.5625, 1.0, 1.0], block, atlas, world, is_translucent, &mut mesh);
                            }
                            if conn_w {
                                Self::generate_custom_box(wx, wy, wz, [0.0, 0.0, 0.4375], [0.4375, 1.0, 0.5625], block, atlas, world, is_translucent, &mut mesh);
                            }
                            if conn_e {
                                Self::generate_custom_box(wx, wy, wz, [0.5625, 0.0, 0.4375], [1.0, 1.0, 0.5625], block, atlas, world, is_translucent, &mut mesh);
                            }
                            if !conn_n && !conn_s && !conn_w && !conn_e {
                                Self::generate_custom_box(wx, wy, wz, [0.4375, 0.0, 0.0], [0.5625, 1.0, 1.0], block, atlas, world, is_translucent, &mut mesh);
                                Self::generate_custom_box(wx, wy, wz, [0.0, 0.0, 0.4375], [1.0, 1.0, 0.5625], block, atlas, world, is_translucent, &mut mesh);
                            }
                            continue;
                        }
                        // Fences
                        85 | 113 | 188..=192 => {
                            Self::generate_custom_box(wx, wy, wz, [0.375, 0.0, 0.375], [0.625, 1.0, 0.625], block, atlas, world, is_translucent, &mut mesh);
                            let can_connect_f = |nx: i32, ny: i32, nz: i32| -> bool {
                                let b = world.block_at(nx, ny, nz);
                                b.is_opaque() || matches!(b.id(), 85 | 107 | 113 | 183..=192)
                            };
                            if can_connect_f(wx, wy, wz - 1) {
                                Self::generate_custom_box(wx, wy, wz, [0.4375, 0.375, 0.0], [0.5625, 0.9375, 0.375], block, atlas, world, is_translucent, &mut mesh);
                            }
                            if can_connect_f(wx, wy, wz + 1) {
                                Self::generate_custom_box(wx, wy, wz, [0.4375, 0.375, 0.625], [0.5625, 0.9375, 1.0], block, atlas, world, is_translucent, &mut mesh);
                            }
                            if can_connect_f(wx - 1, wy, wz) {
                                Self::generate_custom_box(wx, wy, wz, [0.0, 0.375, 0.4375], [0.375, 0.9375, 0.5625], block, atlas, world, is_translucent, &mut mesh);
                            }
                            if can_connect_f(wx + 1, wy, wz) {
                                Self::generate_custom_box(wx, wy, wz, [0.625, 0.375, 0.4375], [1.0, 0.9375, 0.5625], block, atlas, world, is_translucent, &mut mesh);
                            }
                            continue;
                        }
                        _ => {}
                    }

                    let is_leaves = TextureAtlas::is_leaves(block);

                    // Check all 6 faces
                    for face in BlockFace::ALL {
                        let (dx, dy, dz) = face.offset();
                        let nx = wx + dx;
                        let ny = wy + dy;
                        let nz = wz + dz;

                        let neighbor = world.block_at(nx, ny, nz);

                        // Face culling conditions:
                        // 1. Neighbor is opaque -> face is completely hidden!
                        if neighbor.is_opaque() {
                            continue;
                        }

                        // 2. Neighbor is same translucent liquid/glass -> cull internal face!
                        if is_translucent && neighbor.id() == block.id() {
                            continue;
                        }

                        // 3. Smart leaf culling: cull internal faces between adjacent leaves!
                        if is_leaves && TextureAtlas::is_leaves(neighbor) {
                            continue;
                        }

                        // Face is visible! Generate quad
                        let (region, tint) = atlas.get_block_uv(block, face);

                        Self::generate_face(
                            wx as f32,
                            wy as f32,
                            wz as f32,
                            face,
                            region.u_min,
                            region.v_min,
                            region.u_max,
                            region.v_max,
                            tint,
                            nx,
                            ny,
                            nz,
                            world,
                            is_translucent,
                            &mut mesh,
                        );
                    }
                }
            }
        }

        mesh.visibility = VisGraph::compute_visibility(section);
        mesh
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn generate_face<W: BlockWorldAccessor>(
        x: f32,
        y: f32,
        z: f32,
        face: BlockFace,
        u_min: f32,
        v_min: f32,
        u_max: f32,
        v_max: f32,
        tint: [f32; 3],
        nx: i32,
        ny: i32,
        nz: i32,
        world: &W,
        is_translucent: bool,
        mesh: &mut SectionMeshData,
    ) {
        Self::generate_sub_face(
            x,
            y,
            z,
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            face,
            u_min,
            v_min,
            u_max,
            v_max,
            tint,
            nx,
            ny,
            nz,
            world,
            is_translucent,
            mesh,
        );
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn generate_custom_box<W: BlockWorldAccessor>(
        wx: i32,
        wy: i32,
        wz: i32,
        min: [f32; 3],
        max: [f32; 3],
        block: BlockState,
        atlas: &TextureAtlas,
        world: &W,
        is_translucent: bool,
        mesh: &mut SectionMeshData,
    ) {
        let x = wx as f32;
        let y = wy as f32;
        let z = wz as f32;

        for face in BlockFace::ALL {
            let (dx, dy, dz) = face.offset();
            let nx = wx + dx;
            let ny = wy + dy;
            let nz = wz + dz;

            let touches_boundary = match face {
                BlockFace::Up => max[1] >= 1.0,
                BlockFace::Down => min[1] <= 0.0,
                BlockFace::North => min[2] <= 0.0,
                BlockFace::South => max[2] >= 1.0,
                BlockFace::West => min[0] <= 0.0,
                BlockFace::East => max[0] >= 1.0,
            };

            if touches_boundary && world.block_at(nx, ny, nz).is_opaque() {
                continue;
            }

            let (sample_x, sample_y, sample_z) = if touches_boundary {
                (nx, ny, nz)
            } else {
                (wx, wy, wz)
            };

            let (region, tint) = atlas.get_block_uv(block, face);
            Self::generate_sub_face(
                x,
                y,
                z,
                min,
                max,
                face,
                region.u_min,
                region.v_min,
                region.u_max,
                region.v_max,
                tint,
                sample_x,
                sample_y,
                sample_z,
                world,
                is_translucent,
                mesh,
            );
        }
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn generate_sub_face<W: BlockWorldAccessor>(
        x: f32,
        y: f32,
        z: f32,
        min: [f32; 3],
        max: [f32; 3],
        face: BlockFace,
        u_min: f32,
        v_min: f32,
        u_max: f32,
        v_max: f32,
        tint: [f32; 3],
        nx: i32,
        ny: i32,
        nz: i32,
        world: &W,
        is_translucent: bool,
        mesh: &mut SectionMeshData,
    ) {
        let x0 = min[0];
        let y0 = min[1];
        let z0 = min[2];
        let x1 = max[0];
        let y1 = max[1];
        let z1 = max[2];
        let du = u_max - u_min;
        let dv = v_max - v_min;

        // Quad 4 local vertex positions & UVs
        let (positions, uvs, tangents) = match face {
            BlockFace::Up => (
                [
                    [x + x0, y + y1, z + z1],
                    [x + x1, y + y1, z + z1],
                    [x + x1, y + y1, z + z0],
                    [x + x0, y + y1, z + z0],
                ],
                [
                    [u_min + x0 * du, v_min + z1 * dv],
                    [u_min + x1 * du, v_min + z1 * dv],
                    [u_min + x1 * du, v_min + z0 * dv],
                    [u_min + x0 * du, v_min + z0 * dv],
                ],
                (
                    (-1, 0, 0), // left
                    (1, 0, 0),  // right
                    (0, 0, -1), // back
                    (0, 0, 1),  // front
                ),
            ),
            BlockFace::Down => (
                [
                    [x + x0, y + y0, z + z0],
                    [x + x1, y + y0, z + z0],
                    [x + x1, y + y0, z + z1],
                    [x + x0, y + y0, z + z1],
                ],
                [
                    [u_min + x0 * du, v_min + z0 * dv],
                    [u_min + x1 * du, v_min + z0 * dv],
                    [u_min + x1 * du, v_min + z1 * dv],
                    [u_min + x0 * du, v_min + z1 * dv],
                ],
                (
                    (-1, 0, 0),
                    (1, 0, 0),
                    (0, 0, 1),
                    (0, 0, -1),
                ),
            ),
            BlockFace::North => (
                [
                    [x + x1, y + y0, z + z0],
                    [x + x0, y + y0, z + z0],
                    [x + x0, y + y1, z + z0],
                    [x + x1, y + y1, z + z0],
                ],
                [
                    [u_min + (1.0 - x1) * du, v_min + (1.0 - y0) * dv],
                    [u_min + (1.0 - x0) * du, v_min + (1.0 - y0) * dv],
                    [u_min + (1.0 - x0) * du, v_min + (1.0 - y1) * dv],
                    [u_min + (1.0 - x1) * du, v_min + (1.0 - y1) * dv],
                ],
                (
                    (1, 0, 0),
                    (-1, 0, 0),
                    (0, -1, 0),
                    (0, 1, 0),
                ),
            ),
            BlockFace::South => (
                [
                    [x + x0, y + y0, z + z1],
                    [x + x1, y + y0, z + z1],
                    [x + x1, y + y1, z + z1],
                    [x + x0, y + y1, z + z1],
                ],
                [
                    [u_min + x0 * du, v_min + (1.0 - y0) * dv],
                    [u_min + x1 * du, v_min + (1.0 - y0) * dv],
                    [u_min + x1 * du, v_min + (1.0 - y1) * dv],
                    [u_min + x0 * du, v_min + (1.0 - y1) * dv],
                ],
                (
                    (-1, 0, 0),
                    (1, 0, 0),
                    (0, -1, 0),
                    (0, 1, 0),
                ),
            ),
            BlockFace::West => (
                [
                    [x + x0, y + y0, z + z0],
                    [x + x0, y + y0, z + z1],
                    [x + x0, y + y1, z + z1],
                    [x + x0, y + y1, z + z0],
                ],
                [
                    [u_min + z0 * du, v_min + (1.0 - y0) * dv],
                    [u_min + z1 * du, v_min + (1.0 - y0) * dv],
                    [u_min + z1 * du, v_min + (1.0 - y1) * dv],
                    [u_min + z0 * du, v_min + (1.0 - y1) * dv],
                ],
                (
                    (0, 0, -1),
                    (0, 0, 1),
                    (0, -1, 0),
                    (0, 1, 0),
                ),
            ),
            BlockFace::East => (
                [
                    [x + x1, y + y0, z + z1],
                    [x + x1, y + y0, z + z0],
                    [x + x1, y + y1, z + z0],
                    [x + x1, y + y1, z + z1],
                ],
                [
                    [u_min + (1.0 - z1) * du, v_min + (1.0 - y0) * dv],
                    [u_min + (1.0 - z0) * du, v_min + (1.0 - y0) * dv],
                    [u_min + (1.0 - z0) * du, v_min + (1.0 - y1) * dv],
                    [u_min + (1.0 - z1) * du, v_min + (1.0 - y1) * dv],
                ],
                (
                    (0, 0, 1),
                    (0, 0, -1),
                    (0, -1, 0),
                    (0, 1, 0),
                ),
            ),
        };

        // Ambient Occlusion and Smooth Light calculation for 4 vertices
        let (s_left, s_right, s_down, s_up) = tangents;

        // v0 corner: (s_left, s_down)
        let (ao0, light0) = Self::compute_corner_ao_light(nx, ny, nz, s_left, s_down, world);
        // v1 corner: (s_right, s_down)
        let (ao1, light1) = Self::compute_corner_ao_light(nx, ny, nz, s_right, s_down, world);
        // v2 corner: (s_right, s_up)
        let (ao2, light2) = Self::compute_corner_ao_light(nx, ny, nz, s_right, s_up, world);
        // v3 corner: (s_left, s_up)
        let (ao3, light3) = Self::compute_corner_ao_light(nx, ny, nz, s_left, s_up, world);

        let v0 = BlockVertex {
            position: positions[0],
            uv: uvs[0],
            light: light0,
            ao: ao0,
            color: tint,
            normal_index: face.index(),
        };
        let v1 = BlockVertex {
            position: positions[1],
            uv: uvs[1],
            light: light1,
            ao: ao1,
            color: tint,
            normal_index: face.index(),
        };
        let v2 = BlockVertex {
            position: positions[2],
            uv: uvs[2],
            light: light2,
            ao: ao2,
            color: tint,
            normal_index: face.index(),
        };
        let v3 = BlockVertex {
            position: positions[3],
            uv: uvs[3],
            light: light3,
            ao: ao3,
            color: tint,
            normal_index: face.index(),
        };

        let (target_vertices, target_indices) = if is_translucent {
            (&mut mesh.translucent_vertices, &mut mesh.translucent_indices)
        } else {
            (&mut mesh.opaque_vertices, &mut mesh.opaque_indices)
        };

        let base_idx = target_vertices.len() as u32;
        target_vertices.extend_from_slice(&[v0, v1, v2, v3]);

        // Diagonal Flip for anisotropic AO artifact elimination
        if ao0 + ao2 > ao1 + ao3 {
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

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn generate_cross_quads<W: BlockWorldAccessor>(
        x: f32,
        y: f32,
        z: f32,
        u_min: f32,
        v_min: f32,
        u_max: f32,
        v_max: f32,
        tint: [f32; 3],
        wx: i32,
        wy: i32,
        wz: i32,
        world: &W,
        is_translucent: bool,
        mesh: &mut SectionMeshData,
    ) {
        let bl = world.block_light_at(wx, wy, wz);
        let sl = world.sky_light_at(wx, wy, wz);
        let light = [(bl as f32) / 15.0, (sl as f32) / 15.0];
        let ao = 1.0f32;
        let normal_index = 0u32;

        let (target_vertices, target_indices) = if is_translucent {
            (&mut mesh.translucent_vertices, &mut mesh.translucent_indices)
        } else {
            (&mut mesh.opaque_vertices, &mut mesh.opaque_indices)
        };

        // Diagonal 1: (x + 0.1, z + 0.1) to (x + 0.9, z + 0.9)
        let d1_p0 = [x + 0.1, y, z + 0.1];
        let d1_p1 = [x + 0.9, y, z + 0.9];
        let d1_p2 = [x + 0.9, y + 1.0, z + 0.9];
        let d1_p3 = [x + 0.1, y + 1.0, z + 0.1];

        // Diagonal 2: (x + 0.9, z + 0.1) to (x + 0.1, z + 0.9)
        let d2_p0 = [x + 0.9, y, z + 0.1];
        let d2_p1 = [x + 0.1, y, z + 0.9];
        let d2_p2 = [x + 0.1, y + 1.0, z + 0.9];
        let d2_p3 = [x + 0.9, y + 1.0, z + 0.1];

        let uvs = [
            [u_min, v_max],
            [u_max, v_max],
            [u_max, v_min],
            [u_min, v_min],
        ];

        // Diagonal 1 vertices (both front & back facing triangles)
        let base1 = target_vertices.len() as u32;
        for i in 0..4 {
            let pos = match i {
                0 => d1_p0,
                1 => d1_p1,
                2 => d1_p2,
                _ => d1_p3,
            };
            target_vertices.push(BlockVertex {
                position: pos,
                uv: uvs[i],
                light,
                ao,
                color: tint,
                normal_index,
            });
        }
        target_indices.extend_from_slice(&[
            base1, base1 + 1, base1 + 2, base1, base1 + 2, base1 + 3,
            base1, base1 + 2, base1 + 1, base1, base1 + 3, base1 + 2,
        ]);

        // Diagonal 2 vertices (both front & back facing triangles)
        let base2 = target_vertices.len() as u32;
        for i in 0..4 {
            let pos = match i {
                0 => d2_p0,
                1 => d2_p1,
                2 => d2_p2,
                _ => d2_p3,
            };
            target_vertices.push(BlockVertex {
                position: pos,
                uv: uvs[i],
                light,
                ao,
                color: tint,
                normal_index,
            });
        }
        target_indices.extend_from_slice(&[
            base2, base2 + 1, base2 + 2, base2, base2 + 2, base2 + 3,
            base2, base2 + 2, base2 + 1, base2, base2 + 3, base2 + 2,
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
    ) -> (f32, [f32; 2]) {
        let b1 = world.block_at(nx + side1.0, ny + side1.1, nz + side1.2).is_opaque();
        let b2 = world.block_at(nx + side2.0, ny + side2.1, nz + side2.2).is_opaque();
        let corner = world
            .block_at(
                nx + side1.0 + side2.0,
                ny + side1.1 + side2.1,
                nz + side1.2 + side2.2,
            )
            .is_opaque();

        let ao = if b1 && b2 {
            0
        } else {
            3 - (b1 as u8 + b2 as u8 + corner as u8)
        };

        let ao_mult = match ao {
            0 => 0.45,
            1 => 0.65,
            2 => 0.82,
            _ => 1.0,
        };

        // Smooth lighting interpolation
        let center_bl = world.block_light_at(nx, ny, nz) as f32;
        let center_sl = world.sky_light_at(nx, ny, nz) as f32;

        let s1_bl = world.block_light_at(nx + side1.0, ny + side1.1, nz + side1.2) as f32;
        let s1_sl = world.sky_light_at(nx + side1.0, ny + side1.1, nz + side1.2) as f32;

        let s2_bl = world.block_light_at(nx + side2.0, ny + side2.1, nz + side2.2) as f32;
        let s2_sl = world.sky_light_at(nx + side2.0, ny + side2.1, nz + side2.2) as f32;

        let c_bl = world.block_light_at(
            nx + side1.0 + side2.0,
            ny + side1.1 + side2.1,
            nz + side1.2 + side2.2,
        ) as f32;
        let c_sl = world.sky_light_at(
            nx + side1.0 + side2.0,
            ny + side1.1 + side2.1,
            nz + side1.2 + side2.2,
        ) as f32;

        let avg_bl = ((center_bl + s1_bl + s2_bl + c_bl) / 4.0).clamp(0.0, 15.0) / 15.0;
        let avg_sl = ((center_sl + s1_sl + s2_sl + c_sl) / 4.0).clamp(0.0, 15.0) / 15.0;

        (ao_mult, [avg_bl, avg_sl])
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
            self.blocks.get(&(x, y, z)).copied().unwrap_or(BlockState::AIR)
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
        assert_eq!(mesh.opaque_indices.len(), 36);  // 6 faces * 6 indices
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
        // 6 + 6 - 2 = 10 faces!
        assert_eq!(mesh.total_quads(), 10);
        assert_eq!(mesh.opaque_vertices.len(), 40);
        assert_eq!(mesh.opaque_indices.len(), 60);
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

        // Verification: meshing a full realistic section should easily take < 5000 microseconds (5 ms) in debug mode,
        // and < 200 microseconds in release mode!
        assert!(us_per_section < 20000, "Mesher took too long: {} us", us_per_section);
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
        // 2 diagonal planes, double-sided = 4 quads, 8 vertices, 24 indices
        assert_eq!(mesh.total_quads(), 4);
        assert_eq!(mesh.opaque_vertices.len(), 8);
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
        let max_y = mesh_slab.opaque_vertices.iter().map(|v| v.position[1]).fold(f32::NEG_INFINITY, f32::max);
        assert!((max_y - 2.5).abs() < 1e-4);

        // 2. Carpet
        let mut section_carpet = ChunkSection::new(true);
        section_carpet.set_block(3, 3, 3, BlockState::new(171, 14)); // Red carpet
        let mesh_carpet = ChunkMesher::mesh_section(&section_carpet, 0, 0, 0, &world, &atlas);
        assert_eq!(mesh_carpet.total_quads(), 6);
        let max_y_carpet = mesh_carpet.opaque_vertices.iter().map(|v| v.position[1]).fold(f32::NEG_INFINITY, f32::max);
        assert!((max_y_carpet - 3.0625).abs() < 1e-4);

        // 3. Stairs
        let mut section_stairs = ChunkSection::new(true);
        section_stairs.set_block(4, 4, 4, BlockState::new(53, 0)); // Oak stairs facing East
        let mesh_stairs = ChunkMesher::mesh_section(&section_stairs, 0, 0, 0, &world, &atlas);
        // Base slab (6) + step (6) = 12 faces
        assert_eq!(mesh_stairs.total_quads(), 12);
    }
}
