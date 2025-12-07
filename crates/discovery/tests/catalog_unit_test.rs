use media_gateway_discovery::catalog::types::*;

#[test]
fn test_content_type_serialization() {
    let movie = ContentType::Movie;
    let json = serde_json::to_string(&movie).unwrap();
    assert_eq!(json, r#""movie""#);

    let series = ContentType::Series;
    let json = serde_json::to_string(&series).unwrap();
    assert_eq!(json, r#""series""#);

    let episode = ContentType::Episode;
    let json = serde_json::to_string(&episode).unwrap();
    assert_eq!(json, r#""episode""#);
}

#[test]
fn test_content_type_deserialization() {
    let movie: ContentType = serde_json::from_str(r#""movie""#).unwrap();
    assert_eq!(movie, ContentType::Movie);

    let series: ContentType = serde_json::from_str(r#""series""#).unwrap();
    assert_eq!(series, ContentType::Series);
}

#[test]
fn test_create_request_validation_valid() {
    let request = CreateContentRequest {
        title: "The Matrix".to_string(),
        content_type: ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "nf_matrix_1999".to_string(),
        overview: Some("A computer hacker learns about the true nature of reality".to_string()),
        release_year: Some(1999),
        runtime_minutes: Some(136),
        genres: vec!["sci-fi".to_string(), "action".to_string()],
        rating: Some("R".to_string()),
        images: ImageSet::default(),
    };

    assert!(request.validate().is_ok());
}

#[test]
fn test_create_request_validation_empty_title() {
    let request = CreateContentRequest {
        title: "   ".to_string(),
        content_type: ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "test".to_string(),
        overview: None,
        release_year: None,
        runtime_minutes: None,
        genres: vec![],
        rating: None,
        images: ImageSet::default(),
    };

    assert!(request.validate().is_err());
    assert_eq!(request.validate().unwrap_err(), "Title is required");
}

#[test]
fn test_create_request_validation_empty_platform() {
    let request = CreateContentRequest {
        title: "Test Movie".to_string(),
        content_type: ContentType::Movie,
        platform: "".to_string(),
        platform_content_id: "test".to_string(),
        overview: None,
        release_year: None,
        runtime_minutes: None,
        genres: vec![],
        rating: None,
        images: ImageSet::default(),
    };

    assert!(request.validate().is_err());
    assert_eq!(request.validate().unwrap_err(), "Platform is required");
}

#[test]
fn test_create_request_validation_empty_platform_id() {
    let request = CreateContentRequest {
        title: "Test Movie".to_string(),
        content_type: ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "  ".to_string(),
        overview: None,
        release_year: None,
        runtime_minutes: None,
        genres: vec![],
        rating: None,
        images: ImageSet::default(),
    };

    assert!(request.validate().is_err());
    assert_eq!(
        request.validate().unwrap_err(),
        "Platform content ID is required"
    );
}

#[test]
fn test_create_request_validation_invalid_year_too_old() {
    let request = CreateContentRequest {
        title: "Ancient Movie".to_string(),
        content_type: ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "test".to_string(),
        overview: None,
        release_year: Some(1500),
        runtime_minutes: None,
        genres: vec![],
        rating: None,
        images: ImageSet::default(),
    };

    assert!(request.validate().is_err());
}

#[test]
fn test_create_request_validation_invalid_year_future() {
    let request = CreateContentRequest {
        title: "Future Movie".to_string(),
        content_type: ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "test".to_string(),
        overview: None,
        release_year: Some(2150),
        runtime_minutes: None,
        genres: vec![],
        rating: None,
        images: ImageSet::default(),
    };

    assert!(request.validate().is_err());
}

#[test]
fn test_create_request_validation_negative_runtime() {
    let request = CreateContentRequest {
        title: "Test Movie".to_string(),
        content_type: ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "test".to_string(),
        overview: None,
        release_year: None,
        runtime_minutes: Some(-10),
        genres: vec![],
        rating: None,
        images: ImageSet::default(),
    };

    assert!(request.validate().is_err());
    assert_eq!(request.validate().unwrap_err(), "Runtime must be positive");
}

#[test]
fn test_image_set_default() {
    let images = ImageSet::default();
    assert!(images.poster_small.is_none());
    assert!(images.poster_medium.is_none());
    assert!(images.poster_large.is_none());
    assert!(images.backdrop.is_none());
}

#[test]
fn test_content_response_serialization() {
    use chrono::Utc;
    use uuid::Uuid;

    let response = ContentResponse {
        id: Uuid::nil(),
        title: "Test Movie".to_string(),
        content_type: ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "nf_test".to_string(),
        overview: Some("Test overview".to_string()),
        release_year: Some(2024),
        runtime_minutes: Some(120),
        genres: vec!["action".to_string()],
        rating: Some("PG-13".to_string()),
        images: ImageSet::default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("Test Movie"));
    assert!(json.contains("netflix"));
}
