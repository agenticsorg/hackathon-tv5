//! Main ingestion pipeline with scheduling and orchestration

use crate::{
    embedding::EmbeddingGenerator,
    entity_resolution::EntityResolver,
    genre_mapping::GenreMapper,
    normalizer::{CanonicalContent, PlatformNormalizer, RawContent},
    qdrant::{to_content_point, QdrantClient},
    rate_limit::RateLimitManager,
    repository::{ContentRepository, PostgresContentRepository},
    IngestionError, Result,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Ingestion schedule configuration
#[derive(Debug, Clone)]
pub struct IngestionSchedule {
    /// Full catalog refresh interval (6 hours)
    pub catalog_refresh: Duration,
    /// Availability sync interval (1 hour)
    pub availability_sync: Duration,
    /// Expiring content check interval (15 minutes)
    pub expiring_content: Duration,
    /// Metadata enrichment interval (24 hours)
    pub metadata_enrichment: Duration,
}

impl Default for IngestionSchedule {
    fn default() -> Self {
        Self {
            catalog_refresh: Duration::from_secs(6 * 3600),
            availability_sync: Duration::from_secs(3600),
            expiring_content: Duration::from_secs(900),
            metadata_enrichment: Duration::from_secs(24 * 3600),
        }
    }
}

/// Main ingestion pipeline orchestrator
pub struct IngestionPipeline {
    normalizers: Vec<Arc<dyn PlatformNormalizer>>,
    entity_resolver: Arc<EntityResolver>,
    genre_mapper: Arc<GenreMapper>,
    embedding_generator: Arc<EmbeddingGenerator>,
    qdrant_client: Option<Arc<QdrantClient>>,
    rate_limiter: Arc<RateLimitManager>,
    repository: Arc<dyn ContentRepository>,
    event_producer: Option<Arc<dyn crate::events::EventProducer>>,
    schedule: IngestionSchedule,
    regions: Vec<String>,
}

impl IngestionPipeline {
    /// Create a new ingestion pipeline
    pub fn new(
        normalizers: Vec<Arc<dyn PlatformNormalizer>>,
        entity_resolver: EntityResolver,
        genre_mapper: GenreMapper,
        embedding_generator: EmbeddingGenerator,
        rate_limiter: RateLimitManager,
        pool: PgPool,
        schedule: IngestionSchedule,
        regions: Vec<String>,
    ) -> Self {
        Self {
            normalizers,
            entity_resolver: Arc::new(entity_resolver),
            genre_mapper: Arc::new(genre_mapper),
            embedding_generator: Arc::new(embedding_generator),
            qdrant_client: None,
            rate_limiter: Arc::new(rate_limiter),
            repository: Arc::new(PostgresContentRepository::new(pool)),
            event_producer: None,
            schedule,
            regions,
        }
    }

    /// Set the Qdrant client for vector indexing
    ///
    /// # Arguments
    /// * `qdrant_client` - Optional Qdrant client for storing embeddings
    pub fn with_qdrant(mut self, qdrant_client: Option<QdrantClient>) -> Self {
        self.qdrant_client = qdrant_client.map(Arc::new);
        self
    }

    /// Set the event producer for Kafka events
    ///
    /// # Arguments
    /// * `event_producer` - Optional event producer for publishing events
    pub fn with_event_producer(
        mut self,
        event_producer: Option<Arc<dyn crate::events::EventProducer>>,
    ) -> Self {
        self.event_producer = event_producer;
        self
    }

    /// Start the ingestion pipeline with all scheduled tasks
    pub async fn start(&self) -> Result<()> {
        info!(
            "Starting ingestion pipeline with {} platforms",
            self.normalizers.len()
        );

        // Spawn concurrent tasks for different schedules
        let catalog_handle = self.spawn_catalog_refresh_task();
        let availability_handle = self.spawn_availability_sync_task();
        let expiring_handle = self.spawn_expiring_content_task();
        let enrichment_handle = self.spawn_metadata_enrichment_task();

        // Wait for all tasks (they run indefinitely)
        tokio::select! {
            result = catalog_handle => {
                error!("Catalog refresh task terminated: {:?}", result);
            }
            result = availability_handle => {
                error!("Availability sync task terminated: {:?}", result);
            }
            result = expiring_handle => {
                error!("Expiring content task terminated: {:?}", result);
            }
            result = enrichment_handle => {
                error!("Metadata enrichment task terminated: {:?}", result);
            }
        }

        Ok(())
    }

    /// Spawn catalog refresh task (every 6 hours)
    fn spawn_catalog_refresh_task(&self) -> tokio::task::JoinHandle<()> {
        let normalizers = self.normalizers.clone();
        let entity_resolver = self.entity_resolver.clone();
        let genre_mapper = self.genre_mapper.clone();
        let embedding_generator = self.embedding_generator.clone();
        let qdrant_client = self.qdrant_client.clone();
        let rate_limiter = self.rate_limiter.clone();
        let repository = self.repository.clone();
        let regions = self.regions.clone();
        let schedule_duration = self.schedule.catalog_refresh;

        tokio::spawn(async move {
            let mut interval = interval(schedule_duration);
            loop {
                interval.tick().await;
                info!("Starting catalog refresh cycle");

                for normalizer in &normalizers {
                    for region in &regions {
                        if let Err(e) = Self::process_catalog_refresh(
                            normalizer.clone(),
                            &entity_resolver,
                            &genre_mapper,
                            &embedding_generator,
                            qdrant_client.as_ref().map(|c| c.as_ref()),
                            &rate_limiter,
                            repository.as_ref(),
                            region,
                        )
                        .await
                        {
                            error!(
                                "Catalog refresh failed for {} in {}: {}",
                                normalizer.platform_id(),
                                region,
                                e
                            );
                        }
                    }
                }

                info!("Catalog refresh cycle completed");
            }
        })
    }

    /// Spawn availability sync task (every 1 hour)
    fn spawn_availability_sync_task(&self) -> tokio::task::JoinHandle<()> {
        let normalizers = self.normalizers.clone();
        let rate_limiter = self.rate_limiter.clone();
        let repository = self.repository.clone();
        let event_producer = self.event_producer.clone();
        let regions = self.regions.clone();
        let schedule_duration = self.schedule.availability_sync;

        tokio::spawn(async move {
            let mut interval = interval(schedule_duration);
            loop {
                interval.tick().await;
                info!("Starting availability sync cycle");

                for normalizer in &normalizers {
                    for region in &regions {
                        if let Err(e) = Self::sync_availability(
                            normalizer.clone(),
                            &rate_limiter,
                            repository.as_ref(),
                            event_producer.as_ref(),
                            region,
                        )
                        .await
                        {
                            error!(
                                "Availability sync failed for {} in {}: {}",
                                normalizer.platform_id(),
                                region,
                                e
                            );
                        }
                    }
                }

                info!("Availability sync cycle completed");
            }
        })
    }

    /// Spawn expiring content check task (every 15 minutes)
    fn spawn_expiring_content_task(&self) -> tokio::task::JoinHandle<()> {
        let normalizers = self.normalizers.clone();
        let rate_limiter = self.rate_limiter.clone();
        let repository = self.repository.clone();
        let regions = self.regions.clone();
        let schedule_duration = self.schedule.expiring_content;

        tokio::spawn(async move {
            let mut interval = interval(schedule_duration);
            loop {
                interval.tick().await;
                debug!("Checking expiring content");

                for normalizer in &normalizers {
                    for region in &regions {
                        if let Err(e) = Self::check_expiring_content(
                            normalizer.clone(),
                            &rate_limiter,
                            repository.as_ref(),
                            region,
                        )
                        .await
                        {
                            warn!(
                                "Expiring content check failed for {} in {}: {}",
                                normalizer.platform_id(),
                                region,
                                e
                            );
                        }
                    }
                }
            }
        })
    }

    /// Spawn metadata enrichment task (every 24 hours)
    fn spawn_metadata_enrichment_task(&self) -> tokio::task::JoinHandle<()> {
        let embedding_generator = self.embedding_generator.clone();
        let qdrant_client = self.qdrant_client.clone();
        let repository = self.repository.clone();
        let event_producer = self.event_producer.clone();
        let schedule_duration = self.schedule.metadata_enrichment;

        tokio::spawn(async move {
            let mut interval = interval(schedule_duration);
            loop {
                interval.tick().await;
                info!("Starting metadata enrichment cycle");

                if let Err(e) = Self::enrich_metadata(
                    &embedding_generator,
                    qdrant_client.as_ref().map(|c| c.as_ref()),
                    repository.as_ref(),
                    event_producer.as_ref(),
                )
                .await
                {
                    error!("Metadata enrichment failed: {}", e);
                }

                info!("Metadata enrichment cycle completed");
            }
        })
    }

    /// Process full catalog refresh for a platform/region
    async fn process_catalog_refresh(
        normalizer: Arc<dyn PlatformNormalizer>,
        entity_resolver: &EntityResolver,
        genre_mapper: &GenreMapper,
        embedding_generator: &EmbeddingGenerator,
        qdrant_client: Option<&QdrantClient>,
        rate_limiter: &RateLimitManager,
        repository: &dyn ContentRepository,
        region: &str,
    ) -> Result<()> {
        let platform_id = normalizer.platform_id();
        info!("Fetching catalog delta for {} in {}", platform_id, region);

        // Check rate limit
        rate_limiter.check_and_wait(platform_id).await?;

        // Calculate "since" timestamp (last successful run or 7 days ago)
        let since = Utc::now() - ChronoDuration::days(7);

        // Fetch catalog delta
        let raw_items = normalizer.fetch_catalog_delta(since, region).await?;
        info!(
            "Fetched {} items from {} for {}",
            raw_items.len(),
            platform_id,
            region
        );

        // Process items in batches for performance (target: 500 items/s)
        const BATCH_SIZE: usize = 100;
        for batch in raw_items.chunks(BATCH_SIZE) {
            Self::process_batch(
                batch,
                normalizer.as_ref(),
                entity_resolver,
                genre_mapper,
                embedding_generator,
                qdrant_client,
                repository,
            )
            .await?;
        }

        Ok(())
    }

    /// Process a batch of raw content items
    async fn process_batch(
        batch: &[RawContent],
        normalizer: &dyn PlatformNormalizer,
        entity_resolver: &EntityResolver,
        genre_mapper: &GenreMapper,
        embedding_generator: &EmbeddingGenerator,
        qdrant_client: Option<&QdrantClient>,
        repository: &dyn ContentRepository,
    ) -> Result<()> {
        let mut qdrant_points = Vec::new();

        for raw in batch {
            // Normalize to canonical format
            let mut canonical = normalizer
                .normalize(raw.clone())
                .map_err(|e| IngestionError::NormalizationFailed(e.to_string()))?;

            // Resolve entity (EIDR, external IDs, fuzzy matching)
            let entity_match = entity_resolver
                .resolve(&canonical)
                .await
                .map_err(|e| IngestionError::EntityResolutionFailed(e.to_string()))?;

            if let Some(matched_entity_id) = entity_match.entity_id {
                canonical.entity_id = Some(matched_entity_id);
            }

            // Map genres to canonical taxonomy
            canonical.genres = genre_mapper.map_genres(&canonical.genres, normalizer.platform_id());

            // Generate embeddings
            canonical.embedding = Some(
                embedding_generator
                    .generate(&canonical)
                    .await
                    .map_err(|e| IngestionError::NormalizationFailed(e.to_string()))?,
            );

            // Persist to database
            let content_id = repository.upsert(&canonical).await.map_err(|e| {
                IngestionError::NormalizationFailed(format!("Database error: {}", e))
            })?;

            debug!(
                "Processed and persisted content: {} (id: {}, entity: {:?})",
                canonical.title, content_id, canonical.entity_id
            );

            // Prepare for Qdrant batch upsert if client is available
            if qdrant_client.is_some() {
                match to_content_point(&canonical, content_id) {
                    Ok(point) => qdrant_points.push(point),
                    Err(e) => warn!("Failed to create Qdrant point for {}: {}", content_id, e),
                }
            }
        }

        // Batch upsert to Qdrant after DB persistence
        if let Some(client) = qdrant_client {
            if !qdrant_points.is_empty() {
                client.upsert_batch(qdrant_points).await?;
            }
        }

        Ok(())
    }

    /// Sync availability data (pricing, subscription status)
    async fn sync_availability(
        normalizer: Arc<dyn PlatformNormalizer>,
        rate_limiter: &RateLimitManager,
        repository: &dyn ContentRepository,
        event_producer: Option<&Arc<dyn crate::events::EventProducer>>,
        region: &str,
    ) -> Result<()> {
        let platform_id = normalizer.platform_id();
        debug!("Syncing availability for {} in {}", platform_id, region);

        rate_limiter.check_and_wait(platform_id).await?;

        // Fetch recent availability updates (last hour)
        let since = Utc::now() - ChronoDuration::hours(1);
        let raw_items = normalizer.fetch_catalog_delta(since, region).await?;

        let items_count = raw_items.len();

        // Process each raw item to extract and update availability
        for raw in raw_items {
            // Normalize to extract availability information
            let canonical = match normalizer.normalize(raw) {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to normalize content for availability sync: {}", e);
                    continue;
                }
            };

            // Look up content by platform_content_id and platform_id
            let content_id = match repository
                .find_by_platform_id(&canonical.platform_content_id, &canonical.platform_id)
                .await
            {
                Ok(Some(id)) => id,
                Ok(None) => {
                    debug!(
                        "Content not found for platform_id={} content_id={}",
                        canonical.platform_id, canonical.platform_content_id
                    );
                    continue;
                }
                Err(e) => {
                    warn!("Failed to lookup content: {}", e);
                    continue;
                }
            };

            let availability = &canonical.availability;

            // Determine availability status
            let is_available = !availability.regions.is_empty();

            // Update availability in database
            if let Err(e) = repository
                .update_availability(
                    content_id,
                    &canonical.platform_id,
                    region,
                    is_available,
                    availability.available_until,
                )
                .await
            {
                warn!(
                    "Failed to update availability for content {}: {}",
                    content_id, e
                );
                continue;
            }

            // Emit AvailabilityChangedEvent via Kafka if producer is available
            if let Some(producer) = event_producer {
                use crate::events::{AvailabilityChangedEvent, ContentEvent};

                let mut event = AvailabilityChangedEvent::new(
                    content_id,
                    canonical.platform_id.clone(),
                    is_available,
                    false, // We don't track previous state in this context
                    availability.regions.clone(),
                );

                if let Some(expires_at) = availability.available_until {
                    event = event.with_expiration(expires_at);
                }

                if let Err(e) = producer
                    .publish_event(ContentEvent::AvailabilityChanged(event))
                    .await
                {
                    warn!("Failed to publish availability changed event: {}", e);
                }
            }
        }

        debug!(
            "Updated availability for {} items from {}",
            items_count, platform_id
        );

        Ok(())
    }

    /// Check for expiring content (leaving platforms soon)
    async fn check_expiring_content(
        normalizer: Arc<dyn PlatformNormalizer>,
        rate_limiter: &RateLimitManager,
        repository: &dyn ContentRepository,
        region: &str,
    ) -> Result<()> {
        let platform_id = normalizer.platform_id();

        rate_limiter.check_and_wait(platform_id).await?;

        // Query database for content expiring in next 7 days
        let expiring = repository
            .find_expiring_within(ChronoDuration::days(7))
            .await
            .map_err(|e| IngestionError::NormalizationFailed(e.to_string()))?;

        for item in expiring {
            if item.platform == platform_id && item.region == region {
                info!(
                    "Content '{}' on {} expires at {}",
                    item.title, item.platform, item.expires_at
                );
            }
        }

        Ok(())
    }

    /// Enrich metadata with updated embeddings
    async fn enrich_metadata(
        embedding_generator: &EmbeddingGenerator,
        qdrant_client: Option<&QdrantClient>,
        repository: &dyn ContentRepository,
        event_producer: Option<&Arc<dyn crate::events::EventProducer>>,
    ) -> Result<()> {
        // Query for content with stale embeddings (older than 7 days)
        let stale_threshold = Utc::now() - ChronoDuration::days(7);
        let stale_content = repository
            .find_stale_embeddings(stale_threshold)
            .await
            .map_err(|e| IngestionError::DatabaseError(e.to_string()))?;

        if stale_content.is_empty() {
            info!("No stale content found for enrichment");
            return Ok(());
        }

        info!(
            "Found {} content items with stale embeddings",
            stale_content.len()
        );

        // Process in batches of 100 items
        const BATCH_SIZE: usize = 100;
        let mut total_enriched = 0;
        let mut total_quality_computed = 0;

        for (batch_idx, batch) in stale_content.chunks(BATCH_SIZE).enumerate() {
            info!(
                "Processing enrichment batch {}/{} ({} items)",
                batch_idx + 1,
                (stale_content.len() + BATCH_SIZE - 1) / BATCH_SIZE,
                batch.len()
            );

            let mut qdrant_points = Vec::new();
            let mut events = Vec::new();

            for item in batch {
                // Regenerate embedding
                match embedding_generator.generate(&item.content).await {
                    Ok(embedding) => {
                        // Update embedding in database
                        if let Err(e) = repository
                            .update_embedding(item.content_id, &embedding)
                            .await
                        {
                            warn!(
                                "Failed to update embedding for content {}: {}",
                                item.content_id, e
                            );
                            continue;
                        }

                        // Prepare Qdrant point for batch upsert
                        if qdrant_client.is_some() {
                            let mut content_with_embedding = item.content.clone();
                            content_with_embedding.embedding = Some(embedding.clone());

                            match to_content_point(&content_with_embedding, item.content_id) {
                                Ok(point) => qdrant_points.push(point),
                                Err(e) => warn!(
                                    "Failed to create Qdrant point for {}: {}",
                                    item.content_id, e
                                ),
                            }
                        }

                        // Compute quality score
                        let quality_score = Self::compute_quality_score(&item.content);
                        if let Err(e) = repository
                            .update_quality_score(item.content_id, quality_score)
                            .await
                        {
                            warn!(
                                "Failed to update quality score for content {}: {}",
                                item.content_id, e
                            );
                        } else {
                            total_quality_computed += 1;
                        }

                        // Create metadata enrichment event
                        let enriched_fields =
                            vec!["embedding".to_string(), "quality_score".to_string()];

                        events.push(crate::events::ContentEvent::MetadataEnriched(
                            crate::events::MetadataEnrichedEvent::new(
                                item.content_id,
                                "embedding_generator".to_string(),
                                enriched_fields,
                                quality_score,
                            ),
                        ));

                        total_enriched += 1;
                        debug!(
                            "Enriched metadata for content: {} ({})",
                            item.content.title, item.content_id
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to generate embedding for content {}: {}",
                            item.content_id, e
                        );
                        continue;
                    }
                }
            }

            // Batch upsert to Qdrant
            if let Some(client) = qdrant_client {
                if !qdrant_points.is_empty() {
                    if let Err(e) = client.upsert_batch(qdrant_points).await {
                        error!("Failed to upsert batch to Qdrant: {}", e);
                    } else {
                        debug!("Updated {} vectors in Qdrant", batch.len());
                    }
                }
            }

            // Publish metadata enrichment events
            if let Some(producer) = event_producer {
                if let Err(e) = producer.publish_batch(events).await {
                    warn!("Failed to publish metadata enrichment events: {}", e);
                }
            }

            info!(
                "Completed batch {}/{}: enriched {} items, computed {} quality scores",
                batch_idx + 1,
                (stale_content.len() + BATCH_SIZE - 1) / BATCH_SIZE,
                batch.len(),
                batch.len()
            );
        }

        info!(
            "Metadata enrichment completed: {} embeddings regenerated, {} quality scores computed",
            total_enriched, total_quality_computed
        );

        Ok(())
    }

    /// Compute quality score based on metadata completeness
    ///
    /// Score ranges from 0.0 to 1.0 based on:
    /// - Has title: 0.1
    /// - Has overview: 0.2
    /// - Has release year: 0.1
    /// - Has runtime: 0.1
    /// - Has genres: 0.2
    /// - Has rating: 0.1
    /// - Has user rating: 0.1
    /// - Has embedding: 0.1
    fn compute_quality_score(content: &CanonicalContent) -> f64 {
        let mut score = 0.0;

        // Title (always present, but check if non-empty)
        if !content.title.is_empty() {
            score += 0.1;
        }

        // Overview
        if content
            .overview
            .as_ref()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
        {
            score += 0.2;
        }

        // Release year
        if content.release_year.is_some() {
            score += 0.1;
        }

        // Runtime
        if content.runtime_minutes.is_some() {
            score += 0.1;
        }

        // Genres (at least one)
        if !content.genres.is_empty() {
            score += 0.2;
        }

        // Rating
        if content.rating.is_some() {
            score += 0.1;
        }

        // User rating
        if content.user_rating.is_some() {
            score += 0.1;
        }

        // Embedding
        if content.embedding.is_some() {
            score += 0.1;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_schedule() {
        let schedule = IngestionSchedule::default();
        assert_eq!(schedule.catalog_refresh, Duration::from_secs(6 * 3600));
        assert_eq!(schedule.availability_sync, Duration::from_secs(3600));
        assert_eq!(schedule.expiring_content, Duration::from_secs(900));
        assert_eq!(schedule.metadata_enrichment, Duration::from_secs(24 * 3600));
    }
}
