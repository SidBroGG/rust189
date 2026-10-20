use egui::{Pos2, Vec2};
use gui::hud::{
    compute_facing_direction, snap_val, HudElementConfig, HudItemInfo, HudLayoutConfig,
    HudPotionInfo, COMPETITIVE_PROFILE_NAME, DEFAULT_PROFILE_NAME, MINIMAL_PROFILE_NAME,
};

#[test]
fn test_hud_layout_toml_serialization_roundtrip() {
    let mut config = HudLayoutConfig::default();
    config.grid_snap_size = 16;
    config.active_profile = COMPETITIVE_PROFILE_NAME.to_string();

    // Modify an element in active profile
    if let Some(keystrokes) = config.get_element_mut("keystrokes") {
        keystrokes.rel_x = 0.05;
        keystrokes.rel_y = 0.40;
        keystrokes.rel_w = 0.15;
        keystrokes.rel_h = 0.20;
        keystrokes.visible = true;
    }

    // Add a custom profile
    config.create_profile("Ranked Duels", Some(COMPETITIVE_PROFILE_NAME));
    if let Some(cps) = config.get_element_mut("fps_ping") {
        cps.rel_x = 0.03;
        cps.rel_y = 0.04;
    }

    // Serialize to TOML
    let toml_str = toml::to_string_pretty(&config)
        .expect("Failed to serialize HudLayoutConfig to TOML");

    assert!(toml_str.contains("active_profile = \"Ranked Duels\""));
    assert!(toml_str.contains("grid_snap_size = 16"));
    assert!(toml_str.contains("Ranked Duels"));
    assert!(toml_str.contains("Competitive PvP"));
    assert!(toml_str.contains("Default"));

    // Deserialize back
    let restored: HudLayoutConfig = toml::from_str(&toml_str)
        .expect("Failed to deserialize HudLayoutConfig from TOML");

    assert_eq!(restored.active_profile, "Ranked Duels");
    assert_eq!(restored.grid_snap_size, 16);
    assert!(restored.profiles.contains_key(DEFAULT_PROFILE_NAME));
    assert!(restored.profiles.contains_key(COMPETITIVE_PROFILE_NAME));
    assert!(restored.profiles.contains_key(MINIMAL_PROFILE_NAME));
    assert!(restored.profiles.contains_key("Ranked Duels"));

    let ks = restored.profiles.get(COMPETITIVE_PROFILE_NAME).unwrap().get("keystrokes").unwrap();
    assert_eq!(ks.rel_x, 0.05);
    assert_eq!(ks.rel_y, 0.40);
    assert_eq!(ks.rel_w, 0.15);
    assert_eq!(ks.rel_h, 0.20);
    assert!(ks.visible);
}

#[test]
fn test_grid_snapping_precision_and_free_movement() {
    // 1. Free movement (step = 0) must preserve original floats exactly
    assert_eq!(snap_val(13.37, 0), 13.37);
    assert_eq!(snap_val(-42.19, 0), -42.19);
    assert_eq!(snap_val(0.0, 0), 0.0);

    // 2. 4px Snap
    assert_eq!(snap_val(0.0, 4), 0.0);
    assert_eq!(snap_val(1.9, 4), 0.0);
    assert_eq!(snap_val(2.0, 4), 4.0);
    assert_eq!(snap_val(3.8, 4), 4.0);
    assert_eq!(snap_val(7.5, 4), 8.0);
    assert_eq!(snap_val(-5.9, 4), -4.0);
    assert_eq!(snap_val(-6.1, 4), -8.0);

    // 3. 8px Snap
    assert_eq!(snap_val(3.9, 8), 0.0);
    assert_eq!(snap_val(4.0, 8), 8.0);
    assert_eq!(snap_val(12.0, 8), 16.0);
    assert_eq!(snap_val(20.5, 8), 24.0);

    // 4. 16px Snap
    assert_eq!(snap_val(7.9, 16), 0.0);
    assert_eq!(snap_val(8.0, 16), 16.0);
    assert_eq!(snap_val(25.0, 16), 32.0);

    // 5. 32px Snap
    assert_eq!(snap_val(15.9, 32), 0.0);
    assert_eq!(snap_val(16.0, 32), 32.0);
    assert_eq!(snap_val(49.0, 32), 64.0);
}

