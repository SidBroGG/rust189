use glam::DVec3;
use world::block::BlockState;
use world::physics::{
    get_block_collision_boxes, get_block_slipperiness, PlayerInput, PlayerMovement, AABB,
};
use world::World;

#[test]
fn test_aabb_offset_and_collision_math() {
    let box1 = AABB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    let box2 = AABB::new(2.0, 0.0, 0.0, 3.0, 1.0, 1.0);

    // X-axis collision test: box moving towards box2
    let offset_x = box2.calculate_x_offset(&box1, 5.0);
    assert!((offset_x - 1.0).abs() < 1e-9, "X offset should clamp to 1.0 gap");

    // Y-axis collision test
    let box_above = AABB::new(0.0, 2.0, 0.0, 1.0, 3.0, 1.0);
    let offset_y = box_above.calculate_y_offset(&box1, 5.0);
    assert!((offset_y - 1.0).abs() < 1e-9, "Y offset should clamp to 1.0 gap");

    // Z-axis collision test
    let box_south = AABB::new(0.0, 0.0, 2.0, 1.0, 1.0, 3.0);
    let offset_z = box_south.calculate_z_offset(&box1, 5.0);
    assert!((offset_z - 1.0).abs() < 1e-9, "Z offset should clamp to 1.0 gap");

    // Non-intersecting parallel axis should not clamp offset
    let box_offset = AABB::new(5.0, 5.0, 5.0, 6.0, 6.0, 6.0);
    let free_offset_x = box_offset.calculate_x_offset(&box1, 10.0);
    assert_eq!(free_offset_x, 10.0, "Disjoint boxes along Y/Z must not affect X offset");
}

#[test]
fn test_special_block_collision_bounds() {
    let world = World::new();
    let mask = AABB::new(-10.0, -10.0, -10.0, 20.0, 20.0, 20.0);

    // 1. Half-Slab (bottom, meta 0)
    let mut boxes = Vec::new();
    let slab_bottom = BlockState::new(44, 0);
    get_block_collision_boxes(&world, 0, 1, 0, slab_bottom, &mask, &mut boxes);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0], AABB::new(0.0, 1.0, 0.0, 1.0, 1.5, 1.0));

    // 2. Half-Slab (top, meta 8)
    boxes.clear();
    let slab_top = BlockState::new(44, 8);
    get_block_collision_boxes(&world, 0, 1, 0, slab_top, &mask, &mut boxes);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0], AABB::new(0.0, 1.5, 0.0, 1.0, 2.0, 1.0));

    // 3. Fence (85): height 1.5 blocks
    boxes.clear();
    let fence = BlockState::new(85, 0);
    get_block_collision_boxes(&world, 5, 1, 5, fence, &mask, &mut boxes);
    assert!(!boxes.is_empty());
    for b in &boxes {
        assert_eq!(b.max_y, 2.5, "Fence collision box max_y must be fy + 1.5");
    }

    // 4. Fence Gate (107): open gate has zero collision boxes
    boxes.clear();
    let gate_open = BlockState::new(107, 4); // meta & 4 != 0 => open
    get_block_collision_boxes(&world, 6, 1, 6, gate_open, &mask, &mut boxes);
    assert!(boxes.is_empty(), "Open fence gate must have no collision boxes");

    // Closed gate has height 1.5
    boxes.clear();
    let gate_closed = BlockState::new(107, 0);
    get_block_collision_boxes(&world, 6, 1, 6, gate_closed, &mask, &mut boxes);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].max_y, 2.5, "Closed fence gate must have max_y = fy + 1.5");

    // 5. Cactus (81): contracted by 1/16 (0.0625)
    boxes.clear();
    let cactus = BlockState::new(81, 0);
    get_block_collision_boxes(&world, 7, 1, 7, cactus, &mask, &mut boxes);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0], AABB::new(7.0625, 1.0, 7.0625, 7.9375, 1.9375, 7.9375));

    // 6. Carpet (171): height 1/16 (0.0625)
    boxes.clear();
    let carpet = BlockState::new(171, 0);
    get_block_collision_boxes(&world, 8, 1, 8, carpet, &mask, &mut boxes);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0], AABB::new(8.0, 1.0, 8.0, 9.0, 1.0625, 9.0));

    // 7. Cobweb (30) and Fluids: no solid collision
    boxes.clear();
    get_block_collision_boxes(&world, 9, 1, 9, BlockState::new(30, 0), &mask, &mut boxes);
    assert!(boxes.is_empty(), "Cobweb has no solid collision boxes");
    get_block_collision_boxes(&world, 10, 1, 10, BlockState::new(8, 0), &mask, &mut boxes);
    assert!(boxes.is_empty(), "Water has no solid collision boxes");
}

