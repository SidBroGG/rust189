use super::{root_entity_matrix, ModelPart};
use crate::entity_renderer::EntityVertex;
use glam::{Mat4, Vec3};
use std::f32::consts::PI;
use world::entity::TrackedEntity;

/// Builds the Armor Stand model (MCP ModelArmorStand).
pub fn build_armor_stand_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);

    // Base plate (0, 32)
    let mut base = ModelPart::new(0.0, 32.0, 64.0, 64.0);
    base.set_rotation_point(0.0, 12.0, 0.0);
    base.add_box(0.0, 32.0, -6.0, 11.0, -6.0, 12.0, 1.0, 12.0, 0.0);
    base.render(root, vertices, indices, color);

    // Left and right legs (8, 0) and (40, 16)
    let mut right_leg = ModelPart::new(8.0, 0.0, 64.0, 64.0);
    right_leg.set_rotation_point(-1.9, 12.0, 0.0);
    right_leg.add_box(8.0, 0.0, -1.0, 0.0, -1.0, 2.0, 11.0, 2.0, 0.0);
    right_leg.render(root, vertices, indices, color);

    let mut left_leg = ModelPart::new(40.0, 16.0, 64.0, 64.0);
    left_leg.mirror = true;
    left_leg.set_rotation_point(1.9, 12.0, 0.0);
    left_leg.add_box(40.0, 16.0, -1.0, 0.0, -1.0, 2.0, 11.0, 2.0, 0.0);
    left_leg.render(root, vertices, indices, color);

    // Waist (0, 48)
    let mut waist = ModelPart::new(0.0, 48.0, 64.0, 64.0);
    waist.add_box(0.0, 48.0, -4.0, 10.0, -1.0, 8.0, 2.0, 2.0, 0.0);
    waist.render(root, vertices, indices, color);

    // Side rods (16, 0) and (48, 16)
    let mut right_side = ModelPart::new(16.0, 0.0, 64.0, 64.0);
    right_side.add_box(16.0, 0.0, -3.0, 3.0, -1.0, 2.0, 7.0, 2.0, 0.0);
    right_side.render(root, vertices, indices, color);

    let mut left_side = ModelPart::new(48.0, 16.0, 64.0, 64.0);
    left_side.add_box(48.0, 16.0, 1.0, 3.0, -1.0, 2.0, 7.0, 2.0, 0.0);
    left_side.render(root, vertices, indices, color);

    // Shoulders (0, 26)
    let mut body = ModelPart::new(0.0, 26.0, 64.0, 64.0);
    body.add_box(0.0, 26.0, -6.0, 0.0, -1.5, 12.0, 3.0, 3.0, 0.0);
    body.render(root, vertices, indices, color);

    // Head post (0, 0)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 64.0);
    head.set_rotation_point(0.0, 0.0, 0.0);
    head.add_box(0.0, 0.0, -1.0, -7.0, -1.0, 2.0, 7.0, 2.0, 0.0);
    head.render(root, vertices, indices, color);

    // Arms (24, 0) and (32, 16)
    let mut right_arm = ModelPart::new(24.0, 0.0, 64.0, 64.0);
    right_arm.set_rotation_point(-5.0, 2.0, 0.0);
    right_arm.add_box(24.0, 0.0, -2.0, -2.0, -1.0, 2.0, 12.0, 2.0, 0.0);
    right_arm.render(root, vertices, indices, color);

    let mut left_arm = ModelPart::new(32.0, 16.0, 64.0, 64.0);
    left_arm.mirror = true;
    left_arm.set_rotation_point(5.0, 2.0, 0.0);
    left_arm.add_box(32.0, 16.0, 0.0, -2.0, -1.0, 2.0, 12.0, 2.0, 0.0);
    left_arm.render(root, vertices, indices, color);
}

/// Builds the Boat model (MCP ModelBoat).
pub fn build_boat_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let yaw = entity.interpolated_yaw(partial_ticks);
    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - yaw).to_radians())
        * Mat4::from_scale(Vec3::new(-1.0, -1.0, 1.0))
        * Mat4::from_translation(Vec3::new(0.0, -0.375, 0.0));

    // Bottom (0, 8)
    let mut bottom = ModelPart::new(0.0, 8.0, 64.0, 32.0);
    bottom.set_rotation_point(0.0, 4.0, 0.0);
    bottom.set_rotation(PI / 2.0, 0.0, 0.0);
    bottom.add_box(0.0, 8.0, -12.0, -8.0, -3.0, 24.0, 16.0, 4.0, 0.0);
    bottom.render(root, vertices, indices, color);

    // Left side (0, 0)
    let mut left = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    left.set_rotation_point(-11.0, 4.0, 0.0);
    left.set_rotation(0.0, PI * 1.5, 0.0);
    left.add_box(0.0, 0.0, -10.0, -7.0, -1.0, 20.0, 6.0, 2.0, 0.0);
    left.render(root, vertices, indices, color);

    // Right side (0, 0)
    let mut right = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    right.set_rotation_point(11.0, 4.0, 0.0);
    right.set_rotation(0.0, PI / 2.0, 0.0);
    right.add_box(0.0, 0.0, -10.0, -7.0, -1.0, 20.0, 6.0, 2.0, 0.0);
    right.render(root, vertices, indices, color);

    // Back side (0, 0)
    let mut back = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    back.set_rotation_point(0.0, 4.0, -9.0);
    back.set_rotation(0.0, PI, 0.0);
    back.add_box(0.0, 0.0, -10.0, -7.0, -1.0, 20.0, 6.0, 2.0, 0.0);
    back.render(root, vertices, indices, color);

    // Front side (0, 0)
    let mut front = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    front.set_rotation_point(0.0, 4.0, 9.0);
    front.add_box(0.0, 0.0, -10.0, -7.0, -1.0, 20.0, 6.0, 2.0, 0.0);
    front.render(root, vertices, indices, color);
}

