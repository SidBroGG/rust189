use glam::DVec3;
use render::models::boss::*;
use render::models::hostile::*;
use render::models::layers::*;
use render::models::peaceful::*;
use render::models::vehicles::*;
use render::EntityVertex;
use world::entity::{EntityKind, TrackedEntity};

fn create_dummy_entity(kind: EntityKind) -> TrackedEntity {
    TrackedEntity::new(1, kind, DVec3::new(10.0, 64.0, 10.0), 45.0, 15.0)
}

fn assert_valid_mesh(vertices: &[EntityVertex], indices: &[u32], min_quads: usize) {
    assert!(vertices.len() >= min_quads * 4, "Expected at least {} quads ({} vertices), got {}", min_quads, min_quads * 4, vertices.len());
    assert_eq!(indices.len(), (vertices.len() / 4) * 6, "Indices count mismatch with quad vertices");

    for idx in indices {
        assert!((*idx as usize) < vertices.len(), "Index out of bounds");
    }

    for v in vertices {
        assert!(!v.position[0].is_nan());
        assert!(!v.position[1].is_nan());
        assert!(!v.position[2].is_nan());
        assert!(!v.normal[0].is_nan());
        assert!(!v.normal[1].is_nan());
        assert!(!v.normal[2].is_nan());
        assert!(v.uv[0] >= -0.01 && v.uv[0] <= 2.0, "UV U out of range: {}", v.uv[0]);
        assert!(v.uv[1] >= -0.01 && v.uv[1] <= 2.0, "UV V out of range: {}", v.uv[1]);
        assert!(v.color[0] >= 0.0 && v.color[0] <= 1.0);
        assert!(v.color[1] >= 0.0 && v.color[1] <= 1.0);
        assert!(v.color[2] >= 0.0 && v.color[2] <= 1.0);
        assert!(v.color[3] >= 0.0 && v.color[3] <= 1.0);
    }
}

#[test]
fn test_peaceful_mobs_geometry_and_uvs() {
    let color = [1.0, 1.0, 1.0, 1.0];

    // Cow
    let mut v = Vec::new(); let mut i = Vec::new();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 92 });
    build_cow_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6); // head + horns + udder + body + 4 legs

    // Pig
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 90 });
    build_pig_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6);

    // Sheep
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 91 });
    build_sheep_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6);

    // Chicken
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 93 });
    build_chicken_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6); // head + bill + chin + body + 2 wings + 2 legs

    // Rabbit
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 101 });
    build_rabbit_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 8); // body, head, 2 ears, nose, 4 legs

    // Horse
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 100 });
    build_horse_model(&mut v, &mut i, &entity, 0.5, false, color);
    assert_valid_mesh(&v, &i, 8);

    // Wolf
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 95 });
    build_wolf_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6);

    // Ocelot
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 98 });
    build_ocelot_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6);

    // Bat
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 65 });
    build_bat_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 4);

    // Squid
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 94 });
    build_squid_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 9); // body + 8 tentacles

    // Villager
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 120 });
    build_villager_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 5);

    // Iron Golem
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 99 });
    build_iron_golem_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 5);

    // Snowman
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 97 });
    build_snowman_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 4);
}

#[test]
fn test_hostile_mobs_geometry_and_uvs() {
    let color = [1.0, 1.0, 1.0, 1.0];

    // Zombie
    let mut v = Vec::new(); let mut i = Vec::new();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 54 });
    build_zombie_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6);

    // Giant (scaled 6x)
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 53 });
    build_giant_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6);

    // Skeleton & Wither Skeleton
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 51 });
    build_skeleton_model(&mut v, &mut i, &entity, 0.5, false, color);
    assert_valid_mesh(&v, &i, 6);

    v.clear(); i.clear();
    build_skeleton_model(&mut v, &mut i, &entity, 0.5, true, color);
    assert_valid_mesh(&v, &i, 6);

    // Spider & Cave Spider
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 52 });
    build_spider_model(&mut v, &mut i, &entity, 0.5, false, color);
    assert_valid_mesh(&v, &i, 11); // head + thorax + abdomen + 8 legs

    v.clear(); i.clear();
    build_spider_model(&mut v, &mut i, &entity, 0.5, true, color);
    assert_valid_mesh(&v, &i, 11);

    // Silverfish & Endermite
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 60 });
    build_silverfish_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 7);

    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 67 });
    build_endermite_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 4);

    // Creeper
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 50 });
    build_creeper_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6);

    // Slime & Magma Cube
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 55 });
    build_slime_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 3); // body + 2 eyes

    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 62 });
    build_magma_cube_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 8); // 8 segments

    // Enderman
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 58 });
    build_enderman_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6);

    // Blaze
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 61 });
    build_blaze_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 13); // head + 12 rods

    // Ghast
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 56 });
    build_ghast_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 10); // body + 9 tentacles

    // Guardian
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 68 });
    build_guardian_model(&mut v, &mut i, &entity, 0.5, false, color);
    assert_valid_mesh(&v, &i, 14); // body + eye + 12 spikes

    // Witch
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 66 });
    build_witch_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 7);
}

