use glam::{DVec3, Vec3};
use hashbrown::HashMap;
use protocol::codec::{MetadataEntry, MetadataValue, Slot};
use uuid::Uuid;

/// Category of entity in the world.
#[derive(Debug, Clone, PartialEq)]
pub enum EntityKind {
    Player {
        uuid: Uuid,
        name: String,
        held_item: i16,
    },
    Mob {
        mob_type: u8,
    },
    Object {
        object_type: u8,
        data: i32,
    },
}

impl EntityKind {
    pub fn is_player(&self) -> bool {
        matches!(self, EntityKind::Player { .. })
    }

    pub fn is_projectile_or_object(&self) -> bool {
        matches!(self, EntityKind::Object { .. })
    }
}

/// A tracked entity instance with tick-level smoothing and frame-rate interpolation.
#[derive(Debug, Clone)]
pub struct TrackedEntity {
    pub id: i32,
    pub kind: EntityKind,
    pub pos: DVec3,
    pub prev_pos: DVec3,
    pub target_pos: DVec3,
    pub server_pos_fixed: (i32, i32, i32),
    pub yaw: f32,
    pub pitch: f32,
    pub head_yaw: f32,
    pub prev_yaw: f32,
    pub prev_pitch: f32,
    pub prev_head_yaw: f32,
    pub target_yaw: f32,
    pub target_pitch: f32,
    pub target_head_yaw: f32,
    pub interpolation_steps: u8,
    pub velocity: Vec3,
    pub on_ground: bool,

    pub render_yaw_offset: f32,
    pub prev_render_yaw_offset: f32,
    pub ticks_existed: u32,
    // Animation & State
    pub limb_swing: f32,
    pub limb_swing_amount: f32,
    pub prev_limb_swing_amount: f32,
    pub swing_progress: f32,
    pub is_swinging: bool,
    pub swing_ticks: u8,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub is_invisible: bool,
    pub custom_name: Option<String>,
    pub custom_name_visible: bool,
    pub equipment: [Option<Slot>; 5],
    pub metadata: Vec<MetadataEntry>,
}

impl TrackedEntity {
    pub fn new(id: i32, kind: EntityKind, pos: DVec3, yaw: f32, pitch: f32) -> Self {
        let fixed_x = (pos.x * 32.0).floor() as i32;
        let fixed_y = (pos.y * 32.0).floor() as i32;
        let fixed_z = (pos.z * 32.0).floor() as i32;

        Self {
            id,
            kind,
            pos,
            prev_pos: pos,
            target_pos: pos,
            server_pos_fixed: (fixed_x, fixed_y, fixed_z),
            yaw,
            pitch,
            head_yaw: yaw,
            prev_yaw: yaw,
            prev_pitch: pitch,
            prev_head_yaw: yaw,
            target_yaw: yaw,
            target_pitch: pitch,
            target_head_yaw: yaw,
            render_yaw_offset: yaw,
            prev_render_yaw_offset: yaw,
            ticks_existed: 0,
            interpolation_steps: 0,
            velocity: Vec3::ZERO,
            on_ground: true,
            limb_swing: 0.0,
            limb_swing_amount: 0.0,
            prev_limb_swing_amount: 0.0,
            swing_progress: 0.0,
            is_swinging: false,
            swing_ticks: 0,
            is_sneaking: false,
            is_sprinting: false,
            is_invisible: false,
            custom_name: None,
            custom_name_visible: false,
            equipment: [None, None, None, None, None],
            metadata: Vec::new(),
        }
    }

    /// Sets new target position and rotation, enabling multi-step interpolation.
    pub fn set_target_transform(
        &mut self,
        target_pos: DVec3,
        target_yaw: f32,
        target_pitch: f32,
        steps: u8,
    ) {
        self.target_pos = target_pos;
        self.target_yaw = target_yaw;
        self.target_pitch = target_pitch;
        self.interpolation_steps = steps.max(1);
    }

