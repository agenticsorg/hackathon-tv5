# Remote Control Protocol Pseudocode

## Overview
Provides low-latency command routing, acknowledgment handling, and reliable delivery for cross-device remote control functionality.

## Data Structures

```
STRUCTURE RemoteCommand:
    command_id: string             // UUID
    source_device_id: string       // Controller device
    target_device_id: string       // Target device
    command_type: CommandType
    parameters: Map<string, any>
    timestamp: HLC
    timeout_ms: integer
    requires_ack: boolean

STRUCTURE CommandAcknowledgment:
    command_id: string
    target_device_id: string
    status: enum(RECEIVED, EXECUTING, COMPLETED, FAILED)
    result: any OR null
    error_message: string OR null
    timestamp: HLC

STRUCTURE CommandState:
    command: RemoteCommand
    status: enum(PENDING, SENT, ACKNOWLEDGED, COMPLETED, TIMEOUT, FAILED)
    sent_at: timestamp
    ack_received_at: timestamp OR null
    retry_count: integer
    timeout_timer_id: string OR null

ENUM CommandType:
    PLAY
    PAUSE
    SEEK
    STOP
    SET_VOLUME
    MUTE
    UNMUTE
    NEXT_EPISODE
    PREVIOUS_EPISODE
    CHANGE_QUALITY
    CHANGE_SUBTITLE
    CHANGE_AUDIO_TRACK
    FAST_FORWARD
    REWIND
    CAST_TO_DEVICE
    SCREENSHOT
    ADD_TO_WATCHLIST
    REMOVE_FROM_WATCHLIST

STRUCTURE RemoteControlManager:
    device_id: string
    user_id: string
    pending_commands: Map<command_id, CommandState>
    command_handlers: Map<CommandType, CommandHandler>
    hlc: HLC

CONSTANTS:
    DEFAULT_COMMAND_TIMEOUT = 5000     // 5 seconds
    MAX_RETRY_ATTEMPTS = 3
    RETRY_BACKOFF_MS = 500
    ACK_TIMEOUT_MS = 2000              // 2 seconds for initial ACK
```

## Core Algorithms

### 1. Initialize Remote Control Manager

```
ALGORITHM: InitializeRemoteControlManager
INPUT: user_id (string), device_id (string)
OUTPUT: manager (RemoteControlManager)

BEGIN
    manager ← RemoteControlManager()
    manager.device_id ← device_id
    manager.user_id ← user_id
    manager.pending_commands ← EmptyMap()
    manager.command_handlers ← EmptyMap()
    manager.hlc ← CreateHLC(GetWallClock())

    // Register default command handlers
    RegisterDefaultHandlers(manager)

    // Subscribe to command channel
    command_channel ← GetChannelName("devices", user_id, null)
    RegisterMessageHandler(client, command_channel, FUNCTION(message) DO
        HandleIncomingMessage(manager, message)
    END)

    LogInfo("Remote control manager initialized", device_id)

    RETURN manager
END
```

### 2. Send Remote Command

```
ALGORITHM: SendRemoteCommand
INPUT:
    manager (RemoteControlManager),
    target_device_id (string),
    command_type (CommandType),
    parameters (Map<string, any>),
    timeout_ms (integer) OR null
OUTPUT: command_id (string) OR error

BEGIN
    // Validate target device
    IF target_device_id == manager.device_id THEN
        RETURN error("Cannot send command to self")
    END IF

    // Validate target is online
    target_presence ← GetDevicePresence(target_device_id)
    IF target_presence is null OR target_presence.status == OFFLINE THEN
        RETURN error("Target device offline")
    END IF

    // Create command
    command ← RemoteCommand()
    command.command_id ← GenerateUUID()
    command.source_device_id ← manager.device_id
    command.target_device_id ← target_device_id
    command.command_type ← command_type
    command.parameters ← parameters
    command.timestamp ← IncrementHLC(manager.hlc, GetWallClock())
    command.timeout_ms ← timeout_ms OR DEFAULT_COMMAND_TIMEOUT
    command.requires_ack ← true

    // Create command state
    command_state ← CommandState()
    command_state.command ← command
    command_state.status ← PENDING
    command_state.retry_count ← 0

    // Store pending command
    manager.pending_commands.set(command.command_id, command_state)

    // Send command
    success ← TransmitCommand(manager, command_state)

    IF NOT success THEN
        command_state.status ← FAILED
        RETURN error("Failed to send command")
    END IF

    LogInfo("Remote command sent", command.command_id, command_type, target_device_id)

    RETURN command.command_id
END
```

### 3. Transmit Command

