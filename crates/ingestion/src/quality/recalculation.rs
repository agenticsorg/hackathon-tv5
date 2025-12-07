use crate::normalizer::CanonicalContent;
use crate::quality::QualityScorer;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::{error, info};

pub struct RecalculationJob {
    scorer: QualityScorer,
    pool: PgPool,
}

impl RecalculationJob {
    pub fn new(scorer: QualityScorer, pool: PgPool) -> Self {
        Self { scorer, pool }
    }

    pub async fn recalculate_all_scores(&self) -> Result<RecalculationReport, RecalculationError> {
        info!("Starting weekly quality score recalculation");

        let content_items = self.fetch_all_content().await?;
        let total_items = content_items.len();

        let mut updated_count = 0;
        let mut failed_count = 0;

        for (content, last_updated) in content_items {
            match self.recalculate_single_score(&content, last_updated).await {
                Ok(_) => updated_count += 1,
                Err(e) => {
                    error!(
                        "Failed to recalculate score for {}: {}",
                        content.platform_content_id, e
                    );
                    failed_count += 1;
                }
            }
        }

        info!(
            "Recalculation complete: {} updated, {} failed out of {} total",
            updated_count, failed_count, total_items
        );

        Ok(RecalculationReport {
            total_items,
            updated_count,
            failed_count,
            execution_time_seconds: 0,
        })
    }

    async fn fetch_all_content(
        &self,
    ) -> Result<Vec<(CanonicalContent, DateTime<Utc>)>, RecalculationError> {
        #[derive(sqlx::FromRow)]
        struct ContentRow {
            platform_id: String,
            platform_content_id: String,
            title: String,
            content_type: String,
            overview: Option<String>,
            release_year: Option<i32>,
            runtime_minutes: Option<i32>,
            user_rating: Option<f64>,
            genres: serde_json::Value,
            images: serde_json::Value,
            external_ids: serde_json::Value,
            availability: serde_json::Value,
            updated_at: DateTime<Utc>,
            entity_id: Option<String>,
            embedding: Option<serde_json::Value>,
        }

        let rows: Vec<ContentRow> = sqlx::query_as(
            r#"
            SELECT
                platform_id,
                platform_content_id,
                title,
                content_type,
                overview,
                release_year,
                runtime_minutes,
                user_rating,
                genres,
                images,
                external_ids,
                availability,
                updated_at,
                entity_id,
                embedding
            FROM content
            WHERE deleted_at IS NULL
            ORDER BY updated_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RecalculationError::DatabaseError(e.to_string()))?;

        let mut content_items = Vec::new();

        for row in rows {
            let content_type = match row.content_type.as_str() {
                "movie" => crate::normalizer::ContentType::Movie,
                "episode" => crate::normalizer::ContentType::Episode,
                _ => continue,
            };

            let images =
                serde_json::from_value(row.images).unwrap_or(crate::normalizer::ImageSet {
                    poster_small: None,
                    poster_medium: None,
                    poster_large: None,
                    backdrop: None,
                });

            let external_ids = serde_json::from_value(row.external_ids).unwrap_or_default();

            let availability = serde_json::from_value(row.availability).unwrap_or(
                crate::normalizer::AvailabilityInfo {
                    regions: vec![],
                    subscription_required: false,
                    purchase_price: None,
                    rental_price: None,
                    currency: None,
                    available_from: None,
                    available_until: None,
                },
            );

            let genres: Vec<String> = serde_json::from_value(row.genres).unwrap_or_default();

            let embedding: Option<Vec<f32>> =
                row.embedding.and_then(|e| serde_json::from_value(e).ok());

            let content = CanonicalContent {
                platform_id: row.platform_id,
                platform_content_id: row.platform_content_id,
                content_type,
                title: row.title,
                overview: row.overview,
                release_year: row.release_year,
                runtime_minutes: row.runtime_minutes,
                genres,
                rating: None,
                user_rating: row.user_rating.map(|r| r as f32),
                images,
                external_ids,
                availability,
                entity_id: row.entity_id,
                embedding,
                updated_at: row.updated_at,
            };

            content_items.push((content, row.updated_at));
        }

        Ok(content_items)
    }

