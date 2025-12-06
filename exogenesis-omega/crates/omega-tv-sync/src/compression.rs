//! Compression utilities for omega-tv-sync protocol
//!
//! Implements zstd compression to achieve target bandwidth:
//! - Push (TV → Constellation): ~1KB compressed
//! - Pull (Constellation → TV): ~5KB compressed

use crate::{Error, GlobalPatterns, Result, SyncDelta};
use std::io::{Read, Write};
use tracing::{debug, warn};
use zstd::stream::{read::Decoder, write::Encoder};

/// Target compression level (1-21, higher = better compression but slower)
/// Level 3 provides good balance of speed and compression for real-time sync
const COMPRESSION_LEVEL: i32 = 3;

/// Maximum compressed size for delta upload (safety limit)
const MAX_DELTA_COMPRESSED: usize = 2048; // 2KB max

/// Maximum compressed size for global patterns download (safety limit)
const MAX_GLOBAL_COMPRESSED: usize = 10240; // 10KB max

/// Compress a SyncDelta for upload to Constellation
///
/// Target: ~1KB compressed output
///
/// # Arguments
/// * `delta` - The sync delta to compress
///
/// # Returns
/// Compressed bytes suitable for HTTP transmission
///
/// # Errors
/// - `Error::Serialization` if JSON serialization fails
/// - `Error::Compression` if zstd compression fails
/// - `Error::CompressionLimit` if compressed size exceeds limit
pub fn compress(data: &SyncDelta) -> Result<Vec<u8>> {
    // Validate before serializing
    data.validate()?;

    // Serialize to JSON
    let json = serde_json::to_string(data)?;
    let uncompressed_size = json.len();
    debug!(
        "Compressing delta: {} bytes uncompressed",
        uncompressed_size
    );

    // Compress with zstd
    let mut encoder = Encoder::new(Vec::new(), COMPRESSION_LEVEL)?;
    encoder.write_all(json.as_bytes())?;
    let compressed = encoder.finish()?;

    let compressed_size = compressed.len();
    let ratio = uncompressed_size as f64 / compressed_size as f64;

    debug!(
        "Compressed delta: {} → {} bytes (ratio: {:.2}x)",
        uncompressed_size, compressed_size, ratio
    );

    // Check size limit
    if compressed_size > MAX_DELTA_COMPRESSED {
        warn!(
            "Delta compressed size {} exceeds limit {}",
            compressed_size, MAX_DELTA_COMPRESSED
        );
        return Err(Error::CompressionLimit(
            compressed_size,
            MAX_DELTA_COMPRESSED,
        ));
    }

    Ok(compressed)
}

/// Decompress GlobalPatterns received from Constellation
///
/// Expected input: ~5KB compressed
///
/// # Arguments
/// * `data` - Compressed bytes from HTTP response
///
/// # Returns
/// Decompressed global patterns
///
/// # Errors
/// - `Error::Compression` if zstd decompression fails
/// - `Error::Serialization` if JSON deserialization fails
/// - `Error::CompressionLimit` if input size exceeds limit
pub fn decompress(data: &[u8]) -> Result<GlobalPatterns> {
    let compressed_size = data.len();

    // Check input size limit
    if compressed_size > MAX_GLOBAL_COMPRESSED {
        return Err(Error::CompressionLimit(
            compressed_size,
            MAX_GLOBAL_COMPRESSED,
        ));
    }

    debug!("Decompressing global patterns: {} bytes", compressed_size);

    // Decompress with zstd
    let mut decoder = Decoder::new(data)?;
    let mut json = String::new();
    decoder.read_to_string(&mut json)?;

    let uncompressed_size = json.len();
    let ratio = uncompressed_size as f64 / compressed_size as f64;

    debug!(
        "Decompressed global patterns: {} → {} bytes (ratio: {:.2}x)",
        compressed_size, uncompressed_size, ratio
    );

    // Deserialize from JSON
    let global: GlobalPatterns = serde_json::from_str(&json)?;

    Ok(global)
}

