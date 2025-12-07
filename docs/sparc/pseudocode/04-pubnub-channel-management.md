# PubNub Channel Management Pseudocode

## Overview
Provides real-time message routing, subscription management, and channel-based synchronization using PubNub infrastructure.

## Data Structures

```
STRUCTURE PubNubClient:
    subscribe_key: string
    publish_key: string
    user_id: string
    device_id: string
    subscriptions: Set<string>          // Active channel subscriptions
    message_handlers: Map<string, MessageHandler>
    presence_handlers: Map<string, PresenceHandler>
    connection_state: ConnectionState

STRUCTURE ConnectionState:
    status: enum(DISCONNECTED, CONNECTING, CONNECTED, RECONNECTING, ERROR)
    connected_at: timestamp
    reconnect_count: integer
    last_error: string

STRUCTURE Message:
    channel: string
    message_type: string
    payload: any
    timestamp: HLC
    sender_device_id: string
    message_id: string             // UUID for deduplication

STRUCTURE PresenceEvent:
    channel: string
    action: enum(JOIN, LEAVE, TIMEOUT, STATE_CHANGE)
    uuid: string
    timestamp: timestamp
    state: any

CONSTANTS:
    MAX_CHANNELS_PER_SUBSCRIBE = 50
    MESSAGE_TIMEOUT = 5000              // 5 seconds
    HEARTBEAT_INTERVAL = 30000          // 30 seconds
    PRESENCE_TIMEOUT = 60000            // 60 seconds
    MAX_RETRY_ATTEMPTS = 5
    BACKOFF_MULTIPLIER = 2
```

## Channel Naming Conventions

```
ALGORITHM: GetChannelName
INPUT: channel_type (string), user_id (string), resource_id (string)
OUTPUT: channel_name (string)

CHANNEL STRUCTURE:
    user.{userId}.sync              // User state synchronization
    user.{userId}.devices           // Device presence
    user.{userId}.notifications     // User notifications
    user.{userId}.watchlist         // Watchlist updates
    user.{userId}.progress          // Watch progress
    user.{userId}.preferences       // User preferences
    global.trending                 // Global trending content
    global.announcements            // System announcements
    content.{mediaId}.comments      // Content-specific comments
    content.{mediaId}.reactions     // Content reactions

BEGIN
    CASE channel_type OF
        "sync":
            RETURN CONCAT("user.", user_id, ".sync")

        "devices":
            RETURN CONCAT("user.", user_id, ".devices")

        "notifications":
            RETURN CONCAT("user.", user_id, ".notifications")

        "watchlist":
            RETURN CONCAT("user.", user_id, ".watchlist")

        "progress":
            RETURN CONCAT("user.", user_id, ".progress")

        "preferences":
            RETURN CONCAT("user.", user_id, ".preferences")

        "trending":
            RETURN "global.trending"

        "announcements":
            RETURN "global.announcements"

        "comments":
            RETURN CONCAT("content.", resource_id, ".comments")

        "reactions":
            RETURN CONCAT("content.", resource_id, ".reactions")

        DEFAULT:
            RETURN error("Unknown channel type")
    END CASE
END
```

## Core Channel Operations

### 1. Client Initialization

```
ALGORITHM: InitializePubNubClient
INPUT: config (PubNubConfig)
OUTPUT: client (PubNubClient)

BEGIN
    client ← PubNubClient()
    client.subscribe_key ← config.subscribe_key
    client.publish_key ← config.publish_key
    client.user_id ← config.user_id
    client.device_id ← config.device_id OR GenerateDeviceId()
    client.subscriptions ← EmptySet()
    client.message_handlers ← EmptyMap()
    client.presence_handlers ← EmptyMap()

    // Initialize connection state
    client.connection_state ← ConnectionState()
    client.connection_state.status ← DISCONNECTED
    client.connection_state.reconnect_count ← 0

    // Setup global message handler
    SetupGlobalMessageHandler(client)

    // Setup global presence handler
    SetupGlobalPresenceHandler(client)

    // Setup connection status listeners
    SetupConnectionListeners(client)

    LogInfo("PubNub client initialized", client.device_id)

    RETURN client
END
```

