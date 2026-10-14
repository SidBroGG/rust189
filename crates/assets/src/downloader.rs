use crate::error::AssetError;
use sha1::{Digest, Sha1};
use std::path::Path;
use tokio::io::AsyncWriteExt;

/// Calculate SHA-1 hexadecimal hash for a byte slice.
pub fn calculate_sha1(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex_encode(&result)
}

/// Calculate SHA-1 hexadecimal hash for a file on disk asynchronously.
pub async fn calculate_file_sha1(path: &Path) -> Result<String, AssetError> {
    use tokio::io::AsyncReadExt;

    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hex_encode(&hasher.finalize()))
}

/// Verify if a file exists and has the expected SHA-1 hash (case-insensitive).
pub async fn verify_file_sha1(path: &Path, expected_sha1: &str) -> bool {
    if !path.exists() {
        return false;
    }

    match calculate_file_sha1(path).await {
        Ok(hash) => hash.eq_ignore_ascii_case(expected_sha1),
        Err(_) => false,
    }
}

/// Downloads a file from `url` to `target_path`, validating SHA-1 if provided.
/// Uses a temporary file before renaming to prevent corrupted partially-written files.
pub async fn download_file(
    client: &reqwest::Client,
    url: &str,
    target_path: &Path,
    expected_sha1: Option<&str>,
) -> Result<(), AssetError> {
    if let Some(parent) = target_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Check if target file already exists with matching SHA-1
    if let Some(expected) = expected_sha1 {
        if verify_file_sha1(target_path, expected).await {
            tracing::debug!(target = ?target_path, "File already exists with valid SHA-1, skipping download");
            return Ok(());
        }
    }

    let temp_path = target_path.with_extension("download_tmp");

    let mut last_error = None;
    for attempt in 1..=3 {
        match client.get(url).send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(resp) => match resp.bytes().await {
                    Ok(bytes) => {
                        let actual_sha1 = calculate_sha1(&bytes);
                        if let Some(expected) = expected_sha1 {
                            if !actual_sha1.eq_ignore_ascii_case(expected) {
                                return Err(AssetError::Sha1Mismatch {
                                    path: target_path.to_path_buf(),
                                    expected: expected.to_string(),
                                    actual: actual_sha1,
                                });
                            }
                        }

                        let mut temp_file = tokio::fs::File::create(&temp_path).await?;
                        temp_file.write_all(&bytes).await?;
                        temp_file.flush().await?;
                        drop(temp_file);

                        tokio::fs::rename(&temp_path, target_path).await?;
                        return Ok(());
                    }
                    Err(e) => last_error = Some(AssetError::Http(e)),
                },
                Err(e) => last_error = Some(AssetError::Http(e)),
            },
            Err(e) => last_error = Some(AssetError::Http(e)),
        }

        if attempt < 3 {
            tokio::time::sleep(std::time::Duration::from_millis(500 * attempt as u64)).await;
        }
    }

    Err(last_error.unwrap_or_else(|| AssetError::Other(format!("Failed to download {}", url))))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sha1() {
        assert_eq!(
            calculate_sha1(b""),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(
            calculate_sha1(b"Hello, Minecraft!"),
            "bfd7b2ce718f4f90c206cc85cacdb7c54c1b46d3"
        );
    }
}