/// Builds the Minecart model (MCP ModelMinecart).
pub fn build_minecart_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let yaw = entity.interpolated_yaw(partial_ticks);
    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - yaw).to_radians())
        * Mat4::from_scale(Vec3::new(-1.0, -1.0, 1.0))
        * Mat4::from_translation(Vec3::new(0.0, -0.375, 0.0));

    // Bottom outer (0, 10)
    let mut bottom = ModelPart::new(0.0, 10.0, 64.0, 32.0);
    bottom.set_rotation_point(0.0, 4.0, 0.0);
    bottom.set_rotation(PI / 2.0, 0.0, 0.0);
    bottom.add_box(0.0, 10.0, -10.0, -8.0, -1.0, 20.0, 16.0, 2.0, 0.0);
    bottom.render(root, vertices, indices, color);

    // Inner floor (44, 10)
    let mut floor = ModelPart::new(44.0, 10.0, 64.0, 32.0);
    floor.set_rotation_point(0.0, 4.0, 0.0);
    floor.set_rotation(-PI / 2.0, 0.0, 0.0);
    floor.add_box(44.0, 10.0, -9.0, -7.0, -1.0, 18.0, 14.0, 1.0, 0.0);
    floor.render(root, vertices, indices, color);

    // Left side (0, 0)
    let mut left = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    left.set_rotation_point(-9.0, 4.0, 0.0);
    left.set_rotation(0.0, PI * 1.5, 0.0);
    left.add_box(0.0, 0.0, -8.0, -9.0, -1.0, 16.0, 8.0, 2.0, 0.0);
    left.render(root, vertices, indices, color);

    // Right side (0, 0)
    let mut right = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    right.set_rotation_point(9.0, 4.0, 0.0);
    right.set_rotation(0.0, PI / 2.0, 0.0);
    right.add_box(0.0, 0.0, -8.0, -9.0, -1.0, 16.0, 8.0, 2.0, 0.0);
    right.render(root, vertices, indices, color);

    // Back side (0, 0)
    let mut back = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    back.set_rotation_point(0.0, 4.0, -7.0);
    back.set_rotation(0.0, PI, 0.0);
    back.add_box(0.0, 0.0, -8.0, -9.0, -1.0, 16.0, 8.0, 2.0, 0.0);
    back.render(root, vertices, indices, color);

    // Front side (0, 0)
    let mut front = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    front.set_rotation_point(0.0, 4.0, 7.0);
    front.add_box(0.0, 0.0, -8.0, -9.0, -1.0, 16.0, 8.0, 2.0, 0.0);
    front.render(root, vertices, indices, color);
}

/// Builds the Ender Crystal model (MCP ModelEnderCrystal).
pub fn build_ender_crystal_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let age = entity.ticks_existed as f32 + partial_ticks;
    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_scale(Vec3::splat(2.0))
        * Mat4::from_translation(Vec3::new(0.0, -0.5, 0.0));

    // Base obsidian pedestal (0, 16)
    let mut base = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    base.add_box(0.0, 16.0, -6.0, 0.0, -6.0, 12.0, 4.0, 12.0, 0.0);
    base.render(root, vertices, indices, color);

    // Outer rotating glass cube (0, 0)
    let rot_y = (age * 3.0).to_radians();
    let bob = (age * 0.2).sin() * 0.2 + 0.8;
    let glass_matrix = root
        * Mat4::from_rotation_y(rot_y)
        * Mat4::from_translation(Vec3::new(0.0, bob, 0.0))
        * Mat4::from_axis_angle(Vec3::new(0.7071, 0.0, 0.7071), (60.0_f32).to_radians());

    let mut glass = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    glass.add_box(0.0, 0.0, -4.0, -4.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    glass.render(glass_matrix, vertices, indices, color);

    // Inner counter-rotating glass cube (0, 0) scaled 0.875
    let inner_glass_matrix = glass_matrix
        * Mat4::from_scale(Vec3::splat(0.875))
        * Mat4::from_axis_angle(Vec3::new(0.7071, 0.0, 0.7071), (60.0_f32).to_radians())
        * Mat4::from_rotation_y(rot_y);
    glass.render(inner_glass_matrix, vertices, indices, color);

    // Core cube (32, 0) scaled 0.76
    let core_matrix = inner_glass_matrix
        * Mat4::from_scale(Vec3::splat(0.875))
        * Mat4::from_axis_angle(Vec3::new(0.7071, 0.0, 0.7071), (60.0_f32).to_radians())
        * Mat4::from_rotation_y(rot_y);
    let mut core = ModelPart::new(32.0, 0.0, 64.0, 32.0);
    core.add_box(32.0, 0.0, -4.0, -4.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    core.render(core_matrix, vertices, indices, color);
}

/// Builds the Leash Knot model (MCP ModelLeashKnot).
pub fn build_leash_knot_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32));
    let mut knot = ModelPart::new(0.0, 0.0, 32.0, 32.0);
    knot.add_box(0.0, 0.0, -3.0, -6.0, -3.0, 6.0, 8.0, 6.0, 0.0);
    knot.render(root, vertices, indices, color);
}

/// Builds the Item Frame borders model (MCP RenderItemFrame, planks_birch).
pub fn build_item_frame_borders(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let yaw = entity.interpolated_yaw(partial_ticks);
    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - yaw).to_radians())
        * Mat4::from_scale(Vec3::splat(1.0 / 16.0));

    let mut frame = ModelPart::new(0.0, 0.0, 16.0, 16.0);
    // Matching vanilla item_frame.json elements (resting against wall surface at z = 0, sticking forward to z = -1):
    // Top border: 12 wide, 1 high, 1 deep
    frame.add_box(0.0, 0.0, -6.0, 5.0, -1.0, 12.0, 1.0, 1.0, 0.0);
    // Bottom border: 12 wide, 1 high, 1 deep
    frame.add_box(0.0, 0.0, -6.0, -6.0, -1.0, 12.0, 1.0, 1.0, 0.0);
    // Left border: 1 wide, 10 high, 1 deep
    frame.add_box(0.0, 0.0, -6.0, -5.0, -1.0, 1.0, 10.0, 1.0, 0.0);
    // Right border: 1 wide, 10 high, 1 deep
    frame.add_box(0.0, 0.0, 5.0, -5.0, -1.0, 1.0, 10.0, 1.0, 0.0);
    frame.render(root, vertices, indices, color);
}