    /// Advances simulation one tick (20 TPS).
    pub fn tick(&mut self) {
        self.ticks_existed = self.ticks_existed.wrapping_add(1);
        self.prev_pos = self.pos;
        self.prev_yaw = self.yaw;
        self.prev_pitch = self.pitch;
        self.prev_head_yaw = self.head_yaw;
        self.prev_render_yaw_offset = self.render_yaw_offset;
        self.prev_limb_swing_amount = self.limb_swing_amount;

        if self.interpolation_steps > 0 {
            let steps = self.interpolation_steps as f64;
            self.pos += (self.target_pos - self.pos) / steps;

            let diff_yaw = wrap_degrees(self.target_yaw - self.yaw);
            let diff_pitch = wrap_degrees(self.target_pitch - self.pitch);
            let diff_head = wrap_degrees(self.target_head_yaw - self.head_yaw);

            self.yaw += diff_yaw / (steps as f32);
            self.pitch += diff_pitch / (steps as f32);
            self.head_yaw += diff_head / (steps as f32);

            self.interpolation_steps -= 1;
        }

        // Update limb swing (walking animation)
        let delta_x = self.pos.x - self.prev_pos.x;
        let delta_z = self.pos.z - self.prev_pos.z;
        let dist_xz = delta_x.hypot(delta_z);
        let target_swing_amount = ((dist_xz * 4.0) as f32).min(1.0);
        self.limb_swing_amount += (target_swing_amount - self.limb_swing_amount) * 0.4;
        self.limb_swing += self.limb_swing_amount;

        // Update render_yaw_offset (body rotation lagging head, matching EntityLivingBase)
        let dist_sq = (delta_x * delta_x + delta_z * delta_z) as f32;
        let mut move_yaw = self.render_yaw_offset;
        if dist_sq > 0.0025 {
            move_yaw = (delta_z as f32).atan2(delta_x as f32).to_degrees() - 90.0;
        }
        if self.is_swinging {
            move_yaw = self.yaw;
        }

        let f = wrap_degrees(move_yaw - self.render_yaw_offset);
        self.render_yaw_offset += f * 0.3;

        let mut f1 = wrap_degrees(self.yaw - self.render_yaw_offset);
        if f1 < -75.0 {
            f1 = -75.0;
        }
        if f1 >= 75.0 {
            f1 = 75.0;
        }
        self.render_yaw_offset = self.yaw - f1;
        if f1 * f1 > 2500.0 {
            self.render_yaw_offset += f1 * 0.2;
        }

        // Arm swing animation (6 ticks duration, matching vanilla entity player)
        if self.is_swinging {
            self.swing_ticks += 1;
            if self.swing_ticks >= 6 {
                self.is_swinging = false;
                self.swing_ticks = 0;
                self.swing_progress = 0.0;
            } else {
                self.swing_progress = (self.swing_ticks as f32) / 6.0;
            }
        }

        // Apply metadata flags
        self.apply_metadata_flags();
    }

    /// Extracts sneak/sprint/invisible flags and custom name from entity metadata.
    fn apply_metadata_flags(&mut self) {
        for entry in &self.metadata {
            match entry.index {
                0 => {
                    if let MetadataValue::Byte(b) = entry.value {
                        self.is_sneaking = (b & 0x02) != 0;
                        self.is_sprinting = (b & 0x08) != 0;
                        self.is_invisible = (b & 0x20) != 0;
                    }
                }
                2 => {
                    if let MetadataValue::String(s) = &entry.value {
                        self.custom_name = if s.is_empty() { None } else { Some(s.clone()) };
                    }
                }
                3 => {
                    if let MetadataValue::Byte(b) = entry.value {
                        self.custom_name_visible = b == 1;
                    }
                }
                _ => {}
            }
        }
    }

