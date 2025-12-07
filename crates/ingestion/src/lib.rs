//! Media Gateway Ingestion Pipeline
//!
//! This crate provides the data ingestion pipeline for the Media Gateway platform,
//! including platform normalizers, entity resolution, and content enrichment.

pub mod aggregator;
pub mod deep_link;
pub mod embedding;
pub mod entity_resolution;
pub mod events;
pub mod genre_mapping;
pub mod normalizer;
pub mod notifications;
pub mod pipeline;
pub mod qdrant;
pub mod quality;
pub mod rate_limit;
pub mod repository;
pub mod webhooks;

// Re-export main types
pub use deep_link::{DeepLinkGenerator, DeepLinkResult};
pub use embedding::EmbeddingGenerator;
pub use entity_resolution::EntityResolver;
pub use events::{
    AvailabilityChangedEvent, ContentEvent, ContentIngestedEvent, ContentUpdatedEvent, EventError,
    EventProducer, EventResult, KafkaEventProducer, MetadataEnrichedEvent,
};
pub use genre_mapping::GenreMapper;
pub use normalizer::PlatformNormalizer;
pub use notifications::{
    ContentExpiringEvent, ExpirationNotificationConfig, ExpirationNotificationJob,
    NotificationStatus, NotificationWindow,
};
pub use pipeline::{IngestionPipeline, IngestionSchedule};
pub use qdrant::{to_content_point, ContentPayload, ContentPoint, QdrantClient, VECTOR_DIM};
pub use quality::{
    batch_score_content, generate_quality_report, FreshnessDecay, LowQualityItem, QualityReport,
    QualityScorer, QualityWeights, RecalculationError, RecalculationJob, RecalculationReport,
};
pub use rate_limit::RateLimitManager;
pub use repository::{
    ContentRepository, ExpiringContent, LowQualityContentItem, PostgresContentRepository,
    StaleContent,
};
pub use webhooks::{
    PlatformWebhookConfig, ProcessedWebhook, ProcessingStatus, QueueStats, RedisWebhookQueue,
    WebhookDeduplicator, WebhookEventType, WebhookHandler, WebhookMetrics, WebhookPayload,
    WebhookProcessor, WebhookQueue, WebhookReceiver,
};

/// Common error type for the ingestion pipeline
#[derive(Debug, thiserror::Error)]
pub enum IngestionError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Rate limit exceeded for {platform}")]
    RateLimitExceeded { platform: String },

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("Entity resolution failed: {0}")]
    EntityResolutionFailed(String),

    #[error("Normalization failed: {0}")]
    NormalizationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Webhook error: {0}")]
    WebhookError(String),

    #[error("External service error: {0}")]
    External(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, IngestionError>;
pub type Error = IngestionError;
