use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};
use uuid::Uuid;

pub mod autocomplete;
pub mod facets;
pub mod filters;
pub mod keyword;
pub mod personalization;
pub mod query_processor;
pub mod ranking;
pub mod vector;

pub use autocomplete::AutocompleteService;
pub use facets::{FacetCount, FacetService};
pub use filters::SearchFilters;
pub use keyword::KeywordSearch;
pub use personalization::PersonalizationService;
pub use query_processor::QueryProcessor;
pub use ranking::{RankingConfig, RankingConfigStore, UpdateRankingConfigRequest};
pub use vector::VectorSearch;

use crate::analytics::SearchAnalytics;
use crate::cache::RedisCache;
use crate::config::DiscoveryConfig;
use crate::intent::{IntentParser, ParsedIntent};
use media_gateway_core::{
    ActivityEventType, KafkaActivityProducer, UserActivityEvent, UserActivityProducer,
};

/// Hybrid search service orchestrator
pub struct HybridSearchService {
    config: Arc<DiscoveryConfig>,
    intent_parser: Arc<IntentParser>,
    vector_search: Arc<vector::VectorSearch>,
    keyword_search: Arc<keyword::KeywordSearch>,
    db_pool: sqlx::PgPool,
    cache: Arc<RedisCache>,
    facet_service: Arc<FacetService>,
    personalization_service: Arc<PersonalizationService>,
    analytics: Option<Arc<SearchAnalytics>>,
    activity_producer: Option<Arc<KafkaActivityProducer>>,
}

/// Search request
#[derive(Debug, Clone, Serialize)]
pub struct SearchRequest {
    pub query: String,
    pub filters: Option<SearchFilters>,
    pub page: u32,
    pub page_size: u32,
    pub user_id: Option<Uuid>,
    /// A/B test experiment variant (e.g., "control", "low_boost", "high_boost")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experiment_variant: Option<String>,
}

/// Search response
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: usize,
    pub page: u32,
    pub page_size: u32,
    pub query_parsed: ParsedIntent,
    pub search_time_ms: u64,
    /// Facet counts by dimension (genres, platforms, years, ratings)
    pub facets: HashMap<String, Vec<FacetCount>>,
}

/// Individual search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub content: ContentSummary,
    pub relevance_score: f32,
    pub match_reasons: Vec<String>,
    pub vector_similarity: Option<f32>,
    pub graph_score: Option<f32>,
    pub keyword_score: Option<f32>,
}

/// Content summary for search results
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContentSummary {
    pub id: Uuid,
    pub title: String,
    pub overview: String,
    pub release_year: i32,
    pub genres: Vec<String>,
    pub platforms: Vec<String>,
    pub popularity_score: f32,
}

impl HybridSearchService {
    /// Create new hybrid search service
    pub fn new(
        config: Arc<DiscoveryConfig>,
        intent_parser: Arc<IntentParser>,
        vector_search: Arc<vector::VectorSearch>,
        keyword_search: Arc<keyword::KeywordSearch>,
        db_pool: sqlx::PgPool,
        cache: Arc<RedisCache>,
    ) -> Self {
        // Initialize personalization service with default config
        let personalization_service = Arc::new(PersonalizationService::new(
            crate::config::PersonalizationConfig::default(),
            cache.clone(),
        ));

        // Initialize analytics service
        let analytics = Some(Arc::new(SearchAnalytics::new(db_pool.clone())));

        // Initialize activity event producer (optional, logs warning on failure)
        let activity_producer = match KafkaActivityProducer::from_env() {
            Ok(producer) => Some(Arc::new(producer)),
            Err(e) => {
                tracing::warn!(error = %e, "Failed to initialize activity event producer");
                None
            }
        };

        Self {
            config,
            intent_parser,
            vector_search,
            keyword_search,
            db_pool,
            cache,
            facet_service: Arc::new(FacetService::new()),
            personalization_service,
            analytics,
            activity_producer,
        }
    }