    /// Calculates smooth per-frame interpolated position using partial ticks [0.0..1.0].
    pub fn interpolated_pos(&self, partial_ticks: f32) -> DVec3 {
        let pt = partial_ticks.clamp(0.0, 1.0) as f64;
        self.prev_pos + (self.pos - self.prev_pos) * pt
    }

    /// Calculates smooth per-frame interpolated yaw.
    pub fn interpolated_yaw(&self, partial_ticks: f32) -> f32 {
        let pt = partial_ticks.clamp(0.0, 1.0);
        let diff = wrap_degrees(self.yaw - self.prev_yaw);
        self.prev_yaw + diff * pt
    }

    /// Calculates smooth per-frame interpolated pitch.
    pub fn interpolated_pitch(&self, partial_ticks: f32) -> f32 {
        let pt = partial_ticks.clamp(0.0, 1.0);
        let diff = wrap_degrees(self.pitch - self.prev_pitch);
        self.prev_pitch + diff * pt
    }

    /// Calculates smooth per-frame interpolated head yaw.
    pub fn interpolated_head_yaw(&self, partial_ticks: f32) -> f32 {
        let pt = partial_ticks.clamp(0.0, 1.0);
        let diff = wrap_degrees(self.head_yaw - self.prev_head_yaw);
        self.prev_head_yaw + diff * pt
    }

    /// Calculates smooth per-frame interpolated render yaw offset (body yaw).
    pub fn interpolated_render_yaw_offset(&self, partial_ticks: f32) -> f32 {
        let pt = partial_ticks.clamp(0.0, 1.0);
        let diff = wrap_degrees(self.render_yaw_offset - self.prev_render_yaw_offset);
        self.prev_render_yaw_offset + diff * pt
    }

    /// Returns interpolated (limb_swing, limb_swing_amount).
    pub fn interpolated_limb_swing(&self, partial_ticks: f32) -> (f32, f32) {
        let pt = partial_ticks.clamp(0.0, 1.0);
        let amount = self.prev_limb_swing_amount
            + (self.limb_swing_amount - self.prev_limb_swing_amount) * pt;
        let swing = self.limb_swing - self.limb_swing_amount * (1.0 - pt);
        (swing, amount.min(1.0))
    }

    /// Returns the AxisAlignedBB for this entity according to vanilla dimensions.
    pub fn bounding_box(&self) -> crate::physics::aabb::AABB {
        let (width, height) = match &self.kind {
            EntityKind::Player { .. } => {
                if self.is_sneaking {
                    (0.6, 1.65)
                } else {
                    (0.6, 1.8)
                }
            }
            EntityKind::Mob { mob_type } => match *mob_type {
                54 => (0.6, 1.95), // Zombie
                51 => (0.6, 1.99), // Skeleton
                50 => (0.6, 1.7),  // Creeper
                52 => (1.4, 0.9),  // Spider
                58 => (0.6, 2.9),  // Enderman
                100 => (1.4, 2.7), // Iron Golem
                65 => (0.4, 0.3),  // Bat
                _ => (0.6, 1.8),
            },
            EntityKind::Object { object_type, .. } => match *object_type {
                60 | 61 | 62 => (0.5, 0.5), // Projectiles
                2 => (0.25, 0.25),          // Item drop
                _ => (0.5, 0.5),
            },
        };
        let half_w = width * 0.5;
        crate::physics::aabb::AABB::new(
            self.pos.x - half_w,
            self.pos.y,
            self.pos.z - half_w,
            self.pos.x + half_w,
            self.pos.y + height,
            self.pos.z + half_w,
        )
    }
}

/// Entity tracker that manages all alive entities in the world.
#[derive(Debug, Default)]
pub struct EntityTracker {
    pub entities: HashMap<i32, TrackedEntity>,
}

impl EntityTracker {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn count(&self) -> usize {
        self.entities.len()
    }

    pub fn get(&self, id: i32) -> Option<&TrackedEntity> {
        self.entities.get(&id)
    }

