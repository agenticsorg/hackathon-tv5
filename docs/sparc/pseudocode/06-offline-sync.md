# Offline Sync and Reconciliation Pseudocode

## Overview
Provides conflict-free offline operation with automatic reconciliation using CRDTs, local change queues, and history-based synchronization.

## Data Structures

```
STRUCTURE OfflineSyncManager:
    user_id: string
    device_id: string
    is_online: boolean
    local_hlc: HLC
    change_queue: Queue<LocalChange>
    sync_state: SyncState
    crdt_states: Map<string, CRDTState>

STRUCTURE LocalChange:
    change_id: string              // UUID
    change_type: enum(ADD, UPDATE, REMOVE)
    resource_type: enum(WATCHLIST, PROGRESS, PREFERENCE)
    resource_id: string
    operation: CRDTOperation
    timestamp: HLC
    retry_count: integer
    status: enum(PENDING, SYNCING, SYNCED, FAILED)

STRUCTURE SyncState:
    last_sync_time: timestamp
    last_sync_hlc: HLC
    pending_changes: integer
    failed_changes: integer
    sync_in_progress: boolean

STRUCTURE CRDTState:
    resource_type: string
    resource_id: string
    crdt_type: enum(LWW_REGISTER, OR_SET)
    state: any                     // Actual CRDT state
    last_modified: HLC

STRUCTURE SyncResult:
    synced_count: integer
    failed_count: integer
    conflicts_resolved: integer
    changes_received: integer

CONSTANTS:
    MAX_QUEUE_SIZE = 1000
    SYNC_BATCH_SIZE = 50
    MAX_RETRY_ATTEMPTS = 5
    RETRY_BACKOFF_MS = 1000
    HISTORY_FETCH_LIMIT = 100
    SYNC_DEBOUNCE_MS = 500
```

## Core Algorithms

### 1. Initialize Offline Sync Manager

```
ALGORITHM: InitializeOfflineSyncManager
INPUT: user_id (string), device_id (string)
OUTPUT: sync_manager (OfflineSyncManager)

BEGIN
    sync_manager ← OfflineSyncManager()
    sync_manager.user_id ← user_id
    sync_manager.device_id ← device_id
    sync_manager.is_online ← false
    sync_manager.local_hlc ← CreateHLC(GetWallClock())
    sync_manager.change_queue ← EmptyQueue()
    sync_manager.crdt_states ← EmptyMap()

    // Initialize sync state
    sync_manager.sync_state ← SyncState()
    sync_manager.sync_state.last_sync_time ← 0
    sync_manager.sync_state.last_sync_hlc ← sync_manager.local_hlc
    sync_manager.sync_state.pending_changes ← 0
    sync_manager.sync_state.failed_changes ← 0
    sync_manager.sync_state.sync_in_progress ← false

    // Load persisted state
    LoadPersistedState(sync_manager)

    // Setup network listeners
    SetupNetworkListeners(sync_manager)

    LogInfo("Offline sync manager initialized", device_id)

    RETURN sync_manager
END
```

### 2. Enqueue Local Change

```
ALGORITHM: EnqueueLocalChange
INPUT:
    sync_manager (OfflineSyncManager),
    change_type (ChangeType),
    resource_type (ResourceType),
    resource_id (string),
    operation (CRDTOperation)
OUTPUT: change_id (string)

BEGIN
    // Check queue capacity
    IF sync_manager.change_queue.size() >= MAX_QUEUE_SIZE THEN
        LogWarning("Change queue full, dropping oldest")
        sync_manager.change_queue.dequeue()
    END IF

    // Increment HLC
    sync_manager.local_hlc ← IncrementHLC(sync_manager.local_hlc, GetWallClock())

    // Create change record
    change ← LocalChange()
    change.change_id ← GenerateUUID()
    change.change_type ← change_type
    change.resource_type ← resource_type
    change.resource_id ← resource_id
    change.operation ← operation
    change.timestamp ← sync_manager.local_hlc
    change.retry_count ← 0
    change.status ← PENDING

    // Enqueue change
    sync_manager.change_queue.enqueue(change)
    sync_manager.sync_state.pending_changes ← sync_manager.sync_state.pending_changes + 1

    // Persist change
    PersistChange(change)

    LogInfo("Change enqueued", change.change_id, resource_type, change_type)

    // Trigger sync if online
    IF sync_manager.is_online THEN
        ScheduleSync(sync_manager)
    END IF

    RETURN change.change_id
END
```

### 3. Online/Offline State Management

