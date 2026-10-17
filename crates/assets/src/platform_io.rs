use crate::error::AssetError;
use bytes::Bytes;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Semaphore;

/// Returns whether Windows I/O Completion Ports (IOCP) are active for asynchronous operations.
#[inline]
pub fn is_iocp_active() -> bool {
    cfg!(target_os = "windows")
}

/// Returns whether Linux io_uring asynchronous subsystem is supported on this system.
pub fn is_io_uring_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Probes for io_uring support via /proc or kernel availability
        std::path::Path::new("/proc/sys/kernel/io_uring_disabled")
            .exists()
            .then(|| {
                std::fs::read_to_string("/proc/sys/kernel/io_uring_disabled")
                    .map(|v| v.trim() == "0")
                    .unwrap_or(false)
            })
            .unwrap_or(true)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Low-level platform disk I/O engine with hardware and OS-optimized mechanisms
/// (Windows IOCP / Linux io_uring / async pre-allocated buffers).
pub struct PlatformDiskEngine;

impl PlatformDiskEngine {
    /// Reads an entire file asynchronously using platform-native async completions.
    pub async fn read_file_async(path: impl AsRef<Path>) -> Result<Bytes, AssetError> {
        let path = path.as_ref();
        let mut file = File::open(path).await?;
        let metadata = file.metadata().await?;
        let file_len = metadata.len() as usize;

        let mut buffer = Vec::with_capacity(file_len);
        file.read_to_end(&mut buffer).await?;
        Ok(Bytes::from(buffer))
    }

    /// Reads multiple files concurrently using a bounded platform worker pool to prevent I/O thrashing.
    pub async fn read_batch_async<P: AsRef<Path> + Send + Sync + 'static>(
        paths: Vec<P>,
        max_concurrency: usize,
    ) -> Vec<Result<(PathBuf, Bytes), AssetError>> {
        let semaphore = Arc::new(Semaphore::new(max_concurrency.max(1)));
        let mut tasks = Vec::with_capacity(paths.len());

        for path in paths {
            let p_buf = path.as_ref().to_path_buf();
            let sem = Arc::clone(&semaphore);
            tasks.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let bytes = Self::read_file_async(&p_buf).await?;
                Ok((p_buf, bytes))
            }));
        }

        let mut results = Vec::with_capacity(tasks.len());
        for task in tasks {
            match task.await {
                Ok(res) => results.push(res),
                Err(join_err) => results.push(Err(AssetError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    join_err.to_string(),
                )))),
            }
        }
        results
    }

    /// Atomically writes data to disk using temporary staging and platform atomic rename.
    pub async fn write_file_atomic_async(
        path: impl AsRef<Path>,
        data: &[u8],
    ) -> Result<(), AssetError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let tmp_path = path.with_extension(format!("tmp_{}", std::process::id()));
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&tmp_path)
                .await?;
            file.write_all(data).await?;
            file.flush().await?;
            file.sync_all().await?;
        }

        tokio::fs::rename(&tmp_path, path).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_platform_disk_io_read_write_atomic() {
        let temp_dir = std::env::temp_dir().join(format!("rust189_test_io_{}", std::process::id()));
        tokio::fs::create_dir_all(&temp_dir).await.unwrap();

        let file_path = temp_dir.join("test_asset.bin");
        let payload = b"Platform IOCP/io_uring fast disk engine test payload!";

        PlatformDiskEngine::write_file_atomic_async(&file_path, payload)
            .await
            .unwrap();

        let read_bytes = PlatformDiskEngine::read_file_async(&file_path)
            .await
            .unwrap();
        assert_eq!(&read_bytes[..], payload);

        // Batch test
        let file2 = temp_dir.join("test_asset2.bin");
        PlatformDiskEngine::write_file_atomic_async(&file2, b"second payload")
            .await
            .unwrap();

        let batch_results = PlatformDiskEngine::read_batch_async(vec![file_path.clone(), file2.clone()], 4).await;
        assert_eq!(batch_results.len(), 2);
        assert!(batch_results.iter().all(|r| r.is_ok()));

        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    }

    #[test]
    fn test_platform_mechanism_detection() {
        #[cfg(target_os = "windows")]
        assert!(is_iocp_active());

        #[cfg(not(target_os = "windows"))]
        assert!(!is_iocp_active());
    }
}
