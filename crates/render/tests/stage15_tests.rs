use bytes::{Bytes, BytesMut};
use glam::Vec3;
use protocol::codec::Position;
use protocol::packets::play::clientbound::{
    ClientboundBlockActionPacket, ClientboundUpdateSignPacket, ClientboundUpdateTileEntityPacket,
};
use protocol::NbtTag;
use std::f32::consts::PI;
use render::BlockEntityRenderer;
use world::block_entity::{
    BannerBlockEntity, BeaconBlockEntity, ChestBlockEntity, ChestType, DoubleChestType,
    EnchantmentTableBlockEntity, PistonBlockEntity,
};

#[test]
fn test_chest_lid_interpolation_and_easing() {
    let mut chest = ChestBlockEntity::new(ChestType::Normal, 2, DoubleChestType::Single);
    assert!(chest.is_static_closed());
    assert_eq!(chest.interpolated_lid_angle(0.0), 0.0);
    assert_eq!(chest.lid_rotation_radians(0.0), 0.0);

    // Player opens chest
    chest.num_players_using = 1;
    assert!(!chest.is_static_closed());

    // 1 tick of opening
    chest.tick();
    assert_eq!(chest.prev_lid_angle, 0.0);
    assert_eq!(chest.lid_angle, 0.1);

    // Test cubic easing formula: f = 1.0 - (1.0 - raw)^3
    // At partial_ticks = 0.5, raw = 0.05
    // 1.0 - 0.05 = 0.95; 0.95^3 = 0.857375; eased = 1.0 - 0.857375 = 0.142625
    let eased = chest.interpolated_lid_angle(0.5);
    assert!((eased - 0.142625).abs() < 1e-5);

    // Angle in radians should be -eased * (PI / 2.0)
    let rad = chest.lid_rotation_radians(0.5);
    assert!((rad - (-0.142625 * (PI / 2.0))).abs() < 1e-5);

    // Open fully (10 ticks)
    for _ in 0..10 {
        chest.tick();
    }
    assert_eq!(chest.lid_angle, 1.0);
    assert!((chest.interpolated_lid_angle(1.0) - 1.0).abs() < 1e-5);
    assert!((chest.lid_rotation_radians(1.0) - (-PI / 2.0)).abs() < 1e-5);

    // Close chest (takes 10 ticks for lid_angle -> 0.0, and 1 more tick for prev_lid_angle -> 0.0)
    chest.num_players_using = 0;
    for _ in 0..11 {
        chest.tick();
    }
    assert_eq!(chest.lid_angle, 0.0);
    assert_eq!(chest.prev_lid_angle, 0.0);
    assert!(chest.is_static_closed());
}

#[test]
fn test_banner_patterns_and_wind_oscillation() {
    let mut banner = BannerBlockEntity::new(15, false, 0); // Black banner
    banner.patterns.push(("bs".to_string(), 1)); // Red base stripe
    banner.patterns.push(("cs".to_string(), 4)); // Yellow center stripe
    banner.patterns.push(("cr".to_string(), 0)); // White cross

    assert_eq!(banner.patterns.len(), 3);
    assert_eq!(BannerBlockEntity::pattern_texture_name("bs"), "stripe_bottom");
    assert_eq!(BannerBlockEntity::pattern_texture_name("cs"), "stripe_center");
    assert_eq!(BannerBlockEntity::pattern_texture_name("cr"), "cross");
    assert_eq!(BannerBlockEntity::pattern_texture_name("unknown"), "base");

    // Wind oscillation formula: sin((time * 0.05) + pos_hash) * 0.15
    let osc1 = BannerBlockEntity::slate_rotation_radians((10, 64, 20), 100, 0.0);
    let osc2 = BannerBlockEntity::slate_rotation_radians((10, 64, 20), 100, 0.5);
    let osc3 = BannerBlockEntity::slate_rotation_radians((10, 64, 20), 120, 0.0);

    // Rotation should oscillate within [-0.15, 0.15]
    assert!(osc1 >= -0.15 && osc1 <= 0.15);
    assert!(osc2 >= -0.15 && osc2 <= 0.15);
    assert!(osc3 >= -0.15 && osc3 <= 0.15);
    assert_ne!(osc1, osc3);
}

#[test]
fn test_beacon_beam_color_mixing_and_obstruction() {
    let mut beacon = BeaconBlockEntity::new();

    // 1. Clear sky above beacon at y=64: air (id=0) all the way up
    beacon.recalculate_beam(64, |_y| (0, 0));
    assert!(beacon.is_active);
    assert_eq!(beacon.segments.len(), 1);
    assert_eq!(beacon.segments[0].color, [1.0, 1.0, 1.0]);
    assert_eq!(beacon.segments[0].height, 256 - 65);

    // 2. Red stained glass at y=70 (id=95, meta=14)
    beacon.recalculate_beam(64, |y| {
        if y == 70 {
            (95, 14) // Red stained glass
        } else {
            (0, 0)
        }
    });
    assert!(beacon.is_active);
    assert_eq!(beacon.segments.len(), 2);
    // Segment 1 (below red glass): white
    assert_eq!(beacon.segments[0].color, [1.0, 1.0, 1.0]);
    assert_eq!(beacon.segments[0].height, 5);
    // Segment 2 (through red glass): red tint [0.6, 0.2, 0.2]
    assert_eq!(beacon.segments[1].color, [0.6, 0.2, 0.2]);

    // 3. Obstructed by stone (id=1) at y=80
    beacon.recalculate_beam(64, |y| {
        if y == 80 {
            (1, 0) // Stone
        } else {
            (0, 0)
        }
    });
    assert!(!beacon.is_active);
    assert!(beacon.segments.is_empty());
}

