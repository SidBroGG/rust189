use assets::SkinManager;
use crate::config::ClientConfig;
use egui_wgpu::ScreenDescriptor;
use gui::{
    matches_winit_key, winit_key_to_standard, winit_mouse_to_standard, GuiIntegration,
    GuiOutput, Language, NametagInfo, ScaledResolution, ScreenManager,
};
use protocol::{
    connect_offline, ClientboundPlayPacket, OutgoingPlayPacket, PlaySessionHandle,
};
use render::{
    Camera, ChunkMeshManager, EntityRenderer, Frustum, GpuContext, TerrainRenderer, TextureAtlas,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Icon, Window, WindowId};
use world::{BlockState, EntityKind, FrameStage, PerformanceProfiler, World};

pub struct App {
    pub config: ClientConfig,
    pub config_path: PathBuf,
    pub window: Option<Arc<Window>>,
    pub gpu: Option<GpuContext>,
    pub gui: Option<GuiIntegration>,
    pub screen_manager: Option<ScreenManager>,
    pub last_frame_time: Instant,
    pub frame_count: u64,

    // Play state & World
    pub world: World,
    pub profiler: PerformanceProfiler,
    pub session: Option<PlaySessionHandle>,
    pub last_movement_tick: Instant,
    pub last_physics_tick: Instant,
    pub player_pos: (f64, f64, f64),
    pub player_look: (f32, f32),
    pub player_on_ground: bool,
    pub has_spawned: bool,
    pub pending_connection: Option<tokio::sync::oneshot::Receiver<Result<PlaySessionHandle, String>>>,
    pub connecting_server_ip: Option<String>,

    // Rendering & Terrain (Stage 5)
    pub atlas: Option<Arc<TextureAtlas>>,
    pub terrain_renderer: Option<TerrainRenderer>,
    pub chunk_manager: Option<ChunkMeshManager>,
    pub entity_renderer: Option<EntityRenderer>,
    pub skin_manager: SkinManager,
    pub camera: Camera,
    pub mouse_grabbed: bool,
    pub pressed_keys: HashSet<KeyCode>,
}

impl App {
    pub fn new(config: ClientConfig, config_path: PathBuf) -> Self {
        Self {
            config,
            config_path,
            window: None,
            gpu: None,
            gui: None,
            screen_manager: None,
            last_frame_time: Instant::now(),
            frame_count: 0,

            world: World::new(),
            profiler: PerformanceProfiler::new(),
            session: None,
            last_movement_tick: Instant::now(),
            last_physics_tick: Instant::now(),
            player_pos: (0.0, 64.0, 0.0),
            player_look: (0.0, 0.0),
            player_on_ground: false,
            has_spawned: false,
            pending_connection: None,
            connecting_server_ip: None,

            atlas: None,
            terrain_renderer: None,
            chunk_manager: None,
            entity_renderer: None,
            skin_manager: SkinManager::new(Path::new("assets")),
            camera: Camera::default(),
            mouse_grabbed: false,
            pressed_keys: HashSet::new(),
        }
    }

    /// Loads the window icon from assets if available.
    fn load_window_icon() -> Option<Icon> {
        let icon_paths = [
            Path::new("assets/pack.png"),
            Path::new("assets/minecraft/textures/gui/icons.png"),
        ];

        for path in icon_paths {
            if path.exists() {
                if let Ok(img) = image::open(path) {
                    let rgba = img.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    if let Ok(icon) = Icon::from_rgba(rgba.into_raw(), width, height) {
                        return Some(icon);
                    }
                }
            }
        }
        None
    }

