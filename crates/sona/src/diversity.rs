//! Diversity Filter using Maximal Marginal Relevance (MMR)
//!
//! Implements ApplyDiversityFilter algorithm from SPARC pseudocode.
//! Balances relevance with diversity to avoid redundant recommendations.

use crate::types::ScoredContent;
use anyhow::Result;
use uuid::Uuid;

const LAMBDA: f32 = 0.7; // Balance between relevance and diversity

/// Apply diversity filter using MMR algorithm
///
/// Algorithm: ApplyDiversityFilter (from SPARC pseudocode Part 2)
///
/// MMR formula: score = λ * relevance - (1-λ) * max_similarity_to_selected
///
/// Steps:
/// 1. Sort candidates by score
/// 2. Iteratively select items that maximize MMR score
/// 3. Balance relevance (original score) with diversity (similarity to already selected)
pub struct ApplyDiversityFilter;

impl ApplyDiversityFilter {
    pub fn execute(
        mut candidates: Vec<ScoredContent>,
        _threshold: f32,
        limit: usize,
        get_content_embedding: impl Fn(Uuid) -> Result<Vec<f32>>,
    ) -> Result<Vec<ScoredContent>> {
        // Sort by score (descending)
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        let mut selected: Vec<ScoredContent> = Vec::new();
        let mut remaining = candidates;

        while selected.len() < limit && !remaining.is_empty() {
            let mut best_score = f32::NEG_INFINITY;
            let mut best_index = 0;
            let mut best_candidate = None;

            for (index, candidate) in remaining.iter().enumerate() {
                // MMR score = λ * relevance - (1-λ) * max_similarity_to_selected
                let relevance = candidate.score;

                let max_similarity = if selected.is_empty() {
                    0.0
                } else {
                    let candidate_embedding = get_content_embedding(candidate.content_id)?;

                    let mut max_sim: f32 = 0.0;
                    for s in &selected {
                        let selected_embedding = get_content_embedding(s.content_id)?;
                        let sim =
                            Self::cosine_similarity(&candidate_embedding, &selected_embedding);
                        max_sim = max_sim.max(sim);
                    }
                    max_sim
                };

                let mmr_score = LAMBDA * relevance - (1.0 - LAMBDA) * max_similarity;

                if mmr_score > best_score {
                    best_score = mmr_score;
                    best_index = index;
                    best_candidate = Some(candidate.clone());
                }
            }

            if let Some(candidate) = best_candidate {
                selected.push(candidate);
                remaining.remove(best_index);
            } else {
                break;
            }
        }

        Ok(selected)
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = ApplyDiversityFilter::cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        let sim2 = ApplyDiversityFilter::cosine_similarity(&c, &d);
        assert!((sim2 - 0.0).abs() < 0.001);
    }
}
