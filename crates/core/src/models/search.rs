//! Search models for the Media Gateway platform
//!
//! This module contains data structures for search queries, filters,
//! results, and search strategies.

use crate::types::{AvailabilityType, ContentType, Genre, MaturityRating, Platform, Region};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Search strategy selection
///
/// Determines which search algorithm(s) to use for a query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchStrategy {
    /// Vector similarity search using embeddings
    Vector,
    /// Graph-based relationship search
    Graph,
    /// Traditional keyword search
    Keyword,
    /// Combination of multiple strategies
    Hybrid,
}

/// Sort order for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    /// Most relevant first (default)
    Relevance,
    /// Newest first
    Newest,
    /// Oldest first
    Oldest,
    /// Highest rated first
    HighestRated,
    /// Lowest rated first
    LowestRated,
    /// Most popular first
    MostPopular,
    /// Alphabetical A-Z
    Alphabetical,
    /// Alphabetical Z-A
    ReverseAlphabetical,
}

/// Search filters for refining results
///
/// Provides comprehensive filtering options for content search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct SearchFilters {
    /// Filter by content types
    pub content_types: Vec<ContentType>,

    /// Filter by genres
    pub genres: Vec<Genre>,

    /// Filter by platforms
    pub platforms: Vec<Platform>,

    /// Filter by availability types
    pub availability_types: Vec<AvailabilityType>,

    /// Filter by regions (ISO 3166-1 alpha-2)
    pub regions: Vec<Region>,

    /// Filter by minimum release year
    #[validate(range(min = 1800, max = 2100))]
    pub min_release_year: Option<i32>,

    /// Filter by maximum release year
    #[validate(range(min = 1800, max = 2100))]
    pub max_release_year: Option<i32>,

    /// Filter by minimum runtime (minutes)
    #[validate(range(min = 0))]
    pub min_runtime_minutes: Option<i32>,

    /// Filter by maximum runtime (minutes)
    #[validate(range(min = 0))]
    pub max_runtime_minutes: Option<i32>,

    /// Filter by minimum rating (0.0 - 10.0)
    #[validate(range(min = 0.0, max = 10.0))]
    pub min_rating: Option<f32>,

    /// Filter by maximum maturity rating
    pub max_maturity_rating: Option<MaturityRating>,

    /// Filter by audio languages (ISO 639-1 codes)
    pub audio_languages: Vec<String>,

    /// Filter by subtitle languages (ISO 639-1 codes)
    pub subtitle_languages: Vec<String>,

    /// Filter by keywords
    pub keywords: Vec<String>,

    /// Filter by actors/directors
    pub creators: Vec<String>,

    /// Filter by production companies
    pub production_companies: Vec<String>,

    /// Filter by production countries (ISO 3166-1 alpha-2)
    pub production_countries: Vec<Region>,

    /// Only include content available for free
    pub free_only: bool,

    /// Only include content with high data quality
    pub high_quality_only: bool,

    /// Minimum data quality score (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub min_data_quality_score: Option<f32>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            content_types: Vec::new(),
            genres: Vec::new(),
            platforms: Vec::new(),
            availability_types: Vec::new(),
            regions: Vec::new(),
            min_release_year: None,
            max_release_year: None,
            min_runtime_minutes: None,
            max_runtime_minutes: None,
            min_rating: None,
            max_maturity_rating: None,
            audio_languages: Vec::new(),
            subtitle_languages: Vec::new(),
            keywords: Vec::new(),
            creators: Vec::new(),
            production_companies: Vec::new(),
            production_countries: Vec::new(),
            free_only: false,
            high_quality_only: false,
            min_data_quality_score: None,
        }
    }
}

/// Search query parameters
///
/// Encapsulates all parameters for a content search request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct SearchQuery {
    /// Unique query identifier
    pub query_id: Uuid,

    /// User ID making the query (optional, for personalization)
    pub user_id: Option<Uuid>,

    /// Search query string
    #[validate(length(min = 1, max = 500))]
    pub query: String,

    /// Search filters
    #[validate]
    pub filters: SearchFilters,

    /// Search strategy to use
    pub strategy: SearchStrategy,

    /// Sort order for results
    pub sort_order: SortOrder,

    /// Maximum number of results to return
    #[validate(range(min = 1, max = 1000))]
    pub limit: usize,

    /// Offset for pagination
    #[validate(range(min = 0))]
    pub offset: usize,

    /// Whether to include facet counts in results
    pub include_facets: bool,

    /// Whether to include aggregations
    pub include_aggregations: bool,

    /// Whether to use personalization (if user_id provided)
    pub use_personalization: bool,

    /// Minimum relevance score threshold (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub min_relevance_score: Option<f32>,

    /// When this query was created
    pub created_at: DateTime<Utc>,

    /// Client/device making the query (for analytics)
    pub client_info: Option<ClientInfo>,
}

