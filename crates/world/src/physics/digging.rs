/// Tool material categories in Minecraft 1.8.9.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMaterial {
    Wood,
    Stone,
    Iron,
    Diamond,
    Gold,
}

impl ToolMaterial {
    #[inline]
    pub fn efficiency_multiplier(&self) -> f32 {
        match self {
            Self::Wood => 2.0,
            Self::Stone => 4.0,
            Self::Iron => 6.0,
            Self::Diamond => 8.0,
            Self::Gold => 12.0,
        }
    }

    #[inline]
    pub fn harvest_level(&self) -> u32 {
        match self {
            Self::Wood | Self::Gold => 0,
            Self::Stone => 1,
            Self::Iron => 2,
            Self::Diamond => 3,
        }
    }
}

/// Tool types recognized for breaking blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    Pickaxe(ToolMaterial),
    Shovel(ToolMaterial),
    Axe(ToolMaterial),
    Sword(ToolMaterial),
    Shears,
    None,
}

/// Returns the ToolType from a 1.8.9 item ID.
pub fn get_tool_type(item_id: i16) -> ToolType {
    match item_id {
        269 => ToolType::Shovel(ToolMaterial::Wood),
        270 => ToolType::Pickaxe(ToolMaterial::Wood),
        271 => ToolType::Axe(ToolMaterial::Wood),
        268 => ToolType::Sword(ToolMaterial::Wood),

        273 => ToolType::Shovel(ToolMaterial::Stone),
        274 => ToolType::Pickaxe(ToolMaterial::Stone),
        275 => ToolType::Axe(ToolMaterial::Stone),
        272 => ToolType::Sword(ToolMaterial::Stone),

        256 => ToolType::Shovel(ToolMaterial::Iron),
        257 => ToolType::Pickaxe(ToolMaterial::Iron),
        258 => ToolType::Axe(ToolMaterial::Iron),
        267 => ToolType::Sword(ToolMaterial::Iron),

        277 => ToolType::Shovel(ToolMaterial::Diamond),
        278 => ToolType::Pickaxe(ToolMaterial::Diamond),
        279 => ToolType::Axe(ToolMaterial::Diamond),
        276 => ToolType::Sword(ToolMaterial::Diamond),

        284 => ToolType::Shovel(ToolMaterial::Gold),
        285 => ToolType::Pickaxe(ToolMaterial::Gold),
        286 => ToolType::Axe(ToolMaterial::Gold),
        283 => ToolType::Sword(ToolMaterial::Gold),

        359 => ToolType::Shears,
        _ => ToolType::None,
    }
}

/// Returns vanilla block hardness according to MCP 1.8.9 Block definitions.
pub fn get_block_hardness(block_id: u16) -> f32 {
    match block_id {
        0 => 0.0,       // Air
        7 => -1.0,      // Bedrock (unbreakable)
        49 => 50.0,     // Obsidian
        1 => 1.5,       // Stone
        2 => 0.6,       // Grass
        3 => 0.5,       // Dirt
        4 => 2.0,       // Cobblestone
        5 => 2.0,       // Planks
        12 => 0.5,      // Sand
        13 => 0.6,      // Gravel
        14 => 3.0,      // Gold Ore
        15 => 3.0,      // Iron Ore
        16 => 3.0,      // Coal Ore
        17 | 162 => 2.0,// Wood Log
        18 | 161 => 0.2,// Leaves
        20 | 95 | 102 | 160 => 0.3, // Glass
        21 => 3.0,      // Lapis Ore
        22 => 3.0,      // Lapis Block
        24 => 0.8,      // Sandstone
        30 => 4.0,      // Web
        35 => 0.8,      // Wool
        41 => 3.0,      // Gold Block
        42 => 5.0,      // Iron Block
        45 => 2.0,      // Brick Block
        46 => 0.0,      // TNT
        47 => 1.5,      // Bookshelf
        48 => 2.0,      // Mossy Cobblestone
        50 => 0.0,      // Torch
        54 | 130 | 146 => 2.5, // Chests
        56 => 3.0,      // Diamond Ore
        57 => 5.0,      // Diamond Block
        58 => 2.5,      // Crafting Table
        61 | 62 => 3.5, // Furnaces
        73 | 74 => 3.0, // Redstone Ore
        79 => 0.5,      // Ice
        80 => 0.2,      // Snow Block
        81 => 0.4,      // Cactus
        82 => 0.6,      // Clay
        87 => 0.4,      // Netherrack
        88 => 0.5,      // Soul Sand
        89 => 0.3,      // Glowstone
        98 => 1.5,      // Stone Bricks
        121 => 3.0,     // End Stone
        129 => 3.0,     // Emerald Ore
        133 => 5.0,     // Emerald Block
        152 => 5.0,     // Redstone Block
        165 => 0.0,     // Slime Block
        173 => 5.0,     // Coal Block
        174 => 0.5,     // Packed Ice
        _ => 1.5,       // Default hardness
    }
}

