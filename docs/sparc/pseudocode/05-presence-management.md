# Presence Management Pseudocode

## Overview
Provides device presence tracking, heartbeat monitoring, state synchronization, and graceful disconnect handling for multi-device user sessions.

## Data Structures

```
STRUCTURE DevicePresence:
    device_id: string
    user_id: string
    device_type: enum(MOBILE, TV, WEB, TABLET, DESKTOP)
    device_name: string
    status: enum(IDLE, BROWSING, WATCHING, OFFLINE)
    last_heartbeat: timestamp
    last_activity: timestamp
    current_media_id: string OR null
    playback_position: float OR null
    network_quality: enum(EXCELLENT, GOOD, FAIR, POOR, UNKNOWN)
    battery_level: integer OR null      // 0-100, null for non-battery devices
    capabilities: Set<DeviceCapability>

STRUCTURE DeviceCapability:
    capability: enum(PLAYBACK, REMOTE_CONTROL, OFFLINE_DOWNLOAD, HDR, 4K, DOLBY_ATMOS)

STRUCTURE HeartbeatMessage:
    device_id: string
    timestamp: HLC
    status: DeviceStatus
    current_media_id: string OR null
    playback_position: float OR null
    network_quality: NetworkQuality
    battery_level: integer OR null

STRUCTURE PresenceState:
    devices: Map<device_id, DevicePresence>      // All user devices
    active_device_id: string OR null             // Currently active device
    last_sync: timestamp

CONSTANTS:
    HEARTBEAT_INTERVAL = 30000          // 30 seconds
    HEARTBEAT_TIMEOUT = 60000           // 60 seconds (2 missed heartbeats)
    IDLE_TIMEOUT = 300000               // 5 minutes
    OFFLINE_THRESHOLD = 90000           // 90 seconds
    MAX_DEVICES_PER_USER = 10
```

## Core Algorithms

### 1. Initialize Presence Manager

```
ALGORITHM: InitializePresenceManager
INPUT: user_id (string), device_id (string), device_info (DeviceInfo)
OUTPUT: presence_manager (PresenceManager)

BEGIN
    presence_manager ← PresenceManager()
    presence_manager.user_id ← user_id
    presence_manager.device_id ← device_id
    presence_manager.state ← PresenceState()
    presence_manager.state.devices ← EmptyMap()

    // Create presence for current device
    device_presence ← CreateDevicePresence(device_id, user_id, device_info)
    presence_manager.state.devices.set(device_id, device_presence)
    presence_manager.state.active_device_id ← device_id

    // Start heartbeat timer
    StartHeartbeatTimer(presence_manager)

    // Start timeout monitor
    StartTimeoutMonitor(presence_manager)

    LogInfo("Presence manager initialized", device_id)

    RETURN presence_manager
END

ALGORITHM: CreateDevicePresence
INPUT: device_id (string), user_id (string), device_info (DeviceInfo)
OUTPUT: presence (DevicePresence)

BEGIN
    presence ← DevicePresence()
    presence.device_id ← device_id
    presence.user_id ← user_id
    presence.device_type ← device_info.type
    presence.device_name ← device_info.name
    presence.status ← IDLE
    presence.last_heartbeat ← GetCurrentTime()
    presence.last_activity ← GetCurrentTime()
    presence.current_media_id ← null
    presence.playback_position ← null
    presence.network_quality ← UNKNOWN
    presence.battery_level ← device_info.battery_level
    presence.capabilities ← device_info.capabilities

    RETURN presence
END
```

### 2. Heartbeat Management

```
ALGORITHM: StartHeartbeatTimer
INPUT: presence_manager (PresenceManager)
OUTPUT: timer_id (string)

BEGIN
    timer_id ← SetInterval(
        FUNCTION() DO
            SendHeartbeat(presence_manager)
        END,
        HEARTBEAT_INTERVAL
    )

    presence_manager.heartbeat_timer_id ← timer_id

    LogInfo("Heartbeat timer started", HEARTBEAT_INTERVAL)

    RETURN timer_id
END

ALGORITHM: SendHeartbeat
INPUT: presence_manager (PresenceManager)
OUTPUT: success (boolean)

BEGIN
    device_id ← presence_manager.device_id
    local_presence ← presence_manager.state.devices.get(device_id)

    // Create heartbeat message
    heartbeat ← HeartbeatMessage()
    heartbeat.device_id ← device_id
    heartbeat.timestamp ← IncrementHLC(presence_manager.hlc, GetWallClock())
    heartbeat.status ← local_presence.status
    heartbeat.current_media_id ← local_presence.current_media_id
    heartbeat.playback_position ← local_presence.playback_position
    heartbeat.network_quality ← DetectNetworkQuality()
    heartbeat.battery_level ← GetBatteryLevel()

    // Update local state
    local_presence.last_heartbeat ← GetCurrentTime()
    local_presence.network_quality ← heartbeat.network_quality
    local_presence.battery_level ← heartbeat.battery_level

    // Publish to devices channel
    channel ← GetChannelName("devices", presence_manager.user_id, null)

    TRY
        PublishMessage(client, channel, "heartbeat", heartbeat, heartbeat.timestamp)
        LogDebug("Heartbeat sent", device_id)
        RETURN true

    CATCH error
        LogWarning("Heartbeat failed", error.message)
        RETURN false
    END TRY
END
```

