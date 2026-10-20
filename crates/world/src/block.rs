use serde::{Deserialize, Serialize};

/// Represents a 16-bit block state combining a 12-bit block ID and 4-bit metadata.
/// Matching Protocol 47 raw format: `(id << 4) | (metadata & 0x0F)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct BlockState(pub u16);

impl BlockState {
    pub const AIR: Self = Self(0);

    #[inline]
    pub const fn new(id: u16, meta: u8) -> Self {
        Self(((id & 0x0FFF) << 4) | ((meta as u16) & 0x0F))
    }

    #[inline]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn id(self) -> u16 {
        self.0 >> 4
    }

    #[inline]
    pub const fn meta(self) -> u8 {
        (self.0 & 0x0F) as u8
    }

    #[inline]
    pub const fn raw(self) -> u16 {
        self.0
    }

    #[inline]
    pub const fn is_air(self) -> bool {
        self.id() == 0
    }

    #[inline]
    pub const fn is_opaque(self) -> bool {
        let id = self.id();
        !matches!(
            id,
            0 | 6 | 8 | 9 | 10 | 11 | 18 | 20 | 26 | 27 | 28 | 30 | 31 | 32 | 37 | 38 | 39 | 40
                | 44 | 50 | 51 | 52 | 53 | 54 | 55 | 59 | 60 | 63 | 64 | 65 | 66 | 67 | 68 | 69
                | 70 | 71 | 72 | 75 | 76 | 77 | 78 | 79 | 81 | 83 | 85 | 92 | 93 | 94 | 95 | 96
                | 101 | 102 | 104 | 105 | 106 | 107 | 108 | 109 | 111 | 113 | 114 | 115 | 116 | 117
                | 118 | 119 | 120 | 126 | 127 | 128 | 130 | 131 | 132 | 134 | 135 | 136 | 139 | 140
                | 141 | 142 | 143 | 144 | 145 | 146 | 147 | 148 | 149 | 150 | 151 | 154 | 156 | 157
                | 160 | 161 | 163 | 164 | 165 | 166 | 167 | 171 | 175 | 176 | 177 | 178 | 180 | 182
                | 183 | 184 | 185 | 186 | 187 | 188 | 189 | 190 | 191 | 192 | 193 | 194 | 195 | 196
                | 197
        )
    }
}

/// Checks if a block is replaceable when placing blocks into its space (MCP 1.8.9 Material.isReplaceable).
#[inline]
pub fn is_replaceable_block(block_id: u16) -> bool {
    matches!(block_id, 0 | 8 | 9 | 10 | 11 | 31 | 32 | 51 | 78 | 106 | 175)
}

/// Checks if a block can be activated/interacted with on right-click (MCP 1.8.9 Block.onBlockActivated).
#[inline]
pub fn is_interactive_block(block_id: u16) -> bool {
    matches!(
        block_id,
        // Wooden Doors
        64 | 193..=197
        // Wooden Trapdoor
        | 96
        // Fence gates
        | 107 | 183..=187
        // Redstone interaction: Lever, Stone/Wood Buttons, Repeater, Daylight Sensor
        | 69 | 77 | 93 | 94 | 143 | 151 | 178
        // Containers & Workstations: Dispenser, Note Block, Bed, Chest, Crafting Table, Furnace, Burning Furnace, Jukebox, Cake, Enchanting Table, Brewing Stand, Ender Chest, Command Block, Beacon, Anvil, Trapped Chest, Comparator, Powered Comparator, Hopper, Dropper
        | 23 | 25 | 26 | 54 | 58 | 61 | 62 | 84 | 92 | 116 | 117 | 130 | 137 | 138 | 145 | 146 | 149 | 150 | 154 | 158
    )
}

/// Checks if an item ID represents a placeable block (IDs 1..255 except non-placeable technical blocks).
#[inline]
pub fn is_placeable_block_item(id: u16) -> bool {
    id > 0
        && id < 256
        && !matches!(
            id,
            0 | 8 | 9 | 10 | 11 | 26 | 34 | 51 | 55 | 59 | 60 | 62 | 64 | 68 | 71 | 74 | 75 | 83 | 90 | 92 | 93 | 94 | 104 | 105 | 115 | 117 | 118 | 119 | 124 | 132 | 140 | 141 | 142 | 144 | 149 | 150 | 178 | 193..=197
        )
}

