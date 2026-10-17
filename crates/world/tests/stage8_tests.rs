use world::block::BlockState;
use world::morton::{decode_morton, encode_morton, linear_to_morton, morton_to_linear};
use world::palette::{PaletteKind, PalettedContainer, SECTION_VOXELS};
use world::section::ChunkSection;

#[test]
fn test_morton_3d_z_order_bijection_and_locality() {
    let mut visited_codes = [false; SECTION_VOXELS];

    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                let code = encode_morton(x, y, z);
                assert!(code < SECTION_VOXELS, "Morton code out of bounds: {code}");
                assert!(!visited_codes[code], "Collision on Morton code {code}");
                visited_codes[code] = true;

                let (rx, ry, rz) = decode_morton(code);
                assert_eq!((x, y, z), (rx, ry, rz), "Decode mismatch for ({x}, {y}, {z})");
            }
        }
    }

    // Verify 2x2x2 local block locality (Morton codes 0..8 map to x in 0..2, y in 0..2, z in 0..2)
    for code in 0..8 {
        let (x, y, z) = decode_morton(code);
        assert!(x < 2 && y < 2 && z < 2, "2x2x2 block locality violated at code {code}: ({x}, {y}, {z})");
    }

    // Verify linear index bijection roundtrip
    for linear in 0..SECTION_VOXELS {
        let m = linear_to_morton(linear);
        let back = morton_to_linear(m);
        assert_eq!(back, linear, "Linear-Morton roundtrip failed at {linear}");
    }
}

#[test]
fn test_palette_dynamic_expansion_and_compaction() {
    let mut container = PalettedContainer::new_empty();

    // 1. Initial SingleValue (0 bits, 0 heap bytes)
    assert_eq!(container.kind(), PaletteKind::SingleValue);
    assert_eq!(container.bits_per_block(), 0);
    assert_eq!(container.heap_memory_usage(), 0);
    assert_eq!(container.get(0, 0, 0), BlockState::AIR);

    // 2. Add first block -> expands to B4 (4 bits, 2 palette entries)
    container.set(0, 0, 0, BlockState::new(1, 0)); // Stone
    assert_eq!(container.kind(), PaletteKind::B4);
    assert_eq!(container.bits_per_block(), 4);
    assert!(container.heap_memory_usage() > 2000);
    assert_eq!(container.get(0, 0, 0), BlockState::new(1, 0));
    assert_eq!(container.get(1, 0, 0), BlockState::AIR);

    // 3. Add up to 16 unique block states (AIR + 15 states) -> remains B4
    for i in 2..=15 {
        container.set(i - 1, 0, 0, BlockState::new(i as u16, 0));
        assert_eq!(container.kind(), PaletteKind::B4);
        assert_eq!(container.bits_per_block(), 4);
    }

    // Verify all 16 states are in palette
    assert_eq!(container.get(0, 0, 0), BlockState::new(1, 0));
    for i in 2..=15 {
        assert_eq!(container.get(i - 1, 0, 0), BlockState::new(i as u16, 0));
    }

    // 4. Add 17th block -> promotes to B8 (8 bits, 17 palette entries)
    container.set(0, 1, 0, BlockState::new(16, 0));
    assert_eq!(container.kind(), PaletteKind::B8);
    assert_eq!(container.bits_per_block(), 8);
    assert!(container.heap_memory_usage() > 4000);

    // Verify prior blocks still intact after B8 promotion
    assert_eq!(container.get(0, 0, 0), BlockState::new(1, 0));
    assert_eq!(container.get(0, 1, 0), BlockState::new(16, 0));
    assert_eq!(container.get(14, 0, 0), BlockState::new(15, 0));

    // 5. Add up to 256 unique block states -> remains B8
    for i in 17..=255 {
        let x = (i - 1) % 16;
        let y = (i - 1) / 16;
        container.set(x, y, 0, BlockState::new(i as u16, 0));
        assert_eq!(container.kind(), PaletteKind::B8);
    }

    // 6. Add 257th block -> promotes to Direct (16 bits)
    container.set(0, 0, 1, BlockState::new(256, 0));
    assert_eq!(container.kind(), PaletteKind::Direct);
    assert_eq!(container.bits_per_block(), 16);
    assert_eq!(container.heap_memory_usage(), 8192);
    assert_eq!(container.get(0, 0, 1), BlockState::new(256, 0));

    // 7. Test Compaction: replace diverse blocks with just 2 block states
    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                if (x + y + z) % 2 == 0 {
                    container.set(x, y, z, BlockState::new(1, 0)); // Stone
                } else {
                    container.set(x, y, z, BlockState::AIR);
                }
            }
        }
    }

    // Demote from Direct back down to B4
    container.compact();
    assert_eq!(container.kind(), PaletteKind::B4);
    assert_eq!(container.bits_per_block(), 4);
    assert_eq!(container.get(0, 0, 0), BlockState::new(1, 0));
    assert_eq!(container.get(1, 0, 0), BlockState::AIR);

    // 8. Test Compaction to SingleValue: replace all with AIR
    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                container.set(x, y, z, BlockState::AIR);
            }
        }
    }
    container.compact();
    assert_eq!(container.kind(), PaletteKind::SingleValue);
    assert_eq!(container.bits_per_block(), 0);
    assert_eq!(container.heap_memory_usage(), 0);
}

