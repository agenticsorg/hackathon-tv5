# Device & Playback Control Tools - Pseudocode Design

## 1. Device Management Tools

### 1.1 List Devices Algorithm

```
ALGORITHM: ListDevices
INPUT:
    user_id (string) - User identifier from auth context
    include_offline (boolean) - Include offline devices (default: false)
OUTPUT:
    devices (array) - List of user's registered devices

DATA STRUCTURES:
    DeviceRegistry: Map of device metadata
        Type: Database table with indexes
        Key: device_id
        Fields: user_id, device_name, device_type, capabilities, last_seen

    DevicePresence: Real-time device status
        Type: In-memory cache (Redis)
        TTL: 5 minutes
        Fields: online, current_activity, ip_address

BEGIN
    // Step 1: Validate user
    IF user_id is null OR user_id is empty THEN
        THROW AuthorizationError("User not authenticated")
    END IF

    // Step 2: Query device registry
    query ← `
        SELECT
            device_id,
            device_name,
            device_type,
            manufacturer,
            model,
            capabilities,
            registered_at,
            last_seen
        FROM devices
        WHERE user_id = ?
        ORDER BY last_seen DESC
    `

    results ← Database.execute(query, [user_id])

    // Step 3: Enrich with real-time presence data
    devices ← []

    FOR EACH row IN results.rows DO
        device ← {
            device_id: row.device_id,
            name: row.device_name,
            type: row.device_type,
            manufacturer: row.manufacturer,
            model: row.model,
            capabilities: ParseCapabilities(row.capabilities),
            registered_at: row.registered_at,
            last_seen: row.last_seen
        }

        // Get real-time presence status
        presence ← GetDevicePresence(row.device_id)

        device.status ← {
            online: presence.online,
            current_activity: presence.current_activity,
            last_ping: presence.last_ping
        }

        // Filter offline devices if requested
        IF include_offline OR presence.online THEN
            devices.append(device)
        END IF
    END FOR

    // Step 4: Sort by online status (online first) then by last_seen
    devices.sort((a, b) => {
        IF a.status.online AND NOT b.status.online THEN
            RETURN -1
        ELSE IF b.status.online AND NOT a.status.online THEN
            RETURN 1
        ELSE
            RETURN b.last_seen - a.last_seen
        END IF
    })

    RETURN {
        devices: devices,
        total: devices.length
    }
END


SUBROUTINE: GetDevicePresence
INPUT: device_id (string)
OUTPUT: presence (object)

BEGIN
    // Check Redis cache for real-time status
    cacheKey ← "device:presence:" + device_id
    cachedPresence ← Redis.get(cacheKey)

    IF cachedPresence is not null THEN
        RETURN JSON_PARSE(cachedPresence)
    END IF

    // Default to offline if not in cache
    RETURN {
        online: false,
        current_activity: null,
        last_ping: null
    }
END


SUBROUTINE: ParseCapabilities
INPUT: capabilities (string or JSON)
OUTPUT: parsed (object)

BEGIN
    IF capabilities is null THEN
        RETURN GetDefaultCapabilities()
    END IF

    IF typeof capabilities EQUALS "string" THEN
        capabilities ← JSON_PARSE(capabilities)
    END IF

    RETURN {
        can_play_video: capabilities.video OR true,
        can_play_audio: capabilities.audio OR true,
        supports_4k: capabilities.supports_4k OR false,
        supports_hdr: capabilities.supports_hdr OR false,
        supported_formats: capabilities.formats OR ["mp4", "hls"],
        supports_casting: capabilities.casting OR false,
        has_remote_control: capabilities.remote OR true
    }
END
```

### 1.2 Get Device Status Algorithm

