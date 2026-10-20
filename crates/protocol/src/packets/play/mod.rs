pub mod clientbound;
pub mod serverbound;

pub use clientbound::*;
pub use serverbound::*;

use crate::error::ProtocolError;
use bytes::Bytes;

/// High-level enum representing recognized clientbound Play packets.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientboundPlayPacket {
    KeepAlive(ClientboundKeepAlivePacket),
    JoinGame(ClientboundJoinGamePacket),
    Chat(ClientboundChatPacket),
    TimeUpdate(ClientboundTimeUpdatePacket),
    Respawn(ClientboundRespawnPacket),
    PlayerPosLook(ClientboundPlayerPosLookPacket),
    ChunkData(ClientboundChunkDataPacket),
    MultiBlockChange(ClientboundMultiBlockChangePacket),
    BlockChange(ClientboundBlockChangePacket),
    MapChunkBulk(ClientboundMapChunkBulkPacket),
    PlayerListItem(ClientboundPlayerListItemPacket),
    ScoreboardObjective(ClientboundScoreboardObjectivePacket),
    UpdateScore(ClientboundUpdateScorePacket),
    DisplayScoreboard(ClientboundDisplayScoreboardPacket),
    Teams(ClientboundTeamsPacket),
    Disconnect(ClientboundDisconnectPlayPacket),
    SpawnPlayer(ClientboundSpawnPlayerPacket),
    SpawnMob(ClientboundSpawnMobPacket),
    SpawnObject(ClientboundSpawnObjectPacket),
    SpawnPainting(ClientboundSpawnPaintingPacket),
    DestroyEntities(ClientboundDestroyEntitiesPacket),
    Entity(ClientboundEntityPacket),
    EntityRelMove(ClientboundEntityRelMovePacket),
    EntityLook(ClientboundEntityLookPacket),
    EntityLookMove(ClientboundEntityLookMovePacket),
    EntityTeleport(ClientboundEntityTeleportPacket),
    EntityHeadLook(ClientboundEntityHeadLookPacket),
    EntityStatus(ClientboundEntityStatusPacket),
    Animation(ClientboundAnimationPacket),
    EntityVelocity(ClientboundEntityVelocityPacket),
    EntityEquipment(ClientboundEntityEquipmentPacket),
    EntityMetadata(ClientboundEntityMetadataPacket),
    Explosion(ClientboundExplosionPacket),
    PlayerAbilities(ClientboundPlayerAbilitiesPacket),
    ConfirmTransaction(ClientboundConfirmTransactionPacket),
    BlockBreakAnim(ClientboundBlockBreakAnimPacket),
    UpdateHealth(ClientboundUpdateHealthPacket),
    SetExperience(ClientboundSetExperiencePacket),
    HeldItemChange(ClientboundHeldItemChangePacket),
    SetSlot(ClientboundSetSlotPacket),
    WindowItems(ClientboundWindowItemsPacket),
    OpenWindow(ClientboundOpenWindowPacket),
    CloseWindow(ClientboundCloseWindowPacket),
    WindowProperty(ClientboundWindowPropertyPacket),
    SignEditorOpen(ClientboundSignEditorOpenPacket),
    EntityEffect(ClientboundEntityEffectPacket),
    RemoveEntityEffect(ClientboundRemoveEntityEffectPacket),
    ChangeGameState(ClientboundChangeGameStatePacket),
    CustomPayload(ClientboundCustomPayloadPacket),
    AttachEntity(ClientboundAttachEntityPacket),
    Maps(ClientboundMapsPacket),
    Unknown { id: i32, payload_len: usize },
}

