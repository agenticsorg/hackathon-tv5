use crate::error::{AuthError, Result};
use crate::middleware::extract_user_context;
use crate::profile::{storage::ProfileStorage, AvatarUploadResponse, UpdateProfileRequest};
use actix_multipart::Multipart;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use std::io::Write;
use std::sync::Arc;
use uuid::Uuid;

const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
const ALLOWED_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/png"];

pub struct ProfileState {
    pub storage: Arc<ProfileStorage>,
    pub upload_dir: String,
}

#[get("/api/v1/users/me")]
pub async fn get_current_user(
    req: HttpRequest,
    state: web::Data<ProfileState>,
) -> Result<impl Responder> {
    let user_context = extract_user_context(&req)?;
    let user_id = Uuid::parse_str(&user_context.user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid user ID: {}", e)))?;

    let profile = state
        .storage
        .get_user_profile(user_id)
        .await?
        .ok_or(AuthError::Unauthorized)?;

    Ok(HttpResponse::Ok().json(profile))
}

#[patch("/api/v1/users/me")]
pub async fn update_current_user(
    req: HttpRequest,
    body: web::Json<UpdateProfileRequest>,
    state: web::Data<ProfileState>,
) -> Result<impl Responder> {
    let user_context = extract_user_context(&req)?;
    let user_id = Uuid::parse_str(&user_context.user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid user ID: {}", e)))?;

    // Validate request
    body.validate()?;

    let updated_profile = state.storage.update_user_profile(user_id, &body).await?;

    Ok(HttpResponse::Ok().json(updated_profile))
}

#[delete("/api/v1/users/me")]
pub async fn delete_current_user(
    req: HttpRequest,
    state: web::Data<ProfileState>,
) -> Result<impl Responder> {
    let user_context = extract_user_context(&req)?;
    let user_id = Uuid::parse_str(&user_context.user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid user ID: {}", e)))?;

    let deleted_at = state.storage.soft_delete_user(user_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Account scheduled for deletion",
        "deleted_at": deleted_at,
        "recovery_period_days": 30,
        "recoverable_until": deleted_at + chrono::Duration::days(30)
    })))
}

