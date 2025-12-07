use crate::normalizer::CanonicalContent;
use chrono::{DateTime, Utc};

pub fn score_canonical_content(content: &CanonicalContent, weights: &super::QualityWeights) -> f32 {
    let mut score = 0.0;

    if content.overview.is_some() {
        score += weights.has_description;
    }
    if content.images.poster_medium.is_some() || content.images.poster_large.is_some() {
        score += weights.has_poster;
    }
    if content.images.backdrop.is_some() {
        score += weights.has_backdrop;
    }
    if content.release_year.is_some() {
        score += weights.has_release_year;
    }
    if content.runtime_minutes.is_some() {
        score += weights.has_runtime;
    }
    if !content.genres.is_empty() {
        score += weights.has_genres;
    }
    if content.user_rating.is_some() {
        score += weights.has_imdb_rating;
    }
    if content.external_ids.get("imdb_id").is_some()
        || content.external_ids.get("tmdb_id").is_some()
    {
        score += weights.has_external_ids;
    }

    score.clamp(0.0, 1.0)
}

pub fn score_canonical_with_decay(
    content: &CanonicalContent,
    last_updated: DateTime<Utc>,
    weights: &super::QualityWeights,
) -> f32 {
    let base_score = score_canonical_content(content, weights);
    let freshness_factor = calculate_freshness(last_updated);
    let final_score =
        base_score * (1.0 - weights.freshness_weight) + freshness_factor * weights.freshness_weight;
    final_score.clamp(0.0, 1.0)
}

fn calculate_freshness(last_updated: DateTime<Utc>) -> f32 {
    let now = Utc::now();
    let days_since_update = (now - last_updated).num_days() as f32;
    let decay_factor = (1.0 - days_since_update / 365.0).max(0.5);
    decay_factor.clamp(0.5, 1.0)
}

pub fn identify_missing_fields_canonical(content: &CanonicalContent) -> Vec<String> {
    let mut missing = Vec::new();

    if content.overview.is_none() {
        missing.push("overview".to_string());
    }
    if content.images.poster_medium.is_none() && content.images.poster_large.is_none() {
        missing.push("poster".to_string());
    }
    if content.images.backdrop.is_none() {
        missing.push("backdrop".to_string());
    }
    if content.release_year.is_none() {
        missing.push("release_year".to_string());
    }
    if content.runtime_minutes.is_none() {
        missing.push("runtime_minutes".to_string());
    }
    if content.genres.is_empty() {
        missing.push("genres".to_string());
    }
    if content.user_rating.is_none() {
        missing.push("user_rating".to_string());
    }
    if content.external_ids.get("imdb_id").is_none()
        && content.external_ids.get("tmdb_id").is_none()
    {
        missing.push("external_ids".to_string());
    }

    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalizer::{AvailabilityInfo, CanonicalContent, ContentType, ImageSet};
    use crate::quality::QualityWeights;
    use std::collections::HashMap;

    fn create_test_content() -> CanonicalContent {
        let mut external_ids = HashMap::new();
        external_ids.insert("imdb_id".to_string(), "tt1234567".to_string());
        external_ids.insert("tmdb_id".to_string(), "12345".to_string());

        CanonicalContent {
            platform_id: "test".to_string(),
            platform_content_id: "test123".to_string(),
            content_type: ContentType::Movie,
            title: "Test Movie".to_string(),
            overview: Some("Great movie".to_string()),
            release_year: Some(2023),
            runtime_minutes: Some(120),
            genres: vec!["Action".to_string()],
            rating: None,
            user_rating: Some(8.5),
            images: ImageSet {
                poster_small: Some("http://example.com/poster-small.jpg".to_string()),
                poster_medium: Some("http://example.com/poster-medium.jpg".to_string()),
                poster_large: Some("http://example.com/poster-large.jpg".to_string()),
                backdrop: Some("http://example.com/backdrop.jpg".to_string()),
            },
            external_ids,
            availability: AvailabilityInfo {
                regions: vec!["US".to_string()],
                subscription_required: true,
                purchase_price: None,
                rental_price: None,
                currency: Some("USD".to_string()),
                available_from: None,
                available_until: None,
            },
            entity_id: None,
            embedding: None,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_score_canonical_content() {
        let content = create_test_content();
        let weights = QualityWeights::default();

        let score = score_canonical_content(&content, &weights);
        assert!(score > 0.8);
    }

    #[test]
    fn test_score_with_decay() {
        let content = create_test_content();
        let weights = QualityWeights::default();
        let last_updated = Utc::now();

        let score = score_canonical_with_decay(&content, last_updated, &weights);
        assert!(score > 0.7);
    }

    #[test]
    fn test_identify_missing_fields() {
        let mut content = create_test_content();
        content.overview = None;
        content.images.poster_medium = None;
        content.images.poster_large = None;

        let missing = identify_missing_fields_canonical(&content);
        assert!(missing.contains(&"overview".to_string()));
        assert!(missing.contains(&"poster".to_string()));
    }
}
