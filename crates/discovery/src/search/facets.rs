//! Faceted search aggregations
//!
//! Provides facet computation over search results for genres, platforms,
//! release years (bucketed), and ratings (bucketed).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, instrument};

use super::{ContentSummary, SearchResult};

/// A single facet count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetCount {
    pub value: String,
    pub count: usize,
}

/// Facet aggregation results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FacetResults {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<FacetCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub platforms: Vec<FacetCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub years: Vec<FacetCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ratings: Vec<FacetCount>,
}

#[derive(Debug, Clone)]
pub struct YearBucket {
    pub label: String,
    pub min_year: i32,
    pub max_year: i32,
}

#[derive(Debug, Clone)]
pub struct RatingBucket {
    pub label: String,
    pub min_rating: f32,
    pub max_rating: f32,
}

/// Facet service for computing aggregations
pub struct FacetService {
    year_buckets: Vec<YearBucket>,
    rating_buckets: Vec<RatingBucket>,
    max_facets: usize,
}

impl FacetService {
    pub fn new() -> Self {
        Self {
            year_buckets: Self::default_year_buckets(),
            rating_buckets: Self::default_rating_buckets(),
            max_facets: 20,
        }
    }

    pub fn with_config(
        year_buckets: Vec<YearBucket>,
        rating_buckets: Vec<RatingBucket>,
        max_facets: usize,
    ) -> Self {
        Self {
            year_buckets,
            rating_buckets,
            max_facets,
        }
    }

    fn default_year_buckets() -> Vec<YearBucket> {
        vec![
            YearBucket {
                label: "2020s".to_string(),
                min_year: 2020,
                max_year: 2029,
            },
            YearBucket {
                label: "2010s".to_string(),
                min_year: 2010,
                max_year: 2019,
            },
            YearBucket {
                label: "2000s".to_string(),
                min_year: 2000,
                max_year: 2009,
            },
            YearBucket {
                label: "1990s".to_string(),
                min_year: 1990,
                max_year: 1999,
            },
            YearBucket {
                label: "1980s".to_string(),
                min_year: 1980,
                max_year: 1989,
            },
            YearBucket {
                label: "Classic (pre-1980)".to_string(),
                min_year: 0,
                max_year: 1979,
            },
        ]
    }

    fn default_rating_buckets() -> Vec<RatingBucket> {
        vec![
            RatingBucket {
                label: "Excellent (8-10)".to_string(),
                min_rating: 8.0,
                max_rating: 10.0,
            },
            RatingBucket {
                label: "Good (7-8)".to_string(),
                min_rating: 7.0,
                max_rating: 8.0,
            },
            RatingBucket {
                label: "Average (5-7)".to_string(),
                min_rating: 5.0,
                max_rating: 7.0,
            },
            RatingBucket {
                label: "Below Average (0-5)".to_string(),
                min_rating: 0.0,
                max_rating: 5.0,
            },
        ]
    }

    #[instrument(skip(self, results), fields(result_count = results.len()))]
    pub fn compute_facets(&self, results: &[SearchResult]) -> HashMap<String, Vec<FacetCount>> {
        let start = std::time::Instant::now();
        let mut facets = HashMap::new();

        let genres = self.compute_genre_facets(results);
        if !genres.is_empty() {
            facets.insert("genres".to_string(), genres);
        }

        let platforms = self.compute_platform_facets(results);
        if !platforms.is_empty() {
            facets.insert("platforms".to_string(), platforms);
        }

        let years = self.compute_year_facets(results);
        if !years.is_empty() {
            facets.insert("years".to_string(), years);
        }

        let ratings = self.compute_rating_facets(results);
        if !ratings.is_empty() {
            facets.insert("ratings".to_string(), ratings);
        }

        debug!(
            elapsed_ms = start.elapsed().as_millis(),
            facet_count = facets.len(),
            "Computed facets"
        );
        facets
    }

    /// Compute facets and return as FacetResults struct
    #[instrument(skip(self, results), fields(result_count = results.len()))]
    pub fn compute_facets_structured(&self, results: &[SearchResult]) -> FacetResults {
        let start = std::time::Instant::now();
        let genres = self.compute_genre_facets(results);
        let platforms = self.compute_platform_facets(results);
        let years = self.compute_year_facets(results);
        let ratings = self.compute_rating_facets(results);
        debug!(elapsed_ms = start.elapsed().as_millis(), "Computed facets");
        FacetResults {
            genres,
            platforms,
            years,
            ratings,
        }
    }

    pub fn compute_facets_from_content(
        &self,
        content: &[ContentSummary],
    ) -> HashMap<String, Vec<FacetCount>> {
        let results: Vec<SearchResult> = content
            .iter()
            .map(|c| SearchResult {
                content: c.clone(),
                relevance_score: 0.0,
                match_reasons: vec![],
                vector_similarity: None,
                graph_score: None,
                keyword_score: None,
            })
            .collect();
        self.compute_facets(&results)
    }

