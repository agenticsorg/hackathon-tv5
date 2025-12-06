//! Pattern storage abstraction
//!
//! TODO: Integrate with RuVector-Postgres for production deployment
//! Currently using in-memory storage for development and testing

use crate::{DeviceId, PatternId, Result, TrendSignal, ViewingPattern};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::debug;

/// Pattern storage trait
#[async_trait]
pub trait PatternStorage: Send + Sync {
    /// Store patterns for a device
    async fn store_patterns(
        &self,
        device_id: &DeviceId,
        patterns: Vec<ViewingPattern>,
    ) -> Result<()>;

    /// Update an existing pattern's metrics
    async fn update_pattern(
        &self,
        pattern_id: &PatternId,
        new_success_rate: f32,
        additional_samples: u32,
    ) -> Result<()>;

    /// Remove a pattern
    async fn remove_pattern(&self, pattern_id: &PatternId) -> Result<()>;

    /// Get similar patterns from other devices
    async fn get_similar_patterns(
        &self,
        device_id: &DeviceId,
        limit: usize,
    ) -> Result<Vec<ViewingPattern>>;

    /// Get trending content in a region
    async fn get_trending(&self, region: &str, limit: usize) -> Result<Vec<TrendSignal>>;

    /// Get total pattern count
    async fn get_total_patterns(&self) -> Result<usize>;

    /// Get pattern count for a device
    async fn get_device_pattern_count(&self, device_id: &DeviceId) -> Result<usize>;
}

