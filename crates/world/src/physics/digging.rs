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
        1 => 1.5,       // Stone
        2 => 0.6,       // Grass
        3 => 0.5,       // Dirt
        4 => 2.0,       // Cobblestone
        5 => 2.0,       // Planks
        6 => 0.0,       // Saplings
        7 => -1.0,      // Bedrock (unbreakable)
        8 | 9 => 100.0, // Flowing / Still Water
        10 | 11 => 100.0, // Flowing / Still Lava
        12 => 0.5,      // Sand
        13 => 0.6,      // Gravel
        14 => 3.0,      // Gold Ore
        15 => 3.0,      // Iron Ore
        16 => 3.0,      // Coal Ore
        17 | 162 => 2.0,// Wood Log, Log2
        18 | 161 => 0.2,// Leaves, Leaves2
        19 => 0.6,      // Sponge
        20 | 95 => 0.3, // Glass, Stained Glass
        21 => 3.0,      // Lapis Ore
        22 => 3.0,      // Lapis Block
        23 => 3.5,      // Dispenser
        24 => 0.8,      // Sandstone
        25 => 0.8,      // Note Block
        26 => 0.2,      // Bed
        27 | 28 | 66 | 157 => 0.7, // Rails (Powered, Detector, Normal, Activator)
        29 | 33 | 34 => 0.5, // Sticky Piston, Piston, Piston Head
        30 => 4.0,      // Web
        31 | 32 => 0.0, // Tall Grass, Dead Bush
        35 => 0.8,      // Wool
        36 => 0.0,      // Piston Moving
        37 | 38 => 0.0, // Yellow / Red Flower
        39 | 40 => 0.0, // Brown / Red Mushroom
        41 => 3.0,      // Gold Block
        42 => 5.0,      // Iron Block
        43 | 44 => 2.0, // Double Stone Slab, Stone Slab
        45 => 2.0,      // Brick Block
        46 => 0.0,      // TNT
        47 => 1.5,      // Bookshelf
        48 => 2.0,      // Mossy Cobblestone
        49 => 50.0,     // Obsidian
        50 => 0.0,      // Torch
        51 => 0.0,      // Fire
        52 => 5.0,      // Mob Spawner
        53 => 2.0,      // Oak Stairs
        54 | 146 => 2.5,// Chest, Trapped Chest
        55 => 0.0,      // Redstone Wire
        56 => 3.0,      // Diamond Ore
        57 => 5.0,      // Diamond Block
        58 => 2.5,      // Crafting Table
        59 | 141 | 142 => 0.0, // Crops (Wheat, Carrots, Potatoes)
        60 => 0.6,      // Farmland
        61 | 62 => 3.5, // Furnaces
        63 | 68 => 1.0, // Standing / Wall Sign
        64 | 193..=197 => 3.0, // Wooden Doors (Oak, Spruce, Birch, Jungle, Acacia, Dark Oak)
        65 => 0.4,      // Ladder
        67 => 2.0,      // Cobblestone Stairs
        69 => 0.5,      // Lever
        70 | 72 | 147 | 148 => 0.5, // Pressure Plates (Stone, Wood, Light, Heavy)
        71 => 5.0,      // Iron Door
        73 | 74 => 3.0, // Redstone Ore (Unlit / Lit)
        75 | 76 => 0.0, // Redstone Torch (Unlit / Lit)
        77 | 143 => 0.5,// Buttons (Stone, Wood)
        78 => 0.1,      // Snow Layer
        79 => 0.5,      // Ice
        80 => 0.2,      // Snow Block
        81 => 0.4,      // Cactus
        82 => 0.6,      // Clay
        83 => 0.0,      // Reeds / Sugar Cane
        84 => 2.0,      // Jukebox
        85 | 188..=192 => 2.0, // Fences
        86 | 91 => 1.0, // Pumpkin, Jack-o'-Lantern
        87 => 0.4,      // Netherrack
        88 => 0.5,      // Soul Sand
        89 => 0.3,      // Glowstone
        90 | 119 => -1.0, // Nether Portal, End Portal
        92 => 0.5,      // Cake
        93 | 94 | 149 | 150 => 0.0, // Repeaters, Comparators
        96 => 3.0,      // Wooden Trapdoor
        97 => 0.75,     // Monster Egg / Silverfish
        98 => 1.5,      // Stone Bricks
        99 | 100 => 0.2,// Huge Mushroom (Brown, Red)
        101 => 5.0,     // Iron Bars
        102 | 160 => 0.3, // Glass Pane, Stained Glass Pane
        103 => 1.0,     // Melon Block
        104 | 105 => 0.0, // Pumpkin / Melon Stem
        106 => 0.2,     // Vines
        107 | 183..=187 => 2.0, // Fence Gates
        108 => 2.0,     // Brick Stairs
        109 => 1.5,     // Stone Brick Stairs
        110 => 0.6,     // Mycelium
        111 => 0.0,     // Lily Pad
        112 | 113 | 114 => 2.0, // Nether Brick, Nether Brick Fence, Nether Brick Stairs
        115 => 0.0,     // Nether Wart
        116 => 5.0,     // Enchantment Table
        117 => 0.5,     // Brewing Stand
        118 => 2.0,     // Cauldron
        120 => -1.0,    // End Portal Frame
        121 => 3.0,     // End Stone
        122 => 3.0,     // Dragon Egg
        123 | 124 => 0.3, // Redstone Lamp (Unlit / Lit)
        125 | 126 => 2.0, // Double Wooden Slab, Wooden Slab
        127 => 0.2,     // Cocoa
        128 => 0.8,     // Sandstone Stairs
        129 => 3.0,     // Emerald Ore
        130 => 22.5,    // Ender Chest
        131 | 132 => 0.0, // Tripwire Hook, Tripwire
        133 => 5.0,     // Emerald Block
        134..=136 | 163 | 164 => 2.0, // Wood Stairs (Spruce, Birch, Jungle, Acacia, Dark Oak)
        137 | 210 | 211 => -1.0, // Command Blocks
        138 => 3.0,     // Beacon
        139 => 2.0,     // Cobblestone Wall
        140 => 0.0,     // Flower Pot
        144 => 1.0,     // Skull / Head
        145 => 5.0,     // Anvil
        151 | 178 => 0.2, // Daylight Detectors
        152 => 5.0,     // Redstone Block
        153 => 3.0,     // Nether Quartz Ore
        154 => 3.0,     // Hopper
        155 | 156 => 0.8, // Quartz Block, Quartz Stairs
        158 => 3.5,     // Dropper
        159 | 172 => 1.25, // Stained Clay / Terracotta, Hardened Clay
        165 => 0.0,     // Slime Block
        166 => -1.0,    // Barrier
        167 => 5.0,     // Iron Trapdoor
        168 => 1.5,     // Prismarine
        169 => 0.3,     // Sea Lantern
        170 => 0.5,     // Hay Bale
        171 => 0.1,     // Carpet
        173 => 5.0,     // Coal Block
        174 => 0.5,     // Packed Ice
        175 => 0.0,     // Double Plant
        176 | 177 => 1.0, // Banners
        179..=182 => 0.8, // Red Sandstone, Stairs, Slabs
        _ => 1.5,       // Default fallback hardness
    }
}

