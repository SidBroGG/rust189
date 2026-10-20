use super::{root_entity_matrix, ModelPart};
use crate::entity_renderer::EntityVertex;
use glam::{Mat4, Vec3};
use std::f32::consts::PI;
use world::entity::{MetadataValue, TrackedEntity};

/// Builds the Zombie / Zombie Pigman / Biped model (MCP ModelZombie).
pub fn build_zombie_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    // Head (0, 0)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 64.0);
    head.set_rotation_point(0.0, 0.0, 0.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    head.add_box(32.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, 0.5); // headwear
    head.render(root, vertices, indices, color);

    // Body (16, 16)
    let mut body = ModelPart::new(16.0, 16.0, 64.0, 64.0);
    body.set_rotation_point(0.0, 0.0, 0.0);
    body.add_box(16.0, 16.0, -4.0, 0.0, -2.0, 8.0, 12.0, 4.0, 0.0);
    body.render(root, vertices, indices, color);

    // Arms held straight forward (rot_x = -PI / 2) with slight idle bobbing
    let arm_bob = (entity.ticks_existed as f32 * 0.067).sin() * 0.05;
    let arm_rot_x = -PI / 2.0 + arm_bob;

    let mut right_arm = ModelPart::new(40.0, 16.0, 64.0, 64.0);
    right_arm.set_rotation_point(-5.0, 2.0, 0.0);
    right_arm.set_rotation(arm_rot_x, 0.0, 0.0);
    right_arm.add_box(40.0, 16.0, -3.0, -2.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    right_arm.render(root, vertices, indices, color);

    let mut left_arm = ModelPart::new(40.0, 16.0, 64.0, 64.0);
    left_arm.mirror = true;
    left_arm.set_rotation_point(5.0, 2.0, 0.0);
    left_arm.set_rotation(arm_rot_x, 0.0, 0.0);
    left_arm.add_box(40.0, 16.0, -1.0, -2.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    left_arm.render(root, vertices, indices, color);

    // Legs (0, 16)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    let mut right_leg = ModelPart::new(0.0, 16.0, 64.0, 64.0);
    right_leg.set_rotation_point(-1.9, 12.0, 0.0);
    right_leg.set_rotation(leg_rot1, 0.0, 0.0);
    right_leg.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    right_leg.render(root, vertices, indices, color);

    let mut left_leg = ModelPart::new(0.0, 16.0, 64.0, 64.0);
    left_leg.mirror = true;
    left_leg.set_rotation_point(1.9, 12.0, 0.0);
    left_leg.set_rotation(leg_rot2, 0.0, 0.0);
    left_leg.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    left_leg.render(root, vertices, indices, color);
}

/// Builds Giant Zombie model (MCP RenderGiantZombie - scale 6.0).
pub fn build_giant_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let mut root = root_entity_matrix(entity, partial_ticks);
    root *= Mat4::from_scale(Vec3::splat(6.0));
    build_zombie_model(vertices, indices, entity, partial_ticks, color);
}

/// Builds the Skeleton / Wither Skeleton model (MCP ModelSkeleton).
pub fn build_skeleton_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    is_wither: bool,
    color: [f32; 4],
) {
    let mut root = root_entity_matrix(entity, partial_ticks);
    if is_wither {
        root *= Mat4::from_scale(Vec3::splat(1.2)); // Wither skeleton is taller
    }

    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    // Head (0, 0)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 0.0, 0.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (16, 16)
    let mut body = ModelPart::new(16.0, 16.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 0.0, 0.0);
    body.add_box(16.0, 16.0, -4.0, 0.0, -2.0, 8.0, 12.0, 4.0, 0.0);
    body.render(root, vertices, indices, color);

    // Arms: skinny (2x12x2). Aiming bow animation:
    let right_arm_rot_x = -PI / 2.0 + head_pitch;
    let right_arm_rot_y = -0.1 + head_yaw;
    let left_arm_rot_x = -PI / 2.0 + head_pitch;
    let left_arm_rot_y = 0.1 + head_yaw;

    let mut right_arm = ModelPart::new(40.0, 16.0, 64.0, 32.0);
    right_arm.set_rotation_point(-5.0, 2.0, 0.0);
    right_arm.set_rotation(right_arm_rot_x, right_arm_rot_y, 0.0);
    right_arm.add_box(40.0, 16.0, -1.0, -2.0, -1.0, 2.0, 12.0, 2.0, 0.0);
    right_arm.render(root, vertices, indices, color);

    let mut left_arm = ModelPart::new(40.0, 16.0, 64.0, 32.0);
    left_arm.mirror = true;
    left_arm.set_rotation_point(5.0, 2.0, 0.0);
    left_arm.set_rotation(left_arm_rot_x, left_arm_rot_y, 0.0);
    left_arm.add_box(40.0, 16.0, -1.0, -2.0, -1.0, 2.0, 12.0, 2.0, 0.0);
    left_arm.render(root, vertices, indices, color);

    // Legs (0, 16): skinny (2x12x2)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    let mut right_leg = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    right_leg.set_rotation_point(-2.0, 12.0, 0.0);
    right_leg.set_rotation(leg_rot1, 0.0, 0.0);
    right_leg.add_box(0.0, 16.0, -1.0, 0.0, -1.0, 2.0, 12.0, 2.0, 0.0);
    right_leg.render(root, vertices, indices, color);

    let mut left_leg = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    left_leg.mirror = true;
    left_leg.set_rotation_point(2.0, 12.0, 0.0);
    left_leg.set_rotation(leg_rot2, 0.0, 0.0);
    left_leg.add_box(0.0, 16.0, -1.0, 0.0, -1.0, 2.0, 12.0, 2.0, 0.0);
    left_leg.render(root, vertices, indices, color);
}