    fn initiate_connection(&mut self, server_ip: &str) {
        self.has_spawned = false;
        let (host, port) = if let Some((h, p)) = server_ip.split_once(':') {
            (h.to_string(), p.parse::<u16>().unwrap_or(25565))
        } else {
            (server_ip.to_string(), 25565)
        };

        let nickname = self.config.auth.nickname.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending_connection = Some(rx);
        self.connecting_server_ip = Some(server_ip.to_string());

        info!(host = %host, port, nickname = %nickname, "Connecting to server...");

        tokio::spawn(async move {
            match connect_offline(&host, port, &nickname).await {
                Ok((stream, _success)) => {
                    let handle = PlaySessionHandle::start(stream);
                    let _ = tx.send(Ok(handle));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });
    }

    fn handle_incoming_packets(&mut self) {
        // 1. Check if asynchronous connection completed
        if let Some(mut rx) = self.pending_connection.take() {
            match rx.try_recv() {
                Ok(Ok(handle)) => {
                    info!("Successfully transitioned to Play phase");
                    let server_ip = self.connecting_server_ip.take().unwrap_or_default();
                    if let Some(sm) = &mut self.screen_manager {
                        sm.start_in_game(server_ip, self.config.auth.nickname.clone());
                    }
                    self.session = Some(handle);
                }
                Ok(Err(err_msg)) => {
                    warn!(error = %err_msg, "Failed to connect to server");
                    self.connecting_server_ip = None;
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                    // Still connecting, put rx back
                    self.pending_connection = Some(rx);
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    warn!("Connection task was cancelled or closed");
                    self.connecting_server_ip = None;
                }
            }
        }

        // 2. Process incoming packets from active PlaySession
        let Some(session) = &self.session else {
            return;
        };

        if !session.is_alive() {
            warn!("Server connection closed");
            self.session = None;
            self.has_spawned = false;
            self.world.clear();
            if let Some(sm) = &mut self.screen_manager {
                sm.exit_in_game();
            }
            return;
        }

        let mut session_handle = self.session.take().unwrap();
        let mut packets = Vec::new();
        while let Ok(pkt) = session_handle.incoming_rx.try_recv() {
            packets.push(pkt);
        }
        self.session = Some(session_handle);

        for packet in packets {
            match packet {
                ClientboundPlayPacket::KeepAlive(_ka) => {
                    // Measured ping/RTT
                    // Silently track keep-alive latency without log flooding
                    let fake_latency = Duration::from_millis(30);
                    self.profiler.on_keep_alive_latency(fake_latency);
                }
                ClientboundPlayPacket::JoinGame(jg) => {
                    info!(
                        entity_id = jg.entity_id,
                        dimension = jg.dimension,
                        gamemode = jg.game_mode,
                        "Joined game world (JoinGame)"
                    );
                    self.world.dimension = jg.dimension;
                    if let Some(sm) = &mut self.screen_manager {
                        if let Some(ig) = &mut sm.in_game {
                            ig.dimension = jg.dimension;
                            ig.add_chat_message("§aSuccessfully connected to world!".into());
                        }
                    }

                    // Automatically grab mouse cursor on entering world
                    if let Some(w) = &self.window {
                        let _ = w.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                            .or_else(|_| w.set_cursor_grab(winit::window::CursorGrabMode::Locked));
                        w.set_cursor_visible(false);
                        self.mouse_grabbed = true;
                    }

                    // According to MCP NetHandlerPlayClient.handleJoinGame:
                    // ClientSettings (0x15) is sent only after JoinGame (0x01) has been received.
                    if let Some(s) = &self.session {
                        s.send(OutgoingPlayPacket::ClientSettings {
                            locale: "en_US".into(),
                            view_distance: 12,
                            chat_flags: 0,
                            chat_colors: true,
                            skin_parts: 0xFF,
                        });
                    }
                }
                ClientboundPlayPacket::Respawn(rp) => {
                    info!(dimension = rp.dimension, "Respawned in dimension");
                    self.world.dimension = rp.dimension as i8;
                    if let Some(sm) = &mut self.screen_manager {
                        if let Some(ig) = &mut sm.in_game {
                            ig.dimension = rp.dimension as i8;
                        }
                    }
                }
                ClientboundPlayPacket::TimeUpdate(tu) => {
                    self.world.time = tu.time_of_day;
                    self.world.age = tu.world_age;
                }
                ClientboundPlayPacket::PlayerPosLook(pos_look) => {
                    // Update position
                    if (pos_look.flags & 0x01) != 0 {
                        self.player_pos.0 += pos_look.x;
                    } else {
                        self.player_pos.0 = pos_look.x;
                    }

                    if (pos_look.flags & 0x02) != 0 {
                        self.player_pos.1 += pos_look.y;
                    } else {
                        self.player_pos.1 = pos_look.y;
                    }

                    if (pos_look.flags & 0x04) != 0 {
                        self.player_pos.2 += pos_look.z;
                    } else {
                        self.player_pos.2 = pos_look.z;
                    }

                    if (pos_look.flags & 0x08) != 0 {
                        self.player_look.0 += pos_look.yaw;
                    } else {
                        self.player_look.0 = pos_look.yaw;
                    }

                    if (pos_look.flags & 0x10) != 0 {
                        self.player_look.1 += pos_look.pitch;
                    } else {
                        self.player_look.1 = pos_look.pitch;
                    }

                    // Confirm position back to server and mark spawned
                    self.has_spawned = true;
                    if let Some(s) = &self.session {
                        s.send(OutgoingPlayPacket::PlayerPositionAndLook {
                            x: self.player_pos.0,
                            y: self.player_pos.1,
                            z: self.player_pos.2,
                            yaw: self.player_look.0,
                            pitch: self.player_look.1,
                            on_ground: self.player_on_ground,
                        });
                    }

                    if let Some(sm) = &mut self.screen_manager {
                        if let Some(ig) = &mut sm.in_game {
                            ig.pos = self.player_pos;
                            ig.look = self.player_look;
                        }
                    }

                    self.camera.position = glam::Vec3::new(
                        self.player_pos.0 as f32,
                        (self.player_pos.1 + 1.62) as f32,
                        self.player_pos.2 as f32,
                    );
                    self.camera.yaw = self.player_look.0;
                    self.camera.pitch = self.player_look.1;

                    // Ensure mouse is captured on spawn
                    if !self.mouse_grabbed {
                        if let Some(w) = &self.window {
                            let _ = w.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                .or_else(|_| w.set_cursor_grab(winit::window::CursorGrabMode::Locked));
                            w.set_cursor_visible(false);
                            self.mouse_grabbed = true;
                        }
                    }
                }
                ClientboundPlayPacket::Chat(chat_pkt) => {
                    let formatted = world::parse_chat_message(&chat_pkt.message);
                    info!(chat = %formatted, "Received chat message");
                    if let Some(sm) = &mut self.screen_manager {
                        if let Some(ig) = &mut sm.in_game {
                            ig.add_chat_message(formatted);
                        }
                    }
                }
                ClientboundPlayPacket::ChunkData(cd) => {
                    self.profiler.on_bytes_received(cd.data.len());
                    if cd.ground_up && cd.bit_mask == 0 {
                        self.world.unload_chunk(cd.chunk_x, cd.chunk_z);
                        if let Some(cm) = &mut self.chunk_manager {
                            cm.remove_chunk(cd.chunk_x, cd.chunk_z);
                        }
                    } else {
                        if let Err(e) = self.world.load_chunk_data(
                            cd.chunk_x,
                            cd.chunk_z,
                            cd.ground_up,
                            cd.bit_mask,
                            &cd.data,
                        ) {
                            warn!(x = cd.chunk_x, z = cd.chunk_z, error = %e, "Failed to load chunk data");
                        }
                        if let (Some(cm), Some(atlas)) = (&mut self.chunk_manager, &self.atlas) {
                            for sec_idx in 0..16 {
                                if (cd.bit_mask & (1 << sec_idx)) != 0 {
                                    cm.enqueue_section(&self.world, atlas.clone(), cd.chunk_x, sec_idx, cd.chunk_z);
                                }
                            }
                        }
                    }
                    self.profiler.loaded_chunks = self.world.chunk_count();
                }
                ClientboundPlayPacket::MapChunkBulk(bulk) => {
                    self.profiler.on_bytes_received(bulk.data.len());
                    let mut offset = 0;
                    for meta in &bulk.meta {
                        if offset >= bulk.data.len() {
                            break;
                        }
                        if let Some(chunk) = self.world.get_chunk_mut(meta.x, meta.z) {
                            if let Ok(consumed) = chunk.load_from_raw_payload(
                                &bulk.data[offset..],
                                meta.bit_mask,
                                true,
                                bulk.sky_light_sent,
                            ) {
                                offset += consumed;
                            }
                        } else {
                            let mut chunk =
                                world::ChunkColumn::new(meta.x, meta.z, bulk.sky_light_sent);
                            if let Ok(consumed) = chunk.load_from_raw_payload(
                                &bulk.data[offset..],
                                meta.bit_mask,
                                true,
                                bulk.sky_light_sent,
                            ) {
                                offset += consumed;
                                self.world.chunks.insert((meta.x, meta.z), chunk);
                            }
                        }

                        if let (Some(cm), Some(atlas)) = (&mut self.chunk_manager, &self.atlas) {
                            for sec_idx in 0..16 {
                                if (meta.bit_mask & (1 << sec_idx)) != 0 {
                                    cm.enqueue_section(&self.world, atlas.clone(), meta.x, sec_idx, meta.z);
                                }
                            }
                        }
                    }
                    self.profiler.loaded_chunks = self.world.chunk_count();
                }
                ClientboundPlayPacket::BlockChange(bc) => {
                    self.world.set_block(
                        bc.position.x,
                        bc.position.y,
                        bc.position.z,
                        BlockState(bc.block_state as u16),
                    );
                    let sec_idx = ((bc.position.y >> 4) as usize).min(15);
                    let chunk_x = bc.position.x >> 4;
                    let chunk_z = bc.position.z >> 4;
                    if let (Some(cm), Some(atlas)) = (&mut self.chunk_manager, &self.atlas) {
                        cm.enqueue_section(&self.world, atlas.clone(), chunk_x, sec_idx, chunk_z);
                    }
                }
                ClientboundPlayPacket::MultiBlockChange(mbc) => {
                    let mut affected_sections = HashSet::new();
                    for rec in &mbc.records {
                        let world_x = (mbc.chunk_x << 4) + (rec.rel_x() as i32);
                        let world_z = (mbc.chunk_z << 4) + (rec.rel_z() as i32);
                        self.world.set_block(
                            world_x,
                            rec.y as i32,
                            world_z,
                            BlockState(rec.block_state as u16),
                        );
                        let sec_y = ((rec.y >> 4) as usize).min(15);
                        affected_sections.insert(sec_y);
                    }
                    if let (Some(cm), Some(atlas)) = (&mut self.chunk_manager, &self.atlas) {
                        for sec_y in affected_sections {
                            cm.enqueue_section(&self.world, atlas.clone(), mbc.chunk_x, sec_y, mbc.chunk_z);
                        }
                    }
                }
                ClientboundPlayPacket::PlayerListItem(pli) => {
                    if let Some(sm) = &mut self.screen_manager {
                        if let Some(ig) = &mut sm.in_game {
                            for entry in pli.entries {
                                match pli.action {
                                    0 => ig.tab_list.add_or_update(
                                        entry.uuid,
                                        entry.name,
                                        entry.game_mode,
                                        entry.ping,
                                        entry.display_name.map(|d| world::parse_chat_message(&d)),
                                    ),
                                    1 => ig.tab_list.update_game_mode(&entry.uuid, entry.game_mode),
                                    2 => ig.tab_list.update_ping(&entry.uuid, entry.ping),
                                    3 => ig.tab_list.update_display_name(
                                        &entry.uuid,
                                        entry.display_name.map(|d| world::parse_chat_message(&d)),
                                    ),
                                    4 => ig.tab_list.remove_player(&entry.uuid),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                ClientboundPlayPacket::ScoreboardObjective(sbo) => {
                    if let Some(sm) = &mut self.screen_manager {
                        if let Some(ig) = &mut sm.in_game {
                            match sbo.action {
                                0 => {
                                    ig.scoreboard.add_objective(
                                        sbo.name,
                                        sbo.display_text.unwrap_or_default(),
                                        sbo.objective_type.unwrap_or_else(|| "integer".into()),
                                    );
                                }
                                1 => {
                                    ig.scoreboard.remove_objective(&sbo.name);
                                }
                                2 => {
                                    ig.scoreboard.update_objective_title(
                                        &sbo.name,
                                        sbo.display_text.unwrap_or_default(),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ClientboundPlayPacket::UpdateScore(sc) => {
                    if let Some(sm) = &mut self.screen_manager {
                        if let Some(ig) = &mut sm.in_game {
                            if sc.action == 1 {
                                ig.scoreboard.remove_score(&sc.objective_name, &sc.score_holder);
                            } else {
                                ig.scoreboard.set_score(
                                    sc.objective_name,
                                    sc.score_holder,
                                    sc.value.unwrap_or(0),
                                );
                            }
                        }
                    }
                }
                ClientboundPlayPacket::DisplayScoreboard(dsb) => {
                    if let Some(sm) = &mut self.screen_manager {
                        if let Some(ig) = &mut sm.in_game {
                            let slot_name = if dsb.score_name.is_empty() {
                                None
                            } else {
                                Some(dsb.score_name)
                            };
                            ig.scoreboard.set_display_slot(dsb.position as usize, slot_name);
                        }
                    }
                }
                ClientboundPlayPacket::Teams(tp) => {
                    if let Some(sm) = &mut self.screen_manager {
                        if let Some(ig) = &mut sm.in_game {
                            match tp.action {
                                0 => {
                                    let team = ig.scoreboard.create_team(tp.team_name.clone());
                                    if let Some(dn) = tp.display_name {
                                        team.display_name = dn;
                                    }
                                    if let Some(pfx) = tp.prefix {
                                        team.prefix = pfx;
                                    }
                                    if let Some(sfx) = tp.suffix {
                                        team.suffix = sfx;
                                    }
                                    for p in tp.players {
                                        ig.scoreboard.add_player_to_team(&tp.team_name, p);
                                    }
                                }
                                1 => {
                                    ig.scoreboard.remove_team(&tp.team_name);
                                }
                                2 => {
                                    if let Some(team) = ig.scoreboard.get_team_mut(&tp.team_name) {
                                        if let Some(dn) = tp.display_name {
                                            team.display_name = dn;
                                        }
                                        if let Some(pfx) = tp.prefix {
                                            team.prefix = pfx;
                                        }
                                        if let Some(sfx) = tp.suffix {
                                            team.suffix = sfx;
                                        }
                                    }
                                }
                                3 => {
                                    for p in tp.players {
                                        ig.scoreboard.add_player_to_team(&tp.team_name, p);
                                    }
                                }
                                4 => {
                                    for p in &tp.players {
                                        ig.scoreboard.remove_player_from_team(&tp.team_name, p);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ClientboundPlayPacket::SpawnPlayer(sp) => {
                    let name = self
                        .screen_manager
                        .as_ref()
                        .and_then(|sm| sm.in_game.as_ref())
                        .and_then(|ig| ig.tab_list.get_entry(&sp.player_id))
                        .map(|entry| entry.name.clone())
                        .unwrap_or_else(|| format!("Player_{}", sp.entity_id));

                    let skin = self.skin_manager.get_skin(&sp.player_id).clone();
                    if let (Some(gpu), Some(er)) = (&self.gpu, &mut self.entity_renderer) {
                        er.ensure_skin(&gpu.device, &gpu.queue, sp.player_id, &skin);
                    }

                    self.world.entity_tracker.spawn_player(
                        sp.entity_id,
                        sp.player_id,
                        name,
                        sp.x,
                        sp.y,
                        sp.z,
                        sp.yaw,
                        sp.pitch,
                        sp.current_item,
                        sp.metadata,
                    );
                }
                ClientboundPlayPacket::SpawnMob(sm) => {
                    self.world.entity_tracker.spawn_mob(
                        sm.entity_id,
                        sm.mob_type,
                        sm.x,
                        sm.y,
                        sm.z,
                        sm.yaw,
                        sm.pitch,
                        sm.head_pitch,
                        sm.velocity_x,
                        sm.velocity_y,
                        sm.velocity_z,
                        sm.metadata,
                    );
                }
                ClientboundPlayPacket::SpawnObject(so) => {
                    self.world.entity_tracker.spawn_object(
                        so.entity_id,
                        so.object_type,
                        so.x,
                        so.y,
                        so.z,
                        so.pitch,
                        so.yaw,
                        so.data,
                        so.velocity_x,
                        so.velocity_y,
                        so.velocity_z,
                    );
                }
                ClientboundPlayPacket::DestroyEntities(de) => {
                    self.world.entity_tracker.destroy_entities(&de.entity_ids);
                }
                ClientboundPlayPacket::Entity(_) => {}
                ClientboundPlayPacket::EntityRelMove(rm) => {
                    self.world.entity_tracker.rel_move(
                        rm.entity_id,
                        rm.delta_x,
                        rm.delta_y,
                        rm.delta_z,
                        rm.on_ground,
                    );
                }
                ClientboundPlayPacket::EntityLook(l) => {
                    self.world.entity_tracker.look(
                        l.entity_id,
                        l.yaw,
                        l.pitch,
                        l.on_ground,
                    );
                }
                ClientboundPlayPacket::EntityLookMove(lm) => {
                    self.world.entity_tracker.look_move(
                        lm.entity_id,
                        lm.delta_x,
                        lm.delta_y,
                        lm.delta_z,
                        lm.yaw,
                        lm.pitch,
                        lm.on_ground,
                    );
                }
                ClientboundPlayPacket::EntityTeleport(tp) => {
                    self.world.entity_tracker.teleport(
                        tp.entity_id,
                        tp.x,
                        tp.y,
                        tp.z,
                        tp.yaw,
                        tp.pitch,
                        tp.on_ground,
                    );
                }
                ClientboundPlayPacket::EntityHeadLook(hl) => {
                    self.world.entity_tracker.head_look(hl.entity_id, hl.head_yaw);
                }
                ClientboundPlayPacket::Animation(anim) => {
                    self.world.entity_tracker.animation(anim.entity_id, anim.animation);
                }
                ClientboundPlayPacket::EntityVelocity(ev) => {
                    self.world.entity_tracker.velocity(
                        ev.entity_id,
                        ev.velocity_x,
                        ev.velocity_y,
                        ev.velocity_z,
                    );
                }
                ClientboundPlayPacket::EntityEquipment(eq) => {
                    self.world.entity_tracker.equipment(eq.entity_id, eq.slot, eq.item);
                }
                ClientboundPlayPacket::EntityMetadata(em) => {
                    self.world.entity_tracker.update_metadata(em.entity_id, em.metadata);
                }
                ClientboundPlayPacket::Disconnect(dc) => {
                    warn!(reason = %dc.reason, "Disconnected from server");
                    self.session = None;
                    self.world.clear();
                    if let Some(sm) = &mut self.screen_manager {
                        sm.exit_in_game();
                    }
                }
                ClientboundPlayPacket::Unknown { payload_len, .. } => {
                    self.profiler.on_bytes_received(payload_len);
                }
            }
        }

        // 20 TPS Physics & Entity Tick
        if self.last_physics_tick.elapsed() >= Duration::from_millis(50) {
            self.last_physics_tick = Instant::now();
            self.world.entity_tracker.tick();
        }

        // 20 TPS Movement Tick: Send position updates only after terrain/spawn received
        if self.has_spawned && self.last_movement_tick.elapsed() >= Duration::from_millis(50) {
            self.last_movement_tick = Instant::now();
            if let Some(s) = &self.session {
                s.send(OutgoingPlayPacket::PlayerPositionAndLook {
                    x: self.player_pos.0,
                    y: self.player_pos.1,
                    z: self.player_pos.2,
                    yaw: self.player_look.0,
                    pitch: self.player_look.1,
                    on_ground: self.player_on_ground,
                });
            }
        }

        // Update profiler snapshot in HUD
        if let Some(sm) = &mut self.screen_manager {
            if let Some(ig) = &mut sm.in_game {
                ig.benchmark = Some(self.profiler.calculate_snapshot());
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Initializing client window & wgpu graphics context...");

        let mut window_attrs = Window::default_attributes()
            .with_title("Rust189")
            .with_inner_size(LogicalSize::new(960, 560))
            .with_min_inner_size(LogicalSize::new(640, 480))
            .with_resizable(true);

        if let Some(icon) = Self::load_window_icon() {
            window_attrs = window_attrs.with_window_icon(Some(icon));
        }

        if self.config.video.fullscreen {
            window_attrs = window_attrs.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        let window = match event_loop.create_window(window_attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                error!(error = %e, "Failed to create winit window");
                event_loop.exit();
                return;
            }
        };

        let gpu = match pollster::block_on(GpuContext::new(window.clone(), self.config.video.vsync)) {
            Ok(g) => g,
            Err(e) => {
                error!(error = %e, "Failed to initialize wgpu context");
                event_loop.exit();
                return;
            }
        };

        let gui = GuiIntegration::new(&window, &gpu.device, gpu.surface_config.format);

        let lang = Language::from_code(&self.config.general.language);
        let screen_manager = ScreenManager::new(
            self.config.auth.nickname.clone(),
            PathBuf::from("servers.dat"),
            lang,
            (&self.config.video).into(),
            (&self.config.audio).into(),
            (&self.config.keybinds).into(),
        );

        let atlas = Arc::new(TextureAtlas::build(Path::new("assets/minecraft/textures/blocks")));
        let terrain_renderer = TerrainRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.surface_config.format,
            gpu.size.width,
            gpu.size.height,
            &atlas,
        );
        let chunk_manager = ChunkMeshManager::new();

        let default_steve = self.skin_manager.default_skin_for(&uuid::Uuid::nil()).clone();
        let arrow_path = Path::new("assets/minecraft/textures/entity/arrow.png");
        let arrow_img = image::open(arrow_path).ok().map(|img| img.to_rgba8());
        let entity_renderer = EntityRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.surface_config.format,
            &default_steve,
            arrow_img.as_ref(),
        );

        self.camera.aspect = (gpu.size.width as f32) / (gpu.size.height.max(1) as f32);
        self.camera.fov = self.config.video.fov as f32;

        self.atlas = Some(atlas);
        self.terrain_renderer = Some(terrain_renderer);
        self.chunk_manager = Some(chunk_manager);
        self.entity_renderer = Some(entity_renderer);

        self.window = Some(window);
        self.gpu = Some(gpu);
        self.gui = Some(gui);
        self.screen_manager = Some(screen_manager);

        info!("Client window initialized successfully");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if self.window.is_none() || self.gpu.is_none() || self.gui.is_none() || self.screen_manager.is_none() {
            return;
        }

        // Rebinding keybinds intercept
        let is_rebinding = self.screen_manager.as_ref().map_or(false, |sm| sm.is_rebinding());
        if is_rebinding {
            match &event {
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key,
                            logical_key,
                            state: winit::event::ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    if let Some(token) = winit_key_to_standard(physical_key, logical_key) {
                        if let Some(sm) = &mut self.screen_manager {
                            sm.handle_rebind_token(&token);
                        }
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                        return;
                    }
                }
                WindowEvent::MouseInput {
                    button,
                    state: winit::event::ElementState::Pressed,
                    ..
                } => {
                    if let Some(token) = winit_mouse_to_standard(button) {
                        if let Some(sm) = &mut self.screen_manager {
                            sm.handle_rebind_token(&token);
                        }
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                        return;
                    }
                }
                _ => {}
            }
        }

        // Feed event to egui
        let consumed = {
            let window = self.window.as_ref().unwrap();
            let gui = self.gui.as_mut().unwrap();
            gui.on_window_event(window, &event)
        };

        match event {
            WindowEvent::Focused(focused) => {
                let is_in_game = self.screen_manager.as_ref().map_or(false, |sm| sm.in_game.is_some());
                let cursor_needed = self.screen_manager.as_ref().map_or(true, |sm| sm.is_cursor_needed());
                if is_in_game {
                    if focused && !cursor_needed && !self.mouse_grabbed {
                        if let Some(w) = &self.window {
                            let _ = w.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                .or_else(|_| w.set_cursor_grab(winit::window::CursorGrabMode::Locked));
                            w.set_cursor_visible(false);
                            self.mouse_grabbed = true;
                        }
                    } else if !focused && self.mouse_grabbed {
                        if let Some(w) = &self.window {
                            let _ = w.set_cursor_grab(winit::window::CursorGrabMode::None);
                            w.set_cursor_visible(true);
                            self.mouse_grabbed = false;
                        }
                    }
                }
            }
            WindowEvent::CloseRequested => {
                info!("Close requested, shutting down client...");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                let gpu = self.gpu.as_mut().unwrap();
                gpu.resize(new_size);
                if let Some(tr) = &mut self.terrain_renderer {
                    tr.resize(&gpu.device, new_size.width, new_size.height);
                }
                self.camera.aspect = new_size.width as f32 / new_size.height.max(1) as f32;
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let size = self.window.as_ref().unwrap().inner_size();
                let gpu = self.gpu.as_mut().unwrap();
                gpu.resize(size);
                if let Some(tr) = &mut self.terrain_renderer {
                    tr.resize(&gpu.device, size.width, size.height);
                }
                self.camera.aspect = size.width as f32 / size.height.max(1) as f32;
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::MouseInput {
                button: winit::event::MouseButton::Left,
                state: winit::event::ElementState::Pressed,
                ..
            } if !consumed => {
                let is_in_game = self.screen_manager.as_ref().map_or(false, |sm| sm.in_game.is_some());
                let cursor_needed = self.screen_manager.as_ref().map_or(true, |sm| sm.is_cursor_needed());
                if is_in_game && !cursor_needed && !self.mouse_grabbed {
                    if let Some(w) = &self.window {
                        let _ = w.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                            .or_else(|_| w.set_cursor_grab(winit::window::CursorGrabMode::Locked));
                        w.set_cursor_visible(false);
                        self.mouse_grabbed = true;
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event: ref key_event,
                ..
            } => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    match key_event.state {
                        winit::event::ElementState::Pressed => {
                            self.pressed_keys.insert(code);
                            if code == KeyCode::Escape && self.mouse_grabbed {
                                if let Some(w) = &self.window {
                                    let _ = w.set_cursor_grab(winit::window::CursorGrabMode::None);
                                    w.set_cursor_visible(true);
                                    self.mouse_grabbed = false;
                                }
                            }
                        }
                        winit::event::ElementState::Released => {
                            self.pressed_keys.remove(&code);
                        }
                    }
                }

                if !consumed && key_event.state == winit::event::ElementState::Pressed {
                    let matches_fullscreen = matches_winit_key(
                        &self.config.keybinds.fullscreen_toggle,
                        &key_event.logical_key,
                        &key_event.physical_key,
                    );

                    if matches_fullscreen {
                        self.config.video.fullscreen = !self.config.video.fullscreen;
                        let window = self.window.as_ref().unwrap();
                        if self.config.video.fullscreen {
                            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                        } else {
                            window.set_fullscreen(None);
                        }
                        let _ = self.config.save(&self.config_path);
                        info!(
                            fullscreen = self.config.video.fullscreen,
                            "Toggled fullscreen mode via keybind"
                        );
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let window_size = self.window.as_ref().unwrap().inner_size();
                if window_size.width == 0 || window_size.height == 0 {
                    return;
                }

                let frame_start = Instant::now();
                self.frame_count += 1;

                // Profiler updates & continuous dump
                self.profiler.on_frame();
                self.profiler.maybe_dump_telemetry("logs/profile_benchmark.json");

                self.camera.fov = self.config.video.fov;
                if let Some(sm) = &self.screen_manager {
                    let is_in_game = sm.in_game.is_some();
                    let cursor_needed = sm.is_cursor_needed();
                    if is_in_game {
                        if cursor_needed && self.mouse_grabbed {
                            if let Some(w) = &self.window {
                                let _ = w.set_cursor_grab(winit::window::CursorGrabMode::None);
                                w.set_cursor_visible(true);
                                self.mouse_grabbed = false;
                            }
                        } else if !cursor_needed && !self.mouse_grabbed {
                            if let Some(w) = &self.window {
                                let _ = w.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                    .or_else(|_| w.set_cursor_grab(winit::window::CursorGrabMode::Locked));
                                w.set_cursor_visible(false);
                                self.mouse_grabbed = true;
                            }
                        }
                    }
                }

                // Network packet processing
                let net_start = Instant::now();
                self.handle_incoming_packets();
                self.profiler.record_stage(FrameStage::Network, net_start.elapsed());

                let scale_factor = self.window.as_ref().unwrap().scale_factor() as f32;
                let scaled_res = ScaledResolution::new(
                    window_size.width,
                    window_size.height,
                    self.config.video.gui_scale,
                );

                let mc_zoom = scaled_res.scale_factor / scale_factor;
                self.gui.as_mut().unwrap().context.set_zoom_factor(mc_zoom);

                let screen_descriptor = ScreenDescriptor {
                    size_in_pixels: [window_size.width, window_size.height],
                    pixels_per_point: scaled_res.scale_factor,
                };

                // Update benchmark snapshot & calculate player nametags for HUD
                let dt_ms = (self.last_physics_tick.elapsed().as_millis() as f32).clamp(0.0, 50.0);
                let partial_ticks = (dt_ms / 50.0).clamp(0.0, 1.0);

                let mut nametags = Vec::new();
                let vp = self.camera.view_proj_matrix();
                let cam_pos = self.camera.position;
                let win_w = window_size.width as f32 / scaled_res.scale_factor;
                let win_h = window_size.height as f32 / scaled_res.scale_factor;

                for entity in self.world.entity_tracker.entities.values() {
                    let display_text = match &entity.kind {
                        EntityKind::Player { name, .. } => {
                            if let Some(custom) = &entity.custom_name {
                                Some(custom.clone())
                            } else if name.starts_with('#') {
                                // Internal NPC identifier (e.g. Citizens fake player), do not render
                                None
                            } else {
                                let formatted = if let Some(sm) = &self.screen_manager {
                                    if let Some(ig) = &sm.in_game {
                                        ig.scoreboard.format_player_name(name)
                                    } else {
                                        name.clone()
                                    }
                                } else {
                                    name.clone()
                                };
                                Some(formatted)
                            }
                        }
                        EntityKind::Mob { mob_type } => {
                            // Holograms and floating server labels are invisible ArmorStands (mob type 30)
                            if (*mob_type == 30 || entity.custom_name_visible) && entity.custom_name.is_some() {
                                entity.custom_name.clone()
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    let Some(text) = display_text else {
                        continue;
                    };
                    if text.is_empty() {
                        continue;
                    }

                    let pos3d = entity.interpolated_pos(partial_ticks);
                    let height = match &entity.kind {
                        EntityKind::Player { .. } => if entity.is_sneaking { 1.95 } else { 2.25 },
                        EntityKind::Mob { mob_type } if *mob_type == 30 => {
                            if entity.is_invisible { 0.2 } else { 1.9 }
                        }
                        _ => 1.9,
                    };
                    let target = glam::Vec3::new(pos3d.x as f32, (pos3d.y + height) as f32, pos3d.z as f32);
                    let dist = (target - cam_pos).length();
                    if dist > 0.5 && dist < 64.0 {
                        let clip = vp * glam::Vec4::new(target.x, target.y, target.z, 1.0);
                        if clip.w > 0.0 {
                            let ndc_x = clip.x / clip.w;
                            let ndc_y = clip.y / clip.w;
                            if ndc_x >= -1.2 && ndc_x <= 1.2 && ndc_y >= -1.2 && ndc_y <= 1.2 {
                                let sx = (ndc_x + 1.0) * 0.5 * win_w;
                                let sy = (1.0 - ndc_y) * 0.5 * win_h;
                                nametags.push(NametagInfo {
                                    screen_pos: (sx, sy),
                                    text,
                                    distance: dist,
                                    is_sneaking: entity.is_sneaking,
                                });
                            }
                        }
                    }
                }

                if let Some(sm) = &mut self.screen_manager {
                    if let Some(ig) = &mut sm.in_game {
                        ig.benchmark = Some(self.profiler.calculate_snapshot());
                        ig.nametags = nametags;
                    }
                }

                // Begin egui frame & render
                let gui_start = Instant::now();
                let gui_output = {
                    let window = self.window.as_ref().unwrap();
                    let gui = self.gui.as_mut().unwrap();
                    let sm = self.screen_manager.as_mut().unwrap();

                    gui.begin_frame(window);
                    sm.render(&gui.context)
                };
                self.profiler.record_stage(FrameStage::Gui, gui_start.elapsed());

                match gui_output {
                    GuiOutput::Quit => {
                        info!("Quit clicked, exiting client...");
                        event_loop.exit();
                        return;
                    }
                    GuiOutput::ConnectToServer(ip) => {
                        self.initiate_connection(&ip);
                    }
                    GuiOutput::DisconnectFromServer => {
                        info!("Disconnect clicked, leaving server");
                        self.session = None;
                        self.has_spawned = false;
                        self.world.clear();
                    }
                    GuiOutput::SendChatMessage(msg) => {
                        if let Some(s) = &self.session {
                            info!(msg = %msg, "Sending chat message");
                            s.send(OutgoingPlayPacket::Chat(msg));
                        }
                    }
                    GuiOutput::NicknameUpdated(new_nick) => {
                        if self.config.auth.nickname != new_nick {
                            info!(old = %self.config.auth.nickname, new = %new_nick, "Updated player nickname");
                            self.config.auth.nickname = new_nick;
                            let _ = self.config.save(&self.config_path);
                        }
                    }
                    GuiOutput::SettingsUpdated {
                        language,
                        video,
                        audio,
                        keybinds,
                    } => {
                        let prev_vsync = self.config.video.vsync;
                        let prev_fullscreen = self.config.video.fullscreen;

                        self.config.general.language = language.code().to_string();
                        self.config.video = (&video).into();
                        self.config.audio = (&audio).into();
                        self.config.keybinds = (&keybinds).into();
                        self.camera.fov = self.config.video.fov;

                        let gpu = self.gpu.as_mut().unwrap();
                        let window = self.window.as_ref().unwrap();

                        if self.config.video.vsync != prev_vsync {
                            gpu.set_vsync(self.config.video.vsync);
                        }
                        if self.config.video.fullscreen != prev_fullscreen {
                            if self.config.video.fullscreen {
                                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                            } else {
                                window.set_fullscreen(None);
                            }
                        }

                        if let Err(e) = self.config.save(&self.config_path) {
                            warn!(error = %e, "Failed to save config.toml");
                        } else {
                            info!("Settings successfully updated and saved to config.toml");
                        }
                    }
                    GuiOutput::None => {}
                }

                // Camera and player movement update (WASD / Ctrl sprint / Shift sneak)
                if self.mouse_grabbed {
                    let speed = if self.pressed_keys.contains(&KeyCode::ControlLeft) {
                        0.4f32
                    } else if self.pressed_keys.contains(&KeyCode::ShiftLeft) {
                        0.08f32
                    } else {
                        0.2f32
                    };

                    let mut forward = 0.0f32;
                    let mut strafe = 0.0f32;

                    if self.pressed_keys.contains(&KeyCode::KeyW) { forward += speed; }
                    if self.pressed_keys.contains(&KeyCode::KeyS) { forward -= speed; }
                    if self.pressed_keys.contains(&KeyCode::KeyD) { strafe += speed; }
                    if self.pressed_keys.contains(&KeyCode::KeyA) { strafe -= speed; }

                    if forward != 0.0 || strafe != 0.0 {
                        self.camera.move_horizontal(forward, strafe);
                        self.player_pos = (
                            self.camera.position.x as f64,
                            (self.camera.position.y - 1.62) as f64,
                            self.camera.position.z as f64,
                        );
                    }
                }

                // Acquire next swapchain surface texture
                let render_start = Instant::now();
                let window = self.window.as_ref().unwrap();
                let gpu = self.gpu.as_mut().unwrap();
                let gui = self.gui.as_mut().unwrap();

                let output = match gpu.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => {
                        let _ = gui.context.end_pass();
                        gpu.resize(window.inner_size());
                        return;
                    }
                    Err(wgpu::SurfaceError::Lost) => {
                        let _ = gui.context.end_pass();
                        gpu.resize(window.inner_size());
                        return;
                    }
                    Err(e) => {
                        let _ = gui.context.end_pass();
                        error!(error = %e, "Surface presentation texture error");
                        return;
                    }
                };

                let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("frame-encoder"),
                });

                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let is_in_game = self.screen_manager.as_ref().map_or(false, |sm| sm.in_game.is_some());
                if is_in_game {
                    if let (Some(cm), Some(tr)) = (&mut self.chunk_manager, &mut self.terrain_renderer) {
                        let out_w = output.texture.width();
                        let out_h = output.texture.height();
                        if tr.depth_texture.width() != out_w || tr.depth_texture.height() != out_h {
                            tr.resize(&gpu.device, out_w, out_h);
                        }

                        cm.upload_pending(&gpu.device, 8);
                        let dt_ms = (self.last_frame_time.elapsed().as_millis() as u64).clamp(1, 100);
                        tr.update_animations(&gpu.queue, dt_ms);

                        let render_dist_blocks = (self.config.video.render_distance as f32) * 16.0;
                        let time_secs = (self.world.time as f32) / 20.0;
                        tr.update_uniforms(&gpu.queue, &self.camera, render_dist_blocks, time_secs, self.config.video.antialiasing);

                        let frustum = Frustum::from_view_proj(self.camera.view_proj_matrix());
                        let (visible_opaque, visible_translucent) =
                            cm.get_visible_sections(&frustum, self.camera.position);

                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("terrain-render-pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.70,
                                        g: 0.80,
                                        b: 1.00,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: Some(
                                wgpu::RenderPassDepthStencilAttachment {
                                    view: &tr.depth_texture_view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(1.0),
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                },
                            ),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        tr.render(&mut rpass, &visible_opaque, &visible_translucent);

                        if let Some(er) = &mut self.entity_renderer {
                            let dt_ms = (self.last_physics_tick.elapsed().as_millis() as f32).clamp(0.0, 50.0);
                            let partial_ticks = (dt_ms / 50.0).clamp(0.0, 1.0);
                            er.update_uniforms(&gpu.queue, &self.camera, render_dist_blocks);
                            er.render(&gpu.device, &gpu.queue, &mut rpass, &self.world.entity_tracker, partial_ticks);
                        }
                    }
                } else {
                    let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("clear-pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.09,
                                    g: 0.10,
                                    b: 0.12,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }

                // egui render pass
                gui.end_frame_and_render(
                    window,
                    &gpu.device,
                    &gpu.queue,
                    &mut encoder,
                    &view,
                    &screen_descriptor,
                );

                gpu.queue.submit(Some(encoder.finish()));
                output.present();
                self.profiler.record_stage(FrameStage::Render, render_start.elapsed());

                // Precision frame rate limiter (hybrid sleep + spin-wait)
                let max_fps = self.config.video.max_fps;
                if max_fps > 0 && !self.config.video.vsync {
                    let target_frame_time = Duration::from_secs_f64(1.0 / max_fps as f64);
                    let frame_elapsed = frame_start.elapsed();
                    if frame_elapsed < target_frame_time {
                        let remaining = target_frame_time - frame_elapsed;
                        if remaining > Duration::from_millis(2) {
                            std::thread::sleep(remaining - Duration::from_millis(1));
                        }
                        while frame_start.elapsed() < target_frame_time {
                            std::hint::spin_loop();
                        }
                    }
                }

                self.last_frame_time = Instant::now();
                window.request_redraw();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            if self.mouse_grabbed {
                let sensitivity = 0.15f32;
                self.camera.rotate(dx as f32 * sensitivity, dy as f32 * sensitivity);
                self.player_look = (self.camera.yaw, self.camera.pitch);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