/// Returns the picked item (item_id, damage) for a block according to MCP 1.8.9 (Block.getItem and getDamageValue).
pub fn get_block_pick_item(block_id: u16, meta: u8) -> Option<(i16, i16)> {
    if block_id == 0 {
        return None;
    }
    match block_id {
        // Liquids cannot be picked
        8 | 9 | 10 | 11 => None,
        // Technical / special blocks
        26 => Some((355, 0)), // Bed block -> Bed item
        34 | 36 => None,      // Piston technical blocks
        51 => None,           // Fire
        55 => Some((331, 0)), // Redstone wire -> Redstone dust
        59 => Some((295, 0)), // Wheat crops -> Wheat seeds
        60 => Some((3, 0)),   // Farmland -> Dirt
        62 => Some((61, 0)),  // Lit furnace -> Furnace
        63 | 68 => Some((323, 0)), // Sign -> Sign item
        64 => Some((324, 0)), // Wooden door -> Wooden door item
        71 => Some((330, 0)), // Iron door -> Iron door item
        74 => Some((73, 0)),  // Lit redstone ore -> Redstone ore
        75 => Some((76, 0)),  // Unlit redstone torch -> Redstone torch
        83 => Some((338, 0)), // Reeds -> Sugar Canes item
        92 => Some((354, 0)), // Cake block -> Cake item
        93 | 94 => Some((356, 0)), // Redstone repeater -> Repeater item
        104 | 105 => Some((361, 0)), // Pumpkin / Melon stems -> Pumpkin seeds
        115 => Some((372, 0)), // Nether wart -> Nether wart item
        117 => Some((379, 0)), // Brewing stand -> Brewing stand item
        118 => Some((380, 0)), // Cauldron -> Cauldron item
        119 => None,           // End Portal
        124 => Some((123, 0)), // Lit redstone lamp -> Redstone lamp
        140 => Some((390, 0)), // Flower pot -> Flower pot item
        141 => Some((391, 0)), // Carrots -> Carrot item
        142 => Some((392, 0)), // Potatoes -> Potato item
        144 => Some((397, (meta & 7) as i16)), // Skull
        149 | 150 => Some((404, 0)), // Comparator -> Comparator item
        151 | 178 => Some((151, 0)), // Daylight detector
        176 | 177 => Some((425, (meta & 15) as i16)), // Standing / wall banner -> Banner
        193 => Some((427, 0)), // Spruce door
        194 => Some((428, 0)), // Birch door
        195 => Some((429, 0)), // Jungle door
        196 => Some((430, 0)), // Acacia door
        197 => Some((431, 0)), // Dark oak door
        // Double slabs map to single slabs
        43 => Some((44, (meta & 7) as i16)),
        125 => Some((126, (meta & 7) as i16)),
        181 => Some((182, (meta & 7) as i16)),
        // Subtype blocks (Stone, Dirt, Planks, Sandstone, Wool, Stained Clay, Stained Glass, Logs, Leaves, Stonebrick, etc.)
        1 | 3 | 5 | 6 | 12 | 17 | 18 | 19 | 24 | 31 | 35 | 38 | 44 | 95 | 97 | 98 | 126 | 155 | 159 | 160 | 161 | 162 | 168 | 171 | 175 | 179 | 182 => {
            Some((block_id as i16, meta as i16))
        }
        _ => Some((block_id as i16, 0)),
    }
}

