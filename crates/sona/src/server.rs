//! SONA Personalization Engine HTTP Server
//!
//! Actix-web server providing REST API for personalization services.
//! Runs on port 8082 as specified in SPARC architecture.

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use media_gateway_sona::{
    SonaEngine, SonaConfig, GenerateRecommendations, BuildUserPreferenceVector,
    UpdateUserLoRA, UserProfile, UserLoRAAdapter, ViewingEvent,
};

/// Application state
struct AppState {
    engine: Arc<SonaEngine>,
    lora_storage: Arc<media_gateway_sona::LoRAStorage>,
    experiment_repo: Arc<media_gateway_sona::ExperimentRepository>,
    db_pool: sqlx::PgPool,
}

impl AppState {
    /// Load user profile from database
    async fn load_user_profile(&self, user_id: Uuid) -> anyhow::Result<UserProfile> {
        // Fetch viewing history from database
        let viewing_history = sqlx::query_as::<_, ViewingEventRow>(
            r#"
            SELECT content_id, timestamp, completion_rate, rating, is_rewatch, dismissed
            FROM viewing_events
            WHERE user_id = $1
            ORDER BY timestamp DESC
            LIMIT 100
            "#
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await?;

        let events: Vec<ViewingEvent> = viewing_history.into_iter().map(|row| row.into()).collect();

        // Get content embedding function
        let get_embedding = |content_id: Uuid| -> anyhow::Result<Vec<f32>> {
            // In production, query embedding service
            Ok(vec![0.0; 512])
        };

        // Build preference vector
        let preference_vector = BuildUserPreferenceVector::execute(
            user_id,
            &events,
            get_embedding,
        ).await?;

        Ok(UserProfile {
            user_id,
            preference_vector,
            genre_affinities: std::collections::HashMap::new(),
            temporal_patterns: Default::default(),
            mood_history: Vec::new(),
            interaction_count: events.len(),
            last_update_time: chrono::Utc::now(),
        })
    }
}

#[derive(sqlx::FromRow)]
struct ViewingEventRow {
    content_id: Uuid,
    timestamp: chrono::DateTime<chrono::Utc>,
    completion_rate: f32,
    rating: Option<i16>,
    is_rewatch: bool,
    dismissed: bool,
}

impl From<ViewingEventRow> for ViewingEvent {
    fn from(row: ViewingEventRow) -> Self {
        ViewingEvent {
            content_id: row.content_id,
            timestamp: row.timestamp,
            completion_rate: row.completion_rate,
            rating: row.rating.map(|r| r as u8),
            is_rewatch: row.is_rewatch,
            dismissed: row.dismissed,
        }
    }
}

/// Health check endpoint
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "sona-personalization-engine",
        "version": "0.1.0"
    }))
}