impl ClientboundPlayPacket {
    /// Attempts to parse a raw packet by its packet ID.
    pub fn decode(id: i32, payload: &mut Bytes) -> Result<Self, ProtocolError> {
        match id {
            ClientboundKeepAlivePacket::PACKET_ID => {
                Ok(Self::KeepAlive(ClientboundKeepAlivePacket::decode(payload)?))
            }
            ClientboundJoinGamePacket::PACKET_ID => {
                Ok(Self::JoinGame(ClientboundJoinGamePacket::decode(payload)?))
            }
            ClientboundChatPacket::PACKET_ID => {
                Ok(Self::Chat(ClientboundChatPacket::decode(payload)?))
            }
            ClientboundTimeUpdatePacket::PACKET_ID => {
                Ok(Self::TimeUpdate(ClientboundTimeUpdatePacket::decode(payload)?))
            }
            ClientboundRespawnPacket::PACKET_ID => {
                Ok(Self::Respawn(ClientboundRespawnPacket::decode(payload)?))
            }
            ClientboundPlayerPosLookPacket::PACKET_ID => {
                Ok(Self::PlayerPosLook(ClientboundPlayerPosLookPacket::decode(payload)?))
            }
            ClientboundChunkDataPacket::PACKET_ID => {
                Ok(Self::ChunkData(ClientboundChunkDataPacket::decode(payload)?))
            }
            ClientboundMultiBlockChangePacket::PACKET_ID => {
                Ok(Self::MultiBlockChange(ClientboundMultiBlockChangePacket::decode(payload)?))
            }
            ClientboundBlockChangePacket::PACKET_ID => {
                Ok(Self::BlockChange(ClientboundBlockChangePacket::decode(payload)?))
            }
            ClientboundMapChunkBulkPacket::PACKET_ID => {
                Ok(Self::MapChunkBulk(ClientboundMapChunkBulkPacket::decode(payload)?))
            }
            ClientboundPlayerListItemPacket::PACKET_ID => {
                Ok(Self::PlayerListItem(ClientboundPlayerListItemPacket::decode(payload)?))
            }
            ClientboundScoreboardObjectivePacket::PACKET_ID => {
                Ok(Self::ScoreboardObjective(ClientboundScoreboardObjectivePacket::decode(payload)?))
            }
            ClientboundUpdateScorePacket::PACKET_ID => {
                Ok(Self::UpdateScore(ClientboundUpdateScorePacket::decode(payload)?))
            }
            ClientboundDisplayScoreboardPacket::PACKET_ID => {
                Ok(Self::DisplayScoreboard(ClientboundDisplayScoreboardPacket::decode(payload)?))
            }
            ClientboundTeamsPacket::PACKET_ID => {
                Ok(Self::Teams(ClientboundTeamsPacket::decode(payload)?))
            }
            ClientboundDisconnectPlayPacket::PACKET_ID => {
                Ok(Self::Disconnect(ClientboundDisconnectPlayPacket::decode(payload)?))
            }
            ClientboundSpawnPlayerPacket::PACKET_ID => {
                Ok(Self::SpawnPlayer(ClientboundSpawnPlayerPacket::decode(payload)?))
            }
            ClientboundSpawnMobPacket::PACKET_ID => {
                Ok(Self::SpawnMob(ClientboundSpawnMobPacket::decode(payload)?))
            }
            ClientboundSpawnObjectPacket::PACKET_ID => {
                Ok(Self::SpawnObject(ClientboundSpawnObjectPacket::decode(payload)?))
            }
            ClientboundSpawnPaintingPacket::PACKET_ID => {
                Ok(Self::SpawnPainting(ClientboundSpawnPaintingPacket::decode(payload)?))
            }
            ClientboundDestroyEntitiesPacket::PACKET_ID => {
                Ok(Self::DestroyEntities(ClientboundDestroyEntitiesPacket::decode(payload)?))
            }
            ClientboundEntityPacket::PACKET_ID => {
                Ok(Self::Entity(ClientboundEntityPacket::decode(payload)?))
            }
            ClientboundEntityRelMovePacket::PACKET_ID => {
                Ok(Self::EntityRelMove(ClientboundEntityRelMovePacket::decode(payload)?))
            }
            ClientboundEntityLookPacket::PACKET_ID => {
                Ok(Self::EntityLook(ClientboundEntityLookPacket::decode(payload)?))
            }
            ClientboundEntityLookMovePacket::PACKET_ID => {
                Ok(Self::EntityLookMove(ClientboundEntityLookMovePacket::decode(payload)?))
            }
            ClientboundEntityTeleportPacket::PACKET_ID => {
                Ok(Self::EntityTeleport(ClientboundEntityTeleportPacket::decode(payload)?))
            }
            ClientboundEntityHeadLookPacket::PACKET_ID => {
                Ok(Self::EntityHeadLook(ClientboundEntityHeadLookPacket::decode(payload)?))
            }
            ClientboundEntityStatusPacket::PACKET_ID => {
                Ok(Self::EntityStatus(ClientboundEntityStatusPacket::decode(payload)?))
            }
            ClientboundAnimationPacket::PACKET_ID => {
                Ok(Self::Animation(ClientboundAnimationPacket::decode(payload)?))
            }
            ClientboundEntityVelocityPacket::PACKET_ID => {
                Ok(Self::EntityVelocity(ClientboundEntityVelocityPacket::decode(payload)?))
            }
            ClientboundEntityEquipmentPacket::PACKET_ID => {
                Ok(Self::EntityEquipment(ClientboundEntityEquipmentPacket::decode(payload)?))
            }
            ClientboundEntityMetadataPacket::PACKET_ID => {
                Ok(Self::EntityMetadata(ClientboundEntityMetadataPacket::decode(payload)?))
            }
            ClientboundExplosionPacket::PACKET_ID => {
                Ok(Self::Explosion(ClientboundExplosionPacket::decode(payload)?))
            }
            ClientboundPlayerAbilitiesPacket::PACKET_ID => {
                Ok(Self::PlayerAbilities(ClientboundPlayerAbilitiesPacket::decode(payload)?))
            }
            ClientboundConfirmTransactionPacket::PACKET_ID => {
                Ok(Self::ConfirmTransaction(ClientboundConfirmTransactionPacket::decode(payload)?))
            }
            ClientboundBlockBreakAnimPacket::PACKET_ID => {
                Ok(Self::BlockBreakAnim(ClientboundBlockBreakAnimPacket::decode(payload)?))
            }
            ClientboundUpdateHealthPacket::PACKET_ID => {
                Ok(Self::UpdateHealth(ClientboundUpdateHealthPacket::decode(payload)?))
            }
            ClientboundSetExperiencePacket::PACKET_ID => {
                Ok(Self::SetExperience(ClientboundSetExperiencePacket::decode(payload)?))
            }
            ClientboundHeldItemChangePacket::PACKET_ID => {
                Ok(Self::HeldItemChange(ClientboundHeldItemChangePacket::decode(payload)?))
            }
            ClientboundSetSlotPacket::PACKET_ID => {
                Ok(Self::SetSlot(ClientboundSetSlotPacket::decode(payload)?))
            }
            ClientboundWindowItemsPacket::PACKET_ID => {
                Ok(Self::WindowItems(ClientboundWindowItemsPacket::decode(payload)?))
            }
            ClientboundOpenWindowPacket::PACKET_ID => {
                Ok(Self::OpenWindow(ClientboundOpenWindowPacket::decode(payload)?))
            }
            ClientboundCloseWindowPacket::PACKET_ID => {
                Ok(Self::CloseWindow(ClientboundCloseWindowPacket::decode(payload)?))
            }
            ClientboundWindowPropertyPacket::PACKET_ID => {
                Ok(Self::WindowProperty(ClientboundWindowPropertyPacket::decode(payload)?))
            }
            ClientboundSignEditorOpenPacket::PACKET_ID => {
                Ok(Self::SignEditorOpen(ClientboundSignEditorOpenPacket::decode(payload)?))
            }
            ClientboundEntityEffectPacket::PACKET_ID => {
                Ok(Self::EntityEffect(ClientboundEntityEffectPacket::decode(payload)?))
            }
            ClientboundRemoveEntityEffectPacket::PACKET_ID => {
                Ok(Self::RemoveEntityEffect(ClientboundRemoveEntityEffectPacket::decode(payload)?))
            }
            ClientboundChangeGameStatePacket::PACKET_ID => {
                Ok(Self::ChangeGameState(ClientboundChangeGameStatePacket::decode(payload)?))
            }
            ClientboundCustomPayloadPacket::PACKET_ID => {
                Ok(Self::CustomPayload(ClientboundCustomPayloadPacket::decode(payload)?))
            }
            ClientboundAttachEntityPacket::PACKET_ID => {
                Ok(Self::AttachEntity(ClientboundAttachEntityPacket::decode(payload)?))
            }
            ClientboundMapsPacket::PACKET_ID => {
                Ok(Self::Maps(ClientboundMapsPacket::decode(payload)?))
            }
            other => Ok(Self::Unknown {
                id: other,
                payload_len: payload.len(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_keep_alive_roundtrip() {
        let pkt = ClientboundKeepAlivePacket { keep_alive_id: 123456 };
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ClientboundKeepAlivePacket::decode(&mut bytes).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_join_game_roundtrip() {
        let pkt = ClientboundJoinGamePacket {
            entity_id: 42,
            game_mode: 0,
            dimension: 0,
            difficulty: 2,
            max_players: 100,
            level_type: "default".into(),
            reduced_debug_info: false,
        };
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ClientboundJoinGamePacket::decode(&mut bytes).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_chat_packet_roundtrip() {
        let pkt = ClientboundChatPacket {
            message: "{\"text\":\"Hello Server!\"}".into(),
            position: 0,
        };
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ClientboundChatPacket::decode(&mut bytes).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_player_pos_look_roundtrip() {
        let pkt = ClientboundPlayerPosLookPacket {
            x: 100.5,
            y: 64.0,
            z: -250.75,
            yaw: 90.0,
            pitch: -15.5,
            flags: 0,
        };
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ClientboundPlayerPosLookPacket::decode(&mut bytes).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_scoreboard_packets_roundtrip() {
        let obj = ClientboundScoreboardObjectivePacket {
            name: "test_obj".into(),
            action: 0,
            display_text: Some("§6Rankings".into()),
            objective_type: Some("integer".into()),
        };
        let mut buf = BytesMut::new();
        obj.encode(&mut buf).unwrap();

        let mut bytes = buf.freeze();
        let decoded = ClientboundScoreboardObjectivePacket::decode(&mut bytes).unwrap();
        assert_eq!(obj, decoded);

        let score = ClientboundUpdateScorePacket {
            score_holder: "Alex".into(),
            action: 0,
            objective_name: "test_obj".into(),
            value: Some(1500),
        };
        let mut buf2 = BytesMut::new();
        score.encode(&mut buf2).unwrap();

        let mut bytes2 = buf2.freeze();
        let decoded2 = ClientboundUpdateScorePacket::decode(&mut bytes2).unwrap();
        assert_eq!(score, decoded2);
    }

    #[test]
    fn test_entity_packets_roundtrip() {
        use uuid::Uuid;

        // 1. Spawn player
        let spawn_player = ClientboundSpawnPlayerPacket {
            entity_id: 101,
            player_id: Uuid::from_u128(0x1234567890abcdef1234567890abcdef),
            x: 3200,
            y: 2048,
            z: -1600,
            yaw: 45,
            pitch: -10,
            current_item: 276, // Diamond sword
            metadata: vec![],
        };
        let mut buf = BytesMut::new();
        spawn_player.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundSpawnPlayerPacket::decode(&mut bytes).unwrap();
        assert_eq!(spawn_player, decoded);

        // 2. Spawn mob
        let spawn_mob = ClientboundSpawnMobPacket {
            entity_id: 102,
            mob_type: 54, // Zombie
            x: 100,
            y: 200,
            z: 300,
            yaw: 10,
            pitch: 20,
            head_pitch: 15,
            velocity_x: 50,
            velocity_y: 60,
            velocity_z: 70,
            metadata: vec![],
        };
        let mut buf = BytesMut::new();
        spawn_mob.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundSpawnMobPacket::decode(&mut bytes).unwrap();
        assert_eq!(spawn_mob, decoded);

        // 3. Spawn object
        let spawn_obj = ClientboundSpawnObjectPacket {
            entity_id: 103,
            object_type: 60, // Arrow
            x: 10,
            y: 20,
            z: 30,
            pitch: 5,
            yaw: -5,
            data: 1,
            velocity_x: 100,
            velocity_y: 200,
            velocity_z: 300,
        };
        let mut buf = BytesMut::new();
        spawn_obj.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundSpawnObjectPacket::decode(&mut bytes).unwrap();
        assert_eq!(spawn_obj, decoded);

        // 4. Destroy entities
        let destroy = ClientboundDestroyEntitiesPacket {
            entity_ids: vec![101, 102, 103],
        };
        let mut buf = BytesMut::new();
        destroy.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundDestroyEntitiesPacket::decode(&mut bytes).unwrap();
        assert_eq!(destroy, decoded);

        // 5. Rel move and look
        let rel_move = ClientboundEntityRelMovePacket {
            entity_id: 101,
            delta_x: 4,
            delta_y: -2,
            delta_z: 8,
            on_ground: true,
        };
        let mut buf = BytesMut::new();
        rel_move.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundEntityRelMovePacket::decode(&mut bytes).unwrap();
        assert_eq!(rel_move, decoded);

        let look_move = ClientboundEntityLookMovePacket {
            entity_id: 101,
            delta_x: 2,
            delta_y: 3,
            delta_z: -1,
            yaw: 64,
            pitch: -32,
            on_ground: false,
        };
        let mut buf = BytesMut::new();
        look_move.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundEntityLookMovePacket::decode(&mut bytes).unwrap();
        assert_eq!(look_move, decoded);

        // 6. Teleport
        let tp = ClientboundEntityTeleportPacket {
            entity_id: 101,
            x: 6400,
            y: 3200,
            z: -3200,
            yaw: 120,
            pitch: 0,
            on_ground: true,
        };
        let mut buf = BytesMut::new();
        tp.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundEntityTeleportPacket::decode(&mut bytes).unwrap();
        assert_eq!(tp, decoded);
    }

    #[test]
    fn test_teams_packet_roundtrip() {
        let pkt = ClientboundTeamsPacket {
            team_name: "red_team".into(),
            action: 0,
            display_name: Some("Red Team".into()),
            prefix: Some("§c[RED] ".into()),
            suffix: Some("§r".into()),
            friendly_flags: Some(1),
            name_tag_visibility: Some("always".into()),
            color: Some(12),
            players: vec!["Alice".into(), "Bob".into()],
        };
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundTeamsPacket::decode(&mut bytes).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_confirm_transaction_roundtrip() {
        let client_pkt = ClientboundConfirmTransactionPacket {
            window_id: 0,
            action_number: -42,
            accepted: false,
        };
        let mut buf = BytesMut::new();
        client_pkt.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundConfirmTransactionPacket::decode(&mut bytes).unwrap();
        assert_eq!(client_pkt, decoded);

        let server_pkt = ServerboundConfirmTransactionPacket {
            window_id: 0,
            action_number: -42,
            accepted: true,
        };
        let mut buf2 = BytesMut::new();
        server_pkt.encode(&mut buf2).unwrap();
        let mut bytes2 = buf2.freeze();
        let decoded2 = ServerboundConfirmTransactionPacket::decode(&mut bytes2).unwrap();
        assert_eq!(server_pkt, decoded2);
    }

    #[test]
    fn test_stage11_packets_roundtrip() {
        use crate::codec::position::Position;
        use crate::codec::slot::Slot;

        // 1. Digging
        let dig = ServerboundPlayerDiggingPacket {
            status: DiggingAction::StartDestroyBlock,
            position: Position::new(12, 64, -34),
            facing: BlockFace::Up,
        };
        let mut buf = BytesMut::new();
        dig.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded_dig = ServerboundPlayerDiggingPacket::decode(&mut bytes).unwrap();
        assert_eq!(dig, decoded_dig);

        // 2. Block placement
        let place = ServerboundPlayerBlockPlacementPacket {
            position: Position::new(10, 63, 10),
            placed_block_direction: 1,
            stack: Some(Slot::new(1, 64, 0, None)),
            facing_x: 8,
            facing_y: 16,
            facing_z: 8,
        };
        let mut buf2 = BytesMut::new();
        place.encode(&mut buf2).unwrap();
        let mut bytes2 = buf2.freeze();
        let decoded_place = ServerboundPlayerBlockPlacementPacket::decode(&mut bytes2).unwrap();
        assert_eq!(place, decoded_place);

        // 3. Use entity (Attack)
        let attack = ServerboundUseEntityPacket::attack(42);
        let mut buf3 = BytesMut::new();
        attack.encode(&mut buf3).unwrap();
        let mut bytes3 = buf3.freeze();
        let decoded_attack = ServerboundUseEntityPacket::decode(&mut bytes3).unwrap();
        assert_eq!(attack, decoded_attack);

        // 4. Use entity (InteractAt)
        let interact_at = ServerboundUseEntityPacket::new(42, UseEntityAction::InteractAt, Some((0.5, 1.2, 0.5)));
        let mut buf4 = BytesMut::new();
        interact_at.encode(&mut buf4).unwrap();
        let mut bytes4 = buf4.freeze();
        let decoded_interact = ServerboundUseEntityPacket::decode(&mut bytes4).unwrap();
        assert_eq!(interact_at, decoded_interact);

        // 5. Animation (Arm swing)
        let anim = ServerboundAnimationPacket;
        let mut buf5 = BytesMut::new();
        anim.encode(&mut buf5).unwrap();
        let mut bytes5 = buf5.freeze();
        let decoded_anim = ServerboundAnimationPacket::decode(&mut bytes5).unwrap();
        assert_eq!(anim, decoded_anim);

        // 6. Block break animation
        let break_anim = ClientboundBlockBreakAnimPacket {
            entity_id: 101,
            position: Position::new(5, 70, -8),
            progress: 4,
        };
        let mut buf6 = BytesMut::new();
        break_anim.encode(&mut buf6).unwrap();
        let mut bytes6 = buf6.freeze();
        let decoded_break = ClientboundBlockBreakAnimPacket::decode(&mut bytes6).unwrap();
        assert_eq!(break_anim, decoded_break);
    }

    #[test]
    fn test_hud_packets_roundtrip() {
        use crate::codec::slot::Slot;

        // 1. Update Health
        let hp = ClientboundUpdateHealthPacket {
            health: 14.5,
            food: 18,
            food_saturation: 5.0,
        };
        let mut buf = BytesMut::new();
        hp.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded_hp = ClientboundUpdateHealthPacket::decode(&mut bytes).unwrap();
        assert_eq!(hp, decoded_hp);

        // 2. Set Experience
        let exp = ClientboundSetExperiencePacket {
            experience_bar: 0.65,
            level: 30,
            total_experience: 1395,
        };
        let mut buf2 = BytesMut::new();
        exp.encode(&mut buf2).unwrap();
        let mut bytes2 = buf2.freeze();
        let decoded_exp = ClientboundSetExperiencePacket::decode(&mut bytes2).unwrap();
        assert_eq!(exp, decoded_exp);

        // 3. Held Item Change
        let held = ClientboundHeldItemChangePacket { slot: 3 };
        let mut buf3 = BytesMut::new();
        held.encode(&mut buf3).unwrap();
        let mut bytes3 = buf3.freeze();
        let decoded_held = ClientboundHeldItemChangePacket::decode(&mut bytes3).unwrap();
        assert_eq!(held, decoded_held);

        // 4. Set Slot
        let set_slot = ClientboundSetSlotPacket {
            window_id: 0,
            slot: 36,
            item: Some(Slot::new(276, 1, 15, None)),
        };
        let mut buf4 = BytesMut::new();
        set_slot.encode(&mut buf4).unwrap();
        let mut bytes4 = buf4.freeze();
        let decoded_set_slot = ClientboundSetSlotPacket::decode(&mut bytes4).unwrap();
        assert_eq!(set_slot, decoded_set_slot);

        // 5. Window Items
        let win_items = ClientboundWindowItemsPacket {
            window_id: 0,
            items: vec![
                Some(Slot::new(310, 1, 0, None)),
                None,
                Some(Slot::new(276, 1, 5, None)),
            ],
        };
        let mut buf5 = BytesMut::new();
        win_items.encode(&mut buf5).unwrap();
        let mut bytes5 = buf5.freeze();
        let decoded_win_items = ClientboundWindowItemsPacket::decode(&mut bytes5).unwrap();
        assert_eq!(win_items, decoded_win_items);

        // 6. Entity Effect
        let effect = ClientboundEntityEffectPacket {
            entity_id: 1,
            effect_id: 1, // Speed
            amplifier: 1, // Speed II
            duration: 1200, // 60s
            hide_particles: false,
        };
        let mut buf6 = BytesMut::new();
        effect.encode(&mut buf6).unwrap();
        let mut bytes6 = buf6.freeze();
        let decoded_effect = ClientboundEntityEffectPacket::decode(&mut bytes6).unwrap();
        assert_eq!(effect, decoded_effect);

        // 7. Remove Entity Effect
        let rem_effect = ClientboundRemoveEntityEffectPacket {
            entity_id: 1,
            effect_id: 1,
        };
        let mut buf7 = BytesMut::new();
        rem_effect.encode(&mut buf7).unwrap();
        let mut bytes7 = buf7.freeze();
        let decoded_rem_effect = ClientboundRemoveEntityEffectPacket::decode(&mut bytes7).unwrap();
        assert_eq!(rem_effect, decoded_rem_effect);
    }

    #[test]
    fn test_clientbound_custom_payload_roundtrip() {
        let pkt = ClientboundCustomPayloadPacket::new("MC|Brand", b"BungeeCord".to_vec());
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundCustomPayloadPacket::decode(&mut bytes).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_window_packets_roundtrip() {
        use crate::codec::position::Position;
        use crate::codec::slot::Slot;

        // Open Window
        let open_win = ClientboundOpenWindowPacket {
            window_id: 1,
            inventory_type: "minecraft:chest".into(),
            window_title: "{\"text\":\"Chest\"}".into(),
            slot_count: 27,
            entity_id: None,
        };
        let mut buf = BytesMut::new();
        open_win.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded_open = ClientboundOpenWindowPacket::decode(&mut bytes).unwrap();
        assert_eq!(open_win, decoded_open);

        // Close Window (Clientbound & Serverbound)
        let close_cb = ClientboundCloseWindowPacket { window_id: 1 };
        let mut buf = BytesMut::new();
        close_cb.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded_close_cb = ClientboundCloseWindowPacket::decode(&mut bytes).unwrap();
        assert_eq!(close_cb, decoded_close_cb);

        let close_sb = ServerboundCloseWindowPacket::new(1);
        let mut buf = BytesMut::new();
        close_sb.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded_close_sb = ServerboundCloseWindowPacket::decode(&mut bytes).unwrap();
        assert_eq!(close_sb, decoded_close_sb);

        // Window Property
        let win_prop = ClientboundWindowPropertyPacket {
            window_id: 1,
            property: 0,
            value: 150,
        };
        let mut buf = BytesMut::new();
        win_prop.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded_prop = ClientboundWindowPropertyPacket::decode(&mut bytes).unwrap();
        assert_eq!(win_prop, decoded_prop);

        // Click Window
        let click_win = ServerboundClickWindowPacket::new(
            0,
            36,
            0,
            1,
            0,
            Some(Slot::new(276, 1, 0, None)),
        );
        let mut buf = BytesMut::new();
        click_win.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded_click = ServerboundClickWindowPacket::decode(&mut bytes).unwrap();
        assert_eq!(click_win, decoded_click);

        // Creative Inventory Action
        let creative_act = ServerboundCreativeInventoryActionPacket::new(
            36,
            Some(Slot::new(1, 64, 0, None)),
        );
        let mut buf = BytesMut::new();
        creative_act.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded_creative = ServerboundCreativeInventoryActionPacket::decode(&mut bytes).unwrap();
        assert_eq!(creative_act, decoded_creative);

        // Enchant Item
        let enchant = ServerboundEnchantItemPacket::new(1, 2);
        let mut buf = BytesMut::new();
        enchant.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded_enchant = ServerboundEnchantItemPacket::decode(&mut bytes).unwrap();
        assert_eq!(enchant, decoded_enchant);

        // Update Sign
        let sign = ServerboundUpdateSignPacket::new(
            Position::new(10, 64, 20),
            ["Line 1".into(), "Line 2".into(), "".into(), "Done".into()],
        );
        let mut buf = BytesMut::new();
        sign.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded_sign = ServerboundUpdateSignPacket::decode(&mut bytes).unwrap();
        assert_eq!(sign.position, decoded_sign.position);
        assert_eq!(sign.lines, decoded_sign.lines);
    }

    #[test]
    fn test_maps_packet_roundtrip() {
        let pkt = ClientboundMapsPacket {
            map_id: 17,
            scale: 2,
            icons: vec![
                MapIcon {
                    icon_type: 1,
                    direction: 8,
                    x: -32,
                    z: 64,
                },
            ],
            columns: 2,
            rows: 2,
            x: 10,
            z: 20,
            data: vec![12, 14, 16, 18],
        };
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();
        let mut bytes = buf.freeze();
        let decoded = ClientboundMapsPacket::decode(&mut bytes).unwrap();
        assert_eq!(pkt, decoded);
    }
}


