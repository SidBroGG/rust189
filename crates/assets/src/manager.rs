use crate::downloader::download_file;
use crate::error::AssetError;
use crate::extractor::extract_client_jar;
use crate::integrity::{check_fast_integrity, IntegrityStatus};
use crate::manifest::{AssetIndex, VersionDetails, VersionManifest};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const VERSION_MANIFEST_FALLBACK_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest.json";
const TARGET_MC_VERSION: &str = "1.8.9";

/// Event representing progress during the asset verification and downloading pipeline.
#[derive(Debug, Clone)]
pub enum AssetProgress {
    VerifyingCache,
    FetchingVersionManifest,
    FetchingVersionDetails,
    DownloadingClientJar,
    ExtractingClientJar,
    FetchingAssetIndex,
    DownloadingObjects {
        current: usize,
        total: usize,
        current_file: String,
    },
    Ready,
}

/// Main asset management orchestrator for downloading and validating Minecraft 1.8.9 assets.
pub struct AssetManager {
    assets_dir: PathBuf,
    http_client: reqwest::Client,
}

impl AssetManager {
    /// Initialize a new AssetManager pointing to the specified assets directory.
    pub fn new<P: Into<PathBuf>>(assets_dir: P) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(15))
            .pool_max_idle_per_host(16)
            .user_agent("rust189-client/0.1.0")
            .build()
            .unwrap_or_default();

        Self {
            assets_dir: assets_dir.into(),
            http_client,
        }
    }

    pub fn assets_dir(&self) -> &Path {
        &self.assets_dir
    }

    /// Fast check if the local assets cache is already complete.
    pub fn is_cached(&self) -> bool {
        matches!(check_fast_integrity(&self.assets_dir), IntegrityStatus::Complete)
    }

    /// Verifies local assets cache or downloads missing assets from official Mojang CDN.
    pub async fn ensure_assets<F>(&self, on_progress: F) -> Result<(), AssetError>
    where
        F: Fn(AssetProgress) + Send + Sync,
    {
        on_progress(AssetProgress::VerifyingCache);

        if self.is_cached() {
            info!("All essential Minecraft 1.8.9 assets verified in local cache");
            on_progress(AssetProgress::Ready);
            return Ok(());
        }

        tokio::fs::create_dir_all(&self.assets_dir).await?;
        tokio::fs::create_dir_all(self.assets_dir.join("cache")).await?;
        tokio::fs::create_dir_all(self.assets_dir.join("indexes")).await?;
        tokio::fs::create_dir_all(self.assets_dir.join("objects")).await?;

        // 1. Download Version Manifest
        on_progress(AssetProgress::FetchingVersionManifest);
        let manifest = self.fetch_version_manifest().await?;

        let version_summary = manifest
            .versions
            .iter()
            .find(|v| v.id == TARGET_MC_VERSION)
            .ok_or_else(|| AssetError::VersionNotFound(TARGET_MC_VERSION.to_string()))?;

        // 2. Download Version Details (1.8.9.json)
        on_progress(AssetProgress::FetchingVersionDetails);
        let version_details_path = self.assets_dir.join("indexes/1.8.9.json");
        download_file(
            &self.http_client,
            &version_summary.url,
            &version_details_path,
            version_summary.sha1.as_deref(),
        )
        .await?;

        let version_details_bytes = tokio::fs::read(&version_details_path).await?;
        let version_details: VersionDetails = serde_json::from_slice(&version_details_bytes)?;

        // 3. Download client.jar
        on_progress(AssetProgress::DownloadingClientJar);
        let jar_path = self.assets_dir.join("cache/client-1.8.9.jar");
        download_file(
            &self.http_client,
            &version_details.downloads.client.url,
            &jar_path,
            Some(&version_details.downloads.client.sha1),
        )
        .await?;

        // 4. Extract client.jar assets (textures, models, fonts, blockstates)
        on_progress(AssetProgress::ExtractingClientJar);
        let jar_path_clone = jar_path.clone();
        let target_dir_clone = self.assets_dir.clone();
        tokio::task::spawn_blocking(move || {
            extract_client_jar(&jar_path_clone, &target_dir_clone)
        })
        .await
        .map_err(|e| AssetError::Other(e.to_string()))??;

        // 5. Download Asset Index (1.8.json)
        on_progress(AssetProgress::FetchingAssetIndex);
        let index_path = self.assets_dir.join("indexes/1.8.json");
        download_file(
            &self.http_client,
            &version_details.asset_index.url,
            &index_path,
            Some(&version_details.asset_index.sha1),
        )
        .await?;

        let index_bytes = tokio::fs::read(&index_path).await?;
        let asset_index: AssetIndex = serde_json::from_slice(&index_bytes)?;

        // 6. Download essential asset objects (language files, sounds.json, etc.)
        self.download_essential_objects(&asset_index, &on_progress).await?;

        info!("Minecraft 1.8.9 assets download and extraction finished successfully");
        on_progress(AssetProgress::Ready);
        Ok(())
    }

    async fn fetch_version_manifest(&self) -> Result<VersionManifest, AssetError> {
        info!("Fetching Minecraft version manifest");
        let resp = match self.http_client.get(VERSION_MANIFEST_URL).send().await {
            Ok(r) => r,
            Err(err) => {
                warn!(error = %err, "Failed to reach primary version manifest, trying fallback");
                self.http_client
                    .get(VERSION_MANIFEST_FALLBACK_URL)
                    .send()
                    .await?
            }
        };

        let bytes = resp.error_for_status()?.bytes().await?;
        let manifest: VersionManifest = serde_json::from_slice(&bytes)?;
        Ok(manifest)
    }

    async fn download_essential_objects<F>(
        &self,
        asset_index: &AssetIndex,
        on_progress: &F,
    ) -> Result<(), AssetError>
    where
        F: Fn(AssetProgress) + Send + Sync,
    {
        // Identify essential objects: languages (en_US, ru_RU), sounds index, and audio files
        let essential_keys: Vec<String> = asset_index
            .objects
            .keys()
            .filter(|key| {
                key == &"minecraft/lang/en_US.lang"
                    || key == &"minecraft/lang/ru_RU.lang"
                    || key == &"minecraft/sounds.json"
                    || key.starts_with("minecraft/sounds/")
            })
            .cloned()
            .collect();

        let total = essential_keys.len();
        info!(count = total, "Preparing to download hashed Minecraft assets");

        let chunk_size = 16;
        let mut completed = 0;

        for chunk in essential_keys.chunks(chunk_size) {
            let mut tasks = Vec::with_capacity(chunk.len());

            for key in chunk {
                let obj = match asset_index.objects.get(key) {
                    Some(o) => o.clone(),
                    None => continue,
                };

                let client = self.http_client.clone();
                let assets_dir = self.assets_dir.clone();
                let key_clone = key.clone();

                tasks.push(tokio::spawn(async move {
                    // Object storage path: assets/objects/<hash[0..2]>/<hash>
                    let obj_path = assets_dir.join("objects").join(obj.cdn_subpath());
                    download_file(&client, &obj.cdn_url(), &obj_path, Some(&obj.hash)).await?;

                    // Also place clean copy at direct game path (e.g. assets/minecraft/lang/en_US.lang)
                    let direct_path = assets_dir.join(&key_clone);
                    if let Some(parent) = direct_path.parent() {
                        tokio::fs::create_dir_all(parent).await?;
                    }
                    if !direct_path.exists() {
                        tokio::fs::copy(&obj_path, &direct_path).await?;
                    }

                    Ok::<(), AssetError>(())
                }));
            }

            for task in tasks {
                task.await.map_err(|e| AssetError::Other(e.to_string()))??;
                completed += 1;
            }

            if let Some(last_key) = chunk.last() {
                on_progress(AssetProgress::DownloadingObjects {
                    current: completed,
                    total,
                    current_file: last_key.clone(),
                });
            }
        }

        Ok(())
    }
}