/// Estimate compressed size without actually compressing
///
/// Used for pre-flight checks before sync operations
pub fn estimate_compressed_size(delta: &SyncDelta) -> usize {
    // Rough estimate: JSON size / 4 (zstd typically achieves 3-5x compression)
    let estimated_json = delta.estimate_size();
    estimated_json / 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PatternData;

    fn create_test_delta(pattern_count: usize) -> SyncDelta {
        let patterns: Vec<_> = (0..pattern_count)
            .map(|i| PatternData {
                id: format!("pattern-{}", i),
                embedding: vec![0.1; 384],
                success_rate: 0.85,
                sample_count: 100,
                genre: "action".to_string(),
            })
            .collect();

        SyncDelta::new("tv-test".to_string(), patterns, 1)
    }

    #[test]
    fn test_compress_small_delta() {
        let delta = create_test_delta(1);
        let compressed = compress(&delta).unwrap();

        // Should be well under 1KB for a single pattern
        assert!(compressed.len() < 1024);
        println!("Single pattern compressed: {} bytes", compressed.len());
    }

    #[test]
    fn test_compress_multiple_patterns() {
        let delta = create_test_delta(5);
        let compressed = compress(&delta).unwrap();

        // Should still be reasonably small
        assert!(compressed.len() < MAX_DELTA_COMPRESSED);
        println!("5 patterns compressed: {} bytes", compressed.len());
    }

    #[test]
    fn test_compress_decompress_roundtrip() {
        use crate::types::TrendSignal;

        let original_delta = create_test_delta(3);

        // Compress delta
        let _compressed = compress(&original_delta).unwrap();

        // For testing decompression, create a GlobalPatterns object
        let global = GlobalPatterns {
            patterns: original_delta.patterns.clone(),
            trends: vec![TrendSignal::new(
                "content-1".to_string(),
                0.95,
                "US".to_string(),
                "thriller".to_string(),
            )],
            version: 10,
            timestamp: chrono::Utc::now(),
        };

        // Compress global patterns
        let json = serde_json::to_string(&global).unwrap();
        let mut encoder = Encoder::new(Vec::new(), COMPRESSION_LEVEL).unwrap();
        encoder.write_all(json.as_bytes()).unwrap();
        let compressed_global = encoder.finish().unwrap();

        // Decompress
        let decompressed = decompress(&compressed_global).unwrap();

        assert_eq!(decompressed.patterns.len(), global.patterns.len());
        assert_eq!(decompressed.trends.len(), 1);
    }

    #[test]
    fn test_compression_ratio() {
        let delta = create_test_delta(10);

        // JSON serialize first to get uncompressed size
        let json = serde_json::to_string(&delta).unwrap();
        let uncompressed_size = json.len();

        // Compress
        let compressed = compress(&delta).unwrap();
        let compressed_size = compressed.len();

        let ratio = uncompressed_size as f64 / compressed_size as f64;

        println!(
            "Compression ratio: {} → {} bytes ({:.2}x)",
            uncompressed_size, compressed_size, ratio
        );

        // zstd should achieve at least 3x compression on JSON
        assert!(ratio >= 3.0);
    }

    #[test]
    fn test_size_limit_enforcement() {
        // Create a delta with many patterns to exceed limit
        let delta = create_test_delta(100);

        let result = compress(&delta);

        // Should either succeed under limit or fail with CompressionLimit error
        match result {
            Ok(compressed) => {
                assert!(compressed.len() <= MAX_DELTA_COMPRESSED);
            }
            Err(Error::CompressionLimit(size, limit)) => {
                assert!(size > limit);
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_estimate_compressed_size() {
        let delta = create_test_delta(5);
        let estimated = estimate_compressed_size(&delta);
        let actual = compress(&delta).unwrap().len();

        println!(
            "Estimated: {} bytes, Actual: {} bytes",
            estimated, actual
        );

        // Estimate should be within 2x of actual
        assert!(estimated > 0);
        assert!(actual < estimated * 2);
    }
}
