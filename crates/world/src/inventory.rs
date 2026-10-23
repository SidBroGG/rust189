use crate::item::{get_item_def, get_item_max_stack};
use bytes::Buf;
use protocol::codec::slot::read_slot;
use protocol::Slot;
use std::collections::HashMap;

/// Represents a single trading offer from a villager merchant.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MerchantTradeRecipe {
    pub buy1: Option<Slot>,
    pub buy2: Option<Slot>,
    pub sell: Option<Slot>,
    pub disabled: bool,
    pub uses: i32,
    pub max_uses: i32,
}

impl MerchantTradeRecipe {
    /// Decodes a list of villager trades from an `MC|TrList` custom payload.
    pub fn decode_tr_list(bytes: &[u8]) -> Result<(i32, Vec<Self>), protocol::ProtocolError> {
        let mut buf = bytes;
        if buf.remaining() < 4 {
            return Err(protocol::ProtocolError::UnexpectedEof);
        }
        let window_id = buf.get_i32();
        if buf.remaining() < 1 {
            return Err(protocol::ProtocolError::UnexpectedEof);
        }
        let count = buf.get_u8() as usize;
        let mut recipes = Vec::with_capacity(count);
        for _ in 0..count {
            let buy1 = read_slot(&mut buf)?;
            let sell = read_slot(&mut buf)?;
            if buf.remaining() < 1 {
                return Err(protocol::ProtocolError::UnexpectedEof);
            }
            let has_buy2 = buf.get_u8() != 0;
            let buy2 = if has_buy2 {
                read_slot(&mut buf)?
            } else {
                None
            };
            if buf.remaining() < 1 + 4 + 4 {
                return Err(protocol::ProtocolError::UnexpectedEof);
            }
            let disabled = buf.get_u8() != 0;
            let uses = buf.get_i32();
            let max_uses = buf.get_i32();
            recipes.push(MerchantTradeRecipe {
                buy1,
                buy2,
                sell,
                disabled,
                uses,
                max_uses,
            });
        }
        Ok((window_id, recipes))
    }
}

/// Types of containers supported by Protocol 47.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerKind {
    Player,
    Chest { rows: u8 },
    EnderChest,
    Workbench,
    Furnace,
    Dispenser,
    Dropper,
    Hopper,
    Anvil,
    EnchantingTable,
    BrewingStand,
    Villager,
    Beacon,
    Horse,
}

impl ContainerKind {
    pub fn from_inventory_type(gui_id: &str, slot_count: u8) -> Self {
        match gui_id {
            "minecraft:chest" | "minecraft:container" => {
                let rows = ((slot_count as usize) / 9).clamp(1, 6) as u8;
                Self::Chest { rows }
            }
            "minecraft:crafting_table" => Self::Workbench,
            "minecraft:furnace" => Self::Furnace,
            "minecraft:dispenser" => Self::Dispenser,
            "minecraft:dropper" => Self::Dropper,
            "minecraft:hopper" => Self::Hopper,
            "minecraft:anvil" => Self::Anvil,
            "minecraft:enchanting_table" => Self::EnchantingTable,
            "minecraft:brewing_stand" => Self::BrewingStand,
            "minecraft:villager" => Self::Villager,
            "minecraft:beacon" => Self::Beacon,
            "EntityHorse" => Self::Horse,
            _ => {
                let rows = ((slot_count as usize) / 9).clamp(1, 6) as u8;
                Self::Chest { rows }
            }
        }
    }

    /// Total number of slots in the container window (container-specific + 36 player slots).
    pub fn total_slot_count(&self) -> usize {
        match self {
            Self::Player => 45,       // 1 result + 4 craft + 4 armor + 27 main + 9 hotbar
            Self::Chest { rows } => (*rows as usize) * 9 + 36,
            Self::EnderChest => 27 + 36,  // 63
            Self::Workbench => 1 + 9 + 36, // 46
            Self::Furnace => 3 + 36,      // 39
            Self::Dispenser | Self::Dropper => 9 + 36, // 45
            Self::Hopper => 5 + 36,       // 41
            Self::Anvil => 3 + 36,        // 39
            Self::EnchantingTable => 2 + 36, // 38
            Self::BrewingStand => 4 + 36, // 40
            Self::Villager => 3 + 36,     // 39
            Self::Beacon => 1 + 36,       // 37
            Self::Horse => 2 + 15 + 36,   // 53
        }
    }

    /// The index where player's 36 slots (27 main + 9 hotbar) begin.
    pub fn player_slots_start(&self) -> usize {
        match self {
            Self::Player => 9,
            Self::Chest { rows } => (*rows as usize) * 9,
            Self::EnderChest => 27,
            Self::Workbench => 10,
            Self::Furnace => 3,
            Self::Dispenser | Self::Dropper => 9,
            Self::Hopper => 5,
            Self::Anvil => 3,
            Self::EnchantingTable => 2,
            Self::BrewingStand => 4,
            Self::Villager => 3,
            Self::Beacon => 1,
            Self::Horse => 17,
        }
    }
}