/// Builds the Item Frame background plate (MCP RenderItemFrame, itemframe_background).
pub fn build_item_frame_background(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let yaw = entity.interpolated_yaw(partial_ticks);
    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - yaw).to_radians())
        * Mat4::from_scale(Vec3::splat(1.0 / 16.0));

    let mut back = ModelPart::new(0.0, 0.0, 16.0, 16.0);
    // Recessed plate: 10 wide, 10 high, 0.5 deep (resting on wall z = 0 to z = -0.5)
    back.add_box(0.0, 0.0, -5.0, -5.0, -0.5, 10.0, 10.0, 0.5, 0.0);
    back.render(root, vertices, indices, color);
}

/// Builds the Item inside the Item Frame (MCP RenderItemFrame).
pub fn build_item_frame_item(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    rotation_steps: u8,
    is_block: bool,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let yaw = entity.interpolated_yaw(partial_ticks);
    let item_rot = (rotation_steps as f32) * 45.0;

    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - yaw).to_radians())
        * Mat4::from_translation(Vec3::new(0.0, 0.0, -0.0625))
        * Mat4::from_rotation_z(item_rot.to_radians());

    if is_block {
        let mut block = ModelPart::new(0.0, 0.0, 16.0, 16.0);
        block.add_box(0.0, 0.0, -2.5, -2.5, -2.5, 5.0, 5.0, 5.0, 0.0);
        block.render(root * Mat4::from_scale(Vec3::splat(1.0 / 16.0)), vertices, indices, color);
    } else {
        let s = 0.225;
        let tz = 0.015;
        let base_idx = vertices.len() as u32;

        let p0 = (root * Vec3::new(-s, -s, -tz).extend(1.0)).truncate();
        let p1 = (root * Vec3::new(s, -s, -tz).extend(1.0)).truncate();
        let p2 = (root * Vec3::new(s, s, -tz).extend(1.0)).truncate();
        let p3 = (root * Vec3::new(-s, s, -tz).extend(1.0)).truncate();

        let norm_f = (root * Vec3::new(0.0, 0.0, -1.0).extend(0.0)).truncate().to_array();
        let norm_b = (root * Vec3::new(0.0, 0.0, 1.0).extend(0.0)).truncate().to_array();

        // Front
        vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0, 1.0], normal: norm_f, color });
        vertices.push(EntityVertex { position: p1.to_array(), uv: [1.0, 1.0], normal: norm_f, color });
        vertices.push(EntityVertex { position: p2.to_array(), uv: [1.0, 0.0], normal: norm_f, color });
        vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0, 0.0], normal: norm_f, color });
        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2, base_idx, base_idx + 2, base_idx + 3]);

        // Back
        let bp0 = (root * Vec3::new(s, -s, tz).extend(1.0)).truncate();
        let bp1 = (root * Vec3::new(-s, -s, tz).extend(1.0)).truncate();
        let bp2 = (root * Vec3::new(-s, s, tz).extend(1.0)).truncate();
        let bp3 = (root * Vec3::new(s, s, tz).extend(1.0)).truncate();

        let b_idx = vertices.len() as u32;
        vertices.push(EntityVertex { position: bp0.to_array(), uv: [1.0, 1.0], normal: norm_b, color });
        vertices.push(EntityVertex { position: bp1.to_array(), uv: [0.0, 1.0], normal: norm_b, color });
        vertices.push(EntityVertex { position: bp2.to_array(), uv: [0.0, 0.0], normal: norm_b, color });
        vertices.push(EntityVertex { position: bp3.to_array(), uv: [1.0, 0.0], normal: norm_b, color });
        indices.extend_from_slice(&[b_idx, b_idx + 1, b_idx + 2, b_idx, b_idx + 2, b_idx + 3]);
    }
}

/// Builds the Item Frame model (MCP RenderItemFrame).
pub fn build_item_frame_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    build_item_frame_borders(vertices, indices, entity, partial_ticks, color);
    build_item_frame_background(vertices, indices, entity, partial_ticks, color);
}

