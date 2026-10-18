/// State of using an item in the right hand (MCP 1.8.9 ItemInUse / EnumAction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemUseAction {
    None,
    Block, // 1.8 Sword Blocking
    Bow,   // Bow pulling
    Eat,   // Food consumption
    Drink, // Potion drinking
}

/// Manages right-hand item usage timers, animations, and speed modifiers.
#[derive(Debug, Clone)]
pub struct ItemUseState {
    pub active_action: ItemUseAction,
    pub item_id: i16,
    pub ticks_used: u32,
    pub max_duration: u32,
}

impl Default for ItemUseState {
    fn default() -> Self {
        Self {
            active_action: ItemUseAction::None,
            item_id: 0,
            ticks_used: 0,
            max_duration: 0,
        }
    }
}

impl ItemUseState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Determines the action when right-clicking with an item.
    pub fn get_action_for_item(item_id: i16) -> ItemUseAction {
        match item_id {
            // Swords (1.8 Sword Blocking)
            267 | 268 | 272 | 276 | 283 => ItemUseAction::Block,
            // Bow
            261 => ItemUseAction::Bow,
            // Food items
            260 | 282 | 297 | 319 | 320 | 322 | 349 | 350 | 357 | 360 | 363 | 364 | 365 | 366 | 367 | 375 | 391 | 392 | 393 | 396 | 400 => {
                ItemUseAction::Eat
            }
            // Potions, Milk bucket
            373 | 335 => ItemUseAction::Drink,
            _ => ItemUseAction::None,
        }
    }

    /// Starts using an item.
    pub fn start_using(&mut self, item_id: i16) {
        let action = Self::get_action_for_item(item_id);
        if action == ItemUseAction::None {
            self.stop();
            return;
        }

        self.active_action = action;
        self.item_id = item_id;
        self.ticks_used = 0;
        self.max_duration = match action {
            ItemUseAction::Block => 72000,
            ItemUseAction::Bow => 72000,
            ItemUseAction::Eat | ItemUseAction::Drink => 32, // 32 ticks = 1.6s
            ItemUseAction::None => 0,
        };
    }

    /// Advances 1 physical tick (20 TPS). Returns true if action completed (e.g. food eaten).
    pub fn tick(&mut self) -> bool {
        if self.active_action == ItemUseAction::None {
            return false;
        }
        self.ticks_used += 1;
        if self.max_duration > 0 && self.ticks_used >= self.max_duration {
            // Finished using (e.g. food/potion finished)
            true
        } else {
            false
        }
    }

    /// Stops using current item.
    pub fn stop(&mut self) {
        self.active_action = ItemUseAction::None;
        self.item_id = 0;
        self.ticks_used = 0;
        self.max_duration = 0;
    }

    /// Whether the player is currently blocking with a sword (1.8 PvP Sword Blocking).
    #[inline]
    pub fn is_blocking(&self) -> bool {
        self.active_action == ItemUseAction::Block
    }

    /// Whether the player is drawing a bow.
    #[inline]
    pub fn is_bow_drawing(&self) -> bool {
        self.active_action == ItemUseAction::Bow
    }

    /// Bow pull progress stage: 0 (< 10 ticks), 1 (10..17 ticks), 2 (>= 18 ticks).
    pub fn bow_pull_stage(&self) -> u8 {
        if !self.is_bow_drawing() {
            0
        } else if self.ticks_used >= 18 {
            2
        } else if self.ticks_used >= 10 {
            1
        } else {
            0
        }
    }

    /// Calculates arrow release velocity and critical flag according to MCP ItemBow:
    /// `float f = (float)i / 20.0F; f = (f * f + f * 2.0F) / 3.0F; if (f > 1.0F) f = 1.0F;`
    pub fn release_bow(&mut self) -> Option<(f32, bool)> {
        if !self.is_bow_drawing() {
            return None;
        }
        let ticks = self.ticks_used as f32;
        self.stop();

        let mut f = ticks / 20.0;
        f = (f * f + f * 2.0) / 3.0;
        if f < 0.1 {
            return None; // Not pulled enough
        }
        let is_crit = f >= 1.0;
        let velocity = f.min(1.0) * 2.0;
        Some((velocity, is_crit))
    }

    /// Movement speed multiplier when item is in use (0.2 in MCP 1.8.9).
    #[inline]
    pub fn movement_speed_multiplier(&self) -> f32 {
        if self.active_action != ItemUseAction::None {
            0.2
        } else {
            1.0
        }
    }
}
