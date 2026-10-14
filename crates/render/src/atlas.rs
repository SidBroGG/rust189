use crate::vertex::BlockFace;
use hashbrown::HashMap;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;
use world::BlockState;

/// Texture coordinate bounds within the stitched texture atlas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureRegion {
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}

impl Default for TextureRegion {
    fn default() -> Self {
        Self {
            u_min: 0.0,
            v_min: 0.0,
            u_max: 1.0,
            v_max: 1.0,
        }
    }
}

/// Metadata for animated texture parsed from `.png.mcmeta`.
#[derive(Debug, Deserialize)]
struct AnimationMeta {
    animation: Option<AnimationData>,
}

#[derive(Debug, Deserialize)]
struct AnimationData {
    frametime: Option<u32>,
    #[allow(dead_code)]
    frames: Option<Vec<serde_json::Value>>,
}

/// Tracks runtime animation state of a texture within the atlas.
#[derive(Clone)]
pub struct AnimatedTexture {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Vec<u8>>,
    pub frametime_ms: u64,
    pub current_frame: usize,
    pub elapsed_ms: u64,
}

impl AnimatedTexture {
    pub fn update(&mut self, dt_ms: u64) -> Option<&[u8]> {
        if self.frames.len() <= 1 {
            return None;
        }

        self.elapsed_ms += dt_ms;
        if self.elapsed_ms >= self.frametime_ms {
            self.elapsed_ms %= self.frametime_ms;
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            Some(&self.frames[self.current_frame])
        } else {
            None
        }
    }
}

/// Holds the GPU texture atlas, texture regions, and animated texture controllers.
pub struct TextureAtlas {
    pub width: u32,
    pub height: u32,
    pub regions: HashMap<String, TextureRegion>,
    pub animations: Vec<AnimatedTexture>,
    pub fallback_region: TextureRegion,
    pub image_data: Vec<u8>,
}

impl TextureAtlas {
    /// Builds a stitched texture atlas from the provided directory of block textures.
    pub fn build(blocks_dir: &Path) -> Self {
        let mut loaded_textures: Vec<(String, DynamicImage, Option<PathBuf>)> = Vec::new();

        if blocks_dir.exists() {
            if let Ok(entries) = fs::read_dir(blocks_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("png") {
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                        if let Ok(img) = image::open(&path) {
                            let meta_path = path.with_extension("png.mcmeta");
                            let meta_opt = if meta_path.exists() { Some(meta_path) } else { None };
                            loaded_textures.push((stem, img, meta_opt));
                        }
                    }
                }
            }
        }

        loaded_textures.sort_by(|a, b| a.0.cmp(&b.0));

        // Create fallback texture if none loaded
        let fallback_img = Self::create_fallback_texture();
        let tile_size = 16u32;

        let total_tiles = (loaded_textures.len() + 1) as u32; // +1 for fallback
        let tiles_per_row = (total_tiles as f32).sqrt().ceil().max(16.0) as u32;
        let tiles_per_col = ((total_tiles + tiles_per_row - 1) / tiles_per_row).max(16);

        let atlas_w = (tiles_per_row * tile_size).next_power_of_two().max(512);
        let atlas_h = (tiles_per_col * tile_size).next_power_of_two().max(512);

        let mut atlas_image = RgbaImage::new(atlas_w, atlas_h);
        let mut regions = HashMap::new();
        let mut animations = Vec::new();

        // Slot 0: Fallback checkerboard
        let fallback_region = Self::place_tile(
            &mut atlas_image,
            &fallback_img,
            0,
            0,
            tile_size,
            atlas_w,
            atlas_h,
        );

        let mut current_slot = 1u32;