```
ALGORITHM: TransmitCommand
INPUT: manager (RemoteControlManager), command_state (CommandState)
OUTPUT: success (boolean)

BEGIN
    command ← command_state.command

    // Create message payload
    message_payload ← {
        command_id: command.command_id,
        source_device_id: command.source_device_id,
        command_type: command.command_type,
        parameters: command.parameters,
        requires_ack: command.requires_ack
    }

    // Determine channel
    channel ← GetChannelName("devices", manager.user_id, null)

    TRY
        // Publish command
        message_id ← PublishMessage(
            client,
            channel,
            "remote_command",
            message_payload,
            command.timestamp
        )

        // Update state
        command_state.status ← SENT
        command_state.sent_at ← GetCurrentTime()

        // Start timeout timer
        timeout_timer_id ← SetTimeout(
            FUNCTION() DO
                HandleCommandTimeout(manager, command.command_id)
            END,
            command.timeout_ms
        )

        command_state.timeout_timer_id ← timeout_timer_id

        LogDebug("Command transmitted", command.command_id, message_id)

        RETURN true

    CATCH error
        LogError("Command transmission failed", command.command_id, error.message)
        RETURN false
    END TRY
END
```

### 4. Handle Incoming Message

```
ALGORITHM: HandleIncomingMessage
INPUT: manager (RemoteControlManager), message (Message)
OUTPUT: none

BEGIN
    // Update HLC
    manager.hlc ← ReceiveHLC(manager.hlc, message.timestamp, GetWallClock())

    CASE message.message_type OF
        "remote_command":
            HandleRemoteCommand(manager, message)

        "command_ack":
            HandleCommandAck(manager, message)

        DEFAULT:
            LogDebug("Unknown message type", message.message_type)
    END CASE
END
```

### 5. Handle Remote Command (Receiver Side)

```
ALGORITHM: HandleRemoteCommand
INPUT: manager (RemoteControlManager), message (Message)
OUTPUT: none

BEGIN
    command_data ← message.payload

    // Check if command is for this device
    IF command_data.target_device_id != manager.device_id THEN
        RETURN  // Not for us
    END IF

    command_id ← command_data.command_id
    command_type ← command_data.command_type

    LogInfo("Remote command received", command_id, command_type, command_data.source_device_id)

    // Send immediate acknowledgment (RECEIVED)
    IF command_data.requires_ack THEN
        SendAcknowledgment(
            manager,
            command_id,
            command_data.source_device_id,
            RECEIVED,
            null,
            null
        )
    END IF

    // Check if handler exists
    IF NOT manager.command_handlers.has(command_type) THEN
        LogWarning("No handler for command type", command_type)

        SendAcknowledgment(
            manager,
            command_id,
            command_data.source_device_id,
            FAILED,
            null,
            "Command type not supported"
        )
        RETURN
    END IF

    // Send EXECUTING acknowledgment
    IF command_data.requires_ack THEN
        SendAcknowledgment(
            manager,
            command_id,
            command_data.source_device_id,
            EXECUTING,
            null,
            null
        )
    END IF

    // Execute command asynchronously
    handler ← manager.command_handlers.get(command_type)

    TRY
        result ← handler(command_data.parameters)

        // Send COMPLETED acknowledgment
        IF command_data.requires_ack THEN
            SendAcknowledgment(
                manager,
                command_id,
                command_data.source_device_id,
                COMPLETED,
                result,
                null
            )
        END IF

        LogInfo("Command executed successfully", command_id, command_type)

    CATCH error
        LogError("Command execution failed", command_id, error.message)

        // Send FAILED acknowledgment
        IF command_data.requires_ack THEN
            SendAcknowledgment(
                manager,
                command_id,
                command_data.source_device_id,
                FAILED,
                null,
                error.message
            )
        END IF
    END TRY
END
```

### 6. Send Acknowledgment

```
ALGORITHM: SendAcknowledgment
INPUT:
    manager (RemoteControlManager),
    command_id (string),
    target_device_id (string),
    status (AckStatus),
    result (any OR null),
    error_message (string OR null)
OUTPUT: success (boolean)

BEGIN
    // Create acknowledgment
    ack ← CommandAcknowledgment()
    ack.command_id ← command_id
    ack.target_device_id ← manager.device_id
    ack.status ← status
    ack.result ← result
    ack.error_message ← error_message
    ack.timestamp ← IncrementHLC(manager.hlc, GetWallClock())

    // Publish acknowledgment
    channel ← GetChannelName("devices", manager.user_id, null)

    TRY
        PublishMessage(
            client,
            channel,
            "command_ack",
            ack,
            ack.timestamp
        )

        LogDebug("Acknowledgment sent", command_id, status)
        RETURN true

    CATCH error
        LogError("Failed to send acknowledgment", command_id, error.message)
        RETURN false
    END TRY
END
```

### 7. Handle Command Acknowledgment (Sender Side)

