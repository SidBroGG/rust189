use crate::atlas::TextureAtlas;
use crate::block_model::BakedModelRegistry;
use crate::occlusion::{SectionVisibility, VisGraph};
use crate::vertex::BlockVertex;
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
                        // Face culling check
                        let cull_face =
                            quad.cull_face
                                .or_else(|| if is_leaf { Some(quad.face) } else { None });
                        if let Some(cull_face) = cull_face {
                            let (dx, dy, dz) = cull_face.offset();
                            let nx = wx + dx;
                            let ny = wy + dy;
                            let nz = wz + dz;
                            let neighbor = world.block_at(nx, ny, nz);

                            // 1. Neighbor is opaque -> face is culled
                            if neighbor.is_opaque() {
                                continue;
                            }

                            // 2. Neighbor is same translucent liquid/glass -> cull internal face
                            if is_translucent && neighbor.id() == block_id {
                                continue;
                            }

                            // 3. Smart leaf culling between adjacent leaves
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
                            // Flat / unshaded lighting (torches, flowers, crops, redstone)
                            let bl = (world.block_light_at(wx, wy, wz) as f32) / 15.0;
                            let sl = (world.sky_light_at(wx, wy, wz) as f32) / 15.0;
                            let light = [bl, sl];
                            let ao = 1.0f32;
                            let normal_index = quad.face.index();

                            for i in 0..4 {
                                target_vertices.push(BlockVertex {
                                    position: [
                                        wx as f32 + quad.positions[i][0],
                                        wy as f32 + quad.positions[i][1],
                                        wz as f32 + quad.positions[i][2],
                                    ],
                                    uv: quad.uvs[i],
                                    light,
                                    ao,
                                    color: quad_tint,
                                    normal_index,
                                });
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
                            // Smooth natural lighting & Ambient Occlusion
                            let (dx, dy, dz) = quad.face.offset();
                            let nx = wx + dx;
                            let ny = wy + dy;
                            let nz = wz + dz;
                            let (s_left, s_right, s_down, s_up) = quad.face.tangents();

                            let (ao0, light0) =
                                Self::compute_corner_ao_light(nx, ny, nz, s_left, s_down, world);
                            let (ao1, light1) =
                                Self::compute_corner_ao_light(nx, ny, nz, s_right, s_down, world);
                            let (ao2, light2) =
                                Self::compute_corner_ao_light(nx, ny, nz, s_right, s_up, world);
                            let (ao3, light3) =
                                Self::compute_corner_ao_light(nx, ny, nz, s_left, s_up, world);

                            let normal_index = quad.face.index();

                            let v0 = BlockVertex {
                                position: [
                                    wx as f32 + quad.positions[0][0],
                                    wy as f32 + quad.positions[0][1],
                                    wz as f32 + quad.positions[0][2],
                                ],
                                uv: quad.uvs[0],
                                light: light0,
                                ao: ao0,
                                color: quad_tint,
                                normal_index,
                            };
                            let v1 = BlockVertex {
                                position: [
                                    wx as f32 + quad.positions[1][0],
                                    wy as f32 + quad.positions[1][1],
                                    wz as f32 + quad.positions[1][2],
                                ],
                                uv: quad.uvs[1],
                                light: light1,
                                ao: ao1,
                                color: quad_tint,
                                normal_index,
                            };
                            let v2 = BlockVertex {
                                position: [
                                    wx as f32 + quad.positions[2][0],
                                    wy as f32 + quad.positions[2][1],
                                    wz as f32 + quad.positions[2][2],
                                ],
                                uv: quad.uvs[2],
                                light: light2,
                                ao: ao2,
                                color: quad_tint,
                                normal_index,
                            };
                            let v3 = BlockVertex {
                                position: [
                                    wx as f32 + quad.positions[3][0],
                                    wy as f32 + quad.positions[3][1],
                                    wz as f32 + quad.positions[3][2],
                                ],
                                uv: quad.uvs[3],
                                light: light3,
                                ao: ao3,
                                color: quad_tint,
                                normal_index,
                            };

                            target_vertices.extend_from_slice(&[v0, v1, v2, v3]);

                            // Diagonal flip for anisotropic AO artifact elimination
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
                    }
                }
            }
        }

        mesh.visibility = VisGraph::compute_visibility(section);
        mesh
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

        // Verification: meshing a full realistic section should easily take < 20000 microseconds in debug mode,
        // and < 200 microseconds in release mode!
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
            .map(|v| v.position[1])
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
            .map(|v| v.position[1])
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
            .map(|v| v.position[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let overall_min_x = mesh
            .translucent_vertices
            .iter()
            .map(|v| v.position[0])
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
            .map(|v| v.position[0])
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
            .map(|v| v.position[0])
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
            .map(|v| v.position[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            (max_x_pane_to_glass - 2.0).abs() < 1e-3,
            "Pane max X should reach glass block at 2.0, got {}",
            max_x_pane_to_glass
        );
    }
}
