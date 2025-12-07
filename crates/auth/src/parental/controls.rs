use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::AuthError;
use crate::parental::parse_time;

/// Content rating hierarchy following MPAA standards
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContentRating {
    G = 0,
    PG = 1,
    #[serde(rename = "PG-13")]
    PG13 = 2,
    R = 3,
    #[serde(rename = "NC-17")]
    NC17 = 4,
}

impl ContentRating {
    pub fn from_str(s: &str) -> Result<Self, AuthError> {
        match s.to_uppercase().as_str() {
            "G" => Ok(ContentRating::G),
            "PG" => Ok(ContentRating::PG),
            "PG-13" | "PG13" => Ok(ContentRating::PG13),
            "R" => Ok(ContentRating::R),
            "NC-17" | "NC17" => Ok(ContentRating::NC17),
            _ => Err(AuthError::ValidationError(format!(
                "Invalid content rating: {}",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ContentRating::G => "G",
            ContentRating::PG => "PG",
            ContentRating::PG13 => "PG-13",
            ContentRating::R => "R",
            ContentRating::NC17 => "NC-17",
        }
    }
}

/// Parental controls configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentalControls {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pin_hash: Option<String>, // bcrypt hash of 4-digit PIN
    pub content_rating_limit: ContentRating,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewing_time_start: Option<NaiveTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewing_time_end: Option<NaiveTime>,
    pub blocked_genres: Vec<String>,
}

impl Default for ParentalControls {
    fn default() -> Self {
        Self {
            enabled: false,
            pin_hash: None,
            content_rating_limit: ContentRating::NC17, // No restrictions by default
            viewing_time_start: None,
            viewing_time_end: None,
            blocked_genres: Vec::new(),
        }
    }
}

impl ParentalControls {
    /// Validate PIN format (4 digits)
    pub fn validate_pin(pin: &str) -> Result<(), AuthError> {
        if pin.len() != 4 {
            return Err(AuthError::ValidationError(
                "PIN must be exactly 4 digits".to_string(),
            ));
        }
        if !pin.chars().all(|c| c.is_ascii_digit()) {
            return Err(AuthError::ValidationError(
                "PIN must contain only digits".to_string(),
            ));
        }
        Ok(())
    }

    /// Hash PIN using bcrypt
    pub fn hash_pin(pin: &str) -> Result<String, AuthError> {
        Self::validate_pin(pin)?;
        bcrypt::hash(pin, bcrypt::DEFAULT_COST)
            .map_err(|e| AuthError::InternalError(format!("Failed to hash PIN: {}", e)))
    }

    /// Verify PIN against hash
    pub fn verify_pin(&self, pin: &str) -> Result<bool, AuthError> {
        Self::validate_pin(pin)?;
        match &self.pin_hash {
            Some(hash) => bcrypt::verify(pin, hash)
                .map_err(|e| AuthError::InternalError(format!("Failed to verify PIN: {}", e))),
            None => Ok(false),
        }
    }

    /// Check if content is allowed based on rating
    pub fn is_content_allowed(&self, content_rating: ContentRating) -> bool {
        if !self.enabled {
            return true;
        }
        content_rating <= self.content_rating_limit
    }

    /// Check if genre is blocked
    pub fn is_genre_blocked(&self, genre: &str) -> bool {
        if !self.enabled {
            return false;
        }
        self.blocked_genres
            .iter()
            .any(|blocked| blocked.eq_ignore_ascii_case(genre))
    }
}

/// Request to set parental controls
#[derive(Debug, Deserialize)]
pub struct SetParentalControlsRequest {
    pub enabled: bool,
    pub pin: Option<String>, // 4 digits
    pub content_rating_limit: Option<String>,
    pub viewing_time_start: Option<String>, // "HH:MM"
    pub viewing_time_end: Option<String>,
    pub blocked_genres: Option<Vec<String>>,
}

/// Response after setting parental controls
#[derive(Debug, Serialize)]
pub struct SetParentalControlsResponse {
    pub success: bool,
    pub parental_controls: ParentalControlsPublic,
}

/// Public view of parental controls (without PIN hash)
#[derive(Debug, Serialize)]
pub struct ParentalControlsPublic {
    pub enabled: bool,
    pub has_pin: bool,
    pub content_rating_limit: String,
    pub viewing_time_start: Option<String>,
    pub viewing_time_end: Option<String>,
    pub blocked_genres: Vec<String>,
}

impl From<ParentalControls> for ParentalControlsPublic {
    fn from(controls: ParentalControls) -> Self {
        Self {
            enabled: controls.enabled,
            has_pin: controls.pin_hash.is_some(),
            content_rating_limit: controls.content_rating_limit.as_str().to_string(),
            viewing_time_start: controls
                .viewing_time_start
                .map(|t| t.format("%H:%M").to_string()),
            viewing_time_end: controls
                .viewing_time_end
                .map(|t| t.format("%H:%M").to_string()),
            blocked_genres: controls.blocked_genres,
        }
    }
}

/// Set parental controls for a user
pub async fn set_parental_controls(
    pool: &PgPool,
    user_id: Uuid,
    request: SetParentalControlsRequest,
) -> Result<ParentalControls, AuthError> {
    // Get existing controls or create default
    let existing = get_parental_controls(pool, user_id).await?;

    // Build updated controls
    let mut controls = existing.unwrap_or_default();
    controls.enabled = request.enabled;

    // Update PIN if provided
    if let Some(pin) = request.pin {
        controls.pin_hash = Some(ParentalControls::hash_pin(&pin)?);
    }

    // Update content rating limit
    if let Some(rating_str) = request.content_rating_limit {
        controls.content_rating_limit = ContentRating::from_str(&rating_str)?;
    }

    // Update viewing time window
    if let Some(start_str) = request.viewing_time_start {
        controls.viewing_time_start = Some(parse_time(&start_str)?);
    }
    if let Some(end_str) = request.viewing_time_end {
        controls.viewing_time_end = Some(parse_time(&end_str)?);
    }

    // Update blocked genres
    if let Some(genres) = request.blocked_genres {
        controls.blocked_genres = genres;
    }

    // Save to database
    let controls_json = serde_json::to_value(&controls)
        .map_err(|e| AuthError::InternalError(format!("Failed to serialize controls: {}", e)))?;

    sqlx::query(
        r#"
        UPDATE users
        SET parental_controls = $1
        WHERE id = $2
        "#,
    )
    .bind(controls_json)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to update parental controls: {}", e)))?;

    Ok(controls)
}

/// Get parental controls for a user
pub async fn get_parental_controls(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<ParentalControls>, AuthError> {
    let row = sqlx::query(
        r#"
        SELECT parental_controls
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(format!("Failed to fetch parental controls: {}", e)))?;

    match row {
        Some(r) => {
            let parental_controls_value: Option<serde_json::Value> =
                r.try_get("parental_controls").map_err(|e| {
                    AuthError::DatabaseError(format!(
                        "Failed to get parental_controls column: {}",
                        e
                    ))
                })?;

            match parental_controls_value {
                Some(json) => {
                    let controls: ParentalControls = serde_json::from_value(json).map_err(|e| {
                        AuthError::InternalError(format!("Failed to deserialize controls: {}", e))
                    })?;
                    Ok(Some(controls))
                }
                None => Ok(None),
            }
        }
        None => Err(AuthError::UserNotFound),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_rating_hierarchy() {
        assert!(ContentRating::G < ContentRating::PG);
        assert!(ContentRating::PG < ContentRating::PG13);
        assert!(ContentRating::PG13 < ContentRating::R);
        assert!(ContentRating::R < ContentRating::NC17);
    }

    #[test]
    fn test_content_rating_from_str() {
        assert_eq!(ContentRating::from_str("G").unwrap(), ContentRating::G);
        assert_eq!(ContentRating::from_str("PG").unwrap(), ContentRating::PG);
        assert_eq!(
            ContentRating::from_str("PG-13").unwrap(),
            ContentRating::PG13
        );
        assert_eq!(
            ContentRating::from_str("pg13").unwrap(),
            ContentRating::PG13
        );
        assert_eq!(ContentRating::from_str("R").unwrap(), ContentRating::R);
        assert_eq!(
            ContentRating::from_str("NC-17").unwrap(),
            ContentRating::NC17
        );
        assert!(ContentRating::from_str("invalid").is_err());
    }

    #[test]
    fn test_validate_pin_valid() {
        assert!(ParentalControls::validate_pin("1234").is_ok());
        assert!(ParentalControls::validate_pin("0000").is_ok());
        assert!(ParentalControls::validate_pin("9999").is_ok());
    }

    #[test]
    fn test_validate_pin_invalid_length() {
        assert!(ParentalControls::validate_pin("123").is_err());
        assert!(ParentalControls::validate_pin("12345").is_err());
        assert!(ParentalControls::validate_pin("").is_err());
    }

    #[test]
    fn test_validate_pin_invalid_chars() {
        assert!(ParentalControls::validate_pin("12a4").is_err());
        assert!(ParentalControls::validate_pin("12.4").is_err());
        assert!(ParentalControls::validate_pin("abcd").is_err());
    }

    #[test]
    fn test_hash_and_verify_pin() {
        let pin = "1234";
        let hash = ParentalControls::hash_pin(pin).unwrap();

        let controls = ParentalControls {
            pin_hash: Some(hash),
            ..Default::default()
        };

        assert!(controls.verify_pin("1234").unwrap());
        assert!(!controls.verify_pin("4321").unwrap());
    }

    #[test]
    fn test_is_content_allowed_disabled() {
        let controls = ParentalControls {
            enabled: false,
            content_rating_limit: ContentRating::G,
            ..Default::default()
        };

        assert!(controls.is_content_allowed(ContentRating::NC17));
    }

    #[test]
    fn test_is_content_allowed_enabled() {
        let controls = ParentalControls {
            enabled: true,
            content_rating_limit: ContentRating::PG13,
            ..Default::default()
        };

        assert!(controls.is_content_allowed(ContentRating::G));
        assert!(controls.is_content_allowed(ContentRating::PG));
        assert!(controls.is_content_allowed(ContentRating::PG13));
        assert!(!controls.is_content_allowed(ContentRating::R));
        assert!(!controls.is_content_allowed(ContentRating::NC17));
    }

    #[test]
    fn test_is_genre_blocked() {
        let controls = ParentalControls {
            enabled: true,
            blocked_genres: vec!["horror".to_string(), "thriller".to_string()],
            ..Default::default()
        };

        assert!(controls.is_genre_blocked("horror"));
        assert!(controls.is_genre_blocked("Horror"));
        assert!(controls.is_genre_blocked("THRILLER"));
        assert!(!controls.is_genre_blocked("comedy"));
    }

    #[test]
    fn test_is_genre_blocked_disabled() {
        let controls = ParentalControls {
            enabled: false,
            blocked_genres: vec!["horror".to_string()],
            ..Default::default()
        };

        assert!(!controls.is_genre_blocked("horror"));
    }

    #[test]
    fn test_parental_controls_public_conversion() {
        let controls = ParentalControls {
            enabled: true,
            pin_hash: Some("hashed_pin".to_string()),
            content_rating_limit: ContentRating::PG13,
            viewing_time_start: Some(NaiveTime::from_hms_opt(6, 0, 0).unwrap()),
            viewing_time_end: Some(NaiveTime::from_hms_opt(21, 0, 0).unwrap()),
            blocked_genres: vec!["horror".to_string()],
        };

        let public: ParentalControlsPublic = controls.into();

        assert!(public.enabled);
        assert!(public.has_pin);
        assert_eq!(public.content_rating_limit, "PG-13");
        assert_eq!(public.viewing_time_start, Some("06:00".to_string()));
        assert_eq!(public.viewing_time_end, Some("21:00".to_string()));
        assert_eq!(public.blocked_genres, vec!["horror".to_string()]);
    }
}
