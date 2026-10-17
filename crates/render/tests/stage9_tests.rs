use glam::{Mat4, Vec3};
use render::atlas::TextureAtlas;
use render::frustum::Frustum;
use render::mesher::{BlockWorldAccessor, ChunkMesher};
use render::vertex::{BlockFace, BlockVertex, UnpackedBlockVertex};
use std::path::Path;
use std::time::Instant;
use world::block::BlockState;
use world::section::ChunkSection;

struct FlatPlaneMockWorld {
    plane_y: i32,
    block: BlockState,
}

impl BlockWorldAccessor for FlatPlaneMockWorld {
    fn block_at(&self, x: i32, y: i32, z: i32) -> BlockState {
        if y == self.plane_y && (0..16).contains(&x) && (0..16).contains(&z) {
            self.block
        } else {
            BlockState::AIR
        }
    }

    fn block_light_at(&self, _x: i32, _y: i32, _z: i32) -> u8 {
        0
    }

    fn sky_light_at(&self, _x: i32, _y: i32, _z: i32) -> u8 {
        15
    }
}

struct CheckerboardAoMockWorld {
    plane_y: i32,
    block: BlockState,
}

impl BlockWorldAccessor for CheckerboardAoMockWorld {
    fn block_at(&self, x: i32, y: i32, z: i32) -> BlockState {
        if y == self.plane_y {
            self.block
        } else if y == self.plane_y + 1 && (x + z) % 2 == 0 {
            // Obstacle block directly above creates checkerboard AO shadows
            self.block
        } else {
            BlockState::AIR
        }
    }

    fn block_light_at(&self, _x: i32, _y: i32, _z: i32) -> u8 {
        0
    }

    fn sky_light_at(&self, _x: i32, _y: i32, _z: i32) -> u8 {
        15
    }
}

#[test]
fn test_binary_greedy_meshing_16x16_plane_reduction() {
    let atlas = TextureAtlas::build(Path::new("assets/minecraft/textures/blocks"));

    // 16x16 flat stone floor at y = 0
    let mut section = ChunkSection::new(true);
    for z in 0..16 {
        for x in 0..16 {
            section.set_block(x, 0, z, BlockState::new(1, 0)); // Stone
        }
    }

    let world = FlatPlaneMockWorld {
        plane_y: 0,
        block: BlockState::new(1, 0), // Stone
    };

    let mesh_data = ChunkMesher::mesh_section(&section, 0, 0, 0, &world, &atlas);

    // Stone is opaque
    assert!(mesh_data.translucent_vertices.is_empty());
    assert!(mesh_data.translucent_indices.is_empty());

    // Without greedy meshing, top face (y=0) would produce 16*16 = 256 quads.
    // Bottom face (y=0) would produce 16*16 = 256 quads.
    // 4 borders (north, south, east, west) would produce 4 * 16 = 64 quads.
    // Total quads without greedy meshing = 256 + 256 + 64 = 576 quads = 2304 vertices, 3456 indices.
    //
    // With Binary Greedy Meshing:
    // Top face is completely merged into 1 single quad (4 vertices, 6 indices)!
    // Bottom face is completely merged into 1 single quad (4 vertices, 6 indices)!
    // North, South, East, West borders (each 16x1) are merged into 1 quad each (4 quads)!
    // Total quads with greedy meshing = 1 + 1 + 4 = 6 quads = 24 vertices, 36 indices!
    let total_quads = mesh_data.opaque_indices.len() / 6;
    assert_eq!(total_quads, 6, "Expected 6 merged quads (top, bottom, and 4 sides), got {total_quads}");
    assert_eq!(mesh_data.opaque_vertices.len(), 24, "Expected 24 vertices, got {}", mesh_data.opaque_vertices.len());
    assert_eq!(mesh_data.opaque_indices.len(), 36, "Expected 36 indices, got {}", mesh_data.opaque_indices.len());

    // Find the top-facing quad and verify its unpacked attributes and greedy UV extent
    let mut found_16x16_top_quad = false;
    for chunk in mesh_data.opaque_vertices.chunks_exact(4) {
        let v0 = chunk[0].unpack();
        let v1 = chunk[1].unpack();
        let v2 = chunk[2].unpack();
        let v3 = chunk[3].unpack();

        if v0.normal_index == BlockFace::Up.index() {
            assert!(v0.is_greedy && v1.is_greedy && v2.is_greedy && v3.is_greedy);
            // Verify greedily merged dimensions (span should be 16 in x and z)
            let max_u = v0.greedy_uv.0.max(v1.greedy_uv.0).max(v2.greedy_uv.0).max(v3.greedy_uv.0);
            let max_v = v0.greedy_uv.1.max(v1.greedy_uv.1).max(v2.greedy_uv.1).max(v3.greedy_uv.1);
            if max_u == 16 && max_v == 16 {
                found_16x16_top_quad = true;
            }
        }
    }
    assert!(found_16x16_top_quad, "Did not find merged 16x16 top quad");
}