/// Builds the dropped item model (MCP RenderEntityItem).
pub fn build_dropped_item_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    is_block: bool,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let age = entity.ticks_existed as f32 + partial_ticks;
    let bob = ((age / 10.0 + (entity.id as f32 * 0.1)).sin() * 0.1 + 0.1) as f32;
    // MCP RenderEntityItem spin: ((age + pt) / 20.0 + hoverStart) radians
    let spin = age / 20.0 + (entity.id as f32 * 0.1);

    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32 + bob, pos.z as f32))
        * Mat4::from_rotation_y(spin);

    if is_block {
        let mut block = ModelPart::new(0.0, 0.0, 16.0, 16.0);
        block.add_box(0.0, 0.0, -2.0, -2.0, -2.0, 4.0, 4.0, 4.0, 0.0);
        block.render(root * Mat4::from_scale(Vec3::splat(1.0 / 16.0)), vertices, indices, color);
    } else {
        let s = 0.2;
        let tz = 0.015625; // 3D extruded slab thickness (1/16th block)
        let base_idx = vertices.len() as u32;

        let p0 = (root * Vec3::new(-s, -s, -tz).extend(1.0)).truncate();
        let p1 = (root * Vec3::new(s, -s, -tz).extend(1.0)).truncate();
        let p2 = (root * Vec3::new(s, s, -tz).extend(1.0)).truncate();
        let p3 = (root * Vec3::new(-s, s, -tz).extend(1.0)).truncate();

        let norm_f = (root * Vec3::new(0.0, 0.0, -1.0).extend(0.0)).truncate().to_array();
        let norm_b = (root * Vec3::new(0.0, 0.0, 1.0).extend(0.0)).truncate().to_array();

        // Front
        vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0, 1.0], normal: norm_f, color });
        vertices.push(EntityVertex { position: p1.to_array(), uv: [1.0, 1.0], normal: norm_f, color });
        vertices.push(EntityVertex { position: p2.to_array(), uv: [1.0, 0.0], normal: norm_f, color });
        vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0, 0.0], normal: norm_f, color });
        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2, base_idx, base_idx + 2, base_idx + 3]);

        // Back
        let bp0 = (root * Vec3::new(s, -s, tz).extend(1.0)).truncate();
        let bp1 = (root * Vec3::new(-s, -s, tz).extend(1.0)).truncate();
        let bp2 = (root * Vec3::new(-s, s, tz).extend(1.0)).truncate();
        let bp3 = (root * Vec3::new(s, s, tz).extend(1.0)).truncate();

        let b_idx = vertices.len() as u32;
        vertices.push(EntityVertex { position: bp0.to_array(), uv: [1.0, 1.0], normal: norm_b, color });
        vertices.push(EntityVertex { position: bp1.to_array(), uv: [0.0, 1.0], normal: norm_b, color });
        vertices.push(EntityVertex { position: bp2.to_array(), uv: [0.0, 0.0], normal: norm_b, color });
        vertices.push(EntityVertex { position: bp3.to_array(), uv: [1.0, 0.0], normal: norm_b, color });
        indices.extend_from_slice(&[b_idx, b_idx + 1, b_idx + 2, b_idx, b_idx + 2, b_idx + 3]);

        // 3D edge borders (Left, Right, Top, Bottom) for extruded slab feel
        let norm_l = (root * Vec3::new(-1.0, 0.0, 0.0).extend(0.0)).truncate().to_array();
        let l_idx = vertices.len() as u32;
        vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0, 1.0], normal: norm_l, color });
        vertices.push(EntityVertex { position: bp1.to_array(), uv: [0.0, 1.0], normal: norm_l, color });
        vertices.push(EntityVertex { position: bp2.to_array(), uv: [0.0, 0.0], normal: norm_l, color });
        vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0, 0.0], normal: norm_l, color });
        indices.extend_from_slice(&[l_idx, l_idx + 1, l_idx + 2, l_idx, l_idx + 2, l_idx + 3]);

        let norm_r = (root * Vec3::new(1.0, 0.0, 0.0).extend(0.0)).truncate().to_array();
        let r_idx = vertices.len() as u32;
        vertices.push(EntityVertex { position: bp0.to_array(), uv: [1.0, 1.0], normal: norm_r, color });
        vertices.push(EntityVertex { position: p1.to_array(), uv: [1.0, 1.0], normal: norm_r, color });
        vertices.push(EntityVertex { position: p2.to_array(), uv: [1.0, 0.0], normal: norm_r, color });
        vertices.push(EntityVertex { position: bp3.to_array(), uv: [1.0, 0.0], normal: norm_r, color });
        indices.extend_from_slice(&[r_idx, r_idx + 1, r_idx + 2, r_idx, r_idx + 2, r_idx + 3]);

        let norm_t = (root * Vec3::new(0.0, 1.0, 0.0).extend(0.0)).truncate().to_array();
        let t_idx = vertices.len() as u32;
        vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0, 0.0], normal: norm_t, color });
        vertices.push(EntityVertex { position: p2.to_array(), uv: [1.0, 0.0], normal: norm_t, color });
        vertices.push(EntityVertex { position: bp3.to_array(), uv: [1.0, 0.0], normal: norm_t, color });
        vertices.push(EntityVertex { position: bp2.to_array(), uv: [0.0, 0.0], normal: norm_t, color });
        indices.extend_from_slice(&[t_idx, t_idx + 1, t_idx + 2, t_idx, t_idx + 2, t_idx + 3]);

        let norm_bt = (root * Vec3::new(0.0, -1.0, 0.0).extend(0.0)).truncate().to_array();
        let bt_idx = vertices.len() as u32;
        vertices.push(EntityVertex { position: bp1.to_array(), uv: [0.0, 1.0], normal: norm_bt, color });
        vertices.push(EntityVertex { position: bp0.to_array(), uv: [1.0, 1.0], normal: norm_bt, color });
        vertices.push(EntityVertex { position: p1.to_array(), uv: [1.0, 1.0], normal: norm_bt, color });
        vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0, 1.0], normal: norm_bt, color });
        indices.extend_from_slice(&[bt_idx, bt_idx + 1, bt_idx + 2, bt_idx, bt_idx + 2, bt_idx + 3]);
    }
}

