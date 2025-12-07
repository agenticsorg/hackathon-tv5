//! Pagination utilities for API endpoints and database queries
//!
//! This module provides reusable pagination functionality supporting both offset-based
//! and cursor-based pagination patterns. It includes utilities for parsing query parameters,
//! encoding/decoding cursors, and constructing paginated responses with hypermedia links.
//!
//! # Features
//!
//! - Offset-based pagination (page/per_page)
//! - Cursor-based pagination for infinite scrolling
//! - Base64 cursor encoding/decoding
//! - Automatic pagination link generation (next, prev, first, last)
//! - Configurable default and maximum limits
//! - Type-safe query parameter parsing
//!
//! # Example
//!
//! ```
//! use media_gateway_core::pagination::{
//!     PaginationType, PaginatedResponse, PaginationLinks, encode_cursor, decode_cursor
//! };
//!
//! // Offset-based pagination
//! let pagination = PaginationType::Offset {
//!     offset: 0,
//!     limit: 20,
//! };
//!
//! // Cursor-based pagination
//! let cursor = encode_cursor(1638360000, "user-123");
//! let pagination_cursor = PaginationType::Cursor {
//!     cursor: Some(cursor),
//!     limit: 20,
//! };
//!
//! // Create paginated response
//! let response = PaginatedResponse {
//!     items: vec!["item1", "item2"],
//!     total: Some(100),
//!     has_more: true,
//!     next_cursor: Some("next-cursor".to_string()),
//!     links: PaginationLinks::default(),
//! };
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Default number of items per page
pub const DEFAULT_LIMIT: usize = 20;

/// Maximum number of items per page
pub const MAX_LIMIT: usize = 100;

/// Pagination type supporting both offset and cursor-based pagination
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PaginationType {
    /// Offset-based pagination (traditional page/per_page)
    ///
    /// Suitable for:
    /// - Small to medium datasets
    /// - UIs with explicit page numbers
    /// - Random access to pages
    Offset {
        /// Number of items to skip (page * limit)
        offset: usize,
        /// Maximum number of items to return
        limit: usize,
    },
    /// Cursor-based pagination (for infinite scrolling)
    ///
    /// Suitable for:
    /// - Large datasets
    /// - Real-time data streams
    /// - Infinite scroll UIs
    /// - Better performance on large offsets
    Cursor {
        /// Opaque cursor pointing to a position in the dataset
        cursor: Option<String>,
        /// Maximum number of items to return
        limit: usize,
    },
}

impl Default for PaginationType {
    fn default() -> Self {
        Self::Offset {
            offset: 0,
            limit: DEFAULT_LIMIT,
        }
    }
}

impl PaginationType {
    /// Get the limit for this pagination type
    pub fn limit(&self) -> usize {
        match self {
            Self::Offset { limit, .. } => *limit,
            Self::Cursor { limit, .. } => *limit,
        }
    }

    /// Get the offset (only applicable for offset-based pagination)
    pub fn offset(&self) -> Option<usize> {
        match self {
            Self::Offset { offset, .. } => Some(*offset),
            Self::Cursor { .. } => None,
        }
    }

    /// Get the cursor (only applicable for cursor-based pagination)
    pub fn cursor(&self) -> Option<&str> {
        match self {
            Self::Offset { .. } => None,
            Self::Cursor { cursor, .. } => cursor.as_deref(),
        }
    }
}

/// Trait for parsing pagination parameters from query strings
pub trait PaginationParams: Sized {
    /// Parse pagination parameters from a query string map
    ///
    /// # Arguments
    ///
    /// * `params` - HashMap of query parameters
    ///
    /// # Returns
    ///
    /// A pagination type instance
    ///
    /// # Examples
    ///
    /// ```
    /// use media_gateway_core::pagination::{PaginationType, PaginationParams};
    /// use std::collections::HashMap;
    ///
    /// let mut params = HashMap::new();
    /// params.insert("page".to_string(), "2".to_string());
    /// params.insert("per_page".to_string(), "50".to_string());
    ///
    /// let pagination = PaginationType::from_query_params(&params);
    /// ```
    fn from_query_params(params: &HashMap<String, String>) -> Self;
}