    /// Create new hybrid search service with custom personalization config
    pub fn new_with_personalization(
        config: Arc<DiscoveryConfig>,
        intent_parser: Arc<IntentParser>,
        vector_search: Arc<vector::VectorSearch>,
        keyword_search: Arc<keyword::KeywordSearch>,
        db_pool: sqlx::PgPool,
        cache: Arc<RedisCache>,
        personalization_config: crate::config::PersonalizationConfig,
    ) -> Self {
        let personalization_service = Arc::new(PersonalizationService::new(
            personalization_config,
            cache.clone(),
        ));

        // Initialize analytics service
        let analytics = Some(Arc::new(SearchAnalytics::new(db_pool.clone())));

        // Initialize activity event producer (optional)
        let activity_producer = match KafkaActivityProducer::from_env() {
            Ok(producer) => Some(Arc::new(producer)),
            Err(e) => {
                tracing::warn!(error = %e, "Failed to initialize activity event producer");
                None
            }
        };

        Self {
            config,
            intent_parser,
            vector_search,
            keyword_search,
            db_pool,
            cache,
            facet_service: Arc::new(FacetService::new()),
            personalization_service,
            analytics,
            activity_producer,
        }
    }

    /// Get analytics service
    pub fn analytics(&self) -> Option<Arc<SearchAnalytics>> {
        self.analytics.clone()
    }

    /// Execute hybrid search with caching
    #[instrument(skip(self), fields(query = %request.query, page = %request.page))]
    pub async fn search(&self, request: SearchRequest) -> anyhow::Result<SearchResponse> {
        let start_time = std::time::Instant::now();

        // Generate cache key from request
        let cache_key = self.generate_cache_key(&request);

        // Check cache first
        if let Ok(Some(cached_response)) = self.cache.get::<SearchResponse>(&cache_key).await {
            let cache_time_ms = start_time.elapsed().as_millis() as u64;
            info!(
                cache_key = %cache_key,
                cache_time_ms = %cache_time_ms,
                "Cache hit - returning cached search results"
            );
            return Ok(cached_response);
        }

        debug!(cache_key = %cache_key, "Cache miss - executing full search");

        // Execute full search pipeline
        let response = self.execute_search(&request).await?;

        // Publish user activity event (non-blocking)
        if let (Some(producer), Some(user_id)) = (&self.activity_producer, request.user_id) {
            let clicked_items: Vec<String> = response
                .results
                .iter()
                .take(10)
                .map(|r| r.content.id.to_string())
                .collect();

            let metadata = serde_json::json!({
                "query": request.query,
                "results_count": response.total_count,
                "clicked_items": clicked_items,
                "search_time_ms": response.search_time_ms,
            });

            let event = UserActivityEvent::new(user_id, ActivityEventType::SearchQuery, metadata);

            let producer_clone = producer.clone();
            tokio::spawn(async move {
                if let Err(e) = producer_clone.publish_activity(event).await {
                    tracing::warn!(error = %e, "Failed to publish search activity event");
                }
            });
        }

        // Log search event for analytics (non-blocking)
        let latency_ms = start_time.elapsed().as_millis() as i32;
        if let Some(analytics) = &self.analytics {
            let user_id = request.user_id.as_ref().map(|id| id.to_string());
            let filters = request
                .filters
                .as_ref()
                .map(|f| {
                    let mut map = std::collections::HashMap::new();
                    if !f.genres.is_empty() {
                        map.insert("genres".to_string(), serde_json::json!(f.genres));
                    }
                    if !f.platforms.is_empty() {
                        map.insert("platforms".to_string(), serde_json::json!(f.platforms));
                    }
                    if let Some((min, max)) = f.year_range {
                        map.insert("year_range".to_string(), serde_json::json!([min, max]));
                    }
                    if let Some((min, max)) = f.rating_range {
                        map.insert("rating_range".to_string(), serde_json::json!([min, max]));
                    }
                    map
                })
                .unwrap_or_default();

            let analytics_clone = analytics.clone();
            let query_clone = request.query.clone();
            tokio::spawn(async move {
                let _ = analytics_clone
                    .query_log()
                    .log_search(
                        &query_clone,
                        user_id.as_deref(),
                        response.total_count as i32,
                        latency_ms,
                        filters,
                    )
                    .await;
            });
        }

        // Cache results with 30-minute TTL
        if let Err(e) = self.cache.set(&cache_key, &response, 1800).await {
            // Log cache write error but don't fail the request
            debug!(error = %e, cache_key = %cache_key, "Failed to cache search results");
        } else {
            debug!(cache_key = %cache_key, ttl = 1800, "Cached search results");
        }

        Ok(response)
    }

