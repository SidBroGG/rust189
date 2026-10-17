pub mod downloader;
pub mod error;
pub mod extractor;
pub mod integrity;
pub mod lang;
pub mod manager;
pub mod manifest;
pub mod platform_io;
pub mod skin;

pub use downloader::{calculate_file_sha1, calculate_sha1, download_file, verify_file_sha1};
pub use error::AssetError;
pub use extractor::{extract_client_jar, ExtractionSummary};
pub use integrity::{check_fast_integrity, verify_full_integrity, IntegrityStatus, ESSENTIAL_GAME_FILES};
pub use lang::LanguageMap;
pub use manager::{AssetManager, AssetProgress};
pub use manifest::{AssetIndex, AssetObject, DownloadFile, VersionDetails, VersionManifest, VersionSummary};
pub use platform_io::{is_iocp_active, is_io_uring_available, PlatformDiskEngine};
pub use skin::{normalize_skin_texture, SkinManager};


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_deserialization() {
        let manifest_json = r#"{
            "latest": {
                "release": "1.8.9",
                "snapshot": "15w51b"
            },
            "versions": [
                {
                    "id": "1.8.9",
                    "type": "release",
                    "url": "https://piston-meta.mojang.com/v1/packages/b7b9e7d956e1807469a4ffad748c1a60cc8eb808/1.8.9.json",
                    "time": "2015-12-09T14:48:42+00:00",
                    "releaseTime": "2015-12-09T14:48:42+00:00",
                    "sha1": "b7b9e7d956e1807469a4ffad748c1a60cc8eb808"
                }
            ]
        }"#;

        let manifest: VersionManifest = serde_json::from_str(manifest_json).expect("Failed to parse manifest");
        assert_eq!(manifest.latest.release, "1.8.9");
        assert_eq!(manifest.versions.len(), 1);
        assert_eq!(manifest.versions[0].id, "1.8.9");
    }

    #[test]
    fn test_version_details_deserialization() {
        let details_json = r#"{
            "id": "1.8.9",
            "assetIndex": {
                "id": "1.8",
                "sha1": "017838e1215b021319c80d467ff9498a44b5a45e",
                "size": 133469,
                "totalSize": 115206263,
                "url": "https://launchermeta.mojang.com/v1/packages/017838e1215b021319c80d467ff9498a44b5a45e/1.8.json"
            },
            "downloads": {
                "client": {
                    "sha1": "7132915598fa2bd33e144a1eb40ec7c413b5ee0c",
                    "size": 8627882,
                    "url": "https://piston-data.mojang.com/v1/objects/7132915598fa2bd33e144a1eb40ec7c413b5ee0c/client.jar"
                }
            }
        }"#;

        let details: VersionDetails = serde_json::from_str(details_json).expect("Failed to parse details");
        assert_eq!(details.id, "1.8.9");
        assert_eq!(details.asset_index.id, "1.8");
        assert_eq!(details.downloads.client.sha1, "7132915598fa2bd33e144a1eb40ec7c413b5ee0c");
    }

    #[test]
    fn test_asset_object_cdn_url() {
        let obj = AssetObject {
            hash: "c8077b9bc4023c13a9c17f0f62e8dae412345678".to_string(),
            size: 1024,
        };
        assert_eq!(obj.cdn_subpath(), "c8/c8077b9bc4023c13a9c17f0f62e8dae412345678");
        assert_eq!(
            obj.cdn_url(),
            "https://resources.download.minecraft.net/c8/c8077b9bc4023c13a9c17f0f62e8dae412345678"
        );
    }
}
