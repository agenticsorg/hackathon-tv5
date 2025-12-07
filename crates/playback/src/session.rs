//! Playback session management

use chrono::{DateTime, Utc};
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::events::{
    PlaybackEventProducer, PositionUpdatedEvent, SessionCreatedEvent, SessionEndedEvent,
};
use crate::watch_history::WatchHistoryManager;

const SESSION_TTL_SECS: u64 = 86400; // 24 hours

/// Playback session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub device_id: String,
    pub position_seconds: u32,
    pub duration_seconds: u32,
    pub playback_state: PlaybackState,
    pub quality: VideoQuality,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackState {
    Playing,
    Paused,
    Buffering,
    Stopped,
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoQuality {
    Auto,
    Low,    // 480p
    Medium, // 720p
    High,   // 1080p
    Ultra,  // 4K
}

impl Default for VideoQuality {
    fn default() -> Self {
        VideoQuality::Auto
    }
}

/// Request to create a new playback session
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub device_id: String,
    pub duration_seconds: u32,
    pub quality: Option<VideoQuality>,
}

/// Response when creating a session
#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    #[serde(flatten)]
    pub session: PlaybackSession,
    pub resume_position_seconds: Option<u32>,
}

/// Request to update playback position
#[derive(Debug, Deserialize)]
pub struct UpdatePositionRequest {
    pub position_seconds: u32,
    pub playback_state: Option<PlaybackState>,
}

/// Session manager using Redis storage
pub struct SessionManager {
    client: Client,
    sync_service_url: String,
    http_client: reqwest::Client,
    event_producer: Arc<dyn PlaybackEventProducer>,
    watch_history: Option<Arc<WatchHistoryManager>>,
}

