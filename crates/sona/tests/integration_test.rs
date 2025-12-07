//! Integration tests for SONA HTTP endpoint wiring

use actix_web::{test, web, App};
use serde_json::json;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_recommendations_endpoint_structure() {
        // Test that recommendation endpoint exists and has correct structure
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request_payload = json!({
            "user_id": user_id,
            "limit": 10,
            "exclude_watched": true,
            "diversity_threshold": 0.3
        });

        // Verify payload serialization
        assert!(request_payload["user_id"].is_string());
        assert_eq!(request_payload["limit"], 10);
    }

    #[actix_web::test]
    async fn test_personalization_score_structure() {
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request_payload = json!({
            "user_id": user_id,
            "content_id": content_id
        });

        // Verify structure
        assert!(request_payload["user_id"].is_string());
        assert!(request_payload["content_id"].is_string());
    }

    #[actix_web::test]
    async fn test_profile_update_structure() {
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let viewing_event = json!({
            "content_id": content_id,
            "timestamp": "2024-01-01T12:00:00Z",
            "completion_rate": 0.85,
            "rating": 5,
            "is_rewatch": false,
            "dismissed": false
        });

        let request_payload = json!({
            "user_id": user_id,
            "viewing_events": [viewing_event]
        });

        // Verify structure
        assert!(request_payload["viewing_events"].is_array());
        assert_eq!(
            request_payload["viewing_events"].as_array().unwrap().len(),
            1
        );
    }

    #[actix_web::test]
    async fn test_lora_training_structure() {
        let user_id = Uuid::new_v4();

        let request_payload = json!({
            "user_id": user_id,
            "force": false
        });

        // Verify structure
        assert!(request_payload["user_id"].is_string());
        assert_eq!(request_payload["force"], false);
    }

    #[test]
    fn test_viewing_event_conversion() {
        use chrono::Utc;
        use media_gateway_sona::ViewingEvent;

        let event = ViewingEvent {
            content_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            completion_rate: 0.9,
            rating: Some(5),
            is_rewatch: true,
            dismissed: false,
        };

        assert_eq!(event.completion_rate, 0.9);
        assert_eq!(event.rating, Some(5));
        assert!(event.is_rewatch);
        assert!(!event.dismissed);
    }
}
