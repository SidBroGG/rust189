use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("SHA-1 mismatch for file '{path}': expected {expected}, actual {actual}")]
    Sha1Mismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("Minecraft version '{0}' was not found in Version Manifest")]
    VersionNotFound(String),

    #[error("Client download not available in version details for '{0}'")]
    MissingClientDownload(String),

    #[error("Dangerous zip file path: '{0}'")]
    ZipSlipDetected(String),

    #[error("Asset processing error: {0}")]
    Other(String),
}