/// Builds the 3D Arrow projectile model (MCP RenderArrow).
pub fn build_arrow_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let yaw = entity.interpolated_yaw(partial_ticks);
    let pitch = entity.interpolated_pitch(partial_ticks);

    let f8 = 0.05625f32;
    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((yaw - 90.0).to_radians())
        * Mat4::from_rotation_z(pitch.to_radians())
        * Mat4::from_rotation_x(45.0f32.to_radians())
        * Mat4::from_scale(Vec3::splat(f8))
        * Mat4::from_translation(Vec3::new(-4.0, 0.0, 0.0));

    // Tail fin (double-sided quad at x = -7)
    let tail_norm_f = (root * Vec3::new(1.0, 0.0, 0.0).extend(0.0)).truncate().to_array();
    let tail_norm_b = (root * Vec3::new(-1.0, 0.0, 0.0).extend(0.0)).truncate().to_array();

    let tp0 = (root * Vec3::new(-7.0, -2.0, -2.0).extend(1.0)).truncate();
    let tp1 = (root * Vec3::new(-7.0, -2.0, 2.0).extend(1.0)).truncate();
    let tp2 = (root * Vec3::new(-7.0, 2.0, 2.0).extend(1.0)).truncate();
    let tp3 = (root * Vec3::new(-7.0, 2.0, -2.0).extend(1.0)).truncate();

    let f4 = 0.0f32;
    let f5 = 0.15625f32; // 5/32
    let f6 = 5.0 / 32.0f32;
    let f7 = 10.0 / 32.0f32;

    let t_idx = vertices.len() as u32;
    vertices.push(EntityVertex { position: tp0.to_array(), uv: [f4, f6], normal: tail_norm_f, color });
    vertices.push(EntityVertex { position: tp1.to_array(), uv: [f5, f6], normal: tail_norm_f, color });
    vertices.push(EntityVertex { position: tp2.to_array(), uv: [f5, f7], normal: tail_norm_f, color });
    vertices.push(EntityVertex { position: tp3.to_array(), uv: [f4, f7], normal: tail_norm_f, color });
    indices.extend_from_slice(&[t_idx, t_idx + 1, t_idx + 2, t_idx, t_idx + 2, t_idx + 3]);

    let tb_idx = vertices.len() as u32;
    vertices.push(EntityVertex { position: tp3.to_array(), uv: [f4, f6], normal: tail_norm_b, color });
    vertices.push(EntityVertex { position: tp2.to_array(), uv: [f5, f6], normal: tail_norm_b, color });
    vertices.push(EntityVertex { position: tp1.to_array(), uv: [f5, f7], normal: tail_norm_b, color });
    vertices.push(EntityVertex { position: tp0.to_array(), uv: [f4, f7], normal: tail_norm_b, color });
    indices.extend_from_slice(&[tb_idx, tb_idx + 1, tb_idx + 2, tb_idx, tb_idx + 2, tb_idx + 3]);

    // 4 crossed shaft quads rotated around X
    let f = 0.0f32;
    let f1 = 0.5f32;
    let f2 = 0.0f32;
    let f3 = 5.0 / 32.0f32;

    for j in 0..4 {
        let rot_m = root * Mat4::from_rotation_x((j as f32 * 90.0).to_radians());
        let shaft_norm_f = (rot_m * Vec3::new(0.0, 0.0, 1.0).extend(0.0)).truncate().to_array();
        let shaft_norm_b = (rot_m * Vec3::new(0.0, 0.0, -1.0).extend(0.0)).truncate().to_array();

        let sp0 = (rot_m * Vec3::new(-8.0, -2.0, 0.0).extend(1.0)).truncate();
        let sp1 = (rot_m * Vec3::new(8.0, -2.0, 0.0).extend(1.0)).truncate();
        let sp2 = (rot_m * Vec3::new(8.0, 2.0, 0.0).extend(1.0)).truncate();
        let sp3 = (rot_m * Vec3::new(-8.0, 2.0, 0.0).extend(1.0)).truncate();

        let s_idx = vertices.len() as u32;
        vertices.push(EntityVertex { position: sp0.to_array(), uv: [f, f2], normal: shaft_norm_f, color });
        vertices.push(EntityVertex { position: sp1.to_array(), uv: [f1, f2], normal: shaft_norm_f, color });
        vertices.push(EntityVertex { position: sp2.to_array(), uv: [f1, f3], normal: shaft_norm_f, color });
        vertices.push(EntityVertex { position: sp3.to_array(), uv: [f, f3], normal: shaft_norm_f, color });
        indices.extend_from_slice(&[s_idx, s_idx + 1, s_idx + 2, s_idx, s_idx + 2, s_idx + 3]);

        let sb_idx = vertices.len() as u32;
        vertices.push(EntityVertex { position: sp1.to_array(), uv: [f1, f2], normal: shaft_norm_b, color });
        vertices.push(EntityVertex { position: sp0.to_array(), uv: [f, f2], normal: shaft_norm_b, color });
        vertices.push(EntityVertex { position: sp3.to_array(), uv: [f, f3], normal: shaft_norm_b, color });
        vertices.push(EntityVertex { position: sp2.to_array(), uv: [f1, f3], normal: shaft_norm_b, color });
        indices.extend_from_slice(&[sb_idx, sb_idx + 1, sb_idx + 2, sb_idx, sb_idx + 2, sb_idx + 3]);
    }
}

/// Builds a throwable 2D double-sided quad projectile (MCP RenderSnowball).
pub fn build_throwable_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    size: f32,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let yaw = entity.interpolated_yaw(partial_ticks);
    let pitch = entity.interpolated_pitch(partial_ticks);

    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - yaw).to_radians())
        * Mat4::from_rotation_x(pitch.to_radians());

    let s = size * 0.5;
    let base_idx = vertices.len() as u32;
    let p0 = (root * Vec3::new(-s, -s, 0.0).extend(1.0)).truncate();
    let p1 = (root * Vec3::new(s, -s, 0.0).extend(1.0)).truncate();
    let p2 = (root * Vec3::new(s, s, 0.0).extend(1.0)).truncate();
    let p3 = (root * Vec3::new(-s, s, 0.0).extend(1.0)).truncate();

    let norm_f = (root * Vec3::new(0.0, 0.0, -1.0).extend(0.0)).truncate().to_array();
    let norm_b = (root * Vec3::new(0.0, 0.0, 1.0).extend(0.0)).truncate().to_array();

    // Front
    vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0, 1.0], normal: norm_f, color });
    vertices.push(EntityVertex { position: p1.to_array(), uv: [1.0, 1.0], normal: norm_f, color });
    vertices.push(EntityVertex { position: p2.to_array(), uv: [1.0, 0.0], normal: norm_f, color });
    vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0, 0.0], normal: norm_f, color });
    indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2, base_idx, base_idx + 2, base_idx + 3]);

    // Back
    let b_idx = vertices.len() as u32;
    vertices.push(EntityVertex { position: p1.to_array(), uv: [1.0, 1.0], normal: norm_b, color });
    vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0, 1.0], normal: norm_b, color });
    vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0, 0.0], normal: norm_b, color });
    vertices.push(EntityVertex { position: p2.to_array(), uv: [1.0, 0.0], normal: norm_b, color });
    indices.extend_from_slice(&[b_idx, b_idx + 1, b_idx + 2, b_idx, b_idx + 2, b_idx + 3]);
}

/// Builds the Wither Skull projectile (MCP RenderWitherSkull).
pub fn build_wither_skull_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let yaw = entity.interpolated_yaw(partial_ticks);
    let pitch = entity.interpolated_pitch(partial_ticks);

    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - yaw).to_radians())
        * Mat4::from_rotation_x(pitch.to_radians())
        * Mat4::from_scale(Vec3::splat(1.0 / 16.0 * 0.5));

    let mut head = ModelPart::new(0.0, 0.0, 64.0, 64.0);
    head.add_box(0.0, 0.0, -4.0, -4.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    head.render(root, vertices, indices, color);
}

/// Builds the Fishing Bobber model (MCP RenderFish).
pub fn build_fishing_bobber_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
) {
    build_fishing_bobber_model_oriented(vertices, indices, entity, 0.0, 0.0, partial_ticks);
}

