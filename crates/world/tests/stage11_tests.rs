use glam::DVec3;
use protocol::BlockFace;
use world::block::BlockState;
use world::entity::EntityTracker;
use world::physics::{
    get_block_hardness, get_player_relative_block_hardness, get_weapon_damage,
    is_critical_hit, pick_target, ray_trace_voxels, ClickTracker,
    EntitySpatialHash, ItemUseAction, ItemUseState, TargetHit, ToolMaterial, AABB,
};
use world::World;

#[test]
fn test_amanatides_woo_voxel_dda_traversal() {
    let mut world = World::new();

    // Place stone block at (5, 64, 0)
    world.set_block(5, 64, 0, BlockState::new(1, 0));

    // Raycast from (0.5, 64.5, 0.5) along +X towards the block
    let start = DVec3::new(0.5, 64.5, 0.5);
    let dir = DVec3::new(1.0, 0.0, 0.0);
    let hit = ray_trace_voxels(&world, start, dir, 10.0);

    assert!(hit.is_some(), "Ray should hit the stone block");
    let hit = hit.unwrap();
    assert_eq!(hit.block_pos, (5, 64, 0));
    assert_eq!(hit.face, BlockFace::West);
    assert!((hit.distance - 4.5).abs() < 1e-4, "Distance should be 4.5");

    // Raycast in opposite direction should miss
    let miss = ray_trace_voxels(&world, start, -dir, 10.0);
    assert!(miss.is_none(), "Ray in opposite direction must miss");

    // Raycast with reach less than distance should miss
    let short_hit = ray_trace_voxels(&world, start, dir, 4.0);
    assert!(short_hit.is_none(), "Short ray must not reach block");

    // Diagonal raycast through multiple voxels
    world.set_block(2, 66, 2, BlockState::new(1, 0));
    let diag_start = DVec3::new(0.5, 64.5, 0.5);
    let diag_dir = (DVec3::new(2.5, 66.5, 2.5) - diag_start).normalize();
    let diag_hit = ray_trace_voxels(&world, diag_start, diag_dir, 10.0);
    assert!(diag_hit.is_some(), "Diagonal ray must hit target block");
    assert_eq!(diag_hit.unwrap().block_pos, (2, 66, 2));
}

#[test]
fn test_entity_spatial_hashing_and_raycast() {
    let mut tracker = EntityTracker::new();
    let mut spatial_hash = EntitySpatialHash::new(4.0);

    // Spawn zombie at (0.0, 64.0, 5.0) -> fixed coordinates: 0, 64 * 32 = 2048, 5 * 32 = 160
    tracker.spawn_mob(
        100,
        54, // Zombie
        0,
        2048,
        160,
        0,
        0,
        0,
        0,
        0,
        0,
        Default::default(),
    );

    // Spawn player at (0.0, 64.0, 10.0) -> fixed coordinates: 0, 2048, 10 * 32 = 320
    tracker.spawn_player(
        101,
        uuid::Uuid::nil(),
        "TestPlayer".to_string(),
        0,
        2048,
        320,
        0,
        0,
        0,
        Default::default(),
    );

    spatial_hash.rebuild(&tracker);

    // Raycast along +Z from (0.0, 65.0, 0.0)
    let start = DVec3::new(0.0, 65.0, 0.0);
    let dir = DVec3::new(0.0, 0.0, 1.0);

    let hit = spatial_hash.ray_trace_entities(&tracker, start, dir, 15.0, None, 1.0);
    assert!(hit.is_some(), "Should hit the first entity (Zombie 100)");
    let hit = hit.unwrap();
    assert_eq!(hit.entity_id, 100);
    assert!((hit.distance - 4.6).abs() < 0.2);

    // Ignore zombie (id 100), should hit player behind (id 101)
    let hit_behind = spatial_hash.ray_trace_entities(&tracker, start, dir, 15.0, Some(100), 1.0);
    assert!(hit_behind.is_some(), "Should hit second entity when first is ignored");
    assert_eq!(hit_behind.unwrap().entity_id, 101);
}

