use serde::{Deserialize, Serialize};

/// Content rating for parental controls
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContentRating {
    G = 0,
    PG = 1,
    #[serde(rename = "PG-13")]
    PG13 = 2,
    R = 3,
    #[serde(rename = "NC-17")]
    NC17 = 4,
}

impl ContentRating {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "G" => Some(ContentRating::G),
            "PG" => Some(ContentRating::PG),
            "PG-13" | "PG13" => Some(ContentRating::PG13),
            "R" => Some(ContentRating::R),
            "NC-17" | "NC17" => Some(ContentRating::NC17),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ContentRating::G => "G",
            ContentRating::PG => "PG",
            ContentRating::PG13 => "PG-13",
            ContentRating::R => "R",
            ContentRating::NC17 => "NC-17",
        }
    }
}

/// Search filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Genre filters (OR logic)
    pub genres: Vec<String>,

    /// Platform availability filters
    pub platforms: Vec<String>,

    /// Year range filter (min, max)
    pub year_range: Option<(i32, i32)>,

    /// Rating range filter (min, max)
    pub rating_range: Option<(f32, f32)>,

    /// Content rating limit for parental controls
    pub content_rating_limit: Option<ContentRating>,

    /// Blocked genres for parental controls
    pub blocked_genres: Vec<String>,
}

impl SearchFilters {
    /// Check if any filters are active
    pub fn is_empty(&self) -> bool {
        self.genres.is_empty()
            && self.platforms.is_empty()
            && self.year_range.is_none()
            && self.rating_range.is_none()
            && self.content_rating_limit.is_none()
            && self.blocked_genres.is_empty()
    }

    /// Build SQL WHERE clause for filters
    pub fn to_sql_where_clause(&self) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let params: Vec<String> = Vec::new();

        // Genre filter
        if !self.genres.is_empty() {
            conditions.push(format!(
                "genres && ARRAY[{}]::text[]",
                self.genres
                    .iter()
                    .map(|g| format!("'{}'", g))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Platform filter
        if !self.platforms.is_empty() {
            conditions.push(format!(
                "platforms && ARRAY[{}]::text[]",
                self.platforms
                    .iter()
                    .map(|p| format!("'{}'", p))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Year range filter
        if let Some((min_year, max_year)) = self.year_range {
            conditions.push(format!(
                "release_year BETWEEN {} AND {}",
                min_year, max_year
            ));
        }

        // Rating range filter
        if let Some((min_rating, max_rating)) = self.rating_range {
            conditions.push(format!(
                "average_rating BETWEEN {} AND {}",
                min_rating, max_rating
            ));
        }

        // Content rating filter (parental controls)
        if let Some(rating_limit) = self.content_rating_limit {
            let rating_value = rating_limit as i32;
            conditions.push(format!("content_rating_value <= {}", rating_value));
        }

        // Blocked genres filter (parental controls)
        if !self.blocked_genres.is_empty() {
            conditions.push(format!(
                "NOT (genres && ARRAY[{}]::text[])",
                self.blocked_genres
                    .iter()
                    .map(|g| format!("'{}'", g))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let clause = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        (clause, params)
    }

    /// Estimate filter selectivity (0.0 = very selective, 1.0 = not selective)
    pub fn estimate_selectivity(&self) -> f32 {
        let mut selectivity = 1.0;

        // Genre filter reduces to ~30% of content
        if !self.genres.is_empty() {
            selectivity *= 0.3;
        }

        // Platform filter reduces to ~40% of content
        if !self.platforms.is_empty() {
            selectivity *= 0.4;
        }

        // Year range filter
        if let Some((min_year, max_year)) = self.year_range {
            let range = (max_year - min_year) as f32;
            selectivity *= (range / 100.0).min(1.0); // Assume 100 year catalog
        }

        // Rating filter
        if self.rating_range.is_some() {
            selectivity *= 0.5;
        }

        // Content rating filter (parental controls)
        if let Some(rating) = self.content_rating_limit {
            // Stricter ratings are more selective
            let rating_selectivity = match rating {
                ContentRating::G => 0.2,
                ContentRating::PG => 0.4,
                ContentRating::PG13 => 0.6,
                ContentRating::R => 0.8,
                ContentRating::NC17 => 1.0,
            };
            selectivity *= rating_selectivity;
        }

        // Blocked genres
        if !self.blocked_genres.is_empty() {
            selectivity *= 0.7; // Reduces content by ~30%
        }

        selectivity
    }

    /// Determine if pre-filtering or post-filtering is better
    pub fn should_pre_filter(&self) -> bool {
        self.estimate_selectivity() < 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_filters() {
        let filters = SearchFilters::default();
        assert!(filters.is_empty());
    }

    #[test]
    fn test_sql_where_clause() {
        let filters = SearchFilters {
            genres: vec!["action".to_string(), "thriller".to_string()],
            platforms: vec!["netflix".to_string()],
            year_range: Some((2020, 2024)),
            rating_range: Some((7.0, 10.0)),
        };

        let (clause, _) = filters.to_sql_where_clause();
        assert!(clause.contains("genres &&"));
        assert!(clause.contains("platforms &&"));
        assert!(clause.contains("BETWEEN"));
    }

    #[test]
    fn test_selectivity_estimation() {
        let filters = SearchFilters {
            genres: vec!["action".to_string()],
            platforms: vec!["netflix".to_string()],
            year_range: Some((2020, 2024)),
            rating_range: None,
        };

        let selectivity = filters.estimate_selectivity();
        assert!(selectivity < 0.1); // Should be highly selective
        assert!(filters.should_pre_filter());
    }
}