#[test]
fn test_gravity_and_terminal_velocity() {
    let world = World::new();
    let mut player = PlayerMovement::new(0.0, 1000.0, 0.0);
    player.on_ground = false;

    let input = PlayerInput::default();

    // In MCP 1.8.9, v_{n+1} = (v_n - 0.08) * 0.98.
    // 400 ticks (20s) allows 0.98^400 ≈ 0.0003 to reach 99.97% of terminal velocity (~ -3.92 blocks/tick).
    for _ in 0..400 {
        player.tick(&world, &input);
    }

    assert!(
        (player.motion.y - (-3.92)).abs() < 0.05,
        "Terminal velocity should converge to approx -3.92, got {}",
        player.motion.y
    );
}

#[test]
fn test_jump_velocity_and_sprint_boost() {
    let mut world = World::new();
    for x in -2..=2 {
        for z in -2..=2 {
            world.set_block(x, 0, z, BlockState::new(1, 0)); // Stone
        }
    }

    // 1. Normal jump without forward movement
    let mut normal_player = PlayerMovement::new(0.0, 1.0, 0.0);
    normal_player.on_ground = true;
    let normal_jump = PlayerInput {
        jump: true,
        ..Default::default()
    };
    normal_player.tick(&world, &normal_jump);
    assert!(normal_player.pos.y > 1.0, "Player should have ascended above floor");
    assert!(normal_player.motion.y > 0.3, "Player upward velocity should reflect 0.42 initial impulse");
    assert_eq!(normal_player.motion.z, 0.0);

    // 2. Normal walking jump (forward without sprint)
    let mut walk_jump_player = PlayerMovement::new(0.0, 1.0, 0.0);
    walk_jump_player.on_ground = true;
    let walk_jump = PlayerInput {
        move_forward: 1.0,
        jump: true,
        ..Default::default()
    };
    walk_jump_player.tick(&world, &walk_jump);

    // 3. Sprint jump: forward boost (+0.2 in yaw direction before ground friction)
    let mut sprint_player = PlayerMovement::new(0.0, 1.0, 0.0);
    sprint_player.on_ground = true;
    sprint_player.yaw = 0.0; // Facing south (+Z in Minecraft coords)
    let sprint_jump_input = PlayerInput {
        move_forward: 1.0,
        sprint: true,
        jump: true,
        ..Default::default()
    };
    sprint_player.tick(&world, &sprint_jump_input);
    assert!(sprint_player.is_sprinting);

    // In MCP 1.8.9:
    // walk jump: motion.z = 0.1 * 0.546 = 0.0546
    // sprint jump: motion.z = (0.2 + 0.13) * 0.546 = 0.18018
    assert!(
        sprint_player.motion.z > walk_jump_player.motion.z + 0.1,
        "Sprint jump forward impulse must significantly exceed walking jump: sprint={}, walk={}",
        sprint_player.motion.z,
        walk_jump_player.motion.z
    );
}

#[test]
fn test_media_friction_differences() {
    let mut world_stone = World::new();
    let mut world_ice = World::new();

    for x in -5..=5 {
        for z in -5..=5 {
            world_stone.set_block(x, 0, z, BlockState::new(1, 0)); // Stone
            world_ice.set_block(x, 0, z, BlockState::new(79, 0));   // Ice
        }
    }

    assert_eq!(get_block_slipperiness(1), 0.6);
    assert_eq!(get_block_slipperiness(79), 0.98);

    let mut p_stone = PlayerMovement::new(0.0, 1.0, 0.0);
    p_stone.on_ground = true;
    p_stone.motion.x = 0.5;

    let mut p_ice = PlayerMovement::new(0.0, 1.0, 0.0);
    p_ice.on_ground = true;
    p_ice.motion.x = 0.5;

    let no_input = PlayerInput::default();
    p_stone.tick(&world_stone, &no_input);
    p_ice.tick(&world_ice, &no_input);

    assert!(
        p_ice.motion.x > p_stone.motion.x,
        "Player on ice must maintain higher momentum than on stone: ice={}, stone={}",
        p_ice.motion.x,
        p_stone.motion.x
    );
}