/// Checks whether a tool is effective on a given block.
pub fn is_tool_effective(tool: ToolType, block_id: u16) -> bool {
    match tool {
        ToolType::Pickaxe(_) => matches!(
            block_id,
            1 | 4 | 14 | 15 | 16 | 21 | 22 | 24 | 41 | 42 | 43 | 44 | 45 | 48 | 49
                | 56 | 57 | 61 | 62 | 67 | 70 | 71 | 73 | 74 | 98 | 108 | 109 | 112
                | 113 | 114 | 116 | 121 | 128 | 129 | 133 | 145 | 147 | 148 | 152 | 153
                | 155 | 156 | 157 | 159 | 168 | 172 | 173 | 174
        ),
        ToolType::Shovel(_) => matches!(
            block_id,
            2 | 3 | 12 | 13 | 60 | 78 | 80 | 82 | 88 | 110
        ),
        ToolType::Axe(_) => matches!(
            block_id,
            5 | 17 | 25 | 47 | 53 | 54 | 58 | 64 | 65 | 72 | 84 | 85 | 96 | 107
                | 125 | 126 | 134 | 135 | 136 | 146 | 158 | 162 | 163 | 164 | 183 | 184
                | 185 | 186 | 187
        ),
        ToolType::Sword(_) => block_id == 30, // Cobweb
        ToolType::Shears => matches!(block_id, 18 | 30 | 31 | 32 | 35 | 106 | 161), // Leaves, Wool, Web, Vines
        ToolType::None => false,
    }
}

/// Checks if player can harvest drops from the block with their equipped tool.
pub fn can_harvest_block(tool: ToolType, block_id: u16) -> bool {
    // Blocks that require a specific pickaxe harvest level:
    match block_id {
        49 => {
            // Obsidian requires Diamond pickaxe
            matches!(tool, ToolType::Pickaxe(ToolMaterial::Diamond))
        }
        14 | 56 | 57 | 73 | 74 | 129 | 133 => {
            // Gold, Diamond, Redstone, Emerald require Iron or Diamond pickaxe
            match tool {
                ToolType::Pickaxe(mat) => mat.harvest_level() >= 2,
                _ => false,
            }
        }
        15 | 21 | 22 | 41 | 42 => {
            // Iron, Lapis require Stone, Iron, or Diamond pickaxe
            match tool {
                ToolType::Pickaxe(mat) => mat.harvest_level() >= 1,
                _ => false,
            }
        }
        1 | 4 | 24 | 43 | 44 | 45 | 48 | 61 | 62 | 67 | 98 | 108 | 109 | 112 | 113 | 114 | 121 | 128 | 152 | 153 | 155 | 156 | 157 | 159 | 168 | 172 | 173 | 174 => {
            // Stone, cobblestone, sandstone require any pickaxe
            matches!(tool, ToolType::Pickaxe(_))
        }
        _ => true, // Wood, dirt, sand, gravel etc. can be harvested by hand
    }
}

/// Calculates tool efficiency according to MCP 1.8.9 EntityPlayer.getToolDigEfficiency.
pub fn calculate_tool_dig_efficiency(
    tool: ToolType,
    block_id: u16,
    efficiency_level: u32,
    haste_level: u32,
    mining_fatigue_level: u32,
    in_water: bool,
    has_aqua_affinity: bool,
    on_ground: bool,
) -> f32 {
    let mut speed = 1.0f32;

    if is_tool_effective(tool, block_id) {
        speed = match tool {
            ToolType::Pickaxe(mat) | ToolType::Shovel(mat) | ToolType::Axe(mat) => mat.efficiency_multiplier(),
            ToolType::Sword(_) => 15.0, // Web
            ToolType::Shears => {
                if matches!(block_id, 18 | 161) {
                    15.0 // Leaves
                } else {
                    5.0 // Wool
                }
            }
            ToolType::None => 1.0,
        };

        if speed > 1.0 && efficiency_level > 0 {
            speed += (efficiency_level * efficiency_level + 1) as f32;
        }
    } else if matches!(tool, ToolType::Sword(_)) && matches!(block_id, 18 | 161) {
        speed = 1.5; // Sword on leaves
    }

    // Potion: Haste
    if haste_level > 0 {
        speed *= 1.0 + (haste_level as f32) * 0.2;
    }

    // Potion: Mining Fatigue
    if mining_fatigue_level > 0 {
        let fatigue_factor = match mining_fatigue_level {
            1 => 0.3,
            2 => 0.09,
            3 => 0.0027,
            _ => 0.00081,
        };
        speed *= fatigue_factor;
    }

    // Environmental factors
    if in_water && !has_aqua_affinity {
        speed /= 5.0;
    }

    if !on_ground {
        speed /= 5.0;
    }

    speed
}

/// Calculates relative block hardness per tick (MCP Block.getPlayerRelativeBlockHardness).
/// Progress reaches 1.0 when the block is broken.
pub fn get_player_relative_block_hardness(
    block_id: u16,
    held_item_id: i16,
    efficiency_level: u32,
    haste_level: u32,
    mining_fatigue_level: u32,
    in_water: bool,
    has_aqua_affinity: bool,
    on_ground: bool,
    is_creative: bool,
) -> f32 {
    if is_creative {
        return 1.0;
    }

    let hardness = get_block_hardness(block_id);
    if hardness < 0.0 {
        return 0.0; // Unbreakable (Bedrock)
    }

    let tool = get_tool_type(held_item_id);
    let speed = calculate_tool_dig_efficiency(
        tool,
        block_id,
        efficiency_level,
        haste_level,
        mining_fatigue_level,
        in_water,
        has_aqua_affinity,
        on_ground,
    );

    let can_harvest = can_harvest_block(tool, block_id);
    let divisor = if can_harvest { 30.0 } else { 100.0 };

    speed / hardness / divisor
}