#[test]
fn test_piston_extension_progress_and_direction() {
    // Upward extending piston (facing=1, extending=true)
    let mut piston = PistonBlockEntity::new(world::BlockState::new(1, 0), 1, true);
    assert_eq!(piston.get_progress(0.0), 0.0);
    assert_eq!(piston.get_offset(0.0), Vec3::new(0.0, -1.0, 0.0));

    // Advance 1 tick
    piston.tick();
    assert_eq!(piston.progress, 0.5);
    assert_eq!(piston.last_progress, 0.0);

    // At partial_ticks = 0.5, progress should be 0.25
    let prog = piston.get_progress(0.5);
    assert!((prog - 0.25).abs() < 1e-5);

    // Offset when extending: (p - 1.0) * dir
    // At progress 0.25, (0.25 - 1.0) * (0, 1, 0) = (0, -0.75, 0)
    let offset = piston.get_offset(0.5);
    assert!((offset.y - (-0.75)).abs() < 1e-5);
    assert_eq!(offset.x, 0.0);
    assert_eq!(offset.z, 0.0);

    // 2nd tick: fully extended (progress=1.0)
    piston.tick();
    assert_eq!(piston.progress, 1.0);
    let full_offset = piston.get_offset(1.0);
    assert_eq!(full_offset, Vec3::ZERO);
}

#[test]
fn test_enchantment_table_book_flip_and_rotation() {
    let mut table = EnchantmentTableBlockEntity::new();
    assert_eq!(table.interpolated_spread(0.0), 0.0);

    // Player far away (> 3.0 blocks): book stays closed
    let player_far = Vec3::new(10.0, 64.0, 10.0);
    let table_pos = Vec3::new(0.0, 64.0, 0.0);
    table.tick(Some(player_far), table_pos);
    assert_eq!(table.book_spread, 0.0);

    // Player walks close to table (< 3.0 blocks): book opens and faces player
    let player_near = Vec3::new(1.0, 64.0, 0.0);
    for _ in 0..20 {
        table.tick(Some(player_near), table_pos);
    }
    assert!(table.book_spread > 0.0);
    assert!(table.interpolated_spread(1.0) > 0.0);
}

#[test]
fn test_block_entity_protocol_packets_codec() {
    // 1. BlockAction packet
    let ba = ClientboundBlockActionPacket {
        position: Position { x: 10, y: 64, z: -20 },
        data1: 1, // Chest player count action
        data2: 2, // 2 players using
        block_type: 54, // Chest
    };
    let mut buf = BytesMut::new();
    ba.encode(&mut buf).unwrap();
    let mut read_buf = Bytes::from(buf.freeze());
    let decoded_ba = ClientboundBlockActionPacket::decode(&mut read_buf).unwrap();
    assert_eq!(ba, decoded_ba);

    // 2. UpdateSign packet
    let us = ClientboundUpdateSignPacket {
        position: Position { x: 5, y: 70, z: 12 },
        lines: [
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
            "Line 4".to_string(),
        ],
    };
    let mut buf = BytesMut::new();
    us.encode(&mut buf).unwrap();
    let mut read_buf = Bytes::from(buf.freeze());
    let decoded_us = ClientboundUpdateSignPacket::decode(&mut read_buf).unwrap();
    assert_eq!(us, decoded_us);

    // 3. UpdateTileEntity packet
    let mut nbt_map = std::collections::HashMap::new();
    nbt_map.insert("SkullType".to_string(), NbtTag::Byte(3));
    nbt_map.insert("Rot".to_string(), NbtTag::Byte(8));
    let ute = ClientboundUpdateTileEntityPacket {
        position: Position { x: 0, y: 65, z: 0 },
        action: 4,
        nbt_data: Some(NbtTag::Compound(nbt_map)),
    };
    let mut buf = BytesMut::new();
    ute.encode(&mut buf).unwrap();
    let mut read_buf = Bytes::from(buf.freeze());
    let decoded_ute = ClientboundUpdateTileEntityPacket::decode(&mut read_buf).unwrap();
    assert_eq!(ute, decoded_ute);
}

#[test]
fn test_chest_ground_alignment_and_bounds() {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let chest = ChestBlockEntity::new(ChestType::Normal, 3, DoubleChestType::Single);
    let pos = Vec3::new(10.0, 64.0, 20.0);
    BlockEntityRenderer::build_chest_geom(&mut vertices, &mut indices, &chest, pos, 0.0);

    assert!(!vertices.is_empty());
    assert!(!indices.is_empty());

    let min_y = vertices.iter().map(|v| v.position[1]).fold(f32::INFINITY, f32::min);
    let max_y = vertices.iter().map(|v| v.position[1]).fold(f32::NEG_INFINITY, f32::max);

    // MCP 1.8.9: chest base must sit flush on top of the block below (Y = 64.0), not floating at 64.5
    assert!(
        (min_y - 64.0).abs() < 1e-4,
        "Chest bottom must be flush with ground (64.0), but got {min_y}"
    );

    // Height of closed chest must be exactly 14/16 = 0.875 blocks (matching BlockChest bounding box)
    assert!(
        (max_y - 64.875).abs() < 1e-4,
        "Chest top must be 14 pixels high (64.875), but got {max_y}"
    );

    // Verify double chest right half is skipped so left half renders the full 30-wide model
    let mut right_vertices = Vec::new();
    let mut right_indices = Vec::new();
    let right_chest = ChestBlockEntity::new(ChestType::Normal, 3, DoubleChestType::Right);
    BlockEntityRenderer::build_chest_geom(&mut right_vertices, &mut right_indices, &right_chest, pos, 0.0);
    assert_eq!(right_vertices.len(), 0);
    assert_eq!(right_indices.len(), 0);
}