#[test]
fn test_slime_block_bounce() {
    let mut world = World::new();
    world.set_block(0, 0, 0, BlockState::new(165, 0)); // Slime block at y = 0

    // 1. Landing on slime block without sneak -> bounce
    let mut player = PlayerMovement::new(0.5, 1.0, 0.5);
    player.motion.y = -0.5;
    let input = PlayerInput::default();
    player.tick(&world, &input);

    // MCP: motion.y inverts to +0.5, then (0.5 - 0.08) * 0.98 = 0.4116
    assert!(
        player.motion.y > 0.4,
        "Landing on slime block should invert falling velocity to positive, got {}",
        player.motion.y
    );

    // 2. Landing on slime block with sneak -> absorbs bounce
    let mut sneaking_player = PlayerMovement::new(0.5, 1.0, 0.5);
    sneaking_player.motion.y = -0.5;
    let sneak_input = PlayerInput {
        sneak: true,
        ..Default::default()
    };
    sneaking_player.tick(&world, &sneak_input);
    assert!(
        sneaking_player.motion.y <= 0.0,
        "Sneaking on slime block must absorb bounce, got {}",
        sneaking_player.motion.y
    );
}

#[test]
fn test_step_height_climbing() {
    let mut world = World::new();
    // Solid floor at y = 0
    for x in -2..=5 {
        for z in -2..=2 {
            world.set_block(x, 0, z, BlockState::new(1, 0));
        }
    }
    // Half-slab obstacle at x = 1, y = 1 (height 0.5)
    world.set_block(1, 1, 0, BlockState::new(44, 0));

    let mut player = PlayerMovement::new(0.0, 1.0, 0.5);
    player.on_ground = true;
    player.yaw = -90.0; // Facing east (+X)

    // Player walks forward into the slab WITHOUT jumping
    let forward_input = PlayerInput {
        move_forward: 1.0,
        jump: false,
        ..Default::default()
    };

    for _ in 0..10 {
        player.tick(&world, &forward_input);
    }

    assert!(
        player.pos.x > 0.8,
        "Player should step forward onto the slab, pos.x = {}",
        player.pos.x
    );
    assert!(
        player.pos.y >= 1.5,
        "Player should step up onto the slab (y >= 1.5), pos.y = {}",
        player.pos.y
    );
}

#[test]
fn test_sneaking_ledge_protection() {
    let mut world = World::new();
    // Single block column at (0, 0, 0)
    world.set_block(0, 0, 0, BlockState::new(1, 0));

    let mut player = PlayerMovement::new(0.5, 1.0, 0.5);
    player.on_ground = true;
    player.yaw = -90.0; // Facing east (+X)

    let sneak_forward = PlayerInput {
        move_forward: 1.0,
        sneak: true,
        ..Default::default()
    };

    // Attempt to walk off the edge for 20 ticks
    for _ in 0..20 {
        player.tick(&world, &sneak_forward);
    }

    // In MCP 1.8.9, sneaking stops the player once the trailing edge of their AABB
    // (width 0.6, radius 0.3) reaches the boundary of the block, preventing a fall.
    assert!(
        player.pos.x < 1.30,
        "Sneaking player should not fall off ledge, pos.x = {}",
        player.pos.x
    );
    assert!(
        player.pos.y >= 1.0,
        "Sneaking player must stay on top of the block, pos.y = {}",
        player.pos.y
    );
}

