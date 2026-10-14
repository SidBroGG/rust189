use crate::downloader::verify_file_sha1;
use std::path::Path;
use tracing::debug;

/// List of critical files extracted from client.jar and index required for client rendering and GUI.
pub const ESSENTIAL_GAME_FILES: &[&str] = &[
    "minecraft/textures/gui/widgets.png",
    "minecraft/textures/gui/icons.png",
    "minecraft/textures/font/ascii.png",
    "minecraft/font/glyph_sizes.bin",
    "minecraft/lang/en_US.lang",
    "minecraft/sounds.json",
];

/// Status of local asset cache integrity check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityStatus {
    /// All essential assets and valid caches exist locally.
    Complete,
    /// Assets are missing or corrupted and need downloading/extraction.
    Incomplete { missing_files: Vec<String> },
}

/// Fast non-blocking check to verify if all essential assets already exist on disk.
/// This allows instantaneous client startup without network queries on subsequent runs.
pub fn check_fast_integrity(assets_root: &Path) -> IntegrityStatus {
    let mut missing = Vec::new();

    // Check cached client.jar
    let jar_path = assets_root.join("cache/client-1.8.9.jar");
    if !jar_path.exists() {
        missing.push("cache/client-1.8.9.jar".to_string());
    }

    // Check asset index 1.8.json
    let index_path = assets_root.join("indexes/1.8.json");
    if !index_path.exists() {
        missing.push("indexes/1.8.json".to_string());
    }

    // Check essential extracted assets
    for &rel_file in ESSENTIAL_GAME_FILES {
        let file_path = assets_root.join(rel_file);
        if !file_path.exists() {
            missing.push(rel_file.to_string());
        } else if let Ok(metadata) = std::fs::metadata(&file_path) {
            if metadata.len() == 0 {
                missing.push(format!("{rel_file} (empty)"));
            }
        }
    }

    if missing.is_empty() {
        debug!("Fast asset integrity check passed successfully");
        IntegrityStatus::Complete
    } else {
        debug!(missing = ?missing, "Fast asset integrity check found missing files");
        IntegrityStatus::Incomplete { missing_files: missing }
    }
}

/// Detailed verification that also validates cryptographic SHA-1 hashes of cached archives.
pub async fn verify_full_integrity(
    assets_root: &Path,
    client_sha1: &str,
    index_sha1: &str,
) -> bool {
    let jar_path = assets_root.join("cache/client-1.8.9.jar");
    if !verify_file_sha1(&jar_path, client_sha1).await {
        return false;
    }

    let index_path = assets_root.join("indexes/1.8.json");
    if !verify_file_sha1(&index_path, index_sha1).await {
        return false;
    }

    matches!(check_fast_integrity(assets_root), IntegrityStatus::Complete)
}
