use super::{root_entity_matrix, ModelPart};
use crate::entity_renderer::EntityVertex;
use glam::{Mat4, Vec3};
use std::f32::consts::PI;
use world::entity::TrackedEntity;

/// The 16 vanilla Minecraft dye colors (RGB).
pub const DYE_COLORS: [[f32; 3]; 16] = [
    [1.0, 1.0, 1.0],       // 0: White
    [0.85, 0.5, 0.2],      // 1: Orange
    [0.7, 0.3, 0.85],      // 2: Magenta
    [0.4, 0.6, 0.85],      // 3: Light Blue
    [0.9, 0.9, 0.2],       // 4: Yellow
    [0.5, 0.8, 0.1],       // 5: Lime
    [0.95, 0.5, 0.65],     // 6: Pink
    [0.3, 0.3, 0.3],       // 7: Gray
    [0.6, 0.6, 0.6],       // 8: Light Gray
    [0.3, 0.6, 0.7],       // 9: Cyan
    [0.5, 0.25, 0.7],      // 10: Purple
    [0.2, 0.3, 0.7],       // 11: Blue
    [0.4, 0.3, 0.2],       // 12: Brown
    [0.4, 0.5, 0.2],       // 13: Green
    [0.6, 0.2, 0.2],       // 14: Red
    [0.1, 0.1, 0.1],       // 15: Black
];

/// Returns dye color as RGBA.
pub fn get_dye_rgba(dye: u8) -> [f32; 4] {
    let rgb = DYE_COLORS[(dye & 15) as usize];
    [rgb[0], rgb[1], rgb[2], 1.0]
}

/// Renders Sheep Wool layer (MCP LayerSheepWool / ModelSheep1).
pub fn render_sheep_wool_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    dye_color: u8,
    is_sheared: bool,
) {
    if is_sheared {
        return;
    }

    let root = root_entity_matrix(entity, partial_ticks);
    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();
    let color = get_dye_rgba(dye_color);

    // Head wool (0, 0) expand 0.6
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 6.0, -8.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -3.0, -4.0, -6.0, 6.0, 6.0, 8.0, 0.6);
    head.render(root, vertices, indices, color);

    // Body wool (28, 8) expand 1.75
    let mut body = ModelPart::new(28.0, 8.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 5.0, 2.0);
    body.set_rotation(PI / 2.0, 0.0, 0.0);
    body.add_box(28.0, 8.0, -4.0, -10.0, -7.0, 8.0, 16.0, 6.0, 1.75);
    body.render(root, vertices, indices, color);

    // Legs wool (0, 16) expand 0.5
    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    let mut leg1 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg1.set_rotation_point(-3.0, 12.0, 7.0);
    leg1.set_rotation(leg_rot1, 0.0, 0.0);
    leg1.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.5);
    leg1.render(root, vertices, indices, color);

    let mut leg2 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg2.set_rotation_point(3.0, 12.0, 7.0);
    leg2.set_rotation(leg_rot2, 0.0, 0.0);
    leg2.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.5);
    leg2.render(root, vertices, indices, color);

    let mut leg3 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg3.set_rotation_point(-3.0, 12.0, -5.0);
    leg3.set_rotation(leg_rot2, 0.0, 0.0);
    leg3.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.5);
    leg3.render(root, vertices, indices, color);

    let mut leg4 = ModelPart::new(0.0, 16.0, 64.0, 32.0);
    leg4.set_rotation_point(3.0, 12.0, -5.0);
    leg4.set_rotation(leg_rot1, 0.0, 0.0);
    leg4.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 6.0, 4.0, 0.5);
    leg4.render(root, vertices, indices, color);
}

/// Renders Pig Saddle layer (MCP LayerSaddle).
pub fn render_pig_saddle_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let mut body = ModelPart::new(28.0, 8.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 11.0, 2.0);
    body.set_rotation(PI / 2.0, 0.0, 0.0);
    body.add_box(28.0, 8.0, -5.0, -10.0, -7.0, 10.0, 16.0, 8.0, 0.5);
    body.render(root, vertices, indices, color);
}

/// Renders Wolf Collar layer (MCP LayerWolfCollar).
pub fn render_wolf_collar_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    dye_color: u8,
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let color = get_dye_rgba(dye_color);

    // Mane / Collar (21, 0)
    let mut mane = ModelPart::new(21.0, 0.0, 64.0, 32.0);
    mane.set_rotation_point(-1.0, 14.0, -3.0);
    mane.set_rotation(PI / 2.0, 0.0, 0.0);
    mane.add_box(21.0, 0.0, -3.0, -3.0, -3.0, 8.0, 6.0, 7.0, 0.0);
    mane.render(root, vertices, indices, color);
}

