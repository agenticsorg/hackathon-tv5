use anyhow::Result;
use media_gateway_discovery::cache::RedisCache;
use media_gateway_discovery::config::CacheConfig;
use media_gateway_discovery::{EmbeddingClient, EmbeddingModel, EmbeddingProvider};
use std::sync::Arc;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test OpenAI API request/response
#[tokio::test]
async fn test_openai_embedding_single() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "data": [{
            "embedding": vec![0.1, 0.2, 0.3],
            "index": 0
        }],
        "model": "text-embedding-3-small",
        "usage": {
            "prompt_tokens": 5,
            "total_tokens": 5
        }
    });

    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = EmbeddingClient::new(
        "test-key".to_string(),
        EmbeddingProvider::OpenAI,
        EmbeddingModel::Small,
        None,
    );

    // Override the API URL for testing
    std::env::set_var("OPENAI_API_URL", mock_server.uri());

    let result = client.generate("test query").await;
    assert!(result.is_ok());
}

/// Test batch embedding generation
#[tokio::test]
async fn test_openai_embedding_batch() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "data": [
            {
                "embedding": vec![0.1, 0.2, 0.3],
                "index": 0
            },
            {
                "embedding": vec![0.4, 0.5, 0.6],
                "index": 1
            }
        ],
        "model": "text-embedding-3-small",
        "usage": {
            "prompt_tokens": 10,
            "total_tokens": 10
        }
    });

    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = EmbeddingClient::new(
        "test-key".to_string(),
        EmbeddingProvider::OpenAI,
        EmbeddingModel::Small,
        None,
    );

    let texts = vec!["query1".to_string(), "query2".to_string()];
    let result = client.generate_batch(&texts).await;
    assert!(result.is_ok());
    let embeddings = result.unwrap();
    assert_eq!(embeddings.len(), 2);
}

/// Test OpenAI API error handling
#[tokio::test]
async fn test_openai_error_handling() {
    let mock_server = MockServer::start().await;

    let error_body = serde_json::json!({
        "error": {
            "message": "Invalid API key",
            "type": "invalid_request_error"
        }
    });

    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(401).set_body_json(error_body))
        .expect(3) // MAX_RETRIES
        .mount(&mock_server)
        .await;

    let client = EmbeddingClient::new(
        "invalid-key".to_string(),
        EmbeddingProvider::OpenAI,
        EmbeddingModel::Small,
        None,
    );

    let result = client.generate("test").await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("invalid_request_error"));
}

/// Test Redis caching integration
#[tokio::test]
async fn test_embedding_redis_cache() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_config = Arc::new(CacheConfig {
        redis_url,
        search_ttl_sec: 60,
        embedding_ttl_sec: 86400,
        intent_ttl_sec: 30,
    });

    let cache = match RedisCache::new(cache_config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Clear any existing cache
    let _ = cache.clear_embedding_cache().await;

    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "data": [{
            "embedding": vec![0.7, 0.8, 0.9],
            "index": 0
        }],
        "model": "text-embedding-3-small",
        "usage": {
            "prompt_tokens": 5,
            "total_tokens": 5
        }
    });

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .expect(1) // Should only be called once due to caching
        .mount(&mock_server)
        .await;

    let client = EmbeddingClient::new(
        "test-key".to_string(),
        EmbeddingProvider::OpenAI,
        EmbeddingModel::Small,
        Some(cache.clone()),
    );

    // First call - should hit API
    let result1 = client.generate("cached query").await;
    assert!(result1.is_ok());

    // Second call - should hit cache
    let result2 = client.generate("cached query").await;
    assert!(result2.is_ok());

    // Verify both return same result
    assert_eq!(result1.unwrap(), result2.unwrap());

    // Cleanup
    let _ = cache.clear_embedding_cache().await;
}

/// Test local provider fallback
#[tokio::test]
async fn test_local_provider_fallback() {
    let client = EmbeddingClient::new(
        String::new(),
        EmbeddingProvider::Local,
        EmbeddingModel::Small,
        None,
    );

    let result = client.generate("test").await;
    assert!(result.is_ok());

    let embedding = result.unwrap();
    assert_eq!(embedding.len(), 768);
    assert!(embedding.iter().all(|&v| v == 0.0));
}

/// Test model dimensions
#[test]
fn test_model_dimensions() {
    let small = EmbeddingModel::Small;
    assert_eq!(small.dimension(), 768);
    assert_eq!(small.name(), "text-embedding-3-small");

    let large = EmbeddingModel::Large;
    assert_eq!(large.dimension(), 1536);
    assert_eq!(large.name(), "text-embedding-3-large");
}

