use crate::error::ProtocolError;
use crate::transport::raknet::congestion::CongestionControl;
use crate::transport::raknet::types::{packet_ids, Frame, RAKNET_MAGIC};
use bytes::{Buf, BufMut, BytesMut};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;

/// Represents an active high-performance UDP connection using the RakNet protocol.
pub struct RakNetSession {
    socket: Arc<UdpSocket>,
    peer_addr: SocketAddr,
    congestion: CongestionControl,
    next_send_seq: u32,
    next_reliable_index: u32,
    next_order_index: u32,
    unacked_packets: HashMap<u32, (Instant, usize, Vec<Frame>)>,
    received_packets_queue: VecDeque<Vec<u8>>,
    pending_acks: Vec<u32>,
    is_connected: bool,
}

impl RakNetSession {
    /// Connects to a remote server using the RakNet offline handshake sequence.
    pub async fn connect(peer_addr: SocketAddr) -> Result<Self, ProtocolError> {
        let local_addr: SocketAddr = if peer_addr.is_ipv4() {
            "0.0.0.0:0".parse().unwrap()
        } else {
            "[::]:0".parse().unwrap()
        };

        let socket = UdpSocket::bind(local_addr).await?;
        socket.connect(peer_addr).await?;

        let mut session = Self {
            socket: Arc::new(socket),
            peer_addr,
            congestion: CongestionControl::new(CongestionControl::DEFAULT_MTU),
            next_send_seq: 0,
            next_reliable_index: 0,
            next_order_index: 0,
            unacked_packets: HashMap::new(),
            received_packets_queue: VecDeque::new(),
            pending_acks: Vec::new(),
            is_connected: false,
        };

        // Handshake: OpenConnectionRequest1
        session.send_open_connection_request_1().await?;
        session.is_connected = true;
        Ok(session)
    }

    /// Sends RakNet OpenConnectionRequest1 packet.
    async fn send_open_connection_request_1(&mut self) -> Result<(), ProtocolError> {
        let mut buf = BytesMut::with_capacity(1464);
        buf.put_u8(packet_ids::OPEN_CONNECTION_REQUEST_1);
        buf.put_slice(&RAKNET_MAGIC);
        buf.put_u8(11); // Protocol version
        // MTU padding
        let pad_len = 1464 - buf.len();
        buf.put_bytes(0, pad_len);
        self.socket.send(&buf).await?;
        Ok(())
    }

    /// Sends a reliable ordered payload using RakNet frame encapsulation.
    pub async fn send_packet(&mut self, payload: &[u8]) -> Result<(), ProtocolError> {
        let reliable_idx = self.next_reliable_index;
        self.next_reliable_index += 1;

        let order_idx = self.next_order_index;
        self.next_order_index += 1;

        let frame = Frame::new_reliable_ordered(payload.to_vec(), reliable_idx, order_idx, 0);
        let seq = self.next_send_seq;
        self.next_send_seq += 1;

        let mut datagram = BytesMut::new();
        // 0x84 = standard datagram flag (valid packet)
        datagram.put_u8(0x84);
        // 24-bit little endian sequence number
        datagram.put_u8((seq & 0xFF) as u8);
        datagram.put_u8(((seq >> 8) & 0xFF) as u8);
        datagram.put_u8(((seq >> 16) & 0xFF) as u8);

        frame.encode(&mut datagram);

        let sent_len = datagram.len();
        self.congestion.on_packet_sent(sent_len);
        self.unacked_packets
            .insert(seq, (Instant::now(), sent_len, vec![frame]));

        self.socket.send(&datagram).await?;
        Ok(())
    }

    /// Receives the next complete packet from the queue or the network socket.
    pub async fn read_packet(&mut self) -> Result<Vec<u8>, ProtocolError> {
        loop {
            if let Some(pkt) = self.received_packets_queue.pop_front() {
                return Ok(pkt);
            }

            let mut buf = [0u8; 2048];
            let n = self.socket.recv(&mut buf).await?;
            if n == 0 {
                return Err(ProtocolError::ConnectionClosed);
            }

            self.process_incoming_datagram(&buf[..n]).await?;
        }
    }