/// Builds the Spider / Cave Spider model (MCP ModelSpider).
pub fn build_spider_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    is_cave: bool,
    color: [f32; 4],
) {
    let mut root = root_entity_matrix(entity, partial_ticks);
    if is_cave {
        root *= Mat4::from_scale(Vec3::splat(0.7)); // Cave spider is smaller
    }

    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    // Head (32, 4)
    let mut head = ModelPart::new(32.0, 4.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 15.0, -3.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(32.0, 4.0, -4.0, -4.0, -8.0, 8.0, 8.0, 8.0, 0.0);
    head.render(root, vertices, indices, color);

    // Neck (0, 0)
    let mut neck = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    neck.set_rotation_point(0.0, 15.0, 0.0);
    neck.add_box(0.0, 0.0, -3.0, -3.0, -3.0, 6.0, 6.0, 6.0, 0.0);
    neck.render(root, vertices, indices, color);

    // Abdomen / Body (0, 12)
    let mut body = ModelPart::new(0.0, 12.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 15.0, 9.0);
    body.add_box(0.0, 12.0, -5.0, -4.0, -6.0, 10.0, 8.0, 12.0, 0.0);
    body.render(root, vertices, indices, color);

    // 8 Legs (18, 0)
    let f_pi4 = PI / 4.0;
    let f2 = 0.3926991;

    let f3 = -((limb_swing * 0.6662 * 2.0).cos() * 0.4) * limb_amount;
    let f4 = -((limb_swing * 0.6662 * 2.0 + PI).cos() * 0.4) * limb_amount;
    let f5 = -((limb_swing * 0.6662 * 2.0 + PI * 0.5).cos() * 0.4) * limb_amount;
    let f6 = -((limb_swing * 0.6662 * 2.0 + PI * 1.5).cos() * 0.4) * limb_amount;

    let f7 = ((limb_swing * 0.6662).sin()).abs() * 0.4 * limb_amount;
    let f8 = ((limb_swing * 0.6662 + PI).sin()).abs() * 0.4 * limb_amount;
    let f9 = ((limb_swing * 0.6662 + PI * 0.5).sin()).abs() * 0.4 * limb_amount;
    let f10 = ((limb_swing * 0.6662 + PI * 1.5).sin()).abs() * 0.4 * limb_amount;

    let leg_configs = [
        (-4.0, 2.0, -f_pi4 + f7, f2 * 2.0 + f3, -15.0),
        (4.0, 2.0, f_pi4 - f7, -f2 * 2.0 - f3, -1.0),
        (-4.0, 1.0, -f_pi4 * 0.74 + f8, f2 + f4, -15.0),
        (4.0, 1.0, f_pi4 * 0.74 - f8, -f2 - f4, -1.0),
        (-4.0, 0.0, -f_pi4 * 0.74 + f9, -f2 + f5, -15.0),
        (4.0, 0.0, f_pi4 * 0.74 - f9, f2 - f5, -1.0),
        (-4.0, -1.0, -f_pi4 + f10, -f2 * 2.0 + f6, -15.0),
        (4.0, -1.0, f_pi4 - f10, f2 * 2.0 - f6, -1.0),
    ];

    for (px, pz, rz, ry, box_x) in leg_configs {
        let mut leg = ModelPart::new(18.0, 0.0, 64.0, 32.0);
        leg.set_rotation_point(px, 15.0, pz);
        leg.set_rotation(0.0, ry, rz);
        leg.add_box(18.0, 0.0, box_x, -1.0, -1.0, 16.0, 2.0, 2.0, 0.0);
        leg.render(root, vertices, indices, color);
    }
}

/// Builds the Silverfish / Endermite model (MCP ModelSilverfish).
pub fn build_silverfish_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let age = entity.ticks_existed as f32 + partial_ticks;

    let box_lengths = [
        [3.0, 2.0, 2.0],
        [4.0, 3.0, 2.0],
        [6.0, 4.0, 3.0],
        [3.0, 3.0, 3.0],
        [2.0, 2.0, 3.0],
        [2.0, 1.0, 2.0],
        [1.0, 1.0, 2.0],
    ];
    let tex_pos = [
        (0.0, 0.0), (0.0, 4.0), (0.0, 9.0), (0.0, 16.0),
        (0.0, 22.0), (11.0, 0.0), (13.0, 4.0),
    ];

    let mut z_pos = -3.5f32;
    for i in 0..7 {
        let w = box_lengths[i][0];
        let h = box_lengths[i][1];
        let d = box_lengths[i][2];
        let (tu, tv) = tex_pos[i];

        let rot_y = (age * 0.9 + (i as f32) * 0.15 * PI).cos() * PI * 0.05 * (1.0 + ((i as i32 - 2).abs() as f32));
        let pos_x = (age * 0.9 + (i as f32) * 0.15 * PI).sin() * PI * 0.2 * ((i as i32 - 2).abs() as f32);

        let mut part = ModelPart::new(tu, tv, 64.0, 32.0);
        part.set_rotation_point(pos_x, 24.0 - h, z_pos);
        part.set_rotation(0.0, rot_y, 0.0);
        part.add_box(tu, tv, -w * 0.5, 0.0, -d * 0.5, w, h, d, 0.0);
        part.render(root, vertices, indices, color);

        if i < 6 {
            z_pos += (d + box_lengths[i + 1][2]) * 0.5;
        }
    }
}