### 3. Receive and Process Heartbeat

```
ALGORITHM: ProcessHeartbeat
INPUT: presence_manager (PresenceManager), heartbeat (HeartbeatMessage)
OUTPUT: updated_state (PresenceState)

BEGIN
    device_id ← heartbeat.device_id
    current_time ← GetCurrentTime()

    // Update HLC
    presence_manager.hlc ← ReceiveHLC(presence_manager.hlc, heartbeat.timestamp, GetWallClock())

    // Ignore own heartbeats
    IF device_id == presence_manager.device_id THEN
        RETURN presence_manager.state
    END IF

    // Get or create device presence
    IF presence_manager.state.devices.has(device_id) THEN
        device_presence ← presence_manager.state.devices.get(device_id)
    ELSE
        // New device detected
        device_presence ← DevicePresence()
        device_presence.device_id ← device_id
        device_presence.user_id ← presence_manager.user_id
        LogInfo("New device detected", device_id)
    END IF

    // Update presence information
    device_presence.status ← heartbeat.status
    device_presence.last_heartbeat ← current_time
    device_presence.current_media_id ← heartbeat.current_media_id
    device_presence.playback_position ← heartbeat.playback_position
    device_presence.network_quality ← heartbeat.network_quality
    device_presence.battery_level ← heartbeat.battery_level

    // Update last activity if device is active
    IF heartbeat.status IN [BROWSING, WATCHING] THEN
        device_presence.last_activity ← current_time
    END IF

    // Store updated presence
    presence_manager.state.devices.set(device_id, device_presence)

    // Trigger presence change event
    TriggerPresenceChangeEvent(device_id, device_presence)

    RETURN presence_manager.state
END
```

### 4. Timeout Detection

```
ALGORITHM: StartTimeoutMonitor
INPUT: presence_manager (PresenceManager)
OUTPUT: timer_id (string)

BEGIN
    // Check for timeouts every 10 seconds
    timer_id ← SetInterval(
        FUNCTION() DO
            CheckDeviceTimeouts(presence_manager)
        END,
        10000
    )

    presence_manager.timeout_monitor_id ← timer_id

    LogInfo("Timeout monitor started")

    RETURN timer_id
END

ALGORITHM: CheckDeviceTimeouts
INPUT: presence_manager (PresenceManager)
OUTPUT: timed_out_devices (array of string)

BEGIN
    current_time ← GetCurrentTime()
    timed_out_devices ← []

    FOR EACH (device_id, device_presence) IN presence_manager.state.devices DO
        // Skip current device
        IF device_id == presence_manager.device_id THEN
            CONTINUE
        END IF

        time_since_heartbeat ← current_time - device_presence.last_heartbeat

        // Check for offline timeout
        IF time_since_heartbeat > OFFLINE_THRESHOLD THEN
            IF device_presence.status != OFFLINE THEN
                LogInfo("Device timeout detected", device_id, time_since_heartbeat)

                // Mark as offline
                device_presence.status ← OFFLINE
                device_presence.current_media_id ← null
                device_presence.playback_position ← null

                timed_out_devices.append(device_id)

                // Trigger timeout event
                TriggerDeviceTimeoutEvent(device_id, device_presence)
            END IF
        END IF

        // Check for idle timeout
        time_since_activity ← current_time - device_presence.last_activity

        IF time_since_activity > IDLE_TIMEOUT THEN
            IF device_presence.status == BROWSING OR device_presence.status == WATCHING THEN
                LogDebug("Device idle timeout", device_id)
                device_presence.status ← IDLE
            END IF
        END IF
    END FOR

    // Cleanup old offline devices (>24 hours)
    CleanupOfflineDevices(presence_manager, 86400000)

    RETURN timed_out_devices
END

ALGORITHM: CleanupOfflineDevices
INPUT: presence_manager (PresenceManager), retention_period (milliseconds)
OUTPUT: removed_count (integer)

BEGIN
    current_time ← GetCurrentTime()
    removed_count ← 0
    devices_to_remove ← []

    FOR EACH (device_id, device_presence) IN presence_manager.state.devices DO
        // Skip current device
        IF device_id == presence_manager.device_id THEN
            CONTINUE
        END IF

        IF device_presence.status == OFFLINE THEN
            time_offline ← current_time - device_presence.last_heartbeat

            IF time_offline > retention_period THEN
                devices_to_remove.append(device_id)
            END IF
        END IF
    END FOR

    // Remove old devices
    FOR EACH device_id IN devices_to_remove DO
        presence_manager.state.devices.delete(device_id)
        removed_count ← removed_count + 1
        LogInfo("Removed old offline device", device_id)
    END FOR

    RETURN removed_count
END
```

