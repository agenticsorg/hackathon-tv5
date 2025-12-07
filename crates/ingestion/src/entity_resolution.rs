//! Entity resolution for matching content across platforms
//!
//! Implements the ResolveContentEntity algorithm from SPARC specification:
//! 1. EIDR exact matching (100% confidence)
//! 2. External ID matching (IMDb, TMDb) (99% confidence)
//! 3. Fuzzy title + year matching (90-98% confidence, threshold 0.85)
//! 4. Embedding similarity matching (85-95% confidence, threshold 0.92)

use crate::{normalizer::CanonicalContent, IngestionError, Result};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use strsim::normalized_levenshtein;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Entity match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatch {
    /// Matched entity ID (if found)
    pub entity_id: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Matching method used
    pub method: MatchMethod,
}

/// Method used for entity matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MatchMethod {
    /// Exact EIDR match
    EidrExact,
    /// External ID match (IMDb, TMDb)
    ExternalId { source: String },
    /// Fuzzy title and year matching
    FuzzyTitleYear,
    /// Embedding similarity
    EmbeddingSimilarity,
    /// No match found
    None,
}

/// Entity resolver for matching content across platforms
pub struct EntityResolver {
    pool: PgPool,
    entity_index: Arc<RwLock<HashMap<String, EntityRecord>>>,
    eidr_index: Arc<RwLock<HashMap<String, String>>>,
    imdb_index: Arc<RwLock<HashMap<String, String>>>,
    tmdb_index: Arc<RwLock<HashMap<String, String>>>,
    cache: Cache<String, Option<String>>,
}

/// Internal entity record for matching
#[derive(Debug, Clone)]
struct EntityRecord {
    entity_id: String,
    title: String,
    normalized_title: String,
    release_year: Option<i32>,
    eidr: Option<String>,
    imdb_id: Option<String>,
    tmdb_id: Option<String>,
    embedding: Option<Vec<f32>>,
}

impl EntityResolver {
    /// Create a new entity resolver with database persistence
    pub async fn new(pool: PgPool) -> Result<Self> {
        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(3600))
            .build();

        let resolver = Self {
            pool,
            entity_index: Arc::new(RwLock::new(HashMap::new())),
            eidr_index: Arc::new(RwLock::new(HashMap::new())),
            imdb_index: Arc::new(RwLock::new(HashMap::new())),
            tmdb_index: Arc::new(RwLock::new(HashMap::new())),
            cache,
        };

