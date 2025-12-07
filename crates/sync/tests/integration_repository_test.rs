//! Integration tests for sync repository persistence
//!
//! Tests verify data survives service restart by using real PostgreSQL database

use anyhow::Result;
use chrono::Utc;
use media_gateway_sync::{
    crdt::{HLCTimestamp, ORSet, ORSetEntry, PlaybackPosition, PlaybackState},
    device::{
        AudioCodec, DeviceCapabilities, DeviceInfo, DevicePlatform, DeviceType, HDRFormat,
        VideoResolution,
    },
    repository::{PostgresSyncRepository, SyncRepository},
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

async fn setup_test_db() -> Result<PostgresSyncRepository> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("../../migrations").run(&pool).await?;

    Ok(PostgresSyncRepository::new(pool))
}

fn create_test_device(device_id: &str) -> DeviceInfo {
    DeviceInfo {
        device_id: device_id.to_string(),
        device_type: DeviceType::TV,
        platform: DevicePlatform::Tizen,
        capabilities: DeviceCapabilities {
            max_resolution: VideoResolution::UHD_4K,
            hdr_support: vec![HDRFormat::HDR10, HDRFormat::DolbyVision],
            audio_codecs: vec![AudioCodec::AAC, AudioCodec::DolbyAtmos],
            remote_controllable: true,
            can_cast: false,
            screen_size: Some(65.0),
        },
        app_version: "1.0.0".to_string(),
        last_seen: Utc::now(),
        is_online: true,
        device_name: Some("Test TV".to_string()),
    }
}