/// Renders Snowman Pumpkin Head layer (MCP LayerSnowmanHead).
pub fn render_snowman_head_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    let mut pumpkin = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    pumpkin.set_rotation_point(0.0, 4.0, 0.0);
    pumpkin.set_rotation(head_pitch, head_yaw, 0.0);
    pumpkin.add_box(0.0, 0.0, -5.0, -10.0, -5.0, 10.0, 10.0, 10.0, -0.5);
    pumpkin.render(root, vertices, indices, color);
}

fn render_crossed_quads(
    matrix: Mat4,
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    color: [f32; 4],
) {
    let s = 0.5f32;
    let q1 = [
        Vec3::new(-s, -s, -s),
        Vec3::new(s, -s, s),
        Vec3::new(s, s, s),
        Vec3::new(-s, s, -s),
    ];
    let q2 = [
        Vec3::new(-s, -s, s),
        Vec3::new(s, -s, -s),
        Vec3::new(s, s, -s),
        Vec3::new(-s, s, s),
    ];
    for q in &[q1, q2] {
        let base_idx = vertices.len() as u32;
        let p0 = (matrix * q[0].extend(1.0)).truncate();
        let p1 = (matrix * q[1].extend(1.0)).truncate();
        let p2 = (matrix * q[2].extend(1.0)).truncate();
        let p3 = (matrix * q[3].extend(1.0)).truncate();
        let normal = [0.0, 1.0, 0.0];

        vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0, 1.0], normal, color });
        vertices.push(EntityVertex { position: p1.to_array(), uv: [1.0, 1.0], normal, color });
        vertices.push(EntityVertex { position: p2.to_array(), uv: [1.0, 0.0], normal, color });
        vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0, 0.0], normal, color });
        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2, base_idx, base_idx + 2, base_idx + 3]);

        let b_idx = vertices.len() as u32;
        vertices.push(EntityVertex { position: p1.to_array(), uv: [1.0, 1.0], normal, color });
        vertices.push(EntityVertex { position: p0.to_array(), uv: [0.0, 1.0], normal, color });
        vertices.push(EntityVertex { position: p3.to_array(), uv: [0.0, 0.0], normal, color });
        vertices.push(EntityVertex { position: p2.to_array(), uv: [1.0, 0.0], normal, color });
        indices.extend_from_slice(&[b_idx, b_idx + 1, b_idx + 2, b_idx, b_idx + 2, b_idx + 3]);
    }
}

/// Renders Mooshroom Mushroom layer (MCP LayerMooshroomMushroom - crossed quads).
pub fn render_mooshroom_mushroom_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);

    // Shroom 1 on back right
    let m1 = root
        * Mat4::from_translation(Vec3::new(0.2, 0.75, 0.3))
        * Mat4::from_rotation_y((42.0_f32).to_radians())
        * Mat4::from_scale(Vec3::splat(0.8));
    render_crossed_quads(m1, vertices, indices, color);

    // Shroom 2 on back left
    let m2 = root
        * Mat4::from_translation(Vec3::new(-0.2, 0.75, -0.3))
        * Mat4::from_rotation_y((42.0_f32).to_radians())
        * Mat4::from_scale(Vec3::splat(0.8));
    render_crossed_quads(m2, vertices, indices, color);

    // Shroom 3 on head
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();
    let m3 = root
        * Mat4::from_translation(Vec3::new(0.0, 0.25, -0.5))
        * Mat4::from_rotation_y(head_yaw)
        * Mat4::from_rotation_x(head_pitch)
        * Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0))
        * Mat4::from_rotation_y((12.0_f32).to_radians())
        * Mat4::from_scale(Vec3::splat(0.8));
    render_crossed_quads(m3, vertices, indices, color);
}

/// Renders Iron Golem Poppy Flower layer (MCP LayerIronGolemFlower).
pub fn render_iron_golem_flower_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let mut flower = ModelPart::new(0.0, 0.0, 32.0, 32.0);
    flower.set_rotation_point(-11.5, 15.0, -4.0);
    flower.add_box(0.0, 0.0, -2.0, -2.0, -2.0, 4.0, 8.0, 4.0, 0.0);
    flower.render(root, vertices, indices, color);
}

