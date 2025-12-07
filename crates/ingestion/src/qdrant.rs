//! Qdrant Vector Database Integration
//!
//! This module provides integration with Qdrant for storing and querying
//! content embeddings generated during the ingestion pipeline.
//!
//! Key features:
//! - Automatic collection creation and management
//! - Batch upsert operations for performance (max 100 points per call)
//! - Health checking and error handling
//! - Integration with CanonicalContent and embedding pipeline

use crate::{normalizer::CanonicalContent, IngestionError, Result};
use qdrant_client::qdrant::{
    vectors_config::Config, CreateCollection, Distance, PointStruct, UpsertPointsBuilder,
    Value as QdrantValue, VectorParams, VectorsConfig,
};
use qdrant_client::Qdrant as QdrantClientImpl;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Vector dimension for text-embedding-3-small model
pub const VECTOR_DIM: u64 = 768;

/// Maximum points per batch upsert operation
const MAX_BATCH_SIZE: usize = 100;

/// Qdrant client for managing vector operations
pub struct QdrantClient {
    client: QdrantClientImpl,
    collection_name: String,
}

/// Content metadata payload stored with each vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPayload {
    /// Content ID from database
    pub content_id: Uuid,
    /// Content title
    pub title: String,
    /// Genre list
    pub genres: Vec<String>,
    /// Platform identifier
    pub platform: String,
    /// Release year
    pub release_year: i32,
    /// Popularity/rating score
    pub popularity_score: f32,
}

/// Complete point structure for upserting to Qdrant
#[derive(Debug, Clone)]
pub struct ContentPoint {
    /// Point ID (same as content_id for easy lookup)
    pub id: Uuid,
    /// 768-dimensional embedding vector
    pub vector: Vec<f32>,
    /// Metadata payload
    pub payload: ContentPayload,
}

impl QdrantClient {
    /// Create a new Qdrant client
    ///
    /// # Arguments
    /// * `url` - Qdrant server URL (e.g., "http://localhost:6334")
    /// * `collection` - Collection name for storing content vectors
    ///
    /// # Returns
    /// Result containing QdrantClient or error
    ///
    /// # Example
    /// ```no_run
    /// use media_gateway_ingestion::qdrant::QdrantClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = QdrantClient::new("http://localhost:6334", "content_vectors").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(url: &str, collection: &str) -> Result<Self> {
        let client = QdrantClientImpl::from_url(url).build().map_err(|e| {
            IngestionError::ConfigError(format!("Failed to create Qdrant client: {}", e))
        })?;

        info!("Connected to Qdrant at {}", url);

