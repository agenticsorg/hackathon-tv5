use crate::types::{GlobalPatternsResponse, SyncResult};
use omega_agentdb::Skill;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Sync client for constellation communication
pub struct SyncClient {
    client: Client,
    constellation_url: String,
    device_id: String,
}

/// Delta payload to send to constellation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncDelta {
    device_id: String,
    patterns: Vec<PatternDelta>,
    version: u64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Pattern delta (compressed)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PatternDelta {
    name: String,
    embedding: Vec<f32>,
    usage_count: i32,
    success_rate: f64,
}

impl SyncClient {
    pub fn new(constellation_url: &str, device_id: String) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            constellation_url: constellation_url.to_string(),
            device_id,
        })
    }

    /// Prepare delta from high-quality skills
    pub fn prepare_delta(&self, skills: &[Skill]) -> anyhow::Result<Vec<u8>> {
        debug!("Preparing delta from {} high-quality skills", skills.len());

        let patterns: Vec<PatternDelta> = skills
            .iter()
            .map(|skill| PatternDelta {
                name: skill.name.clone(),
                embedding: skill.embedding.clone(),
                usage_count: skill.usage_count,
                success_rate: skill.success_rate,
            })
            .collect();

        let delta = SyncDelta {
            device_id: self.device_id.clone(),
            patterns,
            version: 1,
            timestamp: chrono::Utc::now(),
        };

        // Serialize to JSON
        let json = serde_json::to_vec(&delta)?;

        // Compress with zstd
        let compressed = zstd::encode_all(&json[..], 3)?;

        info!(
            "Delta prepared: {} patterns, {} bytes (compressed from {} bytes)",
            delta.patterns.len(),
            compressed.len(),
            json.len()
        );

        Ok(compressed)
    }

    /// Sync with constellation server
    pub async fn sync(&self, delta: Vec<u8>) -> anyhow::Result<GlobalPatternsResponse> {
        let url = format!("{}/api/v1/sync", self.constellation_url);

        info!(
            "Syncing with constellation: {} ({} bytes)",
            url,
            delta.len()
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/zstd")
            .header("X-Device-ID", &self.device_id)
            .body(delta)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error = response.text().await.unwrap_or_default();
            anyhow::bail!("Sync failed with status {}: {}", status, error);
        }

        // Decompress response
        let compressed_body = response.bytes().await?;
        let decompressed = zstd::decode_all(&compressed_body[..])?;

        // Parse response
        let global_patterns: GlobalPatternsResponse = serde_json::from_slice(&decompressed)?;

        info!(
            "Received {} global patterns (version {})",
            global_patterns.patterns.len(),
            global_patterns.version
        );

        Ok(global_patterns)
    }

    /// Full sync cycle: prepare delta, sync, return result
    pub async fn sync_cycle(&self, skills: &[Skill]) -> anyhow::Result<SyncResult> {
        let delta = self.prepare_delta(skills)?;
        let delta_size = delta.len();

        let global = self.sync(delta).await?;

        // Estimate global response size
        let global_size = serde_json::to_vec(&global)?.len();

        Ok(SyncResult {
            patterns_pushed: skills.len(),
            patterns_received: global.patterns.len(),
            sync_timestamp: chrono::Utc::now(),
            delta_size_bytes: delta_size,
            global_size_bytes: global_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_delta() {
        let client = SyncClient::new("http://localhost:8080", "test-device".to_string()).unwrap();

        let skills = vec![
            Skill {
                id: None,
                name: "test-skill".to_string(),
                description: "test".to_string(),
                embedding: vec![0.1; 384],
                usage_count: 10,
                success_rate: 0.8,
                created_at: chrono::Utc::now(),
            },
        ];

        let delta = client.prepare_delta(&skills).unwrap();

        // Should be compressed
        assert!(delta.len() > 0);
        assert!(delta.len() < skills.len() * 384 * 4); // Less than raw float data
    }
}