/// Recommendation request
#[derive(Debug, Deserialize)]
struct RecommendationRequest {
    user_id: Uuid,
    context: Option<RecommendationContextDto>,
    limit: Option<usize>,
    exclude_watched: Option<bool>,
    diversity_threshold: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct RecommendationContextDto {
    mood: Option<String>,
    time_of_day: Option<String>,
    device_type: Option<String>,
    viewing_with: Option<Vec<String>>,
}

/// Recommendation response
#[derive(Debug, Serialize)]
struct RecommendationResponse {
    recommendations: Vec<RecommendationDto>,
    generated_at: String,
    ttl_seconds: u32,
}

#[derive(Debug, Serialize)]
struct RecommendationDto {
    content_id: Uuid,
    confidence_score: f32,
    recommendation_type: String,
    based_on: Vec<String>,
    explanation: String,
}

/// POST /api/v1/recommendations
async fn get_recommendations(
    req: web::Json<RecommendationRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Load user profile
    let profile = match state.load_user_profile(req.user_id).await {
        Ok(profile) => profile,
        Err(e) => {
            tracing::error!("Failed to load user profile: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to load user profile",
                "message": e.to_string()
            }));
        }
    };

    // Load LoRA adapter if available
    let lora_adapter = state.lora_storage.load_adapter(req.user_id, "default").await.ok();

    // Convert context
    let context = req.context.as_ref().map(|ctx| {
        media_gateway_sona::RecommendationContext {
            mood: ctx.mood.clone(),
            time_of_day: ctx.time_of_day.clone(),
            device_type: ctx.device_type.clone(),
            viewing_with: ctx.viewing_with.clone().unwrap_or_default(),
        }
    });

    // Get content embedding function (simulated for now)
    let get_embedding = |content_id: Uuid| -> anyhow::Result<Vec<f32>> {
        // In production, this would query the embedding database
        Ok(vec![0.0; 512])
    };

    // Generate recommendations
    match GenerateRecommendations::execute(
        req.user_id,
        &profile,
        context,
        lora_adapter.as_ref(),
        get_embedding,
    ).await {
        Ok(mut recommendations) => {
            // Check if user is in any active recommendation experiments
            if let Ok(experiments) = state.experiment_repo.list_running_experiments().await {
                for experiment in experiments {
                    // Try to assign user to variant
                    if let Ok(variant) = state.experiment_repo.assign_variant(experiment.id, req.user_id).await {
                        // Apply experiment variant to recommendations
                        for rec in &mut recommendations {
                            rec.experiment_variant = Some(format!("{}:{}", experiment.name, variant.name));
                        }

                        // Record exposure
                        let _ = state.experiment_repo.record_exposure(
                            experiment.id,
                            variant.id,
                            req.user_id,
                            Some(serde_json::json!({
                                "endpoint": "recommendations",
                                "num_recommendations": recommendations.len()
                            }))
                        ).await;

                        break; // Only assign to first matching experiment
                    }
                }
            }

            let response = RecommendationResponse {
                recommendations: recommendations.into_iter().map(|r| RecommendationDto {
                    content_id: r.content_id,
                    confidence_score: r.confidence_score,
                    recommendation_type: format!("{:?}", r.recommendation_type),
                    based_on: r.based_on,
                    explanation: r.explanation,
                }).collect(),
                generated_at: chrono::Utc::now().to_rfc3339(),
                ttl_seconds: 3600,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!("Recommendation generation failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Recommendation generation failed",
                "message": e.to_string()
            }))
        }
    }
}

/// Similar content request
#[derive(Debug, Deserialize)]
struct SimilarContentRequest {
    content_id: Uuid,
    limit: Option<usize>,
}

/// POST /api/v1/recommendations/similar
async fn get_similar_content(
    _req: web::Json<SimilarContentRequest>,
    _engine: web::Data<Arc<SonaEngine>>,
) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "similar_content": []
    }))
}

/// Personalization score request
#[derive(Debug, Deserialize)]
struct PersonalizationScoreRequest {
    user_id: Uuid,
    content_id: Uuid,
}

/// POST /api/v1/personalization/score
async fn get_personalization_score(
    req: web::Json<PersonalizationScoreRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Load user profile
    let profile = match state.load_user_profile(req.user_id).await {
        Ok(profile) => profile,
        Err(e) => {
            tracing::error!("Failed to load user profile: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to load user profile",
                "message": e.to_string()
            }));
        }
    };

    // Load LoRA adapter
    let lora_adapter = match state.lora_storage.load_adapter(req.user_id, "default").await {
        Ok(adapter) => adapter,
        Err(e) => {
            tracing::warn!("LoRA adapter not found, using base model: {}", e);
            // Create new adapter for user
            let mut adapter = UserLoRAAdapter::new(req.user_id);
            adapter.initialize_random();
            adapter
        }
    };

    // Get content embedding
    let content_embedding = vec![0.0; 512]; // In production, query embedding service

    // Compute LoRA personalization score
    let lora_score = match media_gateway_sona::lora::compute_lora_score(
        &lora_adapter,
        &content_embedding,
        &profile.preference_vector,
    ) {
        Ok(score) => score,
        Err(e) => {
            tracing::error!("LoRA scoring failed: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "LoRA scoring failed",
                "message": e.to_string()
            }));
        }
    };

    // Compute component scores (simulated for now)
    let collaborative_score = 0.35;
    let content_based_score = 0.25;
    let graph_score = 0.30;
    let context_score = 0.10;

    let total_score = (collaborative_score + content_based_score + graph_score + context_score)
        * (1.0 + lora_score * 0.3);

    HttpResponse::Ok().json(serde_json::json!({
        "user_id": req.user_id,
        "content_id": req.content_id,
        "score": total_score.min(1.0).max(0.0),
        "components": {
            "collaborative": collaborative_score,
            "content_based": content_based_score,
            "graph_based": graph_score,
            "context": context_score,
            "lora_boost": lora_score
        }
    }))
}