/// Test provider configuration from env
#[test]
fn test_provider_from_env() {
    std::env::set_var("EMBEDDING_PROVIDER", "openai");
    assert_eq!(EmbeddingProvider::from_env(), EmbeddingProvider::OpenAI);

    std::env::set_var("EMBEDDING_PROVIDER", "local");
    assert_eq!(EmbeddingProvider::from_env(), EmbeddingProvider::Local);

    std::env::remove_var("EMBEDDING_PROVIDER");
    assert_eq!(EmbeddingProvider::from_env(), EmbeddingProvider::OpenAI);
}

/// Test model configuration from env
#[test]
fn test_model_from_env() {
    std::env::set_var("EMBEDDING_MODEL", "small");
    assert_eq!(EmbeddingModel::from_env().dimension(), 768);

    std::env::set_var("EMBEDDING_MODEL", "large");
    assert_eq!(EmbeddingModel::from_env().dimension(), 1536);

    std::env::remove_var("EMBEDDING_MODEL");
    assert_eq!(EmbeddingModel::from_env().dimension(), 768);
}

/// Test retry mechanism with exponential backoff
#[tokio::test]
async fn test_retry_with_exponential_backoff() {
    let mock_server = MockServer::start().await;

    let success_body = serde_json::json!({
        "data": [{
            "embedding": vec![0.1, 0.2, 0.3],
            "index": 0
        }],
        "model": "text-embedding-3-small",
        "usage": {
            "prompt_tokens": 5,
            "total_tokens": 5
        }
    });

    // Fail twice, then succeed
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_body))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = EmbeddingClient::new(
        "test-key".to_string(),
        EmbeddingProvider::OpenAI,
        EmbeddingModel::Small,
        None,
    );

    let result = client.generate("test").await;
    assert!(result.is_ok());
}

/// Test batch with mixed cache hits and misses
#[tokio::test]
async fn test_batch_with_partial_cache() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_config = Arc::new(CacheConfig {
        redis_url,
        search_ttl_sec: 60,
        embedding_ttl_sec: 86400,
        intent_ttl_sec: 30,
    });

    let cache = match RedisCache::new(cache_config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    // Clear cache
    let _ = cache.clear_embedding_cache().await;

    // Pre-cache one embedding
    let _ = cache
        .cache_embedding(&"cached".to_string(), &vec![1.0, 1.0, 1.0])
        .await;

    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "data": [{
            "embedding": vec![2.0, 2.0, 2.0],
            "index": 0
        }],
        "model": "text-embedding-3-small",
        "usage": {
            "prompt_tokens": 5,
            "total_tokens": 5
        }
    });

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .expect(1) // Only uncached item
        .mount(&mock_server)
        .await;

    let client = EmbeddingClient::new(
        "test-key".to_string(),
        EmbeddingProvider::OpenAI,
        EmbeddingModel::Small,
        Some(cache.clone()),
    );

    let texts = vec!["cached".to_string(), "uncached".to_string()];
    let result = client.generate_batch(&texts).await;
    assert!(result.is_ok());

    let embeddings = result.unwrap();
    assert_eq!(embeddings.len(), 2);
    assert_eq!(embeddings[0], vec![1.0, 1.0, 1.0]); // From cache
    assert_eq!(embeddings[1], vec![2.0, 2.0, 2.0]); // From API

    // Cleanup
    let _ = cache.clear_embedding_cache().await;
}

/// Test empty batch handling
#[tokio::test]
async fn test_empty_batch() {
    let client = EmbeddingClient::new(
        "test-key".to_string(),
        EmbeddingProvider::OpenAI,
        EmbeddingModel::Small,
        None,
    );

    let texts: Vec<String> = vec![];
    let result = client.generate_batch(&texts).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

/// Test cache clearing
#[tokio::test]
async fn test_cache_clearing() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let cache_config = Arc::new(CacheConfig {
        redis_url,
        search_ttl_sec: 60,
        embedding_ttl_sec: 86400,
        intent_ttl_sec: 30,
    });

    let cache = match RedisCache::new(cache_config).await {
        Ok(c) => Arc::new(c),
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let client = EmbeddingClient::new(
        "test-key".to_string(),
        EmbeddingProvider::OpenAI,
        EmbeddingModel::Small,
        Some(cache.clone()),
    );

    // Add some cache entries
    let _ = cache
        .cache_embedding(&"test1".to_string(), &vec![1.0])
        .await;
    let _ = cache
        .cache_embedding(&"test2".to_string(), &vec![2.0])
        .await;

    // Clear cache
    let result = client.clear_cache().await;
    assert!(result.is_ok());

    // Verify cache is empty
    let cached = cache.get_embedding(&"test1".to_string()).await.unwrap();
    assert!(cached.is_none());
}