/// In-memory storage implementation for development
pub struct InMemoryStorage {
    patterns: Arc<DashMap<PatternId, (DeviceId, ViewingPattern)>>,
    device_patterns: Arc<DashMap<DeviceId, Vec<PatternId>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            patterns: Arc::new(DashMap::new()),
            device_patterns: Arc::new(DashMap::new()),
        }
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }

        dot / (mag_a * mag_b)
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PatternStorage for InMemoryStorage {
    async fn store_patterns(
        &self,
        device_id: &DeviceId,
        patterns: Vec<ViewingPattern>,
    ) -> Result<()> {
        debug!("Storing {} patterns for device {}", patterns.len(), device_id);

        for pattern in patterns {
            let pattern_id = pattern.id;
            self.patterns
                .insert(pattern_id, (*device_id, pattern.clone()));

            self.device_patterns
                .entry(*device_id)
                .or_insert_with(Vec::new)
                .push(pattern_id);
        }

        Ok(())
    }

    async fn update_pattern(
        &self,
        pattern_id: &PatternId,
        new_success_rate: f32,
        additional_samples: u32,
    ) -> Result<()> {
        if let Some(mut entry) = self.patterns.get_mut(pattern_id) {
            let (_, pattern) = entry.value_mut();
            // Weighted average of success rates
            let total_samples = pattern.sample_count + additional_samples;
            pattern.success_rate = (pattern.success_rate * pattern.sample_count as f32
                + new_success_rate * additional_samples as f32)
                / total_samples as f32;
            pattern.sample_count = total_samples;
            pattern.updated_at = chrono::Utc::now().timestamp();
        }

        Ok(())
    }

    async fn remove_pattern(&self, pattern_id: &PatternId) -> Result<()> {
        if let Some((_, (device_id, _))) = self.patterns.remove(pattern_id) {
            if let Some(mut patterns) = self.device_patterns.get_mut(&device_id) {
                patterns.retain(|id| id != pattern_id);
            }
        }

        Ok(())
    }

    async fn get_similar_patterns(
        &self,
        device_id: &DeviceId,
        limit: usize,
    ) -> Result<Vec<ViewingPattern>> {
        // Get device's patterns to find interests
        let device_pattern_ids = self
            .device_patterns
            .get(device_id)
            .map(|ids| ids.clone())
            .unwrap_or_default();

        if device_pattern_ids.is_empty() {
            // New device, return high-quality patterns
            let mut patterns: Vec<ViewingPattern> = self
                .patterns
                .iter()
                .filter(|entry| entry.value().0 != *device_id)
                .filter(|entry| entry.value().1.success_rate >= 0.8)
                .map(|entry| entry.value().1.clone())
                .take(limit)
                .collect();

            patterns.sort_by(|a, b| {
                b.success_rate
                    .partial_cmp(&a.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            return Ok(patterns);
        }

        // Calculate average embedding for device interests
        let device_embeddings: Vec<Vec<f32>> = device_pattern_ids
            .iter()
            .filter_map(|id| {
                self.patterns
                    .get(id)
                    .map(|entry| entry.value().1.embedding.clone())
            })
            .collect();

        if device_embeddings.is_empty() {
            return Ok(vec![]);
        }

        let dim = device_embeddings[0].len();
        let avg_embedding: Vec<f32> = (0..dim)
            .map(|i| {
                device_embeddings.iter().map(|e| e[i]).sum::<f32>()
                    / device_embeddings.len() as f32
            })
            .collect();

        // Find similar patterns from other devices
        let mut scored_patterns: Vec<(f32, ViewingPattern)> = self
            .patterns
            .iter()
            .filter(|entry| entry.value().0 != *device_id)
            .filter(|entry| entry.value().1.success_rate >= 0.8)
            .map(|entry| {
                let pattern = &entry.value().1;
                let similarity = Self::cosine_similarity(&avg_embedding, &pattern.embedding);
                (similarity, pattern.clone())
            })
            .collect();

        // Sort by similarity (descending)
        scored_patterns.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored_patterns
            .into_iter()
            .take(limit)
            .map(|(_, pattern)| pattern)
            .collect())
    }

    async fn get_trending(&self, region: &str, limit: usize) -> Result<Vec<TrendSignal>> {
        // Simple trending: most frequent patterns with high success rates
        let mut pattern_counts: std::collections::HashMap<String, (usize, f32)> =
            std::collections::HashMap::new();

        for entry in self.patterns.iter() {
            let pattern = &entry.value().1;
            if pattern.success_rate >= 0.8 {
                // Use first genre as content identifier for simplicity
                let content_id = pattern
                    .context
                    .genre_hints
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());

                let entry = pattern_counts.entry(content_id).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += pattern.success_rate;
            }
        }

        let mut trends: Vec<TrendSignal> = pattern_counts
            .into_iter()
            .map(|(content_id, (count, total_rate))| TrendSignal {
                content_id,
                trending_score: (count as f32) * (total_rate / count as f32),
                region: region.to_string(),
            })
            .collect();

        trends.sort_by(|a, b| {
            b.trending_score
                .partial_cmp(&a.trending_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(trends.into_iter().take(limit).collect())
    }

    async fn get_total_patterns(&self) -> Result<usize> {
        Ok(self.patterns.len())
    }

    async fn get_device_pattern_count(&self, device_id: &DeviceId) -> Result<usize> {
        Ok(self
            .device_patterns
            .get(device_id)
            .map(|ids| ids.len())
            .unwrap_or(0))
    }
}

/// RuVector-Postgres storage implementation for production deployment
///
/// This implementation uses PostgreSQL with the pgvector extension
/// for SIMD-accelerated vector similarity search with HNSW indexing.
pub struct PostgresStorage {
    pool: sqlx::PgPool,
}

impl PostgresStorage {
    /// Create new PostgreSQL storage with vector support
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::PgPool::connect(database_url).await?;

        // Ensure pgvector extension is loaded
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&pool)
            .await?;

        // Create patterns table with vector column
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS patterns (
                id UUID PRIMARY KEY,
                device_id UUID NOT NULL,
                embedding vector(384) NOT NULL,
                success_rate FLOAT NOT NULL,
                sample_count INTEGER NOT NULL,
                context JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#,
        )
        .execute(&pool)
        .await?;

        // Create HNSW index for fast similarity search
        // Uses cosine distance for semantic similarity
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS patterns_embedding_idx
            ON patterns USING hnsw (embedding vector_cosine_ops)
            WITH (m = 32, ef_construction = 100)
        "#,
        )
        .execute(&pool)
        .await?;

        // Create index on device_id for fast device lookups
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS patterns_device_id_idx
            ON patterns (device_id)
        "#,
        )
        .execute(&pool)
        .await?;

        // Create index on success_rate for quality filtering
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS patterns_success_rate_idx
            ON patterns (success_rate)
        "#,
        )
        .execute(&pool)
        .await?;

        debug!("PostgreSQL storage initialized with pgvector support");
        Ok(Self { pool })
    }

    /// Get the database pool for advanced queries
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