/// Represents an active window / container on the client.
#[derive(Debug, Clone)]
pub struct ContainerState {
    pub window_id: u8,
    pub kind: ContainerKind,
    pub title: String,
    pub slots: Vec<Option<Slot>>,
    pub carried_item: Option<Slot>,
    pub properties: HashMap<i16, i16>,
    pub action_counter: i16,
    pub is_dirty: bool,
}

impl ContainerState {
    /// Creates the standard player inventory container (window ID 0).
    pub fn new_player_inventory() -> Self {
        Self {
            window_id: 0,
            kind: ContainerKind::Player,
            title: "Инвентарь".to_string(),
            slots: vec![None; 45],
            carried_item: None,
            properties: HashMap::new(),
            action_counter: 0,
            is_dirty: false,
        }
    }

    /// Creates a container from an OpenWindow packet.
    pub fn from_open_window(
        window_id: u8,
        gui_id: &str,
        title: String,
        slot_count: u8,
    ) -> Self {
        let kind = ContainerKind::from_inventory_type(gui_id, slot_count);
        let count = kind.total_slot_count();
        let parsed_title = crate::chat::parse_chat_message(&title);
        let clean_title = crate::chat::strip_formatting(&parsed_title);
        let display_title = if clean_title.trim().is_empty() {
            title
        } else {
            clean_title
        };
        Self {
            window_id,
            kind,
            title: display_title,
            slots: vec![None; count],
            carried_item: None,
            properties: HashMap::new(),
            action_counter: 0,
            is_dirty: false,
        }
    }

    /// Gets next unique action number for transaction tracking.
    pub fn next_action_number(&mut self) -> i16 {
        self.action_counter = self.action_counter.wrapping_add(1);
        self.action_counter
    }

    /// Sets slot contents safely.
    pub fn set_slot(&mut self, slot_idx: usize, item: Option<Slot>) {
        if slot_idx < self.slots.len() {
            self.slots[slot_idx] = item;
            self.is_dirty = true;
            if self.kind == ContainerKind::Player || self.kind == ContainerKind::Workbench {
                self.update_crafting_result();
            }
        }
    }

    /// Copies player inventory slots (9..44) from player inventory into this container.
    pub fn sync_player_slots_from(&mut self, player_inv: &ContainerState) {
        let start = self.kind.player_slots_start();
        for i in 0..36 {
            let src_idx = 9 + i;
            let dst_idx = start + i;
            if src_idx < player_inv.slots.len() && dst_idx < self.slots.len() {
                self.slots[dst_idx] = player_inv.slots[src_idx].clone();
            }
        }
    }

    /// Copies player inventory slots back from this container to player inventory.
    pub fn sync_player_slots_to(&self, player_inv: &mut ContainerState) {
        let start = self.kind.player_slots_start();
        for i in 0..36 {
            let src_idx = start + i;
            let dst_idx = 9 + i;
            if src_idx < self.slots.len() && dst_idx < player_inv.slots.len() {
                player_inv.slots[dst_idx] = self.slots[src_idx].clone();
            }
        }
    }