/// Builds the Endermite model (MCP ModelEnderMite).
pub fn build_endermite_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let age = entity.ticks_existed as f32 + partial_ticks;

    let dims = [
        (4.0, 3.0, 2.0, 0.0, 0.0),
        (6.0, 4.0, 5.0, 0.0, 5.0),
        (3.0, 3.0, 1.0, 0.0, 14.0),
        (1.0, 2.0, 1.0, 0.0, 18.0),
    ];

    let mut z = -3.5f32;
    for (i, &(w, h, d, tu, tv)) in dims.iter().enumerate() {
        let rot_y = (age * 0.9 + (i as f32) * 0.15 * PI).cos() * PI * 0.1;
        let mut part = ModelPart::new(tu, tv, 64.0, 32.0);
        part.set_rotation_point(0.0, 24.0 - h, z);
        part.set_rotation(0.0, rot_y, 0.0);
        part.add_box(tu, tv, -w * 0.5, 0.0, -d * 0.5, w, h, d, 0.0);
        part.render(root, vertices, indices, color);
        z += d;
    }
}

/// Builds the Creeper model (MCP ModelCreeper).
pub fn build_creeper_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let mut root = root_entity_matrix(entity, partial_ticks);

    // Swelling before explosion (expand scale slightly if swelling)
    for entry in &entity.metadata {
        if entry.index == 16 {
            if let MetadataValue::Byte(b) = entry.value {
                if b > 0 {
                    root *= Mat4::from_scale(Vec3::new(1.15, 1.15, 1.15));
                }
            }
        }
    }

    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    // Head (0, 0)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 6.0, 0.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (16, 16)
    let mut body = ModelPart::new(16.0, 16.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 6.0, 0.0);
    body.add_box(16.0, 16.0, -4.0, 0.0, -2.0, 8.0, 12.0, 4.0, 0.0);
    body.render(root, vertices, indices, color);

    // 4 Legs (0, 16)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    let mut leg1 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg1.set_rotation_point(-2.0, 18.0, 4.0);
    leg1.set_rotation(leg_rot1, 0.0, 0.0);
    leg1.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.0);
    leg1.render(root, vertices, indices, color);

    let mut leg2 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg2.set_rotation_point(2.0, 18.0, 4.0);
    leg2.set_rotation(leg_rot2, 0.0, 0.0);
    leg2.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.0);
    leg2.render(root, vertices, indices, color);

    let mut leg3 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg3.set_rotation_point(-2.0, 18.0, -4.0);
    leg3.set_rotation(leg_rot2, 0.0, 0.0);
    leg3.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.0);
    leg3.render(root, vertices, indices, color);

    let mut leg4 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg4.set_rotation_point(2.0, 18.0, -4.0);
    leg4.set_rotation(leg_rot1, 0.0, 0.0);
    leg4.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.0);
    leg4.render(root, vertices, indices, color);
}

