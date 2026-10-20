use super::{root_entity_matrix, ModelPart};
use crate::entity_renderer::EntityVertex;
use std::f32::consts::PI;
use world::entity::TrackedEntity;

/// Builds the Cow model (MCP ModelCow).
pub fn build_cow_model(
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
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 4.0, -8.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -4.0, -6.0, 8.0, 8.0, 6.0, 0.0);
    // Horns (22, 0)
    head.add_box(22.0, 0.0, -5.0, -5.0, -4.0, 1.0, 3.0, 1.0, 0.0);
    head.add_box(22.0, 0.0, 4.0, -5.0, -4.0, 1.0, 3.0, 1.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (18, 4) rotated 90 deg around X
    let mut body = ModelPart::new(18.0, 4.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 5.0, 2.0);
    body.set_rotation(PI / 2.0, 0.0, 0.0);
    body.add_box(18.0, 4.0, -6.0, -10.0, -7.0, 12.0, 18.0, 10.0, 0.0);
    // Udder (52, 0)
    body.add_box(52.0, 0.0, -2.0, 2.0, -8.0, 4.0, 6.0, 1.0, 0.0);
    body.render(root, vertices, indices, color);

    // 4 Legs (0, 16)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    let mut leg1 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg1.set_rotation_point(-4.0, 12.0, 7.0);
    leg1.set_rotation(leg_rot1, 0.0, 0.0);
    leg1.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    leg1.render(root, vertices, indices, color);

    let mut leg2 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg2.set_rotation_point(4.0, 12.0, 7.0);
    leg2.set_rotation(leg_rot2, 0.0, 0.0);
    leg2.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    leg2.render(root, vertices, indices, color);

    let mut leg3 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg3.set_rotation_point(-4.0, 12.0, -6.0);
    leg3.set_rotation(leg_rot2, 0.0, 0.0);
    leg3.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    leg3.render(root, vertices, indices, color);

    let mut leg4 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg4.set_rotation_point(4.0, 12.0, -6.0);
    leg4.set_rotation(leg_rot1, 0.0, 0.0);
    leg4.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    leg4.render(root, vertices, indices, color);
}

/// Builds the Pig model (MCP ModelPig).
pub fn build_pig_model(
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

    // Head (0, 0) + Snout (16, 16)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 12.0, -6.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -4.0, -8.0, 8.0, 8.0, 8.0, 0.0);
    head.add_box(16.0, 16.0, -2.0, 0.0, -9.0, 4.0, 3.0, 1.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (28, 8) rotated 90 deg around X
    let mut body = ModelPart::new(28.0, 8.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 11.0, 2.0);
    body.set_rotation(PI / 2.0, 0.0, 0.0);
    body.add_box(28.0, 8.0, -5.0, -10.0, -7.0, 10.0, 16.0, 8.0, 0.0);
    body.render(root, vertices, indices, color);

    // 4 short legs (0, 16)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    let mut leg1 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg1.set_rotation_point(-3.0, 18.0, 7.0);
    leg1.set_rotation(leg_rot1, 0.0, 0.0);
    leg1.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.0);
    leg1.render(root, vertices, indices, color);

    let mut leg2 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg2.set_rotation_point(3.0, 18.0, 7.0);
    leg2.set_rotation(leg_rot2, 0.0, 0.0);
    leg2.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.0);
    leg2.render(root, vertices, indices, color);

    let mut leg3 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg3.set_rotation_point(-3.0, 18.0, -5.0);
    leg3.set_rotation(leg_rot2, 0.0, 0.0);
    leg3.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.0);
    leg3.render(root, vertices, indices, color);

    let mut leg4 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg4.set_rotation_point(3.0, 18.0, -5.0);
    leg4.set_rotation(leg_rot1, 0.0, 0.0);
    leg4.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.0);
    leg4.render(root, vertices, indices, color);
}