/// Calculates the metadata for a newly placed block based on its ID, item damage,
/// target block face, cursor coordinates, and player view angles (MCP 1.8.9 Block.onBlockPlaced).
pub fn calculate_placed_block_meta(
    block_id: u16,
    item_damage: i16,
    face: u8,
    cursor_y: u8,
    player_yaw: f32,
    _player_pitch: f32,
) -> u8 {
    // Player horizontal facing index: 0 = South, 1 = West, 2 = North, 3 = East
    let yaw_idx = (((player_yaw * 4.0 / 360.0).round() as i32).rem_euclid(4)) as u8;

    match block_id {
        // End Portal Frame (120): Facing opposite of player, eye bit = 0
        120 => (yaw_idx + 2) % 4,

        // Ladder (65): Face attached (2 = North, 3 = South, 4 = West, 5 = East)
        65 => {
            if (2..=5).contains(&face) {
                face
            } else {
                match yaw_idx {
                    0 => 3, // South
                    1 => 4, // West
                    2 => 2, // North
                    _ => 5, // East
                }
            }
        }

        // Torch (50) and Redstone Torch (76)
        50 | 76 => match face {
            1 => 5, // Placed on top of block (floor torch)
            2 => 4, // Placed on North face of attached block (facing South)
            3 => 3, // Placed on South face of attached block (facing North)
            4 => 2, // Placed on West face of attached block (facing East)
            5 => 1, // Placed on East face of attached block (facing West)
            _ => 5, // Fallback floor
        },

        // Stairs (53, 67, 108, 109, 114, 128, 134, 135, 136, 156, 163, 164, 180)
        53 | 67 | 108 | 109 | 114 | 128 | 134 | 135 | 136 | 156 | 163 | 164 | 180 => {
            let facing_meta = match yaw_idx {
                0 => 2, // South
                1 => 1, // West
                2 => 3, // North
                _ => 0, // East
            };
            let upside_down = if face == 0 || (face != 1 && cursor_y > 8) {
                4
            } else {
                0
            };
            facing_meta | upside_down
        }

        // Wood Logs (17, 162): bits 0..1 wood variant, bits 2..3 axis
        17 | 162 => {
            let variant = (item_damage as u8) & 3;
            let axis = match face {
                0 | 1 => 0,  // Y-axis (vertical)
                2 | 3 => 8,  // Z-axis (north-south)
                4 | 5 => 4,  // X-axis (east-west)
                _ => 0,
            };
            variant | axis
        }

        // Single Slabs (44, 126): bit 3 is top slab
        44 | 126 => {
            let variant = (item_damage as u8) & 7;
            let top_bit = if face == 0 || (face != 1 && cursor_y > 8) {
                8
            } else {
                0
            };
            variant | top_bit
        }

        // Furnaces (61), Chests (54, 146), Dispensers (23), Droppers (158)
        61 | 54 | 146 | 23 | 158 => match yaw_idx {
            0 => 2, // Facing North (opposite of player facing South)
            1 => 5, // Facing East
            2 => 3, // Facing South
            _ => 4, // Facing West
        },

        // Hay block (170): axis
        170 => match face {
            0 | 1 => 0,
            2 | 3 => 8,
            4 | 5 => 4,
            _ => 0,
        },

        // Default: use item damage as metadata
        _ => (item_damage as u8) & 0x0F,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_state_packing() {
        let state = BlockState::new(1, 2); // Stone with variant 2 (Granite)
        assert_eq!(state.id(), 1);
        assert_eq!(state.meta(), 2);
        assert_eq!(state.raw(), (1 << 4) | 2);

        let state2 = BlockState::from_raw((123 << 4) | 15);
        assert_eq!(state2.id(), 123);
        assert_eq!(state2.meta(), 15);
    }

    #[test]
    fn test_replaceable_blocks() {
        assert!(is_replaceable_block(0));   // Air
        assert!(is_replaceable_block(31));  // Tall grass
        assert!(is_replaceable_block(78));  // Snow layer
        assert!(is_replaceable_block(106)); // Vine
        assert!(!is_replaceable_block(1));  // Stone
        assert!(!is_replaceable_block(4));  // Cobblestone
    }

    #[test]
    fn test_interactive_blocks() {
        assert!(is_interactive_block(64));  // Oak Door
        assert!(is_interactive_block(96));  // Wooden Trapdoor
        assert!(is_interactive_block(107)); // Fence Gate
        assert!(is_interactive_block(69));  // Lever
        assert!(is_interactive_block(77));  // Stone Button
        assert!(is_interactive_block(54));  // Chest
        assert!(is_interactive_block(58));  // Crafting Table
        assert!(!is_interactive_block(1));  // Stone
        assert!(!is_interactive_block(71)); // Iron Door (cannot be activated by hand)
    }

    #[test]
    fn test_get_block_pick_item() {
        assert_eq!(get_block_pick_item(0, 0), None); // Air
        assert_eq!(get_block_pick_item(8, 0), None); // Water
        assert_eq!(get_block_pick_item(1, 2), Some((1, 2))); // Granite
        assert_eq!(get_block_pick_item(35, 14), Some((35, 14))); // Red wool
        assert_eq!(get_block_pick_item(59, 7), Some((295, 0))); // Wheat -> Seeds
        assert_eq!(get_block_pick_item(62, 0), Some((61, 0))); // Lit furnace -> Furnace
        assert_eq!(get_block_pick_item(64, 0), Some((324, 0))); // Wooden door block -> item
        assert_eq!(get_block_pick_item(43, 0), Some((44, 0))); // Double slab -> single slab
    }

    #[test]
    fn test_calculate_placed_block_meta() {
        // End portal frame (120): opposite of horizontal facing
        assert_eq!(calculate_placed_block_meta(120, 0, 1, 0, 0.0, 0.0), 2);   // Player faces South (0) -> opposite is North (2)
        assert_eq!(calculate_placed_block_meta(120, 0, 1, 0, 180.0, 0.0), 0); // Player faces North (2) -> opposite is South (0)

        // Ladder (65): Face attached
        assert_eq!(calculate_placed_block_meta(65, 0, 2, 8, 0.0, 0.0), 2); // North face
        assert_eq!(calculate_placed_block_meta(65, 0, 3, 8, 0.0, 0.0), 3); // South face
        assert_eq!(calculate_placed_block_meta(65, 0, 4, 8, 0.0, 0.0), 4); // West face
        assert_eq!(calculate_placed_block_meta(65, 0, 5, 8, 0.0, 0.0), 5); // East face

        // Torch (50)
        assert_eq!(calculate_placed_block_meta(50, 0, 1, 0, 0.0, 0.0), 5); // Floor
        assert_eq!(calculate_placed_block_meta(50, 0, 2, 8, 0.0, 0.0), 4); // Wall North

        // Stairs (53): Upside-down check
        assert_eq!(calculate_placed_block_meta(53, 0, 1, 4, 0.0, 0.0), 2); // Normal
        assert_eq!(calculate_placed_block_meta(53, 0, 0, 4, 0.0, 0.0), 6); // Upside-down (2 | 4)
    }
}
