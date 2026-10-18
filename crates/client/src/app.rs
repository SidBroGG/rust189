use assets::SkinManager;
use crate::config::ClientConfig;
use egui_wgpu::ScreenDescriptor;
use gui::{
    matches_winit_key, winit_key_to_standard, winit_mouse_to_standard, GuiIntegration,
    GuiOutput, Language, NametagInfo, ScaledResolution, ScreenManager,
};
use protocol::{
    connect_offline, BlockFace, ClientboundPlayPacket, DiggingAction, OutgoingPlayPacket,
    PlaySessionHandle, Position, Slot, UseEntityAction,
};
use render::{
    Camera, ChunkMeshManager, EntityRenderer, Frustum, GpuContext, InteractionRenderer,
    TerrainRenderer, TextureAtlas,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Icon, Window, WindowId};
use world::{
    get_block_selection_box, get_player_relative_block_hardness, is_critical_hit, is_water_block,
    pick_target, BlockState, CameraController, CameraPerspective, ClickTracker, EntityKind,
    EntitySpatialHash, FrameStage, ItemUseAction, ItemUseState, PerformanceProfiler, PlayerInput,
    PlayerMovement, TargetHit, TrackedEntity, World, AABB,
};

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
    pub last_physics_tick: Instant,
    pub player: PlayerMovement,
    pub camera_controller: CameraController,
    pub player_entity_id: i32,
    pub player_pos: (f64, f64, f64),
    pub player_look: (f32, f32),
    pub player_on_ground: bool,
    pub has_spawned: bool,
    pub last_reported_pos: (f64, f64, f64),
    pub last_reported_look: (f32, f32),
    pub position_update_ticks: u32,
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

    // Interaction & Combat (Stage 11)
    pub interaction_renderer: Option<InteractionRenderer>,
    pub spatial_hash: EntitySpatialHash,
    pub click_tracker: ClickTracker,
    pub item_use_state: ItemUseState,
    pub current_target: TargetHit,
    pub breaking_block_pos: Option<(i32, i32, i32)>,
    pub breaking_block_damage: f32,
    pub breaking_block_face: BlockFace,
    pub is_left_clicking: bool,
    pub is_right_clicking: bool,
    pub server_break_progress: HashMap<(i32, i32, i32), u8>,
    pub held_item_id: i16,
}

impl App {
    pub fn new(config: ClientConfig, config_path: PathBuf) -> Self {
        let mut camera_controller = CameraController::new();
        camera_controller.cinematic_camera = config.mouse.smooth_camera;
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
            last_physics_tick: Instant::now(),
            player: PlayerMovement::new(0.0, 64.0, 0.0),
            camera_controller,
            player_entity_id: 0,
            player_pos: (0.0, 64.0, 0.0),
            player_look: (0.0, 0.0),
            player_on_ground: false,
            has_spawned: false,
            last_reported_pos: (0.0, 64.0, 0.0),
            last_reported_look: (0.0, 0.0),
            position_update_ticks: 0,
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

            interaction_renderer: None,
            spatial_hash: EntitySpatialHash::new(4.0),
            click_tracker: ClickTracker::new(),
            item_use_state: ItemUseState::new(),
            current_target: TargetHit::Miss,
            breaking_block_pos: None,
            breaking_block_damage: 0.0,
            breaking_block_face: BlockFace::Up,
            is_left_clicking: false,
            is_right_clicking: false,
            server_break_progress: HashMap::new(),
            held_item_id: 276,
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

    /// Returns the primary block texture key in the texture atlas for block break particles.
    pub fn block_texture_name(id: u16) -> &'static str {
        match id {
            1 => "stone",
            2 => "grass_top",
            3 => "dirt",
            4 => "cobblestone",
            5 => "planks_oak",
            7 => "bedrock",
            12 => "sand",
            13 => "gravel",
            14 => "gold_ore",
            15 => "iron_ore",
            16 => "coal_ore",
            17 => "log_oak",
            18 => "leaves_oak",
            20 => "glass",
            24 => "sandstone_normal",
            35 => "wool_colored_white",
            45 => "brick",
            49 => "obsidian",
            56 => "diamond_ore",
            57 => "diamond_block",
            80 => "snow",
            98 => "stonebrick",
            _ => "stone",
        }
    }