/// Builds the Slime inner core model (MCP ModelSlime).
pub fn build_slime_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);

    // Inner core (0, 16) + Eyes & Mouth (32, 0..8)
    let mut core = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    core.set_rotation_point(0.0, 0.0, 0.0);
    core.add_box(0.0, 16.0, -3.0, 17.0, -3.0, 6.0, 6.0, 6.0, 0.0);
    core.add_box(32.0, 0.0, -3.25, 18.0, -3.5, 2.0, 2.0, 2.0, 0.0);
    core.add_box(32.0, 4.0, 1.25, 18.0, -3.5, 2.0, 2.0, 2.0, 0.0);
    core.add_box(32.0, 8.0, 0.0, 21.0, -3.5, 1.0, 1.0, 1.0, 0.0);
    core.render(root, vertices, indices, color);
}

/// Builds the Magma Cube model with 8 segmented plates (MCP ModelMagmaCube).
pub fn build_magma_cube_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let age = entity.ticks_existed as f32 + partial_ticks;
    let jump_stretch = ((age * 0.4).sin().max(0.0)) * 2.0;

    // 8 segments
    for i in 0..8 {
        let (tu, tv) = if i == 2 {
            (24.0, 10.0)
        } else if i == 3 {
            (24.0, 19.0)
        } else {
            (0.0, i as f32)
        };

        let mut seg = ModelPart::new(tu, tv, 64.0, 32.0);
        let y_spread = -((4.0 - i as f32) * jump_stretch * 0.5);
        seg.set_rotation_point(0.0, y_spread, 0.0);
        seg.add_box(tu, tv, -4.0, 16.0 + (i as f32), -4.0, 8.0, 1.0, 8.0, 0.0);
        seg.render(root, vertices, indices, color);
    }

    // Inner Core (0, 16)
    let mut core = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    core.set_rotation_point(0.0, 0.0, 0.0);
    core.add_box(0.0, 16.0, -2.0, 18.0, -2.0, 4.0, 4.0, 4.0, 0.0);
    core.render(root, vertices, indices, color);
}