#[test]
fn test_ray_trace_blocks() {
    let mut world = World::new();
    world.set_block(5, 1, 0, BlockState::new(1, 0)); // Stone at x=5, y=1, z=0

    let start = DVec3::new(0.0, 1.5, 0.5);
    let end = DVec3::new(10.0, 1.5, 0.5);

    let hit = world.ray_trace_blocks(start, end).expect("Should hit stone block");
    assert_eq!(hit.block_pos, (5, 1, 0));
    assert_eq!(hit.hit_normal, DVec3::NEG_X, "Ray enters from west (-X face)");
    assert!((hit.hit_pos.x - 5.0).abs() < 1e-6);
}

#[test]
fn test_ladder_collision_and_climbing() {
    let world = World::new();
    let mask = AABB::new(-10.0, -10.0, -10.0, 20.0, 20.0, 20.0);
    let mut boxes = Vec::new();

    // Ladder facing North (meta 2): thickness 0.125 against z=1.0
    let ladder_north = BlockState::new(65, 2);
    get_block_collision_boxes(&world, 0, 1, 0, ladder_north, &mask, &mut boxes);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0], AABB::new(0.0, 1.0, 0.875, 1.0, 2.0, 1.0));

    // Ladder facing South (meta 3): thickness 0.125 against z=0.0
    boxes.clear();
    let ladder_south = BlockState::new(65, 3);
    get_block_collision_boxes(&world, 0, 1, 0, ladder_south, &mask, &mut boxes);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0], AABB::new(0.0, 1.0, 0.0, 1.0, 2.0, 0.125));
}

#[test]
fn test_jump_delay_ticks() {
    let world = World::new();
    let mut player = PlayerMovement::new(0.0, 0.0, 0.0);
    player.on_ground = true;

    let input_jump = PlayerInput {
        jump: true,
        ..Default::default()
    };

    // First jump: jump_ticks is set to 10
    player.tick(&world, &input_jump);
    assert!(player.motion.y > 0.3, "First jump must produce upward motion");
    assert_eq!(player.jump_ticks, 10, "jump_ticks should be initialized to 10");

    // Next tick while still holding space: jump_ticks decrements to 9
    player.on_ground = true;
    player.motion.y = 0.0;
    player.tick(&world, &input_jump);
    assert_eq!(player.jump_ticks, 9);
    assert!(player.motion.y <= 0.0, "Cannot jump again while jump_ticks > 0");

    // Release jump: jump_ticks resets to 0
    let input_release = PlayerInput {
        jump: false,
        ..Default::default()
    };
    player.tick(&world, &input_release);
    assert_eq!(player.jump_ticks, 0, "Releasing space must reset jump_ticks to 0");
}

#[test]
fn test_creative_flight_double_tap() {
    let world = World::new();
    let mut player = PlayerMovement::new(0.0, 10.0, 0.0);
    player.capabilities.allow_flying = true;
    assert!(!player.capabilities.is_flying);

    // Tap 1: space pressed
    let tap1 = PlayerInput {
        jump: true,
        ..Default::default()
    };
    player.tick(&world, &tap1);
    assert_eq!(player.fly_toggle_timer, 7, "fly_toggle_timer set to 7 on first jump tap");
    assert!(!player.capabilities.is_flying);

    // Space released
    let release = PlayerInput {
        jump: false,
        ..Default::default()
    };
    player.tick(&world, &release);
    assert_eq!(player.fly_toggle_timer, 6);

    // Tap 2 within 7 ticks: activates flying!
    player.tick(&world, &tap1);
    assert!(player.capabilities.is_flying, "Double-tap space must toggle flying on");
    assert_eq!(player.fly_toggle_timer, 0);

    // Ascend while flying
    let fly_input = PlayerInput {
        jump: true,
        ..Default::default()
    };
    player.motion.y = 0.0;
    player.tick(&world, &fly_input);
    assert!(player.motion.y > 0.05, "Holding jump while flying must accelerate upward");
}

