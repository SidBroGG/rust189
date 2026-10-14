use crate::error::ProtocolError;
use crate::packets::handshake::{HandshakePacket, NextState};
use crate::packets::login::{
    DisconnectLoginPacket, LoginStartPacket, LoginSuccessPacket, SetCompressionPacket,
};
use crate::packets::status::{PingPacket, ServerStatus, StatusRequestPacket, StatusResponsePacket};
use crate::transport::stream::AsyncPacketStream;
use bytes::BytesMut;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

/// Pings a Minecraft 1.8.9 server using the Status protocol.
/// Returns the parsed `ServerStatus` and the round-trip latency duration.
pub async fn ping_server(host: &str, port: u16) -> Result<(ServerStatus, Duration), ProtocolError> {
    let tcp_stream = TcpStream::connect((host, port)).await?;
    let mut stream = AsyncPacketStream::new(tcp_stream);

    // 1. Handshake (state = Status)
    let handshake = HandshakePacket::new_v47(host, port, NextState::Status);
    let mut buf = BytesMut::new();
    handshake.encode(&mut buf)?;
    stream.write_packet(HandshakePacket::PACKET_ID, &buf).await?;

    // 2. Status Request
    let request = StatusRequestPacket;
    let mut req_buf = BytesMut::new();
    request.encode(&mut req_buf)?;
    stream
        .write_packet(StatusRequestPacket::PACKET_ID, &req_buf)
        .await?;

    // 3. Status Response
    let response_packet = stream.read_packet().await?;
    if response_packet.id != StatusResponsePacket::PACKET_ID {
        return Err(ProtocolError::UnexpectedPacket {
            expected: "StatusResponse (0x00)",
            actual: response_packet.id,
        });
    }

    let mut resp_payload = response_packet.payload;
    let status_response = StatusResponsePacket::decode(&mut resp_payload)?;
    let server_status = status_response.parse_status()?;

    // 4. Ping and measure round-trip latency
    let start = Instant::now();
    let now_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let ping = PingPacket::new(now_millis);
    let mut ping_buf = BytesMut::new();
    ping.encode(&mut ping_buf)?;
    stream.write_packet(PingPacket::PACKET_ID, &ping_buf).await?;

    // 5. Pong Response
    let pong_packet = stream.read_packet().await?;
    if pong_packet.id != PingPacket::PACKET_ID {
        return Err(ProtocolError::UnexpectedPacket {
            expected: "Pong (0x01)",
            actual: pong_packet.id,
        });
    }

    let latency = start.elapsed();
    Ok((server_status, latency))
}