/// Builds the oriented billboard Fishing Bobber model (MCP RenderFish).
pub fn build_fishing_bobber_model_oriented(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    cam_yaw: f32,
    cam_pitch: f32,
    partial_ticks: f32,
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y((180.0 - cam_yaw).to_radians())
        * Mat4::from_rotation_x((-cam_pitch).to_radians())
        * Mat4::from_scale(Vec3::splat(0.5));

    let norm_f = (root * Vec3::new(0.0, 1.0, 0.0).extend(0.0)).truncate().to_array();
    let norm_b = (root * Vec3::new(0.0, -1.0, 0.0).extend(0.0)).truncate().to_array();

    let p0 = (root * Vec3::new(-0.5, -0.5, 0.0).extend(1.0)).truncate();
    let p1 = (root * Vec3::new(0.5, -0.5, 0.0).extend(1.0)).truncate();
    let p2 = (root * Vec3::new(0.5, 0.5, 0.0).extend(1.0)).truncate();
    let p3 = (root * Vec3::new(-0.5, 0.5, 0.0).extend(1.0)).truncate();

    let base_idx = vertices.len() as u32;
    // Front face
    vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0625, 0.1875], normal: norm_f, color: [1.0, 1.0, 1.0, 1.0] });
    vertices.push(EntityVertex { position: p1.to_array(), uv: [0.125, 0.1875], normal: norm_f, color: [1.0, 1.0, 1.0, 1.0] });
    vertices.push(EntityVertex { position: p2.to_array(), uv: [0.125, 0.125], normal: norm_f, color: [1.0, 1.0, 1.0, 1.0] });
    vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0625, 0.125], normal: norm_f, color: [1.0, 1.0, 1.0, 1.0] });
    indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2, base_idx, base_idx + 2, base_idx + 3]);

    // Back face
    let b_idx = vertices.len() as u32;
    vertices.push(EntityVertex { position: p1.to_array(), uv: [0.125, 0.1875], normal: norm_b, color: [1.0, 1.0, 1.0, 1.0] });
    vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0625, 0.1875], normal: norm_b, color: [1.0, 1.0, 1.0, 1.0] });
    vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0625, 0.125], normal: norm_b, color: [1.0, 1.0, 1.0, 1.0] });
    vertices.push(EntityVertex { position: p2.to_array(), uv: [0.125, 0.125], normal: norm_b, color: [1.0, 1.0, 1.0, 1.0] });
    indices.extend_from_slice(&[b_idx, b_idx + 1, b_idx + 2, b_idx, b_idx + 2, b_idx + 3]);
}

/// Painting artwork definitions from MCP EntityPainting.EnumArt.
#[derive(Copy, Clone, Debug)]
pub struct PaintingArt {
    pub title: &'static str,
    pub width: i32,
    pub height: i32,
    pub u: i32,
    pub v: i32,
}

pub const ALL_PAINTINGS: &[PaintingArt] = &[
    PaintingArt { title: "Kebab", width: 16, height: 16, u: 0, v: 0 },
    PaintingArt { title: "Aztec", width: 16, height: 16, u: 16, v: 0 },
    PaintingArt { title: "Alban", width: 16, height: 16, u: 32, v: 0 },
    PaintingArt { title: "Aztec2", width: 16, height: 16, u: 48, v: 0 },
    PaintingArt { title: "Bomb", width: 16, height: 16, u: 64, v: 0 },
    PaintingArt { title: "Plant", width: 16, height: 16, u: 80, v: 0 },
    PaintingArt { title: "Wasteland", width: 16, height: 16, u: 96, v: 0 },
    PaintingArt { title: "Pool", width: 32, height: 16, u: 0, v: 32 },
    PaintingArt { title: "Courbet", width: 32, height: 16, u: 32, v: 32 },
    PaintingArt { title: "Sea", width: 32, height: 16, u: 64, v: 32 },
    PaintingArt { title: "Sunset", width: 32, height: 16, u: 96, v: 32 },
    PaintingArt { title: "Creebet", width: 32, height: 16, u: 128, v: 32 },
    PaintingArt { title: "Wanderer", width: 16, height: 32, u: 0, v: 64 },
    PaintingArt { title: "Graham", width: 16, height: 32, u: 16, v: 64 },
    PaintingArt { title: "Match", width: 32, height: 32, u: 0, v: 128 },
    PaintingArt { title: "Bust", width: 32, height: 32, u: 32, v: 128 },
    PaintingArt { title: "Stage", width: 32, height: 32, u: 64, v: 128 },
    PaintingArt { title: "Void", width: 32, height: 32, u: 96, v: 128 },
    PaintingArt { title: "SkullAndRoses", width: 32, height: 32, u: 128, v: 128 },
    PaintingArt { title: "Wither", width: 32, height: 32, u: 160, v: 128 },
    PaintingArt { title: "Fighters", width: 64, height: 32, u: 0, v: 96 },
    PaintingArt { title: "Pointer", width: 64, height: 64, u: 0, v: 192 },
    PaintingArt { title: "Pigscene", width: 64, height: 64, u: 64, v: 192 },
    PaintingArt { title: "BurningSkull", width: 64, height: 64, u: 128, v: 192 },
    PaintingArt { title: "Skeleton", width: 64, height: 48, u: 192, v: 64 },
    PaintingArt { title: "DonkeyKong", width: 64, height: 48, u: 192, v: 112 },
];

pub fn find_painting(title: &str) -> PaintingArt {
    ALL_PAINTINGS
        .iter()
        .copied()
        .find(|p| p.title.eq_ignore_ascii_case(title))
        .unwrap_or(ALL_PAINTINGS[0])
}

