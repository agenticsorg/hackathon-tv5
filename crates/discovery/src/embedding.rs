//! Embedding Service for semantic search with multiple providers

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::cache::RedisCache;

const OPENAI_API_URL: &str = "https://api.openai.com/v1/embeddings";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 100;
const CACHE_TTL_SEC: u64 = 86400; // 24 hours

/// Embedding provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingProvider {
    OpenAI,
    Local,
}

impl EmbeddingProvider {
    pub fn from_env() -> Self {
        match std::env::var("EMBEDDING_PROVIDER")
            .unwrap_or_else(|_| "openai".to_string())
            .to_lowercase()
            .as_str()
        {
            "local" => Self::Local,
            _ => Self::OpenAI,
        }
    }
}

/// Embedding model configuration
#[derive(Debug, Clone)]
pub enum EmbeddingModel {
    Small, // text-embedding-3-small (768 dims)
    Large, // text-embedding-3-large (1536 dims)
}

impl EmbeddingModel {
    pub fn name(&self) -> &str {
        match self {
            Self::Small => "text-embedding-3-small",
            Self::Large => "text-embedding-3-large",
        }
    }

    pub fn dimension(&self) -> usize {
        match self {
            Self::Small => 768,
            Self::Large => 1536,
        }
    }

    pub fn from_env() -> Self {
        match std::env::var("EMBEDDING_MODEL")
            .unwrap_or_else(|_| "small".to_string())
            .to_lowercase()
            .as_str()
        {
            "large" => Self::Large,
            _ => Self::Small,
        }
    }
}

/// OpenAI embedding request
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    input: EmbeddingInput,
    model: String,
    dimensions: Option<usize>,
}

/// Embedding input (single or batch)
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum EmbeddingInput {
    Single(String),
    Batch(Vec<String>),
}

/// OpenAI embedding response
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    data: Vec<EmbeddingData>,
    model: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    total_tokens: u32,
}

/// OpenAI error response
#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: ApiError,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
}

/// Embedding client with caching and multiple provider support
#[derive(Clone)]
pub struct EmbeddingClient {
    http_client: Client,
    api_key: String,
    provider: EmbeddingProvider,
    model: EmbeddingModel,
    cache: Option<Arc<RedisCache>>,
}

impl EmbeddingClient {
    /// Create new embedding client
    pub fn new(
        api_key: String,
        provider: EmbeddingProvider,
        model: EmbeddingModel,
        cache: Option<Arc<RedisCache>>,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("Failed to create HTTP client");

        info!(
            provider = ?provider,
            model = model.name(),
            dimension = model.dimension(),
            cache_enabled = cache.is_some(),
            "Initialized embedding client"
        );

        Self {
            http_client,
            api_key,
            provider,
            model,
            cache,
        }
    }

    /// Create from environment variables
    pub fn from_env(cache: Option<Arc<RedisCache>>) -> Result<Self> {
        let provider = EmbeddingProvider::from_env();
        let model = EmbeddingModel::from_env();

        let api_key = match provider {
            EmbeddingProvider::OpenAI => std::env::var("OPENAI_API_KEY")
                .map_err(|_| anyhow!("OPENAI_API_KEY environment variable not set"))?,
            EmbeddingProvider::Local => String::new(),
        };

        Ok(Self::new(api_key, provider, model, cache))
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.model.dimension()
    }

    /// Generate embedding for single text with caching
    pub async fn generate(&self, text: &str) -> Result<Vec<f32>> {
        // Check Redis cache first
        if let Some(cache) = &self.cache {
            match cache.get_embedding(&text).await {
                Ok(Some(embedding)) => {
                    debug!("Redis cache hit for embedding");
                    return Ok(embedding);
                }
                Ok(None) => {
                    debug!("Redis cache miss for embedding");
                }
                Err(e) => {
                    warn!("Redis cache error: {}, proceeding without cache", e);
                }
            }
        }

        // Generate embedding
        let embedding = match self.provider {
            EmbeddingProvider::OpenAI => self.generate_openai_single(text).await?,
            EmbeddingProvider::Local => self.generate_local_single(text).await?,
        };

        // Cache the result
        if let Some(cache) = &self.cache {
            if let Err(e) = cache.cache_embedding(&text, &embedding).await {
                warn!("Failed to cache embedding: {}", e);
            }
        }

        Ok(embedding)
    }

