use qdrant_client::qdrant::{
    Condition, Filter, Range, SearchParams, SearchPoints, SearchResponse as QdrantSearchResponse,
};
use qdrant_client::Qdrant;
use uuid::Uuid;

use super::filters::SearchFilters;
use super::{ContentSummary, SearchResult};
use crate::embedding::EmbeddingClient;

/// Vector search using Qdrant HNSW
pub struct VectorSearch {
    client: Qdrant,
    collection_name: String,
    dimension: usize,
    ef_search: usize,
    top_k: usize,
    similarity_threshold: f32,
    embedding_client: Option<EmbeddingClient>,
}

impl VectorSearch {
    /// Create new vector search instance
    pub fn new(qdrant_url: String, collection_name: String, dimension: usize) -> Self {
        let client = Qdrant::from_url(&qdrant_url).build().unwrap();

        Self {
            client,
            collection_name,
            dimension,
            ef_search: 64,
            top_k: 50,
            similarity_threshold: 0.7,
            embedding_client: None,
        }
    }

    /// Set embedding client
    pub fn with_embedding_client(mut self, client: EmbeddingClient) -> Self {
        self.embedding_client = Some(client);
        self
    }

    /// Execute vector similarity search
    pub async fn search(
        &self,
        query: &str,
        filters: Option<SearchFilters>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        // Generate query embedding (TODO: implement embedding service)
        let query_vector = self.generate_embedding(query).await?;

        // Determine filter strategy
        let use_pre_filter = filters
            .as_ref()
            .map(|f| f.should_pre_filter())
            .unwrap_or(false);

        let results = if use_pre_filter {
            self.search_with_pre_filter(query_vector, filters).await?
        } else {
            self.search_with_post_filter(query_vector, filters).await?
        };

        Ok(results)
    }

