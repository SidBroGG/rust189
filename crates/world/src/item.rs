use std::collections::HashMap;
use std::sync::OnceLock;

/// Creative inventory tabs matching vanilla Minecraft 1.8.9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreativeTab {
    BuildingBlocks,
    DecorationBlocks,
    Redstone,
    Transportation,
    Miscellaneous,
    Food,
    Tools,
    Combat,
    Brewing,
    Materials,
}

impl CreativeTab {
    pub fn title(&self) -> &'static str {
        match self {
            Self::BuildingBlocks => "Строительные блоки",
            Self::DecorationBlocks => "Декоративные блоки",
            Self::Redstone => "Механизмы",
            Self::Transportation => "Транспорт",
            Self::Miscellaneous => "Разное",
            Self::Food => "Еда",
            Self::Tools => "Инструменты",
            Self::Combat => "Оружие и броня",
            Self::Brewing => "Зельеварение",
            Self::Materials => "Материалы",
        }
    }
}

/// Static definition of an item or block in Minecraft 1.8.9.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemDef {
    pub id: i16,
    pub name: &'static str,
    pub display_name: &'static str,
    pub max_stack: u8,
    pub max_damage: i16,
    pub has_subtypes: bool,
    pub is_block: bool,
    pub armor_slot: Option<u8>, // 0: Helmet, 1: Chestplate, 2: Leggings, 3: Boots
    pub armor_defense: u8,
    pub attack_damage: f32,
    pub tab: CreativeTab,
}

impl ItemDef {
    pub const fn new(
        id: i16,
        name: &'static str,
        display_name: &'static str,
        max_stack: u8,
        max_damage: i16,
        has_subtypes: bool,
        is_block: bool,
        tab: CreativeTab,
    ) -> Self {
        Self {
            id,
            name,
            display_name,
            max_stack,
            max_damage,
            has_subtypes,
            is_block,
            armor_slot: None,
            armor_defense: 0,
            attack_damage: 1.0,
            tab,
        }
    }

    pub const fn weapon(
        id: i16,
        name: &'static str,
        display_name: &'static str,
        max_damage: i16,
        damage: f32,
    ) -> Self {
        Self {
            id,
            name,
            display_name,
            max_stack: 1,
            max_damage,
            has_subtypes: false,
            is_block: false,
            armor_slot: None,
            armor_defense: 0,
            attack_damage: damage,
            tab: CreativeTab::Combat,
        }
    }

    pub const fn tool(
        id: i16,
        name: &'static str,
        display_name: &'static str,
        max_damage: i16,
        damage: f32,
    ) -> Self {
        Self {
            id,
            name,
            display_name,
            max_stack: 1,
            max_damage,
            has_subtypes: false,
            is_block: false,
            armor_slot: None,
            armor_defense: 0,
            attack_damage: damage,
            tab: CreativeTab::Tools,
        }
    }

    pub const fn armor(
        id: i16,
        name: &'static str,
        display_name: &'static str,
        max_damage: i16,
        slot: u8,
        defense: u8,
    ) -> Self {
        Self {
            id,
            name,
            display_name,
            max_stack: 1,
            max_damage,
            has_subtypes: false,
            is_block: false,
            armor_slot: Some(slot),
            armor_defense: defense,
            attack_damage: 1.0,
            tab: CreativeTab::Combat,
        }
    }
}