#[test]
fn test_palette_memory_savings_and_benchmarks() {
    let uncompressed_bytes_per_section = SECTION_VOXELS * 2; // 8192 bytes

    // 1. SingleValue palette (empty chunk section or solid bedrock/stone)
    let single = PalettedContainer::new_empty();
    let single_heap = single.heap_memory_usage();
    assert_eq!(single_heap, 0);
    let single_savings_pct = 100.0 * (1.0 - (single_heap as f64 / uncompressed_bytes_per_section as f64));
    assert_eq!(single_savings_pct, 100.0);

    // 2. Typical Overworld section: 5 blocks (Stone, Dirt, Grass, Wood, Leaves)
    let mut overworld = PalettedContainer::new_empty();
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                let state = match y {
                    0..=4 => BlockState::new(1, 0),   // Stone
                    5..=10 => BlockState::new(3, 0),  // Dirt
                    11 => BlockState::new(2, 0),      // Grass
                    12..=14 => BlockState::new(17, 0),// Wood
                    _ => BlockState::new(18, 0),      // Leaves
                };
                overworld.set(x, y, z, state);
            }
        }
    }
    assert_eq!(overworld.kind(), PaletteKind::B4);
    let overworld_heap = overworld.heap_memory_usage();
    let overworld_savings_pct = 100.0 * (1.0 - (overworld_heap as f64 / uncompressed_bytes_per_section as f64));
    println!("B4 (5 unique blocks) heap bytes: {overworld_heap} / {uncompressed_bytes_per_section} ({overworld_savings_pct:.1}% savings)");
    assert!(overworld_savings_pct > 70.0, "Expected >70% RAM savings for typical B4 section");

    // 3. Dense section with 20 blocks (Village/Structure)
    let mut dense = PalettedContainer::new_empty();
    for i in 0..20 {
        dense.set(i % 16, i / 16, 0, BlockState::new((i + 1) as u16, 0));
    }
    assert_eq!(dense.kind(), PaletteKind::B8);
    let dense_heap = dense.heap_memory_usage();
    let dense_savings_pct = 100.0 * (1.0 - (dense_heap as f64 / uncompressed_bytes_per_section as f64));
    println!("B8 (20 unique blocks) heap bytes: {dense_heap} / {uncompressed_bytes_per_section} ({dense_savings_pct:.1}% savings)");
    assert!(dense_savings_pct > 40.0, "Expected >40% RAM savings for dense B8 section");
}

#[test]
fn test_chunk_section_raw_payload_roundtrip() {
    let mut section = ChunkSection::new(true);

    // Create realistic alternating terrain pattern
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                let id = ((x + y * 2 + z * 3) % 7) as u16;
                section.set_block(x, y, z, BlockState::new(id, 0));
            }
        }
    }

    let mut raw_bytes = Vec::new();
    section.write_blocks_to_raw(&mut raw_bytes);
    assert_eq!(raw_bytes.len(), SECTION_VOXELS * 2);

    let mut loaded_section = ChunkSection::new(true);
    loaded_section.load_blocks_from_raw_slice(&raw_bytes);

    // Verify all blocks match exactly
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                assert_eq!(
                    section.block_at(x, y, z),
                    loaded_section.block_at(x, y, z),
                    "Mismatch at ({x}, {y}, {z})"
                );
            }
        }
    }
    assert_eq!(section.non_air_count(), loaded_section.non_air_count());
}

#[test]
fn test_fill_opaque_bits_occlusion() {
    let mut section = ChunkSection::new(true);
    let mut opaque_bits = [0u64; 64];

    // Air section: 0 opaque blocks
    let count = section.fill_opaque_bits(&mut opaque_bits);
    assert_eq!(count, 0);
    assert!(opaque_bits.iter().all(|&w| w == 0));

    // Solid stone section: 4096 opaque blocks
    let stone = BlockState::new(1, 0);
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                section.set_block(x, y, z, stone);
            }
        }
    }
    let count = section.fill_opaque_bits(&mut opaque_bits);
    assert_eq!(count, 4096);
    assert!(opaque_bits.iter().all(|&w| w == !0u64));
}