### 2. Subscribe to Channels

```
ALGORITHM: SubscribeToChannels
INPUT: client (PubNubClient), channels (array of string), with_presence (boolean)
OUTPUT: success (boolean)

BEGIN
    // Validate channels
    IF Length(channels) == 0 THEN
        RETURN error("No channels specified")
    END IF

    IF Length(channels) > MAX_CHANNELS_PER_SUBSCRIBE THEN
        RETURN error("Too many channels")
    END IF

    // Build subscription list
    subscribe_list ← []

    FOR EACH channel IN channels DO
        // Add channel
        subscribe_list.append(channel)
        client.subscriptions.add(channel)

        // Add presence channel if requested
        IF with_presence THEN
            presence_channel ← CONCAT(channel, "-pnpres")
            subscribe_list.append(presence_channel)
            client.subscriptions.add(presence_channel)
        END IF
    END FOR

    // Execute subscribe
    TRY
        PubNubSDK.subscribe({
            channels: subscribe_list,
            withPresence: with_presence,
            timetoken: GetCurrentTimetoken()
        })

        LogInfo("Subscribed to channels", Length(channels))
        RETURN true

    CATCH error
        LogError("Subscribe failed", error.message)
        RETURN false
    END TRY
END
```

### 3. Unsubscribe from Channels

```
ALGORITHM: UnsubscribeFromChannels
INPUT: client (PubNubClient), channels (array of string)
OUTPUT: success (boolean)

BEGIN
    IF Length(channels) == 0 THEN
        RETURN error("No channels specified")
    END IF

    // Build unsubscribe list
    unsubscribe_list ← []

    FOR EACH channel IN channels DO
        IF client.subscriptions.contains(channel) THEN
            unsubscribe_list.append(channel)
            client.subscriptions.remove(channel)

            // Also unsubscribe from presence channel
            presence_channel ← CONCAT(channel, "-pnpres")
            IF client.subscriptions.contains(presence_channel) THEN
                unsubscribe_list.append(presence_channel)
                client.subscriptions.remove(presence_channel)
            END IF

            // Remove handlers
            client.message_handlers.delete(channel)
            client.presence_handlers.delete(channel)
        END IF
    END FOR

    // Execute unsubscribe
    IF Length(unsubscribe_list) > 0 THEN
        PubNubSDK.unsubscribe({
            channels: unsubscribe_list
        })

        LogInfo("Unsubscribed from channels", Length(unsubscribe_list))
    END IF

    RETURN true
END
```

### 4. Publish Message

```
ALGORITHM: PublishMessage
INPUT:
    client (PubNubClient),
    channel (string),
    message_type (string),
    payload (any),
    hlc (HLC)
OUTPUT: message_id (string) or error

BEGIN
    // Validate channel
    IF NOT IsValidChannel(channel) THEN
        RETURN error("Invalid channel")
    END IF

    // Create message envelope
    message ← Message()
    message.channel ← channel
    message.message_type ← message_type
    message.payload ← payload
    message.timestamp ← hlc
    message.sender_device_id ← client.device_id
    message.message_id ← GenerateUUID()

    // Serialize message
    message_json ← SerializeMessage(message)

    // Publish with retry logic
    retry_count ← 0
    backoff_delay ← 100  // Start with 100ms

    WHILE retry_count < MAX_RETRY_ATTEMPTS DO
        TRY
            result ← PubNubSDK.publish({
                channel: channel,
                message: message_json,
                storeInHistory: true,
                ttl: 24  // Store for 24 hours
            })

            IF result.success THEN
                LogInfo("Message published", channel, message.message_id)
                RETURN message.message_id
            END IF

        CATCH error
            LogWarning("Publish failed, retrying", retry_count, error.message)
        END TRY

        // Exponential backoff
        Sleep(backoff_delay)
        backoff_delay ← backoff_delay * BACKOFF_MULTIPLIER
        retry_count ← retry_count + 1
    END WHILE

    LogError("Publish failed after retries", channel)
    RETURN error("Publish failed")
END
```