### 5. Status Updates

```
ALGORITHM: UpdateDeviceStatus
INPUT:
    presence_manager (PresenceManager),
    new_status (DeviceStatus),
    media_id (string OR null),
    position (float OR null)
OUTPUT: success (boolean)

BEGIN
    device_id ← presence_manager.device_id
    device_presence ← presence_manager.state.devices.get(device_id)

    old_status ← device_presence.status

    // Update status
    device_presence.status ← new_status
    device_presence.last_activity ← GetCurrentTime()

    // Update media information if provided
    IF media_id is not null THEN
        device_presence.current_media_id ← media_id
    END IF

    IF position is not null THEN
        device_presence.playback_position ← position
    END IF

    // Clear media info when not watching
    IF new_status != WATCHING THEN
        device_presence.current_media_id ← null
        device_presence.playback_position ← null
    END IF

    LogInfo("Device status updated", old_status, new_status)

    // Send immediate heartbeat with new status
    SendHeartbeat(presence_manager)

    RETURN true
END
```

### 6. Active Device Management

```
ALGORITHM: GetActiveDevices
INPUT: presence_manager (PresenceManager)
OUTPUT: active_devices (array of DevicePresence)

BEGIN
    active_devices ← []
    current_time ← GetCurrentTime()

    FOR EACH (device_id, device_presence) IN presence_manager.state.devices DO
        time_since_heartbeat ← current_time - device_presence.last_heartbeat

        // Consider active if heartbeat within timeout period
        IF time_since_heartbeat < OFFLINE_THRESHOLD THEN
            active_devices.append(device_presence)
        END IF
    END FOR

    // Sort by last activity (most recent first)
    SortByLastActivity(active_devices)

    RETURN active_devices
END

ALGORITHM: GetWatchingDevice
INPUT: presence_manager (PresenceManager)
OUTPUT: device_presence (DevicePresence OR null)

BEGIN
    FOR EACH (device_id, device_presence) IN presence_manager.state.devices DO
        IF device_presence.status == WATCHING THEN
            time_since_heartbeat ← GetCurrentTime() - device_presence.last_heartbeat

            // Ensure device is still active
            IF time_since_heartbeat < OFFLINE_THRESHOLD THEN
                RETURN device_presence
            END IF
        END IF
    END FOR

    RETURN null
END

ALGORITHM: SetActiveDevice
INPUT: presence_manager (PresenceManager), device_id (string)
OUTPUT: success (boolean)

BEGIN
    // Validate device exists and is active
    IF NOT presence_manager.state.devices.has(device_id) THEN
        RETURN false
    END IF

    device_presence ← presence_manager.state.devices.get(device_id)

    time_since_heartbeat ← GetCurrentTime() - device_presence.last_heartbeat

    IF time_since_heartbeat > OFFLINE_THRESHOLD THEN
        LogWarning("Cannot set offline device as active", device_id)
        RETURN false
    END IF

    // Update active device
    old_active_id ← presence_manager.state.active_device_id
    presence_manager.state.active_device_id ← device_id

    LogInfo("Active device changed", old_active_id, device_id)

    // Trigger event
    TriggerActiveDeviceChangeEvent(device_id, device_presence)

    RETURN true
END
```

### 7. Graceful Disconnect

```
ALGORITHM: GracefulDisconnect
INPUT: presence_manager (PresenceManager)
OUTPUT: success (boolean)

BEGIN
    device_id ← presence_manager.device_id
    device_presence ← presence_manager.state.devices.get(device_id)

    // Update status to offline
    device_presence.status ← OFFLINE
    device_presence.last_heartbeat ← GetCurrentTime()
    device_presence.current_media_id ← null
    device_presence.playback_position ← null

    // Send final heartbeat
    heartbeat ← HeartbeatMessage()
    heartbeat.device_id ← device_id
    heartbeat.timestamp ← IncrementHLC(presence_manager.hlc, GetWallClock())
    heartbeat.status ← OFFLINE

    channel ← GetChannelName("devices", presence_manager.user_id, null)

    TRY
        PublishMessage(client, channel, "disconnect", heartbeat, heartbeat.timestamp)

    CATCH error
        LogWarning("Disconnect message failed", error.message)
    END TRY

    // Stop timers
    ClearInterval(presence_manager.heartbeat_timer_id)
    ClearInterval(presence_manager.timeout_monitor_id)

    LogInfo("Graceful disconnect completed", device_id)

    RETURN true
END
```

