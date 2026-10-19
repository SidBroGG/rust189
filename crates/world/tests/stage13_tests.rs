use protocol::{
    ClientboundCloseWindowPacket, ClientboundOpenWindowPacket, ClientboundWindowPropertyPacket,
    Position, ServerboundClickWindowPacket, ServerboundCloseWindowPacket,
    ServerboundCreativeInventoryActionPacket, ServerboundEnchantItemPacket,
    ServerboundUpdateSignPacket, Slot,
};
use world::{
    get_creative_tab_items, get_item_max_damage, get_item_max_stack, search_items, ContainerState,
    CreativeTab,
};

#[test]
fn test_inventory_slot_click_modes() {
    let mut inv = ContainerState::new_player_inventory();

    // 1. Setup items in player inventory (slots 9..45)
    // Slot 36 = 16 Cobblestone (id 4)
    inv.set_slot(36, Some(Slot::new(4, 16, 0, None)));
    // Slot 37 = 32 Cobblestone (id 4)
    inv.set_slot(37, Some(Slot::new(4, 32, 0, None)));

    // Mode 0: Normal LMB click on slot 36 picks up all 16
    let dropped = inv.handle_normal_click(36, false);
    assert!(dropped.is_none());
    assert!(inv.slots[36].is_none());
    assert_eq!(inv.carried_item.as_ref().map(|s| s.item_count), Some(16));

    // Mode 0: Normal RMB click on empty slot 36 places 1 item
    inv.handle_normal_click(36, true);
    assert_eq!(inv.slots[36].as_ref().map(|s| s.item_count), Some(1));
    assert_eq!(inv.carried_item.as_ref().map(|s| s.item_count), Some(15));

    // Mode 0: Click on slot 37 with same item merges stack
    inv.handle_normal_click(37, false);
    assert_eq!(inv.slots[37].as_ref().map(|s| s.item_count), Some(47));
    assert!(inv.carried_item.is_none());

    // Mode 0: Right click on slot 37 splits stack in half (47 -> 23 on slot, 24 carried)
    inv.handle_normal_click(37, true);
    assert_eq!(inv.slots[37].as_ref().map(|s| s.item_count), Some(23));
    assert_eq!(inv.carried_item.as_ref().map(|s| s.item_count), Some(24));

    // Mode 6: Double click gather
    // Place remaining 24 on slot 10
    inv.handle_normal_click(10, false);
    assert!(inv.carried_item.is_none());
    assert_eq!(inv.slots[10].as_ref().map(|s| s.item_count), Some(24));

    // Pick up 1 item from slot 36
    inv.handle_normal_click(36, false);
    assert_eq!(inv.carried_item.as_ref().map(|s| s.item_count), Some(1));

    // Double click gather on slot 37 collects 23 from slot 37 and 24 from slot 10 -> 1 + 23 + 24 = 48
    inv.handle_double_click(37);
    assert_eq!(inv.carried_item.as_ref().map(|s| s.item_count), Some(48));

    // Mode 2: Hotbar key swap
    // Put diamond sword (id 276) into main inventory slot 9
    inv.set_slot(9, Some(Slot::new(276, 1, 0, None)));
    // Hotbar key 1 (button 0 = slot 36)
    inv.handle_hotbar_swap(9, 0);
    assert_eq!(inv.slots[36].as_ref().map(|s| s.item_id), Some(276));
    assert!(inv.slots[9].is_none());

    // Mode 4: Drop item (Q drops 1, button 1 drops full stack)
    inv.set_slot(38, Some(Slot::new(264, 10, 0, None))); // 10 diamonds
    let drop_one = inv.handle_drop(38, false);
    assert_eq!(drop_one.as_ref().map(|s| s.item_count), Some(1));
    assert_eq!(inv.slots[38].as_ref().map(|s| s.item_count), Some(9));

    let drop_stack = inv.handle_drop(38, true);
    assert_eq!(drop_stack.as_ref().map(|s| s.item_count), Some(9));
    assert!(inv.slots[38].is_none());
}

#[test]
fn test_shift_click_container_transfer() {
    let mut chest = ContainerState::from_open_window(1, "minecraft:chest", "Chest".to_string(), 27);
    // Chest slots: 0..27 chest, 27..63 player (27 main + 9 hotbar)
    // Place iron ingot in chest slot 0
    chest.set_slot(0, Some(Slot::new(265, 32, 0, None)));

    // Shift-click chest slot 0 moves items to player slots
    chest.handle_shift_click(0);
    assert!(chest.slots[0].is_none());
    assert!(chest.slots[27..63].iter().any(|s| s.as_ref().map_or(false, |i| i.item_id == 265 && i.item_count == 32)));

    // Shift-click player slot moves items back to chest
    let player_slot_idx = chest.slots[27..63].iter().position(|s| s.is_some()).unwrap() + 27;
    chest.handle_shift_click(player_slot_idx as i16);
    assert!(chest.slots[0..27].iter().any(|s| s.as_ref().map_or(false, |i| i.item_id == 265 && i.item_count == 32)));
}