```
ALGORITHM: HandleCommandAck
INPUT: manager (RemoteControlManager), message (Message)
OUTPUT: none

BEGIN
    ack ← message.payload
    command_id ← ack.command_id

    // Check if we have this pending command
    IF NOT manager.pending_commands.has(command_id) THEN
        LogDebug("Ack for unknown command", command_id)
        RETURN
    END IF

    command_state ← manager.pending_commands.get(command_id)

    LogInfo("Acknowledgment received", command_id, ack.status)

    CASE ack.status OF
        RECEIVED:
            command_state.status ← ACKNOWLEDGED
            command_state.ack_received_at ← GetCurrentTime()
            OnCommandAcknowledged(manager, command_state)

        EXECUTING:
            OnCommandExecuting(manager, command_state)

        COMPLETED:
            command_state.status ← COMPLETED

            // Cancel timeout timer
            IF command_state.timeout_timer_id is not null THEN
                ClearTimeout(command_state.timeout_timer_id)
            END IF

            // Trigger completion callback
            OnCommandCompleted(manager, command_state, ack.result)

            // Remove from pending
            manager.pending_commands.delete(command_id)

        FAILED:
            command_state.status ← FAILED

            // Cancel timeout timer
            IF command_state.timeout_timer_id is not null THEN
                ClearTimeout(command_state.timeout_timer_id)
            END IF

            // Trigger failure callback
            OnCommandFailed(manager, command_state, ack.error_message)

            // Remove from pending
            manager.pending_commands.delete(command_id)
    END CASE
END
```

### 8. Handle Command Timeout

```
ALGORITHM: HandleCommandTimeout
INPUT: manager (RemoteControlManager), command_id (string)
OUTPUT: none

BEGIN
    IF NOT manager.pending_commands.has(command_id) THEN
        RETURN  // Already processed
    END IF

    command_state ← manager.pending_commands.get(command_id)

    LogWarning("Command timeout", command_id, command_state.retry_count)

    // Check if we should retry
    IF command_state.retry_count < MAX_RETRY_ATTEMPTS THEN
        // Retry command
        command_state.retry_count ← command_state.retry_count + 1
        command_state.status ← PENDING

        // Exponential backoff
        backoff_delay ← RETRY_BACKOFF_MS * (2 ^ command_state.retry_count)

        LogInfo("Retrying command", command_id, command_state.retry_count)

        // Schedule retry
        SetTimeout(
            FUNCTION() DO
                TransmitCommand(manager, command_state)
            END,
            backoff_delay
        )

    ELSE
        // Max retries exceeded
        command_state.status ← TIMEOUT

        LogError("Command timeout - max retries exceeded", command_id)

        // Trigger timeout callback
        OnCommandTimeout(manager, command_state)

        // Remove from pending
        manager.pending_commands.delete(command_id)
    END IF
END
```

### 9. Command Handler Registration

```
ALGORITHM: RegisterCommandHandler
INPUT:
    manager (RemoteControlManager),
    command_type (CommandType),
    handler (CommandHandler)
OUTPUT: success (boolean)

BEGIN
    IF manager.command_handlers.has(command_type) THEN
        LogWarning("Overwriting existing handler", command_type)
    END IF

    manager.command_handlers.set(command_type, handler)

    LogInfo("Command handler registered", command_type)

    RETURN true
END

ALGORITHM: RegisterDefaultHandlers
INPUT: manager (RemoteControlManager)
OUTPUT: none

BEGIN
    // Play command
    RegisterCommandHandler(manager, PLAY, FUNCTION(params) DO
        media_id ← params.get("media_id")
        position ← params.get("position") OR 0
        StartPlayback(media_id, position)
        RETURN {status: "playing", media_id: media_id}
    END)

    // Pause command
    RegisterCommandHandler(manager, PAUSE, FUNCTION(params) DO
        current_position ← PausePlayback()
        RETURN {status: "paused", position: current_position}
    END)

    // Seek command
    RegisterCommandHandler(manager, SEEK, FUNCTION(params) DO
        position ← params.get("position")
        SeekToPosition(position)
        RETURN {status: "seeked", position: position}
    END)

    // Set volume command
    RegisterCommandHandler(manager, SET_VOLUME, FUNCTION(params) DO
        volume ← params.get("volume")
        SetVolume(volume)
        RETURN {status: "volume_set", volume: volume}
    END)

    // Add to watchlist command
    RegisterCommandHandler(manager, ADD_TO_WATCHLIST, FUNCTION(params) DO
        media_id ← params.get("media_id")
        AddToWatchlist(media_id)
        RETURN {status: "added", media_id: media_id}
    END)

    // Additional handlers...
END
```

### 10. Optimized Command Batching