```
ALGORITHM: GetDeviceStatus
INPUT:
    device_id (string) - Device identifier
OUTPUT:
    status (object) - Detailed device status

BEGIN
    // Step 1: Validate device ownership
    device ← ValidateDeviceAccess(device_id, authContext.userId)

    IF device is null THEN
        THROW AuthorizationError("Device not found or access denied")
    END IF

    // Step 2: Get real-time presence
    presence ← GetDevicePresence(device_id)

    // Step 3: Get current playback state (if playing)
    playbackState ← null

    IF presence.online AND presence.current_activity EQUALS "playing" THEN
        playbackState ← GetPlaybackState(device_id)
    END IF

    // Step 4: Get network info
    networkInfo ← GetDeviceNetworkInfo(device_id)

    // Step 5: Assemble status
    status ← {
        device_id: device_id,
        name: device.device_name,
        online: presence.online,
        last_ping: presence.last_ping,
        activity: presence.current_activity,
        playback: playbackState,
        network: networkInfo,
        capabilities: device.capabilities
    }

    RETURN status
END


SUBROUTINE: ValidateDeviceAccess
INPUT: device_id (string), user_id (string)
OUTPUT: device (object) or null

BEGIN
    query ← "SELECT * FROM devices WHERE device_id = ? AND user_id = ?"
    result ← Database.execute(query, [device_id, user_id])

    IF result.rows.length EQUALS 0 THEN
        RETURN null
    END IF

    RETURN result.rows[0]
END


SUBROUTINE: GetPlaybackState
INPUT: device_id (string)
OUTPUT: state (object) or null

BEGIN
    // Check Redis for current playback session
    cacheKey ← "device:playback:" + device_id
    playbackData ← Redis.get(cacheKey)

    IF playbackData is null THEN
        RETURN null
    END IF

    state ← JSON_PARSE(playbackData)

    RETURN {
        content_id: state.content_id,
        content_title: state.content_title,
        playing: state.playing,
        position: state.position,
        duration: state.duration,
        volume: state.volume,
        started_at: state.started_at
    }
END


SUBROUTINE: GetDeviceNetworkInfo
INPUT: device_id (string)
OUTPUT: networkInfo (object)

BEGIN
    // Get cached network info
    cacheKey ← "device:network:" + device_id
    networkData ← Redis.get(cacheKey)

    IF networkData is null THEN
        RETURN {
            connection_type: "unknown",
            ip_address: null,
            bandwidth: null
        }
    END IF

    info ← JSON_PARSE(networkData)

    RETURN {
        connection_type: info.type,
        ip_address: info.ip,
        bandwidth: info.bandwidth,
        signal_strength: info.signal
    }
END
```

## 2. Playback Control Tools

### 2.1 Initiate Playback Algorithm