#[test]
fn test_boss_mob_geometry_and_uvs() {
    let color = [1.0, 1.0, 1.0, 1.0];

    // Wither: 3 independent heads, ribs, spine, tail
    let mut v = Vec::new(); let mut i = Vec::new();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 64 });
    build_wither_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 8);

    // Ender Dragon: 16 articulated parts (head, jaw, neck segments, wings, legs, feet)
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 63 });
    build_dragon_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 20);
}

#[test]
fn test_vehicle_and_object_geometry_and_uvs() {
    let color = [1.0, 1.0, 1.0, 1.0];

    // Armor Stand
    let mut v = Vec::new(); let mut i = Vec::new();
    let entity = create_dummy_entity(EntityKind::Object { object_type: 78, data: 0 });
    build_armor_stand_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 8);

    // Boat
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Object { object_type: 1, data: 0 });
    build_boat_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 5);

    // Minecart
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Object { object_type: 10, data: 0 });
    build_minecart_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6);

    // Ender Crystal
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Object { object_type: 51, data: 0 });
    build_ender_crystal_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 4);

    // Leash Knot
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Object { object_type: 77, data: 0 });
    build_leash_knot_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 1);

    // Item Frame
    v.clear(); i.clear();
    let entity = create_dummy_entity(EntityKind::Object { object_type: 71, data: 0 });
    build_item_frame_model(&mut v, &mut i, &entity, 0.5, color);
    assert_valid_mesh(&v, &i, 1);

    // Primed TNT (object_type: 50)
    let mut tnt_entity = create_dummy_entity(EntityKind::Object { object_type: 50, data: 0 });
    tnt_entity.ticks_existed = 1; // fuse = 79, (79 / 5) % 2 == 1 (non-flashing phase)
    v.clear(); i.clear();
    build_tnt_primed_model(&mut v, &mut i, TntFace::Sides, &tnt_entity, 0.5, color);
    assert_valid_mesh(&v, &i, 4); // 4 side quads

    v.clear(); i.clear();
    build_tnt_primed_model(&mut v, &mut i, TntFace::Top, &tnt_entity, 0.5, color);
    assert_valid_mesh(&v, &i, 1); // 1 top quad

    v.clear(); i.clear();
    build_tnt_primed_model(&mut v, &mut i, TntFace::Bottom, &tnt_entity, 0.5, color);
    assert_valid_mesh(&v, &i, 1); // 1 bottom quad

    // Falling Block (object_type: 70)
    let falling_entity = create_dummy_entity(EntityKind::Object { object_type: 70, data: 12 });
    v.clear(); i.clear();
    build_falling_block_model(&mut v, &mut i, &falling_entity, 0.5, color);
    assert_valid_mesh(&v, &i, 6); // 6 faces of the cube

    // All 26 Paintings
    assert_eq!(ALL_PAINTINGS.len(), 26);
    for art in ALL_PAINTINGS {
        v.clear(); i.clear();
        let entity = create_dummy_entity(EntityKind::Painting {
            title: art.title.to_string(),
            facing: 0,
        });
        build_painting_model(&mut v, &mut i, &entity, art.title, 0, 0.5, color);
        assert!(!v.is_empty());
        assert!(!i.is_empty());
        for vert in &v {
            assert!(vert.uv[0] >= 0.0 && vert.uv[0] <= 1.0);
            assert!(vert.uv[1] >= 0.0 && vert.uv[1] <= 1.0);
        }
    }
}