```
ALGORITHM: SendCommandBatch
INPUT:
    manager (RemoteControlManager),
    target_device_id (string),
    commands (array of {command_type, parameters})
OUTPUT: command_ids (array of string)

BEGIN
    command_ids ← []

    // Create batch message
    batch ← {
        batch_id: GenerateUUID(),
        source_device_id: manager.device_id,
        target_device_id: target_device_id,
        commands: []
    }

    FOR EACH cmd IN commands DO
        command_id ← GenerateUUID()
        command_ids.append(command_id)

        batch.commands.append({
            command_id: command_id,
            command_type: cmd.command_type,
            parameters: cmd.parameters
        })
    END FOR

    // Send batch
    channel ← GetChannelName("devices", manager.user_id, null)
    timestamp ← IncrementHLC(manager.hlc, GetWallClock())

    PublishMessage(client, channel, "command_batch", batch, timestamp)

    LogInfo("Command batch sent", batch.batch_id, Length(commands))

    RETURN command_ids
END
```

### 11. Target Device Validation

```
ALGORITHM: ValidateTargetDevice
INPUT: manager (RemoteControlManager), target_device_id (string), command_type (CommandType)
OUTPUT: is_valid (boolean), error_message (string OR null)

BEGIN
    // Get target device presence
    target_presence ← GetDevicePresence(target_device_id)

    IF target_presence is null THEN
        RETURN false, "Device not found"
    END IF

    // Check if device is online
    time_since_heartbeat ← GetCurrentTime() - target_presence.last_heartbeat

    IF time_since_heartbeat > OFFLINE_THRESHOLD THEN
        RETURN false, "Device offline"
    END IF

    // Check if device supports command
    required_capability ← GetRequiredCapability(command_type)

    IF required_capability is not null THEN
        IF NOT target_presence.capabilities.contains(required_capability) THEN
            RETURN false, CONCAT("Device does not support ", command_type)
        END IF
    END IF

    // Check device state compatibility
    CASE command_type OF
        PAUSE, STOP, SEEK:
            IF target_presence.status != WATCHING THEN
                RETURN false, "Device not playing media"
            END IF

        PLAY:
            // Always valid if online

        DEFAULT:
            // No special validation
    END CASE

    RETURN true, null
END

ALGORITHM: GetRequiredCapability
INPUT: command_type (CommandType)
OUTPUT: capability (DeviceCapability OR null)

BEGIN
    CASE command_type OF
        PLAY, PAUSE, SEEK, STOP, SET_VOLUME:
            RETURN PLAYBACK

        CAST_TO_DEVICE:
            RETURN REMOTE_CONTROL

        DEFAULT:
            RETURN null
    END CASE
END
```

### 12. Command Status Tracking

```
ALGORITHM: GetCommandStatus
INPUT: manager (RemoteControlManager), command_id (string)
OUTPUT: status (CommandStatus OR null)

BEGIN
    IF manager.pending_commands.has(command_id) THEN
        command_state ← manager.pending_commands.get(command_id)
        RETURN command_state.status
    END IF

    RETURN null  // Command completed or doesn't exist
END

ALGORITHM: CancelCommand
INPUT: manager (RemoteControlManager), command_id (string)
OUTPUT: success (boolean)

BEGIN
    IF NOT manager.pending_commands.has(command_id) THEN
        RETURN false
    END IF

    command_state ← manager.pending_commands.get(command_id)

    // Cancel timeout timer
    IF command_state.timeout_timer_id is not null THEN
        ClearTimeout(command_state.timeout_timer_id)
    END IF

    // Remove from pending
    manager.pending_commands.delete(command_id)

    LogInfo("Command cancelled", command_id)

    RETURN true
END
```

## Complexity Analysis

### Time Complexity
- `SendRemoteCommand`: O(1) + network latency
- `HandleRemoteCommand`: O(1) + handler execution time
- `SendAcknowledgment`: O(1) + network latency
- `HandleCommandTimeout`: O(1)
- `ValidateTargetDevice`: O(1)
- `SendCommandBatch`: O(n) where n = number of commands

### Space Complexity
- RemoteControlManager: O(p + h) where p = pending commands, h = handlers
- Pending commands: O(p) typically <10
- Command message: ~200-500 bytes

## Performance Characteristics

- **Command Latency**: <50ms (P50), <100ms (P99)
- **ACK Latency**: <2 seconds
- **Timeout**: 5 seconds default
- **Max Retries**: 3 attempts
- **Backoff**: Exponential (500ms, 1s, 2s)
- **Network Overhead**: 200-500 bytes per command

## Edge Cases

1. **Target Offline**: Immediate error before sending
2. **ACK Timeout**: Retry with exponential backoff
3. **Execution Failure**: FAILED ACK with error message
4. **Concurrent Commands**: Independent processing
5. **Self-Command**: Rejected with error
6. **Unsupported Command**: FAILED ACK from target
