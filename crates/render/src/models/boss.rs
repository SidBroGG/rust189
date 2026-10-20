use super::{root_entity_matrix, ModelPart};
use crate::entity_renderer::EntityVertex;
use glam::{Mat4, Vec3};
use std::f32::consts::PI;
use world::entity::TrackedEntity;

/// Builds the Wither boss model (MCP ModelWither).
pub fn build_wither_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let mut root = root_entity_matrix(entity, partial_ticks);
    root *= Mat4::from_scale(Vec3::splat(2.0));
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();
    let age = entity.ticks_existed as f32 + partial_ticks;

    // Center head (0, 0)
    let mut center_head = ModelPart::new(0.0, 0.0, 64.0, 64.0);
    center_head.set_rotation_point(0.0, 0.0, 0.0);
    center_head.set_rotation(head_pitch, head_yaw, 0.0);
    center_head.add_box(0.0, 0.0, -4.0, -4.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    center_head.render(root, vertices, indices, color);

    // Right head (32, 0)
    let right_yaw = head_yaw * 0.75 + (age * 0.13).sin() * 0.1;
    let right_pitch = head_pitch * 0.75 + (age * 0.17).cos() * 0.08;
    let mut right_head = ModelPart::new(32.0, 0.0, 64.0, 64.0);
    right_head.set_rotation_point(-8.0, 4.0, 0.0);
    right_head.set_rotation(right_pitch, right_yaw, 0.0);
    right_head.add_box(32.0, 0.0, -4.0, -4.0, -4.0, 6.0, 6.0, 6.0, 0.0);
    right_head.render(root, vertices, indices, color);

    // Left head (32, 0)
    let left_yaw = head_yaw * 0.75 + (age * 0.15 + 1.0).sin() * 0.1;
    let left_pitch = head_pitch * 0.75 + (age * 0.19 + 1.0).cos() * 0.08;
    let mut left_head = ModelPart::new(32.0, 0.0, 64.0, 64.0);
    left_head.set_rotation_point(10.0, 4.0, 0.0);
    left_head.set_rotation(left_pitch, left_yaw, 0.0);
    left_head.add_box(32.0, 0.0, -4.0, -4.0, -4.0, 6.0, 6.0, 6.0, 0.0);
    left_head.render(root, vertices, indices, color);

    // Upper body horizontal bar (0, 16)
    let mut upper_bar = ModelPart::new(0.0, 16.0, 64.0, 64.0);
    upper_bar.add_box(0.0, 16.0, -10.0, 3.9, -0.5, 20.0, 3.0, 3.0, 0.0);
    upper_bar.render(root, vertices, indices, color);

    // Spine and Ribs (0, 22) & (24, 22)
    let rib_cycle = (age * 0.1).cos();
    let spine_rot_x = (0.065 + 0.05 * rib_cycle) * PI;

    let mut spine = ModelPart::new(0.0, 22.0, 64.0, 64.0);
    spine.set_rotation_point(-2.0, 6.9, -0.5);
    spine.set_rotation(spine_rot_x, 0.0, 0.0);
    spine.add_box(0.0, 22.0, 0.0, 0.0, 0.0, 3.0, 10.0, 3.0, 0.0);
    // 3 ribs
    spine.add_box(24.0, 22.0, -4.0, 1.5, 0.5, 11.0, 2.0, 2.0, 0.0);
    spine.add_box(24.0, 22.0, -4.0, 4.0, 0.5, 11.0, 2.0, 2.0, 0.0);
    spine.add_box(24.0, 22.0, -4.0, 6.5, 0.5, 11.0, 2.0, 2.0, 0.0);
    spine.render(root, vertices, indices, color);

    // Tail (12, 22)
    let tail_rot_x = (0.265 + 0.1 * rib_cycle) * PI;
    let mut tail = ModelPart::new(12.0, 22.0, 64.0, 64.0);
    tail.set_rotation_point(-2.0, 6.9 + spine_rot_x.cos() * 10.0, -0.5 + spine_rot_x.sin() * 10.0);
    tail.set_rotation(tail_rot_x, 0.0, 0.0);
    tail.add_box(12.0, 22.0, 0.0, 0.0, 0.0, 3.0, 6.0, 3.0, 0.0);
    tail.render(root, vertices, indices, color);
}