#[test]
fn test_greedy_meshing_respects_ambient_occlusion_boundaries() {
    let atlas = TextureAtlas::build(Path::new("assets/minecraft/textures/blocks"));

    let mut section = ChunkSection::new(true);
    for z in 0..16 {
        for x in 0..16 {
            section.set_block(x, 0, z, BlockState::new(1, 0)); // Stone
        }
    }

    let world = CheckerboardAoMockWorld {
        plane_y: 0,
        block: BlockState::new(1, 0), // Stone
    };

    let mesh_data = ChunkMesher::mesh_section(&section, 0, 0, 0, &world, &atlas);

    // Because of checkerboard obstacles above, neighboring blocks have differing AO values.
    // Greedy meshing must NOT merge across distinct AO values to avoid lighting artifacts.
    let total_quads = mesh_data.opaque_indices.len() / 6;
    assert!(
        total_quads > 6,
        "Greedy meshing should not merge non-uniform AO blocks into 6 quads, got {total_quads}"
    );
}

#[test]
fn test_block_vertex_bit_packing_roundtrip_and_memory_footprint() {
    // Assert 16 bytes memory footprint per vertex (66.7% reduction from 48 bytes)
    assert_eq!(std::mem::size_of::<BlockVertex>(), 16);
    assert_eq!(std::mem::align_of::<BlockVertex>(), 4);

    let test_cases: Vec<UnpackedBlockVertex> = vec![
        UnpackedBlockVertex {
            local_pos: [0.0, 0.0, 0.0],
            uv: [0.0, 0.0],
            light: [0.0, 0.0],
            ao: 0,
            color: [1.0, 1.0, 1.0],
            normal_index: 0,
            greedy_uv: (0, 0),
            is_greedy: false,
        },
        UnpackedBlockVertex {
            local_pos: [16.0, 16.0, 16.0],
            uv: [1.0, 1.0],
            light: [1.0, 1.0],
            ao: 3,
            color: [0.5, 0.25, 0.75],
            normal_index: 5,
            greedy_uv: (16, 16),
            is_greedy: true,
        },
        UnpackedBlockVertex {
            local_pos: [8.5, 4.25, 12.125],
            uv: [0.25, 0.75],
            light: [0.4, 0.8], // 6/15 and 12/15
            ao: 2,
            color: [0.2, 0.8, 0.4],
            normal_index: 1,
            greedy_uv: (4, 12),
            is_greedy: true,
        },
    ];

    for expected in test_cases {
        let packed = BlockVertex::pack(
            expected.local_pos,
            expected.uv,
            expected.light,
            expected.ao,
            expected.color,
            expected.normal_index,
            expected.greedy_uv,
            expected.is_greedy,
        );

        let unpacked = packed.unpack();

        // Local pos quantized to 1/32 step
        for i in 0..3 {
            assert!(
                (unpacked.local_pos[i] - expected.local_pos[i]).abs() < 0.032,
                "Pos[{i}] mismatch: got {}, expected {}",
                unpacked.local_pos[i],
                expected.local_pos[i]
            );
        }

        // UV normalized to 16-bit
        assert!((unpacked.uv[0] - expected.uv[0]).abs() < 0.001);
        assert!((unpacked.uv[1] - expected.uv[1]).abs() < 0.001);

        // AO and normal
        assert_eq!(unpacked.ao, expected.ao);
        assert_eq!(unpacked.normal_index, expected.normal_index);

        // Greedy flag & UV
        assert_eq!(unpacked.is_greedy, expected.is_greedy);
        assert_eq!(unpacked.greedy_uv, expected.greedy_uv);
    }
}

#[test]
fn test_simd_frustum_culling_benchmark_and_accuracy() {
    let view = Mat4::look_at_rh(Vec3::new(0.0, 64.0, 0.0), Vec3::new(0.0, 64.0, 100.0), Vec3::Y);
    let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 16.0 / 9.0, 0.05, 512.0);
    let frustum = Frustum::from_view_proj(proj * view);

    // Generate 1000 chunk section bounding boxes (-500 to +500 in world coordinates)
    let mut aabbs = Vec::with_capacity(1000);
    for x in -5..5 {
        for y in 0..10 {
            for z in -5..5 {
                let min = Vec3::new((x * 16) as f32, (y * 16) as f32, (z * 16) as f32);
                let max = min + Vec3::splat(16.0);
                aabbs.push((min, max));
            }
        }
    }

    // Benchmark SIMD culling execution speed
    let iterations = 100;
    let start = Instant::now();
    let mut visible_count = 0;
    for _ in 0..iterations {
        for (min, max) in &aabbs {
            if frustum.intersects_aabb(*min, *max) {
                visible_count += 1;
            }
        }
    }
    let elapsed = start.elapsed();
    let total_tests = iterations * aabbs.len();
    let ns_per_test = elapsed.as_nanos() as f64 / total_tests as f64;

    println!(
        "Frustum SIMD test: {total_tests} AABB checks in {:.2?} ({:.1} ns/test), visible: {visible_count}",
        elapsed, ns_per_test
    );

    // Must be extremely fast with SIMD glam::Vec4 (< 2500 ns per AABB on modern CPU in unoptimized debug build, < 20 ns in release)
    assert!(
        ns_per_test < 2500.0,
        "Frustum culling is too slow: {:.1} ns/test",
        ns_per_test
    );

    // Verify consistency: chunks directly in front of camera should be visible, behind invisible
    let in_front = frustum.intersects_aabb(Vec3::new(-8.0, 56.0, 20.0), Vec3::new(8.0, 72.0, 36.0));
    assert!(in_front, "Chunk in front of camera should be visible");

    let behind = frustum.intersects_aabb(Vec3::new(-8.0, 56.0, -100.0), Vec3::new(8.0, 72.0, -84.0));
    assert!(!behind, "Chunk behind camera should be culled");
}
