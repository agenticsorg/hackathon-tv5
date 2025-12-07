# MCP Server Core - Pseudocode Design

## 1. MCP Server Core Architecture

### 1.1 Main Server Class

```
CLASS MCPServer
    PROPERTIES:
        handlers: Map<string, HandlerFunction>
        transport: TransportInterface
        rateLimiter: RateLimiter
        authenticator: Authenticator
        logger: Logger
        config: ServerConfig

    CONSTRUCTOR(config: ServerConfig)
        this.config ← config
        this.handlers ← new Map()
        this.transport ← CreateTransport(config.transport)
        this.rateLimiter ← new RateLimiter(config.rateLimit)
        this.authenticator ← new Authenticator(config.auth)
        this.logger ← new Logger(config.logLevel)

        // Register standard JSON-RPC handlers
        RegisterStandardHandlers()
    END CONSTRUCTOR

    FUNCTION Start()
        LOG("Starting MCP Server on " + config.transport.type)

        // Initialize transport
        transport.Initialize()

        // Set up request handler
        transport.OnRequest(HandleIncomingRequest)

        // Start listening
        TRY:
            transport.Listen(config.port)
            LOG("Server started successfully")
        CATCH error:
            LOG_ERROR("Failed to start server", error)
            THROW ServerStartupError(error)
        END TRY
    END FUNCTION

    FUNCTION HandleIncomingRequest(rawRequest: string)
        // Step 1: Parse JSON-RPC request
        request ← PARSE_JSON(rawRequest)
        IF request is null THEN
            RETURN ErrorResponse(-32700, "Parse error", null)
        END IF

        // Step 2: Validate JSON-RPC format
        validation ← ValidateJSONRPC(request)
        IF NOT validation.valid THEN
            RETURN ErrorResponse(-32600, validation.error, request.id)
        END IF

        // Step 3: Extract authentication token
        token ← ExtractAuthToken(request)

        // Step 4: Rate limiting check
        clientId ← ExtractClientId(request, token)
        IF NOT rateLimiter.CheckLimit(clientId) THEN
            RETURN ErrorResponse(429, "Rate limit exceeded", request.id)
        END IF

        // Step 5: Authenticate request
        authResult ← authenticator.Authenticate(token, request.method)
        IF NOT authResult.valid THEN
            RETURN ErrorResponse(401, "Unauthorized", request.id)
        END IF

        // Step 6: Route to handler
        result ← RouteRequest(request, authResult.context)

        // Step 7: Format response
        RETURN FormatJSONRPCResponse(request.id, result)
    END FUNCTION

    FUNCTION RouteRequest(request: JSONRPCRequest, authContext: AuthContext)
        method ← request.method

        // Check if handler exists
        IF NOT handlers.has(method) THEN
            RETURN ErrorResult(-32601, "Method not found: " + method)
        END IF

        // Get handler
        handler ← handlers.get(method)

        // Execute handler with error handling
        TRY:
            // Start timing for metrics
            startTime ← GetCurrentTime()

            // Execute handler
            result ← handler(request.params, authContext)

            // Record metrics
            duration ← GetCurrentTime() - startTime
            RecordMetric(method, duration, "success")

            RETURN SuccessResult(result)

        CATCH ValidationError AS e:
            RecordMetric(method, 0, "validation_error")
            RETURN ErrorResult(-32602, "Invalid params: " + e.message)

        CATCH AuthorizationError AS e:
            RecordMetric(method, 0, "auth_error")
            RETURN ErrorResult(403, "Forbidden: " + e.message)

        CATCH NotFoundError AS e:
            RecordMetric(method, 0, "not_found")
            RETURN ErrorResult(404, "Not found: " + e.message)

        CATCH error:
            RecordMetric(method, 0, "internal_error")
            LOG_ERROR("Handler error", error)
            RETURN ErrorResult(-32603, "Internal error")
        END TRY
    END FUNCTION

    FUNCTION RegisterHandler(method: string, handler: HandlerFunction)
        IF handlers.has(method) THEN
            LOG_WARN("Overwriting existing handler for: " + method)
        END IF

        handlers.set(method, handler)
        LOG("Registered handler: " + method)
    END FUNCTION

    FUNCTION RegisterStandardHandlers()
        // MCP Protocol handlers
        RegisterHandler("initialize", HandleInitialize)
        RegisterHandler("tools/list", HandleToolsList)
        RegisterHandler("tools/call", HandleToolsCall)
        RegisterHandler("prompts/list", HandlePromptsList)
        RegisterHandler("resources/list", HandleResourcesList)

        // Health check
        RegisterHandler("ping", HandlePing)
    END FUNCTION

    FUNCTION Shutdown()
        LOG("Shutting down MCP Server")

        // Stop accepting new requests
        transport.StopListening()

        // Wait for pending requests (with timeout)
        WaitForPendingRequests(timeout: 30000)

        // Close transport
        transport.Close()

        // Cleanup resources
        rateLimiter.Cleanup()
        authenticator.Cleanup()

        LOG("Server shutdown complete")
    END FUNCTION
END CLASS
```

### 1.2 Transport Interface

```
INTERFACE TransportInterface
    FUNCTION Initialize()
    FUNCTION Listen(port: integer)
    FUNCTION OnRequest(handler: Function)
    FUNCTION Send(clientId: string, message: string)
    FUNCTION StopListening()
    FUNCTION Close()
END INTERFACE
```

### 1.3 JSON-RPC Validation