#[test]
fn test_unified_target_picking() {
    let mut world = World::new();
    let mut spatial_hash = EntitySpatialHash::new(4.0);

    // Block at distance 4.0
    world.set_block(0, 64, 4, BlockState::new(1, 0));

    // Entity at distance 2.0 (in front of block) -> fixed coordinates: 0, 2048, 64
    world.entity_tracker.spawn_mob(200, 54, 0, 2048, 64, 0, 0, 0, 0, 0, 0, Default::default());
    spatial_hash.rebuild(&world.entity_tracker);

    let eye_pos = DVec3::new(0.0, 64.5, 0.0);
    let look_dir = DVec3::new(0.0, 0.0, 1.0);

    // Entity in front should be picked over the block behind it
    let target = pick_target(&world, &spatial_hash, eye_pos, look_dir, false, None, 1.0);
    match target {
        TargetHit::Entity(eh) => assert_eq!(eh.entity_id, 200),
        _ => panic!("Expected Entity hit"),
    }

    // When entity is removed, block should be picked
    world.entity_tracker.entities.clear();
    spatial_hash.rebuild(&world.entity_tracker);
    let target2 = pick_target(&world, &spatial_hash, eye_pos, look_dir, false, None, 1.0);
    match target2 {
        TargetHit::Block(bh) => assert_eq!(bh.block_pos, (0, 64, 4)),
        _ => panic!("Expected Block hit"),
    }
}

#[test]
fn test_npc_armor_stand_interaction_and_marker_exclusion() {
    let mut world = World::new();
    let spatial_hash = EntitySpatialHash::new(4.0);

    // Block behind NPC at z = 4
    world.set_block(0, 64, 4, BlockState::new(1, 0));

    // Spawn ArmorStand NPC at (0.0, 64.0, 2.5) -> object 78
    // Fixed coordinates: 0, 64 * 32 = 2048, 2.5 * 32 = 80
    world.entity_tracker.spawn_object(300, 78, 0, 2048, 80, 0, 0, 0, 0, 0, 0);

    // Player looks forward at eye height 65.5 (torso/head level of ArmorStand)
    let eye_pos = DVec3::new(0.0, 65.5, 0.0);
    let look_dir = DVec3::new(0.0, 0.0, 1.0);

    // Raycast should hit the ArmorStand NPC at height 1.975 (not the block behind)
    let target = pick_target(&world, &spatial_hash, eye_pos, look_dir, false, None, 1.0);
    match target {
        TargetHit::Entity(eh) => {
            assert_eq!(eh.entity_id, 300);
            assert!(eh.distance < 3.0);
        }
        _ => panic!("Expected ArmorStand NPC entity hit"),
    }

    // Now spawn a Marker ArmorStand (floating hologram) between player and NPC at z = 1.0
    // Marker flag has index 10 byte with bit 0x10 set
    let marker_metadata = vec![protocol::codec::MetadataEntry {
        index: 10,
        value: protocol::codec::MetadataValue::Byte(0x10),
    }];
    world.entity_tracker.spawn_object(301, 78, 0, 2048, 32, 0, 0, 0, 0, 0, 0);
    world.entity_tracker.update_metadata(301, marker_metadata);

    // Marker armorstand must NOT be collided with, ray must pass through to NPC (id 300)
    let target_through_marker = pick_target(&world, &spatial_hash, eye_pos, look_dir, false, None, 1.0);
    match target_through_marker {
        TargetHit::Entity(eh) => assert_eq!(eh.entity_id, 300, "Should hit NPC behind marker hologram"),
        _ => panic!("Expected ArmorStand NPC hit through marker"),
    }

    // Also test invisible armor stand (hologram floating text over chests)
    let invisible_metadata = vec![protocol::codec::MetadataEntry {
        index: 0,
        value: protocol::codec::MetadataValue::Byte(0x20), // Bit 5 = invisible
    }];
    world.entity_tracker.spawn_object(302, 78, 0, 2048, 48, 0, 0, 0, 0, 0, 0);
    world.entity_tracker.update_metadata(302, invisible_metadata);

    let target_through_invisible = pick_target(&world, &spatial_hash, eye_pos, look_dir, false, None, 1.0);
    match target_through_invisible {
        TargetHit::Entity(eh) => assert_eq!(eh.entity_id, 300, "Should hit NPC behind invisible hologram"),
        _ => panic!("Expected ArmorStand NPC hit through invisible hologram"),
    }
}