#[test]
fn test_continuous_wall_collision_no_penetration_or_bounce() {
    let mut world = World::new();
    // Solid floor at y = 0
    for x in -5..=5 {
        for z in -5..=5 {
            world.set_block(x, 0, z, BlockState::new(1, 0));
        }
    }
    // Solid 2-high wall at x = 2, y = 1..2
    for z in -5..=5 {
        world.set_block(2, 1, z, BlockState::new(1, 0));
        world.set_block(2, 2, z, BlockState::new(1, 0));
    }

    // Player starts at x = 1.0, y = 1.0, z = 0.5, moving east towards x = 2.0
    let mut player = PlayerMovement::new(1.0, 1.0, 0.5);
    player.on_ground = true;
    player.yaw = -90.0; // Facing east (+X)

    let forward_input = PlayerInput {
        move_forward: 1.0,
        jump: false,
        ..Default::default()
    };

    // Run 50 ticks of running directly into the wall
    for _ in 0..50 {
        player.tick(&world, &forward_input);
        // Player's max_x must NEVER exceed 2.0 (the wall boundary)
        assert!(
            player.bounding_box.max_x <= 2.0 + 1e-9,
            "Player bounding box penetrated wall! max_x = {}",
            player.bounding_box.max_x
        );
        // Once collided against the wall at x=2.0 (half_w=0.3), position must stay at 1.7
        if player.collided_horizontally {
            assert!(
                (player.pos.x - 1.7).abs() <= 1e-4,
                "Player position must stay stable at wall without bouncing back, pos.x = {}",
                player.pos.x
            );
        }
    }
}

#[test]
fn test_push_out_of_blocks_ejection() {
    let mut world = World::new();
    // Solid block at (5, 1, 5)
    world.set_block(5, 1, 5, BlockState::new(1, 0));
    // Air around it at (4, 1, 5), (6, 1, 5), (5, 1, 4), (5, 1, 6)

    let mut player = PlayerMovement::new(0.0, 0.0, 0.0);
    // Test pushing out from a corner inside (5, 1, 5) close to west face (x = 5.05)
    let pushed = player.push_out_of_blocks(&world, 5.05, 1.5, 5.5);
    assert!(pushed, "Should push out of solid block");
    assert_eq!(player.motion.x, -0.1, "Must push west with -0.1 impulse towards closest open block");

    // Test pushing out close to east face (x = 5.95)
    player.motion = DVec3::ZERO;
    let pushed_east = player.push_out_of_blocks(&world, 5.95, 1.5, 5.5);
    assert!(pushed_east);
    assert_eq!(player.motion.x, 0.1, "Must push east with 0.1 impulse towards closest open block");
}

#[test]
fn test_stairs_straight_and_corner_collision() {
    let mut world = World::new();
    let mask = AABB::new(-10.0, -10.0, -10.0, 20.0, 20.0, 20.0);

    // 1. Straight east-facing stair (ID 53, meta 0)
    let straight_stair = BlockState::new(53, 0);
    let mut boxes = Vec::new();
    get_block_collision_boxes(&world, 0, 1, 0, straight_stair, &mask, &mut boxes);
    // Base slab + 1 step piece
    assert_eq!(boxes.len(), 2, "Straight stair must produce base slab and step");
    assert_eq!(boxes[0], AABB::new(0.0, 1.0, 0.0, 1.0, 1.5, 1.0)); // Base slab
    assert_eq!(boxes[1], AABB::new(0.5, 1.5, 0.0, 1.0, 2.0, 1.0)); // East step (+X)

    // 2. Corner stair: East stair connected to South stair on east (+X) side
    world.set_block(0, 1, 0, BlockState::new(53, 0)); // East facing
    world.set_block(1, 1, 0, BlockState::new(53, 2)); // South facing to the east
    boxes.clear();
    get_block_collision_boxes(&world, 0, 1, 0, BlockState::new(53, 0), &mask, &mut boxes);
    // Outer corner stairs produce base slab + quarter step
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[1], AABB::new(0.5, 1.5, 0.5, 1.0, 2.0, 1.0)); // Outer corner quarter step!
}

#[test]
fn test_cobblestone_wall_dimensions() {
    let world = World::new();
    let mask = AABB::new(-10.0, -10.0, -10.0, 20.0, 20.0, 20.0);

    // Isolated Cobblestone Wall (id 139)
    let wall = BlockState::new(139, 0);
    let mut boxes = Vec::new();
    get_block_collision_boxes(&world, 0, 1, 0, wall, &mask, &mut boxes);
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].min_x, 0.25);
    assert_eq!(boxes[0].max_x, 0.75);
    assert_eq!(boxes[0].min_z, 0.25);
    assert_eq!(boxes[0].max_z, 0.75);
    assert_eq!(boxes[0].max_y, 2.5, "Cobblestone wall height must be 1.5 blocks (1.0 + 1.5 = 2.5)");
}

