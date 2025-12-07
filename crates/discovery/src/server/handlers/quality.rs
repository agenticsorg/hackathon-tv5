use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use media_gateway_ingestion::normalizer::CanonicalContent;
use media_gateway_ingestion::{
    ContentRepository, PostgresContentRepository,
    QualityScorer,
};

/// Query parameters for quality report endpoint
#[derive(Debug, Deserialize)]
pub struct QualityReportQuery {
    /// Quality score threshold (default: 0.6)
    #[serde(default = "default_threshold")]
    pub threshold: f32,
    /// Maximum number of low-quality items to return (default: 100)
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_threshold() -> f32 {
    0.6
}

fn default_limit() -> i64 {
    100
}

/// Response for quality report endpoint
#[derive(Debug, Serialize)]
pub struct QualityReportResponse {
    pub total_low_quality: usize,
    pub threshold: f32,
    pub low_quality_items: Vec<LowQualityItemResponse>,
}

/// Individual low-quality item in response
#[derive(Debug, Serialize)]
pub struct LowQualityItemResponse {
    pub id: String,
    pub title: String,
    pub quality_score: f32,
    pub missing_fields: Vec<String>,
    pub platform: String,
    pub content_type: String,
}

/// GET /api/v1/quality/report - Get quality report
///
/// Returns a list of low-quality content items below the specified threshold.
/// Used by administrators to identify content that needs metadata enrichment.
///
/// Query parameters:
/// - threshold: Quality score threshold (0.0-1.0, default: 0.6)
/// - limit: Maximum number of items to return (default: 100)
pub async fn get_quality_report(
    pool: web::Data<sqlx::PgPool>,
    params: web::Query<QualityReportQuery>,
) -> impl Responder {
    info!(
        "Fetching quality report with threshold={}, limit={}",
        params.threshold, params.limit
    );

    // Create repository
    let repository = PostgresContentRepository::new(pool.get_ref().clone());

    // Fetch low-quality content
    let low_quality_items = match repository
        .find_low_quality_content(params.threshold, params.limit)
        .await
    {
        Ok(items) => items,
        Err(e) => {
            error!("Failed to fetch low-quality content: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch low-quality content: {}", e)
            }));
        }
    };

    // Convert to response format with missing fields analysis
    let scorer = QualityScorer::default();
    let response_items: Vec<LowQualityItemResponse> = low_quality_items
        .iter()
        .map(|item| {
            let missing_fields = identify_missing_fields(&item.content);
            LowQualityItemResponse {
                id: item.content_id.to_string(),
                title: item.title.clone(),
                quality_score: item.quality_score,
                missing_fields,
                platform: item.content.platform_id.clone(),
                content_type: format!("{:?}", item.content.content_type).to_lowercase(),
            }
        })
        .collect();

    info!(
        "Found {} low-quality items below threshold {}",
        response_items.len(),
        params.threshold
    );

    HttpResponse::Ok().json(QualityReportResponse {
        total_low_quality: response_items.len(),
        threshold: params.threshold,
        low_quality_items: response_items,
    })
}

/// Identify missing fields for a content item
fn identify_missing_fields(content: &CanonicalContent) -> Vec<String> {
    let mut missing = Vec::new();

    if content.overview.is_none()
        || content
            .overview
            .as_ref()
            .map(|s| s.is_empty())
            .unwrap_or(true)
    {
        missing.push("description".to_string());
    }
    if content.images.poster_medium.is_none() && content.images.poster_large.is_none() {
        missing.push("poster".to_string());
    }
    if content.runtime_minutes.is_none() {
        missing.push("runtime".to_string());
    }
    if content.images.poster_large.is_none() {
        missing.push("high_res_poster".to_string());
    }
    if content.images.backdrop.is_none() {
        missing.push("backdrop".to_string());
    }
    if content.user_rating.is_none() {
        missing.push("imdb_rating".to_string());
    }
    if content.external_ids.get("imdb_id").is_none() {
        missing.push("imdb_id".to_string());
    }
    if content.external_ids.get("tmdb_id").is_none()
        && content.external_ids.get("rottentomatoes_id").is_none()
    {
        missing.push("external_ratings".to_string());
    }
    if content.release_year.is_none() {
        missing.push("release_year".to_string());
    }
    if content.genres.is_empty() {
        missing.push("genres".to_string());
    }

    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use media_gateway_ingestion::normalizer::{
        AvailabilityInfo, CanonicalContent, ContentType, ImageSet,
    };
    use std::collections::HashMap;

    fn create_minimal_content() -> CanonicalContent {
        CanonicalContent {
            platform_content_id: "test123".to_string(),
            platform_id: "test".to_string(),
            entity_id: None,
            title: "Test Movie".to_string(),
            overview: None,
            content_type: ContentType::Movie,
            release_year: None,
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
            embedding: None,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_identify_missing_fields_minimal_content() {
        let content = create_minimal_content();
        let missing = identify_missing_fields(&content);

        assert!(missing.contains(&"description".to_string()));
        assert!(missing.contains(&"poster".to_string()));
        assert!(missing.contains(&"runtime".to_string()));
        assert!(missing.contains(&"release_year".to_string()));
        assert!(missing.contains(&"genres".to_string()));
        assert!(missing.len() >= 5);
    }

    #[test]
    fn test_identify_missing_fields_complete_content() {
        let mut content = create_minimal_content();
        content.overview = Some("Great movie".to_string());
        content.runtime_minutes = Some(120);
        content.release_year = Some(2023);
        content.genres = vec!["Action".to_string()];
        content.images.poster_medium = Some("http://example.com/poster.jpg".to_string());
        content.images.poster_large = Some("http://example.com/poster-large.jpg".to_string());
        content.images.backdrop = Some("http://example.com/backdrop.jpg".to_string());
        content.user_rating = Some(8.5);
        content
            .external_ids
            .insert("imdb_id".to_string(), "tt1234567".to_string());

        let missing = identify_missing_fields(&content);

        assert!(missing.is_empty() || missing.len() <= 1); // Should have very few missing fields
    }

    #[test]
    fn test_default_query_params() {
        assert_eq!(default_threshold(), 0.6);
        assert_eq!(default_limit(), 100);
    }
}