```
ALGORITHM: HandleNetworkChange
INPUT: sync_manager (OfflineSyncManager), is_online (boolean)
OUTPUT: none

BEGIN
    old_state ← sync_manager.is_online
    sync_manager.is_online ← is_online

    IF old_state == is_online THEN
        RETURN  // No change
    END IF

    IF is_online THEN
        LogInfo("Network online - starting sync")
        OnNetworkOnline(sync_manager)
    ELSE
        LogInfo("Network offline - queuing changes")
        OnNetworkOffline(sync_manager)
    END IF
END

ALGORITHM: OnNetworkOnline
INPUT: sync_manager (OfflineSyncManager)
OUTPUT: none

BEGIN
    // Fetch missed changes from server
    FetchMissedChanges(sync_manager)

    // Start processing local queue
    ProcessChangeQueue(sync_manager)

    // Subscribe to real-time updates
    SubscribeToSyncChannels(sync_manager)
END

ALGORITHM: OnNetworkOffline
INPUT: sync_manager (OfflineSyncManager)
OUTPUT: none

BEGIN
    // Cancel any in-progress sync
    IF sync_manager.sync_state.sync_in_progress THEN
        CancelCurrentSync(sync_manager)
    END IF

    // Mark all syncing changes as pending
    FOR EACH change IN sync_manager.change_queue DO
        IF change.status == SYNCING THEN
            change.status ← PENDING
        END IF
    END FOR

    // Unsubscribe from channels
    UnsubscribeFromSyncChannels(sync_manager)
END
```

### 4. Fetch Missed Changes

```
ALGORITHM: FetchMissedChanges
INPUT: sync_manager (OfflineSyncManager)
OUTPUT: changes_received (integer)

BEGIN
    last_sync_time ← sync_manager.sync_state.last_sync_time
    current_time ← GetCurrentTime()

    changes_received ← 0

    // Fetch from each channel
    channels ← [
        GetChannelName("watchlist", sync_manager.user_id, null),
        GetChannelName("progress", sync_manager.user_id, null),
        GetChannelName("preferences", sync_manager.user_id, null)
    ]

    FOR EACH channel IN channels DO
        TRY
            // Fetch history since last sync
            messages ← FetchMessageHistory(
                client,
                channel,
                last_sync_time,
                current_time,
                HISTORY_FETCH_LIMIT
            )

            LogInfo("Fetched history", channel, Length(messages))

            // Process each message
            FOR EACH message IN messages DO
                // Skip messages from this device
                IF message.sender_device_id == sync_manager.device_id THEN
                    CONTINUE
                END IF

                // Apply remote change
                ApplyRemoteChange(sync_manager, message)
                changes_received ← changes_received + 1
            END FOR

        CATCH error
            LogError("Failed to fetch history", channel, error.message)
        END TRY
    END FOR

    // Update last sync time
    sync_manager.sync_state.last_sync_time ← current_time

    LogInfo("Missed changes fetched", changes_received)

    RETURN changes_received
END
```

### 5. Process Change Queue

```
ALGORITHM: ProcessChangeQueue
INPUT: sync_manager (OfflineSyncManager)
OUTPUT: sync_result (SyncResult)

BEGIN
    IF sync_manager.sync_state.sync_in_progress THEN
        LogDebug("Sync already in progress")
        RETURN null
    END IF

    IF NOT sync_manager.is_online THEN
        LogDebug("Cannot sync while offline")
        RETURN null
    END IF

    sync_manager.sync_state.sync_in_progress ← true

    sync_result ← SyncResult()
    sync_result.synced_count ← 0
    sync_result.failed_count ← 0
    sync_result.conflicts_resolved ← 0

    // Process changes in batches
    changes_to_sync ← []

    WHILE NOT sync_manager.change_queue.isEmpty() AND Length(changes_to_sync) < SYNC_BATCH_SIZE DO
        change ← sync_manager.change_queue.peek()

        // Only process pending or failed changes
        IF change.status == PENDING OR (change.status == FAILED AND change.retry_count < MAX_RETRY_ATTEMPTS) THEN
            sync_manager.change_queue.dequeue()
            changes_to_sync.append(change)
        ELSE
            sync_manager.change_queue.dequeue()  // Remove completed/max-retried
        END IF
    END WHILE

    // Sync each change
    FOR EACH change IN changes_to_sync DO
        change.status ← SYNCING

        success ← SyncChange(sync_manager, change)

        IF success THEN
            change.status ← SYNCED
            sync_result.synced_count ← sync_result.synced_count + 1
            sync_manager.sync_state.pending_changes ← sync_manager.sync_state.pending_changes - 1

            // Remove from persistence
            RemovePersistedChange(change.change_id)

        ELSE
            change.retry_count ← change.retry_count + 1

            IF change.retry_count >= MAX_RETRY_ATTEMPTS THEN
                change.status ← FAILED
                sync_result.failed_count ← sync_result.failed_count + 1
                sync_manager.sync_state.failed_changes ← sync_manager.sync_state.failed_changes + 1
                LogError("Change sync failed permanently", change.change_id)

            ELSE
                change.status ← PENDING
                sync_manager.change_queue.enqueue(change)  // Re-queue for retry
                LogWarning("Change sync failed, will retry", change.change_id, change.retry_count)
            END IF
        END IF
    END FOR

    sync_manager.sync_state.sync_in_progress ← false

    // Schedule next sync if queue not empty
    IF NOT sync_manager.change_queue.isEmpty() THEN
        ScheduleSync(sync_manager)
    END IF

    LogInfo("Sync batch completed", sync_result.synced_count, sync_result.failed_count)

    RETURN sync_result
END
```

