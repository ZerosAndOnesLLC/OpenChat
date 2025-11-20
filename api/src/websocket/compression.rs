use flate2::write::{GzDecoder, GzEncoder};
use flate2::Compression;
use std::io::Write;

/// Compress data using gzip
pub fn compress(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

/// Decompress gzip data
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = GzDecoder::new(Vec::new());
    decoder.write_all(data)?;
    decoder.finish()
}

/// Check if compression would be beneficial
/// Returns true if the data is larger than the threshold
pub fn should_compress(data: &[u8], threshold: usize) -> bool {
    data.len() > threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let original = b"Hello, World! This is a test message that should be compressed.";
        let compressed = compress(original).expect("Compression failed");
        let decompressed = decompress(&compressed).expect("Decompression failed");

        assert_eq!(original.to_vec(), decompressed);
        assert!(compressed.len() < original.len(), "Compression should reduce size");
    }

    #[test]
    fn test_should_compress() {
        let small_data = b"Small";
        let large_data = vec![0u8; 2000];

        assert!(!should_compress(small_data, 1024));
        assert!(should_compress(&large_data, 1024));
    }
}