#[test]
fn test_mcp_digging_speed_and_relative_hardness() {
    // 1. Tool material efficiencies
    assert_eq!(ToolMaterial::Wood.efficiency_multiplier(), 2.0);
    assert_eq!(ToolMaterial::Stone.efficiency_multiplier(), 4.0);
    assert_eq!(ToolMaterial::Iron.efficiency_multiplier(), 6.0);
    assert_eq!(ToolMaterial::Diamond.efficiency_multiplier(), 8.0);
    assert_eq!(ToolMaterial::Gold.efficiency_multiplier(), 12.0);

    // 2. Stone (ID 1, hardness 1.5)
    let stone_id = 1u16;
    let hardness = get_block_hardness(stone_id);
    assert_eq!(hardness, 1.5);

    // Bare hand on stone: speed = 1.0, divisor = 100.0 (cannot harvest)
    let hand_damage = get_player_relative_block_hardness(
        stone_id,
        0, // Bare hand
        0,
        0,
        0,
        false,
        false,
        true,
        false,
    );
    let expected_hand = 1.0 / 1.5 / 100.0;
    assert!((hand_damage - expected_hand).abs() < 1e-6);

    // Diamond pickaxe (278) on stone: speed = 8.0, divisor = 30.0 (can harvest)
    let dia_pick_damage = get_player_relative_block_hardness(
        stone_id,
        278,
        0,
        0,
        0,
        false,
        false,
        true,
        false,
    );
    let expected_dia = 8.0 / 1.5 / 30.0;
    assert!((dia_pick_damage - expected_dia).abs() < 1e-6);

    // Efficiency V on diamond pickaxe: speed = 8.0 + (5*5 + 1) = 34.0
    let eff5_damage = get_player_relative_block_hardness(
        stone_id,
        278,
        5, // Efficiency V
        0,
        0,
        false,
        false,
        true,
        false,
    );
    let expected_eff5 = 34.0 / 1.5 / 30.0;
    assert!((eff5_damage - expected_eff5).abs() < 1e-6);

    // Environmental penalties: not on ground (/ 5.0) and in water (/ 5.0)
    let air_damage = get_player_relative_block_hardness(
        stone_id,
        278,
        0,
        0,
        0,
        false,
        false,
        false, // In air
        false,
    );
    assert!((air_damage - (expected_dia / 5.0)).abs() < 1e-6);

    let water_damage = get_player_relative_block_hardness(
        stone_id,
        278,
        0,
        0,
        0,
        true, // In water
        false,
        true,
        false,
    );
    assert!((water_damage - (expected_dia / 5.0)).abs() < 1e-6);

    // Creative mode instant break
    let creative_damage = get_player_relative_block_hardness(
        stone_id,
        0,
        0,
        0,
        0,
        false,
        false,
        true,
        true, // Creative
    );
    assert_eq!(creative_damage, 1.0);

    // Bedrock unbreakable
    let bedrock_damage = get_player_relative_block_hardness(
        7, // Bedrock
        278,
        5,
        2,
        0,
        false,
        false,
        true,
        false,
    );
    assert_eq!(bedrock_damage, 0.0);
}

#[test]
fn test_pvp_1_8_hit_reg_and_critical_hits() {
    // 1. ClickTracker zero-cooldown registration
    let mut tracker = ClickTracker::new();
    for _ in 0..25 {
        tracker.register_left_click();
    }
    assert_eq!(tracker.left_cps(), 25, "1.8 PvP spam-clicking must track all 25 clicks");

    // 2. Critical Hit detection formula (MCP EntityPlayer)
    // Valid crit: falling, not on ground, not in water, not on ladder, not riding, no blindness
    assert!(is_critical_hit(0.5, false, false, false, false, false));

    // Invalid crits
    assert!(!is_critical_hit(0.0, false, false, false, false, false), "No fall distance");
    assert!(!is_critical_hit(0.5, true, false, false, false, false), "On ground");
    assert!(!is_critical_hit(0.5, false, true, false, false, false), "On ladder");
    assert!(!is_critical_hit(0.5, false, false, true, false, false), "In water");
    assert!(!is_critical_hit(0.5, false, false, false, true, false), "Riding entity");
    assert!(!is_critical_hit(0.5, false, false, false, false, true), "Blindness potion");

    // 3. Weapon base damage values
    assert_eq!(get_weapon_damage(276), 7.0); // Diamond Sword
    assert_eq!(get_weapon_damage(267), 6.0); // Iron Sword
    assert_eq!(get_weapon_damage(272), 5.0); // Stone Sword
    assert_eq!(get_weapon_damage(279), 6.0); // Diamond Axe
    assert_eq!(get_weapon_damage(0), 1.0);   // Fist
}