        resolver.load_from_database().await?;
        Ok(resolver)
    }

    /// Load entity mappings from database into memory indices
    async fn load_from_database(&self) -> Result<()> {
        info!("Loading entity mappings from database");

        #[derive(sqlx::FromRow)]
        struct EntityMapping {
            external_id: String,
            id_type: String,
            entity_id: String,
            confidence: f64,
        }

        let mappings: Vec<EntityMapping> = sqlx::query_as(
            r#"
            SELECT external_id, id_type, entity_id, confidence
            FROM entity_mappings
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| IngestionError::DatabaseError(e.to_string()))?;

        let mut eidr_idx = self.eidr_index.write().await;
        let mut imdb_idx = self.imdb_index.write().await;
        let mut tmdb_idx = self.tmdb_index.write().await;

        for mapping in mappings {
            match mapping.id_type.as_str() {
                "eidr" => {
                    eidr_idx.insert(mapping.external_id, mapping.entity_id);
                }
                "imdb" => {
                    imdb_idx.insert(mapping.external_id, mapping.entity_id);
                }
                "tmdb" => {
                    tmdb_idx.insert(mapping.external_id, mapping.entity_id);
                }
                _ => {
                    warn!("Unknown ID type: {}", mapping.id_type);
                }
            }
        }

        info!(
            "Loaded {} EIDR, {} IMDB, {} TMDB mappings",
            eidr_idx.len(),
            imdb_idx.len(),
            tmdb_idx.len()
        );

        Ok(())
    }

    /// Persist a new entity mapping to database
    async fn persist_mapping(
        &self,
        external_id: &str,
        id_type: &str,
        entity_id: &str,
        confidence: f32,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO entity_mappings (external_id, id_type, entity_id, confidence)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (external_id, id_type) DO UPDATE
            SET entity_id = EXCLUDED.entity_id,
                confidence = EXCLUDED.confidence,
                updated_at = NOW()
            "#,
        )
        .bind(external_id)
        .bind(id_type)
        .bind(entity_id)
        .bind(confidence as f64)
        .execute(&self.pool)
        .await
        .map_err(|e| IngestionError::DatabaseError(e.to_string()))?;

        debug!(
            "Persisted {} mapping: {} -> {}",
            id_type, external_id, entity_id
        );
        Ok(())
    }

    /// Check cache for entity ID
    async fn check_cache(&self, cache_key: &str) -> Option<String> {
        self.cache.get(cache_key).await.flatten()
    }

    /// Store entity ID in cache
    async fn store_cache(&self, cache_key: String, entity_id: Option<String>) {
        self.cache.insert(cache_key, entity_id).await;
    }

    /// Resolve entity for given content
    ///
    /// Implements the ResolveContentEntity algorithm with multiple matching strategies.
    /// Complexity: O(log n) with indices
    pub async fn resolve(&self, content: &CanonicalContent) -> Result<EntityMatch> {
        // Strategy 1: EIDR exact match (100% confidence)
        if let Some(eidr) = content.external_ids.get("eidr") {
            let cache_key = format!("eidr:{}", eidr);

            if let Some(cached_id) = self.check_cache(&cache_key).await {
                return Ok(EntityMatch {
                    entity_id: Some(cached_id),
                    confidence: 1.0,
                    method: MatchMethod::EidrExact,
                });
            }

            let eidr_idx = self.eidr_index.read().await;
            if let Some(entity_id) = eidr_idx.get(eidr) {
                let entity_id_clone = entity_id.clone();
                drop(eidr_idx);

                self.store_cache(cache_key, Some(entity_id_clone.clone()))
                    .await;
                return Ok(EntityMatch {
                    entity_id: Some(entity_id_clone),
                    confidence: 1.0,
                    method: MatchMethod::EidrExact,
                });
            }
        }

        // Strategy 2: External ID matching (99% confidence)
        // Check IMDb
        if let Some(imdb_id) = content.external_ids.get("imdb") {
            let cache_key = format!("imdb:{}", imdb_id);

            if let Some(cached_id) = self.check_cache(&cache_key).await {
                return Ok(EntityMatch {
                    entity_id: Some(cached_id),
                    confidence: 0.99,
                    method: MatchMethod::ExternalId {
                        source: "imdb".to_string(),
                    },
                });
            }

            let imdb_idx = self.imdb_index.read().await;
            if let Some(entity_id) = imdb_idx.get(imdb_id) {
                let entity_id_clone = entity_id.clone();
                drop(imdb_idx);

                self.store_cache(cache_key, Some(entity_id_clone.clone()))
                    .await;
                self.persist_mapping(imdb_id, "imdb", &entity_id_clone, 0.99)
                    .await?;

                return Ok(EntityMatch {
                    entity_id: Some(entity_id_clone),
                    confidence: 0.99,
                    method: MatchMethod::ExternalId {
                        source: "imdb".to_string(),
                    },
                });
            }
        }

        // Check TMDb
        if let Some(tmdb_id) = content.external_ids.get("tmdb") {
            let cache_key = format!("tmdb:{}", tmdb_id);

            if let Some(cached_id) = self.check_cache(&cache_key).await {
                return Ok(EntityMatch {
                    entity_id: Some(cached_id),
                    confidence: 0.99,
                    method: MatchMethod::ExternalId {
                        source: "tmdb".to_string(),
                    },
                });
            }

            let tmdb_idx = self.tmdb_index.read().await;
            if let Some(entity_id) = tmdb_idx.get(tmdb_id) {
                let entity_id_clone = entity_id.clone();
                drop(tmdb_idx);

                self.store_cache(cache_key, Some(entity_id_clone.clone()))
                    .await;
                self.persist_mapping(tmdb_id, "tmdb", &entity_id_clone, 0.99)
                    .await?;

                return Ok(EntityMatch {
                    entity_id: Some(entity_id_clone),
                    confidence: 0.99,
                    method: MatchMethod::ExternalId {
                        source: "tmdb".to_string(),
                    },
                });
            }
        }

        // Strategy 3: Fuzzy title + year matching (threshold: 0.85)
        let fuzzy_match = self.fuzzy_title_year_match(content).await;
        if fuzzy_match.confidence >= 0.85 {
            if let Some(ref entity_id) = fuzzy_match.entity_id {
                if let Some(first_external_id) = content.external_ids.iter().next() {
                    let (id_type, external_id) = first_external_id;
                    self.persist_mapping(external_id, id_type, entity_id, fuzzy_match.confidence)
                        .await?;
                }
            }
            return Ok(fuzzy_match);
        }

        // Strategy 4: Embedding similarity (threshold: 0.92)
        if let Some(embedding) = &content.embedding {
            let embedding_match = self.embedding_similarity_match(embedding).await;
            if embedding_match.confidence >= 0.92 {
                if let Some(ref entity_id) = embedding_match.entity_id {
                    if let Some(first_external_id) = content.external_ids.iter().next() {
                        let (id_type, external_id) = first_external_id;
                        self.persist_mapping(
                            external_id,
                            id_type,
                            entity_id,
                            embedding_match.confidence,
                        )
                        .await?;
                    }
                }
                return Ok(embedding_match);
            }
        }

        // No match found
        Ok(EntityMatch {
            entity_id: None,
            confidence: 0.0,
            method: MatchMethod::None,
        })
    }

    /// Fuzzy title and year matching
    ///
    /// Returns 90-98% confidence based on similarity score
    async fn fuzzy_title_year_match(&self, content: &CanonicalContent) -> EntityMatch {
        let normalized_title = Self::normalize_title(&content.title);
        let mut best_match: Option<EntityMatch> = None;
        let mut best_score = 0.0;

        let entity_idx = self.entity_index.read().await;
        for record in entity_idx.values() {
            // Year must match (if available)
            if let (Some(content_year), Some(record_year)) =
                (content.release_year, record.release_year)
            {
                if (content_year - record_year).abs() > 1 {
                    continue; // Allow 1 year tolerance
                }
            }

            // Calculate title similarity
            let similarity = normalized_levenshtein(&normalized_title, &record.normalized_title);

            if similarity > best_score {
                best_score = similarity;
                best_match = Some(EntityMatch {
                    entity_id: Some(record.entity_id.clone()),
                    confidence: Self::calculate_fuzzy_confidence(similarity),
                    method: MatchMethod::FuzzyTitleYear,
                });
            }
        }

        best_match.unwrap_or(EntityMatch {
            entity_id: None,
            confidence: 0.0,
            method: MatchMethod::None,
        })
    }

    /// Embedding similarity matching
    ///
    /// Returns 85-95% confidence based on cosine similarity
    async fn embedding_similarity_match(&self, embedding: &[f32]) -> EntityMatch {
        let mut best_match: Option<EntityMatch> = None;
        let mut best_similarity = 0.0;

        let entity_idx = self.entity_index.read().await;
        for record in entity_idx.values() {
            if let Some(record_embedding) = &record.embedding {
                let similarity = Self::cosine_similarity(embedding, record_embedding);

                if similarity > best_similarity {
                    best_similarity = similarity;
                    best_match = Some(EntityMatch {
                        entity_id: Some(record.entity_id.clone()),
                        confidence: Self::calculate_embedding_confidence(similarity),
                        method: MatchMethod::EmbeddingSimilarity,
                    });
                }
            }
        }

        best_match.unwrap_or(EntityMatch {
            entity_id: None,
            confidence: 0.0,
            method: MatchMethod::None,
        })
    }

    /// Normalize title for fuzzy matching
    fn normalize_title(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Calculate confidence for fuzzy matching (90-98%)
    fn calculate_fuzzy_confidence(similarity: f64) -> f32 {
        // Map similarity [0.85, 1.0] to confidence [0.90, 0.98]
        let normalized = ((similarity - 0.85) / 0.15).max(0.0).min(1.0);
        (0.90 + normalized * 0.08) as f32
    }

    /// Calculate confidence for embedding matching (85-95%)
    fn calculate_embedding_confidence(similarity: f64) -> f32 {
        // Map similarity [0.92, 1.0] to confidence [0.85, 0.95]
        let normalized = ((similarity - 0.92) / 0.08).max(0.0).min(1.0);
        (0.85 + normalized * 0.10) as f32
    }

    /// Calculate cosine similarity between two embeddings
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        (dot_product / (norm_a * norm_b)) as f64
    }

    /// Add entity to index (for testing and database sync)
    #[cfg(test)]
    pub async fn add_entity(
        &self,
        entity_id: String,
        title: String,
        release_year: Option<i32>,
        eidr: Option<String>,
        imdb_id: Option<String>,
        tmdb_id: Option<String>,
        embedding: Option<Vec<f32>>,
    ) -> Result<()> {
        let normalized_title = Self::normalize_title(&title);

        let record = EntityRecord {
            entity_id: entity_id.clone(),
            title: title.clone(),
            normalized_title,
            release_year,
            eidr: eidr.clone(),
            imdb_id: imdb_id.clone(),
            tmdb_id: tmdb_id.clone(),
            embedding,
        };

        // Update indices
        if let Some(eidr_val) = &eidr {
            let mut eidr_idx = self.eidr_index.write().await;
            eidr_idx.insert(eidr_val.clone(), entity_id.clone());
            drop(eidr_idx);
            self.persist_mapping(eidr_val, "eidr", &entity_id, 1.0)
                .await?;
        }
        if let Some(imdb_val) = &imdb_id {
            let mut imdb_idx = self.imdb_index.write().await;
            imdb_idx.insert(imdb_val.clone(), entity_id.clone());
            drop(imdb_idx);
            self.persist_mapping(imdb_val, "imdb", &entity_id, 0.99)
                .await?;
        }
        if let Some(tmdb_val) = &tmdb_id {
            let mut tmdb_idx = self.tmdb_index.write().await;
            tmdb_idx.insert(tmdb_val.clone(), entity_id.clone());
            drop(tmdb_idx);
            self.persist_mapping(tmdb_val, "tmdb", &entity_id, 0.99)
                .await?;
        }

        let mut entity_idx = self.entity_index.write().await;
        entity_idx.insert(entity_id, record);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_normalization() {
        assert_eq!(
            EntityResolver::normalize_title("The Matrix (1999)"),
            "the matrix 1999"
        );
        assert_eq!(
            EntityResolver::normalize_title("Star Wars: A New Hope"),
            "star wars a new hope"
        );
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((EntityResolver::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!(EntityResolver::cosine_similarity(&c, &d).abs() < 0.001);
    }

    #[test]
    fn test_fuzzy_confidence_calculation() {
        assert!((EntityResolver::calculate_fuzzy_confidence(0.85) - 0.90).abs() < 0.01);
        assert!((EntityResolver::calculate_fuzzy_confidence(1.0) - 0.98).abs() < 0.01);
    }

    #[test]
    fn test_embedding_confidence_calculation() {
        assert!((EntityResolver::calculate_embedding_confidence(0.92) - 0.85).abs() < 0.01);
        assert!((EntityResolver::calculate_embedding_confidence(1.0) - 0.95).abs() < 0.01);
    }
}