        Ok(Self {
            client,
            collection_name: collection.to_string(),
        })
    }

    /// Perform health check on Qdrant server
    ///
    /// # Returns
    /// Result indicating if server is healthy
    pub async fn health_check(&self) -> Result<bool> {
        match self.client.health_check().await {
            Ok(_) => {
                debug!("Qdrant health check passed");
                Ok(true)
            }
            Err(e) => {
                warn!("Qdrant health check failed: {}", e);
                Err(IngestionError::DatabaseError(format!(
                    "Qdrant health check failed: {}",
                    e
                )))
            }
        }
    }

    /// Ensure collection exists, creating it if necessary
    ///
    /// # Arguments
    /// * `vector_size` - Dimension of vectors (default: 768)
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn ensure_collection(&self, vector_size: u64) -> Result<()> {
        // Check if collection exists
        let collections = self.client.list_collections().await.map_err(|e| {
            IngestionError::DatabaseError(format!("Failed to list collections: {}", e))
        })?;

        let collection_exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.collection_name);

        if collection_exists {
            debug!("Collection '{}' already exists", self.collection_name);
            return Ok(());
        }

        // Create collection with cosine similarity
        info!(
            "Creating collection '{}' with vector size {}",
            self.collection_name, vector_size
        );

        self.client
            .create_collection(CreateCollection {
                collection_name: self.collection_name.clone(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: vector_size,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await
            .map_err(|e| {
                IngestionError::DatabaseError(format!("Failed to create collection: {}", e))
            })?;

        info!("Successfully created collection '{}'", self.collection_name);
        Ok(())
    }

    /// Upsert a single point to the collection
    ///
    /// # Arguments
    /// * `id` - Point ID (typically matches content_id)
    /// * `vector` - 768-dimensional embedding vector
    /// * `payload` - Content metadata
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn upsert_point(
        &self,
        id: Uuid,
        vector: Vec<f32>,
        payload: ContentPayload,
    ) -> Result<()> {
        let point = self.create_point_struct(id, vector, payload)?;

        self.client
            .upsert_points(UpsertPointsBuilder::new(
                self.collection_name.clone(),
                vec![point],
            ))
            .await
            .map_err(|e| IngestionError::DatabaseError(format!("Failed to upsert point: {}", e)))?;

        debug!("Upserted point {} to Qdrant", id);
        Ok(())
    }

    /// Upsert a batch of points to the collection
    ///
    /// Maximum 100 points per call for optimal performance.
    ///
    /// # Arguments
    /// * `points` - Vector of ContentPoint structures
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Panics
    /// Panics if batch size exceeds MAX_BATCH_SIZE (100)
    pub async fn upsert_batch(&self, points: Vec<ContentPoint>) -> Result<()> {
        if points.is_empty() {
            debug!("Empty batch, skipping upsert");
            return Ok(());
        }

        if points.len() > MAX_BATCH_SIZE {
            return Err(IngestionError::ConfigError(format!(
                "Batch size {} exceeds maximum of {}",
                points.len(),
                MAX_BATCH_SIZE
            )));
        }

        let point_structs: Vec<PointStruct> = points
            .into_iter()
            .map(|p| self.create_point_struct(p.id, p.vector, p.payload))
            .collect::<Result<Vec<_>>>()?;

        self.client
            .upsert_points(UpsertPointsBuilder::new(
                self.collection_name.clone(),
                point_structs.clone(),
            ))
            .await
            .map_err(|e| IngestionError::DatabaseError(format!("Failed to upsert batch: {}", e)))?;

        info!(
            "Upserted batch of {} points to Qdrant collection '{}'",
            point_structs.len(),
            self.collection_name
        );
        Ok(())
    }

    /// Create a PointStruct from individual components
    fn create_point_struct(
        &self,
        id: Uuid,
        vector: Vec<f32>,
        payload: ContentPayload,
    ) -> Result<PointStruct> {
        // Convert payload to HashMap for Qdrant
        let mut payload_map = HashMap::new();

        payload_map.insert(
            "content_id".to_string(),
            QdrantValue::from(payload.content_id.to_string()),
        );
        payload_map.insert("title".to_string(), QdrantValue::from(payload.title));
        payload_map.insert("platform".to_string(), QdrantValue::from(payload.platform));
        payload_map.insert(
            "release_year".to_string(),
            QdrantValue::from(payload.release_year as i64),
        );
        payload_map.insert(
            "popularity_score".to_string(),
            QdrantValue::from(payload.popularity_score as f64),
        );

        // Convert genres to list value
        let genre_values: Vec<QdrantValue> =
            payload.genres.into_iter().map(QdrantValue::from).collect();
        payload_map.insert(
            "genres".to_string(),
            QdrantValue {
                kind: Some(qdrant_client::qdrant::value::Kind::ListValue(
                    qdrant_client::qdrant::ListValue {
                        values: genre_values,
                    },
                )),
            },
        );

        use qdrant_client::Payload;

        let payload: Payload = payload_map.into();

        Ok(PointStruct::new(id.to_string(), vector, payload))
    }

    /// Search for similar content by vector
    ///
    /// # Arguments
    /// * `query_vector` - Query embedding vector
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// Vector of (Uuid, score) tuples
    pub async fn search_similar(
        &self,
        query_vector: Vec<f32>,
        limit: u64,
    ) -> Result<Vec<(Uuid, f32)>> {
        let search_result = self
            .client
            .search_points(qdrant_client::qdrant::SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                limit,
                with_payload: Some(false.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| {
                IngestionError::DatabaseError(format!("Failed to search vectors: {}", e))
            })?;

        let results: Vec<(Uuid, f32)> = search_result
            .result
            .into_iter()
            .filter_map(|point| {
                let point_id = point.id?;
                let id_str = match point_id {
                    qdrant_client::qdrant::PointId {
                        point_id_options:
                            Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid_str)),
                    } => uuid_str,
                    qdrant_client::qdrant::PointId {
                        point_id_options:
                            Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)),
                    } => num.to_string(),
                    _ => return None,
                };
                let id = Uuid::parse_str(&id_str).ok()?;
                Some((id, point.score))
            })
            .collect();

        debug!("Found {} similar vectors", results.len());
        Ok(results)
    }
}