/// User profile update request
#[derive(Debug, Deserialize)]
struct ProfileUpdateRequest {
    user_id: Uuid,
    viewing_events: Vec<ViewingEventDto>,
}

#[derive(Debug, Deserialize)]
struct ViewingEventDto {
    content_id: Uuid,
    timestamp: String,
    completion_rate: f32,
    rating: Option<u8>,
    is_rewatch: bool,
    dismissed: bool,
}

/// POST /api/v1/profile/update
async fn update_profile(
    req: web::Json<ProfileUpdateRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Convert viewing events
    let events: Vec<ViewingEvent> = req.viewing_events.iter().map(|dto| {
        ViewingEvent {
            content_id: dto.content_id,
            timestamp: chrono::DateTime::parse_from_rfc3339(&dto.timestamp)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc),
            completion_rate: dto.completion_rate,
            rating: dto.rating,
            is_rewatch: dto.is_rewatch,
            dismissed: dto.dismissed,
        }
    }).collect();

    // Store events in database
    for event in &events {
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO viewing_events
            (user_id, content_id, timestamp, completion_rate, rating, is_rewatch, dismissed)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id, content_id, timestamp) DO UPDATE
            SET completion_rate = EXCLUDED.completion_rate,
                rating = EXCLUDED.rating,
                is_rewatch = EXCLUDED.is_rewatch,
                dismissed = EXCLUDED.dismissed
            "#
        )
        .bind(req.user_id)
        .bind(event.content_id)
        .bind(event.timestamp)
        .bind(event.completion_rate)
        .bind(event.rating.map(|r| r as i16))
        .bind(event.is_rewatch)
        .bind(event.dismissed)
        .execute(&state.db_pool)
        .await {
            tracing::error!("Failed to store viewing event: {}", e);
        }
    }

    // Get content embedding function
    let get_embedding = |content_id: Uuid| -> anyhow::Result<Vec<f32>> {
        Ok(vec![0.0; 512])
    };

    // Update preference vector
    match BuildUserPreferenceVector::execute(req.user_id, &events, get_embedding).await {
        Ok(preference_vector) => {
            // Store updated preference vector (simulated)
            tracing::info!("Updated preference vector for user {}", req.user_id);

            HttpResponse::Ok().json(serde_json::json!({
                "status": "updated",
                "user_id": req.user_id,
                "events_processed": req.viewing_events.len(),
                "preference_vector_dim": preference_vector.len()
            }))
        }
        Err(e) => {
            tracing::error!("Failed to update preference vector: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update preference vector",
                "message": e.to_string()
            }))
        }
    }
}

/// LoRA training request
#[derive(Debug, Deserialize)]
struct LoraTrainingRequest {
    user_id: Uuid,
    force: Option<bool>,
}

