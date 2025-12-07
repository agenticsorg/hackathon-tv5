use crate::error::{AuthError, Result};
use crate::profile::types::{AuditLogEntry, UpdateProfileRequest, UserProfile};
use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct ProfileStorage {
    pool: PgPool,
}

impl ProfileStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_user_profile(&self, user_id: Uuid) -> Result<Option<UserProfile>> {
        let result = sqlx::query(
            r#"
            SELECT
                u.id,
                u.email,
                u.display_name,
                u.avatar_url,
                u.preferences,
                u.email_verified,
                u.created_at,
                COALESCE(
                    ARRAY_AGG(op.provider) FILTER (WHERE op.provider IS NOT NULL),
                    ARRAY[]::VARCHAR[]
                ) as oauth_providers
            FROM users u
            LEFT JOIN oauth_providers op ON u.id = op.user_id
            WHERE u.id = $1 AND u.deleted_at IS NULL
            GROUP BY u.id, u.email, u.display_name, u.avatar_url, u.preferences, u.email_verified, u.created_at
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Internal(format!("Database error: {}", e)))?;

        Ok(result.map(|row| UserProfile {
            id: row.get("id"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            avatar_url: row.get("avatar_url"),
            preferences: row.get("preferences"),
            email_verified: row.get("email_verified"),
            created_at: row.get::<chrono::NaiveDateTime, _>("created_at").and_utc(),
            oauth_providers: row.get("oauth_providers"),
        }))
    }

    pub async fn update_user_profile(
        &self,
        user_id: Uuid,
        request: &UpdateProfileRequest,
    ) -> Result<UserProfile> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AuthError::Internal(format!("Transaction error: {}", e)))?;

        // Get old values for audit log
        let old_profile = self.get_user_profile(user_id).await?;
        let old_values = old_profile.as_ref().map(|p| {
            serde_json::json!({
                "display_name": p.display_name,
                "avatar_url": p.avatar_url,
                "preferences": p.preferences
            })
        });

        // Build dynamic update query
        let mut query = String::from("UPDATE users SET updated_at = NOW()");
        let mut param_count = 1;

        if request.display_name.is_some() {
            param_count += 1;
            query.push_str(&format!(", display_name = ${}", param_count));
        }
        if request.avatar_url.is_some() {
            param_count += 1;
            query.push_str(&format!(", avatar_url = ${}", param_count));
        }
        if request.preferences.is_some() {
            param_count += 1;
            query.push_str(&format!(", preferences = ${}", param_count));
        }

        query.push_str(" WHERE id = $1 AND deleted_at IS NULL");

        let mut query_builder = sqlx::query(&query).bind(user_id);

        if let Some(ref name) = request.display_name {
            query_builder = query_builder.bind(name);
        }
        if let Some(ref url) = request.avatar_url {
            query_builder = query_builder.bind(url);
        }
        if let Some(ref prefs) = request.preferences {
            query_builder = query_builder.bind(prefs);
        }

        query_builder
            .execute(&mut *tx)
            .await
            .map_err(|e| AuthError::Internal(format!("Update error: {}", e)))?;

        // Get updated profile
        let updated_profile = self
            .get_user_profile(user_id)
            .await?
            .ok_or_else(|| AuthError::Internal("User not found after update".to_string()))?;

        // Create audit log entry
        let new_values = serde_json::json!({
            "display_name": updated_profile.display_name,
            "avatar_url": updated_profile.avatar_url,
            "preferences": updated_profile.preferences
        });

        sqlx::query(
            r#"
            INSERT INTO audit_log (user_id, action, resource_type, resource_id, old_values, new_values)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(user_id)
        .bind("profile.update")
        .bind("user")
        .bind(Some(user_id))
        .bind(old_values)
        .bind(Some(new_values))
        .execute(&mut *tx)
        .await
        .map_err(|e| AuthError::Internal(format!("Audit log error: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AuthError::Internal(format!("Commit error: {}", e)))?;

        Ok(updated_profile)
    }

    pub async fn soft_delete_user(&self, user_id: Uuid) -> Result<DateTime<Utc>> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AuthError::Internal(format!("Transaction error: {}", e)))?;

        let deleted_at = Utc::now();

        sqlx::query("UPDATE users SET deleted_at = $1 WHERE id = $2 AND deleted_at IS NULL")
            .bind(deleted_at.naive_utc())
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AuthError::Internal(format!("Delete error: {}", e)))?;

        // Create audit log entry
        sqlx::query(
            r#"
            INSERT INTO audit_log (user_id, action, resource_type, resource_id, new_values)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user_id)
        .bind("account.soft_delete")
        .bind("user")
        .bind(Some(user_id))
        .bind(Some(serde_json::json!({ "deleted_at": deleted_at })))
        .execute(&mut *tx)
        .await
        .map_err(|e| AuthError::Internal(format!("Audit log error: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AuthError::Internal(format!("Commit error: {}", e)))?;

        Ok(deleted_at)
    }

    pub async fn can_recover_account(&self, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            SELECT deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NOT NULL
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Internal(format!("Database error: {}", e)))?;

        if let Some(row) = result {
            if let Some(deleted_at) = row.get::<Option<chrono::NaiveDateTime>, _>("deleted_at") {
                let grace_period_end = deleted_at.and_utc() + Duration::days(30);
                return Ok(Utc::now() < grace_period_end);
            }
        }

        Ok(false)
    }

    pub async fn get_audit_logs(&self, user_id: Uuid, limit: i64) -> Result<Vec<AuditLogEntry>> {
        let logs = sqlx::query_as::<_, AuditLogEntry>(
            r#"
            SELECT
                id,
                user_id,
                action,
                resource_type,
                resource_id,
                old_values,
                new_values,
                ip_address,
                user_agent,
                created_at
            FROM audit_log
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::Internal(format!("Database error: {}", e)))?;

        Ok(logs)
    }

    pub async fn update_avatar_url(&self, user_id: Uuid, avatar_url: String) -> Result<()> {
        sqlx::query("UPDATE users SET avatar_url = $1, updated_at = NOW() WHERE id = $2 AND deleted_at IS NULL")
            .bind(avatar_url)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::Internal(format!("Update error: {}", e)))?;

        Ok(())
    }
}
