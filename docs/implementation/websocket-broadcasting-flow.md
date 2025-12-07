# WebSocket Broadcasting Data Flow

## Complete Message Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          CLIENT DEVICE A (Source)                            │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ User Action: Add movie to watchlist                                    │ │
│  │ POST /api/v1/sync/watchlist { operation: "add", content_id: "123" }   │ │
│  └────────────────────────────────┬───────────────────────────────────────┘ │
└─────────────────────────────────────┼───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SYNC SERVICE (Backend)                               │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ 1. WatchlistSync.add_to_watchlist("123")                               │ │
│  │    - Create OR-Set operation                                           │ │
│  │    - Generate HLC timestamp                                            │ │
│  │    - Update local state                                                │ │
│  └────────────────────────────────┬───────────────────────────────────────┘ │
│                                   │                                          │
│  ┌────────────────────────────────▼───────────────────────────────────────┐ │
│  │ 2. PubNubClient.publish()                                              │ │
│  │    Channel: "user.{user_id}.sync"                                      │ │
│  │    Message: {                                                          │ │
│  │      type: "watchlist_update",                                         │ │
│  │      operation: "add",                                                 │ │
│  │      content_id: "123",                                                │ │
│  │      unique_tag: "movie-123",                                          │ │
│  │      timestamp: HLCTimestamp { ... },                                  │ │
│  │      device_id: "device-a"                                             │ │
│  │    }                                                                   │ │
│  └────────────────────────────────┬───────────────────────────────────────┘ │
└─────────────────────────────────────┼───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PUBNUB CLOUD                                       │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ Channel: user.{user_id}.sync                                           │ │
│  │ Message stored and distributed to all subscribers                      │ │
│  └────────────────────────────────┬───────────────────────────────────────┘ │
└─────────────────────────────────────┼───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│               SYNC SERVICE - WEBSOCKET BROADCASTER                           │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ 3. PubNub Long-Poll Receives Message                                   │ │
│  │    SubscriptionManager.poll_messages() → message received              │ │
│  └────────────────────────────────┬───────────────────────────────────────┘ │
│                                   │                                          │
│  ┌────────────────────────────────▼───────────────────────────────────────┐ │
│  │ 4. BroadcasterMessageHandler.handle_sync_message()                     │ │
│  │    - Receives PubNubSyncMessage::WatchlistUpdate                       │ │
│  │    - Calls broadcaster.relay_pubnub_message()                          │ │
│  └────────────────────────────────┬───────────────────────────────────────┘ │
│                                   │                                          │
│  ┌────────────────────────────────▼───────────────────────────────────────┐ │
│  │ 5. WebSocketBroadcaster.relay_pubnub_message()                         │ │
│  │    START TIMER (for latency metric)                                    │ │
│  │                                                                         │ │
│  │    5a. convert_pubnub_message()                                        │ │
│  │        PubNubSyncMessage → SyncMessage                                 │ │
│  │        {                                                               │ │
│  │          type: "watchlist_update",                                     │ │
│  │          content_id: Uuid("123"),                                      │ │
│  │          action: "add"                                                 │ │
│  │        }                                                               │ │
│  │                                                                         │ │
│  │    5b. registry.send_to_user(user_id, ws_message)                      │ │
│  │        - Serialize message to JSON                                     │ │
│  │        - Look up all connections for user_id                           │ │
│  │        - Broadcast to all connections                                  │ │
│  │                                                                         │ │
│  │    STOP TIMER                                                          │ │
│  │    metrics.record_message_relayed()                                    │ │
│  │    metrics.record_latency(elapsed_ms)                                  │ │
│  └────────────────────────────────┬───────────────────────────────────────┘ │
└─────────────────────────────────────┼───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      CONNECTION REGISTRY                                     │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ 6. send_to_user(user_id, message)                                      │ │
│  │                                                                         │ │
│  │    user_connections.get(user_id) → Vec<ConnectionInfo>                │ │
│  │    [                                                                   │ │
│  │      { conn_id: uuid-1, device_id: "device-b", addr: Addr<WS> },      │ │
│  │      { conn_id: uuid-2, device_id: "device-c", addr: Addr<WS> },      │ │
│  │      { conn_id: uuid-3, device_id: "device-d", addr: Addr<WS> }       │ │
│  │    ]                                                                   │ │
│  │                                                                         │ │
│  │    For each connection:                                                │ │
│  │      conn.addr.do_send(BroadcastMessage(json))                         │ │
│  │                                                                         │ │
│  │    metrics.messages_sent += 3                                          │ │
│  │    Return: Ok(3) // sent to 3 connections                              │ │
│  └────────────────────────────────┬───────────────────────────────────────┘ │
└─────────────────────────────────────┼───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SYNCWEBSOCKET ACTORS (3 instances)                        │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ 7. Handler<BroadcastMessage>::handle(msg, ctx)                         │ │
│  │                                                                         │ │
│  │    ctx.text(msg.0) // Send JSON string over WebSocket                  │ │
│  │                                                                         │ │
│  │    Device B Actor: ctx.text('{"type":"watchlist_update",...}')         │ │
│  │    Device C Actor: ctx.text('{"type":"watchlist_update",...}')         │ │
│  │    Device D Actor: ctx.text('{"type":"watchlist_update",...}')         │ │
│  └────────────────────────────────┬───────────────────────────────────────┘ │
└─────────────────────────────────────┼───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│              CLIENT DEVICES B, C, D (Receivers)                              │
│                                                                              │
│  ┌──────────────────────┐  ┌──────────────────────┐  ┌──────────────────┐  │
│  │   DEVICE B           │  │   DEVICE C           │  │   DEVICE D       │  │
│  │   (Mobile)           │  │   (Smart TV)         │  │   (Browser)      │  │
│  ├──────────────────────┤  ├──────────────────────┤  ├──────────────────┤  │
│  │ WebSocket.onmessage  │  │ WebSocket.onmessage  │  │ WebSocket.onmess │  │
│  │                      │  │                      │  │                  │  │
│  │ Receive:             │  │ Receive:             │  │ Receive:         │  │
│  │ {                    │  │ {                    │  │ {                │  │
│  │   type: "watchlist_  │  │   type: "watchlist_  │  │   type: "watchl  │  │
│  │         update",     │  │         update",     │  │         update", │  │
│  │   content_id: "123", │  │   content_id: "123", │  │   content_id: "1 │  │
│  │   action: "add"      │  │   action: "add"      │  │   action: "add"  │  │
│  │ }                    │  │ }                    │  │ }                │  │
│  │                      │  │                      │  │                  │  │
│  │ Update UI:           │  │ Update UI:           │  │ Update UI:       │  │
│  │ ✓ Movie added to     │  │ ✓ Movie added to     │  │ ✓ Movie added to │  │
│  │   watchlist          │  │   watchlist          │  │   watchlist      │  │
│  └──────────────────────┘  └──────────────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Metrics Collected During Flow