#[test]
fn test_hud_relative_grid_projection_and_screen_rect() {
    let screen = Vec2::new(1920.0, 1080.0);

    // TopLeft relative positioning
    let tl = HudElementConfig::new("test_tl", "TL", 0.1, 0.2, 0.2, 0.1);
    let r_tl = tl.screen_rect(screen);
    assert_eq!(r_tl.width(), 384.0);
    // Aspect ratio locked to base.x / base.y (100.0 / 40.0 = 2.5)
    assert!((r_tl.height() - 384.0 / 2.5).abs() < 1e-4);

    // Center positioning
    let c = HudElementConfig::new("test_c", "C", 0.4, 0.45, 0.2, 0.1);
    let r_c = c.screen_rect(screen);
    assert_eq!(r_c.width(), 384.0);

    // Dragging & Snapping calculation: set_from_screen_pos with 16px grid
    let mut draggable = HudElementConfig::new("drag", "Drag", 0.0, 0.0, 0.2, 0.1);
    draggable.set_from_screen_pos(Pos2::new(51.3, 78.9), screen, 16.0);
    assert_eq!(draggable.rel_x, 48.0 / 1920.0);
    assert_eq!(draggable.rel_y, 80.0 / 1080.0);

    // Resizing & Snapping calculation: set_from_screen_size with 16px grid
    draggable.set_from_screen_size(Vec2::new(205.0, 155.0), screen, 16.0);
    assert_eq!(draggable.rel_w, 208.0 / 1920.0);
    assert!((draggable.rel_h - (208.0 / 2.5) / 1080.0).abs() < 1e-4);
}

#[test]
fn test_facing_direction_cardinal_compass_yaw_angles() {
    // 0 deg: South (+Z)
    assert_eq!(compute_facing_direction(0.0), ("South", "+Z"));
    assert_eq!(compute_facing_direction(15.0), ("South", "+Z"));
    assert_eq!(compute_facing_direction(-15.0), ("South", "+Z"));

    // 45 deg: South-West (+Z/-X)
    assert_eq!(compute_facing_direction(45.0), ("South-West", "+Z/-X"));
    assert_eq!(compute_facing_direction(35.0), ("South-West", "+Z/-X"));
    assert_eq!(compute_facing_direction(65.0), ("South-West", "+Z/-X"));

    // 90 deg: West (-X)
    assert_eq!(compute_facing_direction(90.0), ("West", "-X"));

    // 135 deg: North-West (-Z/-X)
    assert_eq!(compute_facing_direction(135.0), ("North-West", "-Z/-X"));

    // 180 deg: North (-Z)
    assert_eq!(compute_facing_direction(180.0), ("North", "-Z"));

    // 225 deg: North-East (-Z/+X)
    assert_eq!(compute_facing_direction(225.0), ("North-East", "-Z/+X"));

    // 270 deg: East (+X)
    assert_eq!(compute_facing_direction(270.0), ("East", "+X"));

    // 315 deg: South-East (+Z/+X)
    assert_eq!(compute_facing_direction(315.0), ("South-East", "+Z/+X"));

    // Wrap-around beyond 360 deg
    assert_eq!(compute_facing_direction(360.0), ("South", "+Z"));
    assert_eq!(compute_facing_direction(450.0), ("West", "-X"));
    assert_eq!(compute_facing_direction(-90.0), ("East", "+X"));
    assert_eq!(compute_facing_direction(-180.0), ("North", "-Z"));
}

