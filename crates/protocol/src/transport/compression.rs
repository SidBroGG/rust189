use crate::error::ProtocolError;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

/// Compresses a slice of bytes using Zlib (Deflate).
pub fn compress_zlib(data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder
        .write_all(data)
        .map_err(|e| ProtocolError::CompressionError(e.to_string()))?;
    encoder
        .finish()
        .map_err(|e| ProtocolError::CompressionError(e.to_string()))
}

/// Decompresses Zlib-compressed bytes expecting `uncompressed_size` output bytes.
pub fn decompress_zlib(data: &[u8], uncompressed_size: usize) -> Result<Vec<u8>, ProtocolError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::with_capacity(uncompressed_size);
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| ProtocolError::DecompressionError(e.to_string()))?;
    if decompressed.len() != uncompressed_size {
        return Err(ProtocolError::DecompressionError(format!(
            "Decompressed size mismatch: expected {}, got {}",
            uncompressed_size,
            decompressed.len()
        )));
    }
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zlib_packet_compression_roundtrip() {
        let original = vec![0xAB; 1024];
        let compressed = compress_zlib(&original).unwrap();
        assert!(compressed.len() < original.len());

        let decompressed = decompress_zlib(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }
}