/// Renders glowing Spider Eyes layer (MCP LayerSpiderEyes) in full-bright white.
pub fn render_spider_eyes_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    is_cave_spider: bool,
) {
    let mut root = root_entity_matrix(entity, partial_ticks);
    if is_cave_spider {
        root *= Mat4::from_scale(Vec3::splat(0.7));
    }
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    let mut head = ModelPart::new(32.0, 4.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 15.0, -3.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(32.0, 4.0, -4.0, -4.0, -8.0, 8.0, 8.0, 8.0, 0.1);
    head.render(root, vertices, indices, [1.0, 1.0, 1.0, 1.0]);
}

/// Renders glowing Enderman Eyes layer (MCP LayerEndermanEyes) in full-bright white.
pub fn render_enderman_eyes_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, -14.0, 0.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, 0.1);
    head.render(root, vertices, indices, [1.0, 1.0, 1.0, 1.0]);
}

/// Renders glowing Ender Dragon Eyes layer (MCP LayerEnderDragonEyes) in full-bright white.
pub fn render_dragon_eyes_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    let mut head = ModelPart::new(112.0, 30.0, 256.0, 256.0);
    head.set_rotation_point(0.0, 20.0, -40.0);
    head.set_rotation(head_pitch, head_yaw, 0.0);
    head.add_box(112.0, 30.0, -8.0, -8.0, -10.0, 16.0, 16.0, 16.0, 0.1);
    head.render(root, vertices, indices, [1.0, 1.0, 1.0, 1.0]);
}

/// Renders Creeper Charge Shield layer (MCP LayerCreeperCharge).
pub fn render_creeper_charge_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let age = entity.ticks_existed as f32 + partial_ticks;
    let alpha = (age * 0.1).sin().abs() * 0.3 + 0.4;
    let color = [0.5, 0.8, 1.0, alpha];

    // Head (0, 0) expand 2.0
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    head.set_rotation_point(0.0, 6.0, 0.0);
    head.add_box(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, 2.0);
    head.render(root, vertices, indices, color);

    // Body (16, 16) expand 2.0
    let mut body = ModelPart::new(16.0, 16.0, 64.0, 32.0);
    body.set_rotation_point(0.0, 6.0, 0.0);
    body.add_box(16.0, 16.0, -4.0, 0.0, -2.0, 8.0, 12.0, 4.0, 2.0);
    body.render(root, vertices, indices, color);
}

/// Renders Wither Armor Shield layer (MCP LayerWitherAura).
pub fn render_wither_aura_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let age = entity.ticks_existed as f32 + partial_ticks;
    let alpha = (age * 0.1).cos().abs() * 0.3 + 0.4;
    let color = [0.8, 0.8, 1.0, alpha];

    // Central head expand 0.5
    let mut head = ModelPart::new(0.0, 0.0, 64.0, 64.0);
    head.add_box(0.0, 0.0, -4.0, -4.0, -4.0, 8.0, 8.0, 8.0, 0.5);
    head.render(root, vertices, indices, color);

    // Body expand 0.5
    let mut bar = ModelPart::new(0.0, 16.0, 64.0, 64.0);
    bar.add_box(0.0, 16.0, -10.0, 3.9, -0.5, 20.0, 3.0, 3.0, 0.5);
    bar.render(root, vertices, indices, color);
}

/// Renders outer Slime Gel layer (MCP LayerSlimeGel).
pub fn render_slime_gel_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let slime_size = entity.metadata.iter().find(|m| m.index == 16).and_then(|m| match m.value {
        world::entity::MetadataValue::Byte(b) => Some(b.max(1) as f32),
        world::entity::MetadataValue::Int(i) => Some((i as f32).max(1.0)),
        _ => None,
    }).unwrap_or(1.0);
    let mut root = root_entity_matrix(entity, partial_ticks);
    root *= Mat4::from_scale(Vec3::splat(slime_size));
    let mut gel_color = color;
    gel_color[3] = 0.6; // Translucent outer cube

    let mut outer = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    outer.set_rotation_point(0.0, 24.0, 0.0);
    outer.add_box(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    outer.render(root, vertices, indices, gel_color);
}