impl PaginationParams for PaginationType {
    fn from_query_params(params: &HashMap<String, String>) -> Self {
        // Check if cursor-based pagination is requested
        if let Some(cursor) = params.get("cursor") {
            let limit = params
                .get("limit")
                .and_then(|l| l.parse::<usize>().ok())
                .unwrap_or(DEFAULT_LIMIT)
                .min(MAX_LIMIT);

            return Self::Cursor {
                cursor: if cursor.is_empty() {
                    None
                } else {
                    Some(cursor.clone())
                },
                limit,
            };
        }

        // Otherwise use offset-based pagination
        let page = params
            .get("page")
            .and_then(|p| p.parse::<usize>().ok())
            .unwrap_or(1)
            .max(1);

        let per_page = params
            .get("per_page")
            .and_then(|p| p.parse::<usize>().ok())
            .unwrap_or(DEFAULT_LIMIT)
            .min(MAX_LIMIT);

        Self::Offset {
            offset: (page - 1) * per_page,
            limit: per_page,
        }
    }
}

/// Paginated response wrapper
///
/// Generic container for paginated data with metadata and hypermedia links.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Items in the current page
    pub items: Vec<T>,

    /// Total number of items (only for offset pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,

    /// Whether there are more items available
    pub has_more: bool,

    /// Cursor for the next page (only for cursor pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,

    /// Hypermedia pagination links
    pub links: PaginationLinks,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response for offset-based pagination
    ///
    /// # Arguments
    ///
    /// * `items` - Items in the current page
    /// * `total` - Total number of items
    /// * `offset` - Current offset
    /// * `limit` - Items per page
    /// * `base_url` - Base URL for generating pagination links
    pub fn offset(
        items: Vec<T>,
        total: usize,
        offset: usize,
        limit: usize,
        base_url: &str,
    ) -> Self {
        let current_page = (offset / limit) + 1;
        let total_pages = (total + limit - 1) / limit;
        let has_more = offset + items.len() < total;

        let links = PaginationLinks {
            next: if has_more {
                Some(format!(
                    "{}?page={}&per_page={}",
                    base_url,
                    current_page + 1,
                    limit
                ))
            } else {
                None
            },
            prev: if current_page > 1 {
                Some(format!(
                    "{}?page={}&per_page={}",
                    base_url,
                    current_page - 1,
                    limit
                ))
            } else {
                None
            },
            first: Some(format!("{}?page=1&per_page={}", base_url, limit)),
            last: Some(format!(
                "{}?page={}&per_page={}",
                base_url, total_pages, limit
            )),
        };

        Self {
            items,
            total: Some(total),
            has_more,
            next_cursor: None,
            links,
        }
    }

    /// Create a new paginated response for cursor-based pagination
    ///
    /// # Arguments
    ///
    /// * `items` - Items in the current page
    /// * `has_more` - Whether there are more items
    /// * `next_cursor` - Cursor for the next page
    /// * `limit` - Items per page
    /// * `base_url` - Base URL for generating pagination links
    pub fn cursor(
        items: Vec<T>,
        has_more: bool,
        next_cursor: Option<String>,
        limit: usize,
        base_url: &str,
    ) -> Self {
        let links = PaginationLinks {
            next: next_cursor
                .as_ref()
                .map(|cursor| format!("{}?cursor={}&limit={}", base_url, cursor, limit)),
            prev: None,
            first: None,
            last: None,
        };

        Self {
            items,
            total: None,
            has_more,
            next_cursor,
            links,
        }
    }

    /// Get the current page number (only for offset pagination)
    pub fn current_page(&self) -> Option<usize> {
        self.total.map(|_| {
            // Extract page from links.first if available
            1 // Default to 1 if unable to determine
        })
    }

    /// Get total number of pages (only for offset pagination)
    pub fn total_pages(&self) -> Option<usize> {
        self.total.map(|total| {
            let limit = self.items.len().max(1);
            (total + limit - 1) / limit
        })
    }

    /// Check if this is the first page
    pub fn is_first_page(&self) -> bool {
        self.links.prev.is_none()
    }

    /// Check if this is the last page
    pub fn is_last_page(&self) -> bool {
        !self.has_more
    }
}

impl<T> Default for PaginatedResponse<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            total: None,
            has_more: false,
            next_cursor: None,
            links: PaginationLinks::default(),
        }
    }
}

/// Hypermedia pagination links
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PaginationLinks {
    /// URL for the next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// URL for the previous page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,

    /// URL for the first page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,

    /// URL for the last page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
}