/// POST /api/v1/lora/train
async fn trigger_lora_training(
    req: web::Json<LoraTrainingRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Load or create LoRA adapter
    let mut adapter = match state.lora_storage.load_adapter(req.user_id, "default").await {
        Ok(adapter) => adapter,
        Err(_) => {
            let mut adapter = UserLoRAAdapter::new(req.user_id);
            adapter.initialize_random();
            adapter
        }
    };

    // Load user profile
    let profile = match state.load_user_profile(req.user_id).await {
        Ok(profile) => profile,
        Err(e) => {
            tracing::error!("Failed to load user profile: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to load user profile",
                "message": e.to_string()
            }));
        }
    };

    // Fetch recent viewing events
    let viewing_history = match sqlx::query_as::<_, ViewingEventRow>(
        r#"
        SELECT content_id, timestamp, completion_rate, rating, is_rewatch, dismissed
        FROM viewing_events
        WHERE user_id = $1
        ORDER BY timestamp DESC
        LIMIT 50
        "#
    )
    .bind(req.user_id)
    .fetch_all(&state.db_pool)
    .await {
        Ok(history) => history,
        Err(e) => {
            tracing::error!("Failed to fetch viewing history: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch viewing history",
                "message": e.to_string()
            }));
        }
    };

    let events: Vec<ViewingEvent> = viewing_history.into_iter().map(|row| row.into()).collect();

    if events.len() < 10 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Insufficient training data",
            "message": "At least 10 viewing events required for LoRA training",
            "current_count": events.len()
        }));
    }

    // Get content embedding function
    let get_embedding = |content_id: Uuid| -> anyhow::Result<Vec<f32>> {
        Ok(vec![0.0; 512])
    };

    // Train LoRA adapter
    let start_time = std::time::Instant::now();
    match UpdateUserLoRA::execute(
        &mut adapter,
        &events,
        get_embedding,
        &profile.preference_vector,
    ).await {
        Ok(_) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Save trained adapter
            if let Err(e) = state.lora_storage.save_adapter(&adapter, "default").await {
                tracing::error!("Failed to save LoRA adapter: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to save trained adapter",
                    "message": e.to_string()
                }));
            }

            HttpResponse::Ok().json(serde_json::json!({
                "status": "training_completed",
                "user_id": req.user_id,
                "duration_ms": duration_ms,
                "training_iterations": adapter.training_iterations,
                "events_used": events.len()
            }))
        }
        Err(e) => {
            tracing::error!("LoRA training failed: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "LoRA training failed",
                "message": e.to_string()
            }))
        }
    }
}

/// Create experiment request
#[derive(Debug, Deserialize)]
struct CreateExperimentRequest {
    name: String,
    description: Option<String>,
    traffic_allocation: f32,
    variants: Vec<CreateVariantDto>,
}

#[derive(Debug, Deserialize)]
struct CreateVariantDto {
    name: String,
    weight: f32,
    config: serde_json::Value,
}

/// POST /api/v1/experiments
async fn create_experiment(
    req: web::Json<CreateExperimentRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    use media_gateway_sona::{Experiment, ExperimentStatus};

    let mut experiment = Experiment::new(
        req.name.clone(),
        req.description.clone(),
        req.traffic_allocation,
    );

    for variant_dto in &req.variants {
        experiment.add_variant(
            variant_dto.name.clone(),
            variant_dto.weight,
            variant_dto.config.clone(),
        );
    }

    match state.experiment_repo.create_experiment(&experiment).await {
        Ok(experiment_id) => {
            HttpResponse::Ok().json(serde_json::json!({
                "experiment_id": experiment_id,
                "name": experiment.name,
                "status": "draft",
                "variants": experiment.variants.len()
            }))
        }
        Err(e) => {
            tracing::error!("Failed to create experiment: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create experiment",
                "message": e.to_string()
            }))
        }
    }
}

/// GET /api/v1/experiments/{experiment_id}
async fn get_experiment(
    experiment_id: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    match state.experiment_repo.get_experiment(*experiment_id).await {
        Ok(experiment) => HttpResponse::Ok().json(experiment),
        Err(e) => {
            tracing::error!("Failed to get experiment: {}", e);
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Experiment not found",
                "message": e.to_string()
            }))
        }
    }
}

/// Update experiment status request
#[derive(Debug, Deserialize)]
struct UpdateExperimentStatusRequest {
    status: String,
}