```
┌────────────────────────────────────────────────┐
│         BROADCAST METRICS                      │
├────────────────────────────────────────────────┤
│ messages_relayed:     1                        │
│ broadcast_latency:    2.3ms                    │
│ active_connections:   3                        │
└────────────────────────────────────────────────┘

┌────────────────────────────────────────────────┐
│         REGISTRY METRICS                       │
├────────────────────────────────────────────────┤
│ total_connections:    3                        │
│ active_users:         1                        │
│ messages_sent:        3                        │
└────────────────────────────────────────────────┘
```

## Timing Breakdown

```
Event Timeline (milliseconds):

0.0 ms   │ Client A: POST /api/v1/sync/watchlist
         │
1.2 ms   │ Sync Service: WatchlistSync processes
         │
2.5 ms   │ PubNub: publish() API call
         │
45.0 ms  │ PubNub Cloud: Message distributed
         │
45.5 ms  │ Sync Service: Long-poll receives message
         │
45.8 ms  │ BroadcasterMessageHandler: handle_sync_message()
         │
46.0 ms  │ WebSocketBroadcaster: relay_pubnub_message() START
         │
46.1 ms  │   - convert_pubnub_message()
         │
46.2 ms  │   - registry.send_to_user()
         │
46.3 ms  │     - Lookup user connections
         │
46.4 ms  │     - Serialize to JSON
         │
46.5 ms  │     - Send to Device B actor
         │
46.6 ms  │     - Send to Device C actor
         │
46.7 ms  │     - Send to Device D actor
         │
46.8 ms  │ WebSocketBroadcaster: relay_pubnub_message() END
         │   Latency recorded: 0.8ms
         │
47.0 ms  │ Device B: WebSocket.onmessage fired
47.1 ms  │ Device C: WebSocket.onmessage fired
47.2 ms  │ Device D: WebSocket.onmessage fired
         │
50.0 ms  │ All devices: UI updated
```