### 5. Message Handler Registration

```
ALGORITHM: RegisterMessageHandler
INPUT:
    client (PubNubClient),
    channel (string),
    handler (MessageHandler)
OUTPUT: success (boolean)

BEGIN
    // Validate inputs
    IF channel is empty OR handler is null THEN
        RETURN false
    END IF

    // Register handler
    client.message_handlers.set(channel, handler)

    LogInfo("Message handler registered", channel)
    RETURN true
END

ALGORITHM: HandleIncomingMessage
INPUT: client (PubNubClient), raw_message (any)
OUTPUT: processed (boolean)

BEGIN
    // Deserialize message
    message ← DeserializeMessage(raw_message)

    IF message is null THEN
        LogWarning("Invalid message received")
        RETURN false
    END IF

    // Check for duplicate (idempotence)
    IF IsDuplicateMessage(message.message_id) THEN
        LogDebug("Duplicate message ignored", message.message_id)
        RETURN true
    END IF

    // Mark as processed
    MarkMessageProcessed(message.message_id)

    // Route to channel-specific handler
    IF client.message_handlers.has(message.channel) THEN
        handler ← client.message_handlers.get(message.channel)

        TRY
            handler(message)
            RETURN true

        CATCH error
            LogError("Handler failed", message.channel, error.message)
            RETURN false
        END TRY
    ELSE
        // No specific handler - use default
        LogDebug("No handler for channel", message.channel)
        RETURN false
    END IF
END
```

### 6. Presence Event Handling

```
ALGORITHM: HandlePresenceEvent
INPUT: client (PubNubClient), presence_event (PresenceEvent)
OUTPUT: processed (boolean)

BEGIN
    channel ← presence_event.channel

    LogInfo("Presence event", channel, presence_event.action, presence_event.uuid)

    // Route to channel-specific presence handler
    IF client.presence_handlers.has(channel) THEN
        handler ← client.presence_handlers.get(channel)

        TRY
            handler(presence_event)
            RETURN true

        CATCH error
            LogError("Presence handler failed", channel, error.message)
            RETURN false
        END TRY
    END IF

    // Default presence handling
    CASE presence_event.action OF
        JOIN:
            OnDeviceJoined(presence_event)

        LEAVE:
            OnDeviceLeft(presence_event)

        TIMEOUT:
            OnDeviceTimeout(presence_event)

        STATE_CHANGE:
            OnDeviceStateChanged(presence_event)
    END CASE

    RETURN true
END
```

### 7. History Fetching

```
ALGORITHM: FetchMessageHistory
INPUT:
    client (PubNubClient),
    channel (string),
    start_time (timestamp),
    end_time (timestamp),
    limit (integer)
OUTPUT: messages (array of Message)

BEGIN
    // Validate inputs
    IF limit > 100 THEN
        limit ← 100  // Cap at 100 messages
    END IF

    // Convert timestamps to timetokens
    start_timetoken ← TimestampToTimetoken(start_time)
    end_timetoken ← TimestampToTimetoken(end_time)

    TRY
        result ← PubNubSDK.history({
            channel: channel,
            start: start_timetoken,
            end: end_timetoken,
            count: limit,
            reverse: false,  // Oldest first
            includeTimetoken: true
        })

        messages ← []

        FOR EACH raw_message IN result.messages DO
            message ← DeserializeMessage(raw_message.entry)
            message.timetoken ← raw_message.timetoken
            messages.append(message)
        END FOR

        LogInfo("Fetched message history", channel, Length(messages))
        RETURN messages

    CATCH error
        LogError("History fetch failed", channel, error.message)
        RETURN []
    END TRY
END
```

### 8. Connection State Management