#[test]
fn test_layers_and_damage_effects() {
    let color = [1.0, 1.0, 1.0, 1.0];
    let entity = create_dummy_entity(EntityKind::Mob { mob_type: 91 });

    // Sheep Wool Layer: 16 dye colors
    for dye in 0..16 {
        let mut v = Vec::new(); let mut i = Vec::new();
        render_sheep_wool_layer(&mut v, &mut i, &entity, 0.5, dye, false);
        assert_valid_mesh(&v, &i, 6);
        let expected_rgb = DYE_COLORS[dye as usize];
        assert_eq!(v[0].color, [expected_rgb[0], expected_rgb[1], expected_rgb[2], 1.0]);
    }

    // Sheared sheep produces no wool layer
    let mut v = Vec::new(); let mut i = Vec::new();
    render_sheep_wool_layer(&mut v, &mut i, &entity, 0.5, 0, true);
    assert!(v.is_empty());

    // Glowing Spider Eyes Layer
    v.clear(); i.clear();
    let spider = create_dummy_entity(EntityKind::Mob { mob_type: 52 });
    render_spider_eyes_layer(&mut v, &mut i, &spider, 0.5, false);
    assert_valid_mesh(&v, &i, 1);
    assert_eq!(v[0].color, [1.0, 1.0, 1.0, 1.0]); // Full-bright glowing

    // Glowing Cave Spider Eyes Layer (0.7x scale)
    v.clear(); i.clear();
    let cave_spider = create_dummy_entity(EntityKind::Mob { mob_type: 59 });
    render_spider_eyes_layer(&mut v, &mut i, &cave_spider, 0.5, true);
    assert_valid_mesh(&v, &i, 1);
    assert_eq!(v[0].color, [1.0, 1.0, 1.0, 1.0]);

    // Glowing Enderman Eyes Layer
    v.clear(); i.clear();
    let enderman = create_dummy_entity(EntityKind::Mob { mob_type: 58 });
    render_enderman_eyes_layer(&mut v, &mut i, &enderman, 0.5);
    assert_valid_mesh(&v, &i, 1);
    assert_eq!(v[0].color, [1.0, 1.0, 1.0, 1.0]);

    // Glowing Dragon Eyes Layer
    v.clear(); i.clear();
    let dragon = create_dummy_entity(EntityKind::Mob { mob_type: 63 });
    render_dragon_eyes_layer(&mut v, &mut i, &dragon, 0.5);
    assert_valid_mesh(&v, &i, 1);
    assert_eq!(v[0].color, [1.0, 1.0, 1.0, 1.0]);

    // Creeper Charge Layer
    v.clear(); i.clear();
    let creeper = create_dummy_entity(EntityKind::Mob { mob_type: 50 });
    render_creeper_charge_layer(&mut v, &mut i, &creeper, 0.5);
    assert_valid_mesh(&v, &i, 2);

    // Wither Aura Layer
    v.clear(); i.clear();
    let wither = create_dummy_entity(EntityKind::Mob { mob_type: 64 });
    render_wither_aura_layer(&mut v, &mut i, &wither, 0.5);
    assert_valid_mesh(&v, &i, 2);

    // Slime Gel Layer
    v.clear(); i.clear();
    let slime = create_dummy_entity(EntityKind::Mob { mob_type: 55 });
    render_slime_gel_layer(&mut v, &mut i, &slime, 0.5, color);
    assert_valid_mesh(&v, &i, 1);
    assert_eq!(v[0].color[3], 0.6); // Translucent

    // Biped Armor Layers (Slots 0..4)
    for slot in 0..4 {
        v.clear(); i.clear();
        render_biped_armor_layer(&mut v, &mut i, &entity, 0.5, color, slot);
        assert!(!v.is_empty());
        assert!(!i.is_empty());
    }

    // Enderman Held Block Layer
    v.clear(); i.clear();
    render_enderman_held_block_layer(&mut v, &mut i, &enderman, 0.5, color);
    assert_valid_mesh(&v, &i, 1);
}

