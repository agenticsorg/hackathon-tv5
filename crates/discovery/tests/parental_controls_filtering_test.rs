use media_gateway_discovery::search::filters::{ContentRating, SearchFilters};

#[test]
fn test_content_rating_hierarchy() {
    assert!(ContentRating::G < ContentRating::PG);
    assert!(ContentRating::PG < ContentRating::PG13);
    assert!(ContentRating::PG13 < ContentRating::R);
    assert!(ContentRating::R < ContentRating::NC17);
}

#[test]
fn test_content_rating_serialization() {
    let rating = ContentRating::PG13;
    let json = serde_json::to_string(&rating).unwrap();
    assert_eq!(json, "\"PG-13\"");

    let deserialized: ContentRating = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, ContentRating::PG13);
}

#[test]
fn test_search_filters_with_content_rating_limit() {
    let filters = SearchFilters {
        genres: vec![],
        platforms: vec![],
        year_range: None,
        rating_range: None,
        content_rating_limit: Some(ContentRating::PG13),
        blocked_genres: vec![],
    };

    let (clause, _) = filters.to_sql_where_clause();
    assert!(clause.contains("content_rating_value <= 2"));
}

#[test]
fn test_search_filters_with_blocked_genres() {
    let filters = SearchFilters {
        genres: vec![],
        platforms: vec![],
        year_range: None,
        rating_range: None,
        content_rating_limit: None,
        blocked_genres: vec!["horror".to_string(), "thriller".to_string()],
    };

    let (clause, _) = filters.to_sql_where_clause();
    assert!(clause.contains("NOT (genres &&"));
    assert!(clause.contains("'horror'"));
    assert!(clause.contains("'thriller'"));
}

#[test]
fn test_search_filters_with_parental_controls_affects_selectivity() {
    let filters_without_parental = SearchFilters {
        genres: vec!["action".to_string()],
        platforms: vec![],
        year_range: None,
        rating_range: None,
        content_rating_limit: None,
        blocked_genres: vec![],
    };

    let filters_with_parental = SearchFilters {
        genres: vec!["action".to_string()],
        platforms: vec![],
        year_range: None,
        rating_range: None,
        content_rating_limit: Some(ContentRating::G),
        blocked_genres: vec!["horror".to_string()],
    };

    let selectivity_without = filters_without_parental.estimate_selectivity();
    let selectivity_with = filters_with_parental.estimate_selectivity();

    // Parental controls should make filters more selective (lower selectivity value)
    assert!(selectivity_with < selectivity_without);
}

#[test]
fn test_search_filters_g_rating_most_selective() {
    let filters_g = SearchFilters {
        genres: vec![],
        platforms: vec![],
        year_range: None,
        rating_range: None,
        content_rating_limit: Some(ContentRating::G),
        blocked_genres: vec![],
    };

    let filters_nc17 = SearchFilters {
        genres: vec![],
        platforms: vec![],
        year_range: None,
        rating_range: None,
        content_rating_limit: Some(ContentRating::NC17),
        blocked_genres: vec![],
    };

    assert!(filters_g.estimate_selectivity() < filters_nc17.estimate_selectivity());
}

#[test]
fn test_search_filters_combined_parental_controls() {
    let filters = SearchFilters {
        genres: vec!["action".to_string()],
        platforms: vec!["netflix".to_string()],
        year_range: Some((2020, 2024)),
        rating_range: Some((7.0, 10.0)),
        content_rating_limit: Some(ContentRating::PG),
        blocked_genres: vec!["horror".to_string(), "thriller".to_string()],
    };

    let (clause, _) = filters.to_sql_where_clause();

    // Should include all filters
    assert!(clause.contains("genres &&"));
    assert!(clause.contains("platforms &&"));
    assert!(clause.contains("BETWEEN"));
    assert!(clause.contains("content_rating_value <= 1")); // PG = 1
    assert!(clause.contains("NOT (genres &&"));

    // Verify it's not empty
    assert!(!filters.is_empty());

    // Should be highly selective
    assert!(filters.should_pre_filter());
}

#[test]
fn test_content_rating_from_str() {
    assert_eq!(ContentRating::from_str("G"), Some(ContentRating::G));
    assert_eq!(ContentRating::from_str("PG"), Some(ContentRating::PG));
    assert_eq!(ContentRating::from_str("PG-13"), Some(ContentRating::PG13));
    assert_eq!(ContentRating::from_str("PG13"), Some(ContentRating::PG13));
    assert_eq!(ContentRating::from_str("R"), Some(ContentRating::R));
    assert_eq!(ContentRating::from_str("NC-17"), Some(ContentRating::NC17));
    assert_eq!(ContentRating::from_str("NC17"), Some(ContentRating::NC17));
    assert_eq!(ContentRating::from_str("invalid"), None);
}

#[test]
fn test_content_rating_as_str() {
    assert_eq!(ContentRating::G.as_str(), "G");
    assert_eq!(ContentRating::PG.as_str(), "PG");
    assert_eq!(ContentRating::PG13.as_str(), "PG-13");
    assert_eq!(ContentRating::R.as_str(), "R");
    assert_eq!(ContentRating::NC17.as_str(), "NC-17");
}

#[test]
fn test_empty_filters_without_parental_controls() {
    let filters = SearchFilters::default();
    assert!(filters.is_empty());
}

#[test]
fn test_filters_not_empty_with_content_rating() {
    let filters = SearchFilters {
        genres: vec![],
        platforms: vec![],
        year_range: None,
        rating_range: None,
        content_rating_limit: Some(ContentRating::PG13),
        blocked_genres: vec![],
    };

    assert!(!filters.is_empty());
}

#[test]
fn test_filters_not_empty_with_blocked_genres() {
    let filters = SearchFilters {
        genres: vec![],
        platforms: vec![],
        year_range: None,
        rating_range: None,
        content_rating_limit: None,
        blocked_genres: vec!["horror".to_string()],
    };

    assert!(!filters.is_empty());
}
