# Transport Implementations - Pseudocode Design

## 1. STDIO Transport

### 1.1 STDIO Transport Class

```
CLASS StdioTransport IMPLEMENTS TransportInterface
    PROPERTIES:
        inputStream: ReadableStream
        outputStream: WritableStream
        requestHandler: Function
        lineBuffer: string
        isListening: boolean
        pendingRequests: Set<string>

    CONSTRUCTOR()
        this.inputStream ← process.stdin
        this.outputStream ← process.stdout
        this.lineBuffer ← ""
        this.isListening ← false
        this.pendingRequests ← new Set()
    END CONSTRUCTOR

    FUNCTION Initialize()
        // Set up stdin encoding
        inputStream.setEncoding("utf8")

        // Set up error handlers
        inputStream.on("error", HandleStreamError)
        outputStream.on("error", HandleStreamError)

        LOG("STDIO transport initialized")
    END FUNCTION

    FUNCTION Listen(port: integer)
        // Port is ignored for STDIO (always uses stdin/stdout)
        this.isListening ← true

        // Set up line-delimited parsing
        inputStream.on("data", ProcessIncomingData)
        inputStream.on("end", HandleStreamEnd)

        LOG("STDIO transport listening on stdin/stdout")
    END FUNCTION

    FUNCTION ProcessIncomingData(chunk: string)
        // Add chunk to buffer
        lineBuffer ← lineBuffer + chunk

        // Process complete lines
        WHILE lineBuffer contains newline DO
            // Extract line
            lineEnd ← lineBuffer.indexOf("\n")
            line ← lineBuffer.substring(0, lineEnd)
            lineBuffer ← lineBuffer.substring(lineEnd + 1)

            // Skip empty lines
            IF line.trim() is empty THEN
                CONTINUE
            END IF

            // Process request asynchronously
            ProcessRequest(line)
        END WHILE
    END FUNCTION

    FUNCTION ProcessRequest(line: string)
        // Generate request ID for tracking
        requestId ← GenerateRequestId()
        pendingRequests.add(requestId)

        // Parse JSON to get request ID
        TRY:
            request ← PARSE_JSON(line)
            IF request.id is defined THEN
                requestId ← request.id
            END IF
        CATCH:
            // Keep generated ID
        END TRY

        // Handle request asynchronously
        ASYNC_EXECUTE(() => {
            TRY:
                // Call request handler
                response ← requestHandler(line)

                // Send response
                Send(null, response)

            CATCH error:
                LOG_ERROR("Request processing error", error)

                // Send error response
                errorResponse ← FormatErrorResponse(requestId, error)
                Send(null, errorResponse)

            FINALLY:
                // Remove from pending
                pendingRequests.delete(requestId)
            END TRY
        })
    END FUNCTION

    FUNCTION OnRequest(handler: Function)
        this.requestHandler ← handler
    END FUNCTION

    FUNCTION Send(clientId: string, message: string)
        // clientId is ignored for STDIO (only one client)
        IF NOT isListening THEN
            THROW TransportError("Transport not listening")
        END IF

        TRY:
            // Write line-delimited JSON
            outputStream.write(message + "\n")

        CATCH error:
            LOG_ERROR("Failed to send message", error)
            THROW TransportError("Send failed: " + error.message)
        END TRY
    END FUNCTION

    FUNCTION StopListening()
        this.isListening ← false

        // Remove listeners
        inputStream.removeAllListeners("data")
        inputStream.removeAllListeners("end")

        LOG("STDIO transport stopped listening")
    END FUNCTION

    FUNCTION Close()
        // Wait for pending requests
        timeout ← 5000
        waitStart ← GetCurrentTime()

        WHILE pendingRequests.size > 0 AND (GetCurrentTime() - waitStart) < timeout DO
            SLEEP(100)
        END WHILE

        IF pendingRequests.size > 0 THEN
            LOG_WARN("Closing with " + pendingRequests.size + " pending requests")
        END IF

        // Don't actually close stdin/stdout (process owns them)
        LOG("STDIO transport closed")
    END FUNCTION

    FUNCTION HandleStreamError(error: Error)
        LOG_ERROR("Stream error", error)

        IF isListening THEN
            // Attempt recovery
            LOG("Attempting to recover from stream error")
        END IF
    END FUNCTION

    FUNCTION HandleStreamEnd()
        LOG("Input stream ended")
        StopListening()
    END FUNCTION
END CLASS
```

## 2. SSE (Server-Sent Events) Transport

### 2.1 SSE Transport Class