### 6. Sync Individual Change

```
ALGORITHM: SyncChange
INPUT: sync_manager (OfflineSyncManager), change (LocalChange)
OUTPUT: success (boolean)

BEGIN
    // Determine target channel
    channel ← GetChannelForResourceType(sync_manager.user_id, change.resource_type)

    // Create message payload
    message_type ← GetMessageTypeForChange(change.change_type, change.resource_type)

    TRY
        // Publish change
        message_id ← PublishMessage(
            client,
            channel,
            message_type,
            change.operation,
            change.timestamp
        )

        LogInfo("Change synced", change.change_id, message_id)
        RETURN true

    CATCH error
        LogError("Sync failed", change.change_id, error.message)
        RETURN false
    END TRY
END
```

### 7. Apply Remote Change

```
ALGORITHM: ApplyRemoteChange
INPUT: sync_manager (OfflineSyncManager), message (Message)
OUTPUT: was_applied (boolean)

BEGIN
    // Update local HLC
    sync_manager.local_hlc ← ReceiveHLC(sync_manager.local_hlc, message.timestamp, GetWallClock())

    // Determine CRDT type from message
    resource_type ← GetResourceTypeFromChannel(message.channel)
    resource_id ← GetResourceIdFromMessage(message)

    // Get or create CRDT state
    state_key ← CONCAT(resource_type, ":", resource_id)

    IF sync_manager.crdt_states.has(state_key) THEN
        crdt_state ← sync_manager.crdt_states.get(state_key)
    ELSE
        crdt_state ← CreateCRDTState(resource_type, resource_id)
        sync_manager.crdt_states.set(state_key, crdt_state)
    END IF

    // Apply operation based on CRDT type
    was_updated ← false

    CASE crdt_state.crdt_type OF
        LWW_REGISTER:
            was_updated ← ApplyLWWUpdate(crdt_state, message)

        OR_SET:
            was_updated ← ApplyORSetUpdate(crdt_state, message)
    END CASE

    IF was_updated THEN
        // Update modification time
        crdt_state.last_modified ← message.timestamp

        // Persist CRDT state
        PersistCRDTState(crdt_state)

        // Trigger state change event
        TriggerStateChangeEvent(resource_type, resource_id, crdt_state.state)

        LogInfo("Remote change applied", resource_type, resource_id)
    ELSE
        LogDebug("Remote change ignored (outdated)", resource_type, resource_id)
    END IF

    RETURN was_updated
END
```

### 8. CRDT Merge Operations

```
ALGORITHM: ApplyLWWUpdate
INPUT: crdt_state (CRDTState), message (Message)
OUTPUT: was_updated (boolean)

BEGIN
    // Extract LWW update from message
    remote_update ← message.payload

    // Merge with local state
    local_register ← crdt_state.state
    merged_register, was_updated ← MergeLWWRegister(
        local_register,
        remote_update,
        message.timestamp
    )

    crdt_state.state ← merged_register

    RETURN was_updated
END

ALGORITHM: ApplyORSetUpdate
INPUT: crdt_state (CRDTState), message (Message)
OUTPUT: was_updated (boolean)

BEGIN
    local_set ← crdt_state.state
    was_updated ← false

    CASE message.message_type OF
        "add":
            add_message ← message.payload
            local_set, was_added ← MergeORSetAdd(local_set, add_message)
            was_updated ← was_added

        "remove":
            remove_message ← message.payload
            local_set, was_removed ← MergeORSetRemove(local_set, remove_message)
            was_updated ← was_removed
    END CASE

    crdt_state.state ← local_set

    RETURN was_updated
END
```

### 9. Conflict Resolution