```
ALGORITHM: InitiatePlayback
INPUT:
    device_id (string) - Target device
    content_id (string) - Content to play
    start_position (integer) - Start time in seconds (default: 0)
    auto_play (boolean) - Auto-start playback (default: true)
OUTPUT:
    session (object) - Playback session details

DATA STRUCTURES:
    PlaybackSession: Active playback sessions
        Type: Redis cache with TTL
        TTL: 8 hours
        Fields: device_id, content_id, status, position

    CommandQueue: PubNub channel for device commands
        Type: Message queue (PubNub)
        Format: JSON commands with correlation IDs

BEGIN
    // Step 1: Validate device
    device ← ValidateDeviceAccess(device_id, authContext.userId)

    IF device is null THEN
        THROW AuthorizationError("Device not found or access denied")
    END IF

    // Step 2: Check device is online
    presence ← GetDevicePresence(device_id)

    IF NOT presence.online THEN
        THROW DeviceError("Device is offline")
    END IF

    // Step 3: Validate content access
    content ← ValidateContentAccess(content_id, authContext.userId)

    IF content is null THEN
        THROW AuthorizationError("Content not found or not available")
    END IF

    // Step 4: Get content availability and deep link
    region ← authContext.region OR "US"
    availability ← GetBestAvailability(content_id, region, device.capabilities)

    IF availability is null THEN
        THROW ContentError("Content not available in your region")
    END IF

    // Step 5: Generate deep link URL
    deepLink ← GenerateDeepLink(
        platform: availability.platform_id,
        content_id: content_id,
        start_position: start_position
    )

    // Step 6: Create playback session
    sessionId ← GenerateSessionId()

    session ← {
        session_id: sessionId,
        device_id: device_id,
        content_id: content_id,
        content_title: content.title,
        platform: availability.platform_name,
        status: "initiating",
        start_position: start_position,
        created_at: GetCurrentTime()
    }

    // Store session in Redis
    StorePlaybackSession(sessionId, session)

    // Step 7: Send playback command to device via PubNub
    command ← {
        command: "play",
        correlation_id: sessionId,
        content: {
            id: content_id,
            title: content.title,
            deep_link: deepLink,
            start_position: start_position,
            auto_play: auto_play
        },
        metadata: {
            session_id: sessionId,
            timestamp: GetCurrentTime()
        }
    }

    SendDeviceCommand(device_id, command)

    // Step 8: Wait for acknowledgment (with timeout)
    ack ← WaitForAcknowledgment(
        sessionId: sessionId,
        timeout: 10000  // 10 seconds
    )

    IF ack.success THEN
        session.status ← "playing"
        UpdatePlaybackSession(sessionId, session)
    ELSE
        session.status ← "failed"
        session.error ← ack.error OR "Device did not respond"
        UpdatePlaybackSession(sessionId, session)

        THROW PlaybackError("Failed to start playback: " + session.error)
    END IF

    // Step 9: Track analytics
    TrackPlaybackInitiation(
        user_id: authContext.userId,
        content_id: content_id,
        device_id: device_id,
        platform: availability.platform_name
    )

    RETURN session
END


SUBROUTINE: GetBestAvailability
INPUT: content_id (string), region (string), capabilities (object)
OUTPUT: availability (object) or null

BEGIN
    // Query available platforms
    query ← `
        SELECT
            platform_id,
            platform_name,
            availability_type,
            price,
            quality,
            deep_link_url
        FROM content_availability
        WHERE entity_id = ?
          AND region = ?
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY
            CASE availability_type
                WHEN 'subscription' THEN 1
                WHEN 'free' THEN 2
                WHEN 'rent' THEN 3
                WHEN 'buy' THEN 4
            END,
            price ASC
    `

    results ← Database.execute(query, [content_id, region])

    IF results.rows.length EQUALS 0 THEN
        RETURN null
    END IF

    // Filter by device capabilities
    FOR EACH row IN results.rows DO
        IF IsCompatibleWithDevice(row, capabilities) THEN
            RETURN row
        END IF
    END FOR

    // If no compatible options, return first (may fail on device)
    RETURN results.rows[0]
END


SUBROUTINE: GenerateDeepLink
INPUT: platform (string), content_id (string), start_position (integer)
OUTPUT: url (string)

BEGIN
    // Platform-specific deep link patterns
    deepLinkTemplates ← {
        "netflix": "https://www.netflix.com/watch/{content_id}?t={position}",
        "disney_plus": "https://www.disneyplus.com/video/{content_id}?t={position}",
        "hulu": "https://www.hulu.com/watch/{content_id}?t={position}",
        "amazon_prime": "https://www.amazon.com/gp/video/detail/{content_id}?t={position}",
        "hbo_max": "https://play.hbomax.com/player/urn:hbo:episode:{content_id}?t={position}"
    }

    template ← deepLinkTemplates[platform]

    IF template is null THEN
        // Fallback to generic URL
        RETURN "https://example.com/content/" + content_id
    END IF

    // Replace placeholders
    url ← template.replace("{content_id}", content_id)
    url ← url.replace("{position}", start_position.toString())

    RETURN url
END


SUBROUTINE: SendDeviceCommand
INPUT: device_id (string), command (object)
OUTPUT: void

BEGIN
    // Publish to device-specific PubNub channel
    channel ← "device." + device_id
    message ← JSON_STRINGIFY(command)

    TRY:
        PubNub.publish(
            channel: channel,
            message: message,
            storeInHistory: true,
            ttl: 60  // Message expires after 60 seconds
        )

        LOG("Command sent to device: " + device_id)

    CATCH error:
        LOG_ERROR("Failed to send command to device", error)
        THROW DeviceError("Failed to communicate with device")
    END TRY
END


SUBROUTINE: WaitForAcknowledgment
INPUT: session_id (string), timeout (integer)
OUTPUT: ack (object)

BEGIN
    // Subscribe to acknowledgment channel
    ackChannel ← "ack." + session_id
    startTime ← GetCurrentTime()

    // Set up timeout promise
    timeoutPromise ← CreateTimeout(timeout)

    // Wait for acknowledgment or timeout
    result ← AWAIT_RACE([
        WaitForPubNubMessage(ackChannel),
        timeoutPromise
    ])

    IF result.timeout THEN
        RETURN {
            success: false,
            error: "Timeout waiting for device response"
        }
    END IF

    // Parse acknowledgment
    ack ← result.message

    IF ack.status EQUALS "success" THEN
        RETURN {
            success: true,
            data: ack.data
        }
    ELSE
        RETURN {
            success: false,
            error: ack.error OR "Unknown error"
        }
    END IF
END
```

### 2.2 Control Playback Algorithm