/// Builds all 26 variants of Paintings (MCP RenderPainting).
pub fn build_painting_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    title: &str,
    facing: u8,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let art = find_painting(title);
    let pos = entity.interpolated_pos(partial_ticks);

    // MCP facing: 0 = South (180), 1 = West (90), 2 = North (0), 3 = East (270)
    let yaw: f32 = match facing {
        0 => 180.0,
        1 => 90.0,
        2 => 0.0,
        3 => 270.0,
        _ => 180.0,
    };

    let root = Mat4::from_translation(Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32))
        * Mat4::from_rotation_y(yaw.to_radians())
        * Mat4::from_scale(Vec3::splat(1.0 / 16.0));

    let f_x = -(art.width as f32) / 2.0;
    let f_y = -(art.height as f32) / 2.0;
    let f_z = 0.5;

    // Wood backing and border UV coordinates from paintings_kristoffer_zetterstrand.png
    let u_back_min = 192.0 / 256.0;
    let u_back_max = 208.0 / 256.0;
    let v_back_min = 0.0 / 256.0;
    let v_back_max = 16.0 / 256.0;

    let u_border_min = 192.0 / 256.0;
    let u_border_max = 208.0 / 256.0;
    let v_border_min = 0.0 / 256.0;
    let v_border_max = 1.0 / 256.0;

    let tiles_x = art.width / 16;
    let tiles_y = art.height / 16;

    for tx in 0..tiles_x {
        for ty in 0..tiles_y {
            let x0 = f_x + (tx * 16) as f32;
            let x1 = f_x + ((tx + 1) * 16) as f32;
            let y0 = f_y + (ty * 16) as f32;
            let y1 = f_y + ((ty + 1) * 16) as f32;

            let u0 = (art.u + art.width - tx * 16) as f32 / 256.0;
            let u1 = (art.u + art.width - (tx + 1) * 16) as f32 / 256.0;
            let v0 = (art.v + art.height - ty * 16) as f32 / 256.0;
            let v1 = (art.v + art.height - (ty + 1) * 16) as f32 / 256.0;

            // Front face (Art)
            let base_idx = vertices.len() as u32;
            let p0 = (root * Vec3::new(x1, y0, -f_z).extend(1.0)).truncate();
            let p1 = (root * Vec3::new(x0, y0, -f_z).extend(1.0)).truncate();
            let p2 = (root * Vec3::new(x0, y1, -f_z).extend(1.0)).truncate();
            let p3 = (root * Vec3::new(x1, y1, -f_z).extend(1.0)).truncate();
            let normal = (root * Vec3::new(0.0, 0.0, -1.0).extend(0.0)).truncate().to_array();

            vertices.push(EntityVertex { position: p0.to_array(), uv: [u1, v0], normal, color });
            vertices.push(EntityVertex { position: p1.to_array(), uv: [u0, v0], normal, color });
            vertices.push(EntityVertex { position: p2.to_array(), uv: [u0, v1], normal, color });
            vertices.push(EntityVertex { position: p3.to_array(), uv: [u1, v1], normal, color });
            indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2, base_idx, base_idx + 2, base_idx + 3]);

            // Back face (Wood)
            let back_idx = vertices.len() as u32;
            let bp0 = (root * Vec3::new(x1, y1, f_z).extend(1.0)).truncate();
            let bp1 = (root * Vec3::new(x0, y1, f_z).extend(1.0)).truncate();
            let bp2 = (root * Vec3::new(x0, y0, f_z).extend(1.0)).truncate();
            let bp3 = (root * Vec3::new(x1, y0, f_z).extend(1.0)).truncate();
            let b_normal = (root * Vec3::new(0.0, 0.0, 1.0).extend(0.0)).truncate().to_array();

            vertices.push(EntityVertex { position: bp0.to_array(), uv: [u_back_min, v_back_min], normal: b_normal, color });
            vertices.push(EntityVertex { position: bp1.to_array(), uv: [u_back_max, v_back_min], normal: b_normal, color });
            vertices.push(EntityVertex { position: bp2.to_array(), uv: [u_back_max, v_back_max], normal: b_normal, color });
            vertices.push(EntityVertex { position: bp3.to_array(), uv: [u_back_min, v_back_max], normal: b_normal, color });
            indices.extend_from_slice(&[back_idx, back_idx + 1, back_idx + 2, back_idx, back_idx + 2, back_idx + 3]);

            // Top border
            if ty == tiles_y - 1 {
                let top_idx = vertices.len() as u32;
                let tp0 = (root * Vec3::new(x1, y1, -f_z).extend(1.0)).truncate();
                let tp1 = (root * Vec3::new(x0, y1, -f_z).extend(1.0)).truncate();
                let tp2 = (root * Vec3::new(x0, y1, f_z).extend(1.0)).truncate();
                let tp3 = (root * Vec3::new(x1, y1, f_z).extend(1.0)).truncate();
                let t_norm = (root * Vec3::new(0.0, 1.0, 0.0).extend(0.0)).truncate().to_array();
                vertices.push(EntityVertex { position: tp0.to_array(), uv: [u_border_min, v_border_min], normal: t_norm, color });
                vertices.push(EntityVertex { position: tp1.to_array(), uv: [u_border_max, v_border_min], normal: t_norm, color });
                vertices.push(EntityVertex { position: tp2.to_array(), uv: [u_border_max, v_border_max], normal: t_norm, color });
                vertices.push(EntityVertex { position: tp3.to_array(), uv: [u_border_min, v_border_max], normal: t_norm, color });
                indices.extend_from_slice(&[top_idx, top_idx + 1, top_idx + 2, top_idx, top_idx + 2, top_idx + 3]);
            }

            // Bottom border
            if ty == 0 {
                let bot_idx = vertices.len() as u32;
                let btp0 = (root * Vec3::new(x1, y0, f_z).extend(1.0)).truncate();
                let btp1 = (root * Vec3::new(x0, y0, f_z).extend(1.0)).truncate();
                let btp2 = (root * Vec3::new(x0, y0, -f_z).extend(1.0)).truncate();
                let btp3 = (root * Vec3::new(x1, y0, -f_z).extend(1.0)).truncate();
                let bot_norm = (root * Vec3::new(0.0, -1.0, 0.0).extend(0.0)).truncate().to_array();
                vertices.push(EntityVertex { position: btp0.to_array(), uv: [u_border_min, v_border_min], normal: bot_norm, color });
                vertices.push(EntityVertex { position: btp1.to_array(), uv: [u_border_max, v_border_min], normal: bot_norm, color });
                vertices.push(EntityVertex { position: btp2.to_array(), uv: [u_border_max, v_border_max], normal: bot_norm, color });
                vertices.push(EntityVertex { position: btp3.to_array(), uv: [u_border_min, v_border_max], normal: bot_norm, color });
                indices.extend_from_slice(&[bot_idx, bot_idx + 1, bot_idx + 2, bot_idx, bot_idx + 2, bot_idx + 3]);
            }

            // Left border (MCP RenderPainting x0)
            if tx == 0 {
                let left_idx = vertices.len() as u32;
                let lp0 = (root * Vec3::new(x0, y1, -f_z).extend(1.0)).truncate();
                let lp1 = (root * Vec3::new(x0, y0, -f_z).extend(1.0)).truncate();
                let lp2 = (root * Vec3::new(x0, y0, f_z).extend(1.0)).truncate();
                let lp3 = (root * Vec3::new(x0, y1, f_z).extend(1.0)).truncate();
                let l_norm = (root * Vec3::new(-1.0, 0.0, 0.0).extend(0.0)).truncate().to_array();
                vertices.push(EntityVertex { position: lp0.to_array(), uv: [u_border_min, v_border_min], normal: l_norm, color });
                vertices.push(EntityVertex { position: lp1.to_array(), uv: [u_border_max, v_border_min], normal: l_norm, color });
                vertices.push(EntityVertex { position: lp2.to_array(), uv: [u_border_max, v_border_max], normal: l_norm, color });
                vertices.push(EntityVertex { position: lp3.to_array(), uv: [u_border_min, v_border_max], normal: l_norm, color });
                indices.extend_from_slice(&[left_idx, left_idx + 1, left_idx + 2, left_idx, left_idx + 2, left_idx + 3]);
            }

            // Right border (MCP RenderPainting x1)
            if tx == tiles_x - 1 {
                let right_idx = vertices.len() as u32;
                let rp0 = (root * Vec3::new(x1, y1, f_z).extend(1.0)).truncate();
                let rp1 = (root * Vec3::new(x1, y0, f_z).extend(1.0)).truncate();
                let rp2 = (root * Vec3::new(x1, y0, -f_z).extend(1.0)).truncate();
                let rp3 = (root * Vec3::new(x1, y1, -f_z).extend(1.0)).truncate();
                let r_norm = (root * Vec3::new(1.0, 0.0, 0.0).extend(0.0)).truncate().to_array();
                vertices.push(EntityVertex { position: rp0.to_array(), uv: [u_border_min, v_border_min], normal: r_norm, color });
                vertices.push(EntityVertex { position: rp1.to_array(), uv: [u_border_max, v_border_min], normal: r_norm, color });
                vertices.push(EntityVertex { position: rp2.to_array(), uv: [u_border_max, v_border_max], normal: r_norm, color });
                vertices.push(EntityVertex { position: rp3.to_array(), uv: [u_border_min, v_border_max], normal: r_norm, color });
                indices.extend_from_slice(&[right_idx, right_idx + 1, right_idx + 2, right_idx, right_idx + 2, right_idx + 3]);
            }
        }
    }
}