#[test]
fn test_item_use_sword_blocking_bow_and_eating() {
    let mut state = ItemUseState::new();

    // 1. Sword Blocking (1.8 PvP)
    assert_eq!(ItemUseState::get_action_for_item(276), ItemUseAction::Block);
    state.start_using(276);
    assert!(state.is_blocking());
    assert_eq!(state.movement_speed_multiplier(), 0.2); // 0.2 speed reduction
    state.stop();
    assert!(!state.is_blocking());
    assert_eq!(state.movement_speed_multiplier(), 1.0);

    // 2. Bow Pulling & Release
    assert_eq!(ItemUseState::get_action_for_item(261), ItemUseAction::Bow);
    state.start_using(261);
    assert!(state.is_bow_drawing());
    assert_eq!(state.bow_pull_stage(), 0);

    // Tick 12 times (stage 1)
    for _ in 0..12 {
        state.tick();
    }
    assert_eq!(state.bow_pull_stage(), 1);

    // Tick to 20 times (fully pulled, stage 2)
    for _ in 0..8 {
        state.tick();
    }
    assert_eq!(state.bow_pull_stage(), 2);

    let release = state.release_bow();
    assert!(release.is_some());
    let (vel, is_crit) = release.unwrap();
    assert_eq!(vel, 2.0, "Fully pulled bow has max velocity 2.0");
    assert!(is_crit, "Fully pulled bow shoots critical arrow");
    assert!(!state.is_bow_drawing());

    // 3. Food Eating (Golden Apple 322)
    assert_eq!(ItemUseState::get_action_for_item(322), ItemUseAction::Eat);
    state.start_using(322);
    assert_eq!(state.max_duration, 32); // 32 ticks = 1.6s

    let mut eaten = false;
    for _ in 0..32 {
        if state.tick() {
            eaten = true;
            break;
        }
    }
    assert!(eaten, "Food consumption should complete at 32 ticks");
}

#[test]
fn test_block_placement_collision_validation() {
    // Player AABB at (0.0, 64.0, 0.0) to (0.6, 65.8, 0.6)
    let player_aabb = AABB::new(0.0, 64.0, 0.0, 0.6, 65.8, 0.6);

    // Block at same position (0, 64, 0)
    let inside_block_aabb = AABB::new(0.0, 64.0, 0.0, 1.0, 65.0, 1.0);
    assert!(
        player_aabb.intersects(&inside_block_aabb),
        "Cannot place block inside player bounding box"
    );

    // Block next to player (2, 64, 0)
    let outside_block_aabb = AABB::new(2.0, 64.0, 0.0, 3.0, 65.0, 1.0);
    assert!(
        !player_aabb.intersects(&outside_block_aabb),
        "Block outside player bounding box can be placed"
    );
}

#[test]
fn test_mouse_interaction_predicates_and_toggling() {
    use world::{is_interactive_block, is_replaceable_block};

    // 1. Interactive blocks
    assert!(is_interactive_block(64));  // Oak Door
    assert!(is_interactive_block(193)); // Spruce Door
    assert!(is_interactive_block(96));  // Wooden Trapdoor
    assert!(is_interactive_block(107)); // Fence Gate
    assert!(is_interactive_block(69));  // Lever
    assert!(is_interactive_block(77));  // Stone Button
    assert!(is_interactive_block(143)); // Wood Button
    assert!(is_interactive_block(54));  // Chest
    assert!(is_interactive_block(58));  // Crafting Table
    assert!(!is_interactive_block(1));  // Stone
    assert!(!is_interactive_block(71)); // Iron Door (redstone only)

    // 2. Door / Trapdoor / Gate bit toggling (bit 0x04)
    let trapdoor_closed = BlockState::new(96, 0);
    let trapdoor_open = BlockState::new(96, trapdoor_closed.meta() ^ 4);
    assert_eq!(trapdoor_open.meta(), 4);
    assert_eq!(trapdoor_open.meta() ^ 4, 0);

    // 3. Lever / Button powered bit toggling (bit 0x08)
    let lever_off = BlockState::new(69, 1);
    let lever_on = BlockState::new(69, lever_off.meta() ^ 8);
    assert_eq!(lever_on.meta(), 9);
    assert_eq!(lever_on.meta() ^ 8, 1);

    // 4. Replaceable blocks
    assert!(is_replaceable_block(0));   // Air
    assert!(is_replaceable_block(31));  // Tall Grass
    assert!(is_replaceable_block(78));  // Snow layer
    assert!(is_replaceable_block(106)); // Vine
    assert!(!is_replaceable_block(1));  // Stone
    assert!(!is_replaceable_block(4));  // Cobblestone
}

#[test]
fn test_continuous_right_click_delay_timer() {
    let mut right_click_delay_timer: u8 = 4; // Set on initial right click

    // Simulate 4 physics ticks (20 TPS, 50ms each)
    let mut placements = 0;
    for _ in 0..12 {
        if right_click_delay_timer > 0 {
            right_click_delay_timer -= 1;
        }
        if right_click_delay_timer == 0 {
            placements += 1;
            right_click_delay_timer = 4; // Reset to 4
        }
    }

    // 12 ticks with delay 4 = exactly 3 placements (1 every 4 ticks)
    assert_eq!(placements, 3, "Continuous right click should place 1 block every 4 ticks");
}