    pub fn get_mut(&mut self, id: i32) -> Option<&mut TrackedEntity> {
        self.entities.get_mut(&id)
    }

    pub fn spawn_player(
        &mut self,
        entity_id: i32,
        player_id: Uuid,
        name: String,
        fixed_x: i32,
        fixed_y: i32,
        fixed_z: i32,
        yaw_byte: i8,
        pitch_byte: i8,
        current_item: i16,
        metadata: Vec<MetadataEntry>,
    ) {
        let pos = DVec3::new(
            fixed_x as f64 / 32.0,
            fixed_y as f64 / 32.0,
            fixed_z as f64 / 32.0,
        );
        let yaw = (yaw_byte as f32 * 360.0) / 256.0;
        let pitch = (pitch_byte as f32 * 360.0) / 256.0;

        let mut entity = TrackedEntity::new(
            entity_id,
            EntityKind::Player {
                uuid: player_id,
                name,
                held_item: current_item,
            },
            pos,
            yaw,
            pitch,
        );
        entity.server_pos_fixed = (fixed_x, fixed_y, fixed_z);
        entity.metadata = metadata;
        entity.apply_metadata_flags();

        self.entities.insert(entity_id, entity);
    }

    pub fn spawn_mob(
        &mut self,
        entity_id: i32,
        mob_type: u8,
        fixed_x: i32,
        fixed_y: i32,
        fixed_z: i32,
        yaw_byte: i8,
        pitch_byte: i8,
        head_pitch_byte: i8,
        vel_x: i16,
        vel_y: i16,
        vel_z: i16,
        metadata: Vec<MetadataEntry>,
    ) {
        let pos = DVec3::new(
            fixed_x as f64 / 32.0,
            fixed_y as f64 / 32.0,
            fixed_z as f64 / 32.0,
        );
        let yaw = (yaw_byte as f32 * 360.0) / 256.0;
        let pitch = (pitch_byte as f32 * 360.0) / 256.0;
        let head_yaw = (head_pitch_byte as f32 * 360.0) / 256.0;

        let mut entity = TrackedEntity::new(entity_id, EntityKind::Mob { mob_type }, pos, yaw, pitch);
        entity.head_yaw = head_yaw;
        entity.prev_head_yaw = head_yaw;
        entity.target_head_yaw = head_yaw;
        entity.server_pos_fixed = (fixed_x, fixed_y, fixed_z);
        entity.velocity = Vec3::new(
            vel_x as f32 / 8000.0,
            vel_y as f32 / 8000.0,
            vel_z as f32 / 8000.0,
        );
        entity.metadata = metadata;
        entity.apply_metadata_flags();

        self.entities.insert(entity_id, entity);
    }

    pub fn spawn_object(
        &mut self,
        entity_id: i32,
        object_type: u8,
        fixed_x: i32,
        fixed_y: i32,
        fixed_z: i32,
        pitch_byte: i8,
        yaw_byte: i8,
        data: i32,
        vel_x: i16,
        vel_y: i16,
        vel_z: i16,
    ) {
        let pos = DVec3::new(
            fixed_x as f64 / 32.0,
            fixed_y as f64 / 32.0,
            fixed_z as f64 / 32.0,
        );
        let yaw = (yaw_byte as f32 * 360.0) / 256.0;
        let pitch = (pitch_byte as f32 * 360.0) / 256.0;

        let mut entity = TrackedEntity::new(
            entity_id,
            EntityKind::Object { object_type, data },
            pos,
            yaw,
            pitch,
        );
        entity.server_pos_fixed = (fixed_x, fixed_y, fixed_z);
        entity.velocity = Vec3::new(
            vel_x as f32 / 8000.0,
            vel_y as f32 / 8000.0,
            vel_z as f32 / 8000.0,
        );

        self.entities.insert(entity_id, entity);
    }