```
CLASS SSETransport IMPLEMENTS TransportInterface
    PROPERTIES:
        httpServer: HttpServer
        requestHandler: Function
        clients: Map<string, SSEClient>
        pendingRequests: Map<string, RequestContext>
        isListening: boolean
        port: integer
        corsConfig: CORSConfig

    CONSTRUCTOR(corsConfig: CORSConfig = DEFAULT_CORS)
        this.httpServer ← null
        this.clients ← new Map()
        this.pendingRequests ← new Map()
        this.isListening ← false
        this.corsConfig ← corsConfig
    END CONSTRUCTOR

    FUNCTION Initialize()
        // Create HTTP server
        httpServer ← CreateHttpServer(HandleHttpRequest)

        LOG("SSE transport initialized")
    END FUNCTION

    FUNCTION Listen(port: integer)
        this.port ← port

        // Start HTTP server
        TRY:
            httpServer.listen(port, () => {
                this.isListening ← true
                LOG("SSE transport listening on port " + port)
            })

        CATCH error:
            LOG_ERROR("Failed to start HTTP server", error)
            THROW TransportError("Listen failed: " + error.message)
        END TRY
    END FUNCTION

    FUNCTION HandleHttpRequest(req: HttpRequest, res: HttpResponse)
        // Apply CORS headers
        ApplyCORSHeaders(res, corsConfig)

        // Handle preflight
        IF req.method EQUALS "OPTIONS" THEN
            res.status(204).end()
            RETURN
        END IF

        // Route based on path
        path ← req.url

        IF path EQUALS "/sse" THEN
            HandleSSEConnection(req, res)

        ELSE IF path EQUALS "/message" THEN
            HandleMessageEndpoint(req, res)

        ELSE IF path EQUALS "/health" THEN
            HandleHealthCheck(req, res)

        ELSE:
            res.status(404).json({error: "Not found"})
        END IF
    END FUNCTION

    FUNCTION HandleSSEConnection(req: HttpRequest, res: HttpResponse)
        // Set SSE headers
        res.setHeader("Content-Type", "text/event-stream")
        res.setHeader("Cache-Control", "no-cache")
        res.setHeader("Connection", "keep-alive")

        // Generate client ID
        clientId ← GenerateClientId()

        // Create client object
        client ← {
            id: clientId,
            response: res,
            connectedAt: GetCurrentTime(),
            lastActivity: GetCurrentTime()
        }

        // Store client
        clients.set(clientId, client)

        LOG("SSE client connected: " + clientId)

        // Send connection event
        SendSSEEvent(res, "connected", {
            clientId: clientId,
            timestamp: GetCurrentTime()
        })

        // Handle client disconnect
        req.on("close", () => {
            clients.delete(clientId)
            LOG("SSE client disconnected: " + clientId)
        })

        // Set up keep-alive ping
        SetupKeepAlive(client)
    END FUNCTION

    FUNCTION HandleMessageEndpoint(req: HttpRequest, res: HttpResponse)
        // Only accept POST
        IF req.method NOT EQUALS "POST" THEN
            res.status(405).json({error: "Method not allowed"})
            RETURN
        END IF

        // Parse request body
        body ← ""
        req.on("data", (chunk) => {
            body ← body + chunk
        })

        req.on("end", () => {
            TRY:
                // Parse JSON-RPC request
                request ← PARSE_JSON(body)

                // Extract client ID from headers or body
                clientId ← req.headers["x-client-id"] OR request.clientId

                IF clientId is null THEN
                    res.status(400).json({error: "Client ID required"})
                    RETURN
                END IF

                // Check if client is connected
                IF NOT clients.has(clientId) THEN
                    res.status(404).json({error: "Client not connected"})
                    RETURN
                END IF

                // Store request context for response routing
                requestContext ← {
                    clientId: clientId,
                    requestId: request.id,
                    receivedAt: GetCurrentTime()
                }
                pendingRequests.set(request.id, requestContext)

                // Process request
                ASYNC_EXECUTE(() => {
                    TRY:
                        // Call request handler
                        response ← requestHandler(body)

                        // Send response via SSE
                        Send(clientId, response)

                        // Acknowledge message endpoint
                        res.json({
                            status: "processed",
                            requestId: request.id
                        })

                    CATCH error:
                        LOG_ERROR("Request processing error", error)

                        // Send error via SSE
                        errorResponse ← FormatErrorResponse(request.id, error)
                        Send(clientId, errorResponse)

                        res.status(500).json({error: "Processing failed"})

                    FINALLY:
                        pendingRequests.delete(request.id)
                    END TRY
                })

            CATCH parseError:
                res.status(400).json({error: "Invalid JSON"})
            END TRY
        })
    END FUNCTION

    FUNCTION OnRequest(handler: Function)
        this.requestHandler ← handler
    END FUNCTION

    FUNCTION Send(clientId: string, message: string)
        // Get client
        client ← clients.get(clientId)

        IF client is null THEN
            LOG_WARN("Cannot send to disconnected client: " + clientId)
            RETURN
        END IF

        TRY:
            // Parse message to get request ID
            data ← PARSE_JSON(message)

            // Send as SSE event
            SendSSEEvent(client.response, "message", data)

            // Update last activity
            client.lastActivity ← GetCurrentTime()

        CATCH error:
            LOG_ERROR("Failed to send message", error)
        END TRY
    END FUNCTION

    FUNCTION SendSSEEvent(res: HttpResponse, event: string, data: Object)
        // Format SSE message
        message ← "event: " + event + "\n"
        message ← message + "data: " + JSON_STRINGIFY(data) + "\n\n"

        // Write to stream
        res.write(message)
    END FUNCTION

    FUNCTION SetupKeepAlive(client: SSEClient)
        // Send ping every 30 seconds
        interval ← INTERVAL(() => {
            IF NOT clients.has(client.id) THEN
                CLEAR_INTERVAL(interval)
                RETURN
            END IF

            TRY:
                SendSSEEvent(client.response, "ping", {
                    timestamp: GetCurrentTime()
                })
            CATCH:
                // Client disconnected
                clients.delete(client.id)
                CLEAR_INTERVAL(interval)
            END TRY
        }, 30000)
    END FUNCTION

    FUNCTION HandleHealthCheck(req: HttpRequest, res: HttpResponse)
        res.json({
            status: "healthy",
            transport: "sse",
            port: port,
            clients: clients.size,
            uptime: GetUptime()
        })
    END FUNCTION

    FUNCTION StopListening()
        this.isListening ← false

        // Close all client connections
        FOR EACH [clientId, client] IN clients DO
            TRY:
                SendSSEEvent(client.response, "shutdown", {
                    message: "Server shutting down"
                })
                client.response.end()
            CATCH:
                // Ignore errors on shutdown
            END TRY
        END FOR

        clients.clear()

        LOG("SSE transport stopped listening")
    END FUNCTION

    FUNCTION Close()
        // Stop listening
        StopListening()

        // Wait for pending requests
        timeout ← 5000
        waitStart ← GetCurrentTime()

        WHILE pendingRequests.size > 0 AND (GetCurrentTime() - waitStart) < timeout DO
            SLEEP(100)
        END WHILE

        // Close HTTP server
        IF httpServer is not null THEN
            httpServer.close()
        END IF

        LOG("SSE transport closed")
    END FUNCTION
END CLASS
```