/// Convert CanonicalContent with embedding to ContentPoint
///
/// # Arguments
/// * `content` - Canonical content with embedding
/// * `content_id` - Database ID for the content
///
/// # Returns
/// Result containing ContentPoint or error if embedding is missing
pub fn to_content_point(content: &CanonicalContent, content_id: Uuid) -> Result<ContentPoint> {
    let vector = content.embedding.as_ref().ok_or_else(|| {
        IngestionError::NormalizationFailed("Content missing embedding vector".to_string())
    })?;

    let payload = ContentPayload {
        content_id,
        title: content.title.clone(),
        genres: content.genres.clone(),
        platform: content.platform_id.clone(),
        release_year: content.release_year.unwrap_or(0),
        popularity_score: content.user_rating.unwrap_or(0.0),
    };

    Ok(ContentPoint {
        id: content_id,
        vector: vector.clone(),
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalizer::{AvailabilityInfo, ContentType, ImageSet};
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_payload_serialization() {
        let payload = ContentPayload {
            content_id: Uuid::new_v4(),
            title: "The Matrix".to_string(),
            genres: vec!["Action".to_string(), "Science Fiction".to_string()],
            platform: "netflix".to_string(),
            release_year: 1999,
            popularity_score: 8.7,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("The Matrix"));
        assert!(json.contains("Action"));
        assert!(json.contains("netflix"));

        // Deserialize back
        let deserialized: ContentPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, "The Matrix");
        assert_eq!(deserialized.genres.len(), 2);
        assert_eq!(deserialized.release_year, 1999);
    }

    #[test]
    fn test_to_content_point_success() {
        let content_id = Uuid::new_v4();
        let embedding = vec![0.1; 768]; // 768-dimensional vector

        let content = CanonicalContent {
            platform_content_id: "test-123".to_string(),
            platform_id: "netflix".to_string(),
            entity_id: None,
            title: "The Matrix".to_string(),
            overview: Some("A hacker discovers reality is a simulation".to_string()),
            content_type: ContentType::Movie,
            release_year: Some(1999),
            runtime_minutes: Some(136),
            genres: vec!["Action".to_string(), "Science Fiction".to_string()],
            external_ids: HashMap::new(),
            availability: AvailabilityInfo {
                regions: vec!["US".to_string()],
                subscription_required: true,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: None,
                available_until: None,
            },
            images: ImageSet::default(),
            rating: Some("R".to_string()),
            user_rating: Some(8.7),
            embedding: Some(embedding.clone()),
            updated_at: Utc::now(),
        };

        let point = to_content_point(&content, content_id).unwrap();

        assert_eq!(point.id, content_id);
        assert_eq!(point.vector.len(), 768);
        assert_eq!(point.payload.title, "The Matrix");
        assert_eq!(point.payload.platform, "netflix");
        assert_eq!(point.payload.release_year, 1999);
        assert_eq!(point.payload.popularity_score, 8.7);
        assert_eq!(point.payload.genres.len(), 2);
    }

    #[test]
    fn test_to_content_point_missing_embedding() {
        let content_id = Uuid::new_v4();

        let content = CanonicalContent {
            platform_content_id: "test-123".to_string(),
            platform_id: "netflix".to_string(),
            entity_id: None,
            title: "The Matrix".to_string(),
            overview: Some("A hacker discovers reality is a simulation".to_string()),
            content_type: ContentType::Movie,
            release_year: Some(1999),
            runtime_minutes: Some(136),
            genres: vec!["Action".to_string()],
            external_ids: HashMap::new(),
            availability: AvailabilityInfo {
                regions: vec![],
                subscription_required: false,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: None,
                available_until: None,
            },
            images: ImageSet::default(),
            rating: None,
            user_rating: None,
            embedding: None, // Missing embedding
            updated_at: Utc::now(),
        };

        let result = to_content_point(&content, content_id);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing embedding"));
    }

    #[test]
    fn test_content_payload_default_values() {
        let content_id = Uuid::new_v4();
        let embedding = vec![0.5; 768];

        // Content with missing optional fields
        let content = CanonicalContent {
            platform_content_id: "test-456".to_string(),
            platform_id: "prime_video".to_string(),
            entity_id: None,
            title: "Unknown Movie".to_string(),
            overview: None,
            content_type: ContentType::Movie,
            release_year: None, // Missing year
            runtime_minutes: None,
            genres: vec![],
            external_ids: HashMap::new(),
            availability: AvailabilityInfo {
                regions: vec![],
                subscription_required: false,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: None,
                available_until: None,
            },
            images: ImageSet::default(),
            rating: None,
            user_rating: None, // Missing rating
            embedding: Some(embedding),
            updated_at: Utc::now(),
        };

        let point = to_content_point(&content, content_id).unwrap();

        // Should use default values
        assert_eq!(point.payload.release_year, 0);
        assert_eq!(point.payload.popularity_score, 0.0);
        assert!(point.payload.genres.is_empty());
    }

    #[test]
    fn test_vector_dimension_validation() {
        let content_id = Uuid::new_v4();

        // Test with correct dimension
        let correct_embedding = vec![0.1; 768];
        let content_correct = CanonicalContent {
            platform_content_id: "test-789".to_string(),
            platform_id: "netflix".to_string(),
            entity_id: None,
            title: "Test Content".to_string(),
            overview: None,
            content_type: ContentType::Movie,
            release_year: Some(2024),
            runtime_minutes: None,
            genres: vec![],
            external_ids: HashMap::new(),
            availability: AvailabilityInfo {
                regions: vec![],
                subscription_required: false,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: None,
                available_until: None,
            },
            images: ImageSet::default(),
            rating: None,
            user_rating: None,
            embedding: Some(correct_embedding),
            updated_at: Utc::now(),
        };

        let point = to_content_point(&content_correct, content_id).unwrap();
        assert_eq!(point.vector.len(), 768);
    }
}