**Total End-to-End Latency**: ~50ms
- PubNub network: ~43ms (variable)
- Broadcasting: ~2ms (optimized)
- WebSocket delivery: ~4ms (local)

## Concurrent User Scenario

```
┌──────────────────────────────────────────────────────────────────┐
│                  100 Concurrent Users                             │
│                  Each with 3 devices = 300 total connections     │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
        ┌───────────────────────────────────────────┐
        │      Single WebSocketBroadcaster Instance │
        │                                           │
        │  DashMap<UserId, Vec<ConnectionInfo>>    │
        │  [                                        │
        │    user-1 → [device-a, device-b, device-c]│
        │    user-2 → [device-a, device-b, device-c]│
        │    ...                                    │
        │    user-100 → [device-a, device-b, device-c]│
        │  ]                                        │
        │                                           │
        │  Performance:                             │
        │  - Lookup: O(1) with DashMap             │
        │  - Broadcast per user: O(devices)         │
        │  - Memory: 300 * 1KB = 300KB             │
        │  - CPU: Minimal (message passing)        │
        └───────────────────────────────────────────┘
```

## Failure Scenarios

### Scenario 1: Device Disconnects
```
Device B disconnects
         │
         ▼
SyncWebSocket::stopped()
         │
         ▼
registry.unregister(conn_id)
         │
         ▼
┌─────────────────────────────────┐
│ ConnectionRegistry              │
│ - Remove from connections map   │
│ - Remove from user_connections  │
│ - Decrement total_connections   │
└─────────────────────────────────┘
         │
         ▼
Future messages only sent to Device C & D
```

### Scenario 2: Invalid Message Format
```
PubNub message with invalid UUID
         │
         ▼
convert_pubnub_message() returns None
         │
         ▼
relay_pubnub_message() logs warning and returns
         │
         ▼
metrics.messages_relayed NOT incremented
```

### Scenario 3: PubNub Unavailable
```
PubNub network error
         │
         ▼
SubscriptionManager.poll_messages() returns Err
         │
         ▼
Log error, sleep 5s, retry
         │
         ▼
WebSocket connections remain active
Broadcaster continues with cached messages
```

## Security Flow

```
┌─────────────────────────────────────────────────┐
│ Client: WebSocket connection request            │
│ GET /ws                                         │
│ Authorization: Bearer <JWT>                     │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ Server: Extract & validate JWT                  │
│ - Verify signature                              │
│ - Check expiration                              │
│ - Extract user_id and device_id                 │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ Create SyncWebSocket actor                      │
│ - Bound to authenticated user_id                │
│ - Tagged with device_id                         │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│ Registry: Register connection                   │
│ - Only receive messages for this user_id        │
│ - Isolated from other users' messages           │
└─────────────────────────────────────────────────┘
```

---

**Generated**: 2025-12-06
**Version**: 1.0.0
**Status**: Production Documentation
