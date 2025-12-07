pub mod canonical_adapter;
pub mod recalculation;
pub mod scorer;

pub use scorer::{
    FreshnessDecay, LowQualityItem, MissingFieldsSummary, QualityReport, QualityScorer,
    QualityWeights, ScoreDistribution,
};

pub use recalculation::{RecalculationError, RecalculationJob, RecalculationReport};

use crate::normalizer::CanonicalContent;
use chrono::{DateTime, Utc};

pub async fn batch_score_content(
    scorer: &QualityScorer,
    content_items: Vec<(CanonicalContent, DateTime<Utc>)>,
) -> Vec<(String, f32)> {
    content_items
        .into_iter()
        .map(|(content, last_updated)| {
            let score = canonical_adapter::score_canonical_with_decay(
                &content,
                last_updated,
                &scorer.weights,
            );
            let id = format!("{}-{}", content.platform_id, content.platform_content_id);
            (id, score)
        })
        .collect()
}

pub fn generate_quality_report(
    content_items: Vec<(CanonicalContent, f32)>,
    quality_threshold: f32,
) -> QualityReport {
    let total_content = content_items.len() as u64;

    if total_content == 0 {
        return QualityReport::new();
    }

    let total_score: f32 = content_items.iter().map(|(_, score)| score).sum();
    let average_score = total_score / total_content as f32;

    let ranges = vec![
        ("0.0-0.2", 0.0, 0.2),
        ("0.2-0.4", 0.2, 0.4),
        ("0.4-0.6", 0.4, 0.6),
        ("0.6-0.8", 0.6, 0.8),
        ("0.8-1.0", 0.8, 1.0),
    ];

    let mut distribution_counts = std::collections::HashMap::new();
    for range in &ranges {
        distribution_counts.insert(range.0.to_string(), 0u64);
    }

    for (_, score) in &content_items {
        for (range_name, min, max) in &ranges {
            if *score >= *min && *score < *max {
                *distribution_counts.get_mut(*range_name).unwrap() += 1;
                break;
            } else if *score >= 0.8 && *score <= 1.0 && range_name == &"0.8-1.0" {
                *distribution_counts.get_mut(*range_name).unwrap() += 1;
                break;
            }
        }
    }

    let score_distribution: Vec<ScoreDistribution> = ranges
        .iter()
        .map(|(range, _, _)| ScoreDistribution {
            range: range.to_string(),
            count: *distribution_counts.get(*range).unwrap_or(&0),
        })
        .collect();

    let mut low_quality_content: Vec<LowQualityItem> = content_items
        .iter()
        .filter(|(_, score)| *score < quality_threshold)
        .map(|(content, score)| {
            let missing_fields = canonical_adapter::identify_missing_fields_canonical(content);
            LowQualityItem {
                id: format!("{}-{}", content.platform_id, content.platform_content_id),
                title: content.title.clone(),
                quality_score: *score,
                missing_fields,
            }
        })
        .collect();

    low_quality_content.sort_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap());

    let mut field_counts = std::collections::HashMap::new();
    for item in &low_quality_content {
        for field in &item.missing_fields {
            *field_counts.entry(field.clone()).or_insert(0u64) += 1;
        }
    }

    let mut missing_fields_summary: Vec<MissingFieldsSummary> = field_counts
        .into_iter()
        .map(|(field, missing_count)| MissingFieldsSummary {
            field,
            missing_count,
        })
        .collect();

    missing_fields_summary.sort_by(|a, b| b.missing_count.cmp(&a.missing_count));

    QualityReport {
        total_content,
        average_score,
        score_distribution,
        low_quality_content,
        missing_fields_summary,
    }
}