    /// Generate embeddings for multiple texts (batch)
    pub async fn generate_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Check cache for all texts
        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_indices = Vec::new();
        let mut uncached_texts = Vec::new();

        if let Some(cache) = &self.cache {
            for (i, text) in texts.iter().enumerate() {
                match cache.get_embedding(text).await {
                    Ok(Some(embedding)) => {
                        results.push(Some(embedding));
                    }
                    _ => {
                        results.push(None);
                        uncached_indices.push(i);
                        uncached_texts.push(text.clone());
                    }
                }
            }
        } else {
            // No cache, need to generate all
            uncached_indices = (0..texts.len()).collect();
            uncached_texts = texts.to_vec();
            results = vec![None; texts.len()];
        }

        // Generate embeddings for uncached texts
        if !uncached_texts.is_empty() {
            let embeddings = match self.provider {
                EmbeddingProvider::OpenAI => self.generate_openai_batch(&uncached_texts).await?,
                EmbeddingProvider::Local => self.generate_local_batch(&uncached_texts).await?,
            };

            // Fill in results and cache
            for (i, embedding) in uncached_indices.iter().zip(embeddings.iter()) {
                results[*i] = Some(embedding.clone());

                // Cache the embedding
                if let Some(cache) = &self.cache {
                    if let Err(e) = cache.cache_embedding(&uncached_texts[*i], embedding).await {
                        warn!("Failed to cache embedding: {}", e);
                    }
                }
            }
        }

        // Convert Option<Vec<f32>> to Vec<f32>
        results
            .into_iter()
            .collect::<Option<Vec<Vec<f32>>>>()
            .ok_or_else(|| anyhow!("Missing embeddings in batch result"))
    }

    /// Generate single embedding via OpenAI
    async fn generate_openai_single(&self, text: &str) -> Result<Vec<f32>> {
        let mut last_error = None;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        for attempt in 1..=MAX_RETRIES {
            match self.call_openai_api(&[text.to_string()]).await {
                Ok(mut embeddings) => {
                    return embeddings
                        .pop()
                        .ok_or_else(|| anyhow!("Empty embedding response"));
                }
                Err(e) => {
                    warn!(
                        attempt,
                        backoff_ms,
                        error = %e,
                        "Embedding API call failed, retrying"
                    );
                    last_error = Some(e);

                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms *= 2;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Embedding failed after {} retries", MAX_RETRIES)))
    }

    /// Generate batch embeddings via OpenAI
    async fn generate_openai_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut last_error = None;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        for attempt in 1..=MAX_RETRIES {
            match self.call_openai_api(texts).await {
                Ok(embeddings) => {
                    return Ok(embeddings);
                }
                Err(e) => {
                    warn!(
                        attempt,
                        backoff_ms,
                        batch_size = texts.len(),
                        error = %e,
                        "Batch embedding API call failed, retrying"
                    );
                    last_error = Some(e);

                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms *= 2;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| anyhow!("Batch embedding failed after {} retries", MAX_RETRIES)))
    }

    /// Call OpenAI API
    async fn call_openai_api(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let input = if texts.len() == 1 {
            EmbeddingInput::Single(texts[0].clone())
        } else {
            EmbeddingInput::Batch(texts.to_vec())
        };

        let request = OpenAIRequest {
            input,
            model: self.model.name().to_string(),
            dimensions: Some(self.model.dimension()),
        };

        debug!(
            model = self.model.name(),
            count = texts.len(),
            "Calling OpenAI embedding API"
        );

        let response = self
            .http_client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                return Err(anyhow!(
                    "OpenAI API error ({}): {} - {}",
                    status,
                    error_response.error.error_type,
                    error_response.error.message
                ));
            }
            return Err(anyhow!("OpenAI API error ({}): {}", status, error_text));
        }

        let embedding_response: OpenAIResponse = response.json().await?;

        if embedding_response.data.is_empty() {
            return Err(anyhow!("Empty embedding response from OpenAI"));
        }

        debug!(
            tokens = embedding_response.usage.total_tokens,
            count = embedding_response.data.len(),
            "Received embeddings from OpenAI"
        );

        // Sort by index to ensure correct order
        let mut data = embedding_response.data;
        data.sort_by_key(|d| d.index);

        Ok(data.into_iter().map(|d| d.embedding).collect())
    }

    /// Generate single embedding via local model (stub)
    async fn generate_local_single(&self, text: &str) -> Result<Vec<f32>> {
        warn!("Local embedding model not implemented, returning zero vector");
        Ok(vec![0.0; self.model.dimension()])
    }

    /// Generate batch embeddings via local model (stub)
    async fn generate_local_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        warn!("Local embedding model not implemented, returning zero vectors");
        Ok(vec![vec![0.0; self.model.dimension()]; texts.len()])
    }

    /// Clear Redis cache
    pub async fn clear_cache(&self) -> Result<()> {
        if let Some(cache) = &self.cache {
            cache.clear_embedding_cache().await?;
            info!("Cleared embedding cache");
        }
        Ok(())
    }
}