/// Helper to append a textured quad to vertex and index buffers.
fn add_cube_face(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    positions: [[f32; 3]; 4],
    uvs: [[f32; 2]; 4],
    normal: [f32; 3],
    color: [f32; 4],
) {
    let base = vertices.len() as u32;
    for i in 0..4 {
        vertices.push(EntityVertex {
            position: positions[i],
            uv: uvs[i],
            normal,
            color,
        });
    }
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Face selector for Primed TNT rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TntFace {
    Sides,
    Top,
    Bottom,
}

/// Builds Primed TNT model (MCP RenderTNTPrimed).
pub fn build_tnt_primed_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    face: TntFace,
    entity: &TrackedEntity,
    partial_ticks: f32,
    base_color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let fuse_ticks = (80i32 - entity.ticks_existed as i32).max(0);
    let remaining = (fuse_ticks as f32) - partial_ticks + 1.0;

    let mut scale = 1.0f32;
    if remaining < 10.0 {
        let mut f = (1.0 - remaining / 10.0).clamp(0.0, 1.0);
        f = f * f * f * f;
        scale = 1.0 + f * 0.3;
    }

    let is_flashing = (fuse_ticks / 5) % 2 == 0;
    let color = if is_flashing {
        [2.8, 2.8, 2.8, base_color[3]]
    } else {
        base_color
    };

    let x_c = pos.x;
    let y_c = pos.y + 0.5;
    let z_c = pos.z;
    let h = 0.5 * (scale as f64);

    let x0 = (x_c - h) as f32;
    let x1 = (x_c + h) as f32;
    let y0 = (y_c - h) as f32;
    let y1 = (y_c + h) as f32;
    let z0 = (z_c - h) as f32;
    let z1 = (z_c + h) as f32;

    match face {
        TntFace::Sides => {
            // North face (-Z)
            add_cube_face(
                vertices,
                indices,
                [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]],
                [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                [0.0, 0.0, -1.0],
                color,
            );
            // South face (+Z)
            add_cube_face(
                vertices,
                indices,
                [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]],
                [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                [0.0, 0.0, 1.0],
                color,
            );
            // West face (-X)
            add_cube_face(
                vertices,
                indices,
                [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]],
                [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                [-1.0, 0.0, 0.0],
                color,
            );
            // East face (+X)
            add_cube_face(
                vertices,
                indices,
                [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]],
                [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                [1.0, 0.0, 0.0],
                color,
            );
        }
        TntFace::Top => {
            // Top face (+Y)
            add_cube_face(
                vertices,
                indices,
                [[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]],
                [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                [0.0, 1.0, 0.0],
                color,
            );
        }
        TntFace::Bottom => {
            // Bottom face (-Y)
            add_cube_face(
                vertices,
                indices,
                [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]],
                [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                [0.0, -1.0, 0.0],
                color,
            );
        }
    }
}

/// Builds Falling Block model (MCP RenderFallingBlock).
pub fn build_falling_block_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let x_c = pos.x;
    let y_c = pos.y + 0.5;
    let z_c = pos.z;
    let h = 0.5f64;

    let x0 = (x_c - h) as f32;
    let x1 = (x_c + h) as f32;
    let y0 = (y_c - h) as f32;
    let y1 = (y_c + h) as f32;
    let z0 = (z_c - h) as f32;
    let z1 = (z_c + h) as f32;

    // North face (-Z)
    add_cube_face(
        vertices,
        indices,
        [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        [0.0, 0.0, -1.0],
        color,
    );
    // South face (+Z)
    add_cube_face(
        vertices,
        indices,
        [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        [0.0, 0.0, 1.0],
        color,
    );
    // West face (-X)
    add_cube_face(
        vertices,
        indices,
        [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        [-1.0, 0.0, 0.0],
        color,
    );
    // East face (+X)
    add_cube_face(
        vertices,
        indices,
        [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        [1.0, 0.0, 0.0],
        color,
    );
    // Top face (+Y)
    add_cube_face(
        vertices,
        indices,
        [[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        [0.0, 1.0, 0.0],
        color,
    );
    // Bottom face (-Y)
    add_cube_face(
        vertices,
        indices,
        [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]],
        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        [0.0, -1.0, 0.0],
        color,
    );
}