/// Builds the Ender Dragon model (MCP ModelDragon).
pub fn build_dragon_model(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let pos = entity.interpolated_pos(partial_ticks);
    let yaw = entity.interpolated_yaw(partial_ticks);
    let mut root = Mat4::from_translation(pos.as_vec3());
    root *= Mat4::from_rotation_y((-yaw).to_radians());
    root *= Mat4::from_scale(Vec3::new(-1.0, -1.0, 1.0));
    let age = entity.ticks_existed as f32 + partial_ticks;
    let flap_time = age * 0.2;

    // Body (0, 0)
    let mut body = ModelPart::new(0.0, 0.0, 256.0, 256.0);
    body.set_rotation_point(0.0, 4.0, 8.0);
    body.add_box(0.0, 0.0, -12.0, 0.0, -16.0, 24.0, 24.0, 64.0, 0.0);
    // Scales on body (220, 53)
    body.add_box(220.0, 53.0, -1.0, -6.0, -10.0, 2.0, 6.0, 12.0, 0.0);
    body.add_box(220.0, 53.0, -1.0, -6.0, 10.0, 2.0, 6.0, 12.0, 0.0);
    body.add_box(220.0, 53.0, -1.0, -6.0, 30.0, 2.0, 6.0, 12.0, 0.0);
    body.render(root, vertices, indices, color);

    // Neck segments (5 segments)
    let mut y_offset = 4.0;
    let mut z_offset = -8.0;
    for i in 0..5 {
        let curve = ((i as f32) * 0.45 + flap_time * PI * 2.0).cos() * 0.15;
        let mut spine = ModelPart::new(192.0, 104.0, 256.0, 256.0);
        spine.set_rotation_point(0.0, y_offset, z_offset);
        spine.set_rotation(curve, 0.0, 0.0);
        spine.add_box(192.0, 104.0, -5.0, -5.0, -5.0, 10.0, 10.0, 10.0, 0.0);
        spine.add_box(48.0, 0.0, -1.0, -9.0, -3.0, 2.0, 4.0, 6.0, 0.0);
        spine.render(root, vertices, indices, color);

        y_offset += curve.sin() * 10.0;
        z_offset -= curve.cos() * 10.0;
    }

    // Head (112, 30)
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();
    let mut head = ModelPart::new(112.0, 30.0, 256.0, 256.0);
    head.set_rotation_point(0.0, y_offset, z_offset);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    // Upper lip (176, 44)
    head.add_box(176.0, 44.0, -6.0, -1.0, -24.0, 12.0, 5.0, 16.0, 0.0);
    // Upper head (112, 30)
    head.add_box(112.0, 30.0, -8.0, -8.0, -10.0, 16.0, 16.0, 16.0, 0.0);
    // Nostrils (112, 0)
    head.add_box(112.0, 0.0, -5.0, -3.0, -22.0, 2.0, 2.0, 4.0, 0.0);
    head.add_box(112.0, 0.0, 3.0, -3.0, -22.0, 2.0, 2.0, 4.0, 0.0);
    // Horns / scales (0, 0)
    head.add_box(0.0, 0.0, -5.0, -12.0, -4.0, 2.0, 4.0, 6.0, 0.0);
    head.add_box(0.0, 0.0, 3.0, -12.0, -4.0, 2.0, 4.0, 6.0, 0.0);

    // Jaw (176, 65)
    let jaw_rot_x = ((flap_time * PI * 2.0).sin() + 1.0) * 0.2;
    let mut jaw = ModelPart::new(176.0, 65.0, 256.0, 256.0);
    jaw.set_rotation_point(0.0, 4.0, -8.0);
    jaw.set_rotation(jaw_rot_x, 0.0, 0.0);
    jaw.add_box(176.0, 65.0, -6.0, 0.0, -16.0, 12.0, 4.0, 16.0, 0.0);
    head.add_child(jaw);

    head.render(root, vertices, indices, color);

    // Wings
    let wing_flap = (flap_time * PI * 2.0).sin();
    let wing_rot_x = 0.125 - (flap_time * PI * 2.0).cos() * 0.2;
    let wing_rot_z = (wing_flap + 0.125) * 0.8;
    let wingtip_rot_z = -((flap_time * PI * 2.0 + 2.0).sin() + 0.5) * 0.75;

    // Left Wing
    let mut left_wing = ModelPart::new(112.0, 88.0, 256.0, 256.0);
    left_wing.set_rotation_point(-12.0, 5.0, 2.0);
    left_wing.set_rotation(wing_rot_x, 0.25, wing_rot_z);
    left_wing.add_box(112.0, 88.0, -56.0, -4.0, -4.0, 56.0, 8.0, 8.0, 0.0);
    left_wing.add_box(0.0, 88.0, -56.0, 0.0, 2.0, 56.0, 1.0, 56.0, 0.0);

    let mut left_wingtip = ModelPart::new(112.0, 136.0, 256.0, 256.0);
    left_wingtip.set_rotation_point(-56.0, 0.0, 0.0);
    left_wingtip.set_rotation(0.0, 0.0, wingtip_rot_z);
    left_wingtip.add_box(112.0, 136.0, -56.0, -2.0, -2.0, 56.0, 4.0, 4.0, 0.0);
    left_wingtip.add_box(0.0, 144.0, -56.0, 0.0, 2.0, 56.0, 1.0, 56.0, 0.0);
    left_wing.add_child(left_wingtip);
    left_wing.render(root, vertices, indices, color);

    // Right Wing (mirrored)
    let mut right_wing = ModelPart::new(112.0, 88.0, 256.0, 256.0);
    right_wing.mirror = true;
    right_wing.set_rotation_point(12.0, 5.0, 2.0);
    right_wing.set_rotation(wing_rot_x, -0.25, -wing_rot_z);
    right_wing.add_box(112.0, 88.0, 0.0, -4.0, -4.0, 56.0, 8.0, 8.0, 0.0);
    right_wing.add_box(0.0, 88.0, 0.0, 0.0, 2.0, 56.0, 1.0, 56.0, 0.0);

    let mut right_wingtip = ModelPart::new(112.0, 136.0, 256.0, 256.0);
    right_wingtip.mirror = true;
    right_wingtip.set_rotation_point(56.0, 0.0, 0.0);
    right_wingtip.set_rotation(0.0, 0.0, -wingtip_rot_z);
    right_wingtip.add_box(112.0, 136.0, 0.0, -2.0, -2.0, 56.0, 4.0, 4.0, 0.0);
    right_wingtip.add_box(0.0, 144.0, 0.0, 0.0, 2.0, 56.0, 1.0, 56.0, 0.0);
    right_wing.add_child(right_wingtip);
    right_wing.render(root, vertices, indices, color);

    // Legs
    let leg_anim = 1.3 + (flap_time * PI * 2.0).sin().abs() * 0.1;

    // Left Front Leg
    let mut fl_leg = ModelPart::new(112.0, 104.0, 256.0, 256.0);
    fl_leg.set_rotation_point(-12.0, 20.0, 2.0);
    fl_leg.set_rotation(leg_anim, 0.0, 0.0);
    fl_leg.add_box(112.0, 104.0, -4.0, -4.0, -4.0, 8.0, 24.0, 8.0, 0.0);
    let mut fl_tip = ModelPart::new(226.0, 138.0, 256.0, 256.0);
    fl_tip.set_rotation_point(0.0, 20.0, -1.0);
    fl_tip.add_box(226.0, 138.0, -3.0, -1.0, -3.0, 6.0, 24.0, 6.0, 0.0);
    let mut fl_foot = ModelPart::new(144.0, 104.0, 256.0, 256.0);
    fl_foot.set_rotation_point(0.0, 23.0, 0.0);
    fl_foot.add_box(144.0, 104.0, -4.0, 0.0, -12.0, 8.0, 4.0, 16.0, 0.0);
    fl_tip.add_child(fl_foot);
    fl_leg.add_child(fl_tip);
    fl_leg.render(root, vertices, indices, color);

    // Right Front Leg
    let mut fr_leg = ModelPart::new(112.0, 104.0, 256.0, 256.0);
    fr_leg.mirror = true;
    fr_leg.set_rotation_point(12.0, 20.0, 2.0);
    fr_leg.set_rotation(leg_anim, 0.0, 0.0);
    fr_leg.add_box(112.0, 104.0, -4.0, -4.0, -4.0, 8.0, 24.0, 8.0, 0.0);
    let mut fr_tip = ModelPart::new(226.0, 138.0, 256.0, 256.0);
    fr_tip.set_rotation_point(0.0, 20.0, -1.0);
    fr_tip.add_box(226.0, 138.0, -3.0, -1.0, -3.0, 6.0, 24.0, 6.0, 0.0);
    let mut fr_foot = ModelPart::new(144.0, 104.0, 256.0, 256.0);
    fr_foot.set_rotation_point(0.0, 23.0, 0.0);
    fr_foot.add_box(144.0, 104.0, -4.0, 0.0, -12.0, 8.0, 4.0, 16.0, 0.0);
    fr_tip.add_child(fr_foot);
    fr_leg.add_child(fr_tip);
    fr_leg.render(root, vertices, indices, color);

    // Left Rear Leg
    let mut rl_leg = ModelPart::new(0.0, 0.0, 256.0, 256.0);
    rl_leg.set_rotation_point(-16.0, 16.0, 42.0);
    rl_leg.set_rotation(1.0, 0.0, 0.0);
    rl_leg.add_box(0.0, 0.0, -8.0, -4.0, -8.0, 16.0, 32.0, 16.0, 0.0);
    let mut rl_tip = ModelPart::new(196.0, 0.0, 256.0, 256.0);
    rl_tip.set_rotation_point(0.0, 32.0, -4.0);
    rl_tip.add_box(196.0, 0.0, -6.0, -2.0, 0.0, 12.0, 32.0, 12.0, 0.0);
    let mut rl_foot = ModelPart::new(112.0, 0.0, 256.0, 256.0);
    rl_foot.set_rotation_point(0.0, 31.0, 4.0);
    rl_foot.add_box(112.0, 0.0, -9.0, 0.0, -20.0, 18.0, 6.0, 24.0, 0.0);
    rl_tip.add_child(rl_foot);
    rl_leg.add_child(rl_tip);
    rl_leg.render(root, vertices, indices, color);

    // Right Rear Leg
    let mut rr_leg = ModelPart::new(0.0, 0.0, 256.0, 256.0);
    rr_leg.mirror = true;
    rr_leg.set_rotation_point(16.0, 16.0, 42.0);
    rr_leg.set_rotation(1.0, 0.0, 0.0);
    rr_leg.add_box(0.0, 0.0, -8.0, -4.0, -8.0, 16.0, 32.0, 16.0, 0.0);
    let mut rr_tip = ModelPart::new(196.0, 0.0, 256.0, 256.0);
    rr_tip.set_rotation_point(0.0, 32.0, -4.0);
    rr_tip.add_box(196.0, 0.0, -6.0, -2.0, 0.0, 12.0, 32.0, 12.0, 0.0);
    let mut rr_foot = ModelPart::new(112.0, 0.0, 256.0, 256.0);
    rr_foot.set_rotation_point(0.0, 31.0, 4.0);
    rr_foot.add_box(112.0, 0.0, -9.0, 0.0, -20.0, 18.0, 6.0, 24.0, 0.0);
    rr_tip.add_child(rr_foot);
    rr_leg.add_child(rr_tip);
    rr_leg.render(root, vertices, indices, color);

    // 12 Tail Segments (MCP ModelDragon)
    let mut tail_y = 4.0;
    let mut tail_z = 48.0;
    let mut tail_x = 0.0;
    for j in 0..12 {
        let tail_curve = ((j as f32) * 0.45 + flap_time * PI * 2.0).cos() * 0.08;
        let tail_swing = ((j as f32) * 0.3 + flap_time * PI * 2.0).sin() * 0.1;
        let mut tail_seg = ModelPart::new(192.0, 104.0, 256.0, 256.0);
        tail_seg.set_rotation_point(tail_x, tail_y, tail_z);
        tail_seg.set_rotation(tail_curve, tail_swing, 0.0);
        tail_seg.add_box(192.0, 104.0, -5.0, -5.0, -5.0, 10.0, 10.0, 10.0, 0.0);
        tail_seg.add_box(48.0, 0.0, -1.0, -9.0, -3.0, 2.0, 4.0, 6.0, 0.0);
        tail_seg.render(root, vertices, indices, color);

        tail_y += tail_curve.sin() * 10.0;
        tail_z += tail_curve.cos() * 10.0;
        tail_x += tail_swing.sin() * 10.0;
    }
}