    /// Handles a standard slot click (Mode 0: Normal LMB/RMB).
    pub fn handle_normal_click(&mut self, slot_idx: i16, is_right_click: bool) -> Option<Slot> {
        if slot_idx < 0 {
            // Click outside container: drop carried item
            if is_right_click {
                if let Some(mut carried) = self.carried_item.take() {
                    let dropped = carried.clone();
                    if carried.item_count > 1 {
                        carried.item_count -= 1;
                        self.carried_item = Some(carried);
                    }
                    return Some(Slot::new(dropped.item_id, 1, dropped.item_damage, dropped.nbt));
                }
            } else {
                return self.carried_item.take();
            }
            return None;
        }

        let idx = slot_idx as usize;
        if idx >= self.slots.len() {
            return None;
        }

        let slot_item = self.slots[idx].clone();

        // Special handling if clicking craft result slot (Slot 0)
        let is_craft_result = (self.kind == ContainerKind::Player || self.kind == ContainerKind::Workbench) && idx == 0;
        if is_craft_result {
            if let Some(result) = slot_item {
                let max_stack = get_item_max_stack(result.item_id) as i8;
                let can_take = match &self.carried_item {
                    None => true,
                    Some(carried) => {
                        carried.item_id == result.item_id
                            && carried.item_damage == result.item_damage
                            && carried.item_count + result.item_count <= max_stack
                    }
                };

                if can_take {
                    if let Some(ref mut carried) = self.carried_item {
                        carried.item_count += result.item_count;
                    } else {
                        self.carried_item = Some(result);
                    }
                    self.consume_crafting_ingredients();
                    self.update_crafting_result();
                }
            }
            return None;
        }

        let is_armor_slot = self.kind == ContainerKind::Player && (5..=8).contains(&idx);
        if is_armor_slot {
            if let Some(ref carried) = self.carried_item {
                let required_slot = (idx - 5) as u8;
                let def = get_item_def(carried.item_id);
                let is_valid = match def {
                    Some(d) => {
                        d.armor_slot == Some(required_slot)
                            || (required_slot == 0 && (carried.item_id == 86 || carried.item_id == 397))
                    }
                    None => false,
                };
                if !is_valid {
                    return None;
                }
            }
        }

        match (&mut self.carried_item, slot_item) {
            // Case 1: Cursor empty, Slot has item -> pick up
            (None, Some(item)) => {
                if is_right_click {
                    // Right click: take half
                    let take = (item.item_count + 1) / 2;
                    let remain = item.item_count - take;
                    self.carried_item = Some(Slot::new(item.item_id, take, item.item_damage, item.nbt.clone()));
                    if remain > 0 {
                        self.slots[idx] = Some(Slot::new(item.item_id, remain, item.item_damage, item.nbt));
                    } else {
                        self.slots[idx] = None;
                    }
                } else {
                    // Left click: take all
                    self.carried_item = Some(item);
                    self.slots[idx] = None;
                }
            }
            // Case 2: Cursor has item, Slot empty -> put down
            (Some(ref mut carried), None) => {
                let max_stack = if is_armor_slot {
                    1
                } else {
                    get_item_max_stack(carried.item_id) as i8
                };
                if is_right_click {
                    // Right click: put 1 item
                    self.slots[idx] = Some(Slot::new(carried.item_id, 1, carried.item_damage, carried.nbt.clone()));
                    carried.item_count -= 1;
                    if carried.item_count <= 0 {
                        self.carried_item = None;
                    }
                } else {
                    // Left click: put full stack (or up to max)
                    let put_count = carried.item_count.min(max_stack);
                    self.slots[idx] = Some(Slot::new(carried.item_id, put_count, carried.item_damage, carried.nbt.clone()));
                    carried.item_count -= put_count;
                    if carried.item_count <= 0 {
                        self.carried_item = None;
                    }
                }
            }
            // Case 3: Both cursor and slot have items
            (Some(ref mut carried), Some(mut item)) => {
                if carried.item_id == item.item_id && carried.item_damage == item.item_damage {
                    // Same item: merge
                    let max_stack = get_item_max_stack(carried.item_id) as i8;
                    let space = (max_stack - item.item_count).max(0);
                    if space > 0 {
                        if is_right_click {
                            item.item_count += 1;
                            self.slots[idx] = Some(item);
                            carried.item_count -= 1;
                            if carried.item_count <= 0 {
                                self.carried_item = None;
                            }
                        } else {
                            let add = carried.item_count.min(space);
                            item.item_count += add;
                            self.slots[idx] = Some(item);
                            carried.item_count -= add;
                            if carried.item_count <= 0 {
                                self.carried_item = None;
                            }
                        }
                    }
                } else if !is_right_click {
                    // Different item: swap stacks on left click
                    let old_carried = carried.clone();
                    *carried = item;
                    self.slots[idx] = Some(old_carried);
                }
            }
            (None, None) => {}
        }

        if self.kind == ContainerKind::Player || self.kind == ContainerKind::Workbench {
            self.update_crafting_result();
        }

        None
    }

    /// Handles Shift-click (Mode 1: Quick Move).
    pub fn handle_shift_click(&mut self, slot_idx: i16) {
        if slot_idx < 0 {
            return;
        }
        let idx = slot_idx as usize;
        if idx >= self.slots.len() {
            return;
        }

        let Some(mut item) = self.slots[idx].take() else {
            return;
        };

        // Special handling if shift-clicking craft result
        let is_craft_result = (self.kind == ContainerKind::Player || self.kind == ContainerKind::Workbench) && idx == 0;
        if is_craft_result {
            // Transfer craft result to player inventory
            let transferred = self.merge_into_range(&mut item, 9, 45);
            if transferred {
                self.consume_crafting_ingredients();
                self.update_crafting_result();
            } else {
                self.slots[0] = Some(item);
            }
            return;
        }

        let player_start = self.kind.player_slots_start();
        let total = self.slots.len();

        if idx < player_start {
            // From container -> To player inventory (main + hotbar)
            let _ = self.merge_into_range(&mut item, player_start, total);
        } else {
            // From player inventory:
            // Check if armor item and armor slots are available (Player inventory only)
            let mut armor_placed = false;
            if self.kind == ContainerKind::Player {
                if let Some(def) = get_item_def(item.item_id) {
                    if let Some(armor_slot) = def.armor_slot {
                        let target_slot = 5 + (armor_slot as usize);
                        if self.slots[target_slot].is_none() {
                            self.slots[target_slot] = Some(Slot::new(item.item_id, 1, item.item_damage, item.nbt.clone()));
                            item.item_count -= 1;
                            armor_placed = true;
                        }
                    } else if (item.item_id == 86 || item.item_id == 397) && self.slots[5].is_none() {
                        self.slots[5] = Some(Slot::new(item.item_id, 1, item.item_damage, item.nbt.clone()));
                        item.item_count -= 1;
                        armor_placed = true;
                    }
                }
            }

            if !armor_placed {
                if self.kind == ContainerKind::Player {
                    // Between main (9..36) and hotbar (36..45)
                    if idx >= 36 {
                        let _ = self.merge_into_range(&mut item, 9, 36);
                    } else {
                        let _ = self.merge_into_range(&mut item, 36, 45);
                    }
                } else {
                    // To container slots (0..player_start)
                    let _ = self.merge_into_range(&mut item, 0, player_start);
                }
            }
        }

        if item.item_count > 0 {
            self.slots[idx] = Some(item);
        }

        if self.kind == ContainerKind::Player || self.kind == ContainerKind::Workbench {
            self.update_crafting_result();
        }
    }