/// Builds the Enderman model (MCP ModelEnderman).
pub fn build_enderman_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    // Check if angry / attacking from metadata index 18
    let mut is_attacking = false;
    for entry in &entity.metadata {
        if entry.index == 18 {
            if let MetadataValue::Byte(b) = entry.value {
                is_attacking = b > 0;
            }
        }
    }

    let f = -14.0f32;

    // Head: if attacking, upper head raises 5px opening the mouth!
    let head_y_offset = if is_attacking { -5.0 } else { 0.0 };
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, f + 1.0 + head_y_offset, 0.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    head.add_box(0.0, 16.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, -0.5); // headwear
    head.render(root, vertices, indices, color);

    // Body (32, 16)
    let mut body = ModelPart::new(32.0, 16.0, 64.0, 32.0);
    body.set_rotation_point(0.0, f, 0.0);
    body.add_box(32.0, 16.0, -4.0, 0.0, -2.0, 8.0, 12.0, 4.0, 0.0);
    body.render(root, vertices, indices, color);

    // Elongated thin arms (56, 0) - 2x30x2
    let arm_rot1 = ((limb_swing * 0.6662).cos() * 0.5 * limb_amount).clamp(-0.4, 0.4);
    let arm_rot2 = ((limb_swing * 0.6662 + PI).cos() * 0.5 * limb_amount).clamp(-0.4, 0.4);

    let mut right_arm = ModelPart::new(56.0, 0.0, 64.0, 32.0);
    right_arm.set_rotation_point(-3.0, 2.0 + f, 0.0);
    right_arm.set_rotation(arm_rot1, 0.0, 0.0);
    right_arm.add_box(56.0, 0.0, -1.0, -2.0, -1.0, 2.0, 30.0, 2.0, 0.0);
    right_arm.render(root, vertices, indices, color);

    let mut left_arm = ModelPart::new(56.0, 0.0, 64.0, 32.0);
    left_arm.mirror = true;
    left_arm.set_rotation_point(5.0, 2.0 + f, 0.0);
    left_arm.set_rotation(arm_rot2, 0.0, 0.0);
    left_arm.add_box(56.0, 0.0, -1.0, -2.0, -1.0, 2.0, 30.0, 2.0, 0.0);
    left_arm.render(root, vertices, indices, color);

    // Elongated thin legs (56, 0) - 2x30x2
    let leg_rot1 = ((limb_swing * 0.6662).cos() * 0.5 * limb_amount).clamp(-0.4, 0.4);
    let leg_rot2 = ((limb_swing * 0.6662 + PI).cos() * 0.5 * limb_amount).clamp(-0.4, 0.4);

    let mut right_leg = ModelPart::new(56.0, 0.0, 64.0, 32.0);
    right_leg.set_rotation_point(-2.0, 9.0 + f, 0.0);
    right_leg.set_rotation(leg_rot1, 0.0, 0.0);
    right_leg.add_box(56.0, 0.0, -1.0, 0.0, -1.0, 2.0, 30.0, 2.0, 0.0);
    right_leg.render(root, vertices, indices, color);

    let mut left_leg = ModelPart::new(56.0, 0.0, 64.0, 32.0);
    left_leg.mirror = true;
    left_leg.set_rotation_point(2.0, 9.0 + f, 0.0);
    left_leg.set_rotation(leg_rot2, 0.0, 0.0);
    left_leg.add_box(56.0, 0.0, -1.0, 0.0, -1.0, 2.0, 30.0, 2.0, 0.0);
    left_leg.render(root, vertices, indices, color);
}

/// Builds the Blaze model with 12 rotating rods (MCP ModelBlaze).
pub fn build_blaze_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();
    let age = entity.ticks_existed as f32 + partial_ticks;

    // Head (0, 0)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 0.0, 0.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -4.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    head.render(root, vertices, indices, color);

    // 12 Blaze Rods (0, 16) in 3 orbital tiers
    // Tier 1 (4 rods)
    let mut f = age * PI * -0.1;
    for i in 0..4 {
        let py = -2.0 + (((i * 2) as f32 + age) * 0.25).cos();
        let px = f.cos() * 9.0;
        let pz = f.sin() * 9.0;
        let mut rod = ModelPart::new(0.0, 16.0, 64.0, 32.0);
        rod.set_rotation_point(px, py, pz);
        rod.add_box(0.0, 16.0, 0.0, 0.0, 0.0, 2.0, 8.0, 2.0, 0.0);
        rod.render(root, vertices, indices, color);
        f += 1.0;
    }

    // Tier 2 (4 rods)
    f = PI / 4.0 + age * PI * 0.03;
    for j in 4..8 {
        let py = 2.0 + (((j * 2) as f32 + age) * 0.25).cos();
        let px = f.cos() * 7.0;
        let pz = f.sin() * 7.0;
        let mut rod = ModelPart::new(0.0, 16.0, 64.0, 32.0);
        rod.set_rotation_point(px, py, pz);
        rod.add_box(0.0, 16.0, 0.0, 0.0, 0.0, 2.0, 8.0, 2.0, 0.0);
        rod.render(root, vertices, indices, color);
        f += 1.0;
    }

    // Tier 3 (4 rods)
    f = 0.47123894 + age * PI * -0.05;
    for k in 8..12 {
        let py = 11.0 + (((k as f32) * 1.5 + age) * 0.5).cos();
        let px = f.cos() * 5.0;
        let pz = f.sin() * 5.0;
        let mut rod = ModelPart::new(0.0, 16.0, 64.0, 32.0);
        rod.set_rotation_point(px, py, pz);
        rod.add_box(0.0, 16.0, 0.0, 0.0, 0.0, 2.0, 8.0, 2.0, 0.0);
        rod.render(root, vertices, indices, color);
        f += 1.0;
    }
}