/// Master table of registered 1.8.9 items and blocks.
pub static ITEMS: &[ItemDef] = &[
    // --- Blocks (IDs 1..255) ---
    ItemDef::new(1, "stone", "Камень", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(2, "grass", "Блок травы", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(3, "dirt", "Земля", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(4, "cobblestone", "Булыжник", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(5, "planks", "Доски", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(6, "sapling", "Саженец", 64, 0, true, true, CreativeTab::DecorationBlocks),
    ItemDef::new(7, "bedrock", "Бедрок", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(12, "sand", "Песок", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(13, "gravel", "Гравий", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(14, "gold_ore", "Золотая руда", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(15, "iron_ore", "Железная руда", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(16, "coal_ore", "Угольная руда", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(17, "log", "Древесина", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(18, "leaves", "Листва", 64, 0, true, true, CreativeTab::DecorationBlocks),
    ItemDef::new(19, "sponge", "Губка", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(20, "glass", "Стекло", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(21, "lapis_ore", "Лазуритовая руда", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(22, "lapis_block", "Лазуритовый блок", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(23, "dispenser", "Раздатчик", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(24, "sandstone", "Песчаник", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(25, "noteblock", "Музыкальный блок", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(27, "golden_rail", "Энергорельсы", 64, 0, false, true, CreativeTab::Transportation),
    ItemDef::new(28, "detector_rail", "Рельсы с датчиком", 64, 0, false, true, CreativeTab::Transportation),
    ItemDef::new(29, "sticky_piston", "Липкий поршень", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(30, "web", "Паутина", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(31, "tallgrass", "Высокая трава", 64, 0, true, true, CreativeTab::DecorationBlocks),
    ItemDef::new(32, "deadbush", "Мёртвый куст", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(33, "piston", "Поршень", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(35, "wool", "Шерсть", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(37, "yellow_flower", "Одуванчик", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(38, "red_flower", "Цветок", 64, 0, true, true, CreativeTab::DecorationBlocks),
    ItemDef::new(39, "brown_mushroom", "Коричневый гриб", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(40, "red_mushroom", "Красный гриб", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(41, "gold_block", "Золотой блок", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(42, "iron_block", "Железный блок", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(44, "stone_slab", "Каменная плита", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(45, "brick_block", "Кирпичи", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(46, "tnt", "Динамит", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(47, "bookshelf", "Книжная полка", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(48, "mossy_cobblestone", "Замшелый булыжник", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(49, "obsidian", "Обсидиан", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(50, "torch", "Факел", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(52, "mob_spawner", "Рассадник монстров", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(53, "oak_stairs", "Дубовые ступени", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(54, "chest", "Сундук", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(56, "diamond_ore", "Алмазная руда", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(57, "diamond_block", "Алмазный блок", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(58, "crafting_table", "Верстак", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(61, "furnace", "Печь", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(65, "ladder", "Лестница", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(66, "rail", "Рельсы", 64, 0, false, true, CreativeTab::Transportation),
    ItemDef::new(67, "stone_stairs", "Булыжные ступени", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(69, "lever", "Рычаг", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(70, "stone_pressure_plate", "Каменная нажимная плита", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(72, "wooden_pressure_plate", "Деревянная нажимная плита", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(73, "redstone_ore", "Красная руда", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(76, "redstone_torch", "Красный факел", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(77, "stone_button", "Каменная кнопка", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(78, "snow_layer", "Снег", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(79, "ice", "Лёд", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(80, "snow", "Блок снега", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(81, "cactus", "Кактус", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(82, "clay", "Глина", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(84, "jukebox", "Проигрыватель", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(85, "fence", "Дубовый забор", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(86, "pumpkin", "Тыква", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(87, "netherrack", "Незерак", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(88, "soul_sand", "Песок душ", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(89, "glowstone", "Светокамень", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(91, "lit_pumpkin", "Светильник Джека", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(95, "stained_glass", "Окрашенное стекло", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(96, "trapdoor", "Люк", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(98, "stonebrick", "Каменный кирпич", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(101, "iron_bars", "Железные прутья", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(102, "glass_pane", "Стеклянная панель", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(103, "melon_block", "Арбуз", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(106, "vine", "Лианы", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(107, "fence_gate", "Дубовая калитка", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(108, "brick_stairs", "Кирпичные ступени", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(109, "stone_brick_stairs", "Ступени из каменного кирпича", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(110, "mycelium", "Мицелий", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(111, "waterlily", "Кувшинка", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(112, "nether_brick", "Незерский кирпич", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(113, "nether_brick_fence", "Незерский забор", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(114, "nether_brick_stairs", "Ступени из незерского кирпича", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(116, "enchanting_table", "Стол зачаровывания", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(120, "end_portal_frame", "Рамка портала в Край", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(121, "end_stone", "Эндерняк", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(123, "redstone_lamp", "Лампа", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(126, "wooden_slab", "Деревянная плита", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(128, "sandstone_stairs", "Песчаниковые ступени", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(129, "emerald_ore", "Изумрудная руда", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(130, "ender_chest", "Эндер-сундук", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(131, "tripwire_hook", "Натяжной датчик", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(133, "emerald_block", "Изумрудный блок", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(134, "spruce_stairs", "Еловые ступени", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(135, "birch_stairs", "Берёзовые ступени", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(136, "jungle_stairs", "Ступени из тропического дерева", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(137, "command_block", "Командный блок", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(138, "beacon", "Маяк", 64, 0, false, true, CreativeTab::Miscellaneous),
    ItemDef::new(139, "cobblestone_wall", "Булыжная ограда", 64, 0, true, true, CreativeTab::DecorationBlocks),
    ItemDef::new(143, "wooden_button", "Деревянная кнопка", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(145, "anvil", "Наковальня", 64, 0, true, true, CreativeTab::DecorationBlocks),
    ItemDef::new(146, "trapped_chest", "Сундук-ловушка", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(147, "light_weighted_pressure_plate", "Весовая пластина (золото)", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(148, "heavy_weighted_pressure_plate", "Весовая пластина (железо)", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(151, "daylight_detector", "Датчик дневного света", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(152, "redstone_block", "Блок красного камня", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(153, "quartz_ore", "Кварцевая руда", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(154, "hopper", "Воронка", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(155, "quartz_block", "Кварцевый блок", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(156, "quartz_stairs", "Кварцевые ступени", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(157, "activator_rail", "Активирующие рельсы", 64, 0, false, true, CreativeTab::Transportation),
    ItemDef::new(158, "dropper", "Выбрасыватель", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(159, "stained_hardened_clay", "Окрашенная глина", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(160, "stained_glass_pane", "Окрашенная стеклянная панель", 64, 0, true, true, CreativeTab::DecorationBlocks),
    ItemDef::new(161, "leaves2", "Листва акации/тёмного дуба", 64, 0, true, true, CreativeTab::DecorationBlocks),
    ItemDef::new(162, "log2", "Древесина акации/тёмного дуба", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(163, "acacia_stairs", "Ступени из акации", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(164, "dark_oak_stairs", "Ступени из тёмного дуба", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(165, "slime", "Блок слизи", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(166, "barrier", "Барьер", 64, 0, false, true, CreativeTab::Miscellaneous),
    ItemDef::new(167, "iron_trapdoor", "Железный люк", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(168, "prismarine", "Призмарин", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(169, "sea_lantern", "Морской фонарь", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(170, "hay_block", "Стог сена", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(171, "carpet", "Ковёр", 64, 0, true, true, CreativeTab::DecorationBlocks),
    ItemDef::new(172, "hardened_clay", "Обожжённая глина", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(173, "coal_block", "Угольный блок", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(174, "packed_ice", "Плотный лёд", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(175, "double_plant", "Высокие растения", 64, 0, true, true, CreativeTab::DecorationBlocks),
    ItemDef::new(179, "red_sandstone", "Красный песчаник", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(180, "red_sandstone_stairs", "Ступени из красного песчаника", 64, 0, false, true, CreativeTab::BuildingBlocks),
    ItemDef::new(182, "stone_slab2", "Плита из красного песчаника", 64, 0, true, true, CreativeTab::BuildingBlocks),
    ItemDef::new(183, "spruce_fence_gate", "Еловая калитка", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(184, "birch_fence_gate", "Берёзовая калитка", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(185, "jungle_fence_gate", "Калитка из тропического дерева", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(186, "dark_oak_fence_gate", "Калитка из тёмного дуба", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(187, "acacia_fence_gate", "Калитка из акации", 64, 0, false, true, CreativeTab::Redstone),
    ItemDef::new(188, "spruce_fence", "Еловый забор", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(189, "birch_fence", "Берёзовый забор", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(190, "jungle_fence", "Забор из тропического дерева", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(191, "dark_oak_fence", "Забор из тёмного дуба", 64, 0, false, true, CreativeTab::DecorationBlocks),
    ItemDef::new(192, "acacia_fence", "Забор из акации", 64, 0, false, true, CreativeTab::DecorationBlocks),

    // --- Items (Tools, Weapons, Armor, Materials, Food) (IDs 256..431) ---
    ItemDef::tool(256, "iron_shovel", "Железная лопата", 250, 4.5),
    ItemDef::tool(257, "iron_pickaxe", "Железная кирка", 250, 4.0),
    ItemDef::tool(258, "iron_axe", "Железный топор", 250, 5.0),
    ItemDef::tool(259, "flint_and_steel", "Огниво", 64, 1.0),
    ItemDef::new(260, "apple", "Яблоко", 64, 0, false, false, CreativeTab::Food),
    ItemDef::weapon(261, "bow", "Лук", 384, 1.0),
    ItemDef::new(262, "arrow", "Стрела", 64, 0, false, false, CreativeTab::Combat),
    ItemDef::new(263, "coal", "Уголь", 64, 0, true, false, CreativeTab::Materials),
    ItemDef::new(264, "diamond", "Алмаз", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(265, "iron_ingot", "Железный слиток", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(266, "gold_ingot", "Золотой слиток", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::weapon(267, "iron_sword", "Железный меч", 250, 6.0),
    ItemDef::weapon(268, "wooden_sword", "Деревянный меч", 59, 4.0),
    ItemDef::tool(269, "wooden_shovel", "Деревянная лопата", 59, 2.5),
    ItemDef::tool(270, "wooden_pickaxe", "Деревянная кирка", 59, 2.0),
    ItemDef::tool(271, "wooden_axe", "Деревянный топор", 59, 3.0),
    ItemDef::weapon(272, "stone_sword", "Каменный меч", 131, 5.0),
    ItemDef::tool(273, "stone_shovel", "Каменная лопата", 131, 3.5),
    ItemDef::tool(274, "stone_pickaxe", "Каменная кирка", 131, 3.0),
    ItemDef::tool(275, "stone_axe", "Каменный топор", 131, 4.0),
    ItemDef::weapon(276, "diamond_sword", "Алмазный меч", 1561, 7.0),
    ItemDef::tool(277, "diamond_shovel", "Алмазная лопата", 1561, 5.5),
    ItemDef::tool(278, "diamond_pickaxe", "Алмазная кирка", 1561, 5.0),
    ItemDef::tool(279, "diamond_axe", "Алмазный топор", 1561, 6.0),
    ItemDef::new(280, "stick", "Палка", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(281, "bowl", "Миска", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(282, "mushroom_stew", "Тушёные грибы", 1, 0, false, false, CreativeTab::Food),
    ItemDef::weapon(283, "golden_sword", "Золотой меч", 32, 4.0),
    ItemDef::tool(284, "golden_shovel", "Золотая лопата", 32, 2.5),
    ItemDef::tool(285, "golden_pickaxe", "Золотая кирка", 32, 2.0),
    ItemDef::tool(286, "golden_axe", "Золотой топор", 32, 3.0),
    ItemDef::new(287, "string", "Нить", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(288, "feather", "Перо", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(289, "gunpowder", "Порох", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::tool(290, "wooden_hoe", "Деревянная мотыга", 59, 1.0),
    ItemDef::tool(291, "stone_hoe", "Каменная мотыга", 131, 1.0),
    ItemDef::tool(292, "iron_hoe", "Железная мотыга", 250, 1.0),
    ItemDef::tool(293, "diamond_hoe", "Алмазная мотыга", 1561, 1.0),
    ItemDef::tool(294, "golden_hoe", "Золотая мотыга", 32, 1.0),
    ItemDef::new(295, "wheat_seeds", "Семена пшеницы", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(296, "wheat", "Пшеница", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(297, "bread", "Хлеб", 64, 0, false, false, CreativeTab::Food),
    ItemDef::armor(298, "leather_helmet", "Кожаный шлем", 55, 0, 1),
    ItemDef::armor(299, "leather_chestplate", "Кожаная куртка", 80, 1, 3),
    ItemDef::armor(300, "leather_leggings", "Кожаные штаны", 75, 2, 2),
    ItemDef::armor(301, "leather_boots", "Кожаные ботинки", 65, 3, 1),
    ItemDef::armor(302, "chainmail_helmet", "Кольчужный шлем", 165, 0, 2),
    ItemDef::armor(303, "chainmail_chestplate", "Кольчужный нагрудник", 240, 1, 5),
    ItemDef::armor(304, "chainmail_leggings", "Кольчужные поножи", 225, 2, 4),
    ItemDef::armor(305, "chainmail_boots", "Кольчужные ботинки", 195, 3, 1),
    ItemDef::armor(306, "iron_helmet", "Железный шлем", 165, 0, 2),
    ItemDef::armor(307, "iron_chestplate", "Железный нагрудник", 240, 1, 6),
    ItemDef::armor(308, "iron_leggings", "Железные поножи", 225, 2, 5),
    ItemDef::armor(309, "iron_boots", "Железные ботинки", 195, 3, 2),
    ItemDef::armor(310, "diamond_helmet", "Алмазный шлем", 363, 0, 3),
    ItemDef::armor(311, "diamond_chestplate", "Алмазный нагрудник", 528, 1, 8),
    ItemDef::armor(312, "diamond_leggings", "Алмазные поножи", 495, 2, 6),
    ItemDef::armor(313, "diamond_boots", "Алмазные ботинки", 429, 3, 3),
    ItemDef::armor(314, "golden_helmet", "Золотой шлем", 77, 0, 2),
    ItemDef::armor(315, "golden_chestplate", "Золотой нагрудник", 112, 1, 5),
    ItemDef::armor(316, "golden_leggings", "Золотые поножи", 105, 2, 3),
    ItemDef::armor(317, "golden_boots", "Золотые ботинки", 91, 3, 1),
    ItemDef::new(318, "flint", "Кремень", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(319, "porkchop", "Сырая свинина", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(320, "cooked_porkchop", "Жареная свинина", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(321, "painting", "Картина", 64, 0, false, false, CreativeTab::DecorationBlocks),
    ItemDef::new(322, "golden_apple", "Золотое яблоко", 64, 0, true, false, CreativeTab::Food),
    ItemDef::new(323, "sign", "Табличка", 16, 0, false, false, CreativeTab::DecorationBlocks),
    ItemDef::new(324, "wooden_door", "Дубовая дверь", 64, 0, false, false, CreativeTab::Redstone),
    ItemDef::new(325, "bucket", "Ведро", 16, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(326, "water_bucket", "Ведро воды", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(327, "lava_bucket", "Ведро лавы", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(328, "minecart", "Вагонетка", 1, 0, false, false, CreativeTab::Transportation),
    ItemDef::new(329, "saddle", "Седло", 1, 0, false, false, CreativeTab::Transportation),
    ItemDef::new(330, "iron_door", "Железная дверь", 64, 0, false, false, CreativeTab::Redstone),
    ItemDef::new(331, "redstone", "Редстоун", 64, 0, false, false, CreativeTab::Redstone),
    ItemDef::new(332, "snowball", "Снежок", 16, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(333, "boat", "Лодка", 1, 0, false, false, CreativeTab::Transportation),
    ItemDef::new(334, "leather", "Кожа", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(335, "milk_bucket", "Ведро молока", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(336, "brick", "Кирпич", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(337, "clay_ball", "Комок глины", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(338, "reeds", "Сахарный тростник", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(339, "paper", "Бумага", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(340, "book", "Книга", 64, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(341, "slime_ball", "Сгусток слизи", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(342, "chest_minecart", "Грузовая вагонетка", 1, 0, false, false, CreativeTab::Transportation),
    ItemDef::new(343, "furnace_minecart", "Самоходная вагонетка", 1, 0, false, false, CreativeTab::Transportation),
    ItemDef::new(344, "egg", "Яйцо", 16, 0, false, false, CreativeTab::Materials),
    ItemDef::new(345, "compass", "Компас", 64, 0, false, false, CreativeTab::Tools),
    ItemDef::tool(346, "fishing_rod", "Удочка", 64, 1.0),
    ItemDef::new(347, "clock", "Часы", 64, 0, false, false, CreativeTab::Tools),
    ItemDef::new(348, "glowstone_dust", "Светокаменная пыль", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(349, "fish", "Сырая рыба", 64, 0, true, false, CreativeTab::Food),
    ItemDef::new(350, "cooked_fish", "Жареная рыба", 64, 0, true, false, CreativeTab::Food),
    ItemDef::new(351, "dye", "Краситель", 64, 0, true, false, CreativeTab::Materials),
    ItemDef::new(352, "bone", "Кость", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(353, "sugar", "Сахар", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(354, "cake", "Торт", 1, 0, false, false, CreativeTab::Food),
    ItemDef::new(355, "bed", "Кровать", 1, 0, false, false, CreativeTab::DecorationBlocks),
    ItemDef::new(356, "repeater", "Повторитель", 64, 0, false, false, CreativeTab::Redstone),
    ItemDef::new(357, "cookie", "Печенье", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(358, "filled_map", "Карта местности", 64, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::tool(359, "shears", "Ножницы", 238, 1.0),
    ItemDef::new(360, "melon", "Ломтик арбуза", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(361, "pumpkin_seeds", "Семена тыквы", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(362, "melon_seeds", "Семена арбуза", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(363, "beef", "Сырая говядина", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(364, "cooked_beef", "Стейк", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(365, "chicken", "Сырая курятина", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(366, "cooked_chicken", "Жареная курятина", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(367, "rotten_flesh", "Гнилая плоть", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(368, "ender_pearl", "Эндер-жемчуг", 16, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(369, "blaze_rod", "Огненный стержень", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(370, "ghast_tear", "Слеза гаста", 64, 0, false, false, CreativeTab::Brewing),
    ItemDef::new(371, "gold_nugget", "Кусочек золота", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(372, "nether_wart", "Незерский нарост", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(373, "potion", "Зелье", 1, 0, true, false, CreativeTab::Brewing),
    ItemDef::new(374, "glass_bottle", "Колба", 64, 0, false, false, CreativeTab::Brewing),
    ItemDef::new(375, "spider_eye", "Паучий глаз", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(376, "fermented_spider_eye", "Маринованный паучий глаз", 64, 0, false, false, CreativeTab::Brewing),
    ItemDef::new(377, "blaze_powder", "Огненный порошок", 64, 0, false, false, CreativeTab::Brewing),
    ItemDef::new(378, "magma_cream", "Сгусток магмы", 64, 0, false, false, CreativeTab::Brewing),
    ItemDef::new(379, "brewing_stand", "Зельеварка", 64, 0, false, false, CreativeTab::Brewing),
    ItemDef::new(380, "cauldron", "Котёл", 64, 0, false, false, CreativeTab::Brewing),
    ItemDef::new(381, "ender_eye", "Око Эндера", 64, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(382, "speckled_melon", "Сверкающий ломтик арбуза", 64, 0, false, false, CreativeTab::Brewing),
    ItemDef::new(383, "spawn_egg", "Яйцо призыва", 64, 0, true, false, CreativeTab::Miscellaneous),
    ItemDef::new(384, "experience_bottle", "Пузырёк опыта", 64, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(385, "fire_charge", "Огненный шар", 64, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(386, "writable_book", "Книга и перо", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(387, "written_book", "Завершённая книга", 16, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(388, "emerald", "Изумруд", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(389, "item_frame", "Рамка", 64, 0, false, false, CreativeTab::DecorationBlocks),
    ItemDef::new(390, "flower_pot", "Цветочный горшок", 64, 0, false, false, CreativeTab::DecorationBlocks),
    ItemDef::new(391, "carrot", "Морковь", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(392, "potato", "Картофель", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(393, "baked_potato", "Печёный картофель", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(394, "poisonous_potato", "Ядовитый картофель", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(395, "map", "Чистая карта", 64, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(396, "golden_carrot", "Золотая морковь", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(397, "skull", "Голова", 64, 0, true, false, CreativeTab::DecorationBlocks),
    ItemDef::tool(398, "carrot_on_a_stick", "Удочка с морковью", 25, 1.0),
    ItemDef::new(399, "nether_star", "Звезда Незера", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(400, "pumpkin_pie", "Тыквенный пирог", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(401, "fireworks", "Фейерверк", 64, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(402, "firework_charge", "Пиротехническая звёздочка", 64, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(403, "enchanted_book", "Зачарованная книга", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(404, "comparator", "Компаратор", 64, 0, false, false, CreativeTab::Redstone),
    ItemDef::new(405, "netherbrick", "Незерский кирпич (предмет)", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(406, "quartz", "Кварц Нижнего мира", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(407, "tnt_minecart", "Вагонетка с динамитом", 1, 0, false, false, CreativeTab::Transportation),
    ItemDef::new(408, "hopper_minecart", "Загрузочная вагонетка", 1, 0, false, false, CreativeTab::Transportation),
    ItemDef::new(409, "prismarine_shard", "Осколок призмарина", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(410, "prismarine_crystals", "Кристалл призмарина", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(411, "rabbit", "Сырая крольчатина", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(412, "cooked_rabbit", "Жареная крольчатина", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(413, "rabbit_stew", "Кроличье рагу", 1, 0, false, false, CreativeTab::Food),
    ItemDef::new(414, "rabbit_foot", "Кроличья лапка", 64, 0, false, false, CreativeTab::Brewing),
    ItemDef::new(415, "rabbit_hide", "Кроличья шкурка", 64, 0, false, false, CreativeTab::Materials),
    ItemDef::new(416, "armor_stand", "Стойка для брони", 16, 0, false, false, CreativeTab::DecorationBlocks),
    ItemDef::new(417, "iron_horse_armor", "Железная конская броня", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(418, "golden_horse_armor", "Золотая конская броня", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(419, "diamond_horse_armor", "Алмазная конская броня", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(420, "lead", "Поводок", 64, 0, false, false, CreativeTab::Tools),
    ItemDef::new(421, "name_tag", "Бирка", 64, 0, false, false, CreativeTab::Tools),
    ItemDef::new(423, "mutton", "Сырая баранина", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(424, "cooked_mutton", "Жареная баранина", 64, 0, false, false, CreativeTab::Food),
    ItemDef::new(425, "banner", "Флаг", 16, 0, true, false, CreativeTab::DecorationBlocks),
    ItemDef::new(427, "spruce_door", "Еловая дверь", 64, 0, false, false, CreativeTab::Redstone),
    ItemDef::new(428, "birch_door", "Берёзовая дверь", 64, 0, false, false, CreativeTab::Redstone),
    ItemDef::new(429, "jungle_door", "Дверь из тропического дерева", 64, 0, false, false, CreativeTab::Redstone),
    ItemDef::new(430, "acacia_door", "Дверь из акации", 64, 0, false, false, CreativeTab::Redstone),
    ItemDef::new(431, "dark_oak_door", "Дверь из тёмного дуба", 64, 0, false, false, CreativeTab::Redstone),

    // Records
    ItemDef::new(2256, "record_13", "Пластинка 13", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2257, "record_cat", "Пластинка cat", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2258, "record_blocks", "Пластинка blocks", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2259, "record_chirp", "Пластинка chirp", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2260, "record_far", "Пластинка far", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2261, "record_mall", "Пластинка mall", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2262, "record_mellohi", "Пластинка mellohi", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2263, "record_stal", "Пластинка stal", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2264, "record_strad", "Пластинка strad", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2265, "record_ward", "Пластинка ward", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2266, "record_11", "Пластинка 11", 1, 0, false, false, CreativeTab::Miscellaneous),
    ItemDef::new(2267, "record_wait", "Пластинка wait", 1, 0, false, false, CreativeTab::Miscellaneous),
];

static ITEM_INDEX: OnceLock<HashMap<i16, &'static ItemDef>> = OnceLock::new();

fn get_index() -> &'static HashMap<i16, &'static ItemDef> {
    ITEM_INDEX.get_or_init(|| {
        let mut map = HashMap::with_capacity(ITEMS.len());
        for item in ITEMS {
            map.insert(item.id, item);
        }
        map
    })
}

/// Retrieves the item definition for a given item ID.
pub fn get_item_def(id: i16) -> Option<&'static ItemDef> {
    get_index().get(&id).copied()
}

/// Returns the maximum stack size for the item (defaults to 64 for unknown items).
pub fn get_item_max_stack(id: i16) -> u8 {
    get_item_def(id).map_or(64, |item| item.max_stack)
}

/// Returns the maximum durability for the item (0 if non-damageable).
pub fn get_item_max_damage(id: i16) -> i16 {
    get_item_def(id).map_or(0, |item| item.max_damage)
}

/// Subtype name variations for items with metadata.
pub fn get_subtype_name(item_id: i16, damage: i16) -> Option<&'static str> {
    match item_id {
        // Stone
        1 => match damage {
            1 => Some("Гранит"),
            2 => Some("Полированный гранит"),
            3 => Some("Диорит"),
            4 => Some("Полированный диорит"),
            5 => Some("Андезит"),
            6 => Some("Полированный андезит"),
            _ => None,
        },
        // Dirt
        3 => match damage {
            1 => Some("Каменистая земля"),
            2 => Some("Подзол"),
            _ => None,
        },
        // Planks & Saplings & Logs & Leaves & Slabs
        5 | 6 | 17 | 18 | 126 => match damage & 7 {
            1 => Some("Ель"),
            2 => Some("Берёза"),
            3 => Some("Тропическое дерево"),
            4 => Some("Акация"),
            5 => Some("Тёмный дуб"),
            _ => Some("Дуб"),
        },
        // Sand
        12 => if damage == 1 { Some("Красный песок") } else { None },
        // Sponge
        19 => if damage == 1 { Some("Мокрая губка") } else { None },
        // Sandstone
        24 => match damage {
            1 => Some("Декоративный песчаник"),
            2 => Some("Гладкий песчаник"),
            _ => None,
        },
        // Flowers
        38 => match damage {
            1 => Some("Синяя орхидея"),
            2 => Some("Лук-батун"),
            3 => Some("Хаустония серая"),
            4 => Some("Красный тюльпан"),
            5 => Some("Оранжевый тюльпан"),
            6 => Some("Белый тюльпан"),
            7 => Some("Розовый тюльпан"),
            8 => Some("Ромашка"),
            _ => Some("Мак"),
        },
        // Stone Slabs
        44 => match damage & 7 {
            1 => Some("Песчаниковая плита"),
            2 => Some("Деревянная плита"),
            3 => Some("Булыжная плита"),
            4 => Some("Кирпичная плита"),
            5 => Some("Плита из каменного кирпича"),
            6 => Some("Незерская плита"),
            7 => Some("Кварцевая плита"),
            _ => Some("Каменная плита"),
        },
        // Stonebrick
        98 => match damage {
            1 => Some("Замшелый каменный кирпич"),
            2 => Some("Потрескавшийся каменный кирпич"),
            3 => Some("Резной каменный кирпич"),
            _ => Some("Каменный кирпич"),
        },
        // Wool & Carpet & Stained Clay & Stained Glass & Stained Glass Pane & Banner
        35 | 95 | 159 | 160 | 171 | 425 => match damage & 15 {
            0 => Some("Белый"),
            1 => Some("Оранжевый"),
            2 => Some("Пурпурный"),
            3 => Some("Светло-синий"),
            4 => Some("Жёлтый"),
            5 => Some("Лаймовый"),
            6 => Some("Розовый"),
            7 => Some("Серый"),
            8 => Some("Светло-серый"),
            9 => Some("Бирюзовый"),
            10 => Some("Фиолетовый"),
            11 => Some("Синий"),
            12 => Some("Коричневый"),
            13 => Some("Зелёный"),
            14 => Some("Красный"),
            _ => Some("Чёрный"),
        },
        // Cobblestone Wall
        139 => match damage {
            1 => Some("Замшелая булыжная ограда"),
            _ => Some("Булыжная ограда"),
        },
        // Anvil
        145 => match damage {
            1 => Some("Повреждённая наковальня"),
            2 => Some("Сильно повреждённая наковальня"),
            _ => Some("Наковальня"),
        },
        // Quartz Block
        155 => match damage {
            1 => Some("Резной кварцевый блок"),
            2 => Some("Колонный кварцевый блок"),
            _ => Some("Кварцевый блок"),
        },
        // Prismarine
        168 => match damage {
            1 => Some("Призмариновый кирпич"),
            2 => Some("Тёмный призмарин"),
            _ => Some("Призмарин"),
        },
        // Double Plant
        175 => match damage {
            1 => Some("Сирень"),
            2 => Some("Высокая трава"),
            3 => Some("Большой папоротник"),
            4 => Some("Розовый куст"),
            5 => Some("Пион"),
            _ => Some("Подсолнух"),
        },
        // Red Sandstone
        179 => match damage {
            1 => Some("Декоративный красный песчаник"),
            2 => Some("Гладкий красный песчаник"),
            _ => Some("Красный песчаник"),
        },
        // Coal
        263 => match damage {
            1 => Some("Древесный уголь"),
            _ => Some("Уголь"),
        },
        // Fish
        349 => match damage {
            1 => Some("Сырой лосось"),
            2 => Some("Рыба-клоун"),
            3 => Some("Иглобрюх"),
            _ => Some("Сырая рыба"),
        },
        350 => match damage {
            1 => Some("Жареный лосось"),
            _ => Some("Жареная рыба"),
        },
        // Dye
        351 => match damage & 15 {
            0 => Some("Чернильный мешок"),
            1 => Some("Красный краситель"),
            2 => Some("Кактусовый зелёный"),
            3 => Some("Какао-бобы"),
            4 => Some("Лазурит"),
            5 => Some("Фиолетовый краситель"),
            6 => Some("Бирюзовый краситель"),
            7 => Some("Светло-серый краситель"),
            8 => Some("Серый краситель"),
            9 => Some("Розовый краситель"),
            10 => Some("Лаймовый краситель"),
            11 => Some("Жёлтый краситель"),
            12 => Some("Светло-синий краситель"),
            13 => Some("Пурпурный краситель"),
            14 => Some("Оранжевый краситель"),
            _ => Some("Костная мука"),
        },
        // Golden Apple
        322 => if damage == 1 { Some("Зачарованное золотое яблоко") } else { None },
        // Potion
        373 => match damage {
            0 => Some("Колба воды"),
            16 => Some("Неловкое зелье"),
            8193 => Some("Зелье регенерации (0:45)"),
            8194 => Some("Зелье стремительности (3:00)"),
            8195 => Some("Зелье огнестойкости (3:00)"),
            8196 => Some("Зелье отравления (0:45)"),
            8197 => Some("Зелье исцеления"),
            8198 => Some("Зелье ночного зрения (3:00)"),
            8200 => Some("Зелье слабости (1:30)"),
            8201 => Some("Зелье силы (3:00)"),
            8202 => Some("Зелье замедления (1:30)"),
            8204 => Some("Зелье урона"),
            8205 => Some("Зелье подводного дыхания (3:00)"),
            8206 => Some("Зелье невидимости (3:00)"),
            16385 => Some("Взрывное зелье регенерации (0:33)"),
            16386 => Some("Взрывное зелье стремительности (2:15)"),
            16388 => Some("Взрывное зелье отравления (0:33)"),
            16389 => Some("Взрывное зелье исцеления"),
            16393 => Some("Взрывное зелье силы (2:15)"),
            16396 => Some("Взрывное зелье урона"),
            _ => None,
        },
        // Spawn Egg
        383 => match damage {
            50 => Some("Яйцо призыва: Крипер"),
            51 => Some("Яйцо призыва: Скелет"),
            52 => Some("Яйцо призыва: Паук"),
            54 => Some("Яйцо призыва: Зомби"),
            55 => Some("Яйцо призыва: Слизень"),
            56 => Some("Яйцо призыва: Гаст"),
            57 => Some("Яйцо призыва: Свинозомби"),
            58 => Some("Яйцо призыва: Эндермен"),
            59 => Some("Яйцо призыва: Пещерный паук"),
            60 => Some("Яйцо призыва: Чешуйница"),
            61 => Some("Яйцо призыва: Ифрит"),
            62 => Some("Яйцо призыва: Лавовый куб"),
            65 => Some("Яйцо призыва: Летучая мышь"),
            66 => Some("Яйцо призыва: Ведьма"),
            67 => Some("Яйцо призыва: Эндермит"),
            68 => Some("Яйцо призыва: Страж"),
            90 => Some("Яйцо призыва: Свинья"),
            91 => Some("Яйцо призыва: Овца"),
            92 => Some("Яйцо призыва: Корова"),
            93 => Some("Яйцо призыва: Курица"),
            94 => Some("Яйцо призыва: Спрут"),
            95 => Some("Яйцо призыва: Волк"),
            96 => Some("Яйцо призыва: Грибная корова"),
            98 => Some("Яйцо призыва: Оцелот"),
            100 => Some("Яйцо призыва: Лошадь"),
            101 => Some("Яйцо призыва: Кролик"),
            120 => Some("Яйцо призыва: Деревенский житель"),
            _ => None,
        },
        // Skull
        397 => match damage {
            1 => Some("Череп скелета-иссушителя"),
            2 => Some("Голова зомби"),
            3 => Some("Голова игрока"),
            4 => Some("Голова крипера"),
            _ => Some("Череп скелета"),
        },
        _ => None,
    }
}

/// Returns user-friendly localized display name for an item stack.
pub fn get_item_name(id: i16, damage: i16) -> String {
    if let Some(sub) = get_subtype_name(id, damage) {
        if matches!(id, 35 | 95 | 159 | 160 | 171 | 425) {
            let base = get_item_def(id).map_or("Предмет", |i| i.display_name);
            return format!("{sub} {base}");
        }
        if matches!(
            id,
            1 | 3 | 5 | 6 | 17 | 18 | 24 | 38 | 44 | 98 | 126 | 139 | 145 | 155 | 168 | 175 | 179 | 263 | 322 | 349 | 350 | 351 | 373 | 383 | 397
        ) {
            return sub.to_string();
        }
    }

    get_item_def(id)
        .map(|i| i.display_name.to_string())
        .unwrap_or_else(|| format!("Предмет #{id}"))
}

/// Returns the list of items for a given creative tab.
pub fn get_creative_tab_items(tab: CreativeTab) -> Vec<(i16, i16)> {
    let mut items = Vec::new();
    for item in ITEMS {
        if item.tab != tab {
            continue;
        }

        if matches!(item.id, 35 | 95 | 159 | 160 | 171 | 425) {
            // Add all 16 colors
            for meta in 0..16 {
                items.push((item.id, meta));
            }
        } else if item.id == 1 {
            for meta in 0..=6 {
                items.push((item.id, meta));
            }
        } else if item.id == 3 {
            for meta in 0..=2 {
                items.push((item.id, meta));
            }
        } else if matches!(item.id, 5 | 6 | 17 | 126) {
            for meta in 0..=5 {
                items.push((item.id, meta));
            }
        } else if item.id == 12 {
            items.push((item.id, 0));
            items.push((item.id, 1));
        } else if item.id == 18 {
            for meta in 0..=3 {
                items.push((item.id, meta));
            }
        } else if item.id == 19 {
            items.push((item.id, 0));
            items.push((item.id, 1));
        } else if item.id == 24 || item.id == 179 {
            for meta in 0..=2 {
                items.push((item.id, meta));
            }
        } else if item.id == 38 {
            for meta in 0..=8 {
                items.push((item.id, meta));
            }
        } else if item.id == 44 {
            for meta in 0..=7 {
                items.push((item.id, meta));
            }
        } else if item.id == 98 {
            for meta in 0..=3 {
                items.push((item.id, meta));
            }
        } else if item.id == 139 {
            items.push((item.id, 0));
            items.push((item.id, 1));
        } else if item.id == 145 {
            for meta in 0..=2 {
                items.push((item.id, meta));
            }
        } else if item.id == 155 || item.id == 168 {
            for meta in 0..=2 {
                items.push((item.id, meta));
            }
        } else if item.id == 175 {
            for meta in 0..=5 {
                items.push((item.id, meta));
            }
        } else if item.id == 263 {
            items.push((item.id, 0));
            items.push((item.id, 1));
        } else if item.id == 322 {
            items.push((item.id, 0));
            items.push((item.id, 1));
        } else if item.id == 349 {
            for meta in 0..=3 {
                items.push((item.id, meta));
            }
        } else if item.id == 350 {
            items.push((item.id, 0));
            items.push((item.id, 1));
        } else if item.id == 351 {
            for meta in 0..16 {
                items.push((item.id, meta));
            }
        } else if item.id == 373 {
            let potions = [
                0, 16, 8193, 8194, 8195, 8196, 8197, 8198, 8200, 8201, 8202, 8204, 8205, 8206,
                16385, 16386, 16388, 16389, 16393, 16396,
            ];
            for meta in potions {
                items.push((item.id, meta));
            }
        } else if item.id == 383 {
            let eggs = [
                50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 65, 66, 67, 68, 90, 91, 92, 93,
                94, 95, 96, 98, 100, 101, 120,
            ];
            for meta in eggs {
                items.push((item.id, meta));
            }
        } else if item.id == 397 {
            for meta in 0..=4 {
                items.push((item.id, meta));
            }
        } else {
            items.push((item.id, 0));
        }
    }
    items
}

/// Filters items across all categories by name search.
pub fn search_items(query: &str) -> Vec<(i16, i16)> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return get_creative_tab_items(CreativeTab::BuildingBlocks);
    }

    let mut results = Vec::new();
    for item in ITEMS {
        let name_match = item.name.to_lowercase().contains(&q)
            || item.display_name.to_lowercase().contains(&q);

        if name_match {
            results.push((item.id, 0));
        } else if item.has_subtypes {
            for meta in 0..16 {
                if let Some(sub) = get_subtype_name(item.id, meta) {
                    if sub.to_lowercase().contains(&q) {
                        results.push((item.id, meta));
                    }
                }
            }
        }
    }
    results
}