/// Builds the Sheep base body model (MCP ModelSheep2).
pub fn build_sheep_model(
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
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 6.0, -8.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -3.0, -4.0, -6.0, 6.0, 6.0, 8.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (28, 8)
    let mut body = ModelPart::new(28.0, 8.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 5.0, 2.0);
    body.set_rotation(PI / 2.0, 0.0, 0.0);
    body.add_box(28.0, 8.0, -4.0, -10.0, -7.0, 8.0, 16.0, 6.0, 0.0);
    body.render(root, vertices, indices, color);

    // 4 Legs (0, 16)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    let mut leg1 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg1.set_rotation_point(-3.0, 12.0, 7.0);
    leg1.set_rotation(leg_rot1, 0.0, 0.0);
    leg1.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    leg1.render(root, vertices, indices, color);

    let mut leg2 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg2.set_rotation_point(3.0, 12.0, 7.0);
    leg2.set_rotation(leg_rot2, 0.0, 0.0);
    leg2.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    leg2.render(root, vertices, indices, color);

    let mut leg3 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg3.set_rotation_point(-3.0, 12.0, -5.0);
    leg3.set_rotation(leg_rot2, 0.0, 0.0);
    leg3.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    leg3.render(root, vertices, indices, color);

    let mut leg4 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg4.set_rotation_point(3.0, 12.0, -5.0);
    leg4.set_rotation(leg_rot1, 0.0, 0.0);
    leg4.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    leg4.render(root, vertices, indices, color);
}