#[tokio::test]
async fn test_watchlist_persistence() -> Result<()> {
    let repo = setup_test_db().await?;
    let user_id = Uuid::new_v4().to_string();
    let device_id = "device-test-001";

    // Create OR-Set with watchlist items
    let mut or_set = ORSet::new();
    let timestamp1 = HLCTimestamp::from_components(1000, 0);
    let timestamp2 = HLCTimestamp::from_components(2000, 0);

    let tag1 = or_set.add("content-1".to_string(), timestamp1, device_id.to_string());
    let tag2 = or_set.add("content-2".to_string(), timestamp2, device_id.to_string());

    // Save watchlist
    repo.save_watchlist(&user_id, &or_set).await?;

    // Load watchlist (simulating service restart)
    let loaded_set = repo.load_watchlist(&user_id).await?;

    // Verify items persisted
    assert!(loaded_set.contains("content-1"));
    assert!(loaded_set.contains("content-2"));
    assert_eq!(loaded_set.len(), 2);

    // Test removal
    or_set.remove_by_tag(&tag1);
    repo.save_watchlist(&user_id, &or_set).await?;

    let loaded_set = repo.load_watchlist(&user_id).await?;
    assert!(!loaded_set.contains("content-1"));
    assert!(loaded_set.contains("content-2"));
    assert_eq!(loaded_set.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_watchlist_incremental_updates() -> Result<()> {
    let repo = setup_test_db().await?;
    let user_id = Uuid::new_v4().to_string();
    let device_id = "device-test-002";

    // Add item incrementally
    let entry = ORSetEntry {
        content_id: "content-movie-123".to_string(),
        unique_tag: Uuid::new_v4().to_string(),
        timestamp: HLCTimestamp::from_components(1000, 0),
        device_id: device_id.to_string(),
    };

    repo.add_watchlist_item(&user_id, "content-movie-123", &entry)
        .await?;

    // Load and verify
    let loaded_set = repo.load_watchlist(&user_id).await?;
    assert!(loaded_set.contains("content-movie-123"));

    // Remove item
    repo.remove_watchlist_item(&user_id, &entry.unique_tag)
        .await?;

    let loaded_set = repo.load_watchlist(&user_id).await?;
    assert!(!loaded_set.contains("content-movie-123"));

    Ok(())
}

#[tokio::test]
async fn test_progress_persistence() -> Result<()> {
    let repo = setup_test_db().await?;
    let user_id = Uuid::new_v4().to_string();
    let device_id = "device-test-003";

    // Create progress entry
    let position = PlaybackPosition::new(
        "content-series-456".to_string(),
        1800,
        3600,
        PlaybackState::Paused,
        HLCTimestamp::from_components(5000, 0),
        device_id.to_string(),
    );

    // Save progress
    repo.save_progress(&user_id, &position).await?;

    // Load all progress (simulating service restart)
    let loaded_progress = repo.load_progress(&user_id).await?;
    assert_eq!(loaded_progress.len(), 1);

    let loaded_pos = &loaded_progress[0];
    assert_eq!(loaded_pos.content_id, "content-series-456");
    assert_eq!(loaded_pos.position_seconds, 1800);
    assert_eq!(loaded_pos.duration_seconds, 3600);
    assert_eq!(loaded_pos.state, PlaybackState::Paused);

    Ok(())
}

#[tokio::test]
async fn test_progress_lww_conflict_resolution() -> Result<()> {
    let repo = setup_test_db().await?;
    let user_id = Uuid::new_v4().to_string();

    // Save older progress
    let old_position = PlaybackPosition::new(
        "content-123".to_string(),
        100,
        1000,
        PlaybackState::Playing,
        HLCTimestamp::from_components(1000, 0),
        "device-old".to_string(),
    );
    repo.save_progress(&user_id, &old_position).await?;

    // Save newer progress (should win)
    let new_position = PlaybackPosition::new(
        "content-123".to_string(),
        500,
        1000,
        PlaybackState::Paused,
        HLCTimestamp::from_components(2000, 0),
        "device-new".to_string(),
    );
    repo.save_progress(&user_id, &new_position).await?;

    // Verify newer progress persisted
    let loaded = repo.get_progress(&user_id, "content-123").await?;
    assert!(loaded.is_some());
    let pos = loaded.unwrap();
    assert_eq!(pos.position_seconds, 500);
    assert_eq!(pos.device_id, "device-new");

    Ok(())
}

#[tokio::test]
async fn test_progress_delete() -> Result<()> {
    let repo = setup_test_db().await?;
    let user_id = Uuid::new_v4().to_string();

    let position = PlaybackPosition::new(
        "content-to-delete".to_string(),
        100,
        1000,
        PlaybackState::Stopped,
        HLCTimestamp::from_components(1000, 0),
        "device-test".to_string(),
    );

    repo.save_progress(&user_id, &position).await?;

    // Verify exists
    let loaded = repo.get_progress(&user_id, "content-to-delete").await?;
    assert!(loaded.is_some());

    // Delete
    repo.delete_progress(&user_id, "content-to-delete").await?;

    // Verify deleted
    let loaded = repo.get_progress(&user_id, "content-to-delete").await?;
    assert!(loaded.is_none());

    Ok(())
}

#[tokio::test]
async fn test_device_persistence() -> Result<()> {
    let repo = setup_test_db().await?;
    let user_id = Uuid::new_v4().to_string();
    let device = create_test_device("device-test-004");

    // Save device
    repo.save_device(&user_id, &device).await?;

    // Load all devices (simulating service restart)
    let loaded_devices = repo.load_devices(&user_id).await?;
    assert_eq!(loaded_devices.len(), 1);

    let loaded_device = &loaded_devices[0];
    assert_eq!(loaded_device.device_id, "device-test-004");
    assert_eq!(loaded_device.device_type, DeviceType::TV);
    assert_eq!(
        loaded_device.capabilities.max_resolution,
        VideoResolution::UHD_4K
    );
    assert_eq!(loaded_device.is_online, true);

    Ok(())
}

#[tokio::test]
async fn test_device_heartbeat() -> Result<()> {
    let repo = setup_test_db().await?;
    let user_id = Uuid::new_v4().to_string();
    let mut device = create_test_device("device-test-005");
    device.is_online = false;

    repo.save_device(&user_id, &device).await?;

    // Update heartbeat
    repo.update_device_heartbeat(&user_id, "device-test-005")
        .await?;

    // Verify online status updated
    let loaded = repo.get_device(&user_id, "device-test-005").await?;
    assert!(loaded.is_some());
    assert!(loaded.unwrap().is_online);

    Ok(())
}

#[tokio::test]
async fn test_device_delete() -> Result<()> {
    let repo = setup_test_db().await?;
    let user_id = Uuid::new_v4().to_string();
    let device = create_test_device("device-to-delete");

    repo.save_device(&user_id, &device).await?;

    // Verify exists
    let loaded = repo.get_device(&user_id, "device-to-delete").await?;
    assert!(loaded.is_some());

    // Delete
    repo.delete_device(&user_id, "device-to-delete").await?;

    // Verify deleted
    let loaded = repo.get_device(&user_id, "device-to-delete").await?;
    assert!(loaded.is_none());

    Ok(())
}

#[tokio::test]
async fn test_service_restart_scenario() -> Result<()> {
    let repo = setup_test_db().await?;
    let user_id = Uuid::new_v4().to_string();
    let device_id = "device-restart-test";

    // Simulate service running: save state
    let mut or_set = ORSet::new();
    or_set.add(
        "movie-1".to_string(),
        HLCTimestamp::from_components(1000, 0),
        device_id.to_string(),
    );
    or_set.add(
        "series-2".to_string(),
        HLCTimestamp::from_components(2000, 0),
        device_id.to_string(),
    );

    let progress = PlaybackPosition::new(
        "movie-1".to_string(),
        600,
        7200,
        PlaybackState::Paused,
        HLCTimestamp::from_components(3000, 0),
        device_id.to_string(),
    );

    let device = create_test_device(device_id);

    repo.save_watchlist(&user_id, &or_set).await?;
    repo.save_progress(&user_id, &progress).await?;
    repo.save_device(&user_id, &device).await?;

    // SIMULATE SERVICE RESTART: drop references, create new instances

    // Service starts up, loads state from DB
    let loaded_watchlist = repo.load_watchlist(&user_id).await?;
    let loaded_progress = repo.load_progress(&user_id).await?;
    let loaded_devices = repo.load_devices(&user_id).await?;

    // Verify all state restored
    assert_eq!(loaded_watchlist.len(), 2);
    assert!(loaded_watchlist.contains("movie-1"));
    assert!(loaded_watchlist.contains("series-2"));

    assert_eq!(loaded_progress.len(), 1);
    assert_eq!(loaded_progress[0].content_id, "movie-1");
    assert_eq!(loaded_progress[0].position_seconds, 600);

    assert_eq!(loaded_devices.len(), 1);
    assert_eq!(loaded_devices[0].device_id, device_id);

    Ok(())
}

#[tokio::test]
async fn test_multiple_users_isolation() -> Result<()> {
    let repo = setup_test_db().await?;
    let user1_id = Uuid::new_v4().to_string();
    let user2_id = Uuid::new_v4().to_string();

    // User 1: add watchlist items
    let mut or_set1 = ORSet::new();
    or_set1.add(
        "user1-content".to_string(),
        HLCTimestamp::from_components(1000, 0),
        "device1".to_string(),
    );
    repo.save_watchlist(&user1_id, &or_set1).await?;

    // User 2: add different watchlist items
    let mut or_set2 = ORSet::new();
    or_set2.add(
        "user2-content".to_string(),
        HLCTimestamp::from_components(1000, 0),
        "device2".to_string(),
    );
    repo.save_watchlist(&user2_id, &or_set2).await?;

    // Verify isolation
    let user1_watchlist = repo.load_watchlist(&user1_id).await?;
    let user2_watchlist = repo.load_watchlist(&user2_id).await?;

    assert!(user1_watchlist.contains("user1-content"));
    assert!(!user1_watchlist.contains("user2-content"));

    assert!(user2_watchlist.contains("user2-content"));
    assert!(!user2_watchlist.contains("user1-content"));

    Ok(())
}
