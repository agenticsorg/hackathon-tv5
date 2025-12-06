use omega_protocol::{Recommendation, ViewContext, ViewingPattern};
use crate::vectors::SearchResult;

/// Rank recommendations by success rate
/// Target: <1ms
pub fn rank_recommendations(
    similar: Vec<SearchResult>,
    context: &ViewContext,
) -> Vec<Recommendation> {
    let mut recommendations: Vec<Recommendation> = similar
        .into_iter()
        .enumerate()
        .map(|(idx, result)| {
            // Parse metadata to get pattern info
            let success_rate = result
                .metadata
                .get("success_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5) as f32;

            let title = result
                .metadata
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            // Score combines similarity (distance) and success rate
            // Lower distance is better, higher success_rate is better
            let similarity_score = 1.0 - (result.distance / 2.0); // Normalize to [0, 1]
            let score = (similarity_score * 0.6) + (success_rate * 0.4);

            Recommendation {
                content_id: result.id,
                content_title: title,
                score,
                confidence: similarity_score,
                reason: format!(
                    "Match: {:.0}%, Success: {:.0}%",
                    similarity_score * 100.0,
                    success_rate * 100.0
                ),
            }
        })
        .collect();

    // Sort by score descending
    recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    recommendations
}

/// Filter recommendations by context
/// Target: <1ms
pub fn filter_by_context(
    recommendations: Vec<Recommendation>,
    context: &ViewContext,
) -> Vec<Recommendation> {
    // For now, simple filtering - could be more sophisticated
    // In production, would match time_of_day, day_of_week, mood, etc.

    recommendations
        .into_iter()
        .filter(|rec| {
            // Keep recommendations with sufficient confidence
            rec.confidence >= 0.3
        })
        .collect()
}

/// Deduplicate recommendations
/// Target: <1ms
pub fn deduplicate(recommendations: Vec<Recommendation>) -> Vec<Recommendation> {
    let mut seen = std::collections::HashSet::new();
    let mut deduped = Vec::new();

    for rec in recommendations {
        if seen.insert(rec.content_id.clone()) {
            deduped.push(rec);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vectors::SearchResult;
    use omega_protocol::events::ViewContext;

    #[test]
    fn test_rank_recommendations() {
        let results = vec![
            SearchResult {
                id: "content-1".to_string(),
                distance: 0.1,
                metadata: serde_json::json!({
                    "title": "Movie 1",
                    "success_rate": 0.9
                }),
            },
            SearchResult {
                id: "content-2".to_string(),
                distance: 0.5,
                metadata: serde_json::json!({
                    "title": "Movie 2",
                    "success_rate": 0.7
                }),
            },
        ];

        let context = ViewContext::default();
        let recommendations = rank_recommendations(results, &context);

        assert_eq!(recommendations.len(), 2);
        assert_eq!(recommendations[0].content_id, "content-1");
        assert!(recommendations[0].score > recommendations[1].score);
    }

    #[test]
    fn test_deduplicate() {
        let recommendations = vec![
            Recommendation::new("content-1".to_string(), "Movie 1".to_string(), 0.9),
            Recommendation::new("content-2".to_string(), "Movie 2".to_string(), 0.8),
            Recommendation::new("content-1".to_string(), "Movie 1".to_string(), 0.85),
        ];

        let deduped = deduplicate(recommendations);

        assert_eq!(deduped.len(), 2);
    }
}