    /// Handles Hotbar key swap (Mode 2: 1-9 keys).
    pub fn handle_hotbar_swap(&mut self, slot_idx: i16, hotbar_key: u8) {
        if slot_idx < 0 || hotbar_key > 8 {
            return;
        }
        let idx = slot_idx as usize;
        let player_start = self.kind.player_slots_start();
        let hotbar_idx = player_start + 27 + (hotbar_key as usize);

        if idx < self.slots.len() && hotbar_idx < self.slots.len() {
            let a = self.slots[idx].take();
            let b = self.slots[hotbar_idx].take();
            self.slots[idx] = b;
            self.slots[hotbar_idx] = a;

            if self.kind == ContainerKind::Player || self.kind == ContainerKind::Workbench {
                self.update_crafting_result();
            }
        }
    }

    /// Handles Drop key (Mode 4: Q or Ctrl+Q).
    pub fn handle_drop(&mut self, slot_idx: i16, drop_all: bool) -> Option<Slot> {
        if slot_idx < 0 {
            return None;
        }
        let idx = slot_idx as usize;
        if idx >= self.slots.len() {
            return None;
        }

        if let Some(ref mut item) = self.slots[idx] {
            if drop_all || item.item_count <= 1 {
                return self.slots[idx].take();
            } else {
                item.item_count -= 1;
                return Some(Slot::new(item.item_id, 1, item.item_damage, item.nbt.clone()));
            }
        }
        None
    }

    /// Handles Double Click (Mode 6: collect matching items into cursor stack).
    pub fn handle_double_click(&mut self, _slot_idx: i16) {
        let Some(ref mut carried) = self.carried_item else {
            return;
        };
        let max_stack = get_item_max_stack(carried.item_id) as i8;
        if carried.item_count >= max_stack {
            return;
        }

        for slot in self.slots.iter_mut() {
            if let Some(ref mut item) = slot {
                if item.item_id == carried.item_id && item.item_damage == carried.item_damage {
                    let needed = max_stack - carried.item_count;
                    let take = item.item_count.min(needed);
                    carried.item_count += take;
                    item.item_count -= take;
                    if item.item_count <= 0 {
                        *slot = None;
                    }
                    if carried.item_count >= max_stack {
                        break;
                    }
                }
            }
        }
    }

    /// Handles Drag Finish (Mode 5: 0 = even split, 1 = 1 item per slot).
    pub fn handle_drag_finish(&mut self, drag_mode: u8, drag_slots: &[usize]) {
        let Some(ref mut carried) = self.carried_item else {
            return;
        };
        if drag_slots.is_empty() {
            return;
        }

        let max_stack = get_item_max_stack(carried.item_id) as i8;

        if drag_mode == 0 {
            // Even split
            let count_per_slot = carried.item_count / drag_slots.len() as i8;
            if count_per_slot > 0 {
                let mut placed_total = 0;
                for &idx in drag_slots {
                    if idx < self.slots.len() {
                        let space = match &self.slots[idx] {
                            None => max_stack,
                            Some(s) if s.item_id == carried.item_id && s.item_damage == carried.item_damage => {
                                (max_stack - s.item_count).max(0)
                            }
                            _ => 0,
                        };
                        let to_add = count_per_slot.min(space);
                        if to_add > 0 {
                            if let Some(ref mut s) = self.slots[idx] {
                                s.item_count += to_add;
                            } else {
                                self.slots[idx] = Some(Slot::new(
                                    carried.item_id,
                                    to_add,
                                    carried.item_damage,
                                    carried.nbt.clone(),
                                ));
                            }
                            placed_total += to_add;
                        }
                    }
                }
                carried.item_count -= placed_total;
                if carried.item_count <= 0 {
                    self.carried_item = None;
                }
            }
        } else if drag_mode == 1 {
            // 1 item per slot
            for &idx in drag_slots {
                if carried.item_count <= 0 {
                    break;
                }
                if idx < self.slots.len() {
                    let can_place = match &self.slots[idx] {
                        None => true,
                        Some(s) => {
                            s.item_id == carried.item_id
                                && s.item_damage == carried.item_damage
                                && s.item_count < max_stack
                        }
                    };
                    if can_place {
                        if let Some(ref mut s) = self.slots[idx] {
                            s.item_count += 1;
                        } else {
                            self.slots[idx] = Some(Slot::new(
                                carried.item_id,
                                1,
                                carried.item_damage,
                                carried.nbt.clone(),
                            ));
                        }
                        carried.item_count -= 1;
                    }
                }
            }
            if carried.item_count <= 0 {
                self.carried_item = None;
            }
        }

        if self.kind == ContainerKind::Player || self.kind == ContainerKind::Workbench {
            self.update_crafting_result();
        }
    }