### 2.2 CORS Configuration

```
FUNCTION ApplyCORSHeaders(res: HttpResponse, config: CORSConfig)
    // Set origin
    IF config.allowAllOrigins THEN
        res.setHeader("Access-Control-Allow-Origin", "*")
    ELSE:
        // Check origin against whitelist
        origin ← req.headers["origin"]
        IF origin IN config.allowedOrigins THEN
            res.setHeader("Access-Control-Allow-Origin", origin)
        END IF
    END IF

    // Set allowed methods
    res.setHeader("Access-Control-Allow-Methods", config.allowedMethods.join(", "))

    // Set allowed headers
    res.setHeader("Access-Control-Allow-Headers", config.allowedHeaders.join(", "))

    // Set credentials
    IF config.allowCredentials THEN
        res.setHeader("Access-Control-Allow-Credentials", "true")
    END IF

    // Set max age for preflight cache
    res.setHeader("Access-Control-Max-Age", config.maxAge.toString())
END FUNCTION

CONSTANT DEFAULT_CORS = {
    allowAllOrigins: false,
    allowedOrigins: ["http://localhost:3000", "https://app.example.com"],
    allowedMethods: ["GET", "POST", "OPTIONS"],
    allowedHeaders: ["Content-Type", "Authorization", "X-Client-ID"],
    allowCredentials: true,
    maxAge: 86400
}
```

## 3. Transport Factory

```
FUNCTION CreateTransport(config: TransportConfig): TransportInterface
    type ← config.type

    IF type EQUALS "stdio" THEN
        RETURN new StdioTransport()

    ELSE IF type EQUALS "sse" THEN
        corsConfig ← config.cors OR DEFAULT_CORS
        RETURN new SSETransport(corsConfig)

    ELSE:
        THROW ConfigurationError("Unknown transport type: " + type)
    END IF
END FUNCTION
```

## 4. Complexity Analysis

### STDIO Transport

**Time Complexity:**
- Initialize: O(1)
- Listen: O(1)
- ProcessIncomingData: O(n) where n = chunk size
- Send: O(1)
- Close: O(k) where k = pending requests

**Space Complexity:**
- Line buffer: O(m) where m = max line length
- Pending requests: O(k) where k = concurrent requests
- Total: O(m + k)

### SSE Transport

**Time Complexity:**
- Initialize: O(1)
- Listen: O(1)
- HandleSSEConnection: O(1)
- HandleMessageEndpoint: O(n) where n = request size
- Send: O(c) where c = number of clients
- Broadcast: O(c * m) where m = message size
- Close: O(c + k) where k = pending requests

**Space Complexity:**
- Client connections: O(c * s) where s = average client state
- Pending requests: O(k * r) where r = request context size
- HTTP buffers: O(b) where b = buffer size
- Total: O(c * s + k * r + b)

### Performance Characteristics

| Transport | Latency | Throughput | Scalability |
|-----------|---------|------------|-------------|
| STDIO | Very Low (< 1ms) | High | Single client |
| SSE | Low (< 10ms) | Medium | Multiple clients (1000+) |

### Optimization Strategies

**STDIO:**
1. Use buffered I/O for batching
2. Implement backpressure handling
3. Optimize line parsing with Boyer-Moore

**SSE:**
1. Implement connection pooling
2. Use compression for large messages
3. Implement message queuing for offline clients
4. Use WebSocket upgrade for bidirectional communication
5. Implement client-side reconnection with exponential backoff