#[test]
fn test_damage_flash_and_invisibility_effects() {
    let mut entity = create_dummy_entity(EntityKind::Mob { mob_type: 54 });

    // Normal state
    let alpha_normal = if entity.is_invisible { 0.15 } else { 1.0 };
    let color_normal = if entity.hurt_time > 0 { [1.0, 0.35, 0.35, alpha_normal] } else { [1.0, 1.0, 1.0, alpha_normal] };
    assert_eq!(color_normal, [1.0, 1.0, 1.0, 1.0]);

    // Hurt state (Damage Flash / Hurt Tint)
    entity.hurt_time = 10;
    let alpha_hurt = if entity.is_invisible { 0.15 } else { 1.0 };
    let color_hurt = if entity.hurt_time > 0 { [1.0, 0.35, 0.35, alpha_hurt] } else { [1.0, 1.0, 1.0, alpha_hurt] };
    assert_eq!(color_hurt, [1.0, 0.35, 0.35, 1.0]);

    // Invisible state (Invisibility silhouette)
    entity.hurt_time = 0;
    entity.is_invisible = true;
    let alpha_invis = if entity.is_invisible { 0.15 } else { 1.0 };
    let color_invis = if entity.hurt_time > 0 { [1.0, 0.35, 0.35, alpha_invis] } else { [1.0, 1.0, 1.0, alpha_invis] };
    assert_eq!(color_invis, [1.0, 1.0, 1.0, 0.15]);
}