```
FUNCTION ValidateJSONRPC(request: Object): ValidationResult
    // Check required fields
    IF request.jsonrpc NOT EQUALS "2.0" THEN
        RETURN {valid: false, error: "Invalid jsonrpc version"}
    END IF

    IF request.method is null OR request.method is empty THEN
        RETURN {valid: false, error: "Missing method"}
    END IF

    IF typeof request.method NOT EQUALS "string" THEN
        RETURN {valid: false, error: "Method must be string"}
    END IF

    // Validate id if present (can be string, number, or null)
    IF request.id is defined THEN
        IF typeof request.id NOT IN ["string", "number"] AND request.id NOT NULL THEN
            RETURN {valid: false, error: "Invalid id type"}
        END IF
    END IF

    // Params are optional but must be object or array if present
    IF request.params is defined THEN
        IF typeof request.params NOT IN ["object", "array"] THEN
            RETURN {valid: false, error: "Params must be object or array"}
        END IF
    END IF

    RETURN {valid: true, error: null}
END FUNCTION
```

### 1.4 Response Formatting

```
FUNCTION FormatJSONRPCResponse(requestId: any, result: Result): string
    response ← {
        jsonrpc: "2.0",
        id: requestId
    }

    IF result.isError THEN
        response.error ← {
            code: result.code,
            message: result.message,
            data: result.data
        }
    ELSE
        response.result ← result.data
    END IF

    RETURN JSON_STRINGIFY(response)
END FUNCTION

FUNCTION ErrorResponse(code: integer, message: string, id: any): string
    RETURN FormatJSONRPCResponse(id, {
        isError: true,
        code: code,
        message: message,
        data: null
    })
END FUNCTION

FUNCTION SuccessResult(data: any): Result
    RETURN {
        isError: false,
        data: data
    }
END FUNCTION

FUNCTION ErrorResult(code: integer, message: string, data: any = null): Result
    RETURN {
        isError: true,
        code: code,
        message: message,
        data: data
    }
END FUNCTION
```

## 2. Standard MCP Handlers

### 2.1 Initialize Handler

```
FUNCTION HandleInitialize(params: Object, authContext: AuthContext): Object
    // Validate params
    IF params.protocolVersion is null THEN
        THROW ValidationError("protocolVersion required")
    END IF

    IF params.capabilities is null THEN
        THROW ValidationError("capabilities required")
    END IF

    // Check protocol version compatibility
    IF NOT IsCompatibleVersion(params.protocolVersion) THEN
        THROW ValidationError("Incompatible protocol version")
    END IF

    // Return server capabilities
    RETURN {
        protocolVersion: "2024-11-05",
        capabilities: {
            tools: {
                listChanged: false
            },
            prompts: {
                listChanged: false
            },
            resources: {
                listChanged: false,
                subscribe: false
            },
            logging: {}
        },
        serverInfo: {
            name: "media-gateway-mcp",
            version: "1.0.0"
        }
    }
END FUNCTION
```

### 2.2 Tools List Handler

```
FUNCTION HandleToolsList(params: Object, authContext: AuthContext): Object
    // Get all registered tools
    tools ← GetRegisteredTools()

    // Filter by user permissions
    allowedTools ← []
    FOR EACH tool IN tools DO
        IF authContext.hasPermission(tool.requiredPermission) THEN
            allowedTools.append({
                name: tool.name,
                description: tool.description,
                inputSchema: tool.inputSchema
            })
        END IF
    END FOR

    RETURN {
        tools: allowedTools
    }
END FUNCTION
```

### 2.3 Tools Call Handler

```
FUNCTION HandleToolsCall(params: Object, authContext: AuthContext): Object
    // Validate params
    IF params.name is null THEN
        THROW ValidationError("Tool name required")
    END IF

    // Get tool
    tool ← GetTool(params.name)
    IF tool is null THEN
        THROW NotFoundError("Tool not found: " + params.name)
    END IF

    // Check permissions
    IF NOT authContext.hasPermission(tool.requiredPermission) THEN
        THROW AuthorizationError("Insufficient permissions for tool: " + params.name)
    END IF

    // Validate arguments against schema
    validation ← ValidateAgainstSchema(params.arguments, tool.inputSchema)
    IF NOT validation.valid THEN
        THROW ValidationError("Invalid arguments: " + validation.error)
    END IF

    // Execute tool
    result ← tool.execute(params.arguments, authContext)

    // Format response
    RETURN {
        content: [
            {
                type: "text",
                text: JSON_STRINGIFY(result)
            }
        ]
    }
END FUNCTION
```

## 3. Complexity Analysis

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| HandleIncomingRequest | O(1) | Fixed validation steps |
| ValidateJSONRPC | O(1) | Fixed field checks |
| RouteRequest | O(1) | Map lookup |
| Handler execution | O(n) | Varies by tool |
| FormatJSONRPCResponse | O(m) | m = response size |

### Space Complexity

| Component | Complexity | Notes |
|-----------|-----------|-------|
| Request storage | O(1) | Single request at a time |
| Handler registry | O(k) | k = number of tools |
| Rate limiter cache | O(u) | u = unique users |
| Response buffer | O(m) | m = response size |

### Bottleneck Analysis

**Primary Bottlenecks:**
1. **Tool Execution**: O(n) varies by tool complexity
   - Mitigation: Implement caching for frequently accessed data
   - Optimization: Use indexed database queries

2. **Rate Limiting**: O(1) lookup but requires cache maintenance
   - Mitigation: Use LRU cache with TTL
   - Optimization: Implement sliding window algorithm

3. **Authentication**: O(1) token validation
   - Mitigation: Cache validated tokens
   - Optimization: Use JWT with short expiration

**Optimization Strategies:**
1. Connection pooling for database queries
2. Response caching for identical requests
3. Async I/O for all network operations
4. Batch processing for multiple tool calls
