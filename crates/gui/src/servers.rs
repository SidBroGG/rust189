use anyhow::{Context, Result};
use protocol::codec::nbt::NbtTag;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

/// Represents a single server entry in the multiplayer server browser.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerEntry {
    pub name: String,
    pub ip: String,
    pub icon: Option<String>,
    pub motd: String,
    pub players_online: i32,
    pub players_max: i32,
    pub ping_ms: Option<u64>,
    pub version_name: String,
    pub protocol_version: i32,
    #[serde(skip)]
    pub is_pinging: bool,
    #[serde(skip)]
    pub error: Option<String>,
}

impl ServerEntry {
    pub fn new(name: impl Into<String>, ip: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ip: ip.into(),
            icon: None,
            motd: "Pinging...".to_string(),
            players_online: 0,
            players_max: 0,
            ping_ms: None,
            version_name: "1.8.9".to_string(),
            protocol_version: 47,
            is_pinging: false,
            error: None,
        }
    }
}

/// Result of querying a Minecraft server status.
#[derive(Debug, Clone)]
pub struct ServerPingResult {
    pub motd: String,
    pub players_online: i32,
    pub players_max: i32,
    pub ping_ms: u64,
    pub version_name: String,
    pub protocol_version: i32,
    pub icon: Option<String>,
}

/// Queries server status and latency asynchronously using protocol 47.
pub async fn ping_server_async(target: &str) -> Result<ServerPingResult, String> {
    let (host, port) = if let Some((h, p)) = target.split_once(':') {
        (h, p.parse::<u16>().unwrap_or(25565))
    } else {
        (target, 25565)
    };

    match protocol::ping_server(host, port).await {
        Ok((status, latency)) => Ok(ServerPingResult {
            motd: status.description.to_plain_text(),
            players_online: status.players.online,
            players_max: status.players.max,
            ping_ms: latency.as_millis() as u64,
            version_name: status.version.name,
            protocol_version: status.version.protocol,
            icon: status.favicon,
        }),
        Err(e) => Err(format!("Can't connect to server ({})", e)),
    }
}

/// Loads server list from Minecraft 1.8.9 `servers.dat` (uncompressed NBT).
pub fn load_servers_dat<P: AsRef<Path>>(path: P) -> Vec<ServerEntry> {
    let path = path.as_ref();
    if !path.exists() {
        return default_servers();
    }

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to read servers.dat, using defaults");
            return default_servers();
        }
    };

    let mut cursor = Cursor::new(bytes);
    let (_name, root) = match protocol::codec::nbt::read_named_compound(&mut cursor) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to parse servers.dat NBT, using defaults");
            return default_servers();
        }
    };

    let mut servers = Vec::new();
    if let Some(map) = root.as_compound() {
        if let Some(NbtTag::List { elements, .. }) = map.get("servers") {
            for element in elements {
                if let Some(entry_map) = element.as_compound() {
                    let name = entry_map
                        .get("name")
                        .and_then(|t| t.as_string())
                        .unwrap_or("Server")
                        .to_string();
                    let ip = entry_map
                        .get("ip")
                        .and_then(|t| t.as_string())
                        .unwrap_or("localhost")
                        .to_string();
                    let icon = entry_map
                        .get("icon")
                        .and_then(|t| t.as_string())
                        .map(|s| s.to_string());

                    let mut entry = ServerEntry::new(name, ip);
                    entry.icon = icon;
                    servers.push(entry);
                }
            }
        }
    }

    if servers.is_empty() {
        default_servers()
    } else {
        servers
    }
}

/// Saves server list to Minecraft 1.8.9 `servers.dat` (uncompressed NBT).
pub fn save_servers_dat<P: AsRef<Path>>(servers: &[ServerEntry], path: P) -> Result<()> {
    let path = path.as_ref();
    let mut elements = Vec::new();

    for server in servers {
        let mut entry_map = HashMap::new();
        entry_map.insert("name".to_string(), NbtTag::String(server.name.clone()));
        entry_map.insert("ip".to_string(), NbtTag::String(server.ip.clone()));
        if let Some(ref icon) = server.icon {
            entry_map.insert("icon".to_string(), NbtTag::String(icon.clone()));
        }
        elements.push(NbtTag::Compound(entry_map));
    }

    let mut root_map = HashMap::new();
    root_map.insert(
        "servers".to_string(),
        NbtTag::List {
            element_type: 10, // TAG_Compound
            elements,
        },
    );
    let root = NbtTag::Compound(root_map);

    let mut buf = bytes::BytesMut::new();
    protocol::codec::nbt::write_named_compound("", &root, &mut buf)
        .context("Failed to serialize servers.dat NBT")?;

    // Atomic write
    let tmp_path = path.with_extension("dat_tmp");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&tmp_path, &buf)?;
    std::fs::rename(&tmp_path, path)?;

    tracing::info!(count = servers.len(), path = ?path, "Saved servers.dat");
    Ok(())
}

/// Provides default server entries if no `servers.dat` is present.
pub fn default_servers() -> Vec<ServerEntry> {
    vec![
        ServerEntry::new("Local Server", "127.0.0.1:25565"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_servers_dat_nbt_roundtrip() {
        let servers = vec![
            ServerEntry::new("Primary Server", "127.0.0.1:25565"),
            ServerEntry::new("Backup Server", "localhost:25566"),
        ];

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_servers.dat");

        save_servers_dat(&servers, &path).expect("Failed to save servers.dat");
        let loaded = load_servers_dat(&path);

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "Primary Server");
        assert_eq!(loaded[0].ip, "127.0.0.1:25565");
        assert_eq!(loaded[1].name, "Backup Server");
        assert_eq!(loaded[1].ip, "localhost:25566");

        let _ = std::fs::remove_file(path);
    }
}
