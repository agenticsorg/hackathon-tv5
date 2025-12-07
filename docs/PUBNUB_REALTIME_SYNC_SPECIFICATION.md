# PubNub and Real-Time Synchronization Specification
## Media Gateway Cross-Device Synchronization Architecture

**Version**: 1.0.0
**Last Updated**: 2025-12-06
**Status**: Specification Phase
**Research Sources**:
- https://github.com/agenticsorg/hackathon-tv5
- https://github.com/globalbusinessadvisors/media-gateway-research

---

## Executive Summary

This specification defines PubNub integration and real-time synchronization behavior for the Media Gateway TV discovery system. PubNub serves as the primary real-time messaging infrastructure enabling seamless cross-device state synchronization, device presence management, and content update propagation with sub-100ms latency targets.

**Key Integration Points**:
- **Cross-Device Sync**: CRDT-based state synchronization for watchlists, watch progress, and preferences
- **Device Coordination**: Multi-device presence detection, capability negotiation, and remote control
- **Content Updates**: Real-time catalog changes, availability updates, and personalized notifications
- **Event Streaming**: Platform-agnostic event distribution with regional and global broadcast channels

**Technology Stack**:
- **PubNub**: Real-time publish/subscribe messaging
- **CRDT**: Conflict-free Replicated Data Types (LWW-Register, OR-Set)
- **HLC**: Hybrid Logical Clocks for distributed timestamp ordering
- **Rust Client**: Custom PubNub Rust wrapper (`mg-pubnub-client` repository)

---

## Table of Contents