    /// Mouse Tweaks scroll wheel: Up = pull 1 item from other container, Down = push 1 item to other container.
    pub fn handle_wheel_scroll(&mut self, slot_idx: usize, is_up: bool) -> bool {
        if slot_idx >= self.slots.len() {
            return false;
        }
        let player_start = self.kind.player_slots_start();
        let total = self.slots.len();

        if is_up {
            let target_item = self.slots[slot_idx].clone();
            let (target_id, target_dmg) = match target_item {
                Some(ref s) => (s.item_id, s.item_damage),
                None => {
                    if let Some(ref c) = self.carried_item {
                        (c.item_id, c.item_damage)
                    } else {
                        return false;
                    }
                }
            };

            let max_stack = get_item_max_stack(target_id) as i8;
            if let Some(ref s) = self.slots[slot_idx] {
                if s.item_count >= max_stack {
                    return false;
                }
            }

            let (other_start, other_end) = if slot_idx < player_start {
                (player_start, total)
            } else {
                (0, player_start)
            };

            for i in other_start..other_end.min(self.slots.len()) {
                if let Some(ref mut src) = self.slots[i] {
                    if src.item_id == target_id && src.item_damage == target_dmg && src.item_count > 0 {
                        src.item_count -= 1;
                        let src_nbt = src.nbt.clone();
                        if src.item_count <= 0 {
                            self.slots[i] = None;
                        }

                        if let Some(ref mut dst) = self.slots[slot_idx] {
                            dst.item_count += 1;
                        } else {
                            self.slots[slot_idx] = Some(Slot::new(target_id, 1, target_dmg, src_nbt));
                        }
                        if self.kind == ContainerKind::Player || self.kind == ContainerKind::Workbench {
                            self.update_crafting_result();
                        }
                        return true;
                    }
                }
            }
        } else if let Some(ref s) = self.slots[slot_idx] {
            if s.item_count > 0 {
                let mut single = Slot::new(s.item_id, 1, s.item_damage, s.nbt.clone());
                let (other_start, other_end) = if slot_idx < player_start {
                    (player_start, total)
                } else {
                    (0, player_start)
                };

                let transferred = self.merge_into_range(&mut single, other_start, other_end);
                if transferred || single.item_count == 0 {
                    if let Some(ref mut cur) = self.slots[slot_idx] {
                        cur.item_count -= 1;
                        if cur.item_count <= 0 {
                            self.slots[slot_idx] = None;
                        }
                    }
                    if self.kind == ContainerKind::Player || self.kind == ContainerKind::Workbench {
                        self.update_crafting_result();
                    }
                    return true;
                }
            }
        }
        false
    }

    /// Merges an item into a range of slots [start..end).
    fn merge_into_range(&mut self, item: &mut Slot, start: usize, end: usize) -> bool {
        let max_stack = get_item_max_stack(item.item_id) as i8;
        let mut modified = false;

        // Pass 1: Try stacking into existing non-empty matching slots
        for i in start..end.min(self.slots.len()) {
            if let Some(ref mut target) = self.slots[i] {
                if target.item_id == item.item_id && target.item_damage == item.item_damage {
                    let space = (max_stack - target.item_count).max(0);
                    if space > 0 {
                        let add = item.item_count.min(space);
                        target.item_count += add;
                        item.item_count -= add;
                        modified = true;
                        if item.item_count <= 0 {
                            return true;
                        }
                    }
                }
            }
        }

        // Pass 2: Place into first empty slot
        for i in start..end.min(self.slots.len()) {
            if self.slots[i].is_none() {
                self.slots[i] = Some(item.clone());
                item.item_count = 0;
                return true;
            }
        }

        modified
    }

    /// Consumes 1 item from each non-empty crafting matrix slot.
    fn consume_crafting_ingredients(&mut self) {
        let (start, end) = match self.kind {
            ContainerKind::Player => (1, 5),
            ContainerKind::Workbench => (1, 10),
            _ => return,
        };

        for i in start..end {
            if let Some(ref mut item) = self.slots[i] {
                item.item_count -= 1;
                if item.item_count <= 0 {
                    self.slots[i] = None;
                }
            }
        }
    }

