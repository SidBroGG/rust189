use std::collections::VecDeque;
use std::time::Instant;

/// Checks if an attack qualifies as a Critical Hit according to MCP 1.8.9 EntityPlayer:
/// `this.fallDistance > 0.0F && !this.onGround && !this.isOnLadder() && !this.isInWater() && !this.isPotionActive(Potion.blindness) && this.ridingEntity == null`
#[inline]
pub fn is_critical_hit(
    fall_distance: f32,
    on_ground: bool,
    on_ladder: bool,
    in_water: bool,
    is_riding: bool,
    has_blindness: bool,
) -> bool {
    fall_distance > 0.0 && !on_ground && !on_ladder && !in_water && !is_riding && !has_blindness
}

/// Computes base attack damage for equipped items in 1.8.9.
pub fn get_weapon_damage(item_id: i16) -> f32 {
    match item_id {
        276 => 7.0, // Diamond Sword
        267 => 6.0, // Iron Sword
        272 => 5.0, // Stone Sword
        268 | 283 => 4.0, // Wood/Gold Sword
        279 => 6.0, // Diamond Axe
        258 => 5.0, // Iron Axe
        275 => 4.0, // Stone Axe
        271 | 286 => 3.0, // Wood/Gold Axe
        _ => 1.0, // Fist / default
    }
}

/// High-performance Click Tracker measuring 1.8.9 Left and Right Click Registration and CPS.
#[derive(Debug, Clone)]
pub struct ClickTracker {
    recent_left_clicks: VecDeque<Instant>,
    recent_right_clicks: VecDeque<Instant>,
}

impl Default for ClickTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickTracker {
    pub fn new() -> Self {
        Self {
            recent_left_clicks: VecDeque::with_capacity(64),
            recent_right_clicks: VecDeque::with_capacity(64),
        }
    }

    /// Registers a Left Click event without any 1.9 cooldown delay.
    pub fn register_left_click(&mut self) -> usize {
        let now = Instant::now();
        self.recent_left_clicks.push_back(now);
        self.cleanup();
        self.recent_left_clicks.len()
    }

    /// Registers a Right Click event.
    pub fn register_right_click(&mut self) -> usize {
        let now = Instant::now();
        self.recent_right_clicks.push_back(now);
        self.cleanup();
        self.recent_right_clicks.len()
    }

    /// Current Left Clicks Per Second (CPS).
    pub fn left_cps(&mut self) -> usize {
        self.cleanup();
        self.recent_left_clicks.len()
    }

    /// Current Right Clicks Per Second (CPS).
    pub fn right_cps(&mut self) -> usize {
        self.cleanup();
        self.recent_right_clicks.len()
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        while let Some(&t) = self.recent_left_clicks.front() {
            if now.duration_since(t).as_secs_f32() > 1.0 {
                self.recent_left_clicks.pop_front();
            } else {
                break;
            }
        }
        while let Some(&t) = self.recent_right_clicks.front() {
            if now.duration_since(t).as_secs_f32() > 1.0 {
                self.recent_right_clicks.pop_front();
            } else {
                break;
            }
        }
    }
}