#[post("/api/v1/users/me/avatar")]
pub async fn upload_avatar(
    req: HttpRequest,
    mut payload: Multipart,
    state: web::Data<ProfileState>,
) -> Result<impl Responder> {
    let user_context = extract_user_context(&req)?;
    let user_id = Uuid::parse_str(&user_context.user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid user ID: {}", e)))?;

    let mut file_data = Vec::new();
    let mut content_type = String::new();

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AuthError::Internal(format!("Multipart error: {}", e)))?;

        let field_content_type = field
            .content_type()
            .ok_or_else(|| AuthError::Internal("Missing content type".to_string()))?;

        content_type = field_content_type.to_string();

        // Validate content type
        if !ALLOWED_CONTENT_TYPES.contains(&content_type.as_str()) {
            return Err(AuthError::Internal(format!(
                "Invalid content type: {}. Allowed: jpeg, png",
                content_type
            )));
        }

        // Read file data
        while let Some(chunk) = field.next().await {
            let data =
                chunk.map_err(|e| AuthError::Internal(format!("Chunk read error: {}", e)))?;

            file_data.extend_from_slice(&data);

            // Check size limit
            if file_data.len() > MAX_FILE_SIZE {
                return Err(AuthError::Internal(format!(
                    "File too large. Max size: {} bytes",
                    MAX_FILE_SIZE
                )));
            }
        }
    }

    if file_data.is_empty() {
        return Err(AuthError::Internal("No file data received".to_string()));
    }

    // Generate unique filename
    let extension = if content_type == "image/jpeg" {
        "jpg"
    } else {
        "png"
    };
    let filename = format!("{}.{}", Uuid::new_v4(), extension);
    let file_path = format!("{}/{}", state.upload_dir, filename);

    // Save file (in production, this would upload to S3/GCS)
    std::fs::create_dir_all(&state.upload_dir)
        .map_err(|e| AuthError::Internal(format!("Directory creation error: {}", e)))?;

    let mut file = std::fs::File::create(&file_path)
        .map_err(|e| AuthError::Internal(format!("File creation error: {}", e)))?;

    file.write_all(&file_data)
        .map_err(|e| AuthError::Internal(format!("File write error: {}", e)))?;

    // Update user profile with avatar URL
    let avatar_url = format!("/uploads/avatars/{}", filename);
    state
        .storage
        .update_avatar_url(user_id, avatar_url.clone())
        .await?;

    Ok(HttpResponse::Ok().json(AvatarUploadResponse { avatar_url }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::storage::ProfileStorage;
    use actix_web::{test, web, App};
    use sqlx::PgPool;

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/test_auth".to_string());

        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    async fn create_test_user(pool: &PgPool) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, display_name)
            VALUES ($1, $2, $3, $4)
            "#,
            user_id,
            format!("test{}@example.com", user_id),
            "hashed_password",
            "Test User"
        )
        .execute(pool)
        .await
        .expect("Failed to create test user");

        user_id
    }

    #[actix_web::test]
    async fn test_get_user_profile() {
        let pool = setup_test_db().await;
        let user_id = create_test_user(&pool).await;

        let storage = Arc::new(ProfileStorage::new(pool.clone()));
        let profile = storage
            .get_user_profile(user_id)
            .await
            .expect("Failed to get profile");

        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.id, user_id);
        assert_eq!(profile.display_name, "Test User");
        assert!(profile.oauth_providers.is_empty());
    }

    #[actix_web::test]
    async fn test_update_user_profile() {
        let pool = setup_test_db().await;
        let user_id = create_test_user(&pool).await;

        let storage = Arc::new(ProfileStorage::new(pool.clone()));

        let update_request = UpdateProfileRequest {
            display_name: Some("Updated Name".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            preferences: Some(serde_json::json!({"theme": "dark"})),
        };

        let updated_profile = storage
            .update_user_profile(user_id, &update_request)
            .await
            .expect("Failed to update profile");

        assert_eq!(updated_profile.display_name, "Updated Name");
        assert_eq!(
            updated_profile.avatar_url,
            Some("https://example.com/avatar.jpg".to_string())
        );
        assert_eq!(
            updated_profile.preferences,
            serde_json::json!({"theme": "dark"})
        );
    }

    #[actix_web::test]
    async fn test_soft_delete_user() {
        let pool = setup_test_db().await;
        let user_id = create_test_user(&pool).await;

        let storage = Arc::new(ProfileStorage::new(pool.clone()));

        let deleted_at = storage
            .soft_delete_user(user_id)
            .await
            .expect("Failed to soft delete user");

        assert!(deleted_at <= chrono::Utc::now());

        // Verify user is soft deleted
        let profile = storage
            .get_user_profile(user_id)
            .await
            .expect("Failed to get profile");
        assert!(profile.is_none());

        // Verify can recover
        let can_recover = storage
            .can_recover_account(user_id)
            .await
            .expect("Failed to check recovery");
        assert!(can_recover);
    }

    #[actix_web::test]
    async fn test_validate_update_request() {
        let valid_request = UpdateProfileRequest {
            display_name: Some("Valid Name".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            preferences: Some(serde_json::json!({})),
        };
        assert!(valid_request.validate().is_ok());

        let empty_name = UpdateProfileRequest {
            display_name: Some("".to_string()),
            avatar_url: None,
            preferences: None,
        };
        assert!(empty_name.validate().is_err());

        let long_name = UpdateProfileRequest {
            display_name: Some("a".repeat(101)),
            avatar_url: None,
            preferences: None,
        };
        assert!(long_name.validate().is_err());

        let long_url = UpdateProfileRequest {
            display_name: None,
            avatar_url: Some("a".repeat(501)),
            preferences: None,
        };
        assert!(long_url.validate().is_err());
    }

    #[actix_web::test]
    async fn test_audit_log_creation() {
        let pool = setup_test_db().await;
        let user_id = create_test_user(&pool).await;

        let storage = Arc::new(ProfileStorage::new(pool.clone()));

        let update_request = UpdateProfileRequest {
            display_name: Some("New Name".to_string()),
            avatar_url: None,
            preferences: None,
        };

        storage
            .update_user_profile(user_id, &update_request)
            .await
            .expect("Failed to update profile");

        let logs = storage
            .get_audit_logs(user_id, 10)
            .await
            .expect("Failed to get audit logs");

        assert!(!logs.is_empty());
        assert_eq!(logs[0].action, "profile.update");
        assert_eq!(logs[0].resource_type, "user");
    }
}