    pub fn destroy_entities(&mut self, entity_ids: &[i32]) {
        for id in entity_ids {
            self.entities.remove(id);
        }
    }

    pub fn rel_move(
        &mut self,
        entity_id: i32,
        delta_x: i8,
        delta_y: i8,
        delta_z: i8,
        on_ground: bool,
    ) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.server_pos_fixed.0 += delta_x as i32;
            entity.server_pos_fixed.1 += delta_y as i32;
            entity.server_pos_fixed.2 += delta_z as i32;

            let target = DVec3::new(
                entity.server_pos_fixed.0 as f64 / 32.0,
                entity.server_pos_fixed.1 as f64 / 32.0,
                entity.server_pos_fixed.2 as f64 / 32.0,
            );
            entity.target_pos = target;
            entity.interpolation_steps = 3;
            entity.on_ground = on_ground;
        }
    }

    pub fn look(&mut self, entity_id: i32, yaw_byte: i8, pitch_byte: i8, on_ground: bool) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            let yaw = (yaw_byte as f32 * 360.0) / 256.0;
            let pitch = (pitch_byte as f32 * 360.0) / 256.0;

            entity.target_yaw = yaw;
            entity.target_pitch = pitch;
            entity.interpolation_steps = 3;
            entity.on_ground = on_ground;
        }
    }

    pub fn look_move(
        &mut self,
        entity_id: i32,
        delta_x: i8,
        delta_y: i8,
        delta_z: i8,
        yaw_byte: i8,
        pitch_byte: i8,
        on_ground: bool,
    ) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.server_pos_fixed.0 += delta_x as i32;
            entity.server_pos_fixed.1 += delta_y as i32;
            entity.server_pos_fixed.2 += delta_z as i32;

            let target = DVec3::new(
                entity.server_pos_fixed.0 as f64 / 32.0,
                entity.server_pos_fixed.1 as f64 / 32.0,
                entity.server_pos_fixed.2 as f64 / 32.0,
            );
            let yaw = (yaw_byte as f32 * 360.0) / 256.0;
            let pitch = (pitch_byte as f32 * 360.0) / 256.0;

            entity.target_pos = target;
            entity.target_yaw = yaw;
            entity.target_pitch = pitch;
            entity.interpolation_steps = 3;
            entity.on_ground = on_ground;
        }
    }

    pub fn teleport(
        &mut self,
        entity_id: i32,
        fixed_x: i32,
        fixed_y: i32,
        fixed_z: i32,
        yaw_byte: i8,
        pitch_byte: i8,
        on_ground: bool,
    ) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.server_pos_fixed = (fixed_x, fixed_y, fixed_z);
            let target = DVec3::new(
                fixed_x as f64 / 32.0,
                fixed_y as f64 / 32.0,
                fixed_z as f64 / 32.0,
            );
            let yaw = (yaw_byte as f32 * 360.0) / 256.0;
            let pitch = (pitch_byte as f32 * 360.0) / 256.0;

            let dist_sq = (entity.pos - target).length_squared();
            if dist_sq > 16.0 {
                // Large jump / instant teleport
                entity.pos = target;
                entity.prev_pos = target;
                entity.target_pos = target;
                entity.yaw = yaw;
                entity.prev_yaw = yaw;
                entity.target_yaw = yaw;
                entity.pitch = pitch;
                entity.prev_pitch = pitch;
                entity.target_pitch = pitch;
                entity.interpolation_steps = 0;
            } else {
                entity.target_pos = target;
                entity.target_yaw = yaw;
                entity.target_pitch = pitch;
                entity.interpolation_steps = 3;
            }
            entity.on_ground = on_ground;
        }
    }

    pub fn head_look(&mut self, entity_id: i32, head_yaw_byte: i8) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            let head_yaw = (head_yaw_byte as f32 * 360.0) / 256.0;
            entity.target_head_yaw = head_yaw;
            if entity.interpolation_steps == 0 {
                entity.head_yaw = head_yaw;
                entity.prev_head_yaw = head_yaw;
            }
        }
    }

    pub fn velocity(&mut self, entity_id: i32, vel_x: i16, vel_y: i16, vel_z: i16) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.velocity = Vec3::new(
                vel_x as f32 / 8000.0,
                vel_y as f32 / 8000.0,
                vel_z as f32 / 8000.0,
            );
        }
    }

    pub fn animation(&mut self, entity_id: i32, animation_type: u8) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            if animation_type == 0 {
                // Arm swing
                entity.is_swinging = true;
                entity.swing_ticks = 0;
                entity.swing_progress = 0.0;
            }
        }
    }

    pub fn equipment(&mut self, entity_id: i32, slot: i16, item: Option<Slot>) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            if (0..5).contains(&slot) {
                entity.equipment[slot as usize] = item.clone();
                if slot == 0 {
                    if let EntityKind::Player { held_item, .. } = &mut entity.kind {
                        *held_item = item.map_or(0, |s| s.item_id);
                    }
                }
            }
        }
    }

    pub fn update_metadata(&mut self, entity_id: i32, new_metadata: Vec<MetadataEntry>) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            for new_entry in new_metadata {
                if let Some(existing) = entity
                    .metadata
                    .iter_mut()
                    .find(|e| e.index == new_entry.index)
                {
                    *existing = new_entry;
                } else {
                    entity.metadata.push(new_entry);
                }
            }
            entity.apply_metadata_flags();
        }
    }

    /// Advances simulation one tick across all tracked entities.
    pub fn tick(&mut self) {
        for entity in self.entities.values_mut() {
            entity.tick();
        }
    }
}