/// Checks whether a tool is effective on a given block.
pub fn is_tool_effective(tool: ToolType, block_id: u16) -> bool {
    match tool {
        ToolType::Pickaxe(_) => matches!(
            block_id,
            1 | 4 | 14 | 15 | 16 | 21 | 22 | 23 | 24 | 41 | 42 | 43 | 44 | 45 | 48 | 49
                | 56 | 57 | 61 | 62 | 67 | 70 | 71 | 73 | 74 | 77 | 98 | 101 | 108 | 109 | 112
                | 113 | 114 | 116 | 117 | 118 | 121 | 128 | 129 | 130 | 133 | 138 | 139 | 145 | 147
                | 148 | 152 | 153 | 154 | 155 | 156 | 157 | 158 | 159 | 167 | 168 | 172 | 173 | 174
                | 179..=182 | 27 | 28 | 66
        ),
        ToolType::Shovel(_) => matches!(
            block_id,
            2 | 3 | 12 | 13 | 60 | 78 | 80 | 82 | 88 | 110
        ),
        ToolType::Axe(_) => matches!(
            block_id,
            5 | 17 | 25 | 47 | 53 | 54 | 58 | 63 | 64 | 68 | 72 | 84 | 85 | 86 | 91 | 96 | 103
                | 107 | 125 | 126 | 127 | 134..=136 | 143 | 146 | 151 | 162..=164 | 176 | 177 | 178
                | 183..=197
        ),
        ToolType::Sword(_) => matches!(block_id, 30 | 18 | 161 | 86 | 91 | 103 | 106 | 127 | 170),
        ToolType::Shears => matches!(block_id, 18 | 30 | 31 | 32 | 35 | 106 | 161 | 171),
        ToolType::None => false,
    }
}

/// Checks if player can harvest drops from the block with their equipped tool.
pub fn can_harvest_block(tool: ToolType, block_id: u16) -> bool {
    match block_id {
        49 => {
            // Obsidian requires Diamond pickaxe
            matches!(tool, ToolType::Pickaxe(ToolMaterial::Diamond))
        }
        14 | 41 | 56 | 57 | 73 | 74 | 129 | 133 => {
            // Gold, Diamond, Redstone, Emerald require Iron or Diamond pickaxe
            match tool {
                ToolType::Pickaxe(mat) => mat.harvest_level() >= 2,
                _ => false,
            }
        }
        15 | 21 | 22 | 42 => {
            // Iron, Lapis require Stone, Iron, or Diamond pickaxe
            match tool {
                ToolType::Pickaxe(mat) => mat.harvest_level() >= 1,
                _ => false,
            }
        }
        1 | 4 | 23 | 24 | 43 | 44 | 45 | 48 | 61 | 62 | 67 | 70 | 71 | 77 | 98 | 101 | 108 | 109 | 112 | 113 | 114 | 116 | 117 | 118 | 121 | 128 | 130 | 138 | 139 | 145 | 147 | 148 | 152 | 153 | 154 | 155 | 156 | 157 | 158 | 159 | 167 | 168 | 172 | 173 | 174 | 179..=182 | 27 | 28 | 66 => {
            // Stone, cobblestone, sandstone etc. require any pickaxe
            matches!(tool, ToolType::Pickaxe(_))
        }
        78 | 80 => {
            // Snow and snow block require a shovel to harvest
            matches!(tool, ToolType::Shovel(_))
        }
        30 => {
            // Cobweb can be harvested by shears or sword
            matches!(tool, ToolType::Shears | ToolType::Sword(_))
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
                } else if block_id == 30 {
                    15.0 // Web
                } else {
                    5.0 // Wool, Carpet
                }
            }
            ToolType::None => 1.0,
        };

        if speed > 1.0 && efficiency_level > 0 {
            speed += (efficiency_level * efficiency_level + 1) as f32;
        }
    } else if matches!(tool, ToolType::Sword(_)) && matches!(block_id, 18 | 161 | 86 | 91 | 103 | 106 | 127) {
        speed = 1.5; // Sword on leaves, melons, pumpkins, vines, cocoa
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
        return 0.0; // Unbreakable (Bedrock, Barrier, End Portal)
    }
    if hardness == 0.0 {
        return 1.0; // Instant break
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