    /// HNSW search with pre-filtering
    async fn search_with_pre_filter(
        &self,
        query_vector: Vec<f32>,
        filters: Option<SearchFilters>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        // Build Qdrant filter
        let filter = filters.as_ref().map(|f| self.build_qdrant_filter(f));

        // Execute search
        let search_result = self
            .client
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                filter,
                limit: self.top_k as u64,
                with_payload: Some(true.into()),
                params: Some(SearchParams {
                    hnsw_ef: Some(self.ef_search as u64),
                    exact: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?;

        self.convert_qdrant_results(search_result)
    }

    /// HNSW search with post-filtering
    async fn search_with_post_filter(
        &self,
        query_vector: Vec<f32>,
        filters: Option<SearchFilters>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        // Overquery to account for filtering
        let overquery_k = self.top_k * 3;

        // Execute search without filters
        let search_result = self
            .client
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                filter: None,
                limit: overquery_k as u64,
                with_payload: Some(true.into()),
                params: Some(SearchParams {
                    hnsw_ef: Some(self.ef_search as u64),
                    exact: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?;

        // Convert and filter
        let mut results = self.convert_qdrant_results(search_result)?;

        // Apply filters in memory
        if let Some(filters) = filters {
            results.retain(|result| self.matches_filters(&result.content, &filters));
        }

        // Limit to top_k
        results.truncate(self.top_k);

        Ok(results)
    }

    /// Generate embedding for query with fallback
    async fn generate_embedding(&self, query: &str) -> anyhow::Result<Vec<f32>> {
        match &self.embedding_client {
            Some(client) => match client.generate(query).await {
                Ok(embedding) => Ok(embedding),
                Err(e) => {
                    tracing::error!("Embedding generation failed: {}", e);
                    tracing::warn!("Embedding service failed, vector search will use fallback");
                    Err(e)
                }
            },
            None => {
                tracing::warn!("No embedding client configured, vector search unavailable");
                Err(anyhow::anyhow!("Embedding client not configured"))
            }
        }
    }

    /// Build Qdrant filter from search filters
    fn build_qdrant_filter(&self, filters: &SearchFilters) -> Filter {
        let mut conditions = Vec::new();

        // Genre filter
        if !filters.genres.is_empty() {
            for genre in &filters.genres {
                conditions.push(Condition::matches("genres", genre.clone()));
            }
        }

        // Platform filter
        if !filters.platforms.is_empty() {
            for platform in &filters.platforms {
                conditions.push(Condition::matches("platforms", platform.clone()));
            }
        }

        // Year range filter
        if let Some((min_year, max_year)) = filters.year_range {
            conditions.push(Condition::range(
                "release_year",
                Range {
                    gte: Some(min_year as f64),
                    lte: Some(max_year as f64),
                    ..Default::default()
                },
            ));
        }

        Filter::must(conditions)
    }

    /// Convert Qdrant results to SearchResult
    fn convert_qdrant_results(
        &self,
        response: QdrantSearchResponse,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for scored_point in response.result {
            // Extract payload
            let payload = scored_point.payload;

            let id_str = payload
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string());

            let content = ContentSummary {
                id: Uuid::parse_str(&id_str)?,
                title: payload
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                overview: payload
                    .get("overview")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                release_year: payload
                    .get("release_year")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(0) as i32,
                genres: payload
                    .get("genres")
                    .and_then(|v| v.as_list())
                    .map(|list| {
                        list.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                platforms: payload
                    .get("platforms")
                    .and_then(|v| v.as_list())
                    .map(|list| {
                        list.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                popularity_score: payload
                    .get("popularity_score")
                    .and_then(|v| v.as_double())
                    .unwrap_or(0.0) as f32,
            };

            results.push(SearchResult {
                content,
                relevance_score: scored_point.score,
                match_reasons: vec!["vector_similarity".to_string()],
                vector_similarity: Some(scored_point.score),
                graph_score: None,
                keyword_score: None,
            });
        }

        Ok(results)
    }

    /// Check if content matches filters
    fn matches_filters(&self, content: &ContentSummary, filters: &SearchFilters) -> bool {
        // Genre filter
        if !filters.genres.is_empty() {
            let has_genre = content.genres.iter().any(|g| filters.genres.contains(g));
            if !has_genre {
                return false;
            }
        }

        // Platform filter
        if !filters.platforms.is_empty() {
            let has_platform = content
                .platforms
                .iter()
                .any(|p| filters.platforms.contains(p));
            if !has_platform {
                return false;
            }
        }

        // Year range filter
        if let Some((min_year, max_year)) = filters.year_range {
            if content.release_year < min_year || content.release_year > max_year {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matching() {
        let vector_search =
            VectorSearch::new("http://localhost:6333".to_string(), "test".to_string(), 768);

        let content = ContentSummary {
            id: Uuid::new_v4(),
            title: "Test Movie".to_string(),
            overview: "Description".to_string(),
            release_year: 2020,
            genres: vec!["action".to_string()],
            platforms: vec!["netflix".to_string()],
            popularity_score: 0.8,
        };

        let filters = SearchFilters {
            genres: vec!["action".to_string()],
            platforms: vec!["netflix".to_string()],
            year_range: Some((2018, 2022)),
            rating_range: None,
        };

        assert!(vector_search.matches_filters(&content, &filters));
    }

    #[test]
    fn test_filter_mismatch() {
        let vector_search =
            VectorSearch::new("http://localhost:6333".to_string(), "test".to_string(), 768);

        let content = ContentSummary {
            id: Uuid::new_v4(),
            title: "Test Movie".to_string(),
            overview: "Description".to_string(),
            release_year: 2020,
            genres: vec!["drama".to_string()],
            platforms: vec!["netflix".to_string()],
            popularity_score: 0.8,
        };

        let filters = SearchFilters {
            genres: vec!["action".to_string()],
            platforms: vec![],
            year_range: None,
            rating_range: None,
        };

        assert!(!vector_search.matches_filters(&content, &filters));
    }
}