/// Builds the Ghast model with 9 dangling tentacles (MCP ModelGhast).
pub fn build_ghast_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let mut root = root_entity_matrix(entity, partial_ticks);
    root *= Mat4::from_translation(Vec3::new(0.0, 0.6, 0.0));
    let age = entity.ticks_existed as f32 + partial_ticks;

    // Body (0, 0) 16x16x16
    let mut body = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 8.0, 0.0);
    body.add_box(0.0, 0.0, -8.0, -8.0, -8.0, 16.0, 16.0, 16.0, 0.0);
    body.render(root, vertices, indices, color);

    // 9 Tentacles (0, 0)
    for j in 0..9 {
        let px = (((j % 3) as f32 - ((j / 3 % 2) as f32) * 0.5 + 0.25) / 2.0 * 2.0 - 1.0) * 5.0;
        let pz = (((j / 3) as f32) / 2.0 * 2.0 - 1.0) * 5.0;
        let k = 8.0 + ((j * 17) % 7) as f32; // pseudo-random length

        let rot_x = 0.2 * (age * 0.3 + j as f32).sin() + 0.4;
        let mut tentacle = ModelPart::new(0.0, 0.0, 64.0, 32.0);
        tentacle.set_rotation_point(px, 15.0, pz);
        tentacle.set_rotation(rot_x, 0.0, 0.0);
        tentacle.add_box(0.0, 0.0, -1.0, 0.0, -1.0, 2.0, k, 2.0, 0.0);
        tentacle.render(root, vertices, indices, color);
    }
}

/// Builds the Guardian / Elder Guardian model (MCP ModelGuardian).
pub fn build_guardian_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    is_elder: bool,
    color: [f32; 4],
) {
    let mut root = root_entity_matrix(entity, partial_ticks);
    if is_elder {
        root *= Mat4::from_scale(Vec3::splat(2.35)); // Elder Guardian scale
    }

    let age = entity.ticks_existed as f32 + partial_ticks;

    // Body (0, 0)
    let mut body = ModelPart::new(0.0, 0.0, 64.0, 64.0);
    body.set_rotation_point(0.0, 0.0, 0.0);
    body.add_box(0.0, 0.0, -6.0, 10.0, -8.0, 12.0, 12.0, 16.0, 0.0);
    body.add_box(0.0, 28.0, -8.0, 10.0, -6.0, 2.0, 12.0, 12.0, 0.0);
    body.add_box(0.0, 28.0, 6.0, 10.0, -6.0, 2.0, 12.0, 12.0, 0.0);
    body.add_box(16.0, 40.0, -6.0, 8.0, -6.0, 12.0, 2.0, 12.0, 0.0);
    body.add_box(16.0, 40.0, -6.0, 22.0, -6.0, 12.0, 2.0, 12.0, 0.0);
    // Eye pupil (8, 0)
    body.add_box(8.0, 0.0, -1.0, 15.0, -8.25, 2.0, 2.0, 1.0, 0.0);
    body.render(root, vertices, indices, color);

    // 12 Spikes
    let spike_retract = (age * 0.1).sin() * 0.2;
    for i in 0..12 {
        let angle = (i as f32) * PI * 2.0 / 12.0;
        let mut spike = ModelPart::new(0.0, 0.0, 64.0, 64.0);
        spike.set_rotation_point(angle.cos() * (7.0 + spike_retract), 16.0, angle.sin() * (7.0 + spike_retract));
        spike.set_rotation(0.0, angle, 0.0);
        spike.add_box(0.0, 0.0, -1.0, -4.5, -1.0, 2.0, 9.0, 2.0, 0.0);
        spike.render(root, vertices, indices, color);
    }

    // Tail fin flapping
    let tail_flap = (age * 0.3).sin() * 0.3;
    let mut tail = ModelPart::new(40.0, 0.0, 64.0, 64.0);
    tail.set_rotation_point(0.0, 14.0, 7.0);
    tail.set_rotation(0.0, tail_flap, 0.0);
    tail.add_box(40.0, 0.0, -2.0, 0.0, 0.0, 4.0, 4.0, 8.0, 0.0);
    tail.add_box(25.0, 19.0, 0.0, -3.5, 7.0, 1.0, 9.0, 9.0, 0.0); // Fin
    tail.render(root, vertices, indices, color);
}

