use crate::codec::string::{read_string, write_string};
use crate::error::ProtocolError;
use bytes::{Buf, BufMut};
use serde::{Deserialize, Deserializer, Serialize};

/// Client Status Request packet (Packet ID: 0x00 in Status state).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StatusRequestPacket;

impl StatusRequestPacket {
    pub const PACKET_ID: i32 = 0x00;

    pub fn encode(&self, _buf: &mut impl BufMut) -> Result<(), ProtocolError> {
        Ok(())
    }

    pub fn decode(_buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        Ok(Self)
    }
}

/// Client Ping packet / Server Pong packet (Packet ID: 0x01 in Status state).
#[derive(Debug, Clone, PartialEq)]
pub struct PingPacket {
    pub time: i64,
}

impl PingPacket {
    pub const PACKET_ID: i32 = 0x01;

    pub fn new(time: i64) -> Self {
        Self { time }
    }

    pub fn encode(&self, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
        buf.put_i64(self.time);
        Ok(())
    }

    pub fn decode(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        if buf.remaining() < 8 {
            return Err(ProtocolError::UnexpectedEof);
        }
        Ok(Self {
            time: buf.get_i64(),
        })
    }
}

/// Server Status Response packet (Packet ID: 0x00 in Status state).
#[derive(Debug, Clone, PartialEq)]
pub struct StatusResponsePacket {
    pub json_response: String,
}

impl StatusResponsePacket {
    pub const PACKET_ID: i32 = 0x00;

    pub fn new(json_response: impl Into<String>) -> Self {
        Self {
            json_response: json_response.into(),
        }
    }

    pub fn encode(&self, buf: &mut impl BufMut) -> Result<(), ProtocolError> {
        write_string(&self.json_response, buf)
    }

    pub fn decode(buf: &mut impl Buf) -> Result<Self, ProtocolError> {
        let json_response = read_string(buf, 32767)?;
        Ok(Self { json_response })
    }

    /// Parses the JSON response string into a structured `ServerStatus`.
    pub fn parse_status(&self) -> Result<ServerStatus, ProtocolError> {
        serde_json::from_str(&self.json_response).map_err(ProtocolError::Json)
    }
}

/// Structured Minecraft 1.8.9 Server Status (Ping Response).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerStatus {
    pub version: VersionInfo,
    pub players: PlayersInfo,
    pub description: DescriptionInfo,
    #[serde(default)]
    pub favicon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionInfo {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayersInfo {
    pub max: i32,
    pub online: i32,
    #[serde(default)]
    pub sample: Option<Vec<PlayerSample>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerSample {
    pub name: String,
    pub id: String,
}

/// Description (MOTD) that can be either a raw string or a chat JSON component.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum DescriptionInfo {
    Plain(String),
    ChatObject(ChatObject),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatObject {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub extra: Option<Vec<ChatExtra>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ChatExtra {
    Plain(String),
    Object(ChatObject),
}

impl<'de> Deserialize<'de> for DescriptionInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) => Ok(DescriptionInfo::Plain(s)),
            serde_json::Value::Object(_) => {
                let chat: ChatObject = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(DescriptionInfo::ChatObject(chat))
            }
            _ => Ok(DescriptionInfo::Plain(value.to_string())),
        }
    }
}

impl DescriptionInfo {
    /// Extracts plain text representation of the MOTD.
    pub fn to_plain_text(&self) -> String {
        match self {
            DescriptionInfo::Plain(s) => s.clone(),
            DescriptionInfo::ChatObject(obj) => {
                let mut out = obj.text.clone();
                if let Some(ref extra) = obj.extra {
                    for item in extra {
                        match item {
                            ChatExtra::Plain(p) => out.push_str(p),
                            ChatExtra::Object(sub) => out.push_str(&sub.text),
                        }
                    }
                }
                out
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_status_response_parsing() {
        let json = r#"{
            "version": {
                "name": "1.8.9",
                "protocol": 47
            },
            "players": {
                "max": 200,
                "online": 42,
                "sample": [
                    { "name": "Steve", "id": "00000000-0000-0000-0000-000000000000" }
                ]
            },
            "description": "§6Online Server §7[1.8-1.20]",
            "favicon": "data:image/png;base64,mockfavicon"
        }"#;

        let pkt = StatusResponsePacket::new(json);
        let mut buf = BytesMut::new();
        pkt.encode(&mut buf).unwrap();

        let mut reader = buf.freeze();
        let decoded = StatusResponsePacket::decode(&mut reader).unwrap();
        let status = decoded.parse_status().unwrap();

        assert_eq!(status.version.protocol, 47);
        assert_eq!(status.players.online, 42);
        assert_eq!(status.players.max, 200);
        assert_eq!(status.description.to_plain_text(), "§6Online Server §7[1.8-1.20]");
    }

    #[test]
    fn test_status_response_chat_object_parsing() {
        let json = r#"{
            "version": { "name": "Paper 1.8.8", "protocol": 47 },
            "players": { "max": 100, "online": 10 },
            "description": {
                "text": "Welcome to ",
                "extra": [
                    { "text": "Bedwars!" }
                ]
            }
        }"#;

        let status: ServerStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.description.to_plain_text(), "Welcome to Bedwars!");
    }
}