    async fn recalculate_single_score(
        &self,
        content: &CanonicalContent,
        last_updated: DateTime<Utc>,
    ) -> Result<(), RecalculationError> {
        let new_score = self
            .scorer
            .score_content_with_freshness(content, last_updated);

        sqlx::query(
            r#"
            UPDATE content
            SET quality_score = $1
            WHERE platform_id = $2 AND platform_content_id = $3
            "#,
        )
        .bind(new_score)
        .bind(&content.platform_id)
        .bind(&content.platform_content_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RecalculationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn recalculate_outdated_scores(
        &self,
        days_threshold: i64,
    ) -> Result<RecalculationReport, RecalculationError> {
        info!(
            "Recalculating scores for content updated more than {} days ago",
            days_threshold
        );

        let threshold_date = Utc::now() - chrono::Duration::days(days_threshold);

        let content_items = self.fetch_outdated_content(threshold_date).await?;
        let total_items = content_items.len();

        let mut updated_count = 0;
        let mut failed_count = 0;

        for (content, last_updated) in content_items {
            match self.recalculate_single_score(&content, last_updated).await {
                Ok(_) => updated_count += 1,
                Err(e) => {
                    error!(
                        "Failed to recalculate score for {}: {}",
                        content.platform_content_id, e
                    );
                    failed_count += 1;
                }
            }
        }

        info!(
            "Outdated content recalculation complete: {} updated, {} failed out of {} total",
            updated_count, failed_count, total_items
        );

        Ok(RecalculationReport {
            total_items,
            updated_count,
            failed_count,
            execution_time_seconds: 0,
        })
    }

    async fn fetch_outdated_content(
        &self,
        threshold_date: DateTime<Utc>,
    ) -> Result<Vec<(CanonicalContent, DateTime<Utc>)>, RecalculationError> {
        #[derive(sqlx::FromRow)]
        struct ContentRow {
            platform_id: String,
            platform_content_id: String,
            title: String,
            content_type: String,
            overview: Option<String>,
            release_year: Option<i32>,
            runtime_minutes: Option<i32>,
            user_rating: Option<f64>,
            genres: serde_json::Value,
            images: serde_json::Value,
            external_ids: serde_json::Value,
            availability: serde_json::Value,
            updated_at: DateTime<Utc>,
            entity_id: Option<String>,
            embedding: Option<serde_json::Value>,
        }

        let rows: Vec<ContentRow> = sqlx::query_as(
            r#"
            SELECT
                platform_id,
                platform_content_id,
                title,
                content_type,
                overview,
                release_year,
                runtime_minutes,
                user_rating,
                genres,
                images,
                external_ids,
                availability,
                updated_at,
                entity_id,
                embedding
            FROM content
            WHERE deleted_at IS NULL
              AND updated_at < $1
            ORDER BY updated_at ASC
            "#,
        )
        .bind(threshold_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RecalculationError::DatabaseError(e.to_string()))?;

        let mut content_items = Vec::new();

        for row in rows {
            let content_type = match row.content_type.as_str() {
                "movie" => crate::normalizer::ContentType::Movie,
                "episode" => crate::normalizer::ContentType::Episode,
                _ => continue,
            };

            let images =
                serde_json::from_value(row.images).unwrap_or(crate::normalizer::ImageSet {
                    poster_small: None,
                    poster_medium: None,
                    poster_large: None,
                    backdrop: None,
                });

            let external_ids = serde_json::from_value(row.external_ids).unwrap_or_default();

            let availability = serde_json::from_value(row.availability).unwrap_or(
                crate::normalizer::AvailabilityInfo {
                    regions: vec![],
                    subscription_required: false,
                    purchase_price: None,
                    rental_price: None,
                    currency: None,
                    available_from: None,
                    available_until: None,
                },
            );

            let genres: Vec<String> = serde_json::from_value(row.genres).unwrap_or_default();

            let embedding: Option<Vec<f32>> =
                row.embedding.and_then(|e| serde_json::from_value(e).ok());

            let content = CanonicalContent {
                platform_id: row.platform_id,
                platform_content_id: row.platform_content_id,
                content_type,
                title: row.title,
                overview: row.overview,
                release_year: row.release_year,
                runtime_minutes: row.runtime_minutes,
                genres,
                rating: None,
                user_rating: row.user_rating.map(|r| r as f32),
                images,
                external_ids,
                availability,
                entity_id: row.entity_id,
                embedding,
                updated_at: row.updated_at,
            };

            content_items.push((content, row.updated_at));
        }

        Ok(content_items)
    }
}

#[derive(Debug, Clone)]
pub struct RecalculationReport {
    pub total_items: usize,
    pub updated_count: usize,
    pub failed_count: usize,
    pub execution_time_seconds: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum RecalculationError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recalculation_report() {
        let report = RecalculationReport {
            total_items: 100,
            updated_count: 95,
            failed_count: 5,
            execution_time_seconds: 120,
        };

        assert_eq!(report.total_items, 100);
        assert_eq!(report.updated_count, 95);
        assert_eq!(report.failed_count, 5);
    }
}