/// Helper function to wrap angle differences into [-180.0, 180.0).
#[inline]
pub fn wrap_degrees(mut deg: f32) -> f32 {
    deg %= 360.0;
    if deg >= 180.0 {
        deg -= 360.0;
    }
    if deg < -180.0 {
        deg += 360.0;
    }
    deg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_tracker_spawn_and_destroy() {
        let mut tracker = EntityTracker::new();
        assert_eq!(tracker.count(), 0);

        let uuid = Uuid::new_v4();
        tracker.spawn_player(
            1,
            uuid,
            "Steve".into(),
            3200, // 100.0
            2048, // 64.0
            -1600, // -50.0
            0,
            0,
            276,
            vec![],
        );

        assert_eq!(tracker.count(), 1);
        let player = tracker.get(1).unwrap();
        assert_eq!(player.pos, DVec3::new(100.0, 64.0, -50.0));
        assert!(player.kind.is_player());

        tracker.destroy_entities(&[1]);
        assert_eq!(tracker.count(), 0);
    }

    #[test]
    fn test_entity_interpolation_and_tick() {
        let mut tracker = EntityTracker::new();
        tracker.spawn_mob(
            2,
            54,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            vec![],
        );

        // Relative move by 32 units (+1 block) on X
        tracker.rel_move(2, 32, 0, 0, true);
        let entity = tracker.get(2).unwrap();
        assert_eq!(entity.target_pos.x, 1.0);
        assert_eq!(entity.interpolation_steps, 3);

        // Tick 1
        tracker.tick();
        let entity = tracker.get(2).unwrap();
        assert!((entity.pos.x - 0.333).abs() < 0.01);
        assert_eq!(entity.interpolation_steps, 2);

        // Tick 2
        tracker.tick();
        let entity = tracker.get(2).unwrap();
        assert!((entity.pos.x - 0.666).abs() < 0.01);
        assert_eq!(entity.interpolation_steps, 1);

        // Tick 3
        tracker.tick();
        let entity = tracker.get(2).unwrap();
        assert!((entity.pos.x - 1.0).abs() < 0.01);
        assert_eq!(entity.interpolation_steps, 0);
    }
}