#[test]
fn test_crafting_2x2_and_3x3_recipes() {
    // 2x2 Crafting in Player Inventory
    let mut player_inv = ContainerState::new_player_inventory();
    // Oak log (id 17) into slot 1 (first craft slot)
    player_inv.set_slot(1, Some(Slot::new(17, 1, 0, None)));
    // Result slot 0 should produce 4 Oak Planks (id 5)
    assert_eq!(player_inv.slots[0].as_ref().map(|s| (s.item_id, s.item_count)), Some((5, 4)));

    // Two planks vertically -> 4 Sticks (id 280)
    player_inv.set_slot(1, Some(Slot::new(5, 1, 0, None)));
    player_inv.set_slot(3, Some(Slot::new(5, 1, 0, None)));
    assert_eq!(player_inv.slots[0].as_ref().map(|s| (s.item_id, s.item_count)), Some((280, 4)));

    // 3x3 Crafting in Workbench
    let mut bench = ContainerState::from_open_window(2, "minecraft:crafting_table", "Crafting".to_string(), 0);
    // Workbench slots: 0 result, 1..=9 3x3 grid, 10..46 player slots
    // Pickaxe recipe: top row = 3 Iron (id 265), middle vertical = 2 Sticks (id 280)
    bench.set_slot(1, Some(Slot::new(265, 1, 0, None)));
    bench.set_slot(2, Some(Slot::new(265, 1, 0, None)));
    bench.set_slot(3, Some(Slot::new(265, 1, 0, None)));
    bench.set_slot(5, Some(Slot::new(280, 1, 0, None)));
    bench.set_slot(8, Some(Slot::new(280, 1, 0, None)));

    // Result should be Iron Pickaxe (id 257)
    assert_eq!(bench.slots[0].as_ref().map(|s| (s.item_id, s.item_count)), Some((257, 1)));
}

#[test]
fn test_creative_tabs_and_search() {
    // Check creative tabs have items
    let building = get_creative_tab_items(CreativeTab::BuildingBlocks);
    assert!(!building.is_empty());
    assert!(building.iter().any(|s| s.0 == 1)); // Stone

    let combat = get_creative_tab_items(CreativeTab::Combat);
    assert!(!combat.is_empty());
    assert!(combat.iter().any(|s| s.0 == 276)); // Diamond Sword

    let tools = get_creative_tab_items(CreativeTab::Tools);
    assert!(!tools.is_empty());
    assert!(tools.iter().any(|s| s.0 == 278)); // Diamond Pickaxe

    // Search tests
    let search_diamond = search_items("алмаз");
    assert!(!search_diamond.is_empty());
    assert!(search_diamond.iter().any(|s| s.0 == 264)); // Diamond item

    let search_sword = search_items("меч");
    assert!(!search_sword.is_empty());

    // Stack sizes
    assert_eq!(get_item_max_stack(276), 1); // Diamond Sword max stack 1
    assert_eq!(get_item_max_stack(332), 16); // Snowball max stack 16
    assert_eq!(get_item_max_stack(1), 64); // Stone max stack 64

    // Max damage
    assert_eq!(get_item_max_damage(276), 1561); // Diamond sword durability
    assert_eq!(get_item_max_damage(1), 0); // Stone is not damageable
}

#[test]
fn test_container_packets_roundtrip() {
    // OpenWindow
    let open = ClientboundOpenWindowPacket {
        window_id: 3,
        inventory_type: "minecraft:furnace".into(),
        window_title: "Furnace".into(),
        slot_count: 3,
        entity_id: None,
    };
    assert_eq!(open.window_id, 3);
    assert_eq!(open.inventory_type, "minecraft:furnace");

    // CloseWindow clientbound & serverbound
    let close_cb = ClientboundCloseWindowPacket { window_id: 3 };
    assert_eq!(close_cb.window_id, 3);
    let close_sb = ServerboundCloseWindowPacket::new(3);
    assert_eq!(close_sb.window_id, 3);

    // WindowProperty
    let prop = ClientboundWindowPropertyPacket {
        window_id: 3,
        property: 0,
        value: 150,
    };
    assert_eq!(prop.property, 0);
    assert_eq!(prop.value, 150);

    // ClickWindow
    let click = ServerboundClickWindowPacket::new(
        3,
        2,
        0,
        14,
        0,
        Some(Slot::new(265, 1, 0, None)),
    );
    assert_eq!(click.window_id, 3);
    assert_eq!(click.slot, 2);
    assert_eq!(click.action_number, 14);

    // CreativeInventoryAction
    let creative_act = ServerboundCreativeInventoryActionPacket::new(36, Some(Slot::new(276, 1, 0, None)));
    assert_eq!(creative_act.slot, 36);

    // EnchantItem
    let enchant = ServerboundEnchantItemPacket::new(4, 2);
    assert_eq!(enchant.window_id, 4);
    assert_eq!(enchant.button, 2);

    // UpdateSign
    let sign = ServerboundUpdateSignPacket::new(
        Position::new(10, 64, -20),
        ["Line 1".into(), "Line 2".into(), "Line 3".into(), "Line 4".into()],
    );
    assert_eq!(sign.position, Position::new(10, 64, -20));
    assert_eq!(sign.lines[0], "Line 1");
}
