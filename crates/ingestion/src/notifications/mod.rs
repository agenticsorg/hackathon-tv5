//! Content Expiration Notifications Module
//!
//! This module handles notifications for content that is about to expire,
//! tracking notification status to prevent duplicate notifications.

pub mod expiration;

pub use expiration::{
    ContentExpiringEvent, ExpirationNotificationConfig, ExpirationNotificationJob,
    NotificationStatus, NotificationWindow,
};
