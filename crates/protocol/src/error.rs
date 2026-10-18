use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("VarInt exceeds maximum 5 bytes")]
    VarIntTooBig,

    #[error("VarLong exceeds maximum 10 bytes")]
    VarLongTooBig,

    #[error("String length {actual} exceeds maximum allowed length {max}")]
    StringTooLong { max: usize, actual: usize },

    #[error("Invalid UTF-8 string: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("Unexpected end of buffer/stream while reading protocol data")]
    UnexpectedEof,

    #[error("Buffer underflow: needed {needed} bytes, but only {available} bytes available")]
    BufferUnderflow { needed: usize, available: usize },

    #[error("Invalid NBT data: {0}")]
    InvalidNbt(String),

    #[error("Invalid Entity Metadata: {0}")]
    InvalidMetadata(String),

    #[error("Packet size {actual} exceeds maximum protocol limit {max}")]
    PacketTooLarge { max: usize, actual: usize },

    #[error("Decompression error: {0}")]
    DecompressionError(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Server disconnected client: {0}")]
    Disconnected(String),

    #[error("Unexpected packet ID: expected {expected}, got 0x{actual:02X}")]
    UnexpectedPacket { expected: &'static str, actual: i32 },

    #[error("Connection closed by peer")]
    ConnectionClosed,

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Invalid enum value for {enum_name}: {value}")]
    InvalidEnumValue { enum_name: &'static str, value: i32 },
}