/// Builds the Chicken model (MCP ModelChicken).
pub fn build_chicken_model(
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

    // Head (0, 0) + Bill (14, 0) + Chin (14, 4)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 15.0, -4.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -2.0, -6.0, -2.0, 4.0, 6.0, 3.0, 0.0);
    head.add_box(14.0, 0.0, -2.0, -4.0, -4.0, 4.0, 2.0, 2.0, 0.0);
    head.add_box(14.0, 4.0, -1.0, -2.0, -3.0, 2.0, 2.0, 2.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (0, 9)
    let mut body = ModelPart::new(0.0, 9.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 16.0, 0.0);
    body.set_rotation(PI / 2.0, 0.0, 0.0);
    body.add_box(0.0, 9.0, -3.0, -4.0, -3.0, 6.0, 8.0, 6.0, 0.0);
    body.render(root, vertices, indices, color);

    // Wings (24, 13) flapping
    let flap = (age * 0.3).cos() * (0.1 + limb_amount * 0.5);
    let mut right_wing = ModelPart::new(24.0, 13.0, 64.0, 32.0);
    right_wing.set_rotation_point(-4.0, 13.0, 0.0);
    right_wing.set_rotation(0.0, 0.0, flap);
    right_wing.add_box(24.0, 13.0, 0.0, 0.0, -3.0, 1.0, 4.0, 6.0, 0.0);
    right_wing.render(root, vertices, indices, color);

    let mut left_wing = ModelPart::new(24.0, 13.0, 64.0, 32.0);
    left_wing.set_rotation_point(4.0, 13.0, 0.0);
    left_wing.set_rotation(0.0, 0.0, -flap);
    left_wing.add_box(24.0, 13.0, -1.0, 0.0, -3.0, 1.0, 4.0, 6.0, 0.0);
    left_wing.render(root, vertices, indices, color);

    // Legs (26, 0)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    let mut right_leg = ModelPart::new(26.0, 0.0, 64.0, 32.0);
    right_leg.set_rotation_point(-2.0, 19.0, 1.0);
    right_leg.set_rotation(leg_rot1, 0.0, 0.0);
    right_leg.add_box(26.0, 0.0, -1.0, 0.0, -3.0, 3.0, 5.0, 3.0, 0.0);
    right_leg.render(root, vertices, indices, color);

    let mut left_leg = ModelPart::new(26.0, 0.0, 64.0, 32.0);
    left_leg.set_rotation_point(1.0, 19.0, 1.0);
    left_leg.set_rotation(leg_rot2, 0.0, 0.0);
    left_leg.add_box(26.0, 0.0, -1.0, 0.0, -3.0, 3.0, 5.0, 3.0, 0.0);
    left_leg.render(root, vertices, indices, color);
}

/// Builds the Rabbit model (MCP ModelRabbit).
pub fn build_rabbit_model(
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

    // Head (32, 0) + Nose (32, 9) + Ears (52, 0)
    let mut head = ModelPart::new(32.0, 0.0, 64.0, 64.0);
    head.set_rotation_point(0.0, 16.0, -1.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(32.0, 0.0, -2.5, -4.0, -5.0, 5.0, 4.0, 5.0, 0.0);
    head.add_box(32.0, 9.0, -0.5, -2.5, -5.5, 1.0, 1.0, 1.0, 0.0);
    // Ears
    head.add_box(52.0, 0.0, -2.5, -9.0, -1.0, 2.0, 5.0, 1.0, 0.0);
    head.add_box(58.0, 0.0, 0.5, -9.0, -1.0, 2.0, 5.0, 1.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (0, 0)
    let mut body = ModelPart::new(0.0, 0.0, 64.0, 64.0);
    body.set_rotation_point(0.0, 19.0, 8.0);
    body.set_rotation(-0.34906584, 0.0, 0.0);
    body.add_box(0.0, 0.0, -3.0, -2.0, -10.0, 6.0, 5.0, 10.0, 0.0);
    body.render(root, vertices, indices, color);

    // Tail (52, 6)
    let mut tail = ModelPart::new(52.0, 6.0, 64.0, 64.0);
    tail.set_rotation_point(0.0, 20.0, 7.0);
    tail.set_rotation(-0.34906584, 0.0, 0.0);
    tail.add_box(52.0, 6.0, -1.5, -1.5, 0.0, 3.0, 3.0, 2.0, 0.0);
    tail.render(root, vertices, indices, color);

    // Feet & Thighs
    let hop = (limb_swing * 0.6662).cos() * 1.2 * limb_amount;

    let mut left_thigh = ModelPart::new(30.0, 15.0, 64.0, 64.0);
    left_thigh.set_rotation_point(3.0, 17.5, 3.7);
    left_thigh.set_rotation(hop, 0.0, 0.0);
    left_thigh.add_box(30.0, 15.0, -1.0, 0.0, 0.0, 2.0, 4.0, 5.0, 0.0);
    left_thigh.add_box(26.0, 24.0, -1.0, 5.5, -3.7, 2.0, 1.0, 7.0, 0.0);
    left_thigh.render(root, vertices, indices, color);

    let mut right_thigh = ModelPart::new(16.0, 15.0, 64.0, 64.0);
    right_thigh.set_rotation_point(-3.0, 17.5, 3.7);
    right_thigh.set_rotation(hop, 0.0, 0.0);
    right_thigh.add_box(16.0, 15.0, -1.0, 0.0, 0.0, 2.0, 4.0, 5.0, 0.0);
    right_thigh.add_box(8.0, 24.0, -1.0, 5.5, -3.7, 2.0, 1.0, 7.0, 0.0);
    right_thigh.render(root, vertices, indices, color);

    // Front legs (0, 15)
    let mut left_arm = ModelPart::new(0.0, 15.0, 64.0, 64.0);
    left_arm.set_rotation_point(3.0, 17.0, -1.0);
    left_arm.set_rotation(-hop * 0.5, 0.0, 0.0);
    left_arm.add_box(0.0, 15.0, -1.0, 0.0, -1.0, 2.0, 7.0, 2.0, 0.0);
    left_arm.render(root, vertices, indices, color);

    let mut right_arm = ModelPart::new(8.0, 15.0, 64.0, 64.0);
    right_arm.set_rotation_point(-3.0, 17.0, -1.0);
    right_arm.set_rotation(-hop * 0.5, 0.0, 0.0);
    right_arm.add_box(8.0, 15.0, -1.0, 0.0, -1.0, 2.0, 7.0, 2.0, 0.0);
    right_arm.render(root, vertices, indices, color);
}

/// Builds the Horse / Donkey / Mule model (MCP ModelHorse).
pub fn build_horse_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    is_mule_or_donkey: bool,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    // Body (0, 34)
    let mut body = ModelPart::new(0.0, 34.0, 128.0, 128.0);
    body.set_rotation_point(0.0, 11.0, 9.0);
    body.add_box(0.0, 34.0, -5.0, -8.0, -19.0, 10.0, 10.0, 22.0, 0.0);
    body.render(root, vertices, indices, color);

    // Neck (0, 12) + Head (0, 0)
    let mut neck = ModelPart::new(0.0, 12.0, 128.0, 128.0);
    neck.set_rotation_point(0.0, 4.0, -10.0);
    neck.set_rotation(0.5235988 + head_pitch * 0.5, head_yaw, 0.0);
    neck.add_box(0.0, 12.0, -2.05, -9.8, -2.0, 4.0, 14.0, 8.0, 0.0);

    // Head on neck
    let mut head = ModelPart::new(0.0, 0.0, 128.0, 128.0);
    head.set_rotation_point(0.0, -10.0, 3.0);
    head.add_box(0.0, 0.0, -2.5, -10.0, -1.5, 5.0, 5.0, 7.0, 0.0);
    head.add_box(0.0, 13.0, -2.0, -10.0, -7.0, 4.0, 5.0, 5.0, 0.0);

    if is_mule_or_donkey {
        // Long mule ears
        head.add_box(0.0, 12.0, -2.5, -16.0, 4.0, 2.0, 7.0, 1.0, 0.0);
        head.add_box(0.0, 12.0, 0.5, -16.0, 4.0, 2.0, 7.0, 1.0, 0.0);
    } else {
        // Regular horse ears
        head.add_box(0.0, 0.0, -2.45, -12.0, 4.0, 2.0, 3.0, 1.0, 0.0);
        head.add_box(0.0, 0.0, 0.45, -12.0, 4.0, 2.0, 3.0, 1.0, 0.0);
    }
    neck.add_child(head);
    neck.render(root, vertices, indices, color);

    // Legs with leg swing
    let leg_rot1 = (limb_swing * 0.6662).cos() * 0.8 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 0.8 * limb_amount;

    // Back legs (78, 29)
    let mut back_left = ModelPart::new(78.0, 29.0, 128.0, 128.0);
    back_left.set_rotation_point(4.0, 9.0, 11.0);
    back_left.set_rotation(leg_rot1, 0.0, 0.0);
    back_left.add_box(78.0, 29.0, -2.5, -2.0, -2.5, 4.0, 15.0, 5.0, 0.0);
    back_left.render(root, vertices, indices, color);

    let mut back_right = ModelPart::new(96.0, 29.0, 128.0, 128.0);
    back_right.set_rotation_point(-4.0, 9.0, 11.0);
    back_right.set_rotation(leg_rot2, 0.0, 0.0);
    back_right.add_box(96.0, 29.0, -1.5, -2.0, -2.5, 4.0, 15.0, 5.0, 0.0);
    back_right.render(root, vertices, indices, color);

    // Front legs (44, 29)
    let mut front_left = ModelPart::new(44.0, 29.0, 128.0, 128.0);
    front_left.set_rotation_point(4.0, 9.0, -8.0);
    front_left.set_rotation(leg_rot2, 0.0, 0.0);
    front_left.add_box(44.0, 29.0, -1.9, -1.0, -2.1, 4.0, 16.0, 4.0, 0.0);
    front_left.render(root, vertices, indices, color);

    let mut front_right = ModelPart::new(60.0, 29.0, 128.0, 128.0);
    front_right.set_rotation_point(-4.0, 9.0, -8.0);
    front_right.set_rotation(leg_rot1, 0.0, 0.0);
    front_right.add_box(60.0, 29.0, -2.1, -1.0, -2.1, 4.0, 16.0, 4.0, 0.0);
    front_right.render(root, vertices, indices, color);

    // Tail (44, 0)
    let mut tail = ModelPart::new(44.0, 0.0, 128.0, 128.0);
    tail.set_rotation_point(0.0, 3.0, 14.0);
    tail.set_rotation(0.5235988, 0.0, 0.0);
    tail.add_box(44.0, 0.0, -1.0, -1.0, 0.0, 2.0, 14.0, 2.0, 0.0);
    tail.render(root, vertices, indices, color);

    // Donkey/Mule Chests (26, 21)
    if is_mule_or_donkey {
        let mut left_chest = ModelPart::new(26.0, 21.0, 128.0, 128.0);
        left_chest.set_rotation_point(6.0, 4.0, 2.0);
        left_chest.add_box(26.0, 21.0, 0.0, -2.0, -4.0, 3.0, 8.0, 8.0, 0.0);
        left_chest.render(root, vertices, indices, color);

        let mut right_chest = ModelPart::new(26.0, 21.0, 128.0, 128.0);
        right_chest.set_rotation_point(-6.0, 4.0, 2.0);
        right_chest.add_box(26.0, 21.0, -3.0, -2.0, -4.0, 3.0, 8.0, 8.0, 0.0);
        right_chest.render(root, vertices, indices, color);
    }
}

/// Builds the Wolf model (MCP ModelWolf).
pub fn build_wolf_model(
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

    // Head (0, 0) + Ears (16, 14) + Snout (0, 10)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(-1.0, 13.5, -7.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -2.0, -3.0, -2.0, 6.0, 6.0, 4.0, 0.0);
    head.add_box(16.0, 14.0, -2.0, -5.0, 0.0, 2.0, 2.0, 1.0, 0.0);
    head.add_box(16.0, 14.0, 2.0, -5.0, 0.0, 2.0, 2.0, 1.0, 0.0);
    head.add_box(0.0, 10.0, -0.5, 0.0, -5.0, 3.0, 3.0, 4.0, 0.0);
    head.render(root, vertices, indices, color);

    // Mane / Collar area (21, 0)
    let mut mane = ModelPart::new(21.0, 0.0, 64.0, 32.0);
    mane.set_rotation_point(-1.0, 14.0, 2.0);
    mane.set_rotation(PI / 2.0, 0.0, 0.0);
    mane.add_box(21.0, 0.0, -3.0, -3.0, -3.0, 8.0, 6.0, 7.0, 0.0);
    mane.render(root, vertices, indices, color);

    // Body (18, 14)
    let mut body = ModelPart::new(18.0, 14.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 14.0, 2.0);
    body.set_rotation(PI / 2.0, 0.0, 0.0);
    body.add_box(18.0, 14.0, -3.0, -2.0, -3.0, 6.0, 9.0, 6.0, 0.0);
    body.render(root, vertices, indices, color);

    // 4 Legs (0, 18)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    let mut leg1 = ModelPart::new(0.0, 18.0, 64.0, 32.0);
    leg1.set_rotation_point(-2.5, 16.0, 7.0);
    leg1.set_rotation(leg_rot1, 0.0, 0.0);
    leg1.add_box(0.0, 18.0, -1.0, 0.0, -1.0, 2.0, 8.0, 2.0, 0.0);
    leg1.render(root, vertices, indices, color);

    let mut leg2 = ModelPart::new(0.0, 18.0, 64.0, 32.0);
    leg2.set_rotation_point(0.5, 16.0, 7.0);
    leg2.set_rotation(leg_rot2, 0.0, 0.0);
    leg2.add_box(0.0, 18.0, -1.0, 0.0, -1.0, 2.0, 8.0, 2.0, 0.0);
    leg2.render(root, vertices, indices, color);

    let mut leg3 = ModelPart::new(0.0, 18.0, 64.0, 32.0);
    leg3.set_rotation_point(-2.5, 16.0, -4.0);
    leg3.set_rotation(leg_rot2, 0.0, 0.0);
    leg3.add_box(0.0, 18.0, -1.0, 0.0, -1.0, 2.0, 8.0, 2.0, 0.0);
    leg3.render(root, vertices, indices, color);

    let mut leg4 = ModelPart::new(0.0, 18.0, 64.0, 32.0);
    leg4.set_rotation_point(0.5, 16.0, -4.0);
    leg4.set_rotation(leg_rot1, 0.0, 0.0);
    leg4.add_box(0.0, 18.0, -1.0, 0.0, -1.0, 2.0, 8.0, 2.0, 0.0);
    leg4.render(root, vertices, indices, color);

    // Tail (9, 18) wagging
    let tail_wag = (limb_swing * 0.6662).cos() * 0.8 * limb_amount;
    let mut tail = ModelPart::new(9.0, 18.0, 64.0, 32.0);
    tail.set_rotation_point(-1.0, 12.0, 8.0);
    tail.set_rotation(0.7853982, tail_wag, 0.0);
    tail.add_box(9.0, 18.0, -1.0, 0.0, -1.0, 2.0, 8.0, 2.0, 0.0);
    tail.render(root, vertices, indices, color);
}

/// Builds the Ocelot / Cat model (MCP ModelOcelot).
pub fn build_ocelot_model(
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

    // Head (0, 0) + Ears (0, 10) + Nose (0, 24)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 15.0, -9.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -2.5, -2.0, -3.0, 5.0, 4.0, 5.0, 0.0);
    head.add_box(0.0, 24.0, -1.5, 0.0, -4.0, 3.0, 2.0, 2.0, 0.0);
    head.add_box(0.0, 10.0, -2.0, -3.0, 0.0, 1.0, 1.0, 2.0, 0.0);
    head.add_box(6.0, 10.0, 1.0, -3.0, 0.0, 1.0, 1.0, 2.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (20, 0)
    let mut body = ModelPart::new(20.0, 0.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 12.0, -10.0);
    body.set_rotation(PI / 2.0, 0.0, 0.0);
    body.add_box(20.0, 0.0, -2.0, 3.0, -8.0, 4.0, 16.0, 6.0, 0.0);
    body.render(root, vertices, indices, color);

    // Tail (0, 15)
    let mut tail = ModelPart::new(0.0, 15.0, 64.0, 32.0);
    tail.set_rotation_point(0.0, 15.0, 8.0);
    tail.set_rotation(0.9, 0.0, 0.0);
    tail.add_box(0.0, 15.0, -0.5, 0.0, 0.0, 1.0, 8.0, 1.0, 0.0);
    tail.render(root, vertices, indices, color);

    // 4 Legs (8, 13)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    let mut back_left = ModelPart::new(8.0, 13.0, 64.0, 32.0);
    back_left.set_rotation_point(1.1, 18.0, 5.0);
    back_left.set_rotation(leg_rot1, 0.0, 0.0);
    back_left.add_box(8.0, 13.0, -1.0, 0.0, 1.0, 2.0, 6.0, 2.0, 0.0);
    back_left.render(root, vertices, indices, color);

    let mut back_right = ModelPart::new(8.0, 13.0, 64.0, 32.0);
    back_right.set_rotation_point(-1.1, 18.0, 5.0);
    back_right.set_rotation(leg_rot2, 0.0, 0.0);
    back_right.add_box(8.0, 13.0, -1.0, 0.0, 1.0, 2.0, 6.0, 2.0, 0.0);
    back_right.render(root, vertices, indices, color);

    let mut front_left = ModelPart::new(40.0, 0.0, 64.0, 32.0);
    front_left.set_rotation_point(1.2, 14.1, -5.0);
    front_left.set_rotation(leg_rot2, 0.0, 0.0);
    front_left.add_box(40.0, 0.0, -1.0, 0.0, 0.0, 2.0, 10.0, 2.0, 0.0);
    front_left.render(root, vertices, indices, color);

    let mut front_right = ModelPart::new(40.0, 0.0, 64.0, 32.0);
    front_right.set_rotation_point(-1.2, 14.1, -5.0);
    front_right.set_rotation(leg_rot1, 0.0, 0.0);
    front_right.add_box(40.0, 0.0, -1.0, 0.0, 0.0, 2.0, 10.0, 2.0, 0.0);
    front_right.render(root, vertices, indices, color);
}

/// Builds the Bat model (MCP ModelBat).
pub fn build_bat_model(
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

    // Head (0, 0) + Ears (24, 0)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 64.0);
    head.set_rotation_point(0.0, 0.0, 0.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -3.0, -3.0, -3.0, 6.0, 6.0, 6.0, 0.0);
    head.add_box(24.0, 0.0, -4.0, -6.0, -2.0, 3.0, 4.0, 1.0, 0.0);
    head.add_box(24.0, 0.0, 1.0, -6.0, -2.0, 3.0, 4.0, 1.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (0, 16)
    let mut body = ModelPart::new(0.0, 16.0, 64.0, 64.0);
    body.set_rotation_point(0.0, 0.0, 0.0);
    body.add_box(0.0, 16.0, -3.0, 4.0, -3.0, 6.0, 12.0, 6.0, 0.0);
    body.add_box(0.0, 34.0, -5.0, 16.0, 0.0, 10.0, 6.0, 1.0, 0.0);
    body.render(root, vertices, indices, color);

    // Flapping Wings (42, 0)
    let wing_flap = (age * 1.3).cos() * PI * 0.25;

    let mut right_wing = ModelPart::new(42.0, 0.0, 64.0, 64.0);
    right_wing.set_rotation_point(0.0, 0.0, 0.0);
    right_wing.set_rotation(0.0, -wing_flap, 0.0);
    right_wing.add_box(42.0, 0.0, -12.0, 1.0, 1.5, 10.0, 16.0, 1.0, 0.0);
    right_wing.render(root, vertices, indices, color);

    let mut left_wing = ModelPart::new(42.0, 0.0, 64.0, 64.0);
    left_wing.set_rotation_point(0.0, 0.0, 0.0);
    left_wing.set_rotation(0.0, wing_flap, 0.0);
    left_wing.add_box(42.0, 0.0, 2.0, 1.0, 1.5, 10.0, 16.0, 1.0, 0.0);
    left_wing.render(root, vertices, indices, color);
}

/// Builds the Squid model (MCP ModelSquid).
pub fn build_squid_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let age = entity.ticks_existed as f32 + partial_ticks;

    // Body (0, 0)
    let mut body = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 8.0, 0.0);
    body.add_box(0.0, 0.0, -6.0, -8.0, -6.0, 12.0, 16.0, 12.0, 0.0);
    body.render(root, vertices, indices, color);

    // 8 Tentacles (48, 0) arranged in circle
    let tentacle_angle = (age * 0.2).sin() * 0.4 + 0.2;
    for i in 0..8 {
        let angle = (i as f32) * PI * 2.0 / 8.0;
        let x = angle.cos() * 5.0;
        let z = angle.sin() * 5.0;

        let mut tentacle = ModelPart::new(48.0, 0.0, 64.0, 32.0);
        tentacle.set_rotation_point(x, 15.0, z);
        tentacle.set_rotation(tentacle_angle, angle + PI / 2.0, 0.0);
        tentacle.add_box(48.0, 0.0, -1.0, 0.0, -1.0, 2.0, 18.0, 2.0, 0.0);
        tentacle.render(root, vertices, indices, color);
    }
}

/// Builds the Villager model (MCP ModelVillager).
pub fn build_villager_model(
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

    // Head (0, 0) + Nose (24, 0)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 64.0);
    head.set_rotation_point(0.0, 0.0, 0.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -10.0, -4.0, 8.0, 10.0, 8.0, 0.0);
    head.add_box(24.0, 0.0, -1.0, -3.0, -6.0, 2.0, 4.0, 2.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (16, 20) + Robe (0, 38)
    let mut body = ModelPart::new(16.0, 20.0, 64.0, 64.0);
    body.set_rotation_point(0.0, 0.0, 0.0);
    body.add_box(16.0, 20.0, -4.0, 0.0, -3.0, 8.0, 12.0, 6.0, 0.0);
    body.add_box(0.0, 38.0, -4.0, 0.0, -3.0, 8.0, 18.0, 6.0, 0.5);
    body.render(root, vertices, indices, color);

    // Crossed arms (44, 22) rotated -0.75 rad
    let mut arms = ModelPart::new(44.0, 22.0, 64.0, 64.0);
    arms.set_rotation_point(0.0, 3.0, -1.0);
    arms.set_rotation(-0.75, 0.0, 0.0);
    arms.add_box(44.0, 22.0, -8.0, -2.0, -2.0, 4.0, 8.0, 4.0, 0.0);
    arms.add_box(44.0, 22.0, 4.0, -2.0, -2.0, 4.0, 8.0, 4.0, 0.0);
    arms.add_box(40.0, 38.0, -4.0, 2.0, -2.0, 8.0, 4.0, 4.0, 0.0);
    arms.render(root, vertices, indices, color);

    // 2 Legs (0, 22)
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount * 0.5;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount * 0.5;

    let mut right_leg = ModelPart::new(0.0, 22.0, 64.0, 64.0);
    right_leg.set_rotation_point(-2.0, 12.0, 0.0);
    right_leg.set_rotation(leg_rot1, 0.0, 0.0);
    right_leg.add_box(0.0, 22.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    right_leg.render(root, vertices, indices, color);

    let mut left_leg = ModelPart::new(0.0, 22.0, 64.0, 64.0);
    left_leg.set_rotation_point(2.0, 12.0, 0.0);
    left_leg.set_rotation(leg_rot2, 0.0, 0.0);
    left_leg.add_box(0.0, 22.0, -2.0, 0.0, -2.0, 4.0, 12.0, 4.0, 0.0);
    left_leg.render(root, vertices, indices, color);
}

/// Builds the Iron Golem model (MCP ModelIronGolem).
pub fn build_iron_golem_model(
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

    // Head (0, 0) + Nose (24, 0)
    let mut head = ModelPart::new(0.0, 0.0, 128.0, 128.0);
    head.set_rotation_point(0.0, -7.0, -2.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -12.0, -5.5, 8.0, 10.0, 8.0, 0.0);
    head.add_box(24.0, 0.0, -1.0, -5.0, -7.5, 2.0, 4.0, 2.0, 0.0);
    head.render(root, vertices, indices, color);

    // Body (0, 40) + Waist (0, 70)
    let mut body = ModelPart::new(0.0, 40.0, 128.0, 128.0);
    body.set_rotation_point(0.0, -7.0, 0.0);
    body.add_box(0.0, 40.0, -9.0, -2.0, -6.0, 18.0, 12.0, 11.0, 0.0);
    body.add_box(0.0, 70.0, -4.5, 10.0, -3.0, 9.0, 5.0, 6.0, 0.5);
    body.render(root, vertices, indices, color);

    // Arms (60, 21) & (60, 58)
    let mut arm_rot1 = (limb_swing * 0.6662 + PI).cos() * 2.0 * limb_amount * 0.5;
    let mut arm_rot2 = (limb_swing * 0.6662).cos() * 2.0 * limb_amount * 0.5;

    // Upward arm swing attack animation
    if entity.swing_progress > 0.0 {
        let swing = entity.swing_progress;
        arm_rot1 = -2.0 + 1.5 * (swing * PI).sin();
        arm_rot2 = -2.0 + 1.5 * (swing * PI).sin();
    }

    let mut right_arm = ModelPart::new(60.0, 21.0, 128.0, 128.0);
    right_arm.set_rotation_point(0.0, -7.0, 0.0);
    right_arm.set_rotation(arm_rot1, 0.0, 0.0);
    right_arm.add_box(60.0, 21.0, -13.0, -2.5, -3.0, 4.0, 30.0, 6.0, 0.0);
    right_arm.render(root, vertices, indices, color);

    let mut left_arm = ModelPart::new(60.0, 58.0, 128.0, 128.0);
    left_arm.set_rotation_point(0.0, -7.0, 0.0);
    left_arm.set_rotation(arm_rot2, 0.0, 0.0);
    left_arm.add_box(60.0, 58.0, 9.0, -2.5, -3.0, 4.0, 30.0, 6.0, 0.0);
    left_arm.render(root, vertices, indices, color);

    // Legs (37, 0)
    let leg_rot1 = (limb_swing * 0.6662 + PI).cos() * 1.0 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662).cos() * 1.0 * limb_amount;

    let mut right_leg = ModelPart::new(37.0, 0.0, 128.0, 128.0);
    right_leg.set_rotation_point(5.0, 11.0, 0.0);
    right_leg.set_rotation(leg_rot1, 0.0, 0.0);
    right_leg.add_box(37.0, 0.0, -3.5, -3.0, -3.0, 6.0, 16.0, 5.0, 0.0);
    right_leg.render(root, vertices, indices, color);

    let mut left_leg = ModelPart::new(37.0, 0.0, 128.0, 128.0);
    left_leg.set_rotation_point(-4.0, 11.0, 0.0);
    left_leg.set_rotation(leg_rot2, 0.0, 0.0);
    left_leg.add_box(37.0, 0.0, -2.5, -3.0, -3.0, 6.0, 16.0, 5.0, 0.0);
    left_leg.render(root, vertices, indices, color);
}

/// Builds the Snow Golem model (MCP ModelSnowMan).
pub fn build_snowman_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    // Head (0, 0)
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 64.0);
    head.set_rotation_point(0.0, 4.0, 0.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, -0.5);
    head.render(root, vertices, indices, color);

    // Body (0, 16)
    let body_yaw = head_yaw * 0.25;
    let mut body = ModelPart::new(0.0, 16.0, 64.0, 64.0);
    body.set_rotation_point(0.0, 13.0, 0.0);
    body.set_rotation(0.0, body_yaw, 0.0);
    body.add_box(0.0, 16.0, -5.0, -10.0, -5.0, 10.0, 10.0, 10.0, -0.5);
    body.render(root, vertices, indices, color);

    // Bottom Body (0, 36)
    let mut bottom = ModelPart::new(0.0, 36.0, 64.0, 64.0);
    bottom.set_rotation_point(0.0, 24.0, 0.0);
    bottom.add_box(0.0, 36.0, -6.0, -12.0, -6.0, 12.0, 12.0, 12.0, -0.5);
    bottom.render(root, vertices, indices, color);

    // Stick Hands (32, 0)
    let mut right_hand = ModelPart::new(32.0, 0.0, 64.0, 64.0);
    right_hand.set_rotation_point(body_yaw.cos() * 5.0, 6.0, -body_yaw.sin() * 5.0);
    right_hand.set_rotation(0.0, body_yaw, 1.0);
    right_hand.add_box(32.0, 0.0, -1.0, 0.0, -1.0, 12.0, 2.0, 2.0, -0.5);
    right_hand.render(root, vertices, indices, color);

    let mut left_hand = ModelPart::new(32.0, 0.0, 64.0, 64.0);
    left_hand.set_rotation_point(-body_yaw.cos() * 5.0, 6.0, body_yaw.sin() * 5.0);
    left_hand.set_rotation(0.0, PI + body_yaw, -1.0);
    left_hand.add_box(32.0, 0.0, -1.0, 0.0, -1.0, 12.0, 2.0, 2.0, -0.5);
    left_hand.render(root, vertices, indices, color);
}