    fn compute_genre_facets(&self, results: &[SearchResult]) -> Vec<FacetCount> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for result in results {
            for genre in &result.content.genres {
                *counts.entry(genre.to_lowercase()).or_default() += 1;
            }
        }
        self.to_sorted_facets(counts)
    }

    fn compute_platform_facets(&self, results: &[SearchResult]) -> Vec<FacetCount> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for result in results {
            for platform in &result.content.platforms {
                *counts.entry(platform.to_lowercase()).or_default() += 1;
            }
        }
        self.to_sorted_facets(counts)
    }

    fn compute_year_facets(&self, results: &[SearchResult]) -> Vec<FacetCount> {
        let mut bucket_counts: HashMap<String, usize> = HashMap::new();
        for result in results {
            let year = result.content.release_year;
            for bucket in &self.year_buckets {
                if year >= bucket.min_year && year <= bucket.max_year {
                    *bucket_counts.entry(bucket.label.clone()).or_default() += 1;
                    break;
                }
            }
        }
        self.year_buckets
            .iter()
            .filter_map(|bucket| {
                bucket_counts.get(&bucket.label).map(|&count| FacetCount {
                    value: bucket.label.clone(),
                    count,
                })
            })
            .filter(|f| f.count > 0)
            .collect()
    }

    fn compute_rating_facets(&self, results: &[SearchResult]) -> Vec<FacetCount> {
        let mut bucket_counts: HashMap<String, usize> = HashMap::new();
        for result in results {
            let rating = result.content.popularity_score * 10.0;
            for bucket in &self.rating_buckets {
                if rating >= bucket.min_rating && rating < bucket.max_rating {
                    *bucket_counts.entry(bucket.label.clone()).or_default() += 1;
                    break;
                }
            }
        }
        self.rating_buckets
            .iter()
            .filter_map(|bucket| {
                bucket_counts.get(&bucket.label).map(|&count| FacetCount {
                    value: bucket.label.clone(),
                    count,
                })
            })
            .filter(|f| f.count > 0)
            .collect()
    }

    fn to_sorted_facets(&self, counts: HashMap<String, usize>) -> Vec<FacetCount> {
        let mut facets: Vec<FacetCount> = counts
            .into_iter()
            .map(|(value, count)| FacetCount { value, count })
            .collect();
        facets.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.value.cmp(&b.value)));
        facets.truncate(self.max_facets);
        facets
    }
}

impl Default for FacetService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_result(
        genres: Vec<&str>,
        platforms: Vec<&str>,
        year: i32,
        popularity: f32,
    ) -> SearchResult {
        SearchResult {
            content: ContentSummary {
                id: Uuid::new_v4(),
                title: "Test".to_string(),
                overview: "Desc".to_string(),
                release_year: year,
                genres: genres.into_iter().map(String::from).collect(),
                platforms: platforms.into_iter().map(String::from).collect(),
                popularity_score: popularity,
            },
            relevance_score: 0.8,
            match_reasons: vec![],
            vector_similarity: None,
            graph_score: None,
            keyword_score: None,
        }
    }

    #[test]
    fn test_genre_facets() {
        let service = FacetService::new();
        let results = vec![
            create_test_result(vec!["Action", "Thriller"], vec!["Netflix"], 2020, 0.8),
            create_test_result(vec!["Action", "Comedy"], vec!["Netflix"], 2021, 0.7),
        ];
        let facets = service.compute_facets(&results);
        let genres = facets.get("genres").unwrap();
        let action = genres.iter().find(|f| f.value == "action").unwrap();
        assert_eq!(action.count, 2);
    }

    #[test]
    fn test_year_buckets() {
        let service = FacetService::new();
        let results = vec![
            create_test_result(vec!["Action"], vec!["Netflix"], 2022, 0.8),
            create_test_result(vec!["Drama"], vec!["Netflix"], 2023, 0.7),
        ];
        let facets = service.compute_facets(&results);
        let years = facets.get("years").unwrap();
        let twenties = years.iter().find(|f| f.value == "2020s").unwrap();
        assert_eq!(twenties.count, 2);
    }

    #[test]
    fn test_performance() {
        let service = FacetService::new();
        let genres = ["Action", "Drama", "Comedy", "Horror", "Thriller"];
        let platforms = ["Netflix", "Hulu", "Disney+", "HBO Max"];
        let mut results = Vec::new();
        for i in 0..1000 {
            results.push(create_test_result(
                vec![genres[i % genres.len()]],
                vec![platforms[i % platforms.len()]],
                2000 + (i % 25) as i32,
                (i % 100) as f32 / 100.0,
            ));
        }
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = service.compute_facets(&results);
        }
        assert!(
            start.elapsed().as_millis() < 500,
            "Facet computation too slow"
        );
    }
}