        for (name, img, meta_path) in loaded_textures {
            let (w, h) = img.dimensions();
            let is_animated = h > w || meta_path.is_some();

            let slot_x = (current_slot % tiles_per_row) * tile_size;
            let slot_y = (current_slot / tiles_per_row) * tile_size;

            if is_animated {
                let frame_height = w;
                let frame_count = (h / frame_height).max(1);
                let first_frame = img.crop_imm(0, 0, w, frame_height);

                let region = Self::place_tile(
                    &mut atlas_image,
                    &first_frame,
                    slot_x,
                    slot_y,
                    tile_size,
                    atlas_w,
                    atlas_h,
                );
                regions.insert(name.clone(), region);

                // Parse animation metadata
                let mut frametime_ms = 100u64; // Default 2 game ticks (100ms)
                if let Some(mp) = meta_path {
                    if let Ok(content) = fs::read_to_string(&mp) {
                        if let Ok(meta) = serde_json::from_str::<AnimationMeta>(&content) {
                            if let Some(anim) = meta.animation {
                                if let Some(ft) = anim.frametime {
                                    frametime_ms = (ft as u64 * 50).max(50);
                                }
                            }
                        }
                    }
                }

                // Extract all frames
                let mut frames = Vec::new();
                for f in 0..frame_count {
                    let sub = img.crop_imm(0, f * frame_height, w, frame_height);
                    let resized = if w != tile_size || frame_height != tile_size {
                        sub.resize_exact(tile_size, tile_size, image::imageops::FilterType::Nearest)
                    } else {
                        sub
                    };
                    frames.push(resized.to_rgba8().into_raw());
                }

                animations.push(AnimatedTexture {
                    name,
                    x: slot_x,
                    y: slot_y,
                    width: tile_size,
                    height: tile_size,
                    frames,
                    frametime_ms,
                    current_frame: 0,
                    elapsed_ms: 0,
                });
            } else {
                let region = Self::place_tile(
                    &mut atlas_image,
                    &img,
                    slot_x,
                    slot_y,
                    tile_size,
                    atlas_w,
                    atlas_h,
                );
                regions.insert(name, region);
            }

            current_slot += 1;
        }

        info!(
            tiles = regions.len(),
            animated = animations.len(),
            width = atlas_w,
            height = atlas_h,
            "Built texture atlas"
        );