    /// Re-evaluates crafting matrix and updates Slot 0 (result).
    pub fn update_crafting_result(&mut self) {
        match self.kind {
            ContainerKind::Player => {
                let grid = [
                    self.slots[1].as_ref(),
                    self.slots[2].as_ref(),
                    self.slots[3].as_ref(),
                    self.slots[4].as_ref(),
                ];
                self.slots[0] = match_crafting_2x2(&grid);
            }
            ContainerKind::Workbench => {
                let grid = [
                    self.slots[1].as_ref(),
                    self.slots[2].as_ref(),
                    self.slots[3].as_ref(),
                    self.slots[4].as_ref(),
                    self.slots[5].as_ref(),
                    self.slots[6].as_ref(),
                    self.slots[7].as_ref(),
                    self.slots[8].as_ref(),
                    self.slots[9].as_ref(),
                ];
                self.slots[0] = match_crafting_3x3(&grid);
            }
            _ => {}
        }
    }
}

/// Matches a 2x2 crafting grid and returns the crafting recipe result.
pub fn match_crafting_2x2(grid: &[Option<&Slot>; 4]) -> Option<Slot> {
    let items: Vec<(usize, i16, i16)> = grid
        .iter()
        .enumerate()
        .filter_map(|(idx, s)| s.map(|slot| (idx, slot.item_id, slot.item_damage)))
        .collect();

    // 1. Single Wood Log -> 4 Planks
    if items.len() == 1 {
        let (_, id, damage) = items[0];
        if id == 17 || id == 162 {
            let plank_meta = if id == 162 { (damage & 1) + 4 } else { damage & 3 };
            return Some(Slot::new(5, 4, plank_meta, None));
        }
        // Planks -> Button
        if id == 5 {
            // Single plank anywhere can produce button, or can produce 4 sticks if 2 planks vertical
        }
    }

    // 2. 2 Wood Planks vertical -> 4 Sticks
    if items.len() == 2 {
        let (idx1, id1, _) = items[0];
        let (idx2, id2, _) = items[1];
        if id1 == 5 && id2 == 5 {
            // (0, 2) or (1, 3) are vertical pairs in 2x2 grid
            if (idx1 == 0 && idx2 == 2) || (idx1 == 1 && idx2 == 3) {
                return Some(Slot::new(280, 4, 0, None));
            }
        }
    }

    // 3. 4 Wood Planks -> Crafting Table (58)
    if items.len() == 4 {
        if items.iter().all(|(_, id, _)| *id == 5) {
            return Some(Slot::new(58, 1, 0, None));
        }
    }

    // 4. Coal + Stick vertical -> 4 Torches
    if items.len() == 2 {
        let (idx1, id1, _) = items[0];
        let (idx2, id2, _) = items[1];
        if (id1 == 263 && id2 == 280 && idx1 + 2 == idx2) || (id2 == 263 && id1 == 280 && idx2 + 2 == idx1) {
            return Some(Slot::new(50, 4, 0, None));
        }
    }

    None
}