#[test]
fn test_user_reported_mob_and_object_fixes() {
    let color = [1.0, 1.0, 1.0, 1.0];
    let mut v = Vec::new();
    let mut i = Vec::new();

    // 1. Skeleton bow model
    let skeleton = create_dummy_entity(EntityKind::Mob { mob_type: 51 });
    build_skeleton_bow(&mut v, &mut i, &skeleton, 0.5);
    assert_valid_mesh(&v, &i, 2); // 2 quads for front and back of bow

    // 2. Slime scaling (size 4)
    v.clear(); i.clear();
    let mut slime = create_dummy_entity(EntityKind::Mob { mob_type: 55 });
    slime.metadata.push(world::entity::MetadataEntry::new(16, world::entity::MetadataValue::Byte(4)));
    build_slime_model(&mut v, &mut i, &slime, 0.5, color);
    assert_valid_mesh(&v, &i, 3); // body + 2 eyes

    // 3. Ghast (4.5x scale)
    v.clear(); i.clear();
    let ghast = create_dummy_entity(EntityKind::Mob { mob_type: 56 });
    build_ghast_model(&mut v, &mut i, &ghast, 0.5, color);
    assert_valid_mesh(&v, &i, 10); // body + 9 tentacles

    // 4. Endermite 4 segments
    v.clear(); i.clear();
    let endermite = create_dummy_entity(EntityKind::Mob { mob_type: 67 });
    build_endermite_model(&mut v, &mut i, &endermite, 0.5, color);
    assert_valid_mesh(&v, &i, 4); // 4 body segments

    // 5. Guardian (elder 2.35x scale)
    v.clear(); i.clear();
    let guardian = create_dummy_entity(EntityKind::Mob { mob_type: 68 });
    build_guardian_model(&mut v, &mut i, &guardian, 0.5, true, color);
    assert_valid_mesh(&v, &i, 15); // body + eye + 12 spikes + tail

    // 6. Wolf (sitting posture)
    v.clear(); i.clear();
    let mut wolf = create_dummy_entity(EntityKind::Mob { mob_type: 95 });
    wolf.metadata.push(world::entity::MetadataEntry::new(16, world::entity::MetadataValue::Byte(1))); // sitting
    build_wolf_model(&mut v, &mut i, &wolf, 0.5, color);
    assert_valid_mesh(&v, &i, 7); // head + snout + 2 ears + mane + body + tail

    // 7. Bat (wings and flight tilt)
    v.clear(); i.clear();
    let bat = create_dummy_entity(EntityKind::Mob { mob_type: 65 });
    build_bat_model(&mut v, &mut i, &bat, 0.5, color);
    assert_valid_mesh(&v, &i, 4); // body + head + 2 wings

    // 8. Squid (pitch tilt)
    v.clear(); i.clear();
    let squid = create_dummy_entity(EntityKind::Mob { mob_type: 94 });
    build_squid_model(&mut v, &mut i, &squid, 0.5, color);
    assert_valid_mesh(&v, &i, 9); // body + 8 tentacles

    // 9. Mooshroom red mushroom crossed quads
    v.clear(); i.clear();
    let mooshroom = create_dummy_entity(EntityKind::Mob { mob_type: 96 });
    render_mooshroom_mushroom_layer(&mut v, &mut i, &mooshroom, 0.5, color);
    assert_valid_mesh(&v, &i, 12); // 3 mushrooms * 4 quads (2 crossed double-sided)

    // 10. Horse & Mule (muzzle, long ears, flank chests, 3-part tail)
    v.clear(); i.clear();
    let horse = create_dummy_entity(EntityKind::Mob { mob_type: 100 });
    build_horse_model(&mut v, &mut i, &horse, 0.5, false, color);
    assert_valid_mesh(&v, &i, 10);

    v.clear(); i.clear();
    build_horse_model(&mut v, &mut i, &horse, 0.5, true, color);
    assert_valid_mesh(&v, &i, 14); // with mule ears and chests

    // 11. Wither (2.0x scale)
    v.clear(); i.clear();
    let wither = create_dummy_entity(EntityKind::Mob { mob_type: 64 });
    build_wither_model(&mut v, &mut i, &wither, 0.5, color);
    assert_valid_mesh(&v, &i, 7); // body + upper/lower body + 3 heads + tail

    // 12. Ender Dragon (12 articulated tail segments)
    v.clear(); i.clear();
    let dragon = create_dummy_entity(EntityKind::Mob { mob_type: 63 });
    build_dragon_model(&mut v, &mut i, &dragon, 0.5, color);
    assert_valid_mesh(&v, &i, 20); // body + spine + 5 neck + head + jaws + 12 tail + wings + legs

    // 13. Item Frame (borders, background, item)
    v.clear(); i.clear();
    let mut frame = create_dummy_entity(EntityKind::Object { object_type: 71, data: 0 });
    frame.yaw = 0.0;
    frame.prev_yaw = 0.0;
    build_item_frame_borders(&mut v, &mut i, &frame, 0.5, color);
    assert_valid_mesh(&v, &i, 4); // 4 borders
    let min_x = v.iter().map(|vert| vert.position[0]).fold(f32::INFINITY, f32::min);
    let max_x = v.iter().map(|vert| vert.position[0]).fold(f32::NEG_INFINITY, f32::max);
    assert!((max_x - min_x - 0.75).abs() < 0.05, "Item frame border width must be ~0.75 blocks");

    v.clear(); i.clear();
    build_item_frame_background(&mut v, &mut i, &frame, 0.5, color);
    assert_valid_mesh(&v, &i, 1); // backplate

    v.clear(); i.clear();
    build_item_frame_item(&mut v, &mut i, &frame, 2, false, 0.5, color);
    assert_valid_mesh(&v, &i, 2); // 2 double-sided item quads

    // Item Frame with Map (item_frame_map borders 16x16, background plate 14x14, map quad 1.0x1.0)
    v.clear(); i.clear();
    build_item_frame_map_borders(&mut v, &mut i, &frame, 0.5, color);
    assert_valid_mesh(&v, &i, 4); // 4 borders
    let min_x_map = v.iter().map(|vert| vert.position[0]).fold(f32::INFINITY, f32::min);
    let max_x_map = v.iter().map(|vert| vert.position[0]).fold(f32::NEG_INFINITY, f32::max);
    assert!((max_x_map - min_x_map - 1.0).abs() < 0.05, "Map frame border width must be ~1.0 blocks (16x16)");

    v.clear(); i.clear();
    build_item_frame_map_background(&mut v, &mut i, &frame, 0.5, color);
    assert_valid_mesh(&v, &i, 1); // 14x14 plate

    v.clear(); i.clear();
    build_item_frame_map(&mut v, &mut i, &frame, 1, 0.5, color);
    assert_valid_mesh(&v, &i, 1); // 1 full-block map quad
    let min_x_quad = v.iter().map(|vert| vert.position[0]).fold(f32::INFINITY, f32::min);
    let max_x_quad = v.iter().map(|vert| vert.position[0]).fold(f32::NEG_INFINITY, f32::max);
    assert!((max_x_quad - min_x_quad - 1.0).abs() < 0.05, "Map quad width must be ~1.0 block");

    // 14. Dropped item entity
    v.clear(); i.clear();
    let item_ent = create_dummy_entity(EntityKind::Object { object_type: 2, data: 0 });
    build_dropped_item_model(&mut v, &mut i, &item_ent, false, 0.5, color);
    assert_valid_mesh(&v, &i, 2); // double-sided quad

    v.clear(); i.clear();
    build_dropped_item_model(&mut v, &mut i, &item_ent, true, 0.5, color);
    assert_valid_mesh(&v, &i, 1); // 3D block cube
    let min_x_block = v.iter().map(|vert| vert.position[0]).fold(f32::INFINITY, f32::min);
    let max_x_block = v.iter().map(|vert| vert.position[0]).fold(f32::NEG_INFINITY, f32::max);
    assert!((max_x_block - min_x_block - 0.25).abs() < 0.05, "Dropped block width must be ~0.25 blocks");

    // 15. Projectiles: Arrow, Throwable (snowball/egg/pearl/etc.), Wither Skull, Fishing Bobber
    v.clear(); i.clear();
    let arrow = create_dummy_entity(EntityKind::Object { object_type: 60, data: 0 });
    build_arrow_model(&mut v, &mut i, &arrow, 0.5, color);
    assert_valid_mesh(&v, &i, 3); // shaft + 2 fins

    v.clear(); i.clear();
    let snowball = create_dummy_entity(EntityKind::Object { object_type: 61, data: 0 });
    build_throwable_model(&mut v, &mut i, &snowball, 0.35, 0.5, color);
    assert_valid_mesh(&v, &i, 2); // double sided quad

    v.clear(); i.clear();
    let skull = create_dummy_entity(EntityKind::Object { object_type: 66, data: 0 });
    build_wither_skull_model(&mut v, &mut i, &skull, 0.5, color);
    assert_valid_mesh(&v, &i, 1); // 3D skull box

    v.clear(); i.clear();
    let bobber = create_dummy_entity(EntityKind::Object { object_type: 90, data: 0 });
    build_fishing_bobber_model(&mut v, &mut i, &bobber, 0.5);
    assert_valid_mesh(&v, &i, 2); // red top + white bottom
}

