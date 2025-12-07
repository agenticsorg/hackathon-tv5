//! Event streaming for user activity tracking
//!
//! This module provides event streaming capabilities for tracking user activity
//! events across the Media Gateway platform, including search, playback, authentication,
//! and content interactions.

pub mod user_activity;

pub use user_activity::{
    ActivityEventError, ActivityEventResult, ActivityEventType, KafkaActivityProducer,
    UserActivityEvent, UserActivityProducer,
};
