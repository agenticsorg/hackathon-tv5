use crate::{
    error::{AuthError, Result},
    jwt::JwtManager,
    middleware::extract_user_context,
};
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use media_gateway_core::audit::{AuditAction, AuditFilter, AuditLogger, PostgresAuditLogger};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub search: Option<String>,
    pub status: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

impl ListUsersQuery {
    pub fn validate(&mut self) -> Result<()> {
        if self.per_page > 100 {
            self.per_page = 100;
        }
        if self.per_page == 0 {
            self.per_page = 20;
        }
        if self.page == 0 {
            self.page = 1;
        }
        Ok(())
    }

    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.per_page) as i64
    }

    pub fn limit(&self) -> i64 {
        self.per_page as i64
    }

    pub fn get_sort_column(&self) -> &str {
        match self.sort_by.as_deref() {
            Some("email") => "email",
            Some("display_name") => "display_name",
            Some("created_at") => "created_at",
            _ => "created_at",
        }
    }

    pub fn get_sort_direction(&self) -> &str {
        match self.sort_order.as_deref() {
            Some("asc") => "ASC",
            Some("desc") => "DESC",
            _ => "DESC",
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserListItem {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub email_verified: bool,
    pub suspended: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ListUsersResponse {
    pub users: Vec<UserListItem>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserDetail {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub email_verified: bool,
    pub suspended: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserActivity {
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserDetailResponse {
    pub user: UserDetail,
    pub recent_activity: Vec<UserActivity>,
    pub session_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub role: Option<String>,
    pub suspended: Option<bool>,
    pub email_verified: Option<bool>,
}

impl AdminUpdateUserRequest {
    pub fn validate(&self) -> Result<()> {
        if let Some(ref role) = self.role {
            match role.as_str() {
                "user" | "premium" | "admin" => Ok(()),
                _ => Err(AuthError::Internal("Invalid role".to_string())),
            }
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ImpersonationTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub original_admin_id: String,
    pub impersonated_user_id: String,
}

#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub admin_user_id: Uuid,
    pub action: String,
    pub target_user_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    #[serde(default = "default_audit_page")]
    pub page: u32,
    #[serde(default = "default_audit_per_page")]
    pub per_page: u32,
}

fn default_audit_page() -> u32 {
    1
}

fn default_audit_per_page() -> u32 {
    50
}

impl AuditLogsQuery {
    pub fn validate(&mut self) -> Result<()> {
        if self.per_page > 200 {
            self.per_page = 200;
        }
        if self.per_page == 0 {
            self.per_page = 50;
        }
        if self.page == 0 {
            self.page = 1;
        }
        Ok(())
    }

    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.per_page) as i64
    }

    pub fn limit(&self) -> i64 {
        self.per_page as i64
    }

    pub fn to_audit_filter(&self) -> Result<AuditFilter> {
        let start_date = if let Some(ref date_str) = self.start_date {
            Some(
                DateTime::parse_from_rfc3339(date_str)
                    .map_err(|_| AuthError::Internal("Invalid start_date format".to_string()))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        let end_date = if let Some(ref date_str) = self.end_date {
            Some(
                DateTime::parse_from_rfc3339(date_str)
                    .map_err(|_| AuthError::Internal("Invalid end_date format".to_string()))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        let action = if let Some(ref action_str) = self.action {
            AuditAction::from_str(action_str)
        } else {
            None
        };

        Ok(AuditFilter {
            start_date,
            end_date,
            user_id: self.user_id,
            action,
            resource_type: self.resource_type.clone(),
            limit: Some(self.limit()),
            offset: Some(self.offset()),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AuditLogsResponse {
    pub logs: Vec<media_gateway_core::audit::AuditEvent>,
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

// ============================================================================
// Admin Endpoints
// ============================================================================

#[get("/api/v1/admin/users")]
pub async fn list_users(
    req: HttpRequest,
    query: web::Query<ListUsersQuery>,
    db: web::Data<PgPool>,
) -> Result<impl Responder> {
    let admin_context = extract_user_context(&req)?;
    let mut query = query.into_inner();
    query.validate()?;

    log_audit_action(
        &db,
        &admin_context.user_id,
        "list_users",
        None,
        serde_json::json!({
            "page": query.page,
            "per_page": query.per_page,
            "filters": {
                "search": query.search,
                "status": query.status,
            }
        }),
    )
    .await?;

    let sort_column = query.get_sort_column();
    let sort_direction = query.get_sort_direction();

    let (users, total) = if let Some(ref search) = query.search {
        let search_pattern = format!("%{}%", search);
        let status_filter = query.status.as_deref();

        let users_query = format!(
            "SELECT id, email, display_name, role, email_verified, suspended, created_at, last_login
             FROM users
             WHERE (email ILIKE $1 OR display_name ILIKE $1)
             {}
             ORDER BY {} {}
             LIMIT $2 OFFSET $3",
            if status_filter.is_some() {
                "AND CASE WHEN $4 = 'active' THEN NOT suspended WHEN $4 = 'suspended' THEN suspended ELSE true END"
            } else {
                ""
            },
            sort_column,
            sort_direction
        );

        let count_query = format!(
            "SELECT COUNT(*) FROM users WHERE (email ILIKE $1 OR display_name ILIKE $1) {}",
            if status_filter.is_some() {
                "AND CASE WHEN $2 = 'active' THEN NOT suspended WHEN $2 = 'suspended' THEN suspended ELSE true END"
            } else {
                ""
            }
        );

        let users: Vec<UserListItem> = if let Some(status) = status_filter {
            sqlx::query_as(&users_query)
                .bind(&search_pattern)
                .bind(query.limit())
                .bind(query.offset())
                .bind(status)
                .fetch_all(db.as_ref())
                .await?
        } else {
            sqlx::query_as(&users_query)
                .bind(&search_pattern)
                .bind(query.limit())
                .bind(query.offset())
                .fetch_all(db.as_ref())
                .await?
        };

        let total: (i64,) = if let Some(status) = status_filter {
            sqlx::query_as(&count_query)
                .bind(&search_pattern)
                .bind(status)
                .fetch_one(db.as_ref())
                .await?
        } else {
            sqlx::query_as(&count_query)
                .bind(&search_pattern)
                .fetch_one(db.as_ref())
                .await?
        };

        (users, total.0)
    } else {
        let status_filter = query.status.as_deref();

        let users_query = format!(
            "SELECT id, email, display_name, role, email_verified, suspended, created_at, last_login
             FROM users
             {}
             ORDER BY {} {}
             LIMIT $1 OFFSET $2",
            if status_filter.is_some() {
                "WHERE CASE WHEN $3 = 'active' THEN NOT suspended WHEN $3 = 'suspended' THEN suspended ELSE true END"
            } else {
                ""
            },
            sort_column,
            sort_direction
        );

        let count_query = if status_filter.is_some() {
            "SELECT COUNT(*) FROM users WHERE CASE WHEN $1 = 'active' THEN NOT suspended WHEN $1 = 'suspended' THEN suspended ELSE true END"
        } else {
            "SELECT COUNT(*) FROM users"
        };

        let users: Vec<UserListItem> = if let Some(status) = status_filter {
            sqlx::query_as(&users_query)
                .bind(query.limit())
                .bind(query.offset())
                .bind(status)
                .fetch_all(db.as_ref())
                .await?
        } else {
            sqlx::query_as(&users_query)
                .bind(query.limit())
                .bind(query.offset())
                .fetch_all(db.as_ref())
                .await?
        };

        let total: (i64,) = if let Some(status) = status_filter {
            sqlx::query_as(count_query)
                .bind(status)
                .fetch_one(db.as_ref())
                .await?
        } else {
            sqlx::query_as(count_query).fetch_one(db.as_ref()).await?
        };

        (users, total.0)
    };

    let total_pages = ((total as f64) / (query.per_page as f64)).ceil() as u32;

    Ok(HttpResponse::Ok().json(ListUsersResponse {
        users,
        total,
        page: query.page,
        per_page: query.per_page,
        total_pages,
    }))
}

#[get("/api/v1/admin/users/{id}")]
pub async fn get_user_detail(
    req: HttpRequest,
    path: web::Path<Uuid>,
    db: web::Data<PgPool>,
) -> Result<impl Responder> {
    let admin_context = extract_user_context(&req)?;
    let user_id = path.into_inner();

    log_audit_action(
        &db,
        &admin_context.user_id,
        "get_user_detail",
        Some(&user_id.to_string()),
        serde_json::json!({ "user_id": user_id }),
    )
    .await?;

    let user: UserDetail = sqlx::query_as(
        "SELECT id, email, display_name, role, email_verified, suspended, created_at, last_login
         FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(db.as_ref())
    .await?
    .ok_or(AuthError::Internal("User not found".to_string()))?;

    let recent_activity: Vec<UserActivity> = sqlx::query_as(
        "SELECT event_type, timestamp, ip_address, user_agent
         FROM audit_logs
         WHERE target_user_id = $1
         ORDER BY timestamp DESC
         LIMIT 20",
    )
    .bind(user_id)
    .fetch_all(db.as_ref())
    .await
    .unwrap_or_default();

    let session_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE user_id = $1 AND expires_at > NOW()")
            .bind(user_id.to_string())
            .fetch_one(db.as_ref())
            .await?;

    Ok(HttpResponse::Ok().json(UserDetailResponse {
        user,
        recent_activity,
        session_count: session_count.0,
    }))
}

#[patch("/api/v1/admin/users/{id}")]
pub async fn update_user(
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<AdminUpdateUserRequest>,
    db: web::Data<PgPool>,
) -> Result<impl Responder> {
    let admin_context = extract_user_context(&req)?;
    let user_id = path.into_inner();
    let update_req = body.into_inner();
    update_req.validate()?;

    log_audit_action(
        &db,
        &admin_context.user_id,
        "update_user",
        Some(&user_id.to_string()),
        serde_json::json!({
            "user_id": user_id,
            "changes": {
                "role": update_req.role,
                "suspended": update_req.suspended,
                "email_verified": update_req.email_verified,
            }
        }),
    )
    .await?;

    let mut updates = vec![];
    let mut param_count = 2;

    if update_req.role.is_some() {
        updates.push(format!("role = ${}", param_count));
        param_count += 1;
    }
    if update_req.suspended.is_some() {
        updates.push(format!("suspended = ${}", param_count));
        param_count += 1;
    }
    if update_req.email_verified.is_some() {
        updates.push(format!("email_verified = ${}", param_count));
    }

    if updates.is_empty() {
        return Err(AuthError::Internal("No fields to update".to_string()));
    }

    let query = format!(
        "UPDATE users SET {}, updated_at = NOW() WHERE id = $1 RETURNING id, email, display_name, role, email_verified, suspended, created_at",
        updates.join(", ")
    );

    let mut query_builder = sqlx::query_as::<_, UserDetail>(&query).bind(user_id);

    if let Some(role) = update_req.role {
        query_builder = query_builder.bind(role);
    }
    if let Some(suspended) = update_req.suspended {
        query_builder = query_builder.bind(suspended);
    }
    if let Some(email_verified) = update_req.email_verified {
        query_builder = query_builder.bind(email_verified);
    }

    let updated_user = query_builder.fetch_one(db.as_ref()).await?;

    Ok(HttpResponse::Ok().json(updated_user))
}

#[delete("/api/v1/admin/users/{id}")]
pub async fn delete_user(
    req: HttpRequest,
    path: web::Path<Uuid>,
    db: web::Data<PgPool>,
) -> Result<impl Responder> {
    let admin_context = extract_user_context(&req)?;
    let user_id = path.into_inner();

    // Prevent admin from deleting themselves
    if admin_context.user_id == user_id.to_string() {
        return Err(AuthError::Internal(
            "Cannot delete your own account".to_string(),
        ));
    }

    log_audit_action(
        &db,
        &admin_context.user_id,
        "delete_user",
        Some(&user_id.to_string()),
        serde_json::json!({ "user_id": user_id }),
    )
    .await?;

    // Hard delete user (cascades via foreign keys)
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(db.as_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AuthError::Internal("User not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "User deleted successfully",
        "user_id": user_id
    })))
}

#[post("/api/v1/admin/users/{id}/impersonate")]
pub async fn impersonate_user(
    req: HttpRequest,
    path: web::Path<Uuid>,
    db: web::Data<PgPool>,
    jwt_manager: web::Data<Arc<JwtManager>>,
) -> Result<impl Responder> {
    let admin_context = extract_user_context(&req)?;
    let user_id = path.into_inner();

    // Prevent admin from impersonating themselves
    if admin_context.user_id == user_id.to_string() {
        return Err(AuthError::Internal(
            "Cannot impersonate your own account".to_string(),
        ));
    }

    log_audit_action(
        &db,
        &admin_context.user_id,
        "impersonate_user",
        Some(&user_id.to_string()),
        serde_json::json!({
            "admin_id": admin_context.user_id,
            "impersonated_user_id": user_id
        }),
    )
    .await?;

    // Get user details
    let user: (String, String) =
        sqlx::query_as("SELECT email, role FROM users WHERE id = $1 AND NOT suspended")
            .bind(user_id)
            .fetch_optional(db.as_ref())
            .await?
            .ok_or(AuthError::Internal(
                "User not found or suspended".to_string(),
            ))?;

    // Create short-lived impersonation token (15 minutes)
    let access_token = jwt_manager.create_access_token(
        user_id.to_string(),
        Some(user.0),
        vec![user.1],
        vec!["admin:impersonate".to_string()],
    )?;

    Ok(HttpResponse::Ok().json(ImpersonationTokenResponse {
        access_token,
        expires_in: 900, // 15 minutes
        original_admin_id: admin_context.user_id,
        impersonated_user_id: user_id.to_string(),
    }))
}

#[get("/api/v1/admin/audit-logs")]
pub async fn get_audit_logs(
    req: HttpRequest,
    query: web::Query<AuditLogsQuery>,
    db: web::Data<PgPool>,
) -> Result<impl Responder> {
    let admin_context = extract_user_context(&req)?;
    let mut query = query.into_inner();
    query.validate()?;

    // Create audit logger
    let audit_logger = PostgresAuditLogger::new(db.get_ref().clone());

    // Convert query to audit filter
    let filter = query.to_audit_filter()?;

    // Query audit logs
    let logs = audit_logger
        .query(filter)
        .await
        .map_err(|e| AuthError::Internal(format!("Failed to query audit logs: {}", e)))?;

    let total = logs.len();
    let total_pages = ((total as f64) / (query.per_page as f64)).ceil() as u32;

    // Log this audit query action
    log_audit_action(
        &db,
        &admin_context.user_id,
        "query_audit_logs",
        None,
        serde_json::json!({
            "filters": {
                "start_date": query.start_date,
                "end_date": query.end_date,
                "user_id": query.user_id,
                "action": query.action,
                "resource_type": query.resource_type,
            },
            "page": query.page,
            "per_page": query.per_page,
        }),
    )
    .await?;

    Ok(HttpResponse::Ok().json(AuditLogsResponse {
        logs,
        total,
        page: query.page,
        per_page: query.per_page,
        total_pages,
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn log_audit_action(
    db: &PgPool,
    admin_user_id: &str,
    action: &str,
    target_user_id: Option<&str>,
    details: serde_json::Value,
) -> Result<()> {
    let admin_uuid = Uuid::parse_str(admin_user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid admin UUID: {}", e)))?;

    let target_uuid = if let Some(target) = target_user_id {
        Some(
            Uuid::parse_str(target)
                .map_err(|e| AuthError::Internal(format!("Invalid target UUID: {}", e)))?,
        )
    } else {
        None
    };

    sqlx::query(
        "INSERT INTO audit_logs (admin_user_id, action, target_user_id, details, timestamp)
         VALUES ($1, $2, $3, $4, NOW())",
    )
    .bind(admin_uuid)
    .bind(action)
    .bind(target_uuid)
    .bind(details)
    .execute(db)
    .await?;

    Ok(())
}