impl SearchQuery {
    /// Create a new search query with default values
    pub fn new(query: String) -> Self {
        Self {
            query_id: Uuid::new_v4(),
            user_id: None,
            query,
            filters: SearchFilters::default(),
            strategy: SearchStrategy::Hybrid,
            sort_order: SortOrder::Relevance,
            limit: 20,
            offset: 0,
            include_facets: false,
            include_aggregations: false,
            use_personalization: false,
            min_relevance_score: None,
            created_at: Utc::now(),
            client_info: None,
        }
    }

    /// Create a personalized search query
    pub fn personalized(query: String, user_id: Uuid) -> Self {
        Self {
            user_id: Some(user_id),
            use_personalization: true,
            ..Self::new(query)
        }
    }
}

/// Client information for search analytics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct ClientInfo {
    /// Client application identifier
    #[validate(length(min = 1, max = 100))]
    pub app_id: String,

    /// Client version
    #[validate(length(min = 1, max = 50))]
    pub app_version: String,

    /// Device type
    #[validate(length(min = 1, max = 50))]
    pub device_type: Option<String>,

    /// Operating system
    #[validate(length(min = 1, max = 100))]
    pub os: Option<String>,

    /// User agent string
    #[validate(length(max = 500))]
    pub user_agent: Option<String>,
}

/// Search result item
///
/// Represents a single content item in search results with relevance information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct SearchResultItem {
    /// Content canonical ID
    pub content_id: Uuid,

    /// Content type
    pub content_type: ContentType,

    /// Title
    #[validate(length(min = 1, max = 500))]
    pub title: String,

    /// Original title
    #[validate(length(min = 1, max = 500))]
    pub original_title: Option<String>,

    /// Release year
    #[validate(range(min = 1800, max = 2100))]
    pub release_year: i32,

    /// Description/synopsis (truncated)
    #[validate(length(max = 500))]
    pub description: Option<String>,

    /// Genres
    pub genres: Vec<Genre>,

    /// Available platforms
    pub platforms: Vec<Platform>,

    /// Poster image URL
    #[validate(url)]
    pub poster_url: Option<String>,

    /// Average rating (0.0 - 10.0)
    #[validate(range(min = 0.0, max = 10.0))]
    pub average_rating: Option<f32>,

    /// Popularity score
    #[validate(range(min = 0.0))]
    pub popularity_score: Option<f32>,

    /// Relevance score for this query (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub relevance_score: f32,

    /// Highlighted query matches (field -> highlighted text)
    pub highlights: HashMap<String, Vec<String>>,

    /// Explanation of why this result matched (for debugging)
    pub explanation: Option<String>,
}

/// Facet bucket for aggregations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacetBucket {
    /// Facet value
    pub value: String,

    /// Number of documents with this value
    pub count: usize,

    /// Whether this facet is currently selected
    pub selected: bool,
}

/// Facet results for a field
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Facet {
    /// Field name
    pub field: String,

    /// Facet buckets
    pub buckets: Vec<FacetBucket>,
}

/// Search aggregation results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchAggregations {
    /// Total content items matching query
    pub total_items: usize,

    /// Average rating of matching items
    pub average_rating: Option<f32>,

    /// Release year range
    pub year_range: Option<(i32, i32)>,

    /// Runtime range in minutes
    pub runtime_range: Option<(i32, i32)>,

    /// Content type distribution
    pub content_type_counts: HashMap<ContentType, usize>,

    /// Genre distribution
    pub genre_counts: HashMap<Genre, usize>,

    /// Platform distribution
    pub platform_counts: HashMap<Platform, usize>,
}