```
ALGORITHM: ControlPlayback
INPUT:
    session_id (string) - Playback session ID
    action (string) - Control action (play, pause, seek, stop, volume)
    parameters (object) - Action-specific parameters
OUTPUT:
    result (object) - Control result

VALID_ACTIONS = ["play", "pause", "seek", "stop", "volume", "next", "previous"]

BEGIN
    // Step 1: Validate action
    IF action NOT IN VALID_ACTIONS THEN
        THROW ValidationError("Invalid action: " + action)
    END IF

    // Step 2: Get playback session
    session ← GetPlaybackSession(session_id)

    IF session is null THEN
        THROW NotFoundError("Playback session not found")
    END IF

    // Step 3: Validate session ownership
    device ← ValidateDeviceAccess(session.device_id, authContext.userId)

    IF device is null THEN
        THROW AuthorizationError("Access denied to playback session")
    END IF

    // Step 4: Check device is online
    presence ← GetDevicePresence(session.device_id)

    IF NOT presence.online THEN
        THROW DeviceError("Device is offline")
    END IF

    // Step 5: Validate action-specific parameters
    ValidateControlParameters(action, parameters)

    // Step 6: Create control command
    command ← {
        command: "control",
        correlation_id: GenerateCorrelationId(),
        session_id: session_id,
        action: action,
        parameters: parameters,
        timestamp: GetCurrentTime()
    }

    // Step 7: Send command to device
    SendDeviceCommand(session.device_id, command)

    // Step 8: Wait for acknowledgment
    ack ← WaitForAcknowledgment(
        sessionId: command.correlation_id,
        timeout: 5000
    )

    IF NOT ack.success THEN
        THROW PlaybackError("Control command failed: " + ack.error)
    END IF

    // Step 9: Update session state
    UpdateSessionState(session_id, action, parameters)

    // Step 10: Return result
    RETURN {
        session_id: session_id,
        action: action,
        status: "success",
        updated_at: GetCurrentTime()
    }
END


SUBROUTINE: ValidateControlParameters
INPUT: action (string), parameters (object)
OUTPUT: void (throws on invalid)

BEGIN
    IF action EQUALS "seek" THEN
        IF parameters.position is null THEN
            THROW ValidationError("Position required for seek action")
        END IF

        IF typeof parameters.position NOT EQUALS "number" THEN
            THROW ValidationError("Position must be a number")
        END IF

        IF parameters.position < 0 THEN
            THROW ValidationError("Position must be non-negative")
        END IF

    ELSE IF action EQUALS "volume" THEN
        IF parameters.level is null THEN
            THROW ValidationError("Level required for volume action")
        END IF

        IF typeof parameters.level NOT EQUALS "number" THEN
            THROW ValidationError("Volume level must be a number")
        END IF

        IF parameters.level < 0 OR parameters.level > 100 THEN
            THROW ValidationError("Volume level must be between 0 and 100")
        END IF
    END IF

    // Other actions (play, pause, stop) don't require parameters
END


SUBROUTINE: UpdateSessionState
INPUT: session_id (string), action (string), parameters (object)
OUTPUT: void

BEGIN
    session ← GetPlaybackSession(session_id)

    IF session is null THEN
        RETURN  // Session expired or not found
    END IF

    // Update based on action
    IF action EQUALS "play" THEN
        session.status ← "playing"

    ELSE IF action EQUALS "pause" THEN
        session.status ← "paused"

    ELSE IF action EQUALS "stop" THEN
        session.status ← "stopped"

    ELSE IF action EQUALS "seek" THEN
        session.position ← parameters.position

    ELSE IF action EQUALS "volume" THEN
        session.volume ← parameters.level
    END IF

    session.last_updated ← GetCurrentTime()

    // Update in Redis
    UpdatePlaybackSession(session_id, session)
END
```

## 3. Complexity Analysis

### Device Management

**Time Complexity:**
- ListDevices: O(d + d*p) where d = device count, p = presence lookup
- GetDeviceStatus: O(log n + p) where n = total devices
- **Typical: < 100ms for 10 devices**

**Space Complexity:**
- Device list: O(d * s) where s = average device size
- Presence cache: O(d * p) where p = presence data size
- **Typical: < 50KB for 10 devices**

### Playback Control

**Time Complexity:**
- InitiatePlayback: O(log n + a + c) where n = content, a = availability, c = command send
- WaitForAcknowledgment: O(t) where t = timeout (10s max)
- ControlPlayback: O(log s + c) where s = sessions
- **Typical: < 500ms including network**

**Space Complexity:**
- Session data: O(1) per session
- Command queue: O(q) where q = pending commands
- **Typical: < 5KB per active session**

### Optimization Strategies

1. **Device Presence Caching**
   - Redis with 5-minute TTL
   - Periodic heartbeat from devices
   - Push updates via PubNub

2. **Command Delivery**
   - Use PubNub for real-time delivery
   - Implement retry logic with exponential backoff
   - Queue commands for offline devices

3. **Session Management**
   - Auto-expire sessions after 8 hours
   - Cleanup stopped sessions after 1 hour
   - Compress session data in Redis

4. **Deep Link Generation**
   - Cache platform URL templates
   - Pre-generate links for popular content
   - Use CDN for link validation
