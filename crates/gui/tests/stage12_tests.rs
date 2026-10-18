use egui::{Pos2, Vec2};
use gui::hud::{
    compute_facing_direction, snap_val, HudAnchor, HudElementConfig, HudItemInfo, HudLayoutConfig,
    HudPotionInfo, COMPETITIVE_PROFILE_NAME, DEFAULT_PROFILE_NAME, MINIMAL_PROFILE_NAME,
};

#[test]
fn test_hud_layout_toml_serialization_roundtrip() {
    let mut config = HudLayoutConfig::default();
    config.grid_snap_size = 16;
    config.active_profile = COMPETITIVE_PROFILE_NAME.to_string();

    // Modify an element in active profile
    if let Some(keystrokes) = config.get_element_mut("keystrokes") {
        keystrokes.offset = [24.0, -48.0];
        keystrokes.scale = 1.25;
        keystrokes.visible = true;
    }

    // Add a custom profile
    config.create_profile("Ranked Duels", Some(COMPETITIVE_PROFILE_NAME));
    if let Some(cps) = config.get_element_mut("fps_ping") {
        cps.offset = [10.0, 10.0];
        cps.scale = 1.1;
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
    assert_eq!(ks.offset, [24.0, -48.0]);
    assert_eq!(ks.scale, 1.25);
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
fn test_hud_anchor_projection_and_screen_rect() {
    let screen = Vec2::new(1920.0, 1080.0);
    let content = Vec2::new(100.0, 40.0);

    // TopLeft: base = (0, 0)
    let tl = HudElementConfig::new("test_tl", "TL", HudAnchor::TopLeft, [10.0, 20.0]);
    let r_tl = tl.screen_rect(screen, content);
    assert_eq!(r_tl.min, Pos2::new(10.0, 20.0));
    assert_eq!(r_tl.size(), content);

    // TopCenter: base = (960, 0), center alignment
    let tc = HudElementConfig::new("test_tc", "TC", HudAnchor::TopCenter, [0.0, 15.0]);
    let r_tc = tc.screen_rect(screen, content);
    assert_eq!(r_tc.min, Pos2::new(960.0 - 50.0, 15.0));

    // TopRight: base = (1920, 0), right alignment
    let tr = HudElementConfig::new("test_tr", "TR", HudAnchor::TopRight, [-10.0, 15.0]);
    let r_tr = tr.screen_rect(screen, content);
    assert_eq!(r_tr.min, Pos2::new(1920.0 - 10.0 - 100.0, 15.0));

    // BottomCenter: base = (960, 1080), center & bottom alignment
    let bc = HudElementConfig::new("test_bc", "BC", HudAnchor::BottomCenter, [0.0, -30.0]);
    let r_bc = bc.screen_rect(screen, content);
    assert_eq!(r_bc.min, Pos2::new(960.0 - 50.0, 1080.0 - 30.0 - 40.0));

    // Center: base = (960, 540)
    let c = HudElementConfig::new("test_c", "C", HudAnchor::Center, [0.0, 0.0]);
    let r_c = c.screen_rect(screen, content);
    assert_eq!(r_c.min, Pos2::new(960.0 - 50.0, 540.0 - 20.0));

    // Scaling factor 2.0x
    let mut scaled = tl.clone();
    scaled.scale = 2.0;
    let r_scaled = scaled.screen_rect(screen, content);
    assert_eq!(r_scaled.size(), Vec2::new(200.0, 80.0));

    // Dragging & Snapping calculation: set_from_screen_pos
    let mut draggable = HudElementConfig::new("drag", "Drag", HudAnchor::TopLeft, [0.0, 0.0]);
    draggable.set_from_screen_pos(Pos2::new(51.3, 78.9), screen, content, 8);
    assert_eq!(draggable.offset, [48.0, 80.0]);
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
    assert!(config.active_elements().contains_key("hotbar"));
    assert!(config.active_elements().contains_key("crosshair"));
    assert!(config.active_elements().contains_key("vitals"));

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
