use bytes::BytesMut;
use protocol::{
    BlockChangeRecord, ClientboundBlockChangePacket, ClientboundChunkDataPacket,
    ClientboundMultiBlockChangePacket, Position,
};
use world::{BlockState, ChunkColumn, PerformanceProfiler, World};

#[test]
fn test_chunk_data_packet_and_world_loading() {
    let mut chunk = ChunkColumn::new(3, -5, true);
    // Put stone at (1, 64, 2)
    chunk.set_block(1, 64, 2, BlockState::new(1, 0));
    chunk.set_biome(1, 2, 1);

    let bit_mask = 1 << 4; // Section 4 (Y: 64..79)
    let raw_payload = chunk.serialize_to_raw_payload(bit_mask, true, true);

    let pkt = ClientboundChunkDataPacket {
        chunk_x: 3,
        chunk_z: -5,
        ground_up: true,
        bit_mask,
        data: raw_payload,
    };

    let mut buf = BytesMut::new();
    pkt.encode(&mut buf).unwrap();

    let mut bytes = buf.freeze();
    let decoded_pkt = ClientboundChunkDataPacket::decode(&mut bytes).unwrap();
    assert_eq!(decoded_pkt.chunk_x, 3);
    assert_eq!(decoded_pkt.chunk_z, -5);

    let mut world = World::new();
    world
        .load_chunk_data(
            decoded_pkt.chunk_x,
            decoded_pkt.chunk_z,
            decoded_pkt.ground_up,
            decoded_pkt.bit_mask,
            &decoded_pkt.data,
        )
        .expect("Chunk data should load successfully into World");

    let world_x = 3 * 16 + 1;
    let world_y = 64;
    let world_z = -5 * 16 + 2;
    assert_eq!(world.block_at(world_x, world_y, world_z), BlockState::new(1, 0));
}

#[test]
fn test_block_change_packets_in_world() {
    let mut world = World::new();

    // 1. Single BlockChange packet
    let pos = Position::new(10, 70, 20);
    let pkt = ClientboundBlockChangePacket {
        position: pos,
        block_state: (3 << 4), // Dirt
    };

    let mut buf = BytesMut::new();
    pkt.encode(&mut buf).unwrap();
    let mut bytes = buf.freeze();
    let decoded = ClientboundBlockChangePacket::decode(&mut bytes).unwrap();

    world.set_block(
        decoded.position.x,
        decoded.position.y,
        decoded.position.z,
        BlockState(decoded.block_state as u16),
    );
    assert_eq!(world.block_at(10, 70, 20), BlockState::new(3, 0));

    // 2. MultiBlockChange packet
    let multi_pkt = ClientboundMultiBlockChangePacket {
        chunk_x: 0,
        chunk_z: 0,
        records: vec![
            BlockChangeRecord {
                horizontal_pos: (2 << 4) | 5, // rel_x: 2, rel_z: 5
                y: 65,
                block_state: (4 << 4), // Cobblestone
            },
            BlockChangeRecord {
                horizontal_pos: (7 << 4) | 9, // rel_x: 7, rel_z: 9
                y: 80,
                block_state: (5 << 4), // Oak Wood Planks
            },
        ],
    };

    let mut multi_buf = BytesMut::new();
    multi_pkt.encode(&mut multi_buf).unwrap();
    let mut multi_bytes = multi_buf.freeze();
    let decoded_multi = ClientboundMultiBlockChangePacket::decode(&mut multi_bytes).unwrap();

    for rec in decoded_multi.records {
        let bx = (decoded_multi.chunk_x << 4) + rec.rel_x() as i32;
        let bz = (decoded_multi.chunk_z << 4) + rec.rel_z() as i32;
        world.set_block(bx, rec.y as i32, bz, BlockState(rec.block_state as u16));
    }

    assert_eq!(world.block_at(2, 65, 5), BlockState::new(4, 0));
    assert_eq!(world.block_at(7, 80, 9), BlockState::new(5, 0));
}

#[test]
fn test_profiler_telemetry_file_dump() {
    let mut profiler = PerformanceProfiler::new();
    for _ in 0..100 {
        profiler.on_frame();
    }
    profiler.loaded_chunks = 42;
    profiler.on_bytes_received(1024);

    let temp_dir = std::env::temp_dir();
    let dump_path = temp_dir.join("test_profile_benchmark.json");
    profiler.dump_telemetry(&dump_path).unwrap();

    assert!(dump_path.exists());
    let content = std::fs::read_to_string(&dump_path).unwrap();
    assert!(content.contains("\"loaded_chunks\": 42"));
    let _ = std::fs::remove_file(dump_path);
}