/// Connects to a Minecraft 1.8.9 server in offline mode and completes the Login phase.
/// Automatically handles `SetCompression` if enabled by the server.
/// Returns the active `AsyncPacketStream` (transitioned to Play phase) and the `LoginSuccessPacket`.
pub async fn connect_offline(
    host: &str,
    port: u16,
    username: &str,
) -> Result<(AsyncPacketStream<TcpStream>, LoginSuccessPacket), ProtocolError> {
    let tcp_stream = TcpStream::connect((host, port)).await?;
    let mut stream = AsyncPacketStream::new(tcp_stream);

    // 1. Handshake (state = Login)
    let handshake = HandshakePacket::new_v47(host, port, NextState::Login);
    let mut hs_buf = BytesMut::new();
    handshake.encode(&mut hs_buf)?;
    stream.write_packet(HandshakePacket::PACKET_ID, &hs_buf).await?;

    // 2. Login Start
    let login_start = LoginStartPacket::new(username);
    let mut ls_buf = BytesMut::new();
    login_start.encode(&mut ls_buf)?;
    stream
        .write_packet(LoginStartPacket::PACKET_ID, &ls_buf)
        .await?;

    // 3. Login loop: handle SetCompression (0x03), Disconnect (0x00), or Success (0x02)
    loop {
        let packet = stream.read_packet().await?;
        match packet.id {
            DisconnectLoginPacket::PACKET_ID => {
                let mut payload = packet.payload;
                let disconnect = DisconnectLoginPacket::decode(&mut payload)?;
                return Err(ProtocolError::Disconnected(disconnect.reason));
            }
            SetCompressionPacket::PACKET_ID => {
                let mut payload = packet.payload;
                let set_compression = SetCompressionPacket::decode(&mut payload)?;
                tracing::info!(
                    threshold = set_compression.threshold,
                    "Server enabled packet compression"
                );
                stream.set_compression(set_compression.threshold);
            }
            LoginSuccessPacket::PACKET_ID => {
                let mut payload = packet.payload;
                let success = LoginSuccessPacket::decode(&mut payload)?;
                tracing::info!(
                    uuid = %success.uuid,
                    username = %success.username,
                    "Login successful! Transitioning to Play state"
                );
                return Ok((stream, success));
            }
            other => {
                return Err(ProtocolError::UnexpectedPacket {
                    expected: "LoginSuccess (0x02), SetCompression (0x03), or Disconnect (0x00)",
                    actual: other,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn test_mock_status_ping_flow() {
        let (client_io, server_io) = duplex(4096);
        let mut client_stream = AsyncPacketStream::new(client_io);
        let mut server_stream = AsyncPacketStream::new(server_io);

        let server_task = tokio::spawn(async move {
            // Read Handshake
            let hs = server_stream.read_packet().await.unwrap();
            assert_eq!(hs.id, HandshakePacket::PACKET_ID);

            // Read Request
            let req = server_stream.read_packet().await.unwrap();
            assert_eq!(req.id, StatusRequestPacket::PACKET_ID);

            // Send Response
            let json = r#"{"version":{"name":"1.8.9","protocol":47},"players":{"max":100,"online":5},"description":"Mock Test Server"}"#;
            let resp = StatusResponsePacket::new(json);
            let mut buf = BytesMut::new();
            resp.encode(&mut buf).unwrap();
            server_stream
                .write_packet(StatusResponsePacket::PACKET_ID, &buf)
                .await
                .unwrap();

            // Read Ping
            let ping = server_stream.read_packet().await.unwrap();
            assert_eq!(ping.id, PingPacket::PACKET_ID);

            // Send Pong
            server_stream
                .write_packet(PingPacket::PACKET_ID, &ping.payload)
                .await
                .unwrap();
        });

        // Client flow
        let hs = HandshakePacket::new_v47("localhost", 25565, NextState::Status);
        let mut hs_buf = BytesMut::new();
        hs.encode(&mut hs_buf).unwrap();
        client_stream
            .write_packet(HandshakePacket::PACKET_ID, &hs_buf)
            .await
            .unwrap();

        let req = StatusRequestPacket;
        let mut req_buf = BytesMut::new();
        req.encode(&mut req_buf).unwrap();
        client_stream
            .write_packet(StatusRequestPacket::PACKET_ID, &req_buf)
            .await
            .unwrap();

        let resp_pkt = client_stream.read_packet().await.unwrap();
        assert_eq!(resp_pkt.id, StatusResponsePacket::PACKET_ID);
        let mut resp_payload = resp_pkt.payload;
        let status = StatusResponsePacket::decode(&mut resp_payload)
            .unwrap()
            .parse_status()
            .unwrap();
        assert_eq!(status.players.online, 5);

        let ping = PingPacket::new(123456789);
        let mut ping_buf = BytesMut::new();
        ping.encode(&mut ping_buf).unwrap();
        client_stream
            .write_packet(PingPacket::PACKET_ID, &ping_buf)
            .await
            .unwrap();

        let pong_pkt = client_stream.read_packet().await.unwrap();
        assert_eq!(pong_pkt.id, PingPacket::PACKET_ID);
        let mut pong_payload = pong_pkt.payload;
        let pong = PingPacket::decode(&mut pong_payload).unwrap();
        assert_eq!(pong.time, 123456789);

        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_mock_login_with_compression_flow() {
        let (client_io, server_io) = duplex(4096);
        let mut client_stream = AsyncPacketStream::new(client_io);
        let mut server_stream = AsyncPacketStream::new(server_io);

        let server_task = tokio::spawn(async move {
            // Read Handshake
            let hs = server_stream.read_packet().await.unwrap();
            assert_eq!(hs.id, HandshakePacket::PACKET_ID);

            // Read LoginStart
            let ls = server_stream.read_packet().await.unwrap();
            assert_eq!(ls.id, LoginStartPacket::PACKET_ID);

            // Send SetCompression (threshold = 256)
            let comp = SetCompressionPacket::new(256);
            let mut comp_buf = BytesMut::new();
            comp.encode(&mut comp_buf).unwrap();
            server_stream
                .write_packet(SetCompressionPacket::PACKET_ID, &comp_buf)
                .await
                .unwrap();
            server_stream.set_compression(256);

            // Send LoginSuccess
            let succ = LoginSuccessPacket::new(
                "5627dd98-e6be-3c21-b8a8-e92344183641",
                "Alex",
            );
            let mut succ_buf = BytesMut::new();
            succ.encode(&mut succ_buf).unwrap();
            server_stream
                .write_packet(LoginSuccessPacket::PACKET_ID, &succ_buf)
                .await
                .unwrap();
        });

        // Client flow
        let hs = HandshakePacket::new_v47("localhost", 25565, NextState::Login);
        let mut hs_buf = BytesMut::new();
        hs.encode(&mut hs_buf).unwrap();
        client_stream
            .write_packet(HandshakePacket::PACKET_ID, &hs_buf)
            .await
            .unwrap();

        let ls = LoginStartPacket::new("Alex");
        let mut ls_buf = BytesMut::new();
        ls.encode(&mut ls_buf).unwrap();
        client_stream
            .write_packet(LoginStartPacket::PACKET_ID, &ls_buf)
            .await
            .unwrap();

        // 1st incoming: SetCompression
        let p1 = client_stream.read_packet().await.unwrap();
        assert_eq!(p1.id, SetCompressionPacket::PACKET_ID);
        let mut p1_payload = p1.payload;
        let comp = SetCompressionPacket::decode(&mut p1_payload).unwrap();
        assert_eq!(comp.threshold, 256);
        client_stream.set_compression(256);

        // 2nd incoming: LoginSuccess (already under compression!)
        let p2 = client_stream.read_packet().await.unwrap();
        assert_eq!(p2.id, LoginSuccessPacket::PACKET_ID);
        let mut p2_payload = p2.payload;
        let succ = LoginSuccessPacket::decode(&mut p2_payload).unwrap();
        assert_eq!(succ.username, "Alex");

        server_task.await.unwrap();
    }
}
