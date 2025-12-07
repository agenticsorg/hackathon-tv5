pub mod analytics;
pub mod cache;
pub mod catalog;
pub mod config;
pub mod embedding;
pub mod intent;
pub mod search;
pub mod server;

pub use analytics::{AnalyticsDashboard, PopularQuery, SearchAnalytics, ZeroResultQuery};
pub use cache::{CacheError, CacheStats, RedisCache};
pub use catalog::{
    AvailabilityUpdate, CatalogService, CatalogState, ContentResponse, CreateContentRequest,
    UpdateContentRequest,
};
pub use config::DiscoveryConfig;
pub use embedding::{EmbeddingClient, EmbeddingModel, EmbeddingProvider, EmbeddingService};
pub use intent::{IntentParser, ParsedIntent};
pub use search::{
    HybridSearchService, RankingConfig, RankingConfigStore, SearchRequest, SearchResponse,
};

use std::sync::Arc;

/// Initialize discovery service components
pub async fn init_service(
    config: Arc<DiscoveryConfig>,
) -> anyhow::Result<Arc<HybridSearchService>> {
    // Initialize database pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            config.database.connect_timeout_sec,
        ))
        .connect(&config.database.url)
        .await?;

    // Initialize Redis cache
    let cache_config = Arc::new(crate::config::CacheConfig {
        redis_url: config.cache.redis_url.clone(),
        search_ttl_sec: config.cache.search_ttl_sec,
        embedding_ttl_sec: config.cache.embedding_ttl_sec,
        intent_ttl_sec: config.cache.intent_ttl_sec,
    });
    let cache = Arc::new(RedisCache::new(cache_config).await?);

    // Initialize embedding client
    let embedding_provider = EmbeddingProvider::from_env();
    let embedding_model = EmbeddingModel::from_env();
    let api_key = if !config.embedding.api_key.is_empty() {
        config.embedding.api_key.clone()
    } else {
        std::env::var("OPENAI_API_KEY").unwrap_or_default()
    };

    let embedding_client = Arc::new(EmbeddingClient::new(
        api_key,
        embedding_provider,
        embedding_model,
        Some(cache.clone()),
    ));

    // Initialize intent parser with cache
    let intent_parser = Arc::new(IntentParser::new(
        config.embedding.api_url.clone(),
        config.embedding.api_key.clone(),
        cache.clone(),
    ));

    // Initialize vector search with embedding client
    let vector_search = Arc::new(
        search::vector::VectorSearch::new(
            config.vector.qdrant_url.clone(),
            config.vector.collection_name.clone(),
            config.vector.dimension,
        )
        .with_embedding_client((*embedding_client).clone()),
    );

    // Initialize keyword search
    let keyword_search = Arc::new(search::keyword::KeywordSearch::new(
        config.keyword.index_path.clone(),
    ));

    // Initialize hybrid search service
    let search_service = Arc::new(HybridSearchService::new(
        config.clone(),
        intent_parser,
        vector_search,
        keyword_search,
        db_pool,
        cache,
    ));

    Ok(search_service)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_service_initialization() {
        let config = Arc::new(DiscoveryConfig::default());

        // This will fail without actual database, but tests the structure
        let result = init_service(config).await;
        assert!(result.is_err()); // Expected to fail without real database
    }
}