/// Search results
///
/// Contains search result items, facets, aggregations, and metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct SearchResult {
    /// Query that produced these results
    pub query_id: Uuid,

    /// Result items
    #[validate]
    pub items: Vec<SearchResultItem>,

    /// Total number of matching items (before pagination)
    pub total_count: usize,

    /// Offset used for pagination
    pub offset: usize,

    /// Limit used for pagination
    pub limit: usize,

    /// Whether there are more results
    pub has_more: bool,

    /// Facets for filtering
    pub facets: Vec<Facet>,

    /// Aggregations
    pub aggregations: Option<SearchAggregations>,

    /// Search execution time in milliseconds
    #[validate(range(min = 0))]
    pub execution_time_ms: i64,

    /// Search strategy used
    pub strategy_used: SearchStrategy,

    /// When these results were generated
    pub generated_at: DateTime<Utc>,

    /// Suggestions for query correction/expansion
    pub suggestions: Vec<String>,
}

impl SearchResult {
    /// Create an empty search result
    pub fn empty(query_id: Uuid, strategy: SearchStrategy) -> Self {
        Self {
            query_id,
            items: Vec::new(),
            total_count: 0,
            offset: 0,
            limit: 20,
            has_more: false,
            facets: Vec::new(),
            aggregations: None,
            execution_time_ms: 0,
            strategy_used: strategy,
            generated_at: Utc::now(),
            suggestions: Vec::new(),
        }
    }

    /// Check if search returned any results
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of pages for pagination
    pub fn total_pages(&self) -> usize {
        if self.limit == 0 {
            return 0;
        }
        (self.total_count + self.limit - 1) / self.limit
    }

    /// Get the current page number (1-indexed)
    pub fn current_page(&self) -> usize {
        if self.limit == 0 {
            return 1;
        }
        (self.offset / self.limit) + 1
    }
}

/// Autocomplete suggestion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct AutocompleteSuggestion {
    /// Suggested text
    #[validate(length(min = 1, max = 200))]
    pub text: String,

    /// Type of suggestion (content, person, keyword)
    #[validate(length(min = 1, max = 50))]
    pub suggestion_type: String,

    /// Relevance score (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub score: f32,

    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// Autocomplete request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct AutocompleteRequest {
    /// Partial query string
    #[validate(length(min = 1, max = 100))]
    pub prefix: String,

    /// Maximum number of suggestions
    #[validate(range(min = 1, max = 50))]
    pub limit: usize,

    /// Types of suggestions to include
    pub suggestion_types: Vec<String>,

    /// User ID for personalization
    pub user_id: Option<Uuid>,
}

/// Autocomplete response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct AutocompleteResponse {
    /// Suggestions
    #[validate]
    pub suggestions: Vec<AutocompleteSuggestion>,

    /// Execution time in milliseconds
    #[validate(range(min = 0))]
    pub execution_time_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery::new("test query".to_string());

        assert_eq!(query.query, "test query");
        assert_eq!(query.strategy, SearchStrategy::Hybrid);
        assert_eq!(query.limit, 20);
        assert_eq!(query.offset, 0);
        assert!(!query.use_personalization);
    }

    #[test]
    fn test_personalized_query() {
        let user_id = Uuid::new_v4();
        let query = SearchQuery::personalized("test".to_string(), user_id);

        assert_eq!(query.user_id, Some(user_id));
        assert!(query.use_personalization);
    }

    #[test]
    fn test_empty_search_result() {
        let query_id = Uuid::new_v4();
        let result = SearchResult::empty(query_id, SearchStrategy::Vector);

        assert!(result.is_empty());
        assert_eq!(result.total_count, 0);
        assert_eq!(result.strategy_used, SearchStrategy::Vector);
    }

    #[test]
    fn test_pagination_calculations() {
        let query_id = Uuid::new_v4();
        let mut result = SearchResult::empty(query_id, SearchStrategy::Hybrid);

        result.total_count = 100;
        result.limit = 20;
        result.offset = 40;

        assert_eq!(result.total_pages(), 5);
        assert_eq!(result.current_page(), 3);
    }

    #[test]
    fn test_default_filters() {
        let filters = SearchFilters::default();

        assert!(filters.content_types.is_empty());
        assert!(filters.genres.is_empty());
        assert!(!filters.free_only);
        assert!(!filters.high_quality_only);
    }
}
