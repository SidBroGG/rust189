use crate::error::AssetError;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use tracing::info;
use zip::ZipArchive;

/// Summary of extracted files from client.jar.
#[derive(Debug, Default, Clone)]
pub struct ExtractionSummary {
    pub total_extracted: usize,
    pub textures_count: usize,
    pub models_count: usize,
    pub fonts_count: usize,
    pub texts_count: usize,
    pub other_count: usize,
}

/// Safely extracts game assets (textures, models, fonts, blockstates, texts)
/// from Minecraft `client.jar` into the target `assets` directory.
///
/// Strips the redundant leading `assets/` prefix from JAR entries so that
/// files are extracted cleanly to `<target_dir>/minecraft/...`.
pub fn extract_client_jar(
    jar_path: &Path,
    target_assets_dir: &Path,
) -> Result<ExtractionSummary, AssetError> {
    info!(jar = ?jar_path, target = ?target_assets_dir, "Extracting Minecraft assets from client.jar");

    let file = File::open(jar_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut summary = ExtractionSummary::default();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        // Only extract assets/minecraft/... and pack metadata
        let is_asset = name.starts_with("assets/minecraft/");
        let is_pack_meta = name == "pack.png" || name == "pack.mcmeta";

        if !is_asset && !is_pack_meta {
            continue;
        }

        // Relative destination path: strip "assets/" if present so it extracts to target/minecraft/...
        let rel_path_str = if let Some(stripped) = name.strip_prefix("assets/") {
            stripped
        } else {
            &name
        };

        let rel_path = Path::new(rel_path_str);

        // Security check against Zip Slip path traversal
        for component in rel_path.components() {
            if let std::path::Component::ParentDir = component {
                return Err(AssetError::ZipSlipDetected(name.clone()));
            }
        }

        let out_path = target_assets_dir.join(rel_path);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut out_file = File::create(&out_path)?;
        copy(&mut entry, &mut out_file)?;

        summary.total_extracted += 1;
        if name.contains("/textures/") {
            summary.textures_count += 1;
        } else if name.contains("/models/") {
            summary.models_count += 1;
        } else if name.contains("/font/") {
            summary.fonts_count += 1;
        } else if name.contains("/texts/") {
            summary.texts_count += 1;
        } else {
            summary.other_count += 1;
        }
    }

    info!(
        total = summary.total_extracted,
        textures = summary.textures_count,
        models = summary.models_count,
        fonts = summary.fonts_count,
        "Extraction of client.jar completed successfully"
    );

    Ok(summary)
}