/// Legacy wrapper for compatibility
#[derive(Clone)]
pub struct EmbeddingService {
    client: EmbeddingClient,
}

impl EmbeddingService {
    pub fn new(api_key: String) -> Self {
        let client = EmbeddingClient::new(
            api_key,
            EmbeddingProvider::OpenAI,
            EmbeddingModel::Small,
            None,
        );
        Self { client }
    }

    pub fn from_env() -> Result<Self> {
        let client = EmbeddingClient::from_env(None)?;
        Ok(Self { client })
    }

    pub fn dimension(&self) -> usize {
        self.client.dimension()
    }

    pub async fn generate(&self, text: &str) -> Result<Vec<f32>> {
        self.client.generate(text).await
    }

    pub async fn generate_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let texts_owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        self.client.generate_batch(&texts_owned).await
    }

    pub async fn clear_cache(&self) -> Result<()> {
        self.client.clear_cache().await
    }

    pub async fn cache_size(&self) -> usize {
        0 // No longer applicable with Redis cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_provider_from_env() {
        std::env::set_var("EMBEDDING_PROVIDER", "openai");
        assert_eq!(EmbeddingProvider::from_env(), EmbeddingProvider::OpenAI);

        std::env::set_var("EMBEDDING_PROVIDER", "local");
        assert_eq!(EmbeddingProvider::from_env(), EmbeddingProvider::Local);

        std::env::remove_var("EMBEDDING_PROVIDER");
        assert_eq!(EmbeddingProvider::from_env(), EmbeddingProvider::OpenAI);
    }

    #[test]
    fn test_embedding_model_from_env() {
        std::env::set_var("EMBEDDING_MODEL", "small");
        let model = EmbeddingModel::from_env();
        assert_eq!(model.dimension(), 768);
        assert_eq!(model.name(), "text-embedding-3-small");

        std::env::set_var("EMBEDDING_MODEL", "large");
        let model = EmbeddingModel::from_env();
        assert_eq!(model.dimension(), 1536);
        assert_eq!(model.name(), "text-embedding-3-large");

        std::env::remove_var("EMBEDDING_MODEL");
        let model = EmbeddingModel::from_env();
        assert_eq!(model.dimension(), 768);
    }

    #[test]
    fn test_embedding_client_creation() {
        let client = EmbeddingClient::new(
            "test-key".to_string(),
            EmbeddingProvider::OpenAI,
            EmbeddingModel::Small,
            None,
        );
        assert_eq!(client.dimension(), 768);
    }

    #[test]
    fn test_openai_request_serialization() {
        let request = OpenAIRequest {
            input: EmbeddingInput::Single("test query".to_string()),
            model: "text-embedding-3-small".to_string(),
            dimensions: Some(768),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("text-embedding-3-small"));
        assert!(json.contains("768"));
        assert!(json.contains("test query"));
    }

    #[test]
    fn test_openai_batch_request_serialization() {
        let request = OpenAIRequest {
            input: EmbeddingInput::Batch(vec!["query1".to_string(), "query2".to_string()]),
            model: "text-embedding-3-small".to_string(),
            dimensions: Some(768),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("query1"));
        assert!(json.contains("query2"));
    }

    #[test]
    fn test_legacy_service_creation() {
        let service = EmbeddingService::new("test-key".to_string());
        assert_eq!(service.dimension(), 768);
    }
}