        Self {
            width: atlas_w,
            height: atlas_h,
            regions,
            animations,
            fallback_region,
            image_data: atlas_image.into_raw(),
        }
    }

    fn place_tile(
        atlas: &mut RgbaImage,
        tile: &DynamicImage,
        x: u32,
        y: u32,
        target_size: u32,
        atlas_w: u32,
        atlas_h: u32,
    ) -> TextureRegion {
        let resized = if tile.width() != target_size || tile.height() != target_size {
            tile.resize_exact(target_size, target_size, image::imageops::FilterType::Nearest)
        } else {
            tile.clone()
        };

        let rgba = resized.to_rgba8();
        for py in 0..target_size {
            for px in 0..target_size {
                let pixel = rgba.get_pixel(px, py);
                atlas.put_pixel(x + px, y + py, *pixel);
            }
        }

        // Add 0.5px inset to prevent bleeding when sampled
        let u_min = (x as f32 + 0.05) / atlas_w as f32;
        let v_min = (y as f32 + 0.05) / atlas_h as f32;
        let u_max = ((x + target_size) as f32 - 0.05) / atlas_w as f32;
        let v_max = ((y + target_size) as f32 - 0.05) / atlas_h as f32;

        TextureRegion {
            u_min,
            v_min,
            u_max,
            v_max,
        }
    }

    fn create_fallback_texture() -> DynamicImage {
        let mut img = RgbaImage::new(16, 16);
        let magenta = Rgba([255, 0, 255, 255]);
        let black = Rgba([0, 0, 0, 255]);
        for y in 0..16 {
            for x in 0..16 {
                let color = if ((x / 8) + (y / 8)) % 2 == 0 { magenta } else { black };
                img.put_pixel(x, y, color);
            }
        }
        DynamicImage::ImageRgba8(img)
    }

    /// Retrieves the UV texture coordinates for a block face.
    pub fn get_block_uv(&self, block: BlockState, face: BlockFace) -> (TextureRegion, [f32; 3]) {
        let id = block.id();
        let meta = block.meta();
        let mut tint = [1.0f32, 1.0, 1.0];

        let tex_name = match id {
            0 => return (self.fallback_region, tint),
            // Stone & variants
            1 => match meta {
                1 => "stone_granite",
                2 => "stone_granite_smooth",
                3 => "stone_diorite",
                4 => "stone_diorite_smooth",
                5 => "stone_andesite",
                6 => "stone_andesite_smooth",
                _ => "stone",
            },
            // Grass
            2 => match face {
                BlockFace::Up => {
                    tint = [0.49, 0.74, 0.33];
                    "grass_top"
                }
                BlockFace::Down => "dirt",
                _ => "grass_side",
            },
            // Dirt
            3 => match meta {
                1 => "coarse_dirt",
                2 => match face {
                    BlockFace::Up => "dirt_podzol_top",
                    BlockFace::Down => "dirt",
                    _ => "dirt_podzol_side",
                },
                _ => "dirt",
            },
            // Cobblestone
            4 => "cobblestone",
            // Wooden Planks
            5 => match meta {
                1 => "planks_spruce",
                2 => "planks_birch",
                3 => "planks_jungle",
                4 => "planks_acacia",
                5 => "planks_big_oak",
                _ => "planks_oak",
            },
            // Saplings
            6 => match meta & 7 {
                1 => "sapling_spruce",
                2 => "sapling_birch",
                3 => "sapling_jungle",
                4 => "sapling_acacia",
                5 => "sapling_roofed_oak",
                _ => "sapling_oak",
            },
            // Bedrock
            7 => "bedrock",
            // Water
            8 | 9 => {
                tint = [0.24, 0.46, 0.88];
                "water_still"
            }
            // Lava
            10 | 11 => "lava_still",
            // Sand
            12 => if meta == 1 { "red_sand" } else { "sand" },
            // Gravel
            13 => "gravel",
            // Ores
            14 => "gold_ore",
            15 => "iron_ore",
            16 => "coal_ore",
            // Logs
            17 => {
                let log_name = match meta & 3 {
                    1 => "spruce",
                    2 => "birch",
                    3 => "jungle",
                    _ => "oak",
                };
                match face {
                    BlockFace::Up | BlockFace::Down => match log_name {
                        "spruce" => "log_spruce_top",
                        "birch" => "log_birch_top",
                        "jungle" => "log_jungle_top",
                        _ => "log_oak_top",
                    },
                    _ => match log_name {
                        "spruce" => "log_spruce",
                        "birch" => "log_birch",
                        "jungle" => "log_jungle",
                        _ => "log_oak",
                    },
                }
            }
            // Leaves
            18 => {
                tint = [0.36, 0.62, 0.22];
                match meta & 3 {
                    1 => "leaves_spruce",
                    2 => "leaves_birch",
                    3 => "leaves_jungle",
                    _ => "leaves_oak",
                }
            }
            // Sponge
            19 => if meta == 1 { "sponge_wet" } else { "sponge" },
            // Glass
            20 => "glass",
            // Lapis Ore & Block
            21 => "lapis_ore",
            22 => "lapis_block",
            // Dispenser
            23 => match face {
                BlockFace::Up | BlockFace::Down => "furnace_top",
                BlockFace::North => "dispenser_front_horizontal",
                _ => "furnace_side",
            },
            // Sandstone
            24 => match face {
                BlockFace::Up => "sandstone_top",
                BlockFace::Down => "sandstone_bottom",
                _ => match meta {
                    1 => "sandstone_carved",
                    2 => "sandstone_smooth",
                    _ => "sandstone_normal",
                },
            },
            // Note Block
            25 => "noteblock",
            // Bed
            26 => "bed_head_top",
            // Rails
            27 => if (meta & 8) != 0 { "rail_golden_powered" } else { "rail_golden" },
            28 => if (meta & 8) != 0 { "rail_detector_powered" } else { "rail_detector" },
            29 => match face {
                BlockFace::Up => "piston_top_sticky",
                BlockFace::Down => "piston_bottom",
                _ => "piston_side",
            },
            // Cobweb
            30 => "web",
            // Tallgrass & Fern
            31 => {
                tint = [0.49, 0.74, 0.33];
                if meta == 2 { "fern" } else { "tallgrass" }
            }
            // Dead Bush
            32 => "deadbush",
            // Piston
            33 => match face {
                BlockFace::Up => "piston_top_normal",
                BlockFace::Down => "piston_bottom",
                _ => "piston_side",
            },
            // Wool (16 colors)
            35 => match meta {
                1 => "wool_colored_orange",
                2 => "wool_colored_magenta",
                3 => "wool_colored_light_blue",
                4 => "wool_colored_yellow",
                5 => "wool_colored_lime",
                6 => "wool_colored_pink",
                7 => "wool_colored_gray",
                8 => "wool_colored_silver",
                9 => "wool_colored_cyan",
                10 => "wool_colored_purple",
                11 => "wool_colored_blue",
                12 => "wool_colored_brown",
                13 => "wool_colored_green",
                14 => "wool_colored_red",
                15 => "wool_colored_black",
                _ => "wool_colored_white",
            },
            // Dandelion
            37 => "flower_dandelion",
            // Poppy & Flowers
            38 => match meta {
                1 => "flower_blue_orchid",
                2 => "flower_allium",
                3 => "flower_houstonia",
                4 => "flower_tulip_red",
                5 => "flower_tulip_orange",
                6 => "flower_tulip_white",
                7 => "flower_tulip_pink",
                8 => "flower_oxeye_daisy",
                _ => "flower_rose",
            },
            // Mushrooms
            39 => "mushroom_brown",
            40 => "mushroom_red",
            // Mineral blocks
            41 => "gold_block",
            42 => "iron_block",
            // Double Stone Slab & Stone Slab
            43 | 44 => match meta & 7 {
                0 => match face {
                    BlockFace::Up | BlockFace::Down => "stone_slab_top",
                    _ => "stone_slab_side",
                },
                1 => match face {
                    BlockFace::Up => "sandstone_top",
                    BlockFace::Down => "sandstone_bottom",
                    _ => "sandstone_normal",
                },
                2 => "planks_oak",
                3 => "cobblestone",
                4 => "brick",
                5 => "stonebrick",
                6 => "nether_brick",
                7 => match face {
                    BlockFace::Up => "quartz_block_top",
                    BlockFace::Down => "quartz_block_bottom",
                    _ => "quartz_block_side",
                },
                _ => "stone",
            },
            // Bricks
            45 => "brick",
            // TNT
            46 => match face {
                BlockFace::Up => "tnt_top",
                BlockFace::Down => "tnt_bottom",
                _ => "tnt_side",
            },
            // Bookshelf
            47 => match face {
                BlockFace::Up | BlockFace::Down => "planks_oak",
                _ => "bookshelf",
            },
            // Mossy Cobblestone
            48 => "cobblestone_mossy",
            // Obsidian
            49 => "obsidian",
            // Torch
            50 => "torch_on",
            // Fire
            51 => "fire_layer_0",
            // Mob Spawner
            52 => "mob_spawner",
            // Oak Stairs
            53 => "planks_oak",
            // Chest
            54 | 130 | 146 => "crafting_table_top",
            // Redstone wire
            55 => "redstone_dust_cross",
            // Diamond Ore & Block
            56 => "diamond_ore",
            57 => "diamond_block",
            // Crafting Table
            58 => match face {
                BlockFace::Up => "crafting_table_top",
                BlockFace::Down => "planks_oak",
                BlockFace::North | BlockFace::South => "crafting_table_front",
                _ => "crafting_table_side",
            },
            // Wheat Crops
            59 => match meta {
                0 => "wheat_stage_0",
                1 => "wheat_stage_1",
                2 => "wheat_stage_2",
                3 => "wheat_stage_3",
                4 => "wheat_stage_4",
                5 => "wheat_stage_5",
                6 => "wheat_stage_6",
                _ => "wheat_stage_7",
            },
            // Farmland
            60 => match face {
                BlockFace::Up => if meta > 0 { "farmland_wet" } else { "farmland_dry" },
                _ => "dirt",
            },
            // Furnace
            61 | 62 => match face {
                BlockFace::Up | BlockFace::Down => "furnace_top",
                BlockFace::North => if id == 62 { "furnace_front_on" } else { "furnace_front_off" },
                _ => "furnace_side",
            },
            // Doors
            64 => if meta & 8 != 0 { "door_wood_upper" } else { "door_wood_lower" },
            // Ladder
            65 => "ladder",
            // Rail
            66 => "rail_normal",
            // Cobblestone Stairs
            67 => "cobblestone",
            // Lever
            69 => "lever",
            // Pressure Plates
            70 => "stone",
            71 => if meta & 8 != 0 { "door_iron_upper" } else { "door_iron_lower" },
            72 => "planks_oak",
            // Redstone Ore
            73 | 74 => "redstone_ore",
            // Redstone Torches
            75 => "redstone_torch_off",
            76 => "redstone_torch_on",
            // Stone Button
            77 => "stone",
            // Snow Layer & Block
            78 | 80 => "snow",
            // Ice
            79 => "ice",
            // Cactus
            81 => match face {
                BlockFace::Up => "cactus_top",
                BlockFace::Down => "cactus_bottom",
                _ => "cactus_side",
            },
            // Clay
            82 => "clay",
            // Reeds / Sugar Cane
            83 => {
                tint = [0.49, 0.74, 0.33];
                "reeds"
            }
            // Jukebox
            84 => match face {
                BlockFace::Up => "jukebox_top",
                _ => "jukebox_side",
            },
            // Oak Fence
            85 => "planks_oak",
            // Pumpkin
            86 => match face {
                BlockFace::Up | BlockFace::Down => "pumpkin_top",
                BlockFace::North => "pumpkin_face_off",
                _ => "pumpkin_side",
            },
            // Netherrack
            87 => "netherrack",
            // Soul Sand
            88 => "soul_sand",
            // Glowstone
            89 => "glowstone",
            // Portal
            90 => "portal",
            // Jack o'Lantern
            91 => match face {
                BlockFace::Up | BlockFace::Down => "pumpkin_top",
                BlockFace::North => "pumpkin_face_on",
                _ => "pumpkin_side",
            },
            // Cake
            92 => match face {
                BlockFace::Up => "cake_top",
                BlockFace::Down => "cake_bottom",
                _ => "cake_side",
            },
            // Repeaters
            93 => "repeater_off",
            94 => "repeater_on",
            // Stained Glass (16 colors)
            95 => match meta {
                1 => "glass_orange",
                2 => "glass_magenta",
                3 => "glass_light_blue",
                4 => "glass_yellow",
                5 => "glass_lime",
                6 => "glass_pink",
                7 => "glass_gray",
                8 => "glass_silver",
                9 => "glass_cyan",
                10 => "glass_purple",
                11 => "glass_blue",
                12 => "glass_brown",
                13 => "glass_green",
                14 => "glass_red",
                15 => "glass_black",
                _ => "glass_white",
            },
            // Trapdoors
            96 => "trapdoor",
            // Monster Egg
            97 => match meta {
                1 => "cobblestone",
                2 => "stonebrick",
                3 => "stonebrick_mossy",
                4 => "stonebrick_cracked",
                5 => "stonebrick_carved",
                _ => "stone",
            },
            // Stone Brick
            98 => match meta {
                1 => "stonebrick_mossy",
                2 => "stonebrick_cracked",
                3 => "stonebrick_carved",
                _ => "stonebrick",
            },
            // Huge Mushrooms
            99 => "mushroom_block_skin_brown",
            100 => "mushroom_block_skin_red",
            // Iron Bars
            101 => "iron_bars",
            // Glass Pane
            102 => "glass",
            // Melon
            103 => match face {
                BlockFace::Up | BlockFace::Down => "melon_top",
                _ => "melon_side",
            },
            // Vines
            106 => {
                tint = [0.36, 0.62, 0.22];
                "vine"
            }
            // Fence Gates
            107 => "planks_oak",
            // Stairs
            108 => "brick",
            109 => "stonebrick",
            // Mycelium
            110 => match face {
                BlockFace::Up => "mycelium_top",
                BlockFace::Down => "dirt",
                _ => "mycelium_side",
            },
            // Lily Pad
            111 => {
                tint = [0.13, 0.53, 0.18];
                "waterlily"
            }
            // Nether Brick & components
            112 | 113 | 114 => "nether_brick",
            // Nether Wart
            115 => match meta {
                0 => "nether_wart_stage_0",
                1 => "nether_wart_stage_1",
                _ => "nether_wart_stage_2",
            },
            // Enchanting Table
            116 => match face {
                BlockFace::Up => "enchanting_table_top",
                BlockFace::Down => "enchanting_table_bottom",
                _ => "enchanting_table_side",
            },
            // Brewing Stand
            117 => "brewing_stand",
            // Cauldron
            118 => match face {
                BlockFace::Down => "cauldron_bottom",
                _ => "cauldron_side",
            },
            // End Portal Frame
            120 => match face {
                BlockFace::Up => if meta & 4 != 0 { "endframe_eye" } else { "endframe_top" },
                _ => "endframe_side",
            },
            // End Stone & Dragon Egg
            121 => "end_stone",
            122 => "dragon_egg",
            // Redstone Lamp
            123 => "redstone_lamp_off",
            124 => "redstone_lamp_on",
            // Double Wooden Slab & Wooden Slab
            125 | 126 => match meta & 7 {
                1 => "planks_spruce",
                2 => "planks_birch",
                3 => "planks_jungle",
                4 => "planks_acacia",
                5 => "planks_big_oak",
                _ => "planks_oak",
            },
            // Cocoa
            127 => match meta >> 2 {
                0 => "cocoa_stage_0",
                1 => "cocoa_stage_1",
                _ => "cocoa_stage_2",
            },
            // Sandstone Stairs
            128 => match face {
                BlockFace::Up => "sandstone_top",
                BlockFace::Down => "sandstone_bottom",
                _ => "sandstone_normal",
            },
            // Emerald Ore & Block
            129 => "emerald_ore",
            133 => "emerald_block",
            // Wood Stairs
            134 => "planks_spruce",
            135 => "planks_birch",
            136 => "planks_jungle",
            // Command Block & Beacon
            137 => "command_block",
            138 => "beacon",
            // Cobblestone Wall
            139 => if meta == 1 { "cobblestone_mossy" } else { "cobblestone" },
            // Carrots & Potatoes
            141 => match meta {
                0 => "carrots_stage_0",
                1 => "carrots_stage_1",
                2 => "carrots_stage_2",
                _ => "carrots_stage_3",
            },
            142 => match meta {
                0 => "potatoes_stage_0",
                1 => "potatoes_stage_1",
                2 => "potatoes_stage_2",
                _ => "potatoes_stage_3",
            },
            // Wooden Button
            143 => "planks_oak",
            // Anvil
            145 => match meta >> 2 {
                1 => "anvil_top_damaged_1",
                2 => "anvil_top_damaged_2",
                _ => if face == BlockFace::Up { "anvil_top_damaged_0" } else { "anvil_base" },
            },
            // Weighted Pressure Plates
            147 => "gold_block",
            148 => "iron_block",
            // Comparators
            149 => "comparator_off",
            150 => "comparator_on",
            // Daylight Detector
            151 => match face {
                BlockFace::Up => "daylight_detector_top",
                _ => "daylight_detector_side",
            },
            // Redstone Block & Quartz Ore
            152 => "redstone_block",
            153 => "quartz_ore",
            // Hopper
            154 => match face {
                BlockFace::Up => "hopper_top",
                _ => "hopper_outside",
            },
            // Quartz Block & Stairs
            155 | 156 => match face {
                BlockFace::Up => "quartz_block_top",
                BlockFace::Down => "quartz_block_bottom",
                _ => match meta {
                    1 => "quartz_block_chiseled",
                    2 => "quartz_block_lines",
                    _ => "quartz_block_side",
                },
            },
            // Activator Rail & Dropper
            157 => if (meta & 8) != 0 { "rail_activator_powered" } else { "rail_activator" },
            158 => match face {
                BlockFace::Up | BlockFace::Down => "dropper_front_vertical",
                _ => "dropper_front_horizontal",
            },
            // Stained Clay (16 colors)
            159 => match meta {
                1 => "hardened_clay_stained_orange",
                2 => "hardened_clay_stained_magenta",
                3 => "hardened_clay_stained_light_blue",
                4 => "hardened_clay_stained_yellow",
                5 => "hardened_clay_stained_lime",
                6 => "hardened_clay_stained_pink",
                7 => "hardened_clay_stained_gray",
                8 => "hardened_clay_stained_silver",
                9 => "hardened_clay_stained_cyan",
                10 => "hardened_clay_stained_purple",
                11 => "hardened_clay_stained_blue",
                12 => "hardened_clay_stained_brown",
                13 => "hardened_clay_stained_green",
                14 => "hardened_clay_stained_red",
                15 => "hardened_clay_stained_black",
                _ => "hardened_clay_stained_white",
            },
            // Stained Glass Pane (16 colors)
            160 => match meta {
                1 => "glass_orange",
                2 => "glass_magenta",
                3 => "glass_light_blue",
                4 => "glass_yellow",
                5 => "glass_lime",
                6 => "glass_pink",
                7 => "glass_gray",
                8 => "glass_silver",
                9 => "glass_cyan",
                10 => "glass_purple",
                11 => "glass_blue",
                12 => "glass_brown",
                13 => "glass_green",
                14 => "glass_red",
                15 => "glass_black",
                _ => "glass_white",
            },
            // Leaves 2 (Acacia / Dark Oak)
            161 => {
                tint = [0.36, 0.62, 0.22];
                if (meta & 1) == 1 { "leaves_big_oak" } else { "leaves_acacia" }
            }
            // Logs 2 (Acacia / Dark Oak)
            162 => if (meta & 1) == 1 {
                match face {
                    BlockFace::Up | BlockFace::Down => "log_big_oak_top",
                    _ => "log_big_oak",
                }
            } else {
                match face {
                    BlockFace::Up | BlockFace::Down => "log_acacia_top",
                    _ => "log_acacia",
                }
            },
            // Stairs (Acacia / Dark Oak)
            163 => "planks_acacia",
            164 => "planks_big_oak",
            // Slime Block
            165 => "slime",
            // Barrier
            166 => return (self.fallback_region, tint),
            // Iron Trapdoor
            167 => "iron_trapdoor",
            // Prismarine
            168 => match meta {
                1 => "prismarine_bricks",
                2 => "prismarine_dark",
                _ => "prismarine_rough",
            },
            // Sea Lantern
            169 => "sea_lantern",
            // Hay Block
            170 => match face {
                BlockFace::Up | BlockFace::Down => "hay_block_top",
                _ => "hay_block_side",
            },
            // Carpet (16 colors)
            171 => match meta {
                1 => "wool_colored_orange",
                2 => "wool_colored_magenta",
                3 => "wool_colored_light_blue",
                4 => "wool_colored_yellow",
                5 => "wool_colored_lime",
                6 => "wool_colored_pink",
                7 => "wool_colored_gray",
                8 => "wool_colored_silver",
                9 => "wool_colored_cyan",
                10 => "wool_colored_purple",
                11 => "wool_colored_blue",
                12 => "wool_colored_brown",
                13 => "wool_colored_green",
                14 => "wool_colored_red",
                15 => "wool_colored_black",
                _ => "wool_colored_white",
            },
            // Hardened Clay, Coal Block, Packed Ice
            172 => "hardened_clay",
            173 => "coal_block",
            174 => "ice_packed",
            // Double Plant (Sunflower, Syringa, Double Tallgrass, Large Fern, Rose Bush, Peony)
            175 => {
                let is_upper = (meta & 8) != 0;
                let variant = meta & 7;
                match variant {
                    0 => if is_upper { "double_plant_sunflower_top" } else { "double_plant_sunflower_bottom" },
                    1 => if is_upper { "double_plant_syringa_top" } else { "double_plant_syringa_bottom" },
                    2 => {
                        tint = [0.49, 0.74, 0.33];
                        if is_upper { "double_plant_grass_top" } else { "double_plant_grass_bottom" }
                    }
                    3 => {
                        tint = [0.49, 0.74, 0.33];
                        if is_upper { "double_plant_fern_top" } else { "double_plant_fern_bottom" }
                    }
                    4 => if is_upper { "double_plant_rose_top" } else { "double_plant_rose_bottom" },
                    _ => if is_upper { "double_plant_paeonia_top" } else { "double_plant_paeonia_bottom" },
                }
            }
            // Daylight Detector Inverted
            178 => match face {
                BlockFace::Up => "daylight_detector_inverted_top",
                _ => "daylight_detector_side",
            },
            // Red Sandstone & Slabs/Stairs
            179 | 180 | 181 | 182 => match face {
                BlockFace::Up => "red_sandstone_top",
                BlockFace::Down => "red_sandstone_bottom",
                _ => match meta & 3 {
                    1 => "red_sandstone_carved",
                    2 => "red_sandstone_smooth",
                    _ => "red_sandstone_normal",
                },
            },
            // Fence Gates & Fences (Wood variants)
            183 | 188 => "planks_spruce",
            184 | 189 => "planks_birch",
            185 | 190 => "planks_jungle",
            186 | 191 => "planks_big_oak",
            187 | 192 => "planks_acacia",
            // Doors (Wood variants)
            193 => if meta & 8 != 0 { "door_spruce_upper" } else { "door_spruce_lower" },
            194 => if meta & 8 != 0 { "door_birch_upper" } else { "door_birch_lower" },
            195 => if meta & 8 != 0 { "door_jungle_upper" } else { "door_jungle_lower" },
            196 => if meta & 8 != 0 { "door_acacia_upper" } else { "door_acacia_lower" },
            197 => if meta & 8 != 0 { "door_dark_oak_upper" } else { "door_dark_oak_lower" },
            _ => "stone",
        };

        let region = self.regions.get(tex_name).copied().unwrap_or(self.fallback_region);
        (region, tint)
    }

    /// Determines if a block is rendered in the translucent pass (water, ice, stained glass).
    #[inline]
    pub fn is_translucent(block: BlockState) -> bool {
        matches!(block.id(), 8 | 9 | 79 | 95 | 160 | 165)
    }

    /// Determines if a block is fully transparent or cutout.
    #[inline]
    pub fn is_cutout(block: BlockState) -> bool {
        matches!(
            block.id(),
            6 | 18 | 20 | 27 | 28 | 30 | 31 | 32 | 37 | 38 | 39 | 40 | 50 | 51 | 55 | 59 | 64 | 65 | 66
                | 69 | 70 | 71 | 72 | 75 | 76 | 77 | 78 | 83 | 96 | 101 | 102 | 106 | 111 | 115 | 141
                | 142 | 143 | 147 | 148 | 161 | 167 | 171 | 175 | 193 | 194 | 195 | 196 | 197
        )
    }

    /// Determines if a block is rendered as an X-cross model (tall grass, flowers, saplings).
    #[inline]
    pub fn is_cross(block: BlockState) -> bool {
        matches!(
            block.id(),
            6   // Sapling
            | 30  // Web
            | 31  // Tall grass & Fern
            | 32  // Dead bush
            | 37  // Dandelion
            | 38  // Poppy & Flowers
            | 39  // Brown mushroom
            | 40  // Red mushroom
            | 59  // Crops / Wheat
            | 83  // Reeds / Sugar cane
            | 115 // Nether wart
            | 141 // Carrots
            | 142 // Potatoes
            | 175 // Double plant
        )
    }

    /// Determines if a block is a leaf block (oak, spruce, birch, jungle, acacia, dark oak).
    #[inline]
    pub fn is_leaves(block: BlockState) -> bool {
        matches!(block.id(), 18 | 161)
    }
}