### 8. Device Capability Detection

```
ALGORITHM: DetectDeviceCapabilities
INPUT: device_info (DeviceInfo)
OUTPUT: capabilities (Set<DeviceCapability>)

BEGIN
    capabilities ← EmptySet()

    // Playback capability (all devices)
    capabilities.add(PLAYBACK)

    // Remote control (non-mobile devices typically controllers)
    IF device_info.type IN [TV, WEB, DESKTOP] THEN
        capabilities.add(REMOTE_CONTROL)
    END IF

    // Offline download (mobile and tablet)
    IF device_info.type IN [MOBILE, TABLET] THEN
        capabilities.add(OFFLINE_DOWNLOAD)
    END IF

    // HDR support detection
    IF device_info.supports_hdr THEN
        capabilities.add(HDR)
    END IF

    // 4K support detection
    IF device_info.max_resolution >= 2160 THEN
        capabilities.add(4K)
    END IF

    // Dolby Atmos detection
    IF device_info.audio_codecs.contains("dolby-atmos") THEN
        capabilities.add(DOLBY_ATMOS)
    END IF

    RETURN capabilities
END
```

### 9. Network Quality Detection

```
ALGORITHM: DetectNetworkQuality
INPUT: none
OUTPUT: quality (NetworkQuality)

BEGIN
    // Measure network metrics
    TRY
        // Use Network Information API or custom measurement
        rtt ← MeasureRoundTripTime()
        bandwidth ← EstimateBandwidth()
        packet_loss ← MeasurePacketLoss()

        // Classify quality
        IF rtt < 50 AND bandwidth > 25000000 AND packet_loss < 0.01 THEN
            RETURN EXCELLENT

        ELSE IF rtt < 100 AND bandwidth > 10000000 AND packet_loss < 0.05 THEN
            RETURN GOOD

        ELSE IF rtt < 200 AND bandwidth > 5000000 AND packet_loss < 0.1 THEN
            RETURN FAIR

        ELSE
            RETURN POOR
        END IF

    CATCH error
        LogWarning("Network quality detection failed", error.message)
        RETURN UNKNOWN
    END TRY
END
```

### 10. Presence State Synchronization

```
ALGORITHM: SyncPresenceState
INPUT:
    presence_manager (PresenceManager),
    remote_state (PresenceState)
OUTPUT: merged_state (PresenceState)

BEGIN
    // Merge device presences
    FOR EACH (device_id, remote_presence) IN remote_state.devices DO
        IF device_id == presence_manager.device_id THEN
            CONTINUE  // Skip own device
        END IF

        IF presence_manager.state.devices.has(device_id) THEN
            // Merge existing device
            local_presence ← presence_manager.state.devices.get(device_id)

            // Use most recent heartbeat
            IF remote_presence.last_heartbeat > local_presence.last_heartbeat THEN
                presence_manager.state.devices.set(device_id, remote_presence)
                LogDebug("Updated device presence", device_id)
            END IF
        ELSE
            // Add new device
            presence_manager.state.devices.set(device_id, remote_presence)
            LogInfo("Added new device from sync", device_id)
        END IF
    END FOR

    // Update sync timestamp
    presence_manager.state.last_sync ← GetCurrentTime()

    RETURN presence_manager.state
END
```

## Complexity Analysis

### Time Complexity
- `SendHeartbeat`: O(1)
- `ProcessHeartbeat`: O(1)
- `CheckDeviceTimeouts`: O(d) where d = number of devices
- `GetActiveDevices`: O(d)
- `UpdateDeviceStatus`: O(1)
- `GracefulDisconnect`: O(1)
- `SyncPresenceState`: O(d)

### Space Complexity
- PresenceState: O(d) where d = number of devices (max 10)
- DevicePresence: O(1)
- HeartbeatMessage: O(1)

## Performance Characteristics

- **Heartbeat Interval**: 30 seconds
- **Detection Latency**: <60 seconds for offline detection
- **Network Overhead**: ~200 bytes per heartbeat
- **Memory**: ~500 bytes per device
- **Max Devices**: 10 per user

## Edge Cases

1. **Network Interruption**: Heartbeat timeout triggers offline status
2. **Battery Death**: No graceful disconnect, timeout-based detection
3. **Clock Skew**: HLC provides consistent ordering
4. **Rapid Reconnect**: Deduplication prevents duplicate entries
5. **Device Limit**: Oldest offline device removed when limit reached