```
ALGORITHM: HandleConnectionStateChange
INPUT: client (PubNubClient), new_state (ConnectionState)
OUTPUT: none

BEGIN
    old_state ← client.connection_state.status
    client.connection_state.status ← new_state

    LogInfo("Connection state changed", old_state, new_state)

    CASE new_state OF
        CONNECTED:
            client.connection_state.connected_at ← GetCurrentTime()
            client.connection_state.reconnect_count ← 0
            OnConnected(client)

        DISCONNECTED:
            OnDisconnected(client)

        RECONNECTING:
            client.connection_state.reconnect_count ← client.connection_state.reconnect_count + 1
            OnReconnecting(client)

        ERROR:
            OnConnectionError(client)
    END CASE
END

ALGORITHM: OnConnected
INPUT: client (PubNubClient)
OUTPUT: none

BEGIN
    LogInfo("Connected to PubNub")

    // Resubscribe to all channels
    IF NOT client.subscriptions.isEmpty() THEN
        channels ← client.subscriptions.toArray()
        SubscribeToChannels(client, channels, true)
    END IF

    // Fetch missed messages (offline sync)
    FetchMissedMessages(client)

    // Update presence state
    UpdatePresenceState(client, {status: "online"})
END
```

### 9. Channel Group Management

```
ALGORITHM: CreateChannelGroup
INPUT:
    client (PubNubClient),
    group_name (string),
    channels (array of string)
OUTPUT: success (boolean)

BEGIN
    TRY
        PubNubSDK.channelGroups.addChannels({
            channelGroup: group_name,
            channels: channels
        })

        LogInfo("Channel group created", group_name, Length(channels))
        RETURN true

    CATCH error
        LogError("Channel group creation failed", error.message)
        RETURN false
    END TRY
END

ALGORITHM: SubscribeToChannelGroup
INPUT: client (PubNubClient), group_name (string)
OUTPUT: success (boolean)

BEGIN
    TRY
        PubNubSDK.subscribe({
            channelGroups: [group_name],
            withPresence: true
        })

        LogInfo("Subscribed to channel group", group_name)
        RETURN true

    CATCH error
        LogError("Channel group subscription failed", error.message)
        RETURN false
    END TRY
END
```

## Message Routing Strategies

### 1. User-Specific Routing

```
ALGORITHM: RouteUserMessage
INPUT: user_id (string), message_type (string), payload (any)
OUTPUT: success (boolean)

BEGIN
    // Determine target channel based on message type
    CASE message_type OF
        "watch_progress":
            channel ← GetChannelName("progress", user_id, null)

        "watchlist_update":
            channel ← GetChannelName("watchlist", user_id, null)

        "preference_change":
            channel ← GetChannelName("preferences", user_id, null)

        "device_command":
            channel ← GetChannelName("devices", user_id, null)

        "notification":
            channel ← GetChannelName("notifications", user_id, null)

        DEFAULT:
            channel ← GetChannelName("sync", user_id, null)
    END CASE

    // Publish to channel
    RETURN PublishMessage(client, channel, message_type, payload, current_hlc)
END
```

## Complexity Analysis

### Time Complexity
- `InitializePubNubClient`: O(1)
- `SubscribeToChannels`: O(n) where n = number of channels
- `UnsubscribeFromChannels`: O(n)
- `PublishMessage`: O(1) + network latency
- `HandleIncomingMessage`: O(1) + handler complexity
- `FetchMessageHistory`: O(m) where m = number of messages

### Space Complexity
- PubNubClient: O(s + h) where s = subscriptions, h = handlers
- Message: O(p) where p = payload size
- History: O(m × p) for m messages

## Performance Characteristics

- **Publish Latency**: <50ms (P50), <100ms (P99)
- **Message Delivery**: <100ms globally
- **Max Channels**: 50 per subscribe call
- **Max Message Size**: 32KB
- **History Storage**: 24 hours (configurable)
- **Presence Timeout**: 60 seconds
- **Heartbeat**: 30 seconds

## Edge Cases

1. **Network Partition**: Auto-reconnect with exponential backoff
2. **Message Loss**: History fetch on reconnect
3. **Duplicate Messages**: Deduplication via message_id
4. **Channel Overflow**: Split into multiple subscribe calls
5. **Large Payloads**: Compression or chunking
