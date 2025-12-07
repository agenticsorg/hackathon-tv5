use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Movie,
    Series,
    Episode,
    Short,
    Documentary,
    Special,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageSet {
    pub poster_small: Option<String>,
    pub poster_medium: Option<String>,
    pub poster_large: Option<String>,
    pub backdrop: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateContentRequest {
    pub title: String,
    pub content_type: ContentType,
    pub platform: String,
    pub platform_content_id: String,
    pub overview: Option<String>,
    pub release_year: Option<i32>,
    pub runtime_minutes: Option<i32>,
    pub genres: Vec<String>,
    pub rating: Option<String>,
    pub images: ImageSet,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContentRequest {
    pub title: Option<String>,
    pub overview: Option<String>,
    pub genres: Option<Vec<String>>,
    pub rating: Option<String>,
    pub images: Option<ImageSet>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AvailabilityUpdate {
    pub regions: Vec<String>,
    pub subscription_required: bool,
    pub purchase_price: Option<f64>,
    pub rental_price: Option<f64>,
    pub available_from: Option<DateTime<Utc>>,
    pub available_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentResponse {
    pub id: Uuid,
    pub title: String,
    pub content_type: ContentType,
    pub platform: String,
    pub platform_content_id: String,
    pub overview: Option<String>,
    pub release_year: Option<i32>,
    pub runtime_minutes: Option<i32>,
    pub genres: Vec<String>,
    pub rating: Option<String>,
    pub images: ImageSet,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl CreateContentRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title is required".to_string());
        }
        if self.platform.trim().is_empty() {
            return Err("Platform is required".to_string());
        }
        if self.platform_content_id.trim().is_empty() {
            return Err("Platform content ID is required".to_string());
        }
        if let Some(year) = self.release_year {
            if year < 1800 || year > 2100 {
                return Err("Release year must be between 1800 and 2100".to_string());
            }
        }
        if let Some(runtime) = self.runtime_minutes {
            if runtime <= 0 {
                return Err("Runtime must be positive".to_string());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_content_validation_success() {
        let request = CreateContentRequest {
            title: "Test Movie".to_string(),
            content_type: ContentType::Movie,
            platform: "netflix".to_string(),
            platform_content_id: "nf123".to_string(),
            overview: Some("A test movie".to_string()),
            release_year: Some(2024),
            runtime_minutes: Some(120),
            genres: vec!["action".to_string()],
            rating: Some("PG-13".to_string()),
            images: ImageSet::default(),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_content_validation_empty_title() {
        let request = CreateContentRequest {
            title: "   ".to_string(),
            content_type: ContentType::Movie,
            platform: "netflix".to_string(),
            platform_content_id: "nf123".to_string(),
            overview: None,
            release_year: None,
            runtime_minutes: None,
            genres: vec![],
            rating: None,
            images: ImageSet::default(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_content_validation_empty_platform() {
        let request = CreateContentRequest {
            title: "Test".to_string(),
            content_type: ContentType::Movie,
            platform: "".to_string(),
            platform_content_id: "nf123".to_string(),
            overview: None,
            release_year: None,
            runtime_minutes: None,
            genres: vec![],
            rating: None,
            images: ImageSet::default(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_content_validation_invalid_year() {
        let request = CreateContentRequest {
            title: "Test".to_string(),
            content_type: ContentType::Movie,
            platform: "netflix".to_string(),
            platform_content_id: "nf123".to_string(),
            overview: None,
            release_year: Some(3000),
            runtime_minutes: None,
            genres: vec![],
            rating: None,
            images: ImageSet::default(),
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_content_type_serialization() {
        let ct = ContentType::Movie;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, r#""movie""#);

        let ct = ContentType::Series;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, r#""series""#);
    }
}
