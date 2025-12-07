//! Integration tests for playback service features

use serde_json::json;
use uuid::Uuid;

#[cfg(test)]
mod watch_history_integration {
    use super::*;

    #[tokio::test]
    #[ignore] // Run with: cargo test --ignored
    async fn test_watch_history_resume_workflow() {
        use sqlx::postgres::PgPoolOptions;
        use std::sync::Arc;

        // Setup test database connection
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Initialize managers
        use media_gateway_playback::events::NoOpProducer;
        use media_gateway_playback::session::{
            CreateSessionRequest, SessionManager, UpdatePositionRequest,
        };
        use media_gateway_playback::watch_history::WatchHistoryManager;

        let watch_history = Arc::new(WatchHistoryManager::new(pool));
        let event_producer = Arc::new(NoOpProducer);

        let session_manager = SessionManager::new(
            "redis://127.0.0.1:6379",
            "http://localhost:8083".to_string(),
            event_producer,
        )
        .expect("Failed to create session manager")
        .with_watch_history(watch_history.clone());

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();
        let device_id = "test-device-001".to_string();

        // Test workflow: Watch 50%, delete session, new session returns resume position

        // Step 1: Create initial session (no history yet)
        let create_request = CreateSessionRequest {
            user_id,
            content_id,
            device_id: device_id.clone(),
            duration_seconds: 3600,
            quality: None,
        };

        let response = session_manager
            .create(create_request)
            .await
            .expect("Failed to create session");

        let session_id = response.session.id;

        // Initially no resume position
        assert_eq!(response.resume_position_seconds, None);

        // Step 2: Update position to 50% (1800 seconds)
        let update_request = UpdatePositionRequest {
            position_seconds: 1800,
            playback_state: None,
        };

        session_manager
            .update_position(session_id, update_request)
            .await
            .expect("Failed to update position");

        // Wait for async watch history update
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Step 3: Delete session (simulates user closing app)
        session_manager
            .delete(session_id)
            .await
            .expect("Failed to delete session");

        // Wait for async watch history final update
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Step 4: Create new session (should return resume position)
        let create_request2 = CreateSessionRequest {
            user_id,
            content_id,
            device_id: device_id.clone(),
            duration_seconds: 3600,
            quality: None,
        };

        let response2 = session_manager
            .create(create_request2)
            .await
            .expect("Failed to create second session");

        // Should resume at 1800 seconds (50%)
        assert_eq!(response2.resume_position_seconds, Some(1800));

        // Cleanup
        session_manager
            .delete(response2.session.id)
            .await
            .expect("Failed to cleanup session");

        watch_history
            .clear_history(user_id, content_id)
            .await
            .expect("Failed to cleanup watch history");
    }

    #[tokio::test]
    #[ignore]
    async fn test_watch_history_completed_content() {
        use sqlx::postgres::PgPoolOptions;
        use std::sync::Arc;

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        use media_gateway_playback::events::NoOpProducer;
        use media_gateway_playback::session::{
            CreateSessionRequest, SessionManager, UpdatePositionRequest,
        };
        use media_gateway_playback::watch_history::WatchHistoryManager;

        let watch_history = Arc::new(WatchHistoryManager::new(pool));
        let event_producer = Arc::new(NoOpProducer);

        let session_manager = SessionManager::new(
            "redis://127.0.0.1:6379",
            "http://localhost:8083".to_string(),
            event_producer,
        )
        .expect("Failed to create session manager")
        .with_watch_history(watch_history.clone());

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        // Create session
        let create_request = CreateSessionRequest {
            user_id,
            content_id,
            device_id: "test-device".to_string(),
            duration_seconds: 3600,
            quality: None,
        };

        let response = session_manager.create(create_request).await.unwrap();
        let session_id = response.session.id;

        // Watch to 96% completion
        let update_request = UpdatePositionRequest {
            position_seconds: 3456, // 96%
            playback_state: None,
        };

        session_manager
            .update_position(session_id, update_request)
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        session_manager.delete(session_id).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create new session - should start from beginning (>95% already watched)
        let create_request2 = CreateSessionRequest {
            user_id,
            content_id,
            device_id: "test-device".to_string(),
            duration_seconds: 3600,
            quality: None,
        };

        let response2 = session_manager.create(create_request2).await.unwrap();

        // Should not resume (content already finished)
        assert_eq!(response2.resume_position_seconds, None);

        // Cleanup
        session_manager.delete(response2.session.id).await.unwrap();
        watch_history
            .clear_history(user_id, content_id)
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_service_payload_structure() {
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let progress_update = json!({
            "user_id": user_id,
            "content_id": content_id,
            "position_seconds": 120,
            "device_id": "device-123",
            "timestamp": "2024-01-01T12:00:00Z"
        });

        // Verify required fields
        assert!(progress_update["user_id"].is_string());
        assert!(progress_update["content_id"].is_string());
        assert_eq!(progress_update["position_seconds"], 120);
        assert_eq!(progress_update["device_id"], "device-123");
    }

    #[test]
    fn test_session_created_event() {
        use chrono::Utc;

        let session_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let event_json = json!({
            "session_id": session_id,
            "user_id": user_id,
            "content_id": content_id,
            "device_id": "test-device",
            "duration_seconds": 3600,
            "quality": "high",
            "timestamp": Utc::now().to_rfc3339()
        });

        // Verify structure
        assert!(event_json["session_id"].is_string());
        assert!(event_json["user_id"].is_string());
        assert_eq!(event_json["duration_seconds"], 3600);
    }

    #[test]
    fn test_position_updated_event() {
        use chrono::Utc;

        let session_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let event_json = json!({
            "session_id": session_id,
            "user_id": user_id,
            "content_id": content_id,
            "device_id": "test-device",
            "position_seconds": 120,
            "playback_state": "playing",
            "timestamp": Utc::now().to_rfc3339()
        });

        // Verify structure
        assert_eq!(event_json["position_seconds"], 120);
        assert_eq!(event_json["playback_state"], "playing");
    }

    #[test]
    fn test_session_ended_event() {
        use chrono::Utc;

        let session_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let completion_rate = 0.85;

        let event_json = json!({
            "session_id": session_id,
            "user_id": user_id,
            "content_id": content_id,
            "device_id": "test-device",
            "final_position_seconds": 3060,
            "duration_seconds": 3600,
            "completion_rate": completion_rate,
            "timestamp": Utc::now().to_rfc3339()
        });

        // Verify structure
        assert_eq!(event_json["final_position_seconds"], 3060);
        assert_eq!(event_json["duration_seconds"], 3600);
        assert_eq!(event_json["completion_rate"], 0.85);
    }

    #[test]
    fn test_completion_rate_calculation() {
        let position_seconds = 1800;
        let duration_seconds = 3600;

        let completion_rate = (position_seconds as f32 / duration_seconds as f32).min(1.0);

        assert_eq!(completion_rate, 0.5);
    }

    #[test]
    fn test_completion_rate_clamping() {
        let position_seconds = 4000;
        let duration_seconds = 3600;

        let completion_rate = (position_seconds as f32 / duration_seconds as f32).min(1.0);

        assert_eq!(completion_rate, 1.0);
    }
}