/// Builds the Witch model (MCP ModelWitch).
pub fn build_witch_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();
    let age = entity.ticks_existed as f32 + partial_ticks;

    // Head (0, 0) + Pointed Witch Hat (0, 64)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 128.0);
    head.set_rotation_point(0.0, 0.0, 0.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -10.0, -4.0, 8.0, 10.0, 8.0, 0.0);

    // Wiggling Nose with Mole
    let mole_wiggle = (age * 0.2).sin() * 4.5 * PI / 180.0;
    let mut nose = ModelPart::new(24.0, 0.0, 64.0, 128.0);
    nose.set_rotation_point(0.0, -2.0, 0.0);
    nose.set_rotation(mole_wiggle, 0.0, 0.0);
    nose.add_box(24.0, 0.0, -1.0, -1.0, -6.0, 2.0, 4.0, 2.0, 0.0);
    nose.add_box(0.0, 0.0, 0.0, 0.0, -6.75, 1.0, 1.0, 1.0, -0.25);
    head.add_child(nose);

    // Hat brim & conical segments
    head.add_box(0.0, 64.0, -5.0, -10.0, -5.0, 10.0, 2.0, 10.0, 0.0);
    head.add_box(0.0, 76.0, -3.25, -14.0, -3.0, 7.0, 4.0, 7.0, 0.0);
    head.add_box(0.0, 87.0, -1.5, -18.0, -1.0, 4.0, 4.0, 4.0, 0.0);
    head.add_box(0.0, 95.0, 0.0, -20.0, 0.5, 1.0, 2.0, 1.0, 0.25);
    head.render(root, vertices, indices, color);

    // Body (16, 20) + Robe (0, 38)
    let mut body = ModelPart::new(16.0, 20.0, 64.0, 128.0);
    body.set_rotation_point(0.0, 0.0, 0.0);
    body.add_box(16.0, 20.0, -4.0, 0.0, -3.0, 8.0, 12.0, 6.0, 0.0);
    body.add_box(0.0, 38.0, -4.0, 0.0, -3.0, 8.0, 18.0, 6.0, 0.5);
    body.render(root, vertices, indices, color);

    // Crossed arms (44, 22)
    let mut arms = ModelPart::new(44.0, 22.0, 64.0, 128.0);
    arms.set_rotation_point(0.0, 3.0, -1.0);
    arms.set_rotation(-0.75, 0.0, 0.0);
    arms.add_box(44.0, 22.0, -8.0, -2.0, -2.0, 4.0, 8.0, 4.0, 0.0);
    arms.add_box(44.0, 22.0, 4.0, -2.0, -2.0, 4.0, 8.0, 4.0, 0.0);
    arms.add_box(40.0, 38.0, -4.0, 2.0, -2.0, 8.0, 4.0, 4.0, 0.0);
    arms.render(root, vertices, indices, color);

    // Legs (0, 22)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount * 0.5;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount * 0.5;

    let mut right_leg = ModelPart::new(0.0, 22.0, 64.0, 128.0);
    right_leg.set_rotation_point(-2.0, 12.0, 0.0);
    right_leg.set_rotation(leg_rot1, 0.0, 0.0);
    right_leg.add_box(0.0, 22.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    right_leg.render(root, vertices, indices, color);

    let mut left_leg = ModelPart::new(0.0, 22.0, 64.0, 128.0);
    left_leg.mirror = true;
    left_leg.set_rotation_point(2.0, 12.0, 0.0);
    left_leg.set_rotation(leg_rot2, 0.0, 0.0);
    left_leg.add_box(0.0, 22.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    left_leg.render(root, vertices, indices, color);
}
