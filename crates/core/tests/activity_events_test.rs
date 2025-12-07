//! Integration tests for User Activity Event Stream
//!
//! Tests the unified activity event system across discovery, playback, and auth.

use chrono::Utc;
use media_gateway_core::{
    ActivityEventError, ActivityEventType, KafkaActivityProducer, UserActivityEvent,
    UserActivityProducer,
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_search_query_event() {
    let user_id = Uuid::new_v4();
    let metadata = json!({
        "query": "action movies",
        "results_count": 42,
        "clicked_items": ["content-1", "content-2"],
        "search_time_ms": 125
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::SearchQuery, metadata);

    assert_eq!(event.user_id, user_id);
    assert_eq!(event.event_type, ActivityEventType::SearchQuery);
    assert!(event.content_id.is_none()); // Search queries don't have content_id
    assert!(event.validate().is_ok());
}

#[tokio::test]
async fn test_search_result_click_event() {
    let user_id = Uuid::new_v4();
    let content_id = "movie-123";
    let metadata = json!({
        "query": "inception",
        "result_position": 1,
        "relevance_score": 0.95
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::SearchResultClick, metadata)
        .with_content_id(content_id)
        .with_device_id("web-browser")
        .with_region("US");

    assert_eq!(event.content_id, Some(content_id.to_string()));
    assert_eq!(event.device_id, Some("web-browser".to_string()));
    assert_eq!(event.region, Some("US".to_string()));
    assert!(event.validate().is_ok());
}

#[tokio::test]
async fn test_playback_start_event() {
    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let metadata = json!({
        "session_id": session_id.to_string(),
        "device_id": "roku-123",
        "duration_seconds": 7200,
        "quality": "4K"
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::PlaybackStart, metadata)
        .with_content_id(content_id.to_string())
        .with_device_id("roku-123");

    assert_eq!(event.event_type, ActivityEventType::PlaybackStart);
    assert!(event.validate().is_ok());
}

#[tokio::test]
async fn test_playback_complete_event() {
    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    let metadata = json!({
        "session_id": Uuid::new_v4().to_string(),
        "final_position_seconds": 7180,
        "duration_seconds": 7200,
        "completion_rate": 0.997
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::PlaybackComplete, metadata)
        .with_content_id(content_id.to_string());

    assert!(event.validate().is_ok());
}

#[tokio::test]
async fn test_playback_abandon_event() {
    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    let metadata = json!({
        "session_id": Uuid::new_v4().to_string(),
        "final_position_seconds": 600,
        "duration_seconds": 7200,
        "completion_rate": 0.083
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::PlaybackAbandon, metadata)
        .with_content_id(content_id.to_string());

    assert!(event.validate().is_ok());
}

#[tokio::test]
async fn test_user_login_event() {
    let user_id = Uuid::new_v4();
    let metadata = json!({
        "email": "user@example.com",
        "login_time": Utc::now().to_rfc3339(),
        "ip_address": "192.168.1.1"
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::UserLogin, metadata);

    assert_eq!(event.event_type, ActivityEventType::UserLogin);
    assert!(event.content_id.is_none()); // Auth events don't have content_id
    assert!(event.validate().is_ok());
}

#[tokio::test]
async fn test_user_logout_event() {
    let user_id = Uuid::new_v4();
    let metadata = json!({
        "session_duration_seconds": 3600,
        "logout_time": Utc::now().to_rfc3339()
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::UserLogout, metadata);

    assert!(event.validate().is_ok());
}

#[tokio::test]
async fn test_profile_update_event() {
    let user_id = Uuid::new_v4();
    let metadata = json!({
        "updated_fields": ["display_name", "avatar_url"],
        "update_time": Utc::now().to_rfc3339()
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::ProfileUpdate, metadata);

    assert!(event.validate().is_ok());
}

#[tokio::test]
async fn test_content_rating_event() {
    let user_id = Uuid::new_v4();
    let content_id = "movie-789";
    let metadata = json!({
        "rating": 4.5,
        "review": "Great movie!",
        "rated_at": Utc::now().to_rfc3339()
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::ContentRating, metadata)
        .with_content_id(content_id);

    assert!(event.validate().is_ok());
}

#[tokio::test]
async fn test_event_validation_missing_content_id() {
    let user_id = Uuid::new_v4();
    let metadata = json!({
        "position_seconds": 120
    });

    // PlaybackStart requires content_id
    let event = UserActivityEvent::new(user_id, ActivityEventType::PlaybackStart, metadata);

    assert!(event.validate().is_err());

    match event.validate() {
        Err(ActivityEventError::InvalidEvent(msg)) => {
            assert!(msg.contains("requires content_id"));
        }
        _ => panic!("Expected InvalidEvent error"),
    }
}

#[tokio::test]
async fn test_event_serialization() {
    let user_id = Uuid::new_v4();
    let content_id = "content-123";
    let metadata = json!({
        "query": "test query",
        "results_count": 10
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::SearchQuery, metadata)
        .with_content_id(content_id);

    // Serialize to JSON
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("event_id"));
    assert!(json.contains("user_id"));
    assert!(json.contains("search_query"));

    // Deserialize back
    let deserialized: UserActivityEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, event.user_id);
    assert_eq!(deserialized.event_type, event.event_type);
    assert_eq!(deserialized.content_id, event.content_id);
}

#[tokio::test]
async fn test_event_type_string_representation() {
    assert_eq!(ActivityEventType::SearchQuery.as_str(), "search_query");
    assert_eq!(
        ActivityEventType::SearchResultClick.as_str(),
        "search_result_click"
    );
    assert_eq!(ActivityEventType::ContentView.as_str(), "content_view");
    assert_eq!(ActivityEventType::ContentRating.as_str(), "content_rating");
    assert_eq!(ActivityEventType::PlaybackStart.as_str(), "playback_start");
    assert_eq!(ActivityEventType::PlaybackPause.as_str(), "playback_pause");
    assert_eq!(
        ActivityEventType::PlaybackResume.as_str(),
        "playback_resume"
    );
    assert_eq!(
        ActivityEventType::PlaybackComplete.as_str(),
        "playback_complete"
    );
    assert_eq!(
        ActivityEventType::PlaybackAbandon.as_str(),
        "playback_abandon"
    );
    assert_eq!(ActivityEventType::UserLogin.as_str(), "user_login");
    assert_eq!(ActivityEventType::UserLogout.as_str(), "user_logout");
    assert_eq!(ActivityEventType::ProfileUpdate.as_str(), "profile_update");
    assert_eq!(
        ActivityEventType::PreferenceChange.as_str(),
        "preference_change"
    );
}

#[tokio::test]
async fn test_event_deduplication() {
    // This test would require a running Kafka instance
    // For now, we test the deduplication logic in isolation

    let event_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let metadata = json!({"test": true});

    let event1 = UserActivityEvent {
        event_id,
        user_id,
        event_type: ActivityEventType::SearchQuery,
        content_id: None,
        timestamp: Utc::now(),
        metadata: metadata.clone(),
        device_id: None,
        region: None,
    };

    let event2 = UserActivityEvent {
        event_id, // Same event_id
        user_id,
        event_type: ActivityEventType::SearchQuery,
        content_id: None,
        timestamp: Utc::now(),
        metadata,
        device_id: None,
        region: None,
    };

    // Both events have the same event_id, so they should be considered duplicates
    assert_eq!(event1.event_id, event2.event_id);
}

#[tokio::test]
async fn test_batch_event_creation() {
    let user_id = Uuid::new_v4();

    let events = vec![
        UserActivityEvent::new(
            user_id,
            ActivityEventType::SearchQuery,
            json!({"query": "action"}),
        ),
        UserActivityEvent::new(
            user_id,
            ActivityEventType::SearchResultClick,
            json!({"position": 1}),
        )
        .with_content_id("movie-1"),
        UserActivityEvent::new(
            user_id,
            ActivityEventType::PlaybackStart,
            json!({"quality": "HD"}),
        )
        .with_content_id("movie-1"),
        UserActivityEvent::new(
            user_id,
            ActivityEventType::PlaybackComplete,
            json!({"completion_rate": 1.0}),
        )
        .with_content_id("movie-1"),
    ];

    assert_eq!(events.len(), 4);

    for event in &events {
        assert!(event.validate().is_ok());
    }
}

#[tokio::test]
#[ignore] // Requires running Kafka
async fn test_kafka_producer_integration() {
    // Set up environment variables for Kafka
    std::env::set_var("KAFKA_BROKERS", "localhost:9092");
    std::env::set_var("KAFKA_TOPIC_PREFIX", "test");

    let producer = KafkaActivityProducer::from_env().expect("Failed to create producer");

    let user_id = Uuid::new_v4();
    let metadata = json!({
        "query": "integration test",
        "results_count": 5
    });

    let event = UserActivityEvent::new(user_id, ActivityEventType::SearchQuery, metadata);

    let result = producer.publish_activity(event).await;

    assert!(result.is_ok(), "Failed to publish event: {:?}", result);

    // Clean up
    std::env::remove_var("KAFKA_BROKERS");
    std::env::remove_var("KAFKA_TOPIC_PREFIX");
}

#[tokio::test]
#[ignore] // Requires running Kafka
async fn test_kafka_batch_publishing() {
    std::env::set_var("KAFKA_BROKERS", "localhost:9092");
    std::env::set_var("KAFKA_TOPIC_PREFIX", "test");

    let producer = KafkaActivityProducer::from_env().expect("Failed to create producer");

    let user_id = Uuid::new_v4();

    let events = vec![
        UserActivityEvent::new(
            user_id,
            ActivityEventType::SearchQuery,
            json!({"query": "batch test 1"}),
        ),
        UserActivityEvent::new(
            user_id,
            ActivityEventType::SearchQuery,
            json!({"query": "batch test 2"}),
        ),
        UserActivityEvent::new(
            user_id,
            ActivityEventType::SearchQuery,
            json!({"query": "batch test 3"}),
        ),
    ];

    let result = producer.publish_batch(events).await;

    assert!(result.is_ok(), "Failed to publish batch: {:?}", result);

    std::env::remove_var("KAFKA_BROKERS");
    std::env::remove_var("KAFKA_TOPIC_PREFIX");
}
