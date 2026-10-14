use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Official Mojang Version Manifest (`version_manifest_v2.json` or `version_manifest.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSummary {
    pub id: String,
    #[serde(rename = "type")]
    pub release_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(default)]
    pub sha1: Option<String>,
}

/// Version details file for a specific release, e.g. `1.8.9.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDetails {
    pub id: String,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndexRef,
    pub downloads: VersionDownloads,
}

/// Reference to the asset index inside version details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize", default)]
    pub total_size: Option<u64>,
    pub url: String,
}

/// Downloads section containing client.jar and server.jar info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDownloads {
    pub client: DownloadFile,
    #[serde(default)]
    pub server: Option<DownloadFile>,
}

/// Downloadable file entry with SHA-1 checksum and size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadFile {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

/// Asset index file (`1.8.json`) detailing all vanilla audio, lang and other hashed objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
}

/// Single hashed asset object in Mojang CDN.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

impl AssetObject {
    /// Constructs the relative CDN path for this object: `<hash[0..2]>/<hash>`.
    pub fn cdn_subpath(&self) -> String {
        let prefix = &self.hash[0..2.min(self.hash.len())];
        format!("{}/{}", prefix, self.hash)
    }

    /// Constructs full Mojang CDN download URL for this asset object.
    pub fn cdn_url(&self) -> String {
        format!("https://resources.download.minecraft.net/{}", self.cdn_subpath())
    }
}
