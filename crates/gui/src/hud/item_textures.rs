use egui::{ColorImage, Context, TextureHandle, TextureId, TextureOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub type ItemTextureCache = Arc<Mutex<HashMap<(i16, i16), Option<TextureHandle>>>>;

/// Locates the Minecraft assets directory across standard project directory structures.
pub fn find_assets_dir() -> PathBuf {
    for candidate in [
        Path::new("assets"),
        Path::new("../assets"),
        Path::new("../../assets"),
    ] {
        if candidate.join("minecraft/textures").exists() {
            return candidate.to_path_buf();
        }
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        for rel in ["assets", "../assets", "../../assets"] {
            let candidate = PathBuf::from(&manifest_dir).join(rel);
            if candidate.join("minecraft/textures").exists() {
                return candidate;
            }
        }
    }

    PathBuf::from("assets")
}

/// Maps an item/block ID and metadata to its relative texture asset path within `minecraft/textures/`.
pub fn get_item_asset_path(item_id: i16, damage: i16, name: &str) -> Option<String> {
    // 1. Direct item ID mapping
    let path = match item_id {
        // Block items (1..255)
        1 => match damage {
            1 => "blocks/stone_granite.png",
            2 => "blocks/stone_granite_smooth.png",
            3 => "blocks/stone_diorite.png",
            4 => "blocks/stone_diorite_smooth.png",
            5 => "blocks/stone_andesite.png",
            6 => "blocks/stone_andesite_smooth.png",
            _ => "blocks/stone.png",
        },
        2 => "blocks/grass_top.png",
        3 => match damage {
            1 => "blocks/coarse_dirt.png",
            2 => "blocks/dirt_podzol_top.png",
            _ => "blocks/dirt.png",
        },
        4 => "blocks/cobblestone.png",
        5 => match damage {
            1 => "blocks/planks_spruce.png",
            2 => "blocks/planks_birch.png",
            3 => "blocks/planks_jungle.png",
            4 => "blocks/planks_acacia.png",
            5 => "blocks/planks_big_oak.png",
            _ => "blocks/planks_oak.png",
        },
        6 => match damage {
            1 => "blocks/sapling_spruce.png",
            2 => "blocks/sapling_birch.png",
            3 => "blocks/sapling_jungle.png",
            4 => "blocks/sapling_acacia.png",
            5 => "blocks/sapling_roofed_oak.png",
            _ => "blocks/sapling_oak.png",
        },
        7 => "blocks/bedrock.png",
        8 | 9 => "blocks/water_still.png",
        10 | 11 => "blocks/lava_still.png",
        12 => if damage == 1 { "blocks/red_sand.png" } else { "blocks/sand.png" },
        13 => "blocks/gravel.png",
        14 => "blocks/gold_ore.png",
        15 => "blocks/iron_ore.png",
        16 => "blocks/coal_ore.png",
        17 => match damage {
            1 => "blocks/log_spruce.png",
            2 => "blocks/log_birch.png",
            3 => "blocks/log_jungle.png",
            _ => "blocks/log_oak.png",
        },
        18 => match damage {
            1 => "blocks/leaves_spruce.png",
            2 => "blocks/leaves_birch.png",
            3 => "blocks/leaves_jungle.png",
            _ => "blocks/leaves_oak.png",
        },
        19 => if damage == 1 { "blocks/sponge_wet.png" } else { "blocks/sponge.png" },
        20 => "blocks/glass.png",
        21 => "blocks/lapis_ore.png",
        22 => "blocks/lapis_block.png",
        23 => "blocks/dispenser_front_horizontal.png",
        24 => match damage {
            1 => "blocks/sandstone_carved.png",
            2 => "blocks/sandstone_smooth.png",
            _ => "blocks/sandstone_normal.png",
        },
        25 => "blocks/noteblock.png",
        26 => "items/bed.png",
        27 => "blocks/rail_golden.png",
        28 => "blocks/rail_detector.png",
        29 => "blocks/piston_top_sticky.png",
        30 => "blocks/web.png",
        31 => "blocks/tallgrass.png",
        32 => "blocks/deadbush.png",
        33 => "blocks/piston_top_normal.png",
        35 | 171 => match damage & 15 {
            1 => "blocks/wool_colored_orange.png",
            2 => "blocks/wool_colored_magenta.png",
            3 => "blocks/wool_colored_light_blue.png",
            4 => "blocks/wool_colored_yellow.png",
            5 => "blocks/wool_colored_lime.png",
            6 => "blocks/wool_colored_pink.png",
            7 => "blocks/wool_colored_gray.png",
            8 => "blocks/wool_colored_silver.png",
            9 => "blocks/wool_colored_cyan.png",
            10 => "blocks/wool_colored_purple.png",
            11 => "blocks/wool_colored_blue.png",
            12 => "blocks/wool_colored_brown.png",
            13 => "blocks/wool_colored_green.png",
            14 => "blocks/wool_colored_red.png",
            15 => "blocks/wool_colored_black.png",
            _ => "blocks/wool_colored_white.png",
        },
        37 => "blocks/flower_dandelion.png",
        38 => match damage {
            1 => "blocks/flower_blue_orchid.png",
            2 => "blocks/flower_allium.png",
            3 => "blocks/flower_houstonia.png",
            4 => "blocks/flower_tulip_red.png",
            5 => "blocks/flower_tulip_orange.png",
            6 => "blocks/flower_tulip_white.png",
            7 => "blocks/flower_tulip_pink.png",
            8 => "blocks/flower_oxeye_daisy.png",
            _ => "blocks/flower_rose.png",
        },
        39 => "blocks/mushroom_brown.png",
        40 => "blocks/mushroom_red.png",
        41 => "blocks/gold_block.png",
        42 => "blocks/iron_block.png",
        44 => "blocks/stone_slab_top.png",
        45 => "blocks/brick.png",
        46 => "blocks/tnt_side.png",
        47 => "blocks/bookshelf.png",
        48 => "blocks/cobblestone_mossy.png",
        49 => "blocks/obsidian.png",
        50 => "blocks/torch_on.png",
        51 => "blocks/fire_layer_0.png",
        52 => "blocks/mob_spawner.png",
        53 => "blocks/planks_oak.png",
        54 => "items/chest_minecart.png",
        55 => "items/redstone_dust.png",
        56 => "blocks/diamond_ore.png",
        57 => "blocks/diamond_block.png",
        58 => "blocks/crafting_table_front.png",
        61 => "blocks/furnace_front_off.png",
        62 => "blocks/furnace_front_on.png",
        64 => "items/door_wood.png",
        65 => "blocks/ladder.png",
        66 => "blocks/rail_normal.png",
        67 => "blocks/cobblestone.png",
        69 => "blocks/lever.png",
        70 => "blocks/stone.png",
        71 => "items/door_iron.png",
        72 => "blocks/planks_oak.png",
        73 => "blocks/redstone_ore.png",
        76 => "blocks/redstone_torch_on.png",
        77 => "blocks/stone.png",
        78 => "blocks/snow.png",
        79 => "blocks/ice.png",
        80 => "blocks/snow.png",
        81 => "blocks/cactus_side.png",
        82 => "blocks/clay.png",
        83 => "items/reeds.png",
        84 => "blocks/jukebox_top.png",
        85 => "blocks/planks_oak.png",
        86 => "blocks/pumpkin_face_off.png",
        87 => "blocks/netherrack.png",
        88 => "blocks/soul_sand.png",
        89 => "blocks/glowstone.png",
        90 => "blocks/portal.png",
        91 => "blocks/pumpkin_face_on.png",
        92 => "items/cake.png",
        95 | 160 => match damage & 15 {
            1 => "blocks/glass_orange.png",
            2 => "blocks/glass_magenta.png",
            3 => "blocks/glass_light_blue.png",
            4 => "blocks/glass_yellow.png",
            5 => "blocks/glass_lime.png",
            6 => "blocks/glass_pink.png",
            7 => "blocks/glass_gray.png",
            8 => "blocks/glass_silver.png",
            9 => "blocks/glass_cyan.png",
            10 => "blocks/glass_purple.png",
            11 => "blocks/glass_blue.png",
            12 => "blocks/glass_brown.png",
            13 => "blocks/glass_green.png",
            14 => "blocks/glass_red.png",
            15 => "blocks/glass_black.png",
            _ => "blocks/glass_white.png",
        },
        96 => "blocks/trapdoor.png",
        98 => match damage {
            1 => "blocks/stonebrick_mossy.png",
            2 => "blocks/stonebrick_cracked.png",
            3 => "blocks/stonebrick_carved.png",
            _ => "blocks/stonebrick.png",
        },
        101 => "blocks/iron_bars.png",
        102 => "blocks/glass.png",
        103 => "blocks/melon_side.png",
        106 => "blocks/vine.png",
        107 => "blocks/planks_oak.png",
        108 => "blocks/brick.png",
        109 => "blocks/stonebrick.png",
        110 => "blocks/mycelium_top.png",
        111 => "blocks/waterlily.png",
        112 => "blocks/nether_brick.png",
        115 => "items/nether_wart.png",
        116 => "blocks/enchanting_table_top.png",
        117 => "items/brewing_stand.png",
        118 => "items/cauldron.png",
        120 => "blocks/endframe_top.png",
        121 => "blocks/end_stone.png",
        122 => "blocks/dragon_egg.png",
        123 => "blocks/redstone_lamp_off.png",
        124 => "blocks/redstone_lamp_on.png",
        126 => "blocks/planks_oak.png",
        127 => "blocks/cocoa_stage_2.png",
        128 => "blocks/sandstone_normal.png",
        129 => "blocks/emerald_ore.png",
        130 => "blocks/obsidian.png",
        131 => "blocks/trip_wire_source.png",
        133 => "blocks/emerald_block.png",
        134 => "blocks/planks_spruce.png",
        135 => "blocks/planks_birch.png",
        136 => "blocks/planks_jungle.png",
        137 => "blocks/command_block.png",
        138 => "blocks/beacon.png",
        139 => "blocks/cobblestone.png",
        140 => "items/flower_pot.png",
        141 => "items/carrot.png",
        142 => "items/potato.png",
        143 => "blocks/planks_oak.png",
        145 => match damage {
            1 => "blocks/anvil_top_damaged_1.png",
            2 => "blocks/anvil_top_damaged_2.png",
            _ => "blocks/anvil_base.png",
        },
        146 => "items/chest_minecart.png",
        149 | 150 => "items/comparator.png",
        151 => "blocks/daylight_detector_top.png",
        152 => "blocks/redstone_block.png",
        153 => "blocks/quartz_ore.png",
        154 => "items/hopper.png",
        155 => match damage {
            1 => "blocks/quartz_block_chiseled.png",
            2 => "blocks/quartz_block_lines.png",
            _ => "blocks/quartz_block_side.png",
        },
        157 => "blocks/rail_activator.png",
        158 => "blocks/dropper_front_horizontal.png",
        159 => match damage {
            1 => "blocks/hardened_clay_stained_orange.png",
            2 => "blocks/hardened_clay_stained_magenta.png",
            3 => "blocks/hardened_clay_stained_light_blue.png",
            4 => "blocks/hardened_clay_stained_yellow.png",
            5 => "blocks/hardened_clay_stained_lime.png",
            6 => "blocks/hardened_clay_stained_pink.png",
            7 => "blocks/hardened_clay_stained_gray.png",
            8 => "blocks/hardened_clay_stained_silver.png",
            9 => "blocks/hardened_clay_stained_cyan.png",
            10 => "blocks/hardened_clay_stained_purple.png",
            11 => "blocks/hardened_clay_stained_blue.png",
            12 => "blocks/hardened_clay_stained_brown.png",
            13 => "blocks/hardened_clay_stained_green.png",
            14 => "blocks/hardened_clay_stained_red.png",
            15 => "blocks/hardened_clay_stained_black.png",
            _ => "blocks/hardened_clay_stained_white.png",
        },
        161 => "blocks/leaves_acacia.png",
        162 => "blocks/log_acacia.png",
        163 => "blocks/planks_acacia.png",
        164 => "blocks/planks_big_oak.png",
        165 => "blocks/slime.png",
        166 => "items/barrier.png",
        167 => "blocks/iron_trapdoor.png",
        168 => match damage {
            1 => "blocks/prismarine_bricks.png",
            2 => "blocks/prismarine_dark.png",
            _ => "blocks/prismarine_rough.png",
        },
        169 => "blocks/sea_lantern.png",
        170 => "blocks/hay_block_side.png",
        172 => "blocks/hardened_clay.png",
        173 => "blocks/coal_block.png",
        174 => "blocks/ice_packed.png",
        175 => "blocks/double_plant_sunflower_front.png",
        178 => "blocks/daylight_detector_inverted_top.png",
        179 => match damage {
            1 => "blocks/red_sandstone_carved.png",
            2 => "blocks/red_sandstone_smooth.png",
            _ => "blocks/red_sandstone_normal.png",
        },
        180 => "blocks/red_sandstone_normal.png",
        182 => "blocks/red_sandstone_top.png",
        183 | 188 => "blocks/planks_spruce.png",
        184 | 189 => "blocks/planks_birch.png",
        185 | 190 => "blocks/planks_jungle.png",
        186 | 191 => "blocks/planks_big_oak.png",
        187 | 192 => "blocks/planks_acacia.png",
        193 => "items/door_spruce.png",
        194 => "items/door_birch.png",
        195 => "items/door_jungle.png",
        196 => "items/door_acacia.png",
        197 => "items/door_dark_oak.png",

        // Standard Items (256..431)
        256 => "items/iron_shovel.png",
        257 => "items/iron_pickaxe.png",
        258 => "items/iron_axe.png",
        259 => "items/flint_and_steel.png",
        260 => "items/apple.png",
        261 => "items/bow_standby.png",
        262 => "items/arrow.png",
        263 => if damage == 1 { "items/charcoal.png" } else { "items/coal.png" },
        264 => "items/diamond.png",
        265 => "items/iron_ingot.png",
        266 => "items/gold_ingot.png",
        267 => "items/iron_sword.png",
        268 => "items/wood_sword.png",
        269 => "items/wood_shovel.png",
        270 => "items/wood_pickaxe.png",
        271 => "items/wood_axe.png",
        272 => "items/stone_sword.png",
        273 => "items/stone_shovel.png",
        274 => "items/stone_pickaxe.png",
        275 => "items/stone_axe.png",
        276 => "items/diamond_sword.png",
        277 => "items/diamond_shovel.png",
        278 => "items/diamond_pickaxe.png",
        279 => "items/diamond_axe.png",
        280 => "items/stick.png",
        281 => "items/bowl.png",
        282 => "items/mushroom_stew.png",
        283 => "items/gold_sword.png",
        284 => "items/gold_shovel.png",
        285 => "items/gold_pickaxe.png",
        286 => "items/gold_axe.png",
        287 => "items/string.png",
        288 => "items/feather.png",
        289 => "items/gunpowder.png",
        290 => "items/wood_hoe.png",
        291 => "items/stone_hoe.png",
        292 => "items/iron_hoe.png",
        293 => "items/diamond_hoe.png",
        294 => "items/gold_hoe.png",
        295 => "items/seeds_wheat.png",
        296 => "items/wheat.png",
        297 => "items/bread.png",
        298 => "items/leather_helmet.png",
        299 => "items/leather_chestplate.png",
        300 => "items/leather_leggings.png",
        301 => "items/leather_boots.png",
        302 => "items/chainmail_helmet.png",
        303 => "items/chainmail_chestplate.png",
        304 => "items/chainmail_leggings.png",
        305 => "items/chainmail_boots.png",
        306 => "items/iron_helmet.png",
        307 => "items/iron_chestplate.png",
        308 => "items/iron_leggings.png",
        309 => "items/iron_boots.png",
        310 => "items/diamond_helmet.png",
        311 => "items/diamond_chestplate.png",
        312 => "items/diamond_leggings.png",
        313 => "items/diamond_boots.png",
        314 => "items/gold_helmet.png",
        315 => "items/gold_chestplate.png",
        316 => "items/gold_leggings.png",
        317 => "items/gold_boots.png",
        318 => "items/flint.png",
        319 => "items/porkchop_raw.png",
        320 => "items/porkchop_cooked.png",
        321 => "items/painting.png",
        322 => "items/apple_golden.png",
        323 => "items/sign.png",
        324 => "items/door_wood.png",
        325 => "items/bucket_empty.png",
        326 => "items/bucket_water.png",
        327 => "items/bucket_lava.png",
        328 => "items/minecart_normal.png",
        329 => "items/saddle.png",
        330 => "items/door_iron.png",
        331 => "items/redstone_dust.png",
        332 => "items/snowball.png",
        333 => "items/boat.png",
        334 => "items/leather.png",
        335 => "items/bucket_milk.png",
        336 => "items/brick.png",
        337 => "items/clay_ball.png",
        338 => "items/reeds.png",
        339 => "items/paper.png",
        340 => "items/book_normal.png",
        341 => "items/slimeball.png",
        342 => "items/minecart_chest.png",
        343 => "items/minecart_furnace.png",
        344 => "items/egg.png",
        345 => "items/compass.png",
        346 => "items/fishing_rod_uncast.png",
        347 => "items/clock.png",
        348 => "items/glowstone_dust.png",
        349 => match damage {
            1 => "items/fish_salmon_raw.png",
            2 => "items/fish_clownfish_raw.png",
            3 => "items/fish_pufferfish_raw.png",
            _ => "items/fish_cod_raw.png",
        },
        350 => match damage {
            1 => "items/fish_salmon_cooked.png",
            _ => "items/fish_cod_cooked.png",
        },
        351 => match damage {
            1 => "items/dye_powder_red.png",
            2 => "items/dye_powder_green.png",
            3 => "items/dye_powder_brown.png",
            4 => "items/dye_powder_blue.png",
            5 => "items/dye_powder_purple.png",
            6 => "items/dye_powder_cyan.png",
            7 => "items/dye_powder_silver.png",
            8 => "items/dye_powder_gray.png",
            9 => "items/dye_powder_pink.png",
            10 => "items/dye_powder_lime.png",
            11 => "items/dye_powder_yellow.png",
            12 => "items/dye_powder_light_blue.png",
            13 => "items/dye_powder_magenta.png",
            14 => "items/dye_powder_orange.png",
            15 => "items/dye_powder_white.png",
            _ => "items/dye_powder_black.png",
        },
        352 => "items/bone.png",
        353 => "items/sugar.png",
        354 => "items/cake.png",
        355 => "items/bed.png",
        356 => "items/repeater.png",
        357 => "items/cookie.png",
        358 => "items/map_filled.png",
        359 => "items/shears.png",
        360 => "items/melon.png",
        361 => "items/seeds_pumpkin.png",
        362 => "items/seeds_melon.png",
        363 => "items/beef_raw.png",
        364 => "items/beef_cooked.png",
        365 => "items/chicken_raw.png",
        366 => "items/chicken_cooked.png",
        367 => "items/rotten_flesh.png",
        368 => "items/ender_pearl.png",
        369 => "items/blaze_rod.png",
        370 => "items/ghast_tear.png",
        371 => "items/gold_nugget.png",
        372 => "items/nether_wart.png",
        373 => {
            if damage & 16384 != 0 {
                "items/potion_bottle_splash.png"
            } else {
                "items/potion_bottle_drinkable.png"
            }
        }
        374 => "items/potion_bottle_empty.png",
        375 => "items/spider_eye.png",
        376 => "items/spider_eye_fermented.png",
        377 => "items/blaze_powder.png",
        378 => "items/magma_cream.png",
        379 => "items/brewing_stand.png",
        380 => "items/cauldron.png",
        381 => "items/ender_eye.png",
        382 => "items/melon_speckled.png",
        383 => "items/spawn_egg.png",
        384 => "items/experience_bottle.png",
        385 => "items/fireball.png",
        386 => "items/book_writable.png",
        387 => "items/book_written.png",
        388 => "items/emerald.png",
        389 => "items/item_frame.png",
        390 => "items/flower_pot.png",
        391 => "items/carrot.png",
        392 => "items/potato.png",
        393 => "items/potato_baked.png",
        394 => "items/potato_poisonous.png",
        395 => "items/map_empty.png",
        396 => "items/carrot_golden.png",
        397 => "blocks/dragon_egg.png",
        398 => "items/carrot_on_a_stick.png",
        399 => "items/nether_star.png",
        400 => "items/pumpkin_pie.png",
        401 => "items/fireworks.png",
        402 => "items/fireworks_charge.png",
        403 => "items/book_enchanted.png",
        404 => "items/comparator.png",
        405 => "items/netherbrick.png",
        406 => "items/quartz.png",
        407 => "items/minecart_tnt.png",
        408 => "items/minecart_hopper.png",
        409 => "items/prismarine_shard.png",
        410 => "items/prismarine_crystals.png",
        411 => "items/rabbit_raw.png",
        412 => "items/rabbit_cooked.png",
        413 => "items/rabbit_stew.png",
        414 => "items/rabbit_foot.png",
        415 => "items/rabbit_hide.png",
        416 => "items/wooden_armorstand.png",
        417 => "items/iron_horse_armor.png",
        418 => "items/gold_horse_armor.png",
        419 => "items/diamond_horse_armor.png",
        420 => "items/lead.png",
        421 => "items/name_tag.png",
        422 => "items/minecart_command_block.png",
        423 => "items/mutton_raw.png",
        424 => "items/mutton_cooked.png",
        425 => "items/banner_base.png",
        427 => "items/door_spruce.png",
        428 => "items/door_birch.png",
        429 => "items/door_jungle.png",
        430 => "items/door_acacia.png",
        431 => "items/door_dark_oak.png",

        // Music Discs
        2256 => "items/record_13.png",
        2257 => "items/record_cat.png",
        2258 => "items/record_blocks.png",
        2259 => "items/record_chirp.png",
        2260 => "items/record_far.png",
        2261 => "items/record_mall.png",
        2262 => "items/record_mellohi.png",
        2263 => "items/record_stal.png",
        2264 => "items/record_strad.png",
        2265 => "items/record_ward.png",
        2266 => "items/record_11.png",
        2267 => "items/record_wait.png",

        _ => {
            // Dynamic fallback by name: "Diamond Sword" -> "diamond_sword"
            let sanitized = name
                .to_lowercase()
                .replace(' ', "_")
                .replace('-', "_")
                .replace('\'', "");
            if !sanitized.is_empty() && sanitized != "item" {
                let assets_dir = find_assets_dir();
                let textures_dir = assets_dir.join("minecraft/textures");
                let item_candidate = format!("items/{sanitized}.png");
                if textures_dir.join(&item_candidate).exists() {
                    return Some(item_candidate);
                }
                let block_candidate = format!("blocks/{sanitized}.png");
                if textures_dir.join(&block_candidate).exists() {
                    return Some(block_candidate);
                }
            }
            return None;
        }
    };

    Some(path.to_string())
}

/// Retrieves or loads an item icon texture from disk and registers it with egui.
pub fn get_or_load_item_texture(
    ctx: &Context,
    item_id: i16,
    damage: i16,
    max_damage: i16,
    name: &str,
) -> Option<TextureId> {
    if item_id <= 0 {
        return None;
    }

    // Durability-bearing items (tools, weapons, armor) share the same texture regardless of damage
    let lookup_damage = if max_damage > 0 { 0 } else { damage };
    let key = (item_id, lookup_damage);

    let cache: ItemTextureCache = ctx.data_mut(|d| {
        d.get_temp_mut_or_insert_with(
            egui::Id::new("item_texture_cache_v1"),
            || Arc::new(Mutex::new(HashMap::new())),
        )
        .clone()
    });

    {
        let guard = cache.lock().unwrap();
        if let Some(opt_handle) = guard.get(&key) {
            return opt_handle.as_ref().map(|h| h.id());
        }
    }

    let asset_rel_path = get_item_asset_path(item_id, lookup_damage, name)?;
    let assets_dir = find_assets_dir();
    let full_path = assets_dir.join("minecraft/textures").join(&asset_rel_path);

    let handle_opt = if full_path.exists() {
        match std::fs::read(&full_path) {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let raw_bytes = rgba.into_raw();

                    // If animated strip (e.g. clock 16x1024), extract first square frame
                    let (icon_w, icon_h, frame_slice) = if h > w && w > 0 && h % w == 0 {
                        let frame_bytes = (w * w * 4) as usize;
                        if raw_bytes.len() >= frame_bytes {
                            (w, w, &raw_bytes[..frame_bytes])
                        } else {
                            (w, h, &raw_bytes[..])
                        }
                    } else {
                        (w, h, &raw_bytes[..])
                    };

                    let color_image = ColorImage::from_rgba_unmultiplied(
                        [icon_w as usize, icon_h as usize],
                        frame_slice,
                    );
                    let tex_name = format!("item_{item_id}_{lookup_damage}");
                    let handle = ctx.load_texture(tex_name, color_image, TextureOptions::NEAREST);
                    Some(handle)
                }
                Err(e) => {
                    tracing::warn!(path = ?full_path, error = %e, "Failed to decode item texture PNG");
                    None
                }
            },
            Err(e) => {
                tracing::warn!(path = ?full_path, error = %e, "Failed to read item texture file");
                None
            }
        }
    } else {
        None
    };

    let id_opt = handle_opt.as_ref().map(|h| h.id());
    let mut guard = cache.lock().unwrap();
    guard.insert(key, handle_opt);
    id_opt
}