#[test]
fn test_potion_and_durability_formatting() {
    // Potion Duration formatting
    let p_0 = HudPotionInfo::new(1, "Speed", 0, 0, egui::Color32::WHITE);
    assert_eq!(p_0.format_duration(), "00:00");

    let p_30s = HudPotionInfo::new(1, "Speed", 0, 600, egui::Color32::WHITE);
    assert_eq!(p_30s.format_duration(), "00:30");

    let p_1m45s = HudPotionInfo::new(5, "Strength", 1, 2100, egui::Color32::WHITE);
    assert_eq!(p_1m45s.format_duration(), "01:45");

    let p_8m00s = HudPotionInfo::new(12, "Fire Resistance", 0, 9600, egui::Color32::WHITE);
    assert_eq!(p_8m00s.format_duration(), "08:00");

    // Potion Level Roman Numerals
    assert_eq!(p_0.format_level(), "I");
    assert_eq!(p_1m45s.format_level(), "II");
    let p_iii = HudPotionInfo::new(10, "Regeneration", 2, 400, egui::Color32::WHITE);
    assert_eq!(p_iii.format_level(), "III");
    let p_iv = HudPotionInfo::new(3, "Haste", 3, 400, egui::Color32::WHITE);
    assert_eq!(p_iv.format_level(), "IV");
    let p_v = HudPotionInfo::new(11, "Resistance", 4, 400, egui::Color32::WHITE);
    assert_eq!(p_v.format_level(), "V");

    // Item Durability Ratio
    let sword_full = HudItemInfo::new(276, 1, 0, 1561, "Diamond Sword");
    assert!((sword_full.durability_ratio() - 1.0).abs() < 1e-4);

    let sword_half = HudItemInfo::new(276, 1, 780, 1561, "Diamond Sword");
    assert!((sword_half.durability_ratio() - (1561.0 - 780.0) / 1561.0).abs() < 1e-4);

    let sword_broken = HudItemInfo::new(276, 1, 1561, 1561, "Diamond Sword");
    assert_eq!(sword_broken.durability_ratio(), 0.0);

    // Non-damageable item (max_damage = 0)
    let golden_apple = HudItemInfo::new(322, 64, 0, 0, "Golden Apple");
    assert_eq!(golden_apple.durability_ratio(), 1.0);
}

#[test]
fn test_hud_profiles_management_and_fallback() {
    let mut config = HudLayoutConfig::default();

    // Default profile exists
    assert_eq!(config.active_profile, DEFAULT_PROFILE_NAME);
    assert_eq!(config.global_opacity, 0.85);
    assert!(config.active_elements().contains_key("hotbar"));
    assert!(!config.active_elements().contains_key("crosshair"));

    // Switch to Competitive PvP
    config.switch_profile(COMPETITIVE_PROFILE_NAME);
    assert_eq!(config.active_profile, COMPETITIVE_PROFILE_NAME);
    let comp_elements = config.active_elements();
    assert!(comp_elements.get("keystrokes").unwrap().visible);
    assert!(comp_elements.get("armor_status").unwrap().visible);
    assert!(comp_elements.get("fps_ping").unwrap().visible);

    // Create a new custom profile cloned from Competitive
    config.create_profile("Custom Match", Some(COMPETITIVE_PROFILE_NAME));
    assert_eq!(config.active_profile, "Custom Match");
    assert!(config.profiles.contains_key("Custom Match"));

    // Attempting to delete Default profile should fail
    assert!(!config.delete_profile(DEFAULT_PROFILE_NAME));
    assert!(config.profiles.contains_key(DEFAULT_PROFILE_NAME));

    // Deleting currently active profile "Custom Match" should succeed and fall back to Default
    assert!(config.delete_profile("Custom Match"));
    assert!(!config.profiles.contains_key("Custom Match"));
    assert_eq!(config.active_profile, DEFAULT_PROFILE_NAME);
}

#[test]
fn test_drop_item_action_and_localization() {
    use gui::{t, Language, InGameAction, GuiOutput};

    assert_eq!(t(Language::Russian, "ingame.drop_item"), "Выбросить предмет (Q)");
    assert_eq!(t(Language::English, "ingame.drop_item"), "Drop Item (Q)");

    let action = InGameAction::DropItem { drop_all: false };
    assert_eq!(action, InGameAction::DropItem { drop_all: false });

    let output = GuiOutput::DropItem { drop_all: true };
    match output {
        GuiOutput::DropItem { drop_all } => assert!(drop_all),
        _ => panic!("Expected DropItem output"),
    }
}