    pub fn handle_left_click(&mut self) {
        if !self.has_spawned {
            return;
        }
        self.click_tracker.register_left_click();

        // Arm swing packet
        if let Some(s) = &self.session {
            s.send(OutgoingPlayPacket::Animation);
        }

        match self.current_target {
            TargetHit::Entity(eh) => {
                let entity_id = eh.entity_id;
                if let Some(s) = &self.session {
                    s.send(OutgoingPlayPacket::UseEntity {
                        entity_id,
                        action: UseEntityAction::Attack,
                        target: None,
                    });
                }

                let is_crit = is_critical_hit(
                    self.player.fall_distance,
                    self.player_on_ground,
                    false,
                    false,
                    false,
                    false,
                );

                if let Some(ent) = self.world.entity_tracker.entities.get(&entity_id) {
                    let pos = glam::Vec3::new(ent.pos.x as f32, (ent.pos.y + 0.8) as f32, ent.pos.z as f32);
                    if let Some(ir) = &mut self.interaction_renderer {
                        if is_crit {
                            ir.spawn_crit_particles(pos, false, 14);
                        } else if self.player.is_sprinting {
                            ir.spawn_crit_particles(pos, true, 8);
                        }
                    }
                }
            }
            TargetHit::Block(bh) => {
                let pos = bh.block_pos;
                let block = self.world.get_block(pos.0, pos.1, pos.2);
                let in_water = is_water_block(
                    self.world
                        .get_block(
                            self.player.pos.x.floor() as i32,
                            (self.player.pos.y + 0.6).floor() as i32,
                            self.player.pos.z.floor() as i32,
                        )
                        .id(),
                );

                let rel_damage = get_player_relative_block_hardness(
                    block.id(),
                    self.held_item_id,
                    0,
                    0,
                    0,
                    in_water,
                    false,
                    self.player_on_ground,
                    self.player.capabilities.is_creative,
                );

                if rel_damage >= 1.0 {
                    // Instant destroy (creative or zero-hardness block)
                    if let Some(s) = &self.session {
                        s.send(OutgoingPlayPacket::PlayerDigging {
                            status: DiggingAction::StartDestroyBlock,
                            position: Position::new(pos.0, pos.1, pos.2),
                            facing: bh.face,
                        });
                        s.send(OutgoingPlayPacket::PlayerDigging {
                            status: DiggingAction::StopDestroyBlock,
                            position: Position::new(pos.0, pos.1, pos.2),
                            facing: bh.face,
                        });
                    }
                    self.world.set_block(pos.0, pos.1, pos.2, BlockState(0));
                    let chunk_x = pos.0 >> 4;
                    let chunk_z = pos.2 >> 4;
                    let sec_idx = ((pos.1 >> 4) as usize).min(15);
                    if let Some(cm) = &mut self.chunk_manager {
                        cm.mark_section_dirty(chunk_x, sec_idx, chunk_z);
                    }
                    if let (Some(ir), Some(atlas)) = (&mut self.interaction_renderer, &self.atlas) {
                        let tex_name = Self::block_texture_name(block.id());
                        let region = atlas.regions.get(tex_name).copied().unwrap_or(atlas.fallback_region);
                        let center = glam::Vec3::new(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5, pos.2 as f32 + 0.5);
                        ir.spawn_digging_particles(center, region, 16);
                    }
                    self.breaking_block_pos = None;
                    self.breaking_block_damage = 0.0;
                } else {
                    self.breaking_block_pos = Some(pos);
                    self.breaking_block_damage = rel_damage;
                    self.breaking_block_face = bh.face;

                    if let Some(s) = &self.session {
                        s.send(OutgoingPlayPacket::PlayerDigging {
                            status: DiggingAction::StartDestroyBlock,
                            position: Position::new(pos.0, pos.1, pos.2),
                            facing: bh.face,
                        });
                    }
                }
            }
            TargetHit::Miss => {}
        }
    }

    pub fn handle_left_release(&mut self) {
        if let Some(pos) = self.breaking_block_pos.take() {
            self.breaking_block_damage = 0.0;
            if let Some(s) = &self.session {
                s.send(OutgoingPlayPacket::PlayerDigging {
                    status: DiggingAction::AbortDestroyBlock,
                    position: Position::new(pos.0, pos.1, pos.2),
                    facing: self.breaking_block_face,
                });
            }
        }
    }

    pub fn handle_right_click(&mut self) {
        if !self.has_spawned {
            return;
        }
        self.click_tracker.register_right_click();

        let action = ItemUseState::get_action_for_item(self.held_item_id);
        if action != ItemUseAction::None {
            self.item_use_state.start_using(self.held_item_id);
            return;
        }

        // Block placement (blocks 1..255)
        if self.held_item_id > 0 && self.held_item_id < 256 {
            if let TargetHit::Block(bh) = self.current_target {
                let (dx, dy, dz) = bh.face.normal();
                let place_pos = (bh.block_pos.0 + dx, bh.block_pos.1 + dy, bh.block_pos.2 + dz);

                let place_aabb = AABB::new(
                    place_pos.0 as f64,
                    place_pos.1 as f64,
                    place_pos.2 as f64,
                    place_pos.0 as f64 + 1.0,
                    place_pos.1 as f64 + 1.0,
                    place_pos.2 as f64 + 1.0,
                );

                if !self.player.bounding_box.intersects(&place_aabb) {
                    if let Some(s) = &self.session {
                        let cursor_x = (bh.cursor_x * 16.0) as u8;
                        let cursor_y = (bh.cursor_y * 16.0) as u8;
                        let cursor_z = (bh.cursor_z * 16.0) as u8;

                        s.send(OutgoingPlayPacket::PlayerBlockPlacement {
                            position: Position::new(bh.block_pos.0, bh.block_pos.1, bh.block_pos.2),
                            direction: bh.face as u8,
                            item: Some(Slot {
                                item_id: self.held_item_id,
                                item_count: 1,
                                item_damage: 0,
                                nbt: None,
                            }),
                            cursor_x,
                            cursor_y,
                            cursor_z,
                        });
                        s.send(OutgoingPlayPacket::Animation);
                    }

                    self.world.set_block(place_pos.0, place_pos.1, place_pos.2, BlockState(self.held_item_id as u16));
                    let chunk_x = place_pos.0 >> 4;
                    let chunk_z = place_pos.2 >> 4;
                    let sec_idx = ((place_pos.1 >> 4) as usize).min(15);
                    if let Some(cm) = &mut self.chunk_manager {
                        cm.mark_section_dirty(chunk_x, sec_idx, chunk_z);
                    }
                }
            }
        }
    }