    /// Execute the full search pipeline (without caching)
    #[instrument(skip(self), fields(query = %request.query))]
    async fn execute_search(&self, request: &SearchRequest) -> anyhow::Result<SearchResponse> {
        let start_time = std::time::Instant::now();

        // Phase 1: Parse intent
        let intent = self.intent_parser.parse(&request.query).await?;

        // Phase 2: Execute parallel search strategies
        let (vector_results, keyword_results) = tokio::join!(
            self.vector_search
                .search(&request.query, request.filters.clone()),
            self.keyword_search
                .search(&request.query, request.filters.clone())
        );

        // Phase 3: Merge results using Reciprocal Rank Fusion with fallback
        let merged_results = match (vector_results, keyword_results) {
            (Ok(vector_res), Ok(keyword_res)) => {
                // Both strategies succeeded
                self.reciprocal_rank_fusion(vector_res, keyword_res, self.config.search.rrf_k)
            }
            (Err(e), Ok(keyword_res)) => {
                // Vector search failed, fall back to keyword search only
                tracing::warn!(
                    error = %e,
                    "Vector search failed, falling back to keyword search only"
                );
                keyword_res
            }
            (Ok(vector_res), Err(e)) => {
                // Keyword search failed, use vector search only
                tracing::warn!(
                    error = %e,
                    "Keyword search failed, using vector search only"
                );
                vector_res
            }
            (Err(vector_err), Err(keyword_err)) => {
                // Both strategies failed
                tracing::error!(
                    vector_error = %vector_err,
                    keyword_error = %keyword_err,
                    "Both search strategies failed"
                );
                return Err(anyhow::anyhow!(
                    "All search strategies failed: vector={}, keyword={}",
                    vector_err,
                    keyword_err
                ));
            }
        };

        // Phase 4: Apply personalization if user_id provided
        let ranked_results = if let Some(user_id) = request.user_id {
            match self
                .personalization_service
                .personalize_results(
                    user_id,
                    merged_results.clone(),
                    request.experiment_variant.as_deref(),
                )
                .await
            {
                Ok(personalized) => personalized,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        user_id = %user_id,
                        "Personalization failed, using original ranking"
                    );
                    merged_results
                }
            }
        } else {
            merged_results
        };

        // Phase 5: Compute facets from all results (before pagination)
        let facets = self.facet_service.compute_facets(&ranked_results);

        // Phase 6: Paginate
        let total_count = ranked_results.len();
        let start = ((request.page - 1) * request.page_size) as usize;
        let end = std::cmp::min(start + request.page_size as usize, total_count);
        let page_results = ranked_results[start..end].to_vec();

        let search_time_ms = start_time.elapsed().as_millis() as u64;

        info!(
            search_time_ms = %search_time_ms,
            total_results = %total_count,
            facet_count = %facets.len(),
            "Completed full search execution"
        );

        Ok(SearchResponse {
            results: page_results,
            total_count,
            page: request.page,
            page_size: request.page_size,
            query_parsed: intent,
            search_time_ms,
            facets,
        })
    }

    /// Vector-only search
    pub async fn vector_search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        _limit: Option<usize>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        self.vector_search.search(query, filters).await
    }

    /// Keyword-only search
    pub async fn keyword_search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
        _limit: Option<usize>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        self.keyword_search.search(query, filters).await
    }

    /// Get content by ID
    pub async fn get_content_by_id(&self, id: Uuid) -> anyhow::Result<Option<ContentSummary>> {
        let result = sqlx::query_as::<_, ContentSummary>(
            r#"
            SELECT
                id,
                title,
                overview,
                release_year,
                genres,
                ARRAY[]::text[] as "platforms!: Vec<String>",
                popularity_score
            FROM content
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result)
    }

    /// Reciprocal Rank Fusion (RRF) algorithm
    /// Merges results from multiple search strategies
    ///
    /// # Quality Score Integration Point
    ///
    /// To integrate content quality scoring into search ranking:
    ///
    /// 1. Add quality_score field to ContentSummary struct
    /// 2. Apply quality boost factor in this function:
    ///    ```rust
    ///    let quality_boost = result.content.quality_score * quality_weight;
    ///    let final_score = rrf_score * (1.0 + quality_boost);
    ///    ```
    /// 3. Configure quality_weight in DiscoveryConfig (recommended: 0.1-0.3)
    /// 4. Quality scores come from ingestion::QualityScorer scoring
    ///
    /// This ensures high-quality content (complete metadata, fresh data,
    /// external ratings) ranks higher in search results while maintaining
    /// relevance-based ranking.
    fn reciprocal_rank_fusion(
        &self,
        vector_results: Vec<SearchResult>,
        keyword_results: Vec<SearchResult>,
        k: f32,
    ) -> Vec<SearchResult> {
        let mut scores: HashMap<Uuid, (f32, SearchResult)> = HashMap::new();

        // Process vector results
        for (rank, result) in vector_results.iter().enumerate() {
            let rrf_score = self.config.search.weights.vector / (k + (rank + 1) as f32);
            scores
                .entry(result.content.id)
                .and_modify(|(score, _)| *score += rrf_score)
                .or_insert((rrf_score, result.clone()));
        }

        // Process keyword results
        for (rank, result) in keyword_results.iter().enumerate() {
            let rrf_score = self.config.search.weights.keyword / (k + (rank + 1) as f32);
            scores
                .entry(result.content.id)
                .and_modify(|(score, _)| *score += rrf_score)
                .or_insert((rrf_score, result.clone()));
        }

        // Sort by combined score
        let mut merged: Vec<(f32, SearchResult)> = scores
            .into_iter()
            .map(|(_, (score, mut result))| {
                result.relevance_score = score;
                (score, result)
            })
            .collect();

        merged.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        merged.into_iter().map(|(_, result)| result).collect()
    }

    /// Generate cache key from search request using SHA256 hash
    ///
    /// The cache key includes:
    /// - Query string
    /// - Filters (genres, platforms, year range, rating range)
    /// - Pagination (page, page_size)
    /// - User ID for personalized results
    ///
    /// # Arguments
    /// * `request` - Search request to generate key for
    ///
    /// # Returns
    /// Cache key string in format: "search:{sha256_hash}"
    #[instrument(skip(self, request), fields(query = %request.query))]
    fn generate_cache_key(&self, request: &SearchRequest) -> String {
        // Serialize request to JSON for consistent hashing
        let json =
            serde_json::to_string(request).expect("SearchRequest serialization should never fail");

        // Generate SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let hash = hasher.finalize();
        let hash_hex = hex::encode(hash);

        // Create cache key with search prefix
        let key = format!("search:{}", hash_hex);
        debug!(cache_key = %key, "Generated cache key");

        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reciprocal_rank_fusion() {
        // Create mock results
        let content1 = ContentSummary {
            id: Uuid::new_v4(),
            title: "Movie 1".to_string(),
            overview: "Description".to_string(),
            release_year: 2020,
            genres: vec!["action".to_string()],
            platforms: vec![],
            popularity_score: 0.8,
        };

        let content2 = ContentSummary {
            id: Uuid::new_v4(),
            title: "Movie 2".to_string(),
            overview: "Description".to_string(),
            release_year: 2021,
            genres: vec!["drama".to_string()],
            platforms: vec![],
            popularity_score: 0.7,
        };

        let vector_results = vec![
            SearchResult {
                content: content1.clone(),
                relevance_score: 0.9,
                match_reasons: vec![],
                vector_similarity: Some(0.9),
                graph_score: None,
                keyword_score: None,
            },
            SearchResult {
                content: content2.clone(),
                relevance_score: 0.8,
                match_reasons: vec![],
                vector_similarity: Some(0.8),
                graph_score: None,
                keyword_score: None,
            },
        ];

        let keyword_results = vec![SearchResult {
            content: content2.clone(),
            relevance_score: 0.85,
            match_reasons: vec![],
            vector_similarity: None,
            graph_score: None,
            keyword_score: Some(0.85),
        }];

        // Mock config
        let config = Arc::new(DiscoveryConfig::default());

        // Mock cache config
        let cache_config = Arc::new(crate::config::CacheConfig {
            redis_url: "redis://localhost:6379".to_string(),
            search_ttl_sec: 1800,
            embedding_ttl_sec: 3600,
            intent_ttl_sec: 600,
        });

        // Skip test if Redis is not available
        let cache = match RedisCache::new(cache_config).await {
            Ok(c) => Arc::new(c),
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        // Create mock database pool (would fail if postgres not available)
        let db_pool = match sqlx::PgPool::connect("postgresql://localhost/test").await {
            Ok(pool) => pool,
            Err(_) => {
                eprintln!("Skipping test: PostgreSQL not available");
                return;
            }
        };

        let personalization_service = Arc::new(PersonalizationService::new(
            crate::config::PersonalizationConfig::default(),
            cache.clone(),
        ));

        let service = HybridSearchService {
            config,
            intent_parser: Arc::new(IntentParser::new(
                String::new(),
                String::new(),
                cache.clone(),
            )),
            vector_search: Arc::new(vector::VectorSearch::new(String::new(), String::new(), 768)),
            keyword_search: Arc::new(keyword::KeywordSearch::new(String::new())),
            db_pool: db_pool.clone(),
            cache,
            facet_service: Arc::new(FacetService::new()),
            personalization_service,
            analytics: Some(Arc::new(SearchAnalytics::new(db_pool))),
            activity_producer: None,
        };

        let merged = service.reciprocal_rank_fusion(vector_results, keyword_results, 60.0);

        // content2 should rank higher (appears in both results)
        assert_eq!(merged[0].content.id, content2.id);
    }

    #[test]
    fn test_cache_key_generation() {
        // Test that cache key generation is deterministic
        let request1 = SearchRequest {
            query: "test query".to_string(),
            filters: Some(SearchFilters {
                genres: vec!["action".to_string()],
                platforms: vec!["netflix".to_string()],
                year_range: Some((2020, 2024)),
                rating_range: None,
            }),
            page: 1,
            page_size: 20,
            user_id: Some(Uuid::nil()), // Use nil UUID for deterministic testing
            experiment_variant: None,
        };

        let request2 = request1.clone();

        // Serialize both requests
        let json1 = serde_json::to_string(&request1).unwrap();
        let json2 = serde_json::to_string(&request2).unwrap();

        // Generate hashes
        use sha2::Digest;
        let hash1 = hex::encode(Sha256::digest(json1.as_bytes()));
        let hash2 = hex::encode(Sha256::digest(json2.as_bytes()));

        // Same request should produce same hash
        assert_eq!(hash1, hash2, "Cache keys should be deterministic");

        // Verify key format
        let key = format!("search:{}", hash1);
        assert!(key.starts_with("search:"));
        assert_eq!(key.len(), "search:".len() + 64); // SHA256 = 64 hex chars
    }
}