/// Matches a 3x3 crafting grid and returns the crafting recipe result.
pub fn match_crafting_3x3(grid: &[Option<&Slot>; 9]) -> Option<Slot> {
    // First, check if 2x2 recipe matches within 3x3
    let non_empty: Vec<(usize, i16, i16)> = grid
        .iter()
        .enumerate()
        .filter_map(|(idx, s)| s.map(|slot| (idx, slot.item_id, slot.item_damage)))
        .collect();

    if non_empty.is_empty() {
        return None;
    }

    // 1. Single Wood Log -> 4 Planks
    if non_empty.len() == 1 {
        let (_, id, damage) = non_empty[0];
        if id == 17 || id == 162 {
            let plank_meta = if id == 162 { (damage & 1) + 4 } else { damage & 3 };
            return Some(Slot::new(5, 4, plank_meta, None));
        }
        if id == 41 { // Gold block -> 9 ingots
            return Some(Slot::new(266, 9, 0, None));
        }
        if id == 42 { // Iron block -> 9 ingots
            return Some(Slot::new(265, 9, 0, None));
        }
        if id == 57 { // Diamond block -> 9 diamonds
            return Some(Slot::new(264, 9, 0, None));
        }
        if id == 133 { // Emerald block -> 9 emeralds
            return Some(Slot::new(388, 9, 0, None));
        }
        if id == 152 { // Redstone block -> 9 redstone
            return Some(Slot::new(331, 9, 0, None));
        }
        if id == 22 { // Lapis block -> 9 lapis
            return Some(Slot::new(351, 9, 4, None));
        }
    }

    // 2. 9 Ingots/Gems -> Block
    if non_empty.len() == 9 {
        let first_id = non_empty[0].1;
        if non_empty.iter().all(|(_, id, _)| *id == first_id) {
            match first_id {
                265 => return Some(Slot::new(42, 1, 0, None)), // Iron Block
                266 => return Some(Slot::new(41, 1, 0, None)), // Gold Block
                264 => return Some(Slot::new(57, 1, 0, None)), // Diamond Block
                388 => return Some(Slot::new(133, 1, 0, None)), // Emerald Block
                331 => return Some(Slot::new(152, 1, 0, None)), // Redstone Block
                263 => return Some(Slot::new(173, 1, 0, None)), // Coal Block
                _ => {}
            }
        }
    }

    // 3. 8 Cobblestone around hollow center -> Furnace (61)
    if non_empty.len() == 8 && grid[4].is_none() {
        if non_empty.iter().all(|(_, id, _)| *id == 4) {
            return Some(Slot::new(61, 1, 0, None));
        }
    }

    // 4. 8 Wood Planks around hollow center -> Chest (54)
    if non_empty.len() == 8 && grid[4].is_none() {
        if non_empty.iter().all(|(_, id, _)| *id == 5) {
            return Some(Slot::new(54, 1, 0, None));
        }
    }

    // 5. Tools & Weapons recipes (Swords, Pickaxes, Axes, Shovels)
    // Vertical line in column 0, 1, or 2
    for col in 0..3 {
        let top = col;
        let mid = col + 3;
        let bot = col + 6;

        // Sword: material, material, stick
        if let (Some(m1), Some(m2), Some(stk)) = (grid[top], grid[mid], grid[bot]) {
            if non_empty.len() == 3 && stk.item_id == 280 && m1.item_id == m2.item_id {
                let sword_id = match m1.item_id {
                    276 | 264 => 276, // Diamond
                    265 => 267,       // Iron
                    4 => 272,         // Stone
                    5 => 268,         // Wood
                    266 => 283,       // Gold
                    _ => 0,
                };
                if sword_id > 0 {
                    return Some(Slot::new(sword_id, 1, 0, None));
                }
            }
        }

        // Shovel: material, stick, stick
        if let (Some(m), Some(s1), Some(s2)) = (grid[top], grid[mid], grid[bot]) {
            if non_empty.len() == 3 && s1.item_id == 280 && s2.item_id == 280 {
                let shovel_id = match m.item_id {
                    264 => 277, // Diamond
                    265 => 256, // Iron
                    4 => 273,   // Stone
                    5 => 269,   // Wood
                    266 => 284, // Gold
                    _ => 0,
                };
                if shovel_id > 0 {
                    return Some(Slot::new(shovel_id, 1, 0, None));
                }
            }
        }
    }

    // Pickaxe: 3 material top row (0,1,2), sticks in (4, 7)
    if let (Some(m0), Some(m1), Some(m2), Some(s1), Some(s2)) =
        (grid[0], grid[1], grid[2], grid[4], grid[7])
    {
        if non_empty.len() == 5
            && s1.item_id == 280
            && s2.item_id == 280
            && m0.item_id == m1.item_id
            && m1.item_id == m2.item_id
        {
            let pick_id = match m0.item_id {
                264 => 278, // Diamond
                265 => 257, // Iron
                4 => 274,   // Stone
                5 => 270,   // Wood
                266 => 285, // Gold
                _ => 0,
            };
            if pick_id > 0 {
                return Some(Slot::new(pick_id, 1, 0, None));
            }
        }
    }

    // 6. Sticks from 2 vertical planks
    if non_empty.len() == 2 {
        let (idx1, id1, _) = non_empty[0];
        let (idx2, id2, _) = non_empty[1];
        if id1 == 5 && id2 == 5 && idx1 + 3 == idx2 {
            return Some(Slot::new(280, 4, 0, None));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_click_pickup_and_put() {
        let mut inv = ContainerState::new_player_inventory();
        inv.set_slot(36, Some(Slot::new(276, 1, 0, None))); // Diamond Sword in hotbar 0

        // 1. Left click picks up sword
        inv.handle_normal_click(36, false);
        assert!(inv.slots[36].is_none());
        assert_eq!(inv.carried_item.as_ref().map(|s| s.item_id), Some(276));

        // 2. Left click on slot 37 puts it down
        inv.handle_normal_click(37, false);
        assert_eq!(inv.slots[37].as_ref().map(|s| s.item_id), Some(276));
        assert!(inv.carried_item.is_none());
    }

    #[test]
    fn test_right_click_half_and_place_one() {
        let mut inv = ContainerState::new_player_inventory();
        inv.set_slot(36, Some(Slot::new(4, 10, 0, None))); // 10 Cobblestone

        // Right click takes half (5)
        inv.handle_normal_click(36, true);
        assert_eq!(inv.slots[36].as_ref().map(|s| s.item_count), Some(5));
        assert_eq!(inv.carried_item.as_ref().map(|s| s.item_count), Some(5));

        // Right click on empty slot places 1
        inv.handle_normal_click(37, true);
        assert_eq!(inv.slots[37].as_ref().map(|s| s.item_count), Some(1));
        assert_eq!(inv.carried_item.as_ref().map(|s| s.item_count), Some(4));
    }

    #[test]
    fn test_shift_click_transfer() {
        let mut inv = ContainerState::new_player_inventory();
        inv.set_slot(9, Some(Slot::new(1, 64, 0, None))); // Main slot 0

        // Shift click transfers from main (9) to hotbar (36..45)
        inv.handle_shift_click(9);
        assert!(inv.slots[9].is_none());
        assert_eq!(inv.slots[36].as_ref().map(|s| (s.item_id, s.item_count)), Some((1, 64)));

        // Shift click from hotbar transfers back to main
        inv.handle_shift_click(36);
        assert!(inv.slots[36].is_none());
        assert_eq!(inv.slots[9].as_ref().map(|s| (s.item_id, s.item_count)), Some((1, 64)));
    }

    #[test]
    fn test_hotbar_swap() {
        let mut inv = ContainerState::new_player_inventory();
        inv.set_slot(9, Some(Slot::new(276, 1, 0, None))); // Diamond Sword in main slot 9
        inv.set_slot(38, Some(Slot::new(261, 1, 0, None))); // Bow in hotbar slot 2 (slot 36 + 2 = 38)

        // Swap main slot 9 with hotbar key 2
        inv.handle_hotbar_swap(9, 2);
        assert_eq!(inv.slots[9].as_ref().map(|s| s.item_id), Some(261));
        assert_eq!(inv.slots[38].as_ref().map(|s| s.item_id), Some(276));
    }

    #[test]
    fn test_double_click_gather() {
        let mut inv = ContainerState::new_player_inventory();
        inv.set_slot(9, Some(Slot::new(4, 20, 0, None)));
        inv.set_slot(10, Some(Slot::new(4, 20, 0, None)));
        inv.carried_item = Some(Slot::new(4, 10, 0, None));

        inv.handle_double_click(9);
        // Carried should have gathered 20 + 20 + 10 = 50 cobblestone
        assert_eq!(inv.carried_item.as_ref().map(|s| s.item_count), Some(50));
        assert!(inv.slots[9].is_none());
        assert!(inv.slots[10].is_none());
    }

    #[test]
    fn test_crafting_2x2_log_to_planks_and_sticks() {
        let mut inv = ContainerState::new_player_inventory();
        inv.set_slot(1, Some(Slot::new(17, 2, 0, None))); // Oak log in crafting slot 1

        // Crafting result slot (0) should have 4 Oak Planks (5)
        assert_eq!(inv.slots[0].as_ref().map(|s| (s.item_id, s.item_count)), Some((5, 4)));

        // Take craft result
        inv.handle_normal_click(0, false);
        assert_eq!(inv.carried_item.as_ref().map(|s| (s.item_id, s.item_count)), Some((5, 4)));
        // Crafting slot 1 should have 1 log remaining
        assert_eq!(inv.slots[1].as_ref().map(|s| s.item_count), Some(1));
        // And result updated for remaining log
        assert_eq!(inv.slots[0].as_ref().map(|s| (s.item_id, s.item_count)), Some((5, 4)));
    }

    #[test]
    fn test_crafting_3x3_recipes() {
        let mut bench = ContainerState::from_open_window(1, "minecraft:crafting_table", "Crafting".into(), 0);
        // 3 Diamonds top row (1, 2, 3), Sticks in (5, 8)
        bench.set_slot(1, Some(Slot::new(264, 1, 0, None)));
        bench.set_slot(2, Some(Slot::new(264, 1, 0, None)));
        bench.set_slot(3, Some(Slot::new(264, 1, 0, None)));
        bench.set_slot(5, Some(Slot::new(280, 1, 0, None)));
        bench.set_slot(8, Some(Slot::new(280, 1, 0, None)));

        // Diamond Pickaxe (278) in result
        assert_eq!(bench.slots[0].as_ref().map(|s| s.item_id), Some(278));
    }

    #[test]
    fn test_custom_chest_sync_5_rows() {
        let mut player_inv = ContainerState::new_player_inventory();
        // Put Wooden Sword (268) in player hotbar slot 0 (index 36)
        player_inv.set_slot(36, Some(Slot::new(268, 1, 0, None)));

        // Open 45-slot (5-row) custom chest (e.g. Bedwars Shop)
        let mut shop = ContainerState::from_open_window(1, "minecraft:container", "Shop".into(), 45);
        assert_eq!(shop.kind, ContainerKind::Chest { rows: 5 });
        assert_eq!(shop.slots.len(), 45 + 36);

        // Sync player slots from player_inv into shop container
        shop.sync_player_slots_from(&player_inv);
        // Player hotbar in shop container is at index 45 + 27 = 72
        assert_eq!(shop.slots[72].as_ref().map(|s| s.item_id), Some(268));

        // Sync back to a fresh player inventory
        let mut target_inv = ContainerState::new_player_inventory();
        shop.sync_player_slots_to(&mut target_inv);

        // Slot 36 (hotbar 0) MUST have the Wooden Sword
        assert_eq!(target_inv.slots[36].as_ref().map(|s| s.item_id), Some(268));
        // Row 2 (slots 27..35) MUST be empty, NOT duplicated with hotbar!
        for slot_idx in 27..36 {
            assert!(
                target_inv.slots[slot_idx].is_none(),
                "Slot {} in pre-last row should be empty, not duplicate hotbar",
                slot_idx
            );
        }
    }
}