#[async_trait]
impl PatternStorage for PostgresStorage {
    async fn store_patterns(
        &self,
        device_id: &DeviceId,
        patterns: Vec<ViewingPattern>,
    ) -> Result<()> {
        debug!("Storing {} patterns for device {}", patterns.len(), device_id);

        for pattern in patterns {
            // Convert embedding to pgvector format
            let embedding_str = format!(
                "[{}]",
                pattern
                    .embedding
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );

            sqlx::query(
                r#"
                INSERT INTO patterns (id, device_id, embedding, success_rate, sample_count, context, created_at, updated_at)
                VALUES ($1, $2, $3::vector, $4, $5, $6, to_timestamp($7), to_timestamp($8))
                ON CONFLICT (id) DO UPDATE SET
                    success_rate = EXCLUDED.success_rate,
                    sample_count = EXCLUDED.sample_count,
                    updated_at = EXCLUDED.updated_at
            "#,
            )
            .bind(pattern.id)
            .bind(device_id)
            .bind(&embedding_str)
            .bind(pattern.success_rate)
            .bind(pattern.sample_count as i32)
            .bind(serde_json::to_value(&pattern.context).unwrap())
            .bind(pattern.created_at)
            .bind(pattern.updated_at)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn update_pattern(
        &self,
        pattern_id: &PatternId,
        new_success_rate: f32,
        additional_samples: u32,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE patterns
            SET success_rate = (success_rate * sample_count + $2 * $3) / (sample_count + $3),
                sample_count = sample_count + $3,
                updated_at = NOW()
            WHERE id = $1
        "#,
        )
        .bind(pattern_id)
        .bind(new_success_rate)
        .bind(additional_samples as i32)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_pattern(&self, pattern_id: &PatternId) -> Result<()> {
        sqlx::query("DELETE FROM patterns WHERE id = $1")
            .bind(pattern_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_similar_patterns(
        &self,
        device_id: &DeviceId,
        limit: usize,
    ) -> Result<Vec<ViewingPattern>> {
        // Get device's average embedding
        let avg_result: Option<String> = sqlx::query_scalar(
            r#"
            SELECT AVG(embedding)::text
            FROM patterns
            WHERE device_id = $1
        "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?;

        let avg_embedding = match avg_result {
            Some(emb) => emb,
            None => {
                // New device, return high-quality patterns
                return self.get_high_quality_patterns(limit).await;
            }
        };

        // Query similar patterns using pgvector cosine distance
        // <=> operator computes cosine distance (lower is more similar)
        let rows = sqlx::query(
            r#"
            SELECT id, embedding::text, success_rate, sample_count, context,
                   EXTRACT(EPOCH FROM created_at)::bigint as created_at,
                   EXTRACT(EPOCH FROM updated_at)::bigint as updated_at
            FROM patterns
            WHERE device_id != $1
              AND success_rate >= 0.8
            ORDER BY embedding <=> $2::vector
            LIMIT $3
        "#,
        )
        .bind(device_id)
        .bind(&avg_embedding)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let patterns = rows
            .into_iter()
            .filter_map(|row| {
                let id: uuid::Uuid = row.try_get("id").ok()?;
                let embedding_str: String = row.try_get("embedding").ok()?;
                let success_rate: f32 = row.try_get("success_rate").ok()?;
                let sample_count: i32 = row.try_get("sample_count").ok()?;
                let context: serde_json::Value = row.try_get("context").ok()?;
                let created_at: i64 = row.try_get("created_at").ok()?;
                let updated_at: i64 = row.try_get("updated_at").ok()?;

                // Parse embedding vector from string
                let embedding = parse_pgvector(&embedding_str)?;
                let context: crate::PatternContext = serde_json::from_value(context).ok()?;

                Some(ViewingPattern {
                    id,
                    embedding,
                    success_rate,
                    sample_count: sample_count as u32,
                    context,
                    created_at,
                    updated_at,
                })
            })
            .collect();

        Ok(patterns)
    }

    async fn get_trending(&self, region: &str, limit: usize) -> Result<Vec<TrendSignal>> {
        // Aggregate trending content from high-quality patterns
        let rows = sqlx::query(
            r#"
            SELECT
                context->>'genre_hints'->0 as content_id,
                COUNT(*) as frequency,
                AVG(success_rate) as avg_success_rate
            FROM patterns
            WHERE success_rate >= 0.8
              AND context->>'genre_hints' IS NOT NULL
            GROUP BY context->>'genre_hints'->0
            ORDER BY COUNT(*) * AVG(success_rate) DESC
            LIMIT $1
        "#,
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let trends = rows
            .into_iter()
            .filter_map(|row| {
                let content_id: String = row.try_get("content_id").ok()?;
                let frequency: i64 = row.try_get("frequency").ok()?;
                let avg_success_rate: f64 = row.try_get("avg_success_rate").ok()?;

                Some(TrendSignal {
                    content_id,
                    trending_score: (frequency as f32) * (avg_success_rate as f32),
                    region: region.to_string(),
                })
            })
            .collect();

        Ok(trends)
    }

    async fn get_total_patterns(&self) -> Result<usize> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM patterns")
            .fetch_one(&self.pool)
            .await?;

        Ok(count as usize)
    }

    async fn get_device_pattern_count(&self, device_id: &DeviceId) -> Result<usize> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM patterns WHERE device_id = $1")
            .bind(device_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count as usize)
    }
}

impl PostgresStorage {
    /// Get high-quality patterns for new devices
    async fn get_high_quality_patterns(&self, limit: usize) -> Result<Vec<ViewingPattern>> {
        let rows = sqlx::query(
            r#"
            SELECT id, embedding::text, success_rate, sample_count, context,
                   EXTRACT(EPOCH FROM created_at)::bigint as created_at,
                   EXTRACT(EPOCH FROM updated_at)::bigint as updated_at
            FROM patterns
            WHERE success_rate >= 0.9
            ORDER BY success_rate DESC, sample_count DESC
            LIMIT $1
        "#,
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let patterns = rows
            .into_iter()
            .filter_map(|row| {
                let id: uuid::Uuid = row.try_get("id").ok()?;
                let embedding_str: String = row.try_get("embedding").ok()?;
                let success_rate: f32 = row.try_get("success_rate").ok()?;
                let sample_count: i32 = row.try_get("sample_count").ok()?;
                let context: serde_json::Value = row.try_get("context").ok()?;
                let created_at: i64 = row.try_get("created_at").ok()?;
                let updated_at: i64 = row.try_get("updated_at").ok()?;

                let embedding = parse_pgvector(&embedding_str)?;
                let context: crate::PatternContext = serde_json::from_value(context).ok()?;

                Some(ViewingPattern {
                    id,
                    embedding,
                    success_rate,
                    sample_count: sample_count as u32,
                    context,
                    created_at,
                    updated_at,
                })
            })
            .collect();

        Ok(patterns)
    }
}

/// Parse pgvector string format "[0.1, 0.2, ...]" to Vec<f32>
fn parse_pgvector(s: &str) -> Option<Vec<f32>> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return None;
    }

    let inner = &s[1..s.len() - 1];
    let values: Option<Vec<f32>> = inner
        .split(',')
        .map(|v| v.trim().parse::<f32>().ok())
        .collect();

    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PatternContext;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryStorage::new();
        let device_id = Uuid::new_v4();

        let pattern = ViewingPattern {
            id: Uuid::new_v4(),
            embedding: vec![0.1; 384],
            success_rate: 0.9,
            sample_count: 10,
            context: PatternContext {
                time_of_day: "evening".to_string(),
                day_of_week: "weekend".to_string(),
                content_type: "movie".to_string(),
                genre_hints: vec!["thriller".to_string()],
            },
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        storage
            .store_patterns(&device_id, vec![pattern.clone()])
            .await
            .unwrap();

        let count = storage.get_device_pattern_count(&device_id).await.unwrap();
        assert_eq!(count, 1);

        let total = storage.get_total_patterns().await.unwrap();
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        assert!((InMemoryStorage::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!((InMemoryStorage::cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }
}
