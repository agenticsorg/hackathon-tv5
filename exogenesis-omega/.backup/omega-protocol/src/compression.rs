use crate::patterns::{GlobalPatterns, PatternDelta};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use thiserror::Error;

/// Compression error types
#[derive(Debug, Error)]
pub enum CompressionError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Compression error: {0}")]
    Compression(#[from] std::io::Error),

    #[error("Invalid data format")]
    InvalidFormat,
}

pub type Result<T> = std::result::Result<T, CompressionError>;

/// Compress a PatternDelta to bytes using zstd
///
/// Target: ~1KB compressed size for typical deltas
///
/// # Arguments
/// * `delta` - The pattern delta to compress
///
/// # Returns
/// Compressed bytes ready for network transmission
pub fn compress_delta(delta: &PatternDelta) -> Result<Vec<u8>> {
    // Serialize to bincode
    let serialized = bincode::serialize(delta)?;

    // Compress with zstd at level 3 (balanced speed/compression)
    let mut encoder = zstd::Encoder::new(Vec::new(), 3)?;
    encoder.write_all(&serialized)?;
    let compressed = encoder.finish()?;

    Ok(compressed)
}

/// Decompress bytes to PatternDelta
///
/// # Arguments
/// * `compressed` - Compressed bytes from network
///
/// # Returns
/// Decompressed PatternDelta
pub fn decompress_delta(compressed: &[u8]) -> Result<PatternDelta> {
    // Decompress with zstd
    let mut decoder = zstd::Decoder::new(compressed)?;
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    // Deserialize from bincode
    let delta = bincode::deserialize(&decompressed)?;

    Ok(delta)
}

/// Compress GlobalPatterns to bytes using zstd
///
/// Target: ~5KB compressed size for typical global patterns
///
/// # Arguments
/// * `patterns` - The global patterns to compress
///
/// # Returns
/// Compressed bytes ready for network transmission
pub fn compress_patterns(patterns: &GlobalPatterns) -> Result<Vec<u8>> {
    // Serialize to bincode
    let serialized = bincode::serialize(patterns)?;

    // Compress with zstd at level 5 (higher compression for larger payload)
    let mut encoder = zstd::Encoder::new(Vec::new(), 5)?;
    encoder.write_all(&serialized)?;
    let compressed = encoder.finish()?;

    Ok(compressed)
}

/// Decompress bytes to GlobalPatterns
///
/// # Arguments
/// * `compressed` - Compressed bytes from network
///
/// # Returns
/// Decompressed GlobalPatterns
pub fn decompress_patterns(compressed: &[u8]) -> Result<GlobalPatterns> {
    // Decompress with zstd
    let mut decoder = zstd::Decoder::new(compressed)?;
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    // Deserialize from bincode
    let patterns = bincode::deserialize(&decompressed)?;

    Ok(patterns)
}

/// Compress arbitrary data with zstd
///
/// # Arguments
/// * `data` - Raw bytes to compress
/// * `level` - Compression level (1-22, higher = better compression but slower)
///
/// # Returns
/// Compressed bytes
pub fn compress_bytes(data: &[u8], level: i32) -> Result<Vec<u8>> {
    let mut encoder = zstd::Encoder::new(Vec::new(), level)?;
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}

/// Decompress arbitrary data with zstd
///
/// # Arguments
/// * `compressed` - Compressed bytes
///
/// # Returns
/// Decompressed bytes
pub fn decompress_bytes(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = zstd::Decoder::new(compressed)?;
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Calculate compression ratio
///
/// # Arguments
/// * `original_size` - Size before compression
/// * `compressed_size` - Size after compression
///
/// # Returns
/// Compression ratio (e.g., 10.0 means 10:1 compression)
pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f32 {
    if compressed_size == 0 {
        return 0.0;
    }
    original_size as f32 / compressed_size as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::{AggregatedPattern, PatternContext, ViewingPattern};
    use crate::types::{ContentType, DayType, TimeSlot};

    fn create_test_pattern() -> ViewingPattern {
        ViewingPattern::new(
            vec![0.5; 384],
            PatternContext::new(
                TimeSlot::Evening,
                DayType::Weekend,
                ContentType::Movie,
                vec!["action".to_string(), "thriller".to_string()],
            ),
        )
    }

    #[test]
    fn test_compress_decompress_delta() {
        let mut delta = PatternDelta::new(1);
        delta.patterns_added.push(create_test_pattern());
        delta.patterns_added.push(create_test_pattern());

        // Compress
        let compressed = compress_delta(&delta).unwrap();

        // Should be compressed
        let original_size = bincode::serialize(&delta).unwrap().len();
        assert!(compressed.len() < original_size);

        // Decompress
        let decompressed = decompress_delta(&compressed).unwrap();
        assert_eq!(decompressed.patterns_added.len(), delta.patterns_added.len());
        assert_eq!(decompressed.local_version, delta.local_version);
    }

    #[test]
    fn test_compress_decompress_patterns() {
        let mut patterns = GlobalPatterns::new();
        patterns.similar.push(AggregatedPattern::new(vec![0.3; 384], 0.85, 10));
        patterns.trending.push(AggregatedPattern::new(vec![0.7; 384], 0.92, 50));

        // Compress
        let compressed = compress_patterns(&patterns).unwrap();

        // Should be compressed
        let original_size = bincode::serialize(&patterns).unwrap().len();
        assert!(compressed.len() < original_size);

        // Decompress
        let decompressed = decompress_patterns(&compressed).unwrap();
        assert_eq!(decompressed.similar.len(), patterns.similar.len());
        assert_eq!(decompressed.trending.len(), patterns.trending.len());
    }

    #[test]
    fn test_compression_ratio_calculation() {
        let ratio = compression_ratio(10000, 1000);
        assert_eq!(ratio, 10.0);

        let ratio_no_compression = compression_ratio(1000, 1000);
        assert_eq!(ratio_no_compression, 1.0);
    }

    #[test]
    fn test_compress_decompress_bytes() {
        let data = b"Hello, World! This is test data that should compress well when repeated. ".repeat(10);

        let compressed = compress_bytes(&data, 3).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = decompress_bytes(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_higher_compression_levels() {
        let data = b"Test data ".repeat(100);

        let compressed_level_1 = compress_bytes(&data, 1).unwrap();
        let compressed_level_10 = compress_bytes(&data, 10).unwrap();

        // Higher level should compress better
        assert!(compressed_level_10.len() <= compressed_level_1.len());
    }

    #[test]
    fn test_delta_compression_target() {
        // Create a realistic delta with ~100 patterns
        let mut delta = PatternDelta::new(42);
        for _ in 0..100 {
            delta.patterns_added.push(create_test_pattern());
        }

        let compressed = compress_delta(&delta).unwrap();
        let ratio = compression_ratio(
            bincode::serialize(&delta).unwrap().len(),
            compressed.len()
        );

        println!("Delta compression: {} bytes, ratio: {:.2}:1", compressed.len(), ratio);

        // Should achieve at least 5:1 compression
        assert!(ratio >= 5.0, "Compression ratio {:.2} is below target 5:1", ratio);
    }
}