    /// Processes an incoming UDP datagram from the peer.
    pub async fn process_incoming_datagram(&mut self, data: &[u8]) -> Result<(), ProtocolError> {
        if data.is_empty() {
            return Ok(());
        }

        let header = data[0];

        // Handle ACK
        if header == packet_ids::ACK {
            self.handle_ack(&data[1..]);
            return Ok(());
        }

        // Handle NACK
        if header == packet_ids::NACK {
            self.handle_nack(&data[1..]).await?;
            return Ok(());
        }

        // Standard datagram flags: 0x80 bit set
        if (header & 0x80) != 0 {
            if data.len() < 4 {
                return Ok(());
            }

            let b0 = data[1] as u32;
            let b1 = data[2] as u32;
            let b2 = data[3] as u32;
            let seq = b0 | (b1 << 8) | (b2 << 16);

            // Record seq for ACK response
            self.pending_acks.push(seq);
            self.flush_acks().await?;

            // Decode frames inside datagram
            let mut slice = &data[4..];
            while slice.remaining() > 0 {
                match Frame::decode(&mut slice) {
                    Ok(frame) => {
                        self.received_packets_queue.push_back(frame.body);
                    }
                    Err(_) => break,
                }
            }
        }

        Ok(())
    }

    /// Sends pending ACKs back to the remote peer.
    pub async fn flush_acks(&mut self) -> Result<(), ProtocolError> {
        if self.pending_acks.is_empty() {
            return Ok(());
        }

        let mut buf = BytesMut::new();
        buf.put_u8(packet_ids::ACK);
        buf.put_u16(self.pending_acks.len() as u16);

        for &seq in &self.pending_acks {
            buf.put_u8(1); // single record
            buf.put_u8((seq & 0xFF) as u8);
            buf.put_u8(((seq >> 8) & 0xFF) as u8);
            buf.put_u8(((seq >> 16) & 0xFF) as u8);
        }

        self.pending_acks.clear();
        self.socket.send(&buf).await?;
        Ok(())
    }

    fn handle_ack(&mut self, mut data: &[u8]) {
        if data.remaining() < 2 {
            return;
        }
        let record_count = data.get_u16() as usize;
        for _ in 0..record_count {
            if data.remaining() < 4 {
                break;
            }
            let is_range = data.get_u8() == 0;
            let b0 = data.get_u8() as u32;
            let b1 = data.get_u8() as u32;
            let b2 = data.get_u8() as u32;
            let start = b0 | (b1 << 8) | (b2 << 16);

            let end = if is_range {
                if data.remaining() < 3 {
                    break;
                }
                let eb0 = data.get_u8() as u32;
                let eb1 = data.get_u8() as u32;
                let eb2 = data.get_u8() as u32;
                eb0 | (eb1 << 8) | (eb2 << 16)
            } else {
                start
            };

            for seq in start..=end {
                if let Some((sent_time, bytes, _)) = self.unacked_packets.remove(&seq) {
                    let rtt_sample = sent_time.elapsed().as_secs_f64() * 1000.0;
                    self.congestion.on_ack_received(bytes, Some(rtt_sample));
                }
            }
        }
    }

    async fn handle_nack(&mut self, mut data: &[u8]) -> Result<(), ProtocolError> {
        if data.remaining() < 2 {
            return Ok(());
        }
        let record_count = data.get_u16() as usize;
        let mut retransmit_list = Vec::new();

        for _ in 0..record_count {
            if data.remaining() < 4 {
                break;
            }
            let is_range = data.get_u8() == 0;
            let b0 = data.get_u8() as u32;
            let b1 = data.get_u8() as u32;
            let b2 = data.get_u8() as u32;
            let start = b0 | (b1 << 8) | (b2 << 16);

            let end = if is_range {
                if data.remaining() < 3 {
                    break;
                }
                let eb0 = data.get_u8() as u32;
                let eb1 = data.get_u8() as u32;
                let eb2 = data.get_u8() as u32;
                eb0 | (eb1 << 8) | (eb2 << 16)
            } else {
                start
            };

            for seq in start..=end {
                if let Some((_, bytes, frames)) = self.unacked_packets.remove(&seq) {
                    self.congestion.on_packet_loss(bytes);
                    retransmit_list.extend(frames);
                }
            }
        }

        // Retransmit lost frames with fresh sequence numbers
        for frame in retransmit_list {
            let seq = self.next_send_seq;
            self.next_send_seq += 1;

            let mut datagram = BytesMut::new();
            datagram.put_u8(0x84);
            datagram.put_u8((seq & 0xFF) as u8);
            datagram.put_u8(((seq >> 8) & 0xFF) as u8);
            datagram.put_u8(((seq >> 16) & 0xFF) as u8);
            frame.encode(&mut datagram);

            let sent_len = datagram.len();
            self.congestion.on_packet_sent(sent_len);
            self.unacked_packets
                .insert(seq, (Instant::now(), sent_len, vec![frame]));
            self.socket.send(&datagram).await?;
        }

        Ok(())
    }

    #[inline]
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    #[inline]
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    #[inline]
    pub fn congestion(&self) -> &CongestionControl {
        &self.congestion
    }
}