/// Renders Biped Armor layer (MCP LayerBipedArmor).
pub fn render_biped_armor_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
    slot: usize, // 0: Boots, 1: Leggings, 2: Chestplate, 3: Helmet
) {
    let root = root_entity_matrix(entity, partial_ticks);
    let (limb_swing, limb_amount) = entity.interpolated_limb_swing(partial_ticks);
    let head_yaw = (entity.interpolated_head_yaw(partial_ticks) - entity.interpolated_render_yaw_offset(partial_ticks)).to_radians();
    let head_pitch = entity.interpolated_pitch(partial_ticks).to_radians();

    let leg_rot1 = (limb_swing * 0.6662).cos() * 1.4 * limb_amount;
    let leg_rot2 = (limb_swing * 0.6662 + PI).cos() * 1.4 * limb_amount;

    match slot {
        3 => {
            // Helmet (0, 0) expand 1.0
            let mut helmet = ModelPart::new(0.0, 0.0, 64.0, 32.0);
            helmet.set_rotation_point(0.0, 0.0, 0.0);
            helmet.set_rotation(head_pitch, head_yaw, 0.0);
            helmet.add_box(0.0, 0.0, -4.0, -8.0, -4.0, 8.0, 8.0, 8.0, 1.0);
            helmet.render(root, vertices, indices, color);
        }
        2 => {
            // Chestplate (16, 16) expand 1.0
            let mut chest = ModelPart::new(16.0, 16.0, 64.0, 32.0);
            chest.set_rotation_point(0.0, 0.0, 0.0);
            chest.add_box(16.0, 16.0, -4.0, 0.0, -2.0, 8.0, 12.0, 4.0, 1.0);
            chest.render(root, vertices, indices, color);

            // Sleeves expand 1.0
            let mut r_arm = ModelPart::new(40.0, 16.0, 64.0, 32.0);
            r_arm.set_rotation_point(-5.0, 2.0, 0.0);
            r_arm.set_rotation(leg_rot2 * 0.5, 0.0, 0.0);
            r_arm.add_box(40.0, 16.0, -3.0, -2.0, -2.0, 4.0, 12.0, 4.0, 1.0);
            r_arm.render(root, vertices, indices, color);

            let mut l_arm = ModelPart::new(40.0, 16.0, 64.0, 32.0);
            l_arm.mirror = true;
            l_arm.set_rotation_point(5.0, 2.0, 0.0);
            l_arm.set_rotation(leg_rot1 * 0.5, 0.0, 0.0);
            l_arm.add_box(40.0, 16.0, -1.0, -2.0, -2.0, 4.0, 12.0, 4.0, 1.0);
            l_arm.render(root, vertices, indices, color);
        }
        1 => {
            // Leggings (0, 16) expand 0.5
            let mut r_leg = ModelPart::new(0.0, 16.0, 64.0, 32.0);
            r_leg.set_rotation_point(-1.9, 12.0, 0.0);
            r_leg.set_rotation(leg_rot1, 0.0, 0.0);
            r_leg.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 9.0, 4.0, 0.5);
            r_leg.render(root, vertices, indices, color);

            let mut l_leg = ModelPart::new(0.0, 16.0, 64.0, 32.0);
            l_leg.mirror = true;
            l_leg.set_rotation_point(1.9, 12.0, 0.0);
            l_leg.set_rotation(leg_rot2, 0.0, 0.0);
            l_leg.add_box(0.0, 16.0, -2.0, 0.0, -2.0, 4.0, 9.0, 4.0, 0.5);
            l_leg.render(root, vertices, indices, color);
        }
        0 => {
            // Boots (0, 16) expand 1.0
            let mut r_boot = ModelPart::new(0.0, 16.0, 64.0, 32.0);
            r_boot.set_rotation_point(-1.9, 12.0, 0.0);
            r_boot.set_rotation(leg_rot1, 0.0, 0.0);
            r_boot.add_box(0.0, 16.0, -2.0, 6.0, -2.0, 4.0, 6.0, 4.0, 1.0);
            r_boot.render(root, vertices, indices, color);

            let mut l_boot = ModelPart::new(0.0, 16.0, 64.0, 32.0);
            l_boot.mirror = true;
            l_boot.set_rotation_point(1.9, 12.0, 0.0);
            l_boot.set_rotation(leg_rot2, 0.0, 0.0);
            l_boot.add_box(0.0, 16.0, -2.0, 6.0, -2.0, 4.0, 6.0, 4.0, 1.0);
            l_boot.render(root, vertices, indices, color);
        }
        _ => {}
    }
}

/// Renders Enderman Held Block layer (MCP LayerHeldBlock).
pub fn render_enderman_held_block_layer(
    vertices: &mut Vec<EntityVertex>,
    indices: &mut Vec<u32>,
    entity: &TrackedEntity,
    partial_ticks: f32,
    color: [f32; 4],
) {
    let root = root_entity_matrix(entity, partial_ticks);
    // Block held in hands in front of body
    let mut block = ModelPart::new(0.0, 0.0, 64.0, 32.0);
    block.set_rotation_point(0.0, 1.0, -5.0);
    block.add_box(0.0, 0.0, -4.0, -4.0, -4.0, 8.0, 8.0, 8.0, 0.0);
    block.render(root, vertices, indices, color);
}