/// PUT /api/v1/experiments/{experiment_id}/status
async fn update_experiment_status(
    experiment_id: web::Path<Uuid>,
    req: web::Json<UpdateExperimentStatusRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    use media_gateway_sona::ExperimentStatus;

    let status = match req.status.as_str() {
        "draft" => ExperimentStatus::Draft,
        "running" => ExperimentStatus::Running,
        "paused" => ExperimentStatus::Paused,
        "completed" => ExperimentStatus::Completed,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid status",
                "valid_statuses": ["draft", "running", "paused", "completed"]
            }));
        }
    };

    match state.experiment_repo.update_status(*experiment_id, status).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "experiment_id": *experiment_id,
            "status": req.status
        })),
        Err(e) => {
            tracing::error!("Failed to update experiment status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update status",
                "message": e.to_string()
            }))
        }
    }
}

/// GET /api/v1/experiments
async fn list_experiments(state: web::Data<AppState>) -> impl Responder {
    match state.experiment_repo.list_running_experiments().await {
        Ok(experiments) => HttpResponse::Ok().json(serde_json::json!({
            "experiments": experiments,
            "count": experiments.len()
        })),
        Err(e) => {
            tracing::error!("Failed to list experiments: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list experiments",
                "message": e.to_string()
            }))
        }
    }
}

/// Record conversion request
#[derive(Debug, Deserialize)]
struct RecordConversionRequest {
    experiment_id: Uuid,
    user_id: Uuid,
    metric_name: String,
    value: f32,
    metadata: Option<serde_json::Value>,
}

/// POST /api/v1/experiments/conversions
async fn record_conversion(
    req: web::Json<RecordConversionRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Get user's variant assignment
    let variant = match state.experiment_repo.assign_variant(req.experiment_id, req.user_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("User not in experiment: {}", e);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "User not in experiment"
            }));
        }
    };

    match state.experiment_repo.record_conversion(
        req.experiment_id,
        variant.id,
        req.user_id,
        &req.metric_name,
        req.value,
        req.metadata.clone(),
    ).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "recorded",
            "experiment_id": req.experiment_id,
            "variant": variant.name,
            "metric": req.metric_name,
            "value": req.value
        })),
        Err(e) => {
            tracing::error!("Failed to record conversion: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to record conversion",
                "message": e.to_string()
            }))
        }
    }
}

/// GET /api/v1/experiments/{experiment_id}/metrics
async fn get_experiment_metrics(
    experiment_id: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    match state.experiment_repo.get_experiment_metrics(*experiment_id).await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => {
            tracing::error!("Failed to get experiment metrics: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get metrics",
                "message": e.to_string()
            }))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .json()
        .init();

    tracing::info!("Starting SONA Personalization Engine on port 8082");

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/media_gateway".to_string());

    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Initialize SONA engine
    let config = SonaConfig::default();
    let engine = Arc::new(SonaEngine::new(config));

    // Initialize LoRA storage
    let lora_storage = Arc::new(media_gateway_sona::LoRAStorage::new(db_pool.clone()));

    // Initialize experiment repository
    let experiment_repo = Arc::new(media_gateway_sona::ExperimentRepository::new(db_pool.clone()));

    // Create app state
    let app_state = web::Data::new(AppState {
        engine,
        lora_storage,
        experiment_repo,
        db_pool,
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api/v1")
                    .route("/recommendations", web::post().to(get_recommendations))
                    .route("/recommendations/similar", web::post().to(get_similar_content))
                    .route("/personalization/score", web::post().to(get_personalization_score))
                    .route("/profile/update", web::post().to(update_profile))
                    .route("/lora/train", web::post().to(trigger_lora_training))
                    // A/B Testing endpoints
                    .route("/experiments", web::post().to(create_experiment))
                    .route("/experiments", web::get().to(list_experiments))
                    .route("/experiments/{experiment_id}", web::get().to(get_experiment))
                    .route("/experiments/{experiment_id}/status", web::put().to(update_experiment_status))
                    .route("/experiments/{experiment_id}/metrics", web::get().to(get_experiment_metrics))
                    .route("/experiments/conversions", web::post().to(record_conversion))
            )
    })
    .bind(("0.0.0.0", 8082))?
    .run()
    .await
}