#[test]
fn test_water_climbing_onto_shore() {
    let mut world = World::new();
    // Deep water: bottom floor at y=50
    for x in -2..=3 {
        for z in -2..=3 {
            world.set_block(x, 50, z, BlockState::new(1, 0));
        }
    }
    // Water from y=51 to y=63 for x <= 0
    for y in 51..=63 {
        for x in -2..=0 {
            for z in -2..=2 {
                world.set_block(x, y, z, BlockState::new(8, 0)); // Flowing water
            }
        }
    }
    // Shore block at (1, 63, 0)
    world.set_block(1, 63, 0, BlockState::new(1, 0)); // Stone shore
    world.set_block(1, 63, 1, BlockState::new(1, 0));
    world.set_block(1, 63, -1, BlockState::new(1, 0));

    // Player starts floating near surface at (0.5, 63.0, 0.0), facing +X (yaw = -90.0)
    let mut player = PlayerMovement::new(0.5, 63.0, 0.0);
    player.yaw = -90.0;

    let input = PlayerInput {
        move_forward: 1.0,
        move_strafe: 0.0,
        jump: true,
        sneak: false,
        sprint: true,
    };

    println!("\n=== Starting Water Climb Test ===");
    for tick in 0..40 {
        player.tick(&world, &input);
        println!(
            "Tick {:2}: pos=({:.4}, {:.4}, {:.4}) motion=({:.4}, {:.4}, {:.4}) in_water={} coll_h={} on_ground={}",
            tick,
            player.pos.x,
            player.pos.y,
            player.pos.z,
            player.motion.x,
            player.motion.y,
            player.motion.z,
            player.is_in_water,
            player.collided_horizontally,
            player.on_ground,
        );
    }

    // Verify whether the player successfully climbed onto the shore (pos.x > 1.0, pos.y >= 64.0)
    println!("Final player pos: {:?}", player.pos);
}

#[test]
fn test_water_climbing_natural_bank() {
    let mut world = World::new();
    // Deep water: bottom floor at y=50
    for x in -2..=3 {
        for z in -2..=3 {
            world.set_block(x, 50, z, BlockState::new(1, 0));
        }
    }
    // Water from y=51 to y=62 for x <= 0 (surface at 62.888)
    for y in 51..=62 {
        for x in -2..=0 {
            for z in -2..=2 {
                world.set_block(x, y, z, BlockState::new(8, 0));
            }
        }
    }
    // Shore blocks at x=1 for y <= 63 (top of bank is at y=64.0)
    for y in 51..=63 {
        world.set_block(1, y, 0, BlockState::new(1, 0));
        world.set_block(1, y, 1, BlockState::new(1, 0));
        world.set_block(1, y, -1, BlockState::new(1, 0));
    }

    // Player starts in water near surface at (0.5, 62.0, 0.0), facing +X (yaw = -90.0)
    let mut player = PlayerMovement::new(0.5, 62.0, 0.0);
    player.yaw = -90.0;

    let input = PlayerInput {
        move_forward: 1.0,
        move_strafe: 0.0,
        jump: true,
        sneak: false,
        sprint: false,
    };

    println!("\n=== Starting Natural Bank Climb Test ===");
    for tick in 0..40 {
        player.tick(&world, &input);
        println!(
            "Tick {:2}: pos=({:.4}, {:.4}, {:.4}) motion=({:.4}, {:.4}, {:.4}) in_water={} coll_h={} on_ground={}",
            tick,
            player.pos.x,
            player.pos.y,
            player.pos.z,
            player.motion.x,
            player.motion.y,
            player.motion.z,
            player.is_in_water,
            player.collided_horizontally,
            player.on_ground,
        );
    }

    println!("Final natural bank player pos: {:?}", player.pos);
}