1. [PubNub Integration Role](#1-pubnub-integration-role)
2. [Channel Architecture](#2-channel-architecture)
3. [Message Types and Payloads](#3-message-types-and-payloads)
4. [Real-Time Event Categories](#4-real-time-event-categories)
5. [Synchronization Patterns](#5-synchronization-patterns)
6. [Device Interactions](#6-device-interactions)
7. [CRDT-Based Conflict Resolution](#7-crdt-based-conflict-resolution)
8. [Presence Management](#8-presence-management)
9. [History and Persistence](#9-history-and-persistence)
10. [Performance Requirements](#10-performance-requirements)
11. [Security and Access Control](#11-security-and-access-control)
12. [Implementation Specifications](#12-implementation-specifications)

---

## 1. PubNub Integration Role

### 1.1 Primary Responsibilities

PubNub serves as the **real-time messaging backbone** for the Media Gateway platform, enabling:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      PUBNUB INTEGRATION ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │  LAYER 1: DEVICE GATEWAY (WebSocket Hub + PubNub Client)             │ │
│  │                                                                       │ │
│  │  mg-device-gateway/                                                   │ │
│  │  ├── WebSocket Server (client connections)                           │ │
│  │  ├── PubNub Publisher (publish device events)                        │ │
│  │  └── PubNub Subscriber (receive broadcast events)                    │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                  │                                          │
│                                  ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │  LAYER 1: SYNC ENGINE (CRDT-Based State Synchronization)             │ │
│  │                                                                       │ │
│  │  mg-sync-engine/                                                      │ │
│  │  ├── CRDT Merge Logic (LWW-Register, OR-Set)                         │ │
│  │  ├── HLC Timestamp Generation (Hybrid Logical Clocks)                │ │
│  │  ├── Conflict Resolution (automatic merge)                           │ │
│  │  └── State Persistence (PostgreSQL + Memorystore cache)              │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                  │                                          │
│                                  ▼                                          │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │  PUBNUB DATA LAYER                                                    │ │
│  │                                                                       │ │
│  │  mg-pubnub-client/                                                    │ │
│  │  ├── Rust PubNub SDK Wrapper                                         │ │
│  │  ├── Channel Management (subscribe/unsubscribe)                      │ │
│  │  ├── Presence Tracking (online/offline/heartbeat)                    │ │
│  │  ├── Message History (fetch last N messages)                         │ │
│  │  └── Access Control (PAM integration)                                │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 PubNub vs Google Pub/Sub

| Feature | Google Pub/Sub | PubNub |
|---------|---------------|--------|
| **Primary Use Case** | Server-to-server async events | Client-to-client real-time sync |
| **Latency** | 100-500ms | 25-100ms |
| **Message Ordering** | No guarantee (requires ordering key) | Guaranteed per channel |
| **Presence Detection** | Not supported | Built-in (here_now, presence events) |
| **History** | Replay via subscriptions | Last 100 messages per channel (configurable) |
| **Access Control** | IAM-based | PAM (PubNub Access Manager) per channel |
| **Client Support** | Limited (HTTP, gRPC) | 70+ SDKs (Web, Mobile, Embedded) |
| **Use in Media Gateway** | Backend event streaming | Device synchronization |

**Architecture Decision**: Use **both** services:
- **PubNub**: Cross-device user state synchronization (watchlists, progress, presence)
- **Google Pub/Sub**: Backend service event streaming (content updates, catalog changes)

---

## 2. Channel Architecture

### 2.1 Channel Topology

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PUBNUB CHANNEL TOPOLOGY                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  USER-SPECIFIC CHANNELS (per authenticated user):                          │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ user.{userId}.sync                                                     │ │
│  │   Purpose: Cross-device state synchronization                         │ │
│  │   Publishers: All user devices                                        │ │
│  │   Subscribers: All user devices                                       │ │
│  │   Message Types:                                                      │ │
│  │     - watch_progress_update (CRDT: LWW-Register)                     │ │
│  │     - watchlist_update (CRDT: OR-Set)                                │ │
│  │     - preference_update (CRDT: LWW-Register)                         │ │
│  │   Retention: Last 100 messages (24 hours)                            │ │
│  │   Access: User-specific PAM token                                    │ │
│  ├───────────────────────────────────────────────────────────────────────┤ │
│  │ user.{userId}.devices                                                  │ │
│  │   Purpose: Device presence and capability discovery                   │ │
│  │   Publishers: All user devices (heartbeat)                           │ │
│  │   Subscribers: All user devices + backend (monitoring)               │ │
│  │   Message Types:                                                      │ │
│  │     - device_join (device capabilities, platform, version)           │ │
│  │     - device_leave (graceful disconnect)                             │ │
│  │     - device_heartbeat (liveness check every 30s)                    │ │
│  │     - device_command (remote control: pause, play, cast)             │ │
│  │   Retention: Last 25 messages (1 hour)                               │ │
│  │   Presence: Enabled (occupancy tracking)                             │ │
│  ├───────────────────────────────────────────────────────────────────────┤ │
│  │ user.{userId}.notifications                                            │ │
│  │   Purpose: Personalized push notifications                            │ │
│  │   Publishers: Backend services only                                   │ │
│  │   Subscribers: All user devices                                       │ │
│  │   Message Types:                                                      │ │
│  │     - content_expiring (content leaving platform in 30/7/1 days)     │ │
│  │     - new_recommendation (AI-generated personalized picks)           │ │
│  │     - social_update (friend activity, watch parties)                 │ │
│  │     - platform_announcement (new features, maintenance)              │ │
│  │   Retention: Last 50 messages (7 days)                               │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  GLOBAL BROADCAST CHANNELS:                                                 │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ global.trending                                                        │ │
│  │   Purpose: Platform-wide trending content                             │ │
│  │   Publishers: Recommendation engine (hourly batch)                    │ │
│  │   Subscribers: All connected clients                                  │ │
│  │   Message Types:                                                      │ │
│  │     - trending_update (top 100 content, updated hourly)              │ │
│  │   Retention: Last 24 messages (24 hours)                             │ │
│  │   Access: Public (read-only for clients)                             │ │
│  ├───────────────────────────────────────────────────────────────────────┤ │
│  │ global.announcements                                                   │ │
│  │   Purpose: System-wide announcements                                  │ │
│  │   Publishers: Admin console only                                      │ │
│  │   Subscribers: All connected clients                                  │ │
│  │   Message Types:                                                      │ │
│  │     - system_maintenance (scheduled downtime)                         │ │
│  │     - feature_launch (new features)                                   │ │
│  │     - emergency_alert (critical issues)                               │ │
│  │   Retention: Last 100 messages (30 days)                             │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  REGIONAL CHANNELS:                                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ region.{regionCode}.updates                                            │ │
│  │   Purpose: Regional content availability changes                       │ │
│  │   Publishers: Ingestion services (catalog sync)                       │ │
│  │   Subscribers: Clients in matching region                             │ │
│  │   Message Types:                                                      │ │
│  │     - content_available (new content in region)                       │ │
│  │     - content_leaving (content expiring in region)                    │ │
│  │     - pricing_update (subscription price changes)                     │ │
│  │   Retention: Last 500 messages (7 days)                               │ │
│  │   Regions: us-east, us-west, ca, uk, eu, au, jp, etc.                │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  PLATFORM-SPECIFIC CHANNELS:                                                │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ platform.{platformId}.catalog                                          │ │
│  │   Purpose: Per-platform catalog updates                               │ │
│  │   Publishers: MCP connectors (ingestion)                              │ │
│  │   Subscribers: Metadata fabric, availability index                    │ │
│  │   Message Types:                                                      │ │
│  │     - catalog_addition (new content added)                            │ │
│  │     - catalog_removal (content removed)                               │ │
│  │     - metadata_update (title, description, cast changes)              │ │
│  │   Retention: Last 1000 messages (14 days)                             │ │
│  │   Platforms: netflix, prime, disney, hulu, apple, youtube, etc.      │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Channel Groups

PubNub **Channel Groups** enable efficient multi-channel subscriptions:

```rust
/// Channel group for user's complete subscription set
pub fn user_channel_group(user_id: &str, region: &str) -> String {
    format!("user-{}-all", user_id)
}

/// Channels included in user group
pub fn user_group_channels(user_id: &str, region: &str) -> Vec<String> {
    vec![
        format!("user.{}.sync", user_id),
        format!("user.{}.devices", user_id),
        format!("user.{}.notifications", user_id),
        format!("region.{}.updates", region),
        "global.trending".to_string(),
        "global.announcements".to_string(),
    ]
}
```

**Benefits**:
- **Single Subscribe Call**: Subscribe to channel group instead of individual channels
- **Dynamic Updates**: Add/remove channels without client reconnection
- **Bandwidth Efficiency**: Reduced connection overhead

---

## 3. Message Types and Payloads

### 3.1 Watch Progress Update (LWW-Register)

**Channel**: `user.{userId}.sync`
**CRDT**: Last-Writer-Wins Register (LWW-Register)
**Conflict Resolution**: Latest timestamp wins

```rust
/// Watch progress message using LWW-Register CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchProgressMessage {
    /// Message type identifier
    pub msg_type: String,  // "watch_progress_update"

    /// Content being watched
    pub content_id: String,

    /// Playback position in seconds
    pub progress_seconds: u32,

    /// Total duration in seconds
    pub duration_seconds: u32,

    /// Completion percentage (0.0-1.0)
    pub completion_percent: f32,

    /// HLC timestamp for conflict resolution (microseconds)
    pub timestamp: i64,

    /// Device that generated the update
    pub device_id: String,

    /// Device type (for UX display)
    pub device_type: DeviceType,  // TV, Phone, Web, Tablet

    /// Playback state
    pub state: PlaybackState,  // Playing, Paused, Stopped
}

impl WatchProgressMessage {
    /// Merge with another progress update (LWW conflict resolution)
    pub fn merge(&mut self, other: &WatchProgressMessage) {
        if other.content_id != self.content_id {
            return; // Different content, no merge
        }

        // Last-Writer-Wins: keep message with latest timestamp
        if other.timestamp > self.timestamp {
            *self = other.clone();
        }
    }
}
```

**Example JSON Payload**:
```json
{
  "msg_type": "watch_progress_update",
  "content_id": "tmdb:550",
  "progress_seconds": 3245,
  "duration_seconds": 8217,
  "completion_percent": 0.395,
  "timestamp": 1733500800000000,
  "device_id": "tv-samsung-living-room",
  "device_type": "TV",
  "state": "Paused"
}
```

### 3.2 Watchlist Update (OR-Set)

**Channel**: `user.{userId}.sync`
**CRDT**: Observed-Remove Set (OR-Set)
**Conflict Resolution**: Add-wins bias (additions preserved, explicit removes only)

```rust
/// Watchlist update message using OR-Set CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistMessage {
    /// Message type identifier
    pub msg_type: String,  // "watchlist_update"

    /// Operation type
    pub operation: WatchlistOperation,  // Add, Remove

    /// Content being added/removed
    pub content_id: String,

    /// HLC timestamp (microseconds)
    pub timestamp: i64,

    /// Device that made the change
    pub device_id: String,

    /// Unique tag for OR-Set semantics (UUID)
    pub unique_tag: String,

    /// Optional: Content metadata for immediate UX update
    pub content_metadata: Option<ContentSnippet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchlistOperation {
    Add,
    Remove,
}

/// Lightweight content metadata for client-side display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSnippet {
    pub title: String,
    pub year: Option<u16>,
    pub poster_url: Option<String>,
    pub content_type: ContentType,  // Movie, TVShow
}
```

**Example JSON Payload (Add)**:
```json
{
  "msg_type": "watchlist_update",
  "operation": "Add",
  "content_id": "tmdb:550",
  "timestamp": 1733500800000000,
  "device_id": "phone-ios-alice",
  "unique_tag": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "content_metadata": {
    "title": "Fight Club",
    "year": 1999,
    "poster_url": "https://image.tmdb.org/t/p/w500/pB8BM7pdSp6B6Ih7QZ4DrQ3PmJK.jpg",
    "content_type": "Movie"
  }
}
```

### 3.3 Device Heartbeat

**Channel**: `user.{userId}.devices`
**Purpose**: Liveness check and capability advertisement
**Frequency**: Every 30 seconds

```rust
/// Device heartbeat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceHeartbeatMessage {
    /// Message type identifier
    pub msg_type: String,  // "device_heartbeat"

    /// Device identifier
    pub device_id: String,

    /// Device type
    pub device_type: DeviceType,

    /// Device capabilities
    pub capabilities: DeviceCapabilities,

    /// Current app version
    pub app_version: String,

    /// Device platform
    pub platform: DevicePlatform,  // Tizen, webOS, iOS, Android, Web

    /// Last activity timestamp (HLC microseconds)
    pub last_active: i64,

    /// Battery level (0.0-1.0, None for non-mobile)
    pub battery_level: Option<f32>,

    /// Network quality (1-5, 5 = excellent)
    pub network_quality: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Max supported video resolution
    pub max_resolution: VideoResolution,  // SD, HD, FHD, UHD_4K, UHD_8K

    /// Supported audio codecs
    pub audio_codecs: Vec<AudioCodec>,  // AAC, Dolby_Atmos, DTS_X

    /// HDR support
    pub hdr_support: Vec<HDRFormat>,  // HDR10, Dolby_Vision, HLG

    /// Can receive remote commands
    pub remote_controllable: bool,

    /// Can cast content to other devices
    pub can_cast: bool,

    /// Screen diagonal in inches (None for audio-only)
    pub screen_size: Option<f32>,
}
```

**Example JSON Payload**:
```json
{
  "msg_type": "device_heartbeat",
  "device_id": "tv-samsung-living-room",
  "device_type": "TV",
  "capabilities": {
    "max_resolution": "UHD_4K",
    "audio_codecs": ["AAC", "Dolby_Atmos"],
    "hdr_support": ["HDR10", "Dolby_Vision"],
    "remote_controllable": true,
    "can_cast": false,
    "screen_size": 65.0
  },
  "app_version": "1.2.3",
  "platform": "Tizen",
  "last_active": 1733500800000000,
  "battery_level": null,
  "network_quality": 5
}
```

### 3.4 Remote Control Command

**Channel**: `user.{userId}.devices`
**Purpose**: Cross-device control (e.g., phone controls TV)

```rust
/// Remote control command message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommandMessage {
    /// Message type identifier
    pub msg_type: String,  // "remote_command"

    /// Target device to receive command
    pub target_device_id: String,

    /// Source device sending command
    pub source_device_id: String,

    /// Command to execute
    pub command: RemoteCommand,

    /// Timestamp (HLC microseconds)
    pub timestamp: i64,

    /// Command expiration (commands older than 5s ignored)
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum RemoteCommand {
    /// Playback controls
    Play,
    Pause,
    Stop,
    Seek { position_seconds: u32 },

    /// Volume controls
    VolumeSet { level: f32 },  // 0.0-1.0
    VolumeMute,
    VolumeUnmute,

    /// Content controls
    LoadContent { content_id: String, start_position: Option<u32> },
    SwitchProfile { profile_id: String },

    /// Casting
    CastTo { target_device_id: String, content_id: String },
}
```

**Example JSON Payload**:
```json
{
  "msg_type": "remote_command",
  "target_device_id": "tv-samsung-living-room",
  "source_device_id": "phone-ios-alice",
  "command": {
    "type": "LoadContent",
    "params": {
      "content_id": "tmdb:550",
      "start_position": 3245
    }
  },
  "timestamp": 1733500800000000,
  "expires_at": 1733500805000000
}
```

---

## 4. Real-Time Event Categories

### 4.1 Stream Status Updates

**Use Case**: Notify users of streaming service status changes

**Channel**: `platform.{platformId}.status`
**Publishers**: Platform health monitors
**Subscribers**: All clients watching content on that platform

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStatusMessage {
    pub msg_type: String,  // "stream_status"
    pub platform_id: String,
    pub status: PlatformStatus,
    pub timestamp: i64,
    pub affected_regions: Vec<String>,
    pub estimated_resolution: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformStatus {
    Operational,
    Degraded { reason: String },
    Outage { reason: String },
    Maintenance { end_time: DateTime<Utc> },
}
```

### 4.2 Viewer Analytics Sync

**Use Case**: Aggregate real-time viewing statistics for trending content

**Channel**: `analytics.viewing`
**Publishers**: Device gateways (anonymized events)
**Subscribers**: Recommendation engine, trending aggregator

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewingAnalyticsMessage {
    pub msg_type: String,  // "viewing_analytics"
    pub content_id: String,
    pub event_type: ViewingEvent,
    pub anonymized_user_segment: String,  // "sci-fi-enthusiast", "comedy-lover"
    pub region: String,
    pub platform: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewingEvent {
    Started,
    Completed,
    Abandoned { percent_watched: f32 },
}
```

### 4.3 Chat/Interaction Forwarding

**Use Case**: Watch parties and social viewing

**Channel**: `party.{partyId}`
**Publishers**: All party members
**Subscribers**: All party members

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchPartyChatMessage {
    pub msg_type: String,  // "party_chat"
    pub party_id: String,
    pub user_id: String,
    pub user_display_name: String,
    pub message: ChatContent,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ChatContent {
    Text { text: String },
    Emoji { emoji: String },
    Reaction { reaction_type: String, target_timestamp: i64 },
    PlaybackSync { position_seconds: u32 },  // Sync playback positions
}
```

### 4.4 Alert and Notification Propagation

**Use Case**: Time-sensitive user alerts

**Channel**: `user.{userId}.notifications`
**Publishers**: Backend services
**Subscribers**: User devices

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotificationMessage {
    pub msg_type: String,  // "user_notification"
    pub notification_id: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub title: String,
    pub body: String,
    pub action_url: Option<String>,
    pub timestamp: i64,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    ContentExpiring { content_id: String, days_remaining: u8 },
    NewRecommendation { content_ids: Vec<String> },
    SocialUpdate { friend_id: String, activity: String },
    SystemAlert { severity: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,      // Background processing
    Medium,   // Standard notification
    High,     // Full-screen alert
    Critical, // Persistent until acknowledged
}
```

---

## 5. Synchronization Patterns

### 5.1 Multi-Device State Sync

**Scenario**: User adds content to watchlist on phone → state syncs to TV and web

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                   MULTI-DEVICE WATCHLIST SYNC FLOW                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Phone (CLI/App)              PubNub                    Smart TV            │
│       │                          │                        │                 │
│       │ 1. User adds "Fight Club"│                        │                 │
│       │    to watchlist           │                        │                 │
│       │                          │                        │                 │
│       │ 2. Publish to            │                        │                 │
│       │    user.alice.sync       │                        │                 │
│       │ ────────────────────────>│                        │                 │
│       │                          │                        │                 │
│       │                          │ 3. Broadcast to        │                 │
│       │                          │    all subscribers     │                 │
│       │                          │ ──────────────────────>│                 │
│       │                          │                        │                 │
│       │                          │                        │ 4. TV receives  │
│       │                          │                        │    OR-Set delta │
│       │                          │                        │                 │
│       │                          │                        │ 5. Merge CRDT   │
│       │                          │                        │    (add-wins)   │
│       │                          │                        │                 │
│       │                          │                        │ 6. Update UI    │
│       │                          │                        │    (show banner)│
│       │                          │                        │                 │
│       │                          │ <──────────────────────│                 │
│       │ <────────────────────────│ 7. Acknowledgment      │                 │
│       │                          │    (message received)  │                 │
│       │                          │                        │                 │
│                                                                             │
│  GUARANTEES:                                                                │
│  - Delivery: At-least-once (PubNub guarantees)                            │
│  - Ordering: FIFO per channel (messages arrive in publish order)          │
│  - Latency: <100ms (p99)                                                  │
│  - Conflict Resolution: CRDT automatic merge (no user intervention)       │
│  - Idempotency: Duplicate messages safely ignored (unique_tag dedup)     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Implementation Requirements**:
1. **Immediate Local Update**: Phone updates local state immediately (optimistic UI)
2. **Background Publish**: Phone publishes OR-Set delta to PubNub asynchronously
3. **Broadcast to All Devices**: PubNub distributes message to all user.alice.sync subscribers
4. **CRDT Merge**: Each device merges received delta with local state
5. **UI Reconciliation**: Devices update UI to reflect merged state
6. **Persistence**: Sync engine persists final state to PostgreSQL

### 5.2 Cross-Platform Coordination

**Scenario**: User starts watching on TV → switches to phone seamlessly

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     CROSS-PLATFORM HANDOFF FLOW                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Smart TV                    PubNub                    Phone (iOS)          │
│     │                          │                          │                 │
│     │ 1. User watching         │                          │                 │
│     │    at 34:25              │                          │                 │
│     │                          │                          │                 │
│     │ 2. Publish progress      │                          │                 │
│     │    every 10s             │                          │                 │
│     │ ────────────────────────>│                          │                 │
│     │                          │                          │                 │
│     │                          │ 3. Phone subscribes to   │                 │
│     │                          │    user.alice.sync       │                 │
│     │                          │ <────────────────────────│                 │
│     │                          │                          │                 │
│     │                          │ 4. Phone receives latest │                 │
│     │                          │    progress (34:25)      │                 │
│     │                          │ ────────────────────────>│                 │
│     │                          │                          │                 │
│     │                          │                          │ 5. User taps    │
│     │                          │                          │    "Resume"     │
│     │                          │                          │                 │
│     │                          │ 6. Phone requests        │                 │
│     │                          │    stream at 34:25       │                 │
│     │                          │ <────────────────────────│                 │
│     │                          │                          │                 │
│     │ 7. TV publishes          │                          │                 │
│     │    "Paused" state        │                          │                 │
│     │ ────────────────────────>│ ────────────────────────>│                 │
│     │                          │                          │                 │
│     │                          │                          │ 8. Phone playing│
│     │                          │                          │    at 34:25     │
│     │                          │                          │                 │
│                                                                             │
│  KEY BEHAVIORS:                                                             │
│  - Progress updates published every 10 seconds (LWW-Register)              │
│  - All sync messages use CRDT format (LWW/OR-Set)                          │
│  - Devices detect state changes via PubNub messages                        │
│  - Playback state transitions: Playing → Paused → Playing (new device)    │
│  - No explicit "handoff" API - implicit via state synchronization          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.3 Latency Requirements

| Sync Type | Target Latency (p50) | Target Latency (p99) | Measurement Method |
|-----------|---------------------|----------------------|-------------------|
| Watch Progress | 50ms | 100ms | Client publish → PubNub → Client receive |
| Watchlist Update | 75ms | 150ms | CRDT merge + UI update |
| Device Presence | 100ms | 250ms | Heartbeat → Presence event |
| Remote Command | 25ms | 75ms | Command publish → Device execute |
| Notification | 200ms | 500ms | Backend publish → Client display |
| Content Catalog Update | 1s | 5s | Non-critical, background sync |

**Optimization Strategies**:
1. **Edge Network**: PubNub edge PoPs reduce RTT (14 global locations)
2. **Connection Pooling**: Persistent WebSocket connections (no connection overhead)
3. **Message Batching**: Batch progress updates (max 10s interval) to reduce traffic
4. **Selective Subscription**: Subscribe only to active channels (unsubscribe from idle)
5. **Client-Side Caching**: Cache last known state locally, apply deltas incrementally

### 5.4 Conflict Resolution Strategies

**Strategy 1: Last-Writer-Wins (LWW) for Watch Progress**

```rust
/// Conflict resolution for concurrent watch progress updates
impl WatchProgressMessage {
    pub fn resolve_conflict(&mut self, other: &WatchProgressMessage) {
        // Use Hybrid Logical Clock (HLC) for total ordering
        if other.timestamp > self.timestamp {
            // Other message is newer, replace
            *self = other.clone();
        } else if other.timestamp == self.timestamp {
            // Tie-breaker: device_id lexicographic order
            if other.device_id > self.device_id {
                *self = other.clone();
            }
        }
        // else: self is newer, keep current value
    }
}
```

**Strategy 2: Add-Wins Bias for Watchlist (OR-Set)**

```rust
/// Conflict resolution for watchlist additions/removals
impl WatchlistState {
    pub fn merge(&mut self, delta: &WatchlistMessage) {
        match delta.operation {
            WatchlistOperation::Add => {
                // Add to additions set (idempotent)
                self.additions.push(WatchlistEntry {
                    content_id: delta.content_id.clone(),
                    timestamp: delta.timestamp,
                    device_id: delta.device_id.clone(),
                    unique_tag: delta.unique_tag.clone(),
                });
            }
            WatchlistOperation::Remove => {
                // Add to removals set (idempotent)
                self.removals.push(WatchlistEntry {
                    content_id: delta.content_id.clone(),
                    timestamp: delta.timestamp,
                    device_id: delta.device_id.clone(),
                    unique_tag: delta.unique_tag.clone(),
                });
            }
        }

        // Compute effective watchlist: additions - removals
        self.recompute_effective_items();
    }

    fn recompute_effective_items(&mut self) {
        self.effective_items = self.additions.iter()
            .filter(|add| {
                // Item is in watchlist if:
                // 1. It was added
                // 2. AND (NOT removed OR removal timestamp < addition timestamp)
                !self.removals.iter().any(|rem| {
                    rem.content_id == add.content_id &&
                    rem.timestamp >= add.timestamp
                })
            })
            .cloned()
            .collect();
    }
}
```

---

## 6. Device Interactions

### 6.1 Device Discovery and Registration

**Flow**: New device joins user account

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DEVICE REGISTRATION FLOW                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  New Device (TV)           Device Gateway         PubNub         Backend    │
│       │                          │                   │              │        │
│       │ 1. App launch            │                   │              │        │
│       │    (authenticated)       │                   │              │        │
│       │                          │                   │              │        │
│       │ 2. Register device       │                   │              │        │
│       │ ────────────────────────>│                   │              │        │
│       │                          │                   │              │        │
│       │                          │ 3. Create device  │              │        │
│       │                          │    record         │              │        │
│       │                          │ ─────────────────────────────────>│        │
│       │                          │                   │              │        │
│       │                          │ 4. Subscribe to   │              │        │
│       │                          │    user channels  │              │        │
│       │                          │ ─────────────────>│              │        │
│       │                          │                   │              │        │
│       │                          │ 5. Publish        │              │        │
│       │                          │    device_join    │              │        │
│       │                          │ ─────────────────>│              │        │
│       │                          │                   │              │        │
│       │                          │                   │ 6. Broadcast │        │
│       │                          │                   │    to all    │        │
│       │ <───────────────────────────────────────────────devices     │        │
│       │                          │                   │              │        │
│       │ 7. Receive list of       │                   │              │        │
│       │    other devices         │                   │              │        │
│       │                          │                   │              │        │
│       │ 8. Start heartbeat       │                   │              │        │
│       │    (every 30s)           │                   │              │        │
│       │ ───────────────────────────────────────────> │              │        │
│       │                          │                   │              │        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Device Registration Payload**:

```json
{
  "msg_type": "device_join",
  "device_id": "tv-samsung-living-room",
  "device_type": "TV",
  "platform": "Tizen",
  "app_version": "1.2.3",
  "capabilities": {
    "max_resolution": "UHD_4K",
    "audio_codecs": ["AAC", "Dolby_Atmos"],
    "hdr_support": ["HDR10", "Dolby_Vision"],
    "remote_controllable": true,
    "can_cast": false,
    "screen_size": 65.0
  },
  "timestamp": 1733500800000000,
  "initial_sync_required": true
}
```

### 6.2 Heartbeat and Health Monitoring

**Heartbeat Schedule**:
- **Interval**: 30 seconds
- **Timeout**: 60 seconds (2 missed heartbeats = offline)
- **Retry**: Exponential backoff on failure (1s, 2s, 4s, 8s, max 30s)

**PubNub Presence Configuration**:

```rust
/// PubNub presence configuration
pub struct PresenceConfig {
    /// Heartbeat interval (seconds)
    pub heartbeat_interval: u32,  // 30

    /// Timeout for presence (seconds)
    pub presence_timeout: u32,  // 60

    /// Enable presence for channel
    pub enabled: bool,  // true

    /// Presence announce on join/leave
    pub announce_max: u32,  // 25 (announce first 25 devices)
}
```

**Presence Event Handling**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresenceEvent {
    Join {
        device_id: String,
        occupancy: u32,
        timestamp: i64,
    },
    Leave {
        device_id: String,
        occupancy: u32,
        timestamp: i64,
    },
    Timeout {
        device_id: String,
        occupancy: u32,
        timestamp: i64,
    },
    StateChange {
        device_id: String,
        state: serde_json::Value,
        timestamp: i64,
    },
}
```

### 6.3 Remote Control Commands

**Supported Commands**:

| Command Category | Commands | Latency Target | Use Case |
|-----------------|----------|----------------|----------|
| Playback Control | Play, Pause, Stop, Seek | <50ms | Phone controls TV playback |
| Volume Control | VolumeSet, Mute, Unmute | <50ms | Phone adjusts TV volume |
| Content Loading | LoadContent, SwitchProfile | <200ms | Cast content from phone to TV |
| Screen Casting | CastTo, StopCast | <500ms | Airplay-like functionality |

**Command Validation**:

```rust
impl RemoteCommandMessage {
    /// Validate command before execution
    pub fn validate(&self, target_device: &DeviceState) -> Result<(), CommandError> {
        // 1. Check command not expired
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros() as i64;
        if self.expires_at < now {
            return Err(CommandError::Expired);
        }

        // 2. Check target device is online
        if !target_device.is_online {
            return Err(CommandError::DeviceOffline);
        }

        // 3. Check target device supports remote control
        if !target_device.capabilities.remote_controllable {
            return Err(CommandError::NotSupported);
        }

        // 4. Command-specific validation
        match &self.command {
            RemoteCommand::Seek { position_seconds } => {
                if target_device.current_content.is_none() {
                    return Err(CommandError::NoActiveContent);
                }
            }
            RemoteCommand::CastTo { target_device_id, .. } => {
                if !target_device.capabilities.can_cast {
                    return Err(CommandError::NotSupported);
                }
            }
            _ => {}
        }

        Ok(())
    }
}
```

### 6.4 State Reconciliation

**Problem**: Devices may have inconsistent state due to offline periods or message loss

**Solution**: Periodic state reconciliation

```rust
/// State reconciliation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateReconciliationRequest {
    pub msg_type: String,  // "state_reconciliation_request"
    pub device_id: String,
    pub last_known_state_timestamp: i64,
    pub requested_states: Vec<StateType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateType {
    WatchProgress,
    Watchlist,
    Preferences,
    Devices,
}

/// State reconciliation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateReconciliationResponse {
    pub msg_type: String,  // "state_reconciliation_response"
    pub watch_progress: Vec<WatchProgressMessage>,
    pub watchlist: Vec<WatchlistMessage>,
    pub preferences: Option<UserPreferences>,
    pub devices: Vec<DeviceHeartbeatMessage>,
    pub server_timestamp: i64,
}
```

**Reconciliation Flow**:

1. **Trigger**: Device comes online after being offline >5 minutes
2. **Request**: Device publishes reconciliation request to `user.{userId}.sync`
3. **Backend Response**: Sync engine fetches latest state from PostgreSQL
4. **Publish**: Backend publishes reconciliation response
5. **Merge**: Device merges received state with local CRDTs
6. **Resume**: Device resumes normal operation

---

## 7. CRDT-Based Conflict Resolution

### 7.1 Last-Writer-Wins Register (LWW-Register)

**Use Cases**: Watch progress, user preferences, profile settings

**Properties**:
- **Convergence**: All replicas eventually converge to same value
- **Conflict Resolution**: Timestamp-based (HLC for total ordering)
- **Idempotent**: Applying same update multiple times has same effect

**Implementation**:

```rust
/// LWW-Register for watch progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWWRegister<T> {
    pub value: T,
    pub timestamp: i64,  // HLC timestamp (microseconds)
    pub device_id: String,  // Tie-breaker
}

impl<T: Clone> LWWRegister<T> {
    pub fn new(value: T, timestamp: i64, device_id: String) -> Self {
        Self { value, timestamp, device_id }
    }

    /// Merge with another LWW-Register
    pub fn merge(&mut self, other: &LWWRegister<T>) {
        if other.timestamp > self.timestamp {
            // Other is newer, adopt its value
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.device_id = other.device_id.clone();
        } else if other.timestamp == self.timestamp {
            // Timestamps equal, use device_id as tie-breaker
            if other.device_id > self.device_id {
                self.value = other.value.clone();
                self.device_id = other.device_id.clone();
            }
        }
        // else: self is newer, no-op
    }
}
```

**Example: Watch Progress Conflict**:

```
Device A (TV):   progress = 100s @ t=1000
Device B (Phone): progress = 150s @ t=1001

After merge:
Both devices converge to: progress = 150s @ t=1001 (phone's update is newer)
```

### 7.2 Observed-Remove Set (OR-Set)

**Use Cases**: Watchlist, playlists, favorites

**Properties**:
- **Add-Wins Bias**: Concurrent add/remove → add wins
- **Unique Tags**: Each add operation gets unique UUID
- **Removal**: Remove by unique tag (not just content_id)

**Implementation**:

```rust
/// OR-Set for watchlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSet {
    /// Set of additions (each with unique tag)
    pub additions: HashSet<ORSetEntry>,

    /// Set of removals (by unique tag)
    pub removals: HashSet<String>,  // Set of unique_tags
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ORSetEntry {
    pub content_id: String,
    pub unique_tag: String,  // UUID
    pub timestamp: i64,
    pub device_id: String,
}

impl ORSet {
    /// Add content to set
    pub fn add(&mut self, content_id: String, device_id: String) -> String {
        let unique_tag = Uuid::new_v4().to_string();
        let timestamp = Self::hlc_now();

        self.additions.insert(ORSetEntry {
            content_id,
            unique_tag: unique_tag.clone(),
            timestamp,
            device_id,
        });

        unique_tag
    }

    /// Remove content from set (by unique_tag)
    pub fn remove(&mut self, content_id: &str) {
        // Find all entries for this content_id and mark their tags as removed
        let tags_to_remove: Vec<String> = self.additions.iter()
            .filter(|e| e.content_id == content_id)
            .map(|e| e.unique_tag.clone())
            .collect();

        for tag in tags_to_remove {
            self.removals.insert(tag);
        }
    }

    /// Merge with another OR-Set
    pub fn merge(&mut self, other: &ORSet) {
        // Union of additions
        self.additions.extend(other.additions.iter().cloned());

        // Union of removals
        self.removals.extend(other.removals.iter().cloned());
    }

    /// Compute effective set (additions - removals)
    pub fn effective_items(&self) -> HashSet<String> {
        self.additions.iter()
            .filter(|entry| !self.removals.contains(&entry.unique_tag))
            .map(|entry| entry.content_id.clone())
            .collect()
    }
}
```

**Example: Watchlist Conflict**:

```
Device A (TV):    Add "Fight Club" (tag: abc123) @ t=1000
Device B (Phone): Remove "Fight Club" (removes tag abc123) @ t=1001
Device C (Web):   Add "Fight Club" (tag: def456) @ t=1002

After merge:
All devices converge to: "Fight Club" is IN watchlist
  - Tag abc123 is removed
  - Tag def456 is present
  - Effective result: {"Fight Club"} (add-wins bias)
```

### 7.3 Hybrid Logical Clock (HLC)

**Purpose**: Provide total ordering of events in distributed system without clock synchronization

**Properties**:
- **Monotonic**: Timestamps always increase
- **Causality**: Preserves happens-before relationship
- **Physical Approximation**: Close to physical wall-clock time

**Implementation**:

```rust
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Hybrid Logical Clock for distributed timestamp ordering
pub struct HybridLogicalClock {
    /// Logical counter
    logical: AtomicI64,

    /// Last physical timestamp (microseconds)
    last_physical: AtomicI64,
}

impl HybridLogicalClock {
    pub fn new() -> Self {
        Self {
            logical: AtomicI64::new(0),
            last_physical: AtomicI64::new(0),
        }
    }

    /// Generate new HLC timestamp
    pub fn now(&self) -> i64 {
        // Get current physical time (microseconds since UNIX epoch)
        let physical = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        let last_physical = self.last_physical.load(Ordering::SeqCst);
        let logical = self.logical.load(Ordering::SeqCst);

        let (new_physical, new_logical) = if physical > last_physical {
            // Physical clock advanced, reset logical
            (physical, 0)
        } else {
            // Physical clock same or behind, increment logical
            (last_physical, logical + 1)
        };

        // Update stored values
        self.last_physical.store(new_physical, Ordering::SeqCst);
        self.logical.store(new_logical, Ordering::SeqCst);

        // Encode as single i64: high 48 bits = physical, low 16 bits = logical
        (new_physical << 16) | (new_logical & 0xFFFF)
    }

    /// Update clock based on received timestamp
    pub fn update(&self, received_timestamp: i64) {
        let received_physical = received_timestamp >> 16;
        let received_logical = received_timestamp & 0xFFFF;

        let physical = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        let last_physical = self.last_physical.load(Ordering::SeqCst);
        let logical = self.logical.load(Ordering::SeqCst);

        let (new_physical, new_logical) = if physical > last_physical && physical > received_physical {
            // Local physical clock is newest
            (physical, 0)
        } else if received_physical > last_physical {
            // Received timestamp is newer
            (received_physical, received_logical + 1)
        } else {
            // Same physical time, increment logical
            (last_physical, std::cmp::max(logical, received_logical) + 1)
        };

        self.last_physical.store(new_physical, Ordering::SeqCst);
        self.logical.store(new_logical, Ordering::SeqCst);
    }
}
```

---

## 8. Presence Management

### 8.1 Device Online/Offline Detection

**PubNub Presence Features**:

| Feature | Configuration | Behavior |
|---------|---------------|----------|
| Heartbeat Interval | 30 seconds | Client sends heartbeat every 30s |
| Presence Timeout | 60 seconds | Device marked offline after 60s without heartbeat |
| Announce Join/Leave | First 25 devices | Presence events for first 25 devices |
| Occupancy | Enabled | Channel returns total subscriber count |
| State | Enabled | Devices can attach metadata to presence |

**Presence State Metadata**:

```rust
/// Device presence state (attached to PubNub presence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePresenceState {
    /// Device type
    pub device_type: DeviceType,

    /// Current activity
    pub activity: DeviceActivity,

    /// Last update timestamp
    pub last_update: i64,

    /// App version
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceActivity {
    Idle,
    Browsing,
    Watching { content_id: String },
    Paused { content_id: String },
}
```

**Setting Presence State**:

```rust
use pubnub::{Pubnub, PubNubClientBuilder};

async fn set_device_presence_state(
    pubnub: &Pubnub,
    user_id: &str,
    device_state: &DevicePresenceState,
) -> Result<(), PubNubError> {
    let channel = format!("user.{}.devices", user_id);

    pubnub.set_presence_state()
        .channel(channel)
        .state(serde_json::to_value(device_state)?)
        .execute()
        .await?;

    Ok(())
}
```

### 8.2 Graceful Disconnect Handling

**Scenario 1: App Closed (Graceful)**

```
1. User closes app
2. App publishes device_leave message
3. App calls pubnub.unsubscribe_all()
4. PubNub fires leave presence event
5. Other devices receive leave event
6. Device marked offline immediately
```

**Scenario 2: Network Loss (Ungraceful)**

```
1. Device loses network connection
2. PubNub misses 2 consecutive heartbeats (60s)
3. PubNub fires timeout presence event
4. Other devices receive timeout event (delayed by 60s)
5. Device marked offline after timeout
```

**Implementation**:

```rust
/// Handle app lifecycle events
impl DeviceGatewayClient {
    /// Called when app enters background
    pub async fn on_app_background(&self) -> Result<(), Error> {
        // 1. Publish device state update (activity = Idle)
        self.publish_presence_update(DeviceActivity::Idle).await?;

        // 2. Reduce heartbeat frequency (every 5 minutes instead of 30s)
        self.set_heartbeat_interval(300).await?;

        Ok(())
    }

    /// Called when app enters foreground
    pub async fn on_app_foreground(&self) -> Result<(), Error> {
        // 1. Restore heartbeat frequency (every 30s)
        self.set_heartbeat_interval(30).await?;

        // 2. Trigger state reconciliation (fetch latest state)
        self.request_state_reconciliation().await?;

        Ok(())
    }

    /// Called when app is about to terminate
    pub async fn on_app_terminate(&self) -> Result<(), Error> {
        // 1. Publish device_leave message
        self.publish_device_leave().await?;

        // 2. Unsubscribe from all channels
        self.pubnub.unsubscribe_all().await?;

        // 3. Flush pending messages
        self.pubnub.flush().await?;

        Ok(())
    }
}
```

### 8.3 Multi-Device Presence UI

**Use Case**: Show user which devices are online and what they're doing

**UI Display**:

```
┌─────────────────────────────────────────────┐
│ Your Devices                                │
├─────────────────────────────────────────────┤
│                                             │
│ 📺 Samsung TV (Living Room)                 │
│    Watching: Fight Club (34:25 / 2:19:00)  │
│    Online • Just now                        │
│                                             │
│ 📱 iPhone 14 Pro                            │
│    Browsing                                 │
│    Online • 2 min ago                       │
│                                             │
│ 💻 MacBook Pro                              │
│    Idle                                     │
│    Offline • 1 hour ago                     │
│                                             │
│ [+ Add Device]                              │
│                                             │
└─────────────────────────────────────────────┘
```

**Data Source**: PubNub presence + device heartbeats

---

## 9. History and Persistence

### 9.1 Message Retention Configuration

**PubNub History Settings** (per channel):

| Channel Pattern | Retention Count | Retention Duration | Use Case |
|----------------|-----------------|-------------------|----------|
| `user.{userId}.sync` | 100 messages | 24 hours | State sync history |
| `user.{userId}.devices` | 25 messages | 1 hour | Recent device events |
| `user.{userId}.notifications` | 50 messages | 7 days | Notification history |
| `global.trending` | 24 messages | 24 hours | Hourly trending updates |
| `global.announcements` | 100 messages | 30 days | System announcements |
| `region.{code}.updates` | 500 messages | 7 days | Regional catalog changes |
| `platform.{id}.catalog` | 1000 messages | 14 days | Platform catalog history |

**Fetch History API**:

```rust
use pubnub::{Pubnub, FetchHistoryRequestBuilder};

async fn fetch_sync_history(
    pubnub: &Pubnub,
    user_id: &str,
    limit: usize,
) -> Result<Vec<WatchProgressMessage>, Error> {
    let channel = format!("user.{}.sync", user_id);

    let response = pubnub
        .history()
        .channel(channel)
        .count(limit)
        .execute()
        .await?;

    let messages: Vec<WatchProgressMessage> = response
        .messages
        .into_iter()
        .filter_map(|msg| serde_json::from_value(msg.message).ok())
        .collect();

    Ok(messages)
}
```

### 9.2 State Persistence Strategy

**Two-Tier Persistence**:

1. **Hot Storage (Memorystore/Valkey)**: Recent state for fast loading
2. **Cold Storage (PostgreSQL)**: Full state history for auditing

**Data Flow**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       STATE PERSISTENCE ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Device (Publisher)                                                         │
│       │                                                                     │
│       │ 1. Publish state update                                            │
│       ▼                                                                     │
│  ┌─────────────────┐                                                       │
│  │    PubNub       │                                                       │
│  │   (Broadcast)   │                                                       │
│  └────────┬────────┘                                                       │
│           │                                                                 │
│           │ 2. Message received                                            │
│           ▼                                                                 │
│  ┌─────────────────┐                                                       │
│  │  Sync Engine    │                                                       │
│  │  (Subscriber)   │                                                       │
│  └────────┬────────┘                                                       │
│           │                                                                 │
│           ├─────────────────┬──────────────────┐                           │
│           │                 │                  │                           │
│           ▼                 ▼                  ▼                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                    │
│  │ Memorystore  │  │ PostgreSQL   │  │ Pub/Sub      │                    │
│  │ (Hot Cache)  │  │ (Persistence)│  │ (Analytics)  │                    │
│  │              │  │              │  │              │                    │
│  │ TTL: 30 days │  │ Retention:   │  │ Topic:       │                    │
│  │ Key: user:   │  │ Indefinite   │  │ user-events  │                    │
│  │ {id}:state   │  │              │  │              │                    │
│  └──────────────┘  └──────────────┘  └──────────────┘                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**PostgreSQL Schema**:

```sql
-- User state snapshots
CREATE TABLE user_state_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL,
    snapshot_type TEXT NOT NULL,  -- 'watch_progress', 'watchlist', 'preferences'
    state_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_user_snapshots (user_id, snapshot_type, created_at DESC)
);

-- State change events (audit log)
CREATE TABLE user_state_events (
    id BIGSERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_data JSONB NOT NULL,
    hlc_timestamp BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_user_events (user_id, created_at DESC),
    INDEX idx_hlc_timestamp (hlc_timestamp)
);
```

**Persistence Triggers**:

1. **Immediate**: Watch progress updates (every 10s)
2. **Batched**: Watchlist changes (every 5 minutes)
3. **On-Demand**: Full state snapshot (on app background/terminate)

---

## 10. Performance Requirements

### 10.1 Latency SLOs

| Metric | p50 Target | p99 Target | p99.9 Target | Measurement |
|--------|------------|------------|--------------|-------------|
| **Publish Latency** | 25ms | 75ms | 150ms | Client → PubNub acknowledgment |
| **Delivery Latency** | 50ms | 100ms | 200ms | Publish → Subscriber receive |
| **End-to-End Sync** | 75ms | 150ms | 300ms | Device A update → Device B UI |
| **Presence Detection** | 100ms | 250ms | 500ms | Device join → Presence event |
| **History Fetch** | 100ms | 300ms | 1000ms | API call → Response |
| **State Reconciliation** | 200ms | 500ms | 1500ms | Request → Full state delivered |

### 10.2 Throughput Requirements

| Operation | Target Rate | Peak Rate | Scaling Strategy |
|-----------|-------------|-----------|------------------|
| **Watch Progress Updates** | 1,000 msg/s | 5,000 msg/s | PubNub auto-scales |
| **Watchlist Updates** | 500 msg/s | 2,000 msg/s | Batching on client |
| **Device Heartbeats** | 10,000 msg/s | 50,000 msg/s | Presence optimization |
| **Remote Commands** | 100 msg/s | 500 msg/s | Command validation |
| **Notifications** | 5,000 msg/s | 25,000 msg/s | Backend fanout |

### 10.3 Availability and Reliability

**PubNub SLA**: 99.999% uptime (5 nines)

**Reliability Mechanisms**:

1. **Automatic Reconnection**: Client SDK reconnects on network loss
2. **Message Deduplication**: Unique message IDs prevent duplicate processing
3. **At-Least-Once Delivery**: PubNub guarantees message delivery
4. **FIFO Ordering**: Messages arrive in publish order per channel
5. **Backpressure Handling**: Client-side queue for offline buffering

**Client Reconnection Logic**:

```rust
impl DeviceGatewayClient {
    async fn handle_reconnection(&mut self) -> Result<(), Error> {
        let mut retry_interval = Duration::from_secs(1);
        let max_interval = Duration::from_secs(30);

        loop {
            match self.pubnub.reconnect().await {
                Ok(_) => {
                    // Reconnected successfully
                    self.on_reconnected().await?;
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("Reconnection failed: {}, retrying in {:?}", e, retry_interval);

                    // Exponential backoff
                    tokio::time::sleep(retry_interval).await;
                    retry_interval = std::cmp::min(retry_interval * 2, max_interval);
                }
            }
        }
    }

    async fn on_reconnected(&mut self) -> Result<(), Error> {
        // 1. Resubscribe to channels
        self.subscribe_to_user_channels().await?;

        // 2. Request state reconciliation
        self.request_state_reconciliation().await?;

        // 3. Resume heartbeat
        self.start_heartbeat().await?;

        Ok(())
    }
}
```

---

## 11. Security and Access Control

### 11.1 PubNub Access Manager (PAM)

**Grant Hierarchy**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    PUBNUB ACCESS CONTROL HIERARCHY                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  USER-SPECIFIC CHANNELS:                                                    │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ user.{userId}.sync                                                     │ │
│  │   Read:  user:{userId}                                                │ │
│  │   Write: user:{userId}                                                │ │
│  │   TTL:   24 hours (token refresh)                                     │ │
│  ├───────────────────────────────────────────────────────────────────────┤ │
│  │ user.{userId}.devices                                                  │ │
│  │   Read:  user:{userId}, admin:*                                       │ │
│  │   Write: user:{userId}                                                │ │
│  │   TTL:   24 hours                                                     │ │
│  ├───────────────────────────────────────────────────────────────────────┤ │
│  │ user.{userId}.notifications                                            │ │
│  │   Read:  user:{userId}                                                │ │
│  │   Write: backend:notification-service                                 │ │
│  │   TTL:   24 hours                                                     │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  GLOBAL CHANNELS:                                                           │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │ global.trending                                                        │ │
│  │   Read:  * (public)                                                   │ │
│  │   Write: backend:recommendation-engine                                │ │
│  ├───────────────────────────────────────────────────────────────────────┤ │
│  │ global.announcements                                                   │ │
│  │   Read:  * (public)                                                   │ │
│  │   Write: admin:announcement-service                                   │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Token Generation**:

```rust
use pubnub::grant::{GrantTokenBuilder, ResourceType};

async fn generate_user_token(
    pubnub: &Pubnub,
    user_id: &str,
) -> Result<String, Error> {
    let token = pubnub
        .grant_token()
        .ttl(86400)  // 24 hours
        .authorized_uuid(format!("user:{}", user_id))
        // Grant read/write to user's sync channel
        .resources(vec![
            (ResourceType::Channel, format!("user.{}.sync", user_id), vec!["read", "write"]),
            (ResourceType::Channel, format!("user.{}.devices", user_id), vec!["read", "write"]),
            (ResourceType::Channel, format!("user.{}.notifications", user_id), vec!["read"]),
        ])
        // Grant read to global channels
        .patterns(vec![
            (ResourceType::Channel, "global.*", vec!["read"]),
            (ResourceType::Channel, "region.*", vec!["read"]),
        ])
        .execute()
        .await?;

    Ok(token.token)
}
```

### 11.2 Message Encryption

**Encryption Strategy**: End-to-end encryption for sensitive user data

**Encryption Scope**:
- **Encrypted**: Watch progress, preferences (contains user behavior)
- **Unencrypted**: Watchlist (public content IDs), device capabilities

**Implementation**:

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

/// Encrypt message payload before publishing
pub fn encrypt_message(
    plaintext: &[u8],
    user_encryption_key: &[u8; 32],
) -> Result<Vec<u8>, Error> {
    let key = Key::from_slice(user_encryption_key);
    let cipher = Aes256Gcm::new(key);

    // Generate random nonce
    let nonce = Nonce::from_slice(&rand::random::<[u8; 12]>());

    // Encrypt
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| Error::EncryptionFailed(e.to_string()))?;

    // Prepend nonce to ciphertext
    let mut encrypted = nonce.to_vec();
    encrypted.extend_from_slice(&ciphertext);

    Ok(encrypted)
}

/// Decrypt message payload after receiving
pub fn decrypt_message(
    encrypted: &[u8],
    user_encryption_key: &[u8; 32],
) -> Result<Vec<u8>, Error> {
    if encrypted.len() < 12 {
        return Err(Error::InvalidEncryptedMessage);
    }

    // Extract nonce and ciphertext
    let (nonce, ciphertext) = encrypted.split_at(12);

    let key = Key::from_slice(user_encryption_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    // Decrypt
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| Error::DecryptionFailed(e.to_string()))?;

    Ok(plaintext)
}
```

**Key Management**:
- **User Encryption Key**: Derived from user password (PBKDF2, 100k iterations)
- **Storage**: User encryption key stored in device keychain (never sent to server)
- **Rotation**: Key rotated on password change

---

## 12. Implementation Specifications

### 12.1 Rust PubNub Client Wrapper

**Repository**: `mg-pubnub-client`

**Directory Structure**:

```
mg-pubnub-client/
├── Cargo.toml
├── src/
│   ├── lib.rs               # Public API
│   ├── client.rs            # PubNub client wrapper
│   ├── channels.rs          # Channel management
│   ├── presence.rs          # Presence features
│   ├── encryption.rs        # Message encryption
│   ├── crdt/
│   │   ├── lww_register.rs  # LWW-Register implementation
│   │   ├── or_set.rs        # OR-Set implementation
│   │   └── hlc.rs           # Hybrid Logical Clock
│   ├── messages/
│   │   ├── watch_progress.rs
│   │   ├── watchlist.rs
│   │   ├── device.rs
│   │   └── notification.rs
│   └── error.rs             # Error types
└── examples/
    ├── basic_pub_sub.rs
    ├── crdt_sync.rs
    └── device_presence.rs
```

**Cargo.toml Dependencies**:

```toml
[package]
name = "mg-pubnub-client"
version = "0.1.0"
edition = "2021"

[dependencies]
pubnub = "0.4"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
aes-gcm = "0.10"
rand = "0.8"
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
thiserror = "1.0"
tracing = "0.1"

[dev-dependencies]
tokio-test = "0.4"
```

### 12.2 Key Rust Structures

**Main Client**:

```rust
use pubnub::{Pubnub, PubNubClientBuilder};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Media Gateway PubNub client
pub struct MediaGatewayPubNub {
    /// Underlying PubNub client
    pubnub: Arc<Pubnub>,

    /// User ID
    user_id: String,

    /// Device ID
    device_id: String,

    /// User encryption key
    encryption_key: [u8; 32],

    /// HLC for timestamp generation
    hlc: Arc<HybridLogicalClock>,

    /// Local state cache
    state_cache: Arc<RwLock<StateCache>>,
}

impl MediaGatewayPubNub {
    /// Create new client instance
    pub async fn new(
        pubnub_publish_key: &str,
        pubnub_subscribe_key: &str,
        user_id: String,
        device_id: String,
        auth_token: String,
        encryption_key: [u8; 32],
    ) -> Result<Self, Error> {
        let pubnub = PubNubClientBuilder::with_reqwest_transport()
            .with_keyset(Keyset {
                publish_key: Some(pubnub_publish_key.to_string()),
                subscribe_key: pubnub_subscribe_key.to_string(),
                secret_key: None,
            })
            .with_user_id(format!("user:{}", user_id))
            .with_auth_key(auth_token)
            .build()?;

        Ok(Self {
            pubnub: Arc::new(pubnub),
            user_id,
            device_id,
            encryption_key,
            hlc: Arc::new(HybridLogicalClock::new()),
            state_cache: Arc::new(RwLock::new(StateCache::new())),
        })
    }

    /// Subscribe to user channels
    pub async fn subscribe_to_user_channels(&self) -> Result<(), Error> {
        let channels = vec![
            format!("user.{}.sync", self.user_id),
            format!("user.{}.devices", self.user_id),
            format!("user.{}.notifications", self.user_id),
            "global.trending".to_string(),
            "global.announcements".to_string(),
        ];

        self.pubnub.subscribe()
            .channels(channels)
            .execute()?;

        Ok(())
    }

    /// Publish watch progress update
    pub async fn publish_watch_progress(
        &self,
        content_id: String,
        progress_seconds: u32,
        duration_seconds: u32,
        state: PlaybackState,
    ) -> Result<(), Error> {
        let message = WatchProgressMessage {
            msg_type: "watch_progress_update".to_string(),
            content_id,
            progress_seconds,
            duration_seconds,
            completion_percent: progress_seconds as f32 / duration_seconds as f32,
            timestamp: self.hlc.now(),
            device_id: self.device_id.clone(),
            device_type: DeviceType::TV,  // TODO: detect
            state,
        };

        let channel = format!("user.{}.sync", self.user_id);
        let payload = serde_json::to_vec(&message)?;

        // Encrypt if needed
        let encrypted_payload = encrypt_message(&payload, &self.encryption_key)?;

        self.pubnub.publish()
            .channel(channel)
            .message(encrypted_payload)
            .execute()
            .await?;

        Ok(())
    }
}
```

### 12.3 Configuration

**PubNub Configuration File** (`config/pubnub.yaml`):

```yaml
pubnub:
  # PubNub keys (from environment variables)
  publish_key: "${PUBNUB_PUBLISH_KEY}"
  subscribe_key: "${PUBNUB_SUBSCRIBE_KEY}"
  secret_key: "${PUBNUB_SECRET_KEY}"  # For PAM token generation

  # Client configuration
  origin: "ps.pndsn.com"
  secure: true
  log_verbosity: "info"

  # Presence configuration
  presence:
    enabled: true
    heartbeat_interval: 30  # seconds
    presence_timeout: 60    # seconds
    announce_max: 25        # devices

  # Message retention
  history:
    user_sync:
      count: 100
      duration_hours: 24
    user_devices:
      count: 25
      duration_hours: 1
    user_notifications:
      count: 50
      duration_hours: 168  # 7 days
    global_trending:
      count: 24
      duration_hours: 24
    global_announcements:
      count: 100
      duration_hours: 720  # 30 days
    regional_updates:
      count: 500
      duration_hours: 168  # 7 days

  # Access control
  pam:
    enabled: true
    default_ttl: 86400  # 24 hours

  # Encryption
  encryption:
    enabled: true
    algorithm: "AES-256-GCM"
    encrypt_channels:
      - "user.*.sync"
      - "user.*.notifications"
```

---

## Summary

This specification defines a comprehensive real-time synchronization architecture for the Media Gateway using PubNub as the messaging backbone. Key takeaways:

1. **Channel Architecture**: Hierarchical channel topology (user, global, regional, platform)
2. **Message Types**: 10+ message types with structured payloads (watch progress, watchlist, device control, etc.)
3. **CRDT-Based Sync**: LWW-Register and OR-Set for automatic conflict resolution
4. **Device Coordination**: Presence management, heartbeat, and remote control capabilities
5. **Performance**: Sub-100ms latency targets with 99.999% availability
6. **Security**: PAM-based access control and AES-256-GCM encryption

**File Paths Referenced**:
- `/tmp/media-gateway-research/README.md`
- `/tmp/media-gateway-research/research/FINAL_ARCHITECTURE_BLUEPRINT.md`
- `/tmp/media-gateway-research/research/GCP_DEPLOYMENT_ARCHITECTURE.md`
- `/tmp/media-gateway-research/research/SONA_INTEGRATION_SPECIFICATION.md`

**Next Steps**: Implement `mg-pubnub-client` Rust crate and integrate with `mg-device-gateway` and `mg-sync-engine` services.