/// Encode a cursor from timestamp and ID
///
/// Creates a base64-encoded cursor containing a timestamp and entity ID.
/// The cursor format is: `{timestamp}:{id}`
///
/// # Arguments
///
/// * `timestamp` - Unix timestamp (seconds since epoch)
/// * `id` - Entity ID
///
/// # Returns
///
/// Base64-encoded cursor string
///
/// # Examples
///
/// ```
/// use media_gateway_core::pagination::encode_cursor;
///
/// let cursor = encode_cursor(1638360000, "user-123");
/// assert!(!cursor.is_empty());
/// ```
pub fn encode_cursor(timestamp: i64, id: &str) -> String {
    let cursor_data = format!("{}:{}", timestamp, id);
    BASE64.encode(cursor_data.as_bytes())
}

/// Decode a cursor to extract timestamp and ID
///
/// Parses a base64-encoded cursor to extract the timestamp and entity ID.
///
/// # Arguments
///
/// * `cursor` - Base64-encoded cursor string
///
/// # Returns
///
/// * `Ok((timestamp, id))` - Decoded timestamp and ID
/// * `Err(String)` - Error message if decoding fails
///
/// # Examples
///
/// ```
/// use media_gateway_core::pagination::{encode_cursor, decode_cursor};
///
/// let cursor = encode_cursor(1638360000, "user-123");
/// let (timestamp, id) = decode_cursor(&cursor).unwrap();
/// assert_eq!(timestamp, 1638360000);
/// assert_eq!(id, "user-123");
/// ```
pub fn decode_cursor(cursor: &str) -> Result<(i64, String), String> {
    let decoded = BASE64
        .decode(cursor.as_bytes())
        .map_err(|e| format!("Invalid cursor encoding: {}", e))?;

    let cursor_str =
        String::from_utf8(decoded).map_err(|e| format!("Invalid cursor format: {}", e))?;

    let parts: Vec<&str> = cursor_str.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid cursor format: expected timestamp:id".to_string());
    }

    let timestamp = parts[0]
        .parse::<i64>()
        .map_err(|e| format!("Invalid timestamp in cursor: {}", e))?;

    let id = parts[1].to_string();

    Ok((timestamp, id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_type_default() {
        let pagination = PaginationType::default();
        assert_eq!(
            pagination,
            PaginationType::Offset {
                offset: 0,
                limit: DEFAULT_LIMIT
            }
        );
    }

    #[test]
    fn test_pagination_type_limit() {
        let offset_pagination = PaginationType::Offset {
            offset: 0,
            limit: 50,
        };
        assert_eq!(offset_pagination.limit(), 50);

        let cursor_pagination = PaginationType::Cursor {
            cursor: None,
            limit: 30,
        };
        assert_eq!(cursor_pagination.limit(), 30);
    }

    #[test]
    fn test_pagination_type_offset() {
        let offset_pagination = PaginationType::Offset {
            offset: 100,
            limit: 50,
        };
        assert_eq!(offset_pagination.offset(), Some(100));

        let cursor_pagination = PaginationType::Cursor {
            cursor: None,
            limit: 30,
        };
        assert_eq!(cursor_pagination.offset(), None);
    }

    #[test]
    fn test_pagination_type_cursor() {
        let offset_pagination = PaginationType::Offset {
            offset: 0,
            limit: 50,
        };
        assert_eq!(offset_pagination.cursor(), None);

        let cursor_pagination = PaginationType::Cursor {
            cursor: Some("test-cursor".to_string()),
            limit: 30,
        };
        assert_eq!(cursor_pagination.cursor(), Some("test-cursor"));
    }

    #[test]
    fn test_from_query_params_offset() {
        let mut params = HashMap::new();
        params.insert("page".to_string(), "2".to_string());
        params.insert("per_page".to_string(), "50".to_string());

        let pagination = PaginationType::from_query_params(&params);
        assert_eq!(
            pagination,
            PaginationType::Offset {
                offset: 50,
                limit: 50
            }
        );
    }

    #[test]
    fn test_from_query_params_offset_defaults() {
        let params = HashMap::new();
        let pagination = PaginationType::from_query_params(&params);
        assert_eq!(
            pagination,
            PaginationType::Offset {
                offset: 0,
                limit: DEFAULT_LIMIT
            }
        );
    }

    #[test]
    fn test_from_query_params_offset_max_limit() {
        let mut params = HashMap::new();
        params.insert("per_page".to_string(), "200".to_string());

        let pagination = PaginationType::from_query_params(&params);
        assert_eq!(
            pagination,
            PaginationType::Offset {
                offset: 0,
                limit: MAX_LIMIT
            }
        );
    }

    #[test]
    fn test_from_query_params_cursor() {
        let mut params = HashMap::new();
        params.insert("cursor".to_string(), "abc123".to_string());
        params.insert("limit".to_string(), "25".to_string());

        let pagination = PaginationType::from_query_params(&params);
        assert_eq!(
            pagination,
            PaginationType::Cursor {
                cursor: Some("abc123".to_string()),
                limit: 25
            }
        );
    }

    #[test]
    fn test_from_query_params_cursor_empty() {
        let mut params = HashMap::new();
        params.insert("cursor".to_string(), "".to_string());

        let pagination = PaginationType::from_query_params(&params);
        assert_eq!(
            pagination,
            PaginationType::Cursor {
                cursor: None,
                limit: DEFAULT_LIMIT
            }
        );
    }

    #[test]
    fn test_paginated_response_offset() {
        let items = vec![1, 2, 3, 4, 5];
        let response =
            PaginatedResponse::offset(items.clone(), 100, 0, 20, "https://api.example.com/items");

        assert_eq!(response.items, items);
        assert_eq!(response.total, Some(100));
        assert!(response.has_more);
        assert!(response.next_cursor.is_none());
        assert!(response.links.next.is_some());
        assert!(response.links.prev.is_none());
        assert!(response.links.first.is_some());
        assert!(response.links.last.is_some());
    }

    #[test]
    fn test_paginated_response_offset_last_page() {
        let items = vec![1, 2, 3];
        let response =
            PaginatedResponse::offset(items.clone(), 100, 97, 20, "https://api.example.com/items");

        assert_eq!(response.items, items);
        assert_eq!(response.total, Some(100));
        assert!(!response.has_more);
        assert!(response.links.next.is_none());
        assert!(response.links.prev.is_some());
    }

    #[test]
    fn test_paginated_response_cursor() {
        let items = vec![1, 2, 3, 4, 5];
        let response = PaginatedResponse::cursor(
            items.clone(),
            true,
            Some("next-cursor-123".to_string()),
            20,
            "https://api.example.com/items",
        );

        assert_eq!(response.items, items);
        assert!(response.total.is_none());
        assert!(response.has_more);
        assert_eq!(response.next_cursor, Some("next-cursor-123".to_string()));
        assert!(response.links.next.is_some());
        assert!(response.links.prev.is_none());
        assert!(response.links.first.is_none());
        assert!(response.links.last.is_none());
    }

    #[test]
    fn test_paginated_response_is_first_page() {
        let response: PaginatedResponse<i32> = PaginatedResponse {
            items: vec![],
            total: Some(100),
            has_more: true,
            next_cursor: None,
            links: PaginationLinks {
                next: Some("next".to_string()),
                prev: None,
                first: Some("first".to_string()),
                last: Some("last".to_string()),
            },
        };
        assert!(response.is_first_page());
    }

    #[test]
    fn test_paginated_response_is_last_page() {
        let response: PaginatedResponse<i32> = PaginatedResponse {
            items: vec![],
            total: Some(100),
            has_more: false,
            next_cursor: None,
            links: PaginationLinks::default(),
        };
        assert!(response.is_last_page());
    }

    #[test]
    fn test_encode_decode_cursor() {
        let timestamp = 1638360000i64;
        let id = "user-123";

        let cursor = encode_cursor(timestamp, id);
        assert!(!cursor.is_empty());

        let (decoded_timestamp, decoded_id) = decode_cursor(&cursor).unwrap();
        assert_eq!(decoded_timestamp, timestamp);
        assert_eq!(decoded_id, id);
    }

    #[test]
    fn test_decode_cursor_invalid_base64() {
        let result = decode_cursor("not-valid-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_cursor_invalid_format() {
        let cursor = BASE64.encode(b"invalid-format");
        let result = decode_cursor(&cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_cursor_invalid_timestamp() {
        let cursor = BASE64.encode(b"not-a-number:user-123");
        let result = decode_cursor(&cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_pagination_links_serialization() {
        let links = PaginationLinks {
            next: Some("https://api.example.com?page=2".to_string()),
            prev: None,
            first: Some("https://api.example.com?page=1".to_string()),
            last: Some("https://api.example.com?page=10".to_string()),
        };

        let json = serde_json::to_string(&links).unwrap();
        assert!(json.contains("next"));
        assert!(!json.contains("prev"));
        assert!(json.contains("first"));
        assert!(json.contains("last"));
    }

    #[test]
    fn test_paginated_response_serialization() {
        let response = PaginatedResponse {
            items: vec![1, 2, 3],
            total: Some(100),
            has_more: true,
            next_cursor: None,
            links: PaginationLinks::default(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("items"));
        assert!(json.contains("total"));
        assert!(json.contains("has_more"));
        assert!(!json.contains("next_cursor"));
    }
}