impl SessionManager {
    pub fn new(
        redis_url: &str,
        sync_service_url: String,
        event_producer: Arc<dyn PlaybackEventProducer>,
    ) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_millis(50))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Ok(Self {
            client,
            sync_service_url,
            http_client,
            event_producer,
            watch_history: None,
        })
    }

    /// Set watch history manager for resume position tracking
    pub fn with_watch_history(mut self, watch_history: Arc<WatchHistoryManager>) -> Self {
        self.watch_history = Some(watch_history);
        self
    }

    pub fn from_env() -> Result<Self, redis::RedisError> {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let sync_service_url = std::env::var("SYNC_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8083".to_string());

        // Initialize Kafka producer
        let event_producer: Arc<dyn PlaybackEventProducer> =
            match crate::events::KafkaPlaybackProducer::from_env() {
                Ok(producer) => {
                    tracing::info!("Kafka event producer initialized");
                    Arc::new(producer)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize Kafka producer, using no-op: {}", e);
                    Arc::new(crate::events::NoOpProducer)
                }
            };

        let mut manager = Self::new(&redis_url, sync_service_url, event_producer)?;

        // Initialize watch history manager if DATABASE_URL is set
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            use sqlx::postgres::PgPoolOptions;

            match tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    PgPoolOptions::new()
                        .max_connections(5)
                        .connect(&database_url)
                        .await
                })
            }) {
                Ok(pool) => {
                    let watch_history = Arc::new(WatchHistoryManager::new(pool));
                    manager = manager.with_watch_history(watch_history);
                    tracing::info!("Watch history manager initialized");
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize watch history manager: {}", e);
                }
            }
        } else {
            tracing::warn!("DATABASE_URL not set, watch history disabled");
        }

        Ok(manager)
    }

    async fn get_conn(&self) -> Result<MultiplexedConnection, redis::RedisError> {
        self.client.get_multiplexed_async_connection().await
    }

    /// Create new playback session with resume position support
    pub async fn create(
        &self,
        request: CreateSessionRequest,
    ) -> Result<CreateSessionResponse, SessionError> {
        let mut conn = self
            .get_conn()
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Query watch history for resume position
        let resume_position_seconds = if let Some(watch_history) = &self.watch_history {
            match watch_history
                .get_resume_position(request.user_id, request.content_id)
                .await
            {
                Ok(pos) => pos,
                Err(e) => {
                    tracing::warn!("Failed to get resume position: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let now = Utc::now();
        let session = PlaybackSession {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            content_id: request.content_id,
            device_id: request.device_id.clone(),
            position_seconds: 0,
            duration_seconds: request.duration_seconds,
            playback_state: PlaybackState::Playing,
            quality: request.quality.clone().unwrap_or_default(),
            started_at: now,
            updated_at: now,
        };

        let key = format!("session:{}", session.id);
        let value = serde_json::to_string(&session)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        conn.set_ex::<_, _, ()>(&key, value, SESSION_TTL_SECS)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Also index by user for lookup
        let user_key = format!("user:{}:sessions", session.user_id);
        conn.sadd::<_, _, ()>(&user_key, session.id.to_string())
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;
        conn.expire::<_, ()>(&user_key, SESSION_TTL_SECS as i64)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Publish session created event
        let event = SessionCreatedEvent {
            session_id: session.id,
            user_id: session.user_id,
            content_id: session.content_id,
            device_id: request.device_id,
            duration_seconds: request.duration_seconds,
            quality: format!("{:?}", request.quality.unwrap_or_default()),
            timestamp: now,
        };

        let producer = self.event_producer.clone();
        tokio::spawn(async move {
            if let Err(e) = producer.publish_session_created(event).await {
                tracing::error!("Failed to publish session created event: {}", e);
            }
        });

        Ok(CreateSessionResponse {
            session,
            resume_position_seconds,
        })
    }

    /// Get session by ID
    pub async fn get(&self, session_id: Uuid) -> Result<Option<PlaybackSession>, SessionError> {
        let mut conn = self
            .get_conn()
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let key = format!("session:{}", session_id);
        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        match value {
            Some(v) => {
                let session: PlaybackSession = serde_json::from_str(&v)
                    .map_err(|e| SessionError::Serialization(e.to_string()))?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// Update playback position and watch history
    pub async fn update_position(
        &self,
        session_id: Uuid,
        request: UpdatePositionRequest,
    ) -> Result<PlaybackSession, SessionError> {
        let mut session = self.get(session_id).await?.ok_or(SessionError::NotFound)?;

        session.position_seconds = request.position_seconds;
        session.updated_at = Utc::now();

        if let Some(state) = request.playback_state {
            session.playback_state = state;
        }

        // Check if playback completed
        if session.position_seconds >= session.duration_seconds {
            session.playback_state = PlaybackState::Ended;
        }

        let mut conn = self
            .get_conn()
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let key = format!("session:{}", session.id);
        let value = serde_json::to_string(&session)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        // Get remaining TTL and preserve it
        let ttl: i64 = conn
            .ttl(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let ttl = if ttl > 0 {
            ttl as u64
        } else {
            SESSION_TTL_SECS
        };

        conn.set_ex::<_, _, ()>(&key, value, ttl)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Update watch history
        if let Some(watch_history) = &self.watch_history {
            let user_id = session.user_id;
            let content_id = session.content_id;
            let position = session.position_seconds;
            let duration = session.duration_seconds;
            let wh = watch_history.clone();

            tokio::spawn(async move {
                if let Err(e) = wh
                    .update_watch_history(user_id, content_id, position, duration)
                    .await
                {
                    tracing::error!("Failed to update watch history: {}", e);
                }
            });
        }

        // Fire-and-forget call to sync service
        self.notify_sync_service(&session).await;

        // Publish position updated event
        let event = PositionUpdatedEvent {
            session_id: session.id,
            user_id: session.user_id,
            content_id: session.content_id,
            device_id: session.device_id.clone(),
            position_seconds: session.position_seconds,
            playback_state: format!("{:?}", session.playback_state),
            timestamp: session.updated_at,
        };

        let producer = self.event_producer.clone();
        tokio::spawn(async move {
            if let Err(e) = producer.publish_position_updated(event).await {
                tracing::error!("Failed to publish position updated event: {}", e);
            }
        });

        Ok(session)
    }

    /// Notify sync service of position update (fire-and-forget)
    async fn notify_sync_service(&self, session: &PlaybackSession) {
        let progress_update = serde_json::json!({
            "user_id": session.user_id,
            "content_id": session.content_id,
            "position_seconds": session.position_seconds,
            "device_id": session.device_id,
            "timestamp": session.updated_at.to_rfc3339(),
        });

        let url = format!("{}/api/v1/sync/progress", self.sync_service_url);

        // Fire-and-forget HTTP call
        let client = self.http_client.clone();
        tokio::spawn(async move {
            match client.post(&url).json(&progress_update).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::debug!("Sync service notified successfully");
                }
                Ok(resp) => {
                    tracing::warn!("Sync service responded with status: {}", resp.status());
                }
                Err(e) => {
                    tracing::warn!("Failed to notify sync service: {}", e);
                }
            }
        });
    }

    /// Delete session and update final watch history
    pub async fn delete(&self, session_id: Uuid) -> Result<(), SessionError> {
        let session = self.get(session_id).await?;

        let mut conn = self
            .get_conn()
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let key = format!("session:{}", session_id);
        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // Remove from user index and publish session ended event
        if let Some(s) = session {
            let user_key = format!("user:{}:sessions", s.user_id);
            conn.srem::<_, _, ()>(&user_key, session_id.to_string())
                .await
                .map_err(|e| SessionError::Storage(e.to_string()))?;

            // Final update to watch history
            if let Some(watch_history) = &self.watch_history {
                let user_id = s.user_id;
                let content_id = s.content_id;
                let position = s.position_seconds;
                let duration = s.duration_seconds;
                let wh = watch_history.clone();

                tokio::spawn(async move {
                    if let Err(e) = wh
                        .update_watch_history(user_id, content_id, position, duration)
                        .await
                    {
                        tracing::error!("Failed to update final watch history: {}", e);
                    }
                });
            }

            // Calculate completion rate
            let completion_rate = if s.duration_seconds > 0 {
                (s.position_seconds as f32 / s.duration_seconds as f32).min(1.0)
            } else {
                0.0
            };

            // Publish session ended event
            let event = SessionEndedEvent {
                session_id: s.id,
                user_id: s.user_id,
                content_id: s.content_id,
                device_id: s.device_id,
                final_position_seconds: s.position_seconds,
                duration_seconds: s.duration_seconds,
                completion_rate,
                timestamp: Utc::now(),
            };

            let producer = self.event_producer.clone();
            tokio::spawn(async move {
                if let Err(e) = producer.publish_session_ended(event).await {
                    tracing::error!("Failed to publish session ended event: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Get all sessions for a user
    pub async fn get_user_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<PlaybackSession>, SessionError> {
        let mut conn = self
            .get_conn()
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let user_key = format!("user:{}:sessions", user_id);
        let session_ids: Vec<String> = conn
            .smembers(&user_key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let mut sessions = Vec::new();
        for id_str in session_ids {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Some(session) = self.get(id).await? {
                    sessions.push(session);
                }
            }
        }

        Ok(sessions)
    }

    /// Check health
    pub async fn is_healthy(&self) -> bool {
        match self.get_conn().await {
            Ok(mut conn) => redis::cmd("PING")
                .query_async::<_, String>(&mut conn)
                .await
                .is_ok(),
            Err(_) => false,
        }
    }
}

/// Session errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl actix_web::ResponseError for SessionError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            SessionError::NotFound => actix_web::HttpResponse::NotFound().json(serde_json::json!({
                "error": "Session not found"
            })),
            _ => actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "error": self.to_string()
            })),
        }
    }
}