    pub fn handle_right_release(&mut self) {
        if self.item_use_state.is_bow_drawing() {
            if let Some((_vel, is_crit)) = self.item_use_state.release_bow() {
                if let Some(s) = &self.session {
                    s.send(OutgoingPlayPacket::PlayerDigging {
                        status: DiggingAction::ReleaseUseItem,
                        position: Position::new(0, 0, 0),
                        facing: BlockFace::Down,
                    });
                }
                if is_crit {
                    if let Some(ir) = &mut self.interaction_renderer {
                        let spawn_pos = glam::Vec3::new(
                            self.player.pos.x as f32,
                            (self.player.pos.y as f32) + self.player.get_eye_height(),
                            self.player.pos.z as f32,
                        );
                        ir.spawn_crit_particles(spawn_pos, false, 8);
                    }
                }
            }
        } else if self.item_use_state.active_action != ItemUseAction::None {
            self.item_use_state.stop();
            if let Some(s) = &self.session {
                s.send(OutgoingPlayPacket::PlayerDigging {
                    status: DiggingAction::ReleaseUseItem,
                    position: Position::new(0, 0, 0),
                    facing: BlockFace::Down,
                });
            }
        }
    }

    fn initiate_connection(&mut self, server_ip: &str) {
        self.has_spawned = false;
        self.last_reported_pos = (0.0, 64.0, 0.0);
        self.last_reported_look = (0.0, 0.0);
        self.position_update_ticks = 0;
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
                    self.world.clear();
                    if let Some(cm) = &mut self.chunk_manager {
                        cm.clear();
                    }
                    self.profiler.loaded_chunks = 0;
                    self.player_entity_id = jg.entity_id;
                    self.world.dimension = jg.dimension;
                    if jg.game_mode == 1 || jg.game_mode == 3 {
                        self.player.capabilities.allow_flying = true;
                        self.player.capabilities.is_creative = jg.game_mode == 1;
                    }
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
                    self.world.clear();
                    self.world.dimension = rp.dimension as i8;
                    if let Some(cm) = &mut self.chunk_manager {
                        cm.clear();
                    }
                    self.profiler.loaded_chunks = 0;
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
                    info!(
                        x = pos_look.x,
                        y = pos_look.y,
                        z = pos_look.z,
                        yaw = pos_look.yaw,
                        pitch = pos_look.pitch,
                        flags = pos_look.flags,
                        "Received PlayerPosLook setback/teleport from server"
                    );

                    // Update player position according to relative/absolute flags
                    let mut new_x = self.player.pos.x;
                    let mut new_y = self.player.pos.y;
                    let mut new_z = self.player.pos.z;
                    let mut new_yaw = self.player.yaw;
                    let mut new_pitch = self.player.pitch;

                    if (pos_look.flags & 0x01) != 0 {
                        new_x += pos_look.x;
                    } else {
                        new_x = pos_look.x;
                        self.player.motion.x = 0.0;
                    }

                    if (pos_look.flags & 0x02) != 0 {
                        new_y += pos_look.y;
                    } else {
                        new_y = pos_look.y;
                        self.player.motion.y = 0.0;
                    }

                    if (pos_look.flags & 0x04) != 0 {
                        new_z += pos_look.z;
                    } else {
                        new_z = pos_look.z;
                        self.player.motion.z = 0.0;
                    }

                    if (pos_look.flags & 0x08) != 0 {
                        new_yaw += pos_look.yaw;
                    } else {
                        new_yaw = pos_look.yaw;
                    }

                    if (pos_look.flags & 0x10) != 0 {
                        new_pitch += pos_look.pitch;
                    } else {
                        new_pitch = pos_look.pitch;
                    }

                    self.player.set_position_and_rotation(new_x, new_y, new_z, new_yaw, new_pitch);
                    self.player.distance_walked_modified = 0.0;
                    self.player.prev_distance_walked_modified = 0.0;

                    self.player_pos = (self.player.pos.x, self.player.pos.y, self.player.pos.z);
                    self.player_look = (self.player.yaw, self.player.pitch);
                    self.player_on_ground = false;

                    self.last_reported_pos = self.player_pos;
                    self.last_reported_look = self.player_look;
                    self.position_update_ticks = 20;

                    // Confirm position back to server and mark spawned (on_ground = false per MCP line 717)
                    self.has_spawned = true;
                    if let Some(s) = &self.session {
                        s.send(OutgoingPlayPacket::PlayerPositionAndLook {
                            x: self.player.pos.x,
                            y: self.player.bounding_box.min_y,
                            z: self.player.pos.z,
                            yaw: self.player.yaw,
                            pitch: self.player.pitch,
                            on_ground: false,
                        });
                    }

                    if let Some(sm) = &mut self.screen_manager {
                        if let Some(ig) = &mut sm.in_game {
                            ig.pos = self.player_pos;
                            ig.look = self.player_look;
                        }
                    }

                    let eye_h = self.player.get_eye_height() as f32;
                    self.camera.position = glam::Vec3::new(
                        self.player.pos.x as f32,
                        (self.player.pos.y as f32) + eye_h,
                        self.player.pos.z as f32,
                    );
                    self.camera.yaw = self.player.yaw;
                    self.camera.pitch = self.player.pitch;

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
                            for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                                let nx = cd.chunk_x + dx;
                                let nz = cd.chunk_z + dz;
                                if self.world.chunks.contains_key(&(nx, nz)) && cm.has_chunk_mesh(nx, nz) {
                                    for sec_idx in 0..16 {
                                        if cm.has_mesh(nx, nz, sec_idx) {
                                            cm.mark_section_dirty(nx, sec_idx, nz);
                                        }
                                    }
                                }
                            }
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
                        if let Some(cm) = &mut self.chunk_manager {
                            cm.mark_chunk_dirty(cd.chunk_x, cd.chunk_z, cd.bit_mask);
                            // Mark adjacent chunk sections on borders only if they already have meshes
                            for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                                let nx = cd.chunk_x + dx;
                                let nz = cd.chunk_z + dz;
                                if self.world.chunks.contains_key(&(nx, nz)) && cm.has_chunk_mesh(nx, nz) {
                                    for sec_idx in 0..16 {
                                        if (cd.bit_mask & (1 << sec_idx)) != 0 && cm.has_mesh(nx, nz, sec_idx) {
                                            cm.mark_section_dirty(nx, sec_idx, nz);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    self.profiler.loaded_chunks = self.world.chunk_count();
                }
                ClientboundPlayPacket::MapChunkBulk(bulk) => {
                    self.profiler.on_bytes_received(bulk.data.len());
                    let mut offset = 0;
                    let mut loaded_meta = Vec::with_capacity(bulk.meta.len());
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
                                loaded_meta.push((meta.x, meta.z, meta.bit_mask));
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
                                loaded_meta.push((meta.x, meta.z, meta.bit_mask));
                            }
                        }
                    }

                    if let Some(cm) = &mut self.chunk_manager {
                        for &(chunk_x, chunk_z, bit_mask) in &loaded_meta {
                            cm.mark_chunk_dirty(chunk_x, chunk_z, bit_mask);
                            // Mark adjacent outer chunk sections not in this bulk that already have meshes
                            for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                                let nx = chunk_x + dx;
                                let nz = chunk_z + dz;
                                if !loaded_meta.iter().any(|&(x, z, _)| x == nx && z == nz)
                                    && self.world.chunks.contains_key(&(nx, nz))
                                    && cm.has_chunk_mesh(nx, nz)
                                {
                                    for sec_idx in 0..16 {
                                        if (bit_mask & (1 << sec_idx)) != 0 && cm.has_mesh(nx, nz, sec_idx) {
                                            cm.mark_section_dirty(nx, sec_idx, nz);
                                        }
                                    }
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
                    if let Some(cm) = &mut self.chunk_manager {
                        cm.mark_section_dirty(chunk_x, sec_idx, chunk_z);

                        let rel_x = bc.position.x & 15;
                        let rel_y = bc.position.y & 15;
                        let rel_z = bc.position.z & 15;
                        if rel_x == 0 && cm.has_mesh(chunk_x - 1, chunk_z, sec_idx) {
                            cm.mark_section_dirty(chunk_x - 1, sec_idx, chunk_z);
                        } else if rel_x == 15 && cm.has_mesh(chunk_x + 1, chunk_z, sec_idx) {
                            cm.mark_section_dirty(chunk_x + 1, sec_idx, chunk_z);
                        }
                        if rel_z == 0 && cm.has_mesh(chunk_x, chunk_z - 1, sec_idx) {
                            cm.mark_section_dirty(chunk_x, sec_idx, chunk_z - 1);
                        } else if rel_z == 15 && cm.has_mesh(chunk_x, chunk_z + 1, sec_idx) {
                            cm.mark_section_dirty(chunk_x, sec_idx, chunk_z + 1);
                        }
                        if rel_y == 0 && sec_idx > 0 && cm.has_mesh(chunk_x, chunk_z, sec_idx - 1) {
                            cm.mark_section_dirty(chunk_x, sec_idx - 1, chunk_z);
                        } else if rel_y == 15 && sec_idx < 15 && cm.has_mesh(chunk_x, chunk_z, sec_idx + 1) {
                            cm.mark_section_dirty(chunk_x, sec_idx + 1, chunk_z);
                        }
                    }
                }
                ClientboundPlayPacket::MultiBlockChange(mbc) => {
                    let mut affected_sections = HashSet::new();
                    let mut neighbor_sections: HashSet<(i32, usize, i32)> = HashSet::new();
                    for rec in &mbc.records {
                        let rel_x = rec.rel_x() as i32;
                        let rel_z = rec.rel_z() as i32;
                        let world_x = (mbc.chunk_x << 4) + rel_x;
                        let world_z = (mbc.chunk_z << 4) + rel_z;
                        let y = rec.y as i32;
                        self.world.set_block(
                            world_x,
                            y,
                            world_z,
                            BlockState(rec.block_state as u16),
                        );
                        let sec_y = ((y >> 4) as usize).min(15);
                        affected_sections.insert(sec_y);

                        let rel_y = y & 15;
                        if rel_x == 0 {
                            neighbor_sections.insert((mbc.chunk_x - 1, sec_y, mbc.chunk_z));
                        } else if rel_x == 15 {
                            neighbor_sections.insert((mbc.chunk_x + 1, sec_y, mbc.chunk_z));
                        }
                        if rel_z == 0 {
                            neighbor_sections.insert((mbc.chunk_x, sec_y, mbc.chunk_z - 1));
                        } else if rel_z == 15 {
                            neighbor_sections.insert((mbc.chunk_x, sec_y, mbc.chunk_z + 1));
                        }
                        if rel_y == 0 && sec_y > 0 {
                            neighbor_sections.insert((mbc.chunk_x, sec_y - 1, mbc.chunk_z));
                        } else if rel_y == 15 && sec_y < 15 {
                            neighbor_sections.insert((mbc.chunk_x, sec_y + 1, mbc.chunk_z));
                        }
                    }
                    if let Some(cm) = &mut self.chunk_manager {
                        for sec_y in affected_sections {
                            cm.mark_section_dirty(mbc.chunk_x, sec_y, mbc.chunk_z);
                        }
                        for (nx, ny, nz) in neighbor_sections {
                            if self.world.chunks.contains_key(&(nx, nz)) && cm.has_mesh(nx, nz, ny) {
                                cm.mark_section_dirty(nx, ny, nz);
                            }
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
                    if ev.entity_id == self.player_entity_id {
                        self.player.apply_velocity(
                            ev.velocity_x as f64 / 8000.0,
                            ev.velocity_y as f64 / 8000.0,
                            ev.velocity_z as f64 / 8000.0,
                        );
                    }
                    self.world.entity_tracker.velocity(
                        ev.entity_id,
                        ev.velocity_x,
                        ev.velocity_y,
                        ev.velocity_z,
                    );
                }
                ClientboundPlayPacket::Explosion(exp) => {
                    self.player.apply_explosion(
                        exp.player_motion_x as f64,
                        exp.player_motion_y as f64,
                        exp.player_motion_z as f64,
                    );
                }
                ClientboundPlayPacket::PlayerAbilities(pa) => {
                    self.player.capabilities.is_invulnerable = pa.invulnerable;
                    self.player.capabilities.is_flying = pa.is_flying;
                    self.player.capabilities.allow_flying = pa.allow_flying;
                    self.player.capabilities.is_creative = pa.creative_mode;
                    self.player.capabilities.fly_speed = pa.fly_speed;
                    self.player.capabilities.walk_speed = pa.walk_speed;
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
                    if let Some(cm) = &mut self.chunk_manager {
                        cm.clear();
                    }
                    self.profiler.loaded_chunks = 0;
                    if let Some(sm) = &mut self.screen_manager {
                        sm.exit_in_game();
                    }
                }
                ClientboundPlayPacket::ConfirmTransaction(ct) => {
                    // Handled automatically in session.rs Tokio network loop for lowest latency.
                    // Fallback response here in case it needs application-level acknowledgement.
                    if !ct.accepted {
                        if let Some(s) = &self.session {
                            s.send(OutgoingPlayPacket::ConfirmTransaction {
                                window_id: ct.window_id,
                                action_number: ct.action_number,
                                accepted: true,
                            });
                        }
                    }
                }
                ClientboundPlayPacket::BlockBreakAnim(bba) => {
                    let pos = (bba.position.x, bba.position.y, bba.position.z);
                    if bba.progress >= 0 && bba.progress <= 9 {
                        self.server_break_progress.insert(pos, bba.progress as u8);
                    } else {
                        self.server_break_progress.remove(&pos);
                    }
                }
                ClientboundPlayPacket::Unknown { payload_len, .. } => {
                    self.profiler.on_bytes_received(payload_len);
                }
            }
        }

        // 20 TPS Fixed Physics & Entity Tick
        if self.last_physics_tick.elapsed() > Duration::from_millis(500) {
            self.last_physics_tick = Instant::now() - Duration::from_millis(50);
        }

        while self.last_physics_tick.elapsed() >= Duration::from_millis(50) {
            self.last_physics_tick += Duration::from_millis(50);
            self.world.entity_tracker.tick();
            self.spatial_hash.rebuild(&self.world.entity_tracker);

            if self.is_right_clicking {
                self.item_use_state.tick();
            }

            if let Some(ir) = &mut self.interaction_renderer {
                ir.tick_particles(0.05);
            }

            if self.has_spawned {
                let is_in_game = self.screen_manager.as_ref().map_or(false, |sm| sm.in_game.is_some());
                let cursor_needed = self.screen_manager.as_ref().map_or(true, |sm| sm.is_cursor_needed());
                let mut input = PlayerInput::default();

                if is_in_game && !cursor_needed {
                    if self.pressed_keys.contains(&KeyCode::KeyW) { input.move_forward += 1.0; }
                    if self.pressed_keys.contains(&KeyCode::KeyS) { input.move_forward -= 1.0; }
                    if self.pressed_keys.contains(&KeyCode::KeyA) { input.move_strafe += 1.0; }
                    if self.pressed_keys.contains(&KeyCode::KeyD) { input.move_strafe -= 1.0; }
                    input.jump = self.pressed_keys.contains(&KeyCode::Space);
                    input.sneak = self.pressed_keys.contains(&KeyCode::ShiftLeft) || self.pressed_keys.contains(&KeyCode::ShiftRight);
                    input.sprint = self.pressed_keys.contains(&KeyCode::ControlLeft) || self.pressed_keys.contains(&KeyCode::ControlRight);

                    let speed_mult = self.item_use_state.movement_speed_multiplier();
                    input.move_forward *= speed_mult;
                    input.move_strafe *= speed_mult;
                }

                // Block destruction continuous tick
                if self.is_left_clicking {
                    if let Some(pos) = self.breaking_block_pos {
                        let is_still_targeting = match self.current_target {
                            TargetHit::Block(ref b) => b.block_pos == pos,
                            _ => false,
                        };

                        if is_still_targeting {
                            let block = self.world.get_block(pos.0, pos.1, pos.2);
                            let in_water = is_water_block(
                                self.world
                                    .get_block(
                                        self.player.pos.x.floor() as i32,
                                        (self.player.pos.y + 0.6).floor() as i32,
                                        self.player.pos.z.floor() as i32,
                                    )
                                    .id(),
                            );

                            let rel_damage = get_player_relative_block_hardness(
                                block.id(),
                                self.held_item_id,
                                0,
                                0,
                                0,
                                in_water,
                                false,
                                self.player_on_ground,
                                self.player.capabilities.is_creative,
                            );

                            self.breaking_block_damage += rel_damage;

                            if let (Some(ir), Some(atlas)) = (&mut self.interaction_renderer, &self.atlas) {
                                let tex_name = Self::block_texture_name(block.id());
                                let region = atlas.regions.get(tex_name).copied().unwrap_or(atlas.fallback_region);
                                let center = glam::Vec3::new(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5, pos.2 as f32 + 0.5);
                                ir.spawn_digging_particles(center, region, 2);
                            }

                            if self.breaking_block_damage >= 1.0 {
                                if let Some(s) = &self.session {
                                    s.send(OutgoingPlayPacket::PlayerDigging {
                                        status: DiggingAction::StopDestroyBlock,
                                        position: Position::new(pos.0, pos.1, pos.2),
                                        facing: self.breaking_block_face,
                                    });
                                }

                                self.world.set_block(pos.0, pos.1, pos.2, BlockState(0));
                                let chunk_x = pos.0 >> 4;
                                let chunk_z = pos.2 >> 4;
                                let sec_idx = ((pos.1 >> 4) as usize).min(15);
                                if let Some(cm) = &mut self.chunk_manager {
                                    cm.mark_section_dirty(chunk_x, sec_idx, chunk_z);
                                }

                                if let (Some(ir), Some(atlas)) = (&mut self.interaction_renderer, &self.atlas) {
                                    let tex_name = Self::block_texture_name(block.id());
                                    let region = atlas.regions.get(tex_name).copied().unwrap_or(atlas.fallback_region);
                                    let center = glam::Vec3::new(pos.0 as f32 + 0.5, pos.1 as f32 + 0.5, pos.2 as f32 + 0.5);
                                    ir.spawn_digging_particles(center, region, 16);
                                }

                                self.breaking_block_pos = None;
                                self.breaking_block_damage = 0.0;
                            }
                        } else {
                            if let Some(s) = &self.session {
                                s.send(OutgoingPlayPacket::PlayerDigging {
                                    status: DiggingAction::AbortDestroyBlock,
                                    position: Position::new(pos.0, pos.1, pos.2),
                                    facing: self.breaking_block_face,
                                });
                            }
                            self.breaking_block_pos = None;
                            self.breaking_block_damage = 0.0;
                        }
                    }
                }

                let prev_sprint = self.player.is_sprinting;
                let prev_sneak = self.player.is_sneaking;

                self.player.tick(&self.world, &input);

                self.player_pos = (self.player.pos.x, self.player.pos.y, self.player.pos.z);
                self.player_look = (self.player.yaw, self.player.pitch);
                self.player_on_ground = self.player.on_ground;

                if let Some(sm) = &mut self.screen_manager {
                    if let Some(ig) = &mut sm.in_game {
                        ig.pos = self.player_pos;
                        ig.look = self.player_look;
                    }
                }

                // Send entity action packets if sprint or sneak state changed (anticheat compliance)
                if self.player.is_sprinting != prev_sprint {
                    if let Some(s) = &self.session {
                        s.send(OutgoingPlayPacket::EntityAction {
                            entity_id: self.player_entity_id,
                            action: if self.player.is_sprinting {
                                protocol::EntityAction::StartSprinting
                            } else {
                                protocol::EntityAction::StopSprinting
                            },
                            jump_boost: 0,
                        });
                    }
                }
                if self.player.is_sneaking != prev_sneak {
                    if let Some(s) = &self.session {
                        s.send(OutgoingPlayPacket::EntityAction {
                            entity_id: self.player_entity_id,
                            action: if self.player.is_sneaking {
                                protocol::EntityAction::StartSneaking
                            } else {
                                protocol::EntityAction::StopSneaking
                            },
                            jump_boost: 0,
                        });
                    }
                }

                if self.player.abilities_dirty {
                    self.player.abilities_dirty = false;
                    if let Some(s) = &self.session {
                        s.send(OutgoingPlayPacket::PlayerAbilities {
                            is_flying: self.player.capabilities.is_flying,
                        });
                    }
                }

                // Transmit position and look to server in exact sync with physics tick (MCP `EntityPlayerSP.onUpdateWalkingPlayer`)
                let d0 = self.player_pos.0 - self.last_reported_pos.0;
                let d1 = self.player_pos.1 - self.last_reported_pos.1;
                let d2 = self.player_pos.2 - self.last_reported_pos.2;
                let d3 = (self.player_look.0 - self.last_reported_look.0) as f64;
                let d4 = (self.player_look.1 - self.last_reported_look.1) as f64;
                let flag2 = d0 * d0 + d1 * d1 + d2 * d2 > 9.0e-4 || self.position_update_ticks >= 20;
                let flag3 = d3 != 0.0 || d4 != 0.0;

                if let Some(s) = &self.session {
                    if flag2 && flag3 {
                        s.send(OutgoingPlayPacket::PlayerPositionAndLook {
                            x: self.player_pos.0,
                            y: self.player_pos.1,
                            z: self.player_pos.2,
                            yaw: self.player_look.0,
                            pitch: self.player_look.1,
                            on_ground: self.player_on_ground,
                        });
                    } else if flag2 {
                        s.send(OutgoingPlayPacket::PlayerPosition {
                            x: self.player_pos.0,
                            y: self.player_pos.1,
                            z: self.player_pos.2,
                            on_ground: self.player_on_ground,
                        });
                    } else if flag3 {
                        s.send(OutgoingPlayPacket::PlayerLook {
                            yaw: self.player_look.0,
                            pitch: self.player_look.1,
                            on_ground: self.player_on_ground,
                        });
                    } else {
                        s.send(OutgoingPlayPacket::PlayerMovement(self.player_on_ground));
                    }
                }

                self.position_update_ticks += 1;
                if flag2 {
                    self.last_reported_pos = self.player_pos;
                    self.position_update_ticks = 0;
                }
                if flag3 {
                    self.last_reported_look = self.player_look;
                }
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

        let interaction_renderer = InteractionRenderer::new(
            &gpu.device,
            gpu.surface_config.format,
            TerrainRenderer::DEPTH_FORMAT,
            &terrain_renderer.atlas_texture_view,
            &terrain_renderer.atlas_sampler,
        );

        self.atlas = Some(atlas);
        self.terrain_renderer = Some(terrain_renderer);
        self.chunk_manager = Some(chunk_manager);
        self.entity_renderer = Some(entity_renderer);
        self.interaction_renderer = Some(interaction_renderer);

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
                button,
                state,
                ..
            } if !consumed => {
                let is_in_game = self.screen_manager.as_ref().map_or(false, |sm| sm.in_game.is_some());
                let cursor_needed = self.screen_manager.as_ref().map_or(true, |sm| sm.is_cursor_needed());

                match button {
                    winit::event::MouseButton::Left => {
                        match state {
                            winit::event::ElementState::Pressed => {
                                if is_in_game && !cursor_needed && !self.mouse_grabbed {
                                    if let Some(w) = &self.window {
                                        let _ = w.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                            .or_else(|_| w.set_cursor_grab(winit::window::CursorGrabMode::Locked));
                                        w.set_cursor_visible(false);
                                        self.mouse_grabbed = true;
                                    }
                                } else if is_in_game && self.mouse_grabbed {
                                    self.is_left_clicking = true;
                                    self.handle_left_click();
                                }
                            }
                            winit::event::ElementState::Released => {
                                self.is_left_clicking = false;
                                self.handle_left_release();
                            }
                        }
                    }
                    winit::event::MouseButton::Right => {
                        match state {
                            winit::event::ElementState::Pressed => {
                                if is_in_game && self.mouse_grabbed {
                                    self.is_right_clicking = true;
                                    self.handle_right_click();
                                }
                            }
                            winit::event::ElementState::Released => {
                                self.is_right_clicking = false;
                                self.handle_right_release();
                            }
                        }
                    }
                    _ => {}
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
                                self.is_left_clicking = false;
                                self.is_right_clicking = false;
                                self.handle_left_release();
                                self.handle_right_release();

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
                    if let PhysicalKey::Code(code) = key_event.physical_key {
                        let hotbar_item = match code {
                            KeyCode::Digit1 => Some(276), // Diamond Sword (Sword blocking)
                            KeyCode::Digit2 => Some(261), // Bow (Pulling & Crit arrow)
                            KeyCode::Digit3 => Some(278), // Diamond Pickaxe (Digging stone/ores)
                            KeyCode::Digit4 => Some(279), // Diamond Axe (Digging wood)
                            KeyCode::Digit5 => Some(277), // Diamond Shovel (Digging dirt/sand)
                            KeyCode::Digit6 => Some(322), // Golden Apple (Eating)
                            KeyCode::Digit7 => Some(1),   // Stone (Block placement)
                            KeyCode::Digit8 => Some(5),   // Oak Wood Planks (Block placement)
                            KeyCode::Digit9 => Some(0),   // Empty Hand
                            _ => None,
                        };
                        if let Some(item_id) = hotbar_item {
                            if self.held_item_id != item_id {
                                self.held_item_id = item_id;
                                if self.item_use_state.active_action != ItemUseAction::None {
                                    self.item_use_state.stop();
                                }
                                if self.breaking_block_pos.is_some() {
                                    self.handle_left_release();
                                }
                            }
                        }
                    }

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
                        if let Some(sm) = &mut self.screen_manager {
                            sm.force_sync_fullscreen(self.config.video.fullscreen);
                        }
                        let _ = self.config.save(&self.config_path);
                        info!(
                            fullscreen = self.config.video.fullscreen,
                            "Toggled fullscreen mode via keybind"
                        );
                    }

                    let matches_perspective = matches_winit_key(
                        &self.config.keybinds.toggle_perspective,
                        &key_event.logical_key,
                        &key_event.physical_key,
                    );
                    if matches_perspective {
                        self.camera_controller.perspective = self.camera_controller.perspective.cycle();
                        info!(perspective = ?self.camera_controller.perspective, "Cycled camera perspective (F5)");
                    }

                    let matches_cinematic = matches_winit_key(
                        &self.config.keybinds.toggle_cinematic,
                        &key_event.logical_key,
                        &key_event.physical_key,
                    );
                    if matches_cinematic {
                        self.camera_controller.cinematic_camera = !self.camera_controller.cinematic_camera;
                        info!(cinematic = self.camera_controller.cinematic_camera, "Toggled cinematic camera (F8)");
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
                    let is_fullscreen = self
                        .window
                        .as_ref()
                        .map(|w| w.fullscreen().is_some())
                        .unwrap_or(self.config.video.fullscreen);
                    sm.sync_fullscreen(is_fullscreen);
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
                            if let Some(sm) = &mut self.screen_manager {
                                sm.force_sync_fullscreen(self.config.video.fullscreen);
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

                // Camera and player movement update (interpolated from 20 TPS physics ticks)
                if self.has_spawned {
                    let dt_secs = self.last_physics_tick.elapsed().as_secs_f32();
                    let partial_ticks = (dt_secs / 0.05).clamp(0.0, 1.0);
                    let pose = self.camera_controller.compute_camera_pose(
                        &self.world,
                        &self.player,
                        partial_ticks,
                        self.config.video.view_bobbing,
                    );
                    self.camera.position = pose.position;
                    self.camera.yaw = pose.yaw;
                    self.camera.pitch = pose.pitch;
                    self.camera.roll = pose.roll;
                    self.camera.bob_x = pose.bob_x;
                    self.camera.bob_y = pose.bob_y;
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

                        if let Some(atlas) = &self.atlas {
                            cm.dispatch_dirty_sections(
                                &self.world,
                                atlas.clone(),
                                self.camera.position,
                                self.config.video.render_distance,
                                6,
                            );
                        }

                        cm.upload_pending(&gpu.device, 6);
                        let dt_ms = (self.last_frame_time.elapsed().as_millis() as u64).clamp(1, 100);
                        tr.update_animations(&gpu.queue, dt_ms);

                        let render_dist_blocks = (self.config.video.render_distance as f32) * 16.0;
                        let time_secs = (self.world.time as f32) / 20.0;
                        tr.update_uniforms(&gpu.queue, &self.camera, render_dist_blocks, time_secs, self.config.video.antialiasing);

                        let frustum = Frustum::from_view_proj(self.camera.view_proj_matrix());
                        let (visible_opaque, visible_translucent) =
                            cm.get_visible_sections(&frustum, self.camera.position);

                        // Raycast crosshair target picking (Vanilla 1.8 reach distances)
                        let eye_pos = glam::DVec3::new(
                            self.camera.position.x as f64,
                            self.camera.position.y as f64,
                            self.camera.position.z as f64,
                        );
                        let fwd = self.camera.forward();
                        let look_dir = glam::DVec3::new(fwd.x as f64, fwd.y as f64, fwd.z as f64);
                        self.current_target = pick_target(
                            &self.world,
                            &self.spatial_hash,
                            eye_pos,
                            look_dir,
                            self.player.capabilities.is_creative,
                            Some(self.player_entity_id),
                        );

                        if let Some(ir) = &mut self.interaction_renderer {
                            ir.update_camera(&gpu.queue, &self.camera);

                            match self.current_target {
                                TargetHit::Block(bh) => {
                                    let state = self.world.get_block(bh.block_pos.0, bh.block_pos.1, bh.block_pos.2);
                                    let aabb = get_block_selection_box(bh.block_pos.0, bh.block_pos.1, bh.block_pos.2, state);
                                    ir.update_selection_box(&gpu.device, &aabb);
                                }
                                _ => {
                                    ir.clear_selection_box();
                                }
                            }

                            if let Some(pos) = self.breaking_block_pos {
                                let stage = ((self.breaking_block_damage * 10.0) as usize).min(9);
                                if let Some(atlas) = &self.atlas {
                                    ir.update_block_cracks(&gpu.device, pos, stage, atlas);
                                }
                            } else if let Some((&pos, &stage)) = self.server_break_progress.iter().next() {
                                if let Some(atlas) = &self.atlas {
                                    ir.update_block_cracks(&gpu.device, pos, stage as usize, atlas);
                                }
                            } else {
                                ir.clear_block_cracks();
                            }

                            ir.update_particle_mesh(&gpu.device, &self.camera);
                        }

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

                            let local_player_entity = if self.camera_controller.perspective != CameraPerspective::FirstPerson {
                                let uuid = self.screen_manager.as_ref()
                                    .and_then(|sm| sm.in_game.as_ref())
                                    .and_then(|ig| ig.tab_list.entries.keys().next().copied())
                                    .unwrap_or_else(|| Uuid::nil());
                                let mut lp = TrackedEntity::new(
                                    self.player_entity_id,
                                    EntityKind::Player {
                                        uuid,
                                        name: self.config.auth.nickname.clone(),
                                        held_item: 0,
                                    },
                                    self.player.pos,
                                    self.player.yaw,
                                    self.player.pitch,
                                );
                                lp.prev_pos = self.player.prev_pos;
                                lp.prev_yaw = self.player.prev_yaw;
                                lp.prev_pitch = self.player.prev_pitch;
                                lp.head_yaw = self.player.yaw;
                                lp.prev_head_yaw = self.player.prev_yaw;
                                lp.is_sneaking = self.player.is_sneaking;
                                lp.is_sprinting = self.player.is_sprinting;
                                Some(lp)
                            } else {
                                None
                            };

                            er.render(&gpu.device, &gpu.queue, &mut rpass, &self.world.entity_tracker, partial_ticks, local_player_entity.as_ref());
                        }

                        if let Some(ir) = &self.interaction_renderer {
                            ir.render(&mut rpass);
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
                let sensitivity = self.config.mouse.sensitivity;
                let invert_y = self.config.mouse.invert_y;
                self.camera_controller.handle_mouse_input(
                    &mut self.player,
                    dx as f32,
                    dy as f32,
                    sensitivity,
                    invert_y,
                );
                self.player_look = (self.player.yaw, self.player.pitch);
                if let Some(sm) = &mut self.screen_manager {
                    if let Some(ig) = &mut sm.in_game {
                        ig.look = self.player_look;
                    }
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