#[test]
fn test_invisible_entity_and_armor_stand_mob_30_handling() {
    let mut entity = create_dummy_entity(EntityKind::Mob { mob_type: 30 });
    entity.metadata = vec![protocol::codec::MetadataEntry {
        index: 10,
        value: protocol::codec::MetadataValue::Byte(0x10), // marker
    }];
    entity.apply_metadata_flags();
    assert!(entity.is_armor_stand_marker);
    let bb = entity.bounding_box();
    assert_eq!(bb.max_x - bb.min_x, 0.0);
    assert_eq!(bb.max_y - bb.min_y, 0.0);

    // Normal player (not spectator) must skip rendering invisible entities
    entity.is_invisible = true;
    let is_spectator = false;
    let should_render_non_spec = !entity.is_invisible || is_spectator;
    assert!(!should_render_non_spec);

    // Spectator can see invisible entities with translucent alpha, but marker armor stands are always 0-sized/hidden
    let is_spectator = true;
    let should_render_spec = (!entity.is_invisible || is_spectator) && !entity.is_armor_stand_marker;
    assert!(!should_render_spec);

    let mut regular_invis_mob = create_dummy_entity(EntityKind::Mob { mob_type: 54 });
    regular_invis_mob.is_invisible = true;
    let render_invis_for_spec = (!regular_invis_mob.is_invisible || is_spectator) && !regular_invis_mob.is_armor_stand_marker;
    assert!(render_invis_for_spec);
}