#[test]
fn test_minecraft_item_asset_mapping_and_texture_loading() {
    use gui::hud::{find_assets_dir, get_item_asset_path, get_or_load_item_texture};

    // Check item asset paths
    assert_eq!(get_item_asset_path(276, 0, "Diamond Sword"), Some("items/diamond_sword.png".to_string()));
    assert_eq!(get_item_asset_path(260, 0, "Apple"), Some("items/apple.png".to_string()));
    assert_eq!(get_item_asset_path(322, 0, "Golden Apple"), Some("items/apple_golden.png".to_string()));
    assert_eq!(get_item_asset_path(368, 0, "Ender Pearl"), Some("items/ender_pearl.png".to_string()));
    assert_eq!(get_item_asset_path(1, 0, "Stone"), Some("blocks/stone.png".to_string()));
    assert_eq!(get_item_asset_path(2, 0, "Grass Block"), Some("blocks/grass_top.png".to_string()));
    assert_eq!(get_item_asset_path(4, 0, "Cobblestone"), Some("blocks/cobblestone.png".to_string()));
    assert_eq!(get_item_asset_path(46, 0, "TNT"), Some("blocks/tnt_side.png".to_string()));
    assert_eq!(get_item_asset_path(2256, 0, "13 Disc"), Some("items/record_13.png".to_string()));

    // Verify assets directory resolution
    let assets_dir = find_assets_dir();
    assert!(assets_dir.exists(), "Assets directory should exist: {:?}", assets_dir);

    let sword_png = assets_dir.join("minecraft/textures/items/diamond_sword.png");
    assert!(sword_png.exists(), "Diamond sword PNG should exist at {:?}", sword_png);

    let stone_png = assets_dir.join("minecraft/textures/blocks/stone.png");
    assert!(stone_png.exists(), "Stone PNG should exist at {:?}", stone_png);

    // Verify egui texture loading and caching
    let ctx = egui::Context::default();
    let tex_id = get_or_load_item_texture(&ctx, 276, 0, 1561, "Diamond Sword");
    assert!(tex_id.is_some(), "Should successfully load diamond sword texture into egui");

    // Second call should return the cached texture
    let tex_id_2 = get_or_load_item_texture(&ctx, 276, 42, 1561, "Diamond Sword");
    assert_eq!(tex_id, tex_id_2, "Damaged tool should share the same cached texture ID");
}

#[test]
fn test_custom_item_name_and_tooltip_theme() {
    let ctx = egui::Context::default();
    gui::theme::setup_minecraft_theme(&ctx);

    // Verify instant tooltip configuration and semi-transparency
    let style = ctx.style();
    assert_eq!(style.interaction.tooltip_delay, 0.0);
    assert!(!style.interaction.show_tooltips_only_when_still);
    assert_eq!(style.spacing.window_margin, egui::Margin::symmetric(6, 4));

    let visuals = &style.visuals;
    assert_eq!(visuals.window_fill.a(), 180);
    assert_eq!(visuals.window_stroke.color.a(), 180);

    // Verify custom name layout job generation
    let custom_name_colored = "§6§lLegendary §bSword";
    let job = gui::theme::build_layout_job_styled(custom_name_colored, 13.5, false, None);
    assert_eq!(job.sections.len(), 2);

    // Verify slot with NBT display.Name extraction
    let mut display_map = std::collections::HashMap::new();
    display_map.insert("Name".to_string(), protocol::NbtTag::String("§cSharp Blade".to_string()));
    let mut root_map = std::collections::HashMap::new();
    root_map.insert("display".to_string(), protocol::NbtTag::Compound(display_map));
    let slot = protocol::Slot::new(276, 1, 0, Some(protocol::NbtTag::Compound(root_map)));

    let extracted = slot
        .nbt
        .as_ref()
        .and_then(|n| n.get_compound("display"))
        .and_then(|d| d.get_string("Name"));
    assert_eq!(extracted, Some("§cSharp Blade"));
}

