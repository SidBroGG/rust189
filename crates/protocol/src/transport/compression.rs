use crate::error::ProtocolError;
use libdeflater::{CompressionLvl, Compressor, Decompressor};

/// Compresses a slice of bytes using libdeflate Zlib at fast level.
pub fn compress_zlib(data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let mut compressor = Compressor::new(CompressionLvl::new(1).unwrap());
    let max_len = compressor.zlib_compress_bound(data.len());
    let mut out = vec![0u8; max_len];
    let actual_len = compressor
        .zlib_compress(data, &mut out)
        .map_err(|e| ProtocolError::CompressionError(format!("libdeflate zlib error: {:?}", e)))?;
    out.truncate(actual_len);
    Ok(out)
}

/// Decompresses Zlib-compressed bytes expecting `uncompressed_size` output bytes using libdeflate.
pub fn decompress_zlib(data: &[u8], uncompressed_size: usize) -> Result<Vec<u8>, ProtocolError> {
    let mut decompressed = vec![0u8; uncompressed_size];
    let mut decompressor = Decompressor::new();
    let actual_size = decompressor
        .zlib_decompress(data, &mut decompressed)
        .map_err(|e| ProtocolError::DecompressionError(format!("libdeflate zlib error: {:?}", e)))?;
    if actual_size != uncompressed_size {
        return Err(ProtocolError::DecompressionError(format!(
            "Decompressed size mismatch: expected {}, got {}",
            uncompressed_size, actual_size
        )));
    }
    Ok(decompressed)
}

/// Decompresses Zlib-compressed data directly into a pre-allocated destination slice (Zero-Allocation).
pub fn decompress_zlib_to_slice(data: &[u8], out: &mut [u8]) -> Result<usize, ProtocolError> {
    let mut decompressor = Decompressor::new();
    decompressor
        .zlib_decompress(data, out)
        .map_err(|e| ProtocolError::DecompressionError(format!("libdeflate zlib error: {:?}", e)))
}

/// Compresses data using Gzip format via libdeflate.
pub fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let mut compressor = Compressor::new(CompressionLvl::new(1).unwrap());
    let max_len = compressor.gzip_compress_bound(data.len());
    let mut out = vec![0u8; max_len];
    let actual_len = compressor
        .gzip_compress(data, &mut out)
        .map_err(|e| ProtocolError::CompressionError(format!("libdeflate gzip error: {:?}", e)))?;
    out.truncate(actual_len);
    Ok(out)
}

/// Decompresses Gzip data of unknown size using adaptive buffer growth via libdeflate.
pub fn decompress_gzip_dynamic(data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let mut decompressor = Decompressor::new();
    let mut capacity = (data.len() * 4).max(4096);
    loop {
        let mut out = vec![0u8; capacity];
        match decompressor.gzip_decompress(data, &mut out) {
            Ok(size) => {
                out.truncate(size);
                return Ok(out);
            }
            Err(libdeflater::DecompressionError::InsufficientSpace) => {
                capacity *= 2;
                if capacity > 64 * 1024 * 1024 {
                    return Err(ProtocolError::DecompressionError(
                        "Gzip decompressed size exceeds 64MB limit".into(),
                    ));
                }
            }
            Err(e) => return Err(ProtocolError::DecompressionError(format!("{:?}", e))),
        }
    }
}

/// Decompresses Zlib data of unknown size using adaptive buffer growth via libdeflate.
pub fn decompress_zlib_dynamic(data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let mut decompressor = Decompressor::new();
    let mut capacity = (data.len() * 4).max(4096);
    loop {
        let mut out = vec![0u8; capacity];
        match decompressor.zlib_decompress(data, &mut out) {
            Ok(size) => {
                out.truncate(size);
                return Ok(out);
            }
            Err(libdeflater::DecompressionError::InsufficientSpace) => {
                capacity *= 2;
                if capacity > 64 * 1024 * 1024 {
                    return Err(ProtocolError::DecompressionError(
                        "Zlib decompressed size exceeds 64MB limit".into(),
                    ));
                }
            }
            Err(e) => return Err(ProtocolError::DecompressionError(format!("{:?}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_libdeflate_zlib_roundtrip() {
        let original = vec![0xAB; 4096];
        let compressed = compress_zlib(&original).unwrap();
        assert!(compressed.len() < original.len());

        let decompressed = decompress_zlib(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);

        let dynamic = decompress_zlib_dynamic(&compressed).unwrap();
        assert_eq!(dynamic, original);
    }

    #[test]
    fn test_libdeflate_gzip_roundtrip() {
        let original = b"Minecraft Protocol 47 high performance libdeflate test string payload!";
        let compressed = compress_gzip(original).unwrap();
        let decompressed = decompress_gzip_dynamic(&compressed).unwrap();
        assert_eq!(&decompressed[..], &original[..]);
    }
}