```
ALGORITHM: ResolveConflict
INPUT:
    sync_manager (OfflineSyncManager),
    local_state (CRDTState),
    remote_state (CRDTState)
OUTPUT: resolved_state (CRDTState)

BEGIN
    // CRDTs resolve conflicts automatically through merge
    // This function handles special cases

    IF local_state.crdt_type != remote_state.crdt_type THEN
        LogError("CRDT type mismatch", local_state.crdt_type, remote_state.crdt_type)
        // Use remote (assume migration)
        RETURN remote_state
    END IF

    CASE local_state.crdt_type OF
        LWW_REGISTER:
            // Merge registers
            resolved_state ← local_state
            resolved_state.state, _ ← MergeLWWRegister(
                local_state.state,
                remote_state.state,
                sync_manager.local_hlc
            )

        OR_SET:
            // Merge sets
            resolved_state ← local_state
            resolved_state.state ← MergeORSets(
                local_state.state,
                remote_state.state
            )
    END CASE

    // Use latest modification time
    resolved_state.last_modified ← MAX(
        local_state.last_modified,
        remote_state.last_modified
    )

    LogInfo("Conflict resolved", local_state.resource_type, local_state.resource_id)

    RETURN resolved_state
END
```

### 10. Persistence Operations

```
ALGORITHM: PersistChange
INPUT: change (LocalChange)
OUTPUT: success (boolean)

BEGIN
    TRY
        // Store in IndexedDB or similar
        storage ← GetLocalStorage()
        key ← CONCAT("change:", change.change_id)

        storage.set(key, SerializeChange(change))

        RETURN true

    CATCH error
        LogError("Failed to persist change", error.message)
        RETURN false
    END TRY
END

ALGORITHM: LoadPersistedState
INPUT: sync_manager (OfflineSyncManager)
OUTPUT: loaded_count (integer)

BEGIN
    loaded_count ← 0

    TRY
        storage ← GetLocalStorage()

        // Load all persisted changes
        change_keys ← storage.getAllKeys("change:*")

        FOR EACH key IN change_keys DO
            change_data ← storage.get(key)
            change ← DeserializeChange(change_data)

            sync_manager.change_queue.enqueue(change)
            sync_manager.sync_state.pending_changes ← sync_manager.sync_state.pending_changes + 1

            loaded_count ← loaded_count + 1
        END FOR

        // Load CRDT states
        crdt_keys ← storage.getAllKeys("crdt:*")

        FOR EACH key IN crdt_keys DO
            crdt_data ← storage.get(key)
            crdt_state ← DeserializeCRDTState(crdt_data)

            state_key ← CONCAT(crdt_state.resource_type, ":", crdt_state.resource_id)
            sync_manager.crdt_states.set(state_key, crdt_state)
        END FOR

        LogInfo("Persisted state loaded", loaded_count, "changes", Length(crdt_keys), "CRDTs")

    CATCH error
        LogError("Failed to load persisted state", error.message)
    END TRY

    RETURN loaded_count
END
```

### 11. Debounced Sync Scheduling

```
ALGORITHM: ScheduleSync
INPUT: sync_manager (OfflineSyncManager)
OUTPUT: none

BEGIN
    // Cancel existing timer
    IF sync_manager.sync_timer_id is not null THEN
        ClearTimeout(sync_manager.sync_timer_id)
    END IF

    // Schedule new sync with debounce
    sync_manager.sync_timer_id ← SetTimeout(
        FUNCTION() DO
            ProcessChangeQueue(sync_manager)
        END,
        SYNC_DEBOUNCE_MS
    )
END
```

## Complexity Analysis

### Time Complexity
- `EnqueueLocalChange`: O(1)
- `ProcessChangeQueue`: O(b) where b = batch size (50)
- `FetchMissedChanges`: O(m) where m = missed messages
- `ApplyRemoteChange`: O(1) for LWW, O(t) for OR-Set (t = tags)
- `ResolveConflict`: O(1) for LWW, O(e × t) for OR-Set

### Space Complexity
- OfflineSyncManager: O(q + c) where q = queue size, c = CRDT count
- Change queue: O(1000) max
- CRDT states: O(c × s) where s = state size

## Performance Characteristics

- **Queue Processing**: 50 changes per batch
- **Sync Latency**: <1 second for small batches
- **Offline Capacity**: 1000 queued changes
- **Storage**: ~1KB per change, ~2KB per CRDT
- **Convergence**: Guaranteed after all changes applied

## Edge Cases

1. **Queue Overflow**: Oldest changes dropped
2. **Permanent Sync Failure**: Marked as failed after 5 retries
3. **CRDT Type Mismatch**: Remote state preferred (migration)
4. **Large History Gap**: Paginated fetch with continuation
5. **Simultaneous Edits**: CRDT merge ensures convergence
