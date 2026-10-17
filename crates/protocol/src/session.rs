use crate::error::ProtocolError;
use crate::packets::play::*;
use crate::transport::stream::AsyncPacketStream;
use bytes::BytesMut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Strongly typed outgoing packet from client to server.
#[derive(Debug, Clone)]
pub enum OutgoingPlayPacket {
    KeepAlive(i32),
    Chat(String),
    PlayerMovement(bool),
    PlayerPosition {
        x: f64,
        y: f64,
        z: f64,
        on_ground: bool,
    },
    PlayerLook {
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
    PlayerPositionAndLook {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
    ClientSettings {
        locale: String,
        view_distance: i8,
        chat_flags: i8,
        chat_colors: bool,
        skin_parts: u8,
    },
    EntityAction {
        entity_id: i32,
        action: EntityAction,
        jump_boost: i32,
    },
}

impl OutgoingPlayPacket {
    pub fn encode(&self, buf: &mut BytesMut) -> Result<i32, ProtocolError> {
        match self {
            Self::KeepAlive(id) => {
                let pkt = ServerboundKeepAlivePacket::new(*id);
                pkt.encode(buf)?;
                Ok(ServerboundKeepAlivePacket::PACKET_ID)
            }
            Self::Chat(msg) => {
                let pkt = ServerboundChatPacket::new(msg.as_str());
                pkt.encode(buf)?;
                Ok(ServerboundChatPacket::PACKET_ID)
            }
            Self::PlayerMovement(on_ground) => {
                let pkt = ServerboundPlayerMovementPacket::new(*on_ground);
                pkt.encode(buf)?;
                Ok(ServerboundPlayerMovementPacket::PACKET_ID)
            }
            Self::PlayerPosition {
                x,
                y,
                z,
                on_ground,
            } => {
                let pkt = ServerboundPlayerPositionPacket::new(*x, *y, *z, *on_ground);
                pkt.encode(buf)?;
                Ok(ServerboundPlayerPositionPacket::PACKET_ID)
            }
            Self::PlayerLook {
                yaw,
                pitch,
                on_ground,
            } => {
                let pkt = ServerboundPlayerLookPacket::new(*yaw, *pitch, *on_ground);
                pkt.encode(buf)?;
                Ok(ServerboundPlayerLookPacket::PACKET_ID)
            }
            Self::PlayerPositionAndLook {
                x,
                y,
                z,
                yaw,
                pitch,
                on_ground,
            } => {
                let pkt = ServerboundPlayerPositionAndLookPacket::new(
                    *x, *y, *z, *yaw, *pitch, *on_ground,
                );
                pkt.encode(buf)?;
                Ok(ServerboundPlayerPositionAndLookPacket::PACKET_ID)
            }
            Self::ClientSettings {
                locale,
                view_distance,
                chat_flags,
                chat_colors,
                skin_parts,
            } => {
                let pkt = ServerboundClientSettingsPacket::new(
                    locale.as_str(),
                    *view_distance,
                    *chat_flags,
                    *chat_colors,
                    *skin_parts,
                );
                pkt.encode(buf)?;
                Ok(ServerboundClientSettingsPacket::PACKET_ID)
            }
            Self::EntityAction {
                entity_id,
                action,
                jump_boost,
            } => {
                let pkt = ServerboundEntityActionPacket::new(*entity_id, *action, *jump_boost);
                pkt.encode(buf)?;
                Ok(ServerboundEntityActionPacket::PACKET_ID)
            }
        }
    }
}

/// Statistics and telemetry from the active play session.
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    pub latency_ms: f64,
    pub total_bytes_received: u64,
    pub total_bytes_sent: u64,
}

/// Handles the full duplex Play phase network stream via lock-free Tokio channels.
pub struct PlaySessionHandle {
    pub incoming_rx: UnboundedReceiver<ClientboundPlayPacket>,
    pub outgoing_tx: UnboundedSender<OutgoingPlayPacket>,
    pub is_connected: Arc<AtomicBool>,
}

impl PlaySessionHandle {
    /// Launches the background task managing TCP stream reading and writing.
    pub fn start(mut stream: AsyncPacketStream<TcpStream>) -> Self {
        let (incoming_tx, incoming_rx) = unbounded_channel::<ClientboundPlayPacket>();
        let (outgoing_tx, mut outgoing_rx) = unbounded_channel::<OutgoingPlayPacket>();
        let is_connected = Arc::new(AtomicBool::new(true));
        let connected_flag = Arc::clone(&is_connected);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Outgoing packets to send
                    outgoing = outgoing_rx.recv() => {
                        match outgoing {
                            Some(pkt) => {
                                let mut buf = BytesMut::new();
                                match pkt.encode(&mut buf) {
                                    Ok(pkt_id) => {
                                        if let Err(e) = stream.write_packet(pkt_id, &buf).await {
                                            tracing::error!(error = %e, "Failed to write outgoing Play packet");
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(error = %e, "Failed to encode outgoing Play packet");
                                    }
                                }
                            }
                            None => {
                                // Channel closed, terminate session
                                break;
                            }
                        }
                    }

                    // Incoming packets from server
                    incoming = stream.read_packet() => {
                        match incoming {
                            Ok(raw_packet) => {
                                let mut payload = raw_packet.payload;
                                match ClientboundPlayPacket::decode(raw_packet.id, &mut payload) {
                                    Ok(ClientboundPlayPacket::KeepAlive(ka)) => {
                                        // Auto respond to KeepAlive immediately
                                        let mut resp_buf = BytesMut::new();
                                        let resp = ServerboundKeepAlivePacket::new(ka.keep_alive_id);
                                        if resp.encode(&mut resp_buf).is_ok() {
                                            let _ = stream.write_packet(ServerboundKeepAlivePacket::PACKET_ID, &resp_buf).await;
                                        }

                                        let _ = incoming_tx.send(ClientboundPlayPacket::KeepAlive(ka));
                                    }
                                    Ok(ClientboundPlayPacket::Disconnect(dc)) => {
                                        tracing::warn!(reason = %dc.reason, "Server sent disconnect in Play state");
                                        let _ = incoming_tx.send(ClientboundPlayPacket::Disconnect(dc));
                                        break;
                                    }
                                    Ok(parsed) => {
                                        if incoming_tx.send(parsed).is_err() {
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            id = raw_packet.id,
                                            error = %e,
                                            "Failed to decode clientbound play packet"
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(error = %e, "Play stream connection closed by peer");
                                break;
                            }
                        }
                    }
                }
            }

            connected_flag.store(false, Ordering::SeqCst);
            tracing::info!("Play session ended cleanly");
        });

        Self {
            incoming_rx,
            outgoing_tx,
            is_connected,
        }
    }

    #[inline]
    pub fn is_alive(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }

    pub fn send(&self, packet: OutgoingPlayPacket) {
        let _ = self.outgoing_tx.send(packet);
    }
}
