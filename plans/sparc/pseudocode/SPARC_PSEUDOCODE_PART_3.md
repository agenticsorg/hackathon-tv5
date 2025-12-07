# SPARC Pseudocode Phase - Part 3: Real-time Sync and MCP Server

**Version:** 1.0.0
**Phase:** SPARC Pseudocode
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [Real-time Synchronization (CRDTs)](#real-time-synchronization-crdts)
2. [PubNub Integration](#pubnub-integration)
3. [MCP Server Core](#mcp-server-core)
4. [MCP Tool Implementations](#mcp-tool-implementations)
5. [ARW Protocol](#arw-protocol)

---

## Real-time Synchronization (CRDTs)

### 1. Hybrid Logical Clock (HLC)

```pseudocode
STRUCTURE HybridLogicalClock:
    physicalTime: uint48          // Milliseconds since epoch
    logicalCounter: uint16        // Logical counter for causality
    nodeId: UUID                  // Device/node identifier

CONSTANTS:
    MAX_CLOCK_DRIFT = 60000       // 60 seconds max drift tolerance


ALGORITHM: HLC_Increment
INPUT: currentHLC (HybridLogicalClock), wallClock (timestamp)
OUTPUT: newHLC (HybridLogicalClock)

BEGIN
    newHLC ← NEW HybridLogicalClock()
    newHLC.nodeId ← currentHLC.nodeId

    IF wallClock > currentHLC.physicalTime THEN
        // Wall clock advanced - reset logical counter
        newHLC.physicalTime ← wallClock
        newHLC.logicalCounter ← 0
    ELSE
        // Wall clock same or behind - increment logical
        newHLC.physicalTime ← currentHLC.physicalTime
        newHLC.logicalCounter ← currentHLC.logicalCounter + 1

        // Check for counter overflow
        IF newHLC.logicalCounter > 65535 THEN
            // Wait for wall clock to advance
            WAIT_UNTIL(GetWallClock() > currentHLC.physicalTime)
            newHLC.physicalTime ← GetWallClock()
            newHLC.logicalCounter ← 0
        END IF
    END IF

    RETURN newHLC
END


ALGORITHM: HLC_Receive
INPUT: localHLC (HybridLogicalClock), remoteHLC (HybridLogicalClock)
OUTPUT: mergedHLC (HybridLogicalClock)

BEGIN
    wallClock ← GetWallClock()
    mergedHLC ← NEW HybridLogicalClock()
    mergedHLC.nodeId ← localHLC.nodeId

    // Detect clock drift
    IF ABS(remoteHLC.physicalTime - wallClock) > MAX_CLOCK_DRIFT THEN
        LOG_WARNING("Large clock drift detected: " + (remoteHLC.physicalTime - wallClock))
    END IF

    maxPhysical ← MAX(localHLC.physicalTime, remoteHLC.physicalTime, wallClock)

    IF maxPhysical = localHLC.physicalTime AND maxPhysical = remoteHLC.physicalTime THEN
        // Both have same physical time - take max logical + 1
        mergedHLC.physicalTime ← maxPhysical
        mergedHLC.logicalCounter ← MAX(localHLC.logicalCounter, remoteHLC.logicalCounter) + 1
    ELSE IF maxPhysical = localHLC.physicalTime THEN
        mergedHLC.physicalTime ← maxPhysical
        mergedHLC.logicalCounter ← localHLC.logicalCounter + 1
    ELSE IF maxPhysical = remoteHLC.physicalTime THEN
        mergedHLC.physicalTime ← maxPhysical
        mergedHLC.logicalCounter ← remoteHLC.logicalCounter + 1
    ELSE
        // Wall clock is largest
        mergedHLC.physicalTime ← maxPhysical
        mergedHLC.logicalCounter ← 0
    END IF

    RETURN mergedHLC
END


ALGORITHM: HLC_Compare
INPUT: a (HybridLogicalClock), b (HybridLogicalClock)
OUTPUT: integer (-1, 0, or 1)

BEGIN
    IF a.physicalTime < b.physicalTime THEN
        RETURN -1
    ELSE IF a.physicalTime > b.physicalTime THEN
        RETURN 1
    ELSE IF a.logicalCounter < b.logicalCounter THEN
        RETURN -1
    ELSE IF a.logicalCounter > b.logicalCounter THEN
        RETURN 1
    ELSE IF a.nodeId < b.nodeId THEN
        RETURN -1
    ELSE IF a.nodeId > b.nodeId THEN
        RETURN 1
    ELSE
        RETURN 0
    END IF
END


ALGORITHM: HLC_Encode
INPUT: hlc (HybridLogicalClock)
OUTPUT: bytes (8 bytes)

// Pack HLC into 8 bytes: 48-bit physical + 16-bit logical
BEGIN
    encoded ← NEW ByteArray(8)

    // Physical time: 6 bytes (48 bits)
    FOR i FROM 0 TO 5 DO
        encoded[i] ← (hlc.physicalTime >> (40 - i * 8)) AND 0xFF
    END FOR

    // Logical counter: 2 bytes (16 bits)
    encoded[6] ← (hlc.logicalCounter >> 8) AND 0xFF
    encoded[7] ← hlc.logicalCounter AND 0xFF

    RETURN encoded
END
```

**Complexity:** All operations O(1)

### 2. LWW-Register (Last-Writer-Wins)

```pseudocode
STRUCTURE LWWRegister<T>:
    value: T
    timestamp: HybridLogicalClock
    deviceId: UUID


ALGORITHM: LWW_Set
INPUT: register (LWWRegister<T>), newValue (T), hlc (HybridLogicalClock), deviceId (UUID)
OUTPUT: boolean (true if value was updated)

BEGIN
    IF HLC_Compare(hlc, register.timestamp) > 0 THEN
        // New timestamp is greater - update value
        register.value ← newValue
        register.timestamp ← hlc
        register.deviceId ← deviceId
        RETURN true
    ELSE IF HLC_Compare(hlc, register.timestamp) = 0 AND deviceId > register.deviceId THEN
        // Same timestamp - tie-break by device ID
        register.value ← newValue
        register.timestamp ← hlc
        register.deviceId ← deviceId
        RETURN true
    ELSE
        // Old timestamp - ignore
        RETURN false
    END IF
END


ALGORITHM: LWW_Merge
INPUT: local (LWWRegister<T>), remote (LWWRegister<T>)
OUTPUT: merged (LWWRegister<T>)

BEGIN
    comparison ← HLC_Compare(local.timestamp, remote.timestamp)

    IF comparison > 0 THEN
        RETURN local
    ELSE IF comparison < 0 THEN
        RETURN remote
    ELSE
        // Same timestamp - tie-break by device ID
        IF local.deviceId > remote.deviceId THEN
            RETURN local
        ELSE
            RETURN remote
        END IF
    END IF
END


// Application: Watch Progress
STRUCTURE WatchProgress:
    contentId: string
    progressSeconds: integer
    totalSeconds: integer
    lastWatched: timestamp
    register: LWWRegister<integer>  // The progress value


ALGORITHM: UpdateWatchProgress
INPUT: userId (string), contentId (string), progressSeconds (integer), deviceId (UUID)
OUTPUT: void

BEGIN
    key ← userId + ":" + contentId
    currentProgress ← GetWatchProgress(key)

    IF currentProgress IS NULL THEN
        currentProgress ← NEW WatchProgress()
        currentProgress.contentId ← contentId
        currentProgress.totalSeconds ← GetContentDuration(contentId)
        currentProgress.register ← NEW LWWRegister<integer>()
    END IF

    hlc ← HLC_Increment(GetLocalHLC(), GetWallClock())

    updated ← LWW_Set(
        currentProgress.register,
        progressSeconds,
        hlc,
        deviceId
    )

    IF updated THEN
        currentProgress.progressSeconds ← progressSeconds
        currentProgress.lastWatched ← GetCurrentTime()
        SaveWatchProgress(key, currentProgress)

        // Broadcast to other devices
        PublishSync(userId, SyncMessage(
            type: WATCH_PROGRESS,
            payload: {
                contentId: contentId,
                progress: progressSeconds,
                hlc: hlc,
                deviceId: deviceId
            }
        ))
    END IF
END
```

### 3. OR-Set (Observed-Remove Set)

```pseudocode
STRUCTURE ORSet<T>:
    added: Map<T, Set<UniqueTag>>     // Element -> set of add tags
    removed: Set<UniqueTag>            // Set of removed tags

STRUCTURE UniqueTag:
    elementId: T
    deviceId: UUID
    timestamp: timestamp
    randomId: UUID


ALGORITHM: ORSet_Add
INPUT: set (ORSet<T>), element (T), deviceId (UUID)
OUTPUT: void

BEGIN
    tag ← NEW UniqueTag()
    tag.elementId ← element
    tag.deviceId ← deviceId
    tag.timestamp ← GetCurrentTime()
    tag.randomId ← GenerateUUID()

    IF NOT set.added.has(element) THEN
        set.added.set(element, NEW Set<UniqueTag>())
    END IF

    set.added.get(element).add(tag)
END


ALGORITHM: ORSet_Remove
INPUT: set (ORSet<T>), element (T)
OUTPUT: void

BEGIN
    IF set.added.has(element) THEN
        tags ← set.added.get(element)

        FOR EACH tag IN tags DO
            set.removed.add(tag)
        END FOR
    END IF
END


ALGORITHM: ORSet_Contains
INPUT: set (ORSet<T>), element (T)
OUTPUT: boolean

BEGIN
    IF NOT set.added.has(element) THEN
        RETURN false
    END IF

    tags ← set.added.get(element)

    FOR EACH tag IN tags DO
        IF NOT set.removed.contains(tag) THEN
            RETURN true  // At least one active tag
        END IF
    END FOR

    RETURN false  // All tags removed
END


ALGORITHM: ORSet_GetAll
INPUT: set (ORSet<T>)
OUTPUT: List<T>

BEGIN
    elements ← []

    FOR EACH (element, tags) IN set.added DO
        hasActiveTag ← false

        FOR EACH tag IN tags DO
            IF NOT set.removed.contains(tag) THEN
                hasActiveTag ← true
                BREAK
            END IF
        END FOR

        IF hasActiveTag THEN
            elements.append(element)
        END IF
    END FOR

    RETURN elements
END


ALGORITHM: ORSet_Merge
INPUT: local (ORSet<T>), remote (ORSet<T>)
OUTPUT: merged (ORSet<T>)

BEGIN
    merged ← NEW ORSet<T>()

    // Union of added elements
    FOR EACH (element, localTags) IN local.added DO
        IF remote.added.has(element) THEN
            remoteTags ← remote.added.get(element)
            merged.added.set(element, localTags.union(remoteTags))
        ELSE
            merged.added.set(element, localTags.copy())
        END IF
    END FOR

    FOR EACH (element, remoteTags) IN remote.added DO
        IF NOT merged.added.has(element) THEN
            merged.added.set(element, remoteTags.copy())
        END IF
    END FOR

    // Union of removed tags
    merged.removed ← local.removed.union(remote.removed)

    RETURN merged
END


// Application: Watchlist
STRUCTURE Watchlist:
    userId: string
    items: ORSet<ContentId>
    lastModified: timestamp


ALGORITHM: AddToWatchlist
INPUT: userId (string), contentId (ContentId), deviceId (UUID)
OUTPUT: void

BEGIN
    watchlist ← GetWatchlist(userId)

    IF watchlist IS NULL THEN
        watchlist ← NEW Watchlist()
        watchlist.userId ← userId
        watchlist.items ← NEW ORSet<ContentId>()
    END IF

    ORSet_Add(watchlist.items, contentId, deviceId)
    watchlist.lastModified ← GetCurrentTime()
    SaveWatchlist(watchlist)

    // Broadcast sync
    PublishSync(userId, SyncMessage(
        type: WATCHLIST_ADD,
        payload: {
            contentId: contentId,
            tag: GetLatestTag(watchlist.items, contentId)
        }
    ))
END
```

---

## PubNub Integration

### 1. Channel Management

```pseudocode
CONSTANTS:
    CHANNELS:
        USER_SYNC = "user.{userId}.sync"
        USER_DEVICES = "user.{userId}.devices"
        USER_NOTIFICATIONS = "user.{userId}.notifications"
        GLOBAL_TRENDING = "global.trending"

    HEARTBEAT_INTERVAL = 30  // seconds
    PRESENCE_TIMEOUT = 60    // seconds


STRUCTURE PubNubClient:
    publishKey: string
    subscribeKey: string
    userId: string
    deviceId: string
    subscribedChannels: Set<string>
    messageHandlers: Map<string, Function>


ALGORITHM: InitializePubNub
INPUT: userId (string), deviceId (string)
OUTPUT: PubNubClient

BEGIN
    client ← NEW PubNubClient()
    client.publishKey ← GetConfigValue("PUBNUB_PUBLISH_KEY")
    client.subscribeKey ← GetConfigValue("PUBNUB_SUBSCRIBE_KEY")
    client.userId ← userId
    client.deviceId ← deviceId
    client.subscribedChannels ← NEW Set<string>()
    client.messageHandlers ← NEW Map<string, Function>()

    // Subscribe to user channels
    userSyncChannel ← CHANNELS.USER_SYNC.replace("{userId}", userId)
    userDevicesChannel ← CHANNELS.USER_DEVICES.replace("{userId}", userId)

    SubscribeToChannels(client, [userSyncChannel, userDevicesChannel])

    // Start presence heartbeat
    StartHeartbeat(client)

    RETURN client
END


ALGORITHM: SubscribeToChannels
INPUT: client (PubNubClient), channels (List<string>)
OUTPUT: void

BEGIN
    FOR EACH channel IN channels DO
        IF NOT client.subscribedChannels.contains(channel) THEN
            client.subscribedChannels.add(channel)
        END IF
    END FOR

    // PubNub subscribe call
    PubNub_Subscribe({
        channels: channels,
        withPresence: true,
        messageHandler: (message) => HandleMessage(client, message),
        presenceHandler: (event) => HandlePresence(client, event),
        statusHandler: (status) => HandleStatus(client, status)
    })
END


ALGORITHM: PublishMessage
INPUT: client (PubNubClient), channel (string), message (object)
OUTPUT: Promise<PublishResult>

CONSTANTS:
    MAX_RETRIES = 3
    RETRY_DELAY_BASE = 1000  // ms

BEGIN
    retries ← 0

    LOOP
        TRY
            result ← AWAIT PubNub_Publish({
                channel: channel,
                message: {
                    ...message,
                    senderId: client.deviceId,
                    timestamp: GetCurrentTime()
                }
            })

            RETURN result

        CATCH error AS NetworkError
            retries ← retries + 1

            IF retries >= MAX_RETRIES THEN
                // Queue for later retry
                QueueOfflineMessage(channel, message)
                THROW error
            END IF

            // Exponential backoff with jitter
            delay ← RETRY_DELAY_BASE * (2 ^ retries) + Random(0, 1000)
            AWAIT Sleep(delay)

        END TRY
    END LOOP
END


ALGORITHM: HandleMessage
INPUT: client (PubNubClient), message (PubNubMessage)
OUTPUT: void

BEGIN
    // Ignore own messages
    IF message.senderId = client.deviceId THEN
        RETURN
    END IF

    // Route to appropriate handler
    MATCH message.type
        CASE "WATCH_PROGRESS":
            HandleWatchProgressSync(client.userId, message.payload)

        CASE "WATCHLIST_ADD":
            HandleWatchlistAddSync(client.userId, message.payload)

        CASE "WATCHLIST_REMOVE":
            HandleWatchlistRemoveSync(client.userId, message.payload)

        CASE "PREFERENCE_UPDATE":
            HandlePreferenceSync(client.userId, message.payload)

        CASE "REMOTE_COMMAND":
            HandleRemoteCommand(client.deviceId, message.payload)

        DEFAULT:
            LOG_WARNING("Unknown message type: " + message.type)
    END MATCH
END
```

### 2. Presence Management

```pseudocode
STRUCTURE DevicePresence:
    deviceId: string
    deviceName: string
    deviceType: DeviceType
    state: PresenceState
    lastSeen: timestamp
    currentActivity: ActivityState

ENUM PresenceState:
    ONLINE
    IDLE
    WATCHING
    OFFLINE

ENUM ActivityState:
    NONE
    BROWSING
    SEARCHING
    WATCHING_CONTENT


ALGORITHM: StartHeartbeat
INPUT: client (PubNubClient)
OUTPUT: void

BEGIN
    ASYNC LOOP
        TRY
            // Send heartbeat with state
            state ← GetCurrentDeviceState()

            AWAIT PubNub_SetState({
                channels: [CHANNELS.USER_DEVICES.replace("{userId}", client.userId)],
                state: {
                    deviceId: client.deviceId,
                    deviceName: GetDeviceName(),
                    state: state,
                    lastSeen: GetCurrentTime()
                }
            })

        CATCH error
            LOG_ERROR("Heartbeat failed: " + error.message)
        END TRY

        AWAIT Sleep(HEARTBEAT_INTERVAL * 1000)
    END LOOP
END


ALGORITHM: HandlePresence
INPUT: client (PubNubClient), event (PresenceEvent)
OUTPUT: void

BEGIN
    MATCH event.action
        CASE "join":
            // Device came online
            NotifyDeviceOnline(event.uuid, event.state)

        CASE "leave":
            // Device went offline gracefully
            NotifyDeviceOffline(event.uuid, graceful: true)

        CASE "timeout":
            // Device timed out (ungraceful disconnect)
            NotifyDeviceOffline(event.uuid, graceful: false)

        CASE "state-change":
            // Device state updated
            UpdateDeviceState(event.uuid, event.state)
    END MATCH
END


ALGORITHM: GetOnlineDevices
INPUT: userId (string)
OUTPUT: List<DevicePresence>

BEGIN
    channel ← CHANNELS.USER_DEVICES.replace("{userId}", userId)

    hereNow ← AWAIT PubNub_HereNow({
        channels: [channel],
        includeState: true,
        includeUUIDs: true
    })

    devices ← []
    FOR EACH occupant IN hereNow.occupants DO
        device ← NEW DevicePresence()
        device.deviceId ← occupant.uuid
        device.deviceName ← occupant.state.deviceName
        device.state ← ParsePresenceState(occupant.state.state)
        device.lastSeen ← occupant.state.lastSeen
        devices.append(device)
    END FOR

    RETURN devices
END
```

### 3. Remote Control Protocol

```pseudocode
STRUCTURE RemoteCommand:
    commandId: string
    sourceDevice: string
    targetDevice: string
    action: CommandAction
    payload: object
    timestamp: timestamp
    expiresAt: timestamp

ENUM CommandAction:
    PLAY
    PAUSE
    SEEK
    STOP
    NAVIGATE
    VOLUME


ALGORITHM: SendRemoteCommand
INPUT: sourceDevice (string), targetDevice (string), action (CommandAction), payload (object)
OUTPUT: Promise<CommandResult>

CONSTANTS:
    COMMAND_TIMEOUT = 5000  // 5 seconds
    ACK_TIMEOUT = 2000      // 2 seconds

BEGIN
    commandId ← GenerateUUID()

    command ← NEW RemoteCommand()
    command.commandId ← commandId
    command.sourceDevice ← sourceDevice
    command.targetDevice ← targetDevice
    command.action ← action
    command.payload ← payload
    command.timestamp ← GetCurrentTime()
    command.expiresAt ← command.timestamp + COMMAND_TIMEOUT

    // Create acknowledgment promise
    ackPromise ← CreateAckPromise(commandId, ACK_TIMEOUT)

    // Publish command to target device channel
    targetChannel ← "device." + targetDevice + ".commands"

    TRY
        AWAIT PublishMessage(GetPubNubClient(), targetChannel, {
            type: "REMOTE_COMMAND",
            payload: command
        })

        // Wait for acknowledgment
        ack ← AWAIT ackPromise

        IF ack.status = "RECEIVED" THEN
            // Wait for completion
            completionPromise ← CreateCompletionPromise(commandId, COMMAND_TIMEOUT)
            completion ← AWAIT completionPromise

            RETURN CommandResult(
                success: completion.status = "COMPLETED",
                commandId: commandId,
                error: completion.error
            )
        ELSE
            RETURN CommandResult(success: false, error: "Device did not acknowledge")
        END IF

    CATCH TimeoutError
        RETURN CommandResult(success: false, error: "Command timed out")
    END TRY
END


ALGORITHM: HandleRemoteCommand
INPUT: deviceId (string), command (RemoteCommand)
OUTPUT: void

BEGIN
    // Validate command is for this device
    IF command.targetDevice != deviceId THEN
        RETURN
    END IF

    // Check if command has expired
    IF GetCurrentTime() > command.expiresAt THEN
        LOG_WARNING("Received expired command: " + command.commandId)
        RETURN
    END IF

    // Send acknowledgment
    SendCommandAck(command.sourceDevice, command.commandId, "RECEIVED")

    TRY
        // Execute command
        MATCH command.action
            CASE PLAY:
                result ← ExecutePlay(command.payload.contentId, command.payload.position)

            CASE PAUSE:
                result ← ExecutePause()

            CASE SEEK:
                result ← ExecuteSeek(command.payload.position)

            CASE STOP:
                result ← ExecuteStop()

            CASE NAVIGATE:
                result ← ExecuteNavigate(command.payload.destination)

            CASE VOLUME:
                result ← ExecuteVolume(command.payload.level)
        END MATCH

        // Send completion
        SendCommandAck(command.sourceDevice, command.commandId, "COMPLETED", result)

    CATCH error
        SendCommandAck(command.sourceDevice, command.commandId, "FAILED", { error: error.message })
    END TRY
END
```

---

## MCP Server Core

### 1. Server Architecture

```pseudocode
STRUCTURE MCPServer:
    tools: Map<string, MCPTool>
    resources: Map<string, MCPResource>
    transport: Transport
    requestHandlers: Map<string, Function>
    config: ServerConfig


ALGORITHM: InitializeMCPServer
INPUT: config (ServerConfig)
OUTPUT: MCPServer

BEGIN
    server ← NEW MCPServer()
    server.config ← config
    server.tools ← NEW Map<string, MCPTool>()
    server.resources ← NEW Map<string, MCPResource>()

    // Register standard request handlers
    server.requestHandlers ← {
        "initialize": HandleInitialize,
        "tools/list": HandleToolsList,
        "tools/call": HandleToolCall,
        "resources/list": HandleResourcesList,
        "resources/read": HandleResourceRead
    }

    // Register tools
    RegisterTool(server, SemanticSearchTool)
    RegisterTool(server, GetContentDetailsTool)
    RegisterTool(server, GetRecommendationsTool)
    RegisterTool(server, ListDevicesTool)
    RegisterTool(server, InitiatePlaybackTool)
    RegisterTool(server, ControlPlaybackTool)

    // Initialize transport
    IF config.transport = "stdio" THEN
        server.transport ← NEW StdioTransport()
    ELSE IF config.transport = "sse" THEN
        server.transport ← NEW SSETransport(config.port)
    END IF

    RETURN server
END


ALGORITHM: HandleRequest
INPUT: server (MCPServer), request (MCPRequest)
OUTPUT: MCPResponse

BEGIN
    // Validate JSON-RPC format
    IF request.jsonrpc != "2.0" THEN
        RETURN ErrorResponse(request.id, -32600, "Invalid Request")
    END IF

    // Find handler
    IF NOT server.requestHandlers.has(request.method) THEN
        RETURN ErrorResponse(request.id, -32601, "Method not found")
    END IF

    handler ← server.requestHandlers.get(request.method)

    TRY
        result ← AWAIT handler(server, request.params)
        RETURN SuccessResponse(request.id, result)

    CATCH ValidationError AS e
        RETURN ErrorResponse(request.id, -32602, e.message)

    CATCH error AS e
        LOG_ERROR("Handler error: " + e.message)
        RETURN ErrorResponse(request.id, -32603, "Internal error")
    END TRY
END
```

### 2. Transport Implementations

```pseudocode
// STDIO Transport (for CLI integration)
STRUCTURE StdioTransport:
    inputStream: InputStream
    outputStream: OutputStream
    buffer: string


ALGORITHM: StdioTransport_Start
INPUT: transport (StdioTransport), server (MCPServer)
OUTPUT: void

BEGIN
    transport.inputStream ← Process.stdin
    transport.outputStream ← Process.stdout
    transport.buffer ← ""

    // Read loop
    ASYNC LOOP
        chunk ← AWAIT transport.inputStream.read()

        IF chunk IS NULL THEN
            BREAK  // End of input
        END IF

        transport.buffer ← transport.buffer + chunk

        // Process complete messages (newline-delimited JSON)
        WHILE transport.buffer.contains("\n") DO
            lineEnd ← transport.buffer.indexOf("\n")
            line ← transport.buffer.substring(0, lineEnd)
            transport.buffer ← transport.buffer.substring(lineEnd + 1)

            IF line.trim().length > 0 THEN
                TRY
                    request ← JSON.parse(line)
                    response ← AWAIT HandleRequest(server, request)
                    transport.outputStream.write(JSON.stringify(response) + "\n")
                CATCH error
                    transport.outputStream.write(JSON.stringify(
                        ErrorResponse(null, -32700, "Parse error")
                    ) + "\n")
                END TRY
            END IF
        END WHILE
    END LOOP
END


// SSE Transport (for web integration)
STRUCTURE SSETransport:
    httpServer: HTTPServer
    port: integer
    connections: Map<string, SSEConnection>


ALGORITHM: SSETransport_Start
INPUT: transport (SSETransport), server (MCPServer)
OUTPUT: void

BEGIN
    transport.httpServer ← CreateHTTPServer()

    // POST /mcp - Handle JSON-RPC requests
    transport.httpServer.route("POST", "/mcp", ASYNC (req, res) => BEGIN
        // CORS headers
        res.setHeader("Access-Control-Allow-Origin", "*")
        res.setHeader("Access-Control-Allow-Methods", "POST, OPTIONS")
        res.setHeader("Access-Control-Allow-Headers", "Content-Type")

        IF req.method = "OPTIONS" THEN
            res.status(204).end()
            RETURN
        END IF

        TRY
            body ← AWAIT req.json()
            response ← AWAIT HandleRequest(server, body)
            res.json(response)
        CATCH error
            res.status(400).json(ErrorResponse(null, -32700, "Parse error"))
        END TRY
    END)

    // GET /mcp/events - SSE endpoint for streaming
    transport.httpServer.route("GET", "/mcp/events", (req, res) => BEGIN
        connectionId ← GenerateUUID()

        res.setHeader("Content-Type", "text/event-stream")
        res.setHeader("Cache-Control", "no-cache")
        res.setHeader("Connection", "keep-alive")

        connection ← NEW SSEConnection(connectionId, res)
        transport.connections.set(connectionId, connection)

        // Send initial connection event
        connection.send("connected", { connectionId: connectionId })

        // Keep-alive ping every 30 seconds
        pingInterval ← setInterval(() => BEGIN
            connection.send("ping", { timestamp: GetCurrentTime() })
        END, 30000)

        // Cleanup on disconnect
        req.on("close", () => BEGIN
            clearInterval(pingInterval)
            transport.connections.delete(connectionId)
        END)
    END)

    transport.httpServer.listen(transport.port)
    LOG_INFO("MCP SSE server listening on port " + transport.port)
END
```

---

## MCP Tool Implementations

### 1. Semantic Search Tool

```pseudocode
TOOL: semantic_search

DEFINITION:
    name: "semantic_search"
    description: "Search for movies and TV shows using natural language"
    inputSchema: {
        type: "object",
        properties: {
            query: { type: "string", description: "Natural language search query" },
            filters: {
                type: "object",
                properties: {
                    genres: { type: "array", items: { type: "string" } },
                    year_range: { type: "object", properties: { min: integer, max: integer } },
                    platforms: { type: "array", items: { type: "string" } },
                    rating_min: { type: "number" }
                }
            },
            page: { type: "integer", default: 1 },
            page_size: { type: "integer", default: 20 }
        },
        required: ["query"]
    }


ALGORITHM: ExecuteSemanticSearch
INPUT: params (object)
OUTPUT: SearchResults

BEGIN
    // Validate input
    IF params.query IS NULL OR params.query.trim().length = 0 THEN
        THROW ValidationError("Query is required")
    END IF

    // Build search query
    query ← NEW SearchQuery()
    query.query_text ← params.query
    query.page ← params.page OR 1
    query.page_size ← MIN(params.page_size OR 20, 100)
    query.strategy ← SearchStrategy.HYBRID

    // Apply filters
    IF params.filters IS NOT NULL THEN
        query.filters ← NEW SearchFilters()

        IF params.filters.genres IS NOT NULL THEN
            query.filters.genres ← MapGenreStrings(params.filters.genres)
        END IF

        IF params.filters.year_range IS NOT NULL THEN
            query.filters.year_range ← YearRange(
                min_year: params.filters.year_range.min,
                max_year: params.filters.year_range.max
            )
        END IF

        IF params.filters.platforms IS NOT NULL THEN
            query.filters.platforms ← MapPlatformStrings(params.filters.platforms)
        END IF

        IF params.filters.rating_min IS NOT NULL THEN
            query.filters.rating_range ← RatingRange(
                min_rating: params.filters.rating_min,
                max_rating: 10.0
            )
        END IF
    END IF

    // Execute search
    results ← ExecuteHybridSearch(query)

    // Format response
    RETURN {
        results: results.results.map(r => FormatContentResult(r)),
        total_count: results.total_count,
        page: results.page,
        page_size: results.page_size,
        has_more: results.page * results.page_size < results.total_count
    }
END


FUNCTION: FormatContentResult
INPUT: result (SearchResult)
OUTPUT: object

BEGIN
    content ← result.content

    RETURN {
        id: content.id,
        title: content.title,
        type: content.content_type.toString(),
        year: content.release_date.year,
        overview: TruncateText(content.overview, 200),
        genres: content.genres.map(g => g.toString()),
        rating: content.average_rating,
        popularity: content.popularity_score,
        availability: content.availability.map(a => {
            platform: a.platform.toString(),
            type: a.availability_type.toString(),
            deep_link: a.deep_link
        }),
        match_reason: result.get_primary_reason(),
        relevance_score: result.relevance_score
    }
END
```

### 2. Playback Control Tools

```pseudocode
TOOL: initiate_playback

DEFINITION:
    name: "initiate_playback"
    description: "Start playing content on a specific device"
    inputSchema: {
        type: "object",
        properties: {
            content_id: { type: "string", description: "Content ID to play" },
            device_id: { type: "string", description: "Target device ID" },
            platform: { type: "string", description: "Streaming platform" },
            start_position: { type: "integer", description: "Start position in seconds" }
        },
        required: ["content_id", "device_id"]
    }


ALGORITHM: ExecuteInitiatePlayback
INPUT: params (object), context (RequestContext)
OUTPUT: PlaybackResult

BEGIN
    // Validate device exists and is online
    device ← GetDevice(params.device_id)
    IF device IS NULL THEN
        THROW ValidationError("Device not found: " + params.device_id)
    END IF

    IF device.state = PresenceState.OFFLINE THEN
        THROW ValidationError("Device is offline")
    END IF

    // Get content details
    content ← GetContent(params.content_id)
    IF content IS NULL THEN
        THROW ValidationError("Content not found: " + params.content_id)
    END IF

    // Determine platform
    IF params.platform IS NOT NULL THEN
        platform ← ParsePlatform(params.platform)
    ELSE
        // Auto-select based on user subscriptions
        userPrefs ← GetUserPreferences(context.userId)
        platform ← SelectBestPlatform(content.availability, userPrefs.subscribed_platforms)
    END IF

    // Get deep link
    availability ← content.availability.find(a => a.platform = platform)
    IF availability IS NULL THEN
        THROW ValidationError("Content not available on " + platform.toString())
    END IF

    deepLink ← GenerateDeepLink(availability, params.start_position)

    // Send playback command to device
    result ← AWAIT SendRemoteCommand(
        context.deviceId,
        params.device_id,
        CommandAction.PLAY,
        {
            contentId: params.content_id,
            deepLink: deepLink,
            position: params.start_position OR 0
        }
    )

    RETURN {
        success: result.success,
        device_id: params.device_id,
        content_id: params.content_id,
        platform: platform.toString(),
        deep_link: deepLink,
        error: result.error
    }
END


TOOL: control_playback

DEFINITION:
    name: "control_playback"
    description: "Control playback on a device (pause, resume, seek)"
    inputSchema: {
        type: "object",
        properties: {
            device_id: { type: "string" },
            action: { type: "string", enum: ["pause", "resume", "seek", "stop"] },
            position: { type: "integer", description: "Seek position in seconds" }
        },
        required: ["device_id", "action"]
    }


ALGORITHM: ExecuteControlPlayback
INPUT: params (object), context (RequestContext)
OUTPUT: ControlResult

BEGIN
    // Map action string to enum
    action ← MATCH params.action
        CASE "pause": CommandAction.PAUSE
        CASE "resume": CommandAction.PLAY
        CASE "seek": CommandAction.SEEK
        CASE "stop": CommandAction.STOP
        DEFAULT: THROW ValidationError("Unknown action: " + params.action)
    END MATCH

    // Validate seek position for seek action
    IF action = CommandAction.SEEK AND params.position IS NULL THEN
        THROW ValidationError("Position required for seek action")
    END IF

    // Send command
    result ← AWAIT SendRemoteCommand(
        context.deviceId,
        params.device_id,
        action,
        { position: params.position }
    )

    RETURN {
        success: result.success,
        device_id: params.device_id,
        action: params.action,
        error: result.error
    }
END
```

---

## ARW Protocol

### 1. Manifest Generation

```pseudocode
STRUCTURE ARWManifest:
    protocol_version: string
    base_url: string
    capabilities: List<Capability>
    tools: List<ToolDefinition>
    authentication: AuthConfig
    rate_limits: RateLimitConfig


ALGORITHM: GenerateARWManifest
INPUT: server (MCPServer), baseUrl (string)
OUTPUT: ARWManifest

BEGIN
    manifest ← NEW ARWManifest()
    manifest.protocol_version ← "1.0"
    manifest.base_url ← baseUrl

    // Define capabilities
    manifest.capabilities ← [
        Capability("search", "Semantic content search"),
        Capability("recommendations", "Personalized recommendations"),
        Capability("playback", "Remote playback control"),
        Capability("devices", "Device management"),
        Capability("sync", "Cross-device synchronization")
    ]

    // Export tool definitions
    manifest.tools ← []
    FOR EACH (name, tool) IN server.tools DO
        manifest.tools.append(ToolDefinition(
            name: tool.name,
            description: tool.description,
            input_schema: tool.inputSchema,
            required_scopes: GetToolScopes(tool)
        ))
    END FOR

    // Authentication configuration
    manifest.authentication ← AuthConfig(
        type: "oauth2",
        authorization_url: baseUrl + "/oauth/authorize",
        token_url: baseUrl + "/oauth/token",
        scopes: [
            "read:content",
            "read:recommendations",
            "write:watchlist",
            "control:playback",
            "read:devices"
        ]
    )

    // Rate limits
    manifest.rate_limits ← RateLimitConfig(
        requests_per_minute: 60,
        burst_limit: 10,
        tier_overrides: {
            "premium": { requests_per_minute: 300, burst_limit: 50 }
        }
    )

    RETURN manifest
END


ALGORITHM: ServeARWManifest
INPUT: request (HTTPRequest)
OUTPUT: HTTPResponse

BEGIN
    manifest ← GenerateARWManifest(
        GetMCPServer(),
        GetBaseURL(request)
    )

    RETURN HTTPResponse(
        status: 200,
        headers: {
            "Content-Type": "application/json",
            "Cache-Control": "max-age=3600"
        },
        body: JSON.stringify(manifest, indent: 2)
    )
END
```

---

## Performance Summary

| Component | Latency Target | Complexity |
|-----------|---------------|------------|
| HLC Operations | <1ms | O(1) |
| LWW Merge | <1ms | O(1) |
| OR-Set Merge | <10ms | O(n*k) |
| PubNub Publish | <100ms | O(1) |
| Remote Command | <50ms | O(1) |
| MCP Request | <150ms | O(1) core |
| Semantic Search | <400ms | O(log n) |
| ARW Manifest | <10ms | O(t) |

---

**Document Status:** Complete
**Next Document:** Part 4 - Authentication, CLI, and Error Handling
**Review Required:** Security team, DevOps team

---

END OF PART 3
