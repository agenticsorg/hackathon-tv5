# SPARC Pseudocode Phase - Part 4: Authentication, CLI, and Error Handling

**Version:** 1.0.0
**Phase:** SPARC Pseudocode
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [Authentication and Authorization](#authentication-and-authorization)
2. [CLI Command Implementation](#cli-command-implementation)
3. [Error Handling and Recovery](#error-handling-and-recovery)
4. [Complexity Summary](#complexity-summary)

---

## Authentication and Authorization

### 1. OAuth 2.0 + PKCE Flow

```pseudocode
STRUCTURE OAuthState:
    state: string
    codeVerifier: string
    redirectUri: string
    scopes: List<string>
    createdAt: timestamp
    expiresAt: timestamp

CONSTANTS:
    CODE_VERIFIER_LENGTH = 64
    STATE_LENGTH = 32
    AUTH_CODE_EXPIRY = 600        // 10 minutes
    ACCESS_TOKEN_EXPIRY = 3600    // 1 hour
    REFRESH_TOKEN_EXPIRY = 2592000  // 30 days


ALGORITHM: InitiateOAuthFlow
INPUT: clientId (string), redirectUri (string), scopes (List<string>)
OUTPUT: AuthorizationURL

BEGIN
    // Generate PKCE code verifier (high-entropy random string)
    codeVerifier ← GenerateSecureRandomString(CODE_VERIFIER_LENGTH)

    // Generate code challenge (SHA-256 hash of verifier, base64url encoded)
    codeChallenge ← Base64URLEncode(SHA256(codeVerifier))

    // Generate state parameter (CSRF protection)
    state ← GenerateSecureRandomString(STATE_LENGTH)

    // Store state for verification
    oauthState ← NEW OAuthState()
    oauthState.state ← state
    oauthState.codeVerifier ← codeVerifier
    oauthState.redirectUri ← redirectUri
    oauthState.scopes ← scopes
    oauthState.createdAt ← GetCurrentTime()
    oauthState.expiresAt ← oauthState.createdAt + AUTH_CODE_EXPIRY

    StoreOAuthState(state, oauthState)

    // Build authorization URL
    authUrl ← BuildURL(GetAuthorizationEndpoint(), {
        response_type: "code",
        client_id: clientId,
        redirect_uri: redirectUri,
        scope: scopes.join(" "),
        state: state,
        code_challenge: codeChallenge,
        code_challenge_method: "S256"
    })

    RETURN authUrl
END


ALGORITHM: ExchangeCodeForTokens
INPUT: authCode (string), state (string), codeVerifier (string)
OUTPUT: TokenResponse

BEGIN
    // Verify state matches stored state
    storedState ← GetOAuthState(state)

    IF storedState IS NULL THEN
        THROW AuthError("Invalid state parameter")
    END IF

    IF GetCurrentTime() > storedState.expiresAt THEN
        DeleteOAuthState(state)
        THROW AuthError("Authorization code expired")
    END IF

    IF codeVerifier != storedState.codeVerifier THEN
        THROW AuthError("Code verifier mismatch")
    END IF

    // Exchange code for tokens
    response ← AWAIT HTTPPost(GetTokenEndpoint(), {
        grant_type: "authorization_code",
        code: authCode,
        redirect_uri: storedState.redirectUri,
        code_verifier: codeVerifier,
        client_id: GetClientId()
    })

    IF response.status != 200 THEN
        THROW AuthError("Token exchange failed: " + response.body.error)
    END IF

    // Parse and store tokens
    tokens ← ParseTokenResponse(response.body)

    // Clean up state
    DeleteOAuthState(state)

    RETURN tokens
END


ALGORITHM: ParseTokenResponse
INPUT: responseBody (object)
OUTPUT: TokenResponse

BEGIN
    tokens ← NEW TokenResponse()

    tokens.accessToken ← responseBody.access_token
    tokens.tokenType ← responseBody.token_type
    tokens.expiresIn ← responseBody.expires_in
    tokens.refreshToken ← responseBody.refresh_token
    tokens.scope ← responseBody.scope

    // Calculate expiration times
    tokens.accessTokenExpiry ← GetCurrentTime() + tokens.expiresIn
    tokens.refreshTokenExpiry ← GetCurrentTime() + REFRESH_TOKEN_EXPIRY

    RETURN tokens
END
```

### 2. Device Authorization Grant (RFC 8628)

```pseudocode
STRUCTURE DeviceCode:
    deviceCode: string
    userCode: string
    verificationUri: string
    expiresIn: integer
    interval: integer

CONSTANTS:
    USER_CODE_LENGTH = 8
    DEVICE_CODE_LENGTH = 40
    DEVICE_CODE_EXPIRY = 900      // 15 minutes
    POLL_INTERVAL = 5             // 5 seconds


ALGORITHM: RequestDeviceCode
INPUT: clientId (string), scopes (List<string>)
OUTPUT: DeviceCode

BEGIN
    response ← AWAIT HTTPPost(GetDeviceAuthorizationEndpoint(), {
        client_id: clientId,
        scope: scopes.join(" ")
    })

    IF response.status != 200 THEN
        THROW AuthError("Device authorization request failed")
    END IF

    deviceCode ← NEW DeviceCode()
    deviceCode.deviceCode ← response.body.device_code
    deviceCode.userCode ← response.body.user_code
    deviceCode.verificationUri ← response.body.verification_uri
    deviceCode.expiresIn ← response.body.expires_in
    deviceCode.interval ← response.body.interval OR POLL_INTERVAL

    RETURN deviceCode
END


ALGORITHM: PollForToken
INPUT: deviceCode (DeviceCode)
OUTPUT: TokenResponse

CONSTANTS:
    MAX_POLL_ATTEMPTS = 180  // 15 minutes at 5 second intervals

BEGIN
    attempts ← 0
    pollInterval ← deviceCode.interval

    LOOP
        attempts ← attempts + 1

        IF attempts > MAX_POLL_ATTEMPTS THEN
            THROW AuthError("Device authorization timed out")
        END IF

        // Wait before polling
        AWAIT Sleep(pollInterval * 1000)

        response ← AWAIT HTTPPost(GetTokenEndpoint(), {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code",
            device_code: deviceCode.deviceCode,
            client_id: GetClientId()
        })

        IF response.status = 200 THEN
            // Authorization successful
            RETURN ParseTokenResponse(response.body)
        END IF

        error ← response.body.error

        MATCH error
            CASE "authorization_pending":
                // User hasn't authorized yet, continue polling
                CONTINUE

            CASE "slow_down":
                // Server asking to slow down
                pollInterval ← pollInterval + 5
                CONTINUE

            CASE "expired_token":
                THROW AuthError("Device code expired")

            CASE "access_denied":
                THROW AuthError("User denied authorization")

            DEFAULT:
                THROW AuthError("Unexpected error: " + error)
        END MATCH
    END LOOP
END


ALGORITHM: DisplayDeviceAuthInstructions
INPUT: deviceCode (DeviceCode)
OUTPUT: void

BEGIN
    PRINT "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    PRINT ""
    PRINT "  To authenticate, visit:"
    PRINT ""
    PRINT "    " + deviceCode.verificationUri
    PRINT ""
    PRINT "  And enter the code:"
    PRINT ""
    PRINT "    ╔═══════════════════╗"
    PRINT "    ║  " + FormatUserCode(deviceCode.userCode) + "  ║"
    PRINT "    ╚═══════════════════╝"
    PRINT ""
    PRINT "  This code expires in " + (deviceCode.expiresIn / 60) + " minutes."
    PRINT ""
    PRINT "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    // Also display QR code if terminal supports it
    IF TerminalSupportsGraphics() THEN
        qrCode ← GenerateQRCode(deviceCode.verificationUri + "?code=" + deviceCode.userCode)
        DisplayQRCode(qrCode)
    END IF
END


FUNCTION: FormatUserCode
INPUT: code (string)
OUTPUT: string

BEGIN
    // Format as XXXX-XXXX
    IF code.length = 8 THEN
        RETURN code.substring(0, 4) + "-" + code.substring(4, 8)
    ELSE
        RETURN code
    END IF
END
```

### 3. JWT Token Management

```pseudocode
STRUCTURE JWTClaims:
    sub: string           // Subject (user ID)
    aud: string           // Audience
    iss: string           // Issuer
    exp: integer          // Expiration time
    iat: integer          // Issued at
    jti: string           // JWT ID (for revocation)
    scope: string         // Granted scopes


ALGORITHM: GenerateJWT
INPUT: userId (string), scopes (List<string>)
OUTPUT: string (JWT token)

BEGIN
    now ← GetCurrentTimeUnix()

    header ← {
        alg: "RS256",
        typ: "JWT",
        kid: GetCurrentKeyId()
    }

    claims ← NEW JWTClaims()
    claims.sub ← userId
    claims.aud ← GetAudience()
    claims.iss ← GetIssuer()
    claims.exp ← now + ACCESS_TOKEN_EXPIRY
    claims.iat ← now
    claims.jti ← GenerateUUID()
    claims.scope ← scopes.join(" ")

    // Encode header and claims
    encodedHeader ← Base64URLEncode(JSON.stringify(header))
    encodedClaims ← Base64URLEncode(JSON.stringify(claims))

    // Sign with private key
    message ← encodedHeader + "." + encodedClaims
    signature ← RS256Sign(message, GetPrivateKey())
    encodedSignature ← Base64URLEncode(signature)

    jwt ← message + "." + encodedSignature

    RETURN jwt
END


ALGORITHM: ValidateJWT
INPUT: token (string)
OUTPUT: JWTClaims

BEGIN
    // Split token
    parts ← token.split(".")
    IF parts.length != 3 THEN
        THROW AuthError("Invalid token format")
    END IF

    encodedHeader ← parts[0]
    encodedClaims ← parts[1]
    encodedSignature ← parts[2]

    // Decode header
    header ← JSON.parse(Base64URLDecode(encodedHeader))

    // Get public key for verification
    IF header.kid IS NULL THEN
        THROW AuthError("Missing key ID in token header")
    END IF

    publicKey ← GetPublicKey(header.kid)
    IF publicKey IS NULL THEN
        THROW AuthError("Unknown signing key")
    END IF

    // Verify signature
    message ← encodedHeader + "." + encodedClaims
    signature ← Base64URLDecode(encodedSignature)

    IF NOT RS256Verify(message, signature, publicKey) THEN
        THROW AuthError("Invalid token signature")
    END IF

    // Decode and validate claims
    claims ← JSON.parse(Base64URLDecode(encodedClaims))

    now ← GetCurrentTimeUnix()

    // Check expiration (with 60 second clock skew tolerance)
    IF claims.exp < now - 60 THEN
        THROW AuthError("Token expired")
    END IF

    // Check not-before (iat)
    IF claims.iat > now + 60 THEN
        THROW AuthError("Token not yet valid")
    END IF

    // Check issuer
    IF claims.iss != GetIssuer() THEN
        THROW AuthError("Invalid issuer")
    END IF

    // Check audience
    IF claims.aud != GetAudience() THEN
        THROW AuthError("Invalid audience")
    END IF

    // Check revocation
    IF IsTokenRevoked(claims.jti) THEN
        THROW AuthError("Token has been revoked")
    END IF

    RETURN claims
END
```

### 4. Refresh Token Rotation

```pseudocode
STRUCTURE RefreshTokenRecord:
    tokenId: string
    userId: string
    familyId: string          // For detecting reuse
    previousTokenId: string   // Chain for rotation
    createdAt: timestamp
    expiresAt: timestamp
    usedAt: timestamp NULLABLE
    revokedAt: timestamp NULLABLE


ALGORITHM: IssueRefreshToken
INPUT: userId (string), familyId (string NULLABLE)
OUTPUT: string (refresh token)

BEGIN
    tokenId ← GenerateSecureRandomString(64)

    // If no family, this is a new token family (fresh login)
    IF familyId IS NULL THEN
        familyId ← GenerateUUID()
    END IF

    record ← NEW RefreshTokenRecord()
    record.tokenId ← tokenId
    record.userId ← userId
    record.familyId ← familyId
    record.previousTokenId ← NULL
    record.createdAt ← GetCurrentTime()
    record.expiresAt ← record.createdAt + REFRESH_TOKEN_EXPIRY
    record.usedAt ← NULL
    record.revokedAt ← NULL

    StoreRefreshToken(record)

    RETURN tokenId
END


ALGORITHM: RotateRefreshToken
INPUT: oldToken (string)
OUTPUT: TokenResponse

BEGIN
    // Look up old token
    oldRecord ← GetRefreshToken(oldToken)

    IF oldRecord IS NULL THEN
        THROW AuthError("Invalid refresh token")
    END IF

    // Check if already used (potential token theft)
    IF oldRecord.usedAt IS NOT NULL THEN
        // Revoke entire token family
        RevokeTokenFamily(oldRecord.familyId)
        THROW AuthError("Refresh token reuse detected - all tokens revoked")
    END IF

    // Check expiration
    IF GetCurrentTime() > oldRecord.expiresAt THEN
        THROW AuthError("Refresh token expired")
    END IF

    // Check if revoked
    IF oldRecord.revokedAt IS NOT NULL THEN
        THROW AuthError("Refresh token has been revoked")
    END IF

    // Mark old token as used
    oldRecord.usedAt ← GetCurrentTime()
    UpdateRefreshToken(oldRecord)

    // Issue new refresh token in same family
    newTokenId ← GenerateSecureRandomString(64)

    newRecord ← NEW RefreshTokenRecord()
    newRecord.tokenId ← newTokenId
    newRecord.userId ← oldRecord.userId
    newRecord.familyId ← oldRecord.familyId
    newRecord.previousTokenId ← oldRecord.tokenId
    newRecord.createdAt ← GetCurrentTime()
    newRecord.expiresAt ← newRecord.createdAt + REFRESH_TOKEN_EXPIRY

    StoreRefreshToken(newRecord)

    // Generate new access token
    user ← GetUser(oldRecord.userId)
    accessToken ← GenerateJWT(user.id, user.scopes)

    RETURN TokenResponse(
        accessToken: accessToken,
        refreshToken: newTokenId,
        expiresIn: ACCESS_TOKEN_EXPIRY
    )
END


ALGORITHM: RevokeTokenFamily
INPUT: familyId (string)
OUTPUT: integer (count of revoked tokens)

BEGIN
    tokens ← GetTokensByFamily(familyId)
    revokedCount ← 0

    FOR EACH token IN tokens DO
        IF token.revokedAt IS NULL THEN
            token.revokedAt ← GetCurrentTime()
            UpdateRefreshToken(token)
            revokedCount ← revokedCount + 1
        END IF
    END FOR

    LOG_SECURITY("Token family revoked", {
        familyId: familyId,
        revokedCount: revokedCount
    })

    RETURN revokedCount
END
```

### 5. Rate Limiting

```pseudocode
STRUCTURE RateLimitBucket:
    key: string
    tokens: float
    lastRefill: timestamp
    capacity: integer
    refillRate: float         // Tokens per second


ALGORITHM: TokenBucketRateLimit
INPUT: key (string), cost (integer)
OUTPUT: RateLimitResult

CONSTANTS:
    DEFAULT_CAPACITY = 60
    DEFAULT_REFILL_RATE = 1.0   // 1 token per second

BEGIN
    bucket ← GetOrCreateBucket(key)

    // Refill tokens based on elapsed time
    now ← GetCurrentTime()
    elapsed ← (now - bucket.lastRefill).seconds
    refillAmount ← elapsed * bucket.refillRate

    bucket.tokens ← MIN(bucket.capacity, bucket.tokens + refillAmount)
    bucket.lastRefill ← now

    // Check if request can be allowed
    IF bucket.tokens >= cost THEN
        bucket.tokens ← bucket.tokens - cost
        SaveBucket(bucket)

        RETURN RateLimitResult(
            allowed: true,
            remaining: FLOOR(bucket.tokens),
            resetAt: now + ((bucket.capacity - bucket.tokens) / bucket.refillRate)
        )
    ELSE
        // Rate limited
        retryAfter ← (cost - bucket.tokens) / bucket.refillRate
        SaveBucket(bucket)

        RETURN RateLimitResult(
            allowed: false,
            remaining: 0,
            resetAt: now + retryAfter,
            retryAfter: CEIL(retryAfter)
        )
    END IF
END


ALGORITHM: GetOrCreateBucket
INPUT: key (string)
OUTPUT: RateLimitBucket

BEGIN
    bucket ← LoadBucket(key)

    IF bucket IS NULL THEN
        // Determine limits based on key type
        limits ← DetermineLimits(key)

        bucket ← NEW RateLimitBucket()
        bucket.key ← key
        bucket.tokens ← limits.capacity
        bucket.lastRefill ← GetCurrentTime()
        bucket.capacity ← limits.capacity
        bucket.refillRate ← limits.refillRate

        SaveBucket(bucket)
    END IF

    RETURN bucket
END


FUNCTION: DetermineLimits
INPUT: key (string)
OUTPUT: LimitConfig

BEGIN
    // Key format: "type:identifier"
    // Examples: "user:123", "ip:192.168.1.1", "api_key:abc123"

    parts ← key.split(":")
    keyType ← parts[0]

    MATCH keyType
        CASE "user":
            user ← GetUser(parts[1])
            MATCH user.tier
                CASE "free":
                    RETURN LimitConfig(capacity: 30, refillRate: 0.5)
                CASE "basic":
                    RETURN LimitConfig(capacity: 60, refillRate: 1.0)
                CASE "premium":
                    RETURN LimitConfig(capacity: 300, refillRate: 5.0)
                CASE "enterprise":
                    RETURN LimitConfig(capacity: 1000, refillRate: 20.0)
            END MATCH

        CASE "ip":
            // Lower limits for IP-based limiting
            RETURN LimitConfig(capacity: 20, refillRate: 0.3)

        CASE "api_key":
            apiKey ← GetApiKey(parts[1])
            RETURN LimitConfig(
                capacity: apiKey.rateLimit,
                refillRate: apiKey.rateLimit / 60.0
            )

        DEFAULT:
            RETURN LimitConfig(capacity: DEFAULT_CAPACITY, refillRate: DEFAULT_REFILL_RATE)
    END MATCH
END
```

---

## CLI Command Implementation

### 1. Command Parser Framework

```pseudocode
STRUCTURE CLIApp:
    commands: Map<string, Command>
    config: AppConfig
    logger: Logger
    version: string

CONSTANTS:
    EXIT_SUCCESS = 0
    EXIT_ERROR = 1
    EXIT_INVALID_ARGS = 2
    EXIT_AUTH_REQUIRED = 3
    EXIT_NETWORK_ERROR = 4


ALGORITHM: CLIRun
INPUT: args (List<string>)
OUTPUT: exitCode (integer)

BEGIN
    TRY
        // Parse arguments
        parsed ← ParseArguments(args)

        // Handle global flags
        IF parsed.flags.version THEN
            PRINT "media-gateway v" + GetVersion()
            RETURN EXIT_SUCCESS
        END IF

        IF parsed.flags.help OR parsed.command IS NULL THEN
            ShowHelp()
            RETURN EXIT_SUCCESS
        END IF

        // Validate command exists
        IF NOT commands.has(parsed.command) THEN
            PRINT_ERROR "Unknown command: " + parsed.command
            PRINT "Run 'media-gateway --help' for available commands"
            RETURN EXIT_INVALID_ARGS
        END IF

        command ← commands.get(parsed.command)

        // Check authentication requirement
        IF command.requiresAuth AND NOT IsAuthenticated() THEN
            PRINT_ERROR "Authentication required"
            PRINT "Run 'media-gateway auth login' to authenticate"
            RETURN EXIT_AUTH_REQUIRED
        END IF

        // Execute command
        IF IsInteractiveMode() AND command.hasInteractiveMode THEN
            result ← command.runInteractive(parsed.options)
        ELSE
            result ← command.runBatch(parsed.options)
        END IF

        // Output result
        OutputResult(result, parsed.options.format)

        RETURN result.success ? EXIT_SUCCESS : EXIT_ERROR

    CATCH NetworkError AS e
        PRINT_ERROR "Network error: " + e.message
        RETURN EXIT_NETWORK_ERROR

    CATCH AuthError AS e
        PRINT_ERROR "Authentication error: " + e.message
        RETURN EXIT_AUTH_REQUIRED

    CATCH error AS e
        PRINT_ERROR "Error: " + e.message
        IF config.debug THEN
            PRINT_ERROR e.stackTrace
        END IF
        RETURN EXIT_ERROR
    END TRY
END
```

### 2. Search Command

```pseudocode
COMMAND: search

DEFINITION:
    name: "search"
    description: "Search for movies and TV shows"
    requiresAuth: false
    aliases: ["s", "find"]
    options: [
        Option("query", alias: "q", required: true, description: "Search query"),
        Option("type", alias: "t", values: ["movie", "series", "all"], default: "all"),
        Option("genre", alias: "g", description: "Filter by genre"),
        Option("year", description: "Filter by year or range (e.g., 2020 or 2015-2020)"),
        Option("platform", alias: "p", description: "Filter by platform"),
        Option("interactive", alias: "i", type: boolean, description: "Interactive mode"),
        Option("format", alias: "f", values: ["table", "json", "minimal"], default: "table")
    ]


ALGORITHM: ExecuteSearchCommand
INPUT: options (object)
OUTPUT: CommandResult

BEGIN
    // Build search query
    query ← options.query

    filters ← NEW SearchFilters()

    IF options.type != "all" THEN
        filters.content_types ← [ParseContentType(options.type)]
    END IF

    IF options.genre IS NOT NULL THEN
        filters.genres ← [ParseGenre(options.genre)]
    END IF

    IF options.year IS NOT NULL THEN
        filters.year_range ← ParseYearRange(options.year)
    END IF

    IF options.platform IS NOT NULL THEN
        filters.platforms ← [ParsePlatform(options.platform)]
    END IF

    // Execute search
    ShowSpinner("Searching...")

    results ← AWAIT ExecuteSearch(query, filters)

    HideSpinner()

    IF results.total_count = 0 THEN
        PRINT "No results found for: " + query
        RETURN CommandResult(success: true, data: [])
    END IF

    // Interactive mode
    IF options.interactive THEN
        RETURN RunInteractiveSearchBrowser(results)
    END IF

    // Display results
    DisplaySearchResults(results, options.format)

    RETURN CommandResult(success: true, data: results)
END


ALGORITHM: RunInteractiveSearchBrowser
INPUT: results (SearchResults)
OUTPUT: CommandResult

BEGIN
    currentPage ← 1
    selectedIndex ← 0

    LOOP
        ClearScreen()

        // Display header
        PRINT "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        PRINT "  Search Results (" + results.total_count + " found)"
        PRINT "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        PRINT ""

        // Display results for current page
        startIndex ← (currentPage - 1) * 10
        endIndex ← MIN(startIndex + 10, results.results.length)

        FOR i FROM startIndex TO endIndex - 1 DO
            result ← results.results[i]
            prefix ← IF i = selectedIndex THEN "▶ " ELSE "  "
            highlight ← IF i = selectedIndex THEN ANSI_HIGHLIGHT ELSE ""
            reset ← IF i = selectedIndex THEN ANSI_RESET ELSE ""

            PRINT highlight + prefix + FormatResultLine(result) + reset
        END FOR

        // Display footer
        PRINT ""
        PRINT "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        PRINT "  ↑/↓: Navigate  Enter: View Details  q: Quit"
        PRINT "  n: Next Page   p: Previous Page"
        PRINT "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

        // Handle input
        key ← ReadKeyPress()

        MATCH key
            CASE KEY_UP:
                selectedIndex ← MAX(0, selectedIndex - 1)

            CASE KEY_DOWN:
                selectedIndex ← MIN(results.results.length - 1, selectedIndex + 1)

            CASE KEY_ENTER:
                ShowContentDetails(results.results[selectedIndex].content)

            CASE 'n':
                IF endIndex < results.total_count THEN
                    currentPage ← currentPage + 1
                    selectedIndex ← (currentPage - 1) * 10
                    results ← AWAIT FetchNextPage(results, currentPage)
                END IF

            CASE 'p':
                IF currentPage > 1 THEN
                    currentPage ← currentPage - 1
                    selectedIndex ← (currentPage - 1) * 10
                END IF

            CASE 'q', KEY_ESCAPE:
                BREAK
        END MATCH
    END LOOP

    RETURN CommandResult(success: true)
END
```

### 3. Auth Command

```pseudocode
COMMAND: auth

DEFINITION:
    name: "auth"
    description: "Manage authentication"
    requiresAuth: false
    subcommands: ["login", "logout", "status", "refresh"]


ALGORITHM: ExecuteAuthLogin
INPUT: options (object)
OUTPUT: CommandResult

BEGIN
    // Check if already logged in
    IF IsAuthenticated() THEN
        PRINT "Already logged in as: " + GetCurrentUser().email
        confirm ← Prompt("Do you want to log in with a different account? [y/N]")
        IF confirm.toLowerCase() != "y" THEN
            RETURN CommandResult(success: true)
        END IF
    END IF

    // Device flow for TV/CLI
    PRINT "Initiating device authorization..."

    deviceCode ← AWAIT RequestDeviceCode(
        GetClientId(),
        ["read:content", "read:recommendations", "write:watchlist", "control:playback"]
    )

    DisplayDeviceAuthInstructions(deviceCode)

    PRINT ""
    PRINT "Waiting for authorization..."

    // Start polling spinner
    StartSpinner("Waiting for authorization")

    TRY
        tokens ← AWAIT PollForToken(deviceCode)

        StopSpinner()

        // Store tokens securely
        StoreTokens(tokens)

        // Fetch user info
        user ← AWAIT FetchUserInfo(tokens.accessToken)

        PRINT ""
        PRINT "✓ Successfully logged in as: " + user.email
        PRINT ""

        RETURN CommandResult(success: true, data: { user: user })

    CATCH AuthError AS e
        StopSpinner()
        PRINT_ERROR "Authentication failed: " + e.message
        RETURN CommandResult(success: false, error: e.message)
    END TRY
END


ALGORITHM: ExecuteAuthLogout
INPUT: options (object)
OUTPUT: CommandResult

BEGIN
    IF NOT IsAuthenticated() THEN
        PRINT "Not currently logged in"
        RETURN CommandResult(success: true)
    END IF

    user ← GetCurrentUser()

    // Revoke tokens on server
    TRY
        tokens ← GetStoredTokens()
        AWAIT RevokeToken(tokens.refreshToken)
    CATCH error
        // Continue with local logout even if server revocation fails
        LOG_WARNING("Failed to revoke token on server: " + error.message)
    END TRY

    // Clear local tokens
    ClearStoredTokens()

    PRINT "✓ Successfully logged out"

    RETURN CommandResult(success: true)
END


ALGORITHM: ExecuteAuthStatus
INPUT: options (object)
OUTPUT: CommandResult

BEGIN
    IF NOT IsAuthenticated() THEN
        PRINT "Not logged in"
        PRINT ""
        PRINT "Run 'media-gateway auth login' to authenticate"
        RETURN CommandResult(success: true, data: { authenticated: false })
    END IF

    tokens ← GetStoredTokens()
    user ← GetCurrentUser()

    accessTokenExpiry ← ParseJWTExpiry(tokens.accessToken)
    refreshTokenExpiry ← tokens.refreshTokenExpiry

    PRINT "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    PRINT "  Authentication Status"
    PRINT "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    PRINT ""
    PRINT "  User:          " + user.email
    PRINT "  User ID:       " + user.id
    PRINT "  Tier:          " + user.tier
    PRINT ""
    PRINT "  Access Token:  " + FormatTokenStatus(accessTokenExpiry)
    PRINT "  Refresh Token: " + FormatTokenStatus(refreshTokenExpiry)
    PRINT ""
    PRINT "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    RETURN CommandResult(success: true, data: {
        authenticated: true,
        user: user,
        accessTokenExpiry: accessTokenExpiry,
        refreshTokenExpiry: refreshTokenExpiry
    })
END
```

### 4. Cast Command

```pseudocode
COMMAND: cast

DEFINITION:
    name: "cast"
    description: "Play content on a device"
    requiresAuth: true
    aliases: ["play"]
    options: [
        Option("content", alias: "c", required: true, description: "Content ID or search query"),
        Option("device", alias: "d", description: "Target device ID or name"),
        Option("platform", alias: "p", description: "Streaming platform"),
        Option("from", description: "Start position (e.g., 10m30s, 630)")
    ]


ALGORITHM: ExecuteCastCommand
INPUT: options (object)
OUTPUT: CommandResult

BEGIN
    // Resolve content
    content ← NULL

    IF IsContentId(options.content) THEN
        content ← AWAIT GetContentById(options.content)
    ELSE
        // Search for content
        results ← AWAIT ExecuteSearch(options.content, NEW SearchFilters())

        IF results.total_count = 0 THEN
            PRINT_ERROR "No content found matching: " + options.content
            RETURN CommandResult(success: false)
        END IF

        IF results.total_count = 1 THEN
            content ← results.results[0].content
        ELSE
            // Let user select
            content ← PromptContentSelection(results.results)
        END IF
    END IF

    PRINT "Selected: " + content.title + " (" + content.release_date.year + ")"

    // Resolve device
    device ← NULL

    IF options.device IS NOT NULL THEN
        device ← AWAIT FindDeviceByNameOrId(options.device)
    ELSE
        // Get online devices and let user select
        devices ← AWAIT GetOnlineDevices(GetCurrentUser().id)

        IF devices.length = 0 THEN
            PRINT_ERROR "No online devices found"
            RETURN CommandResult(success: false)
        END IF

        IF devices.length = 1 THEN
            device ← devices[0]
        ELSE
            device ← PromptDeviceSelection(devices)
        END IF
    END IF

    PRINT "Target device: " + device.deviceName

    // Resolve platform
    platform ← NULL

    IF options.platform IS NOT NULL THEN
        platform ← ParsePlatform(options.platform)
        availability ← content.availability.find(a => a.platform = platform)
        IF availability IS NULL THEN
            PRINT_ERROR "Content not available on " + platform
            RETURN CommandResult(success: false)
        END IF
    ELSE
        // Auto-select based on subscriptions
        userPrefs ← AWAIT GetUserPreferences(GetCurrentUser().id)
        platform ← SelectBestPlatform(content.availability, userPrefs.subscribed_platforms)

        IF platform IS NULL THEN
            PRINT_ERROR "Content not available on any of your subscribed platforms"
            PRINT "Available on: " + content.availability.map(a => a.platform).join(", ")
            RETURN CommandResult(success: false)
        END IF
    END IF

    PRINT "Platform: " + platform

    // Parse start position
    startPosition ← 0
    IF options.from IS NOT NULL THEN
        startPosition ← ParseDuration(options.from)
        PRINT "Starting from: " + FormatDuration(startPosition)
    END IF

    // Send cast command
    ShowSpinner("Sending to " + device.deviceName + "...")

    result ← AWAIT InitiatePlayback(
        content.id,
        device.deviceId,
        platform,
        startPosition
    )

    HideSpinner()

    IF result.success THEN
        PRINT ""
        PRINT "✓ Now playing on " + device.deviceName
        PRINT ""
        RETURN CommandResult(success: true, data: result)
    ELSE
        PRINT_ERROR "Failed to start playback: " + result.error
        RETURN CommandResult(success: false, error: result.error)
    END IF
END
```

---

## Error Handling and Recovery

### 1. Error Classification

```pseudocode
HIERARCHY ErrorTypes:

  NetworkError
    - ConnectionTimeout      // retry: exponential backoff
    - DNSFailure             // retry: failover to backup
    - SSLError               // no retry, alert

  AuthError
    - TokenExpired           // auto-refresh, then retry
    - TokenInvalid           // re-authenticate
    - InsufficientScope      // show upgrade prompt

  APIError
    - RateLimited (429)      // wait, retry with backoff
    - ServiceUnavailable (503)  // circuit breaker
    - ValidationError (400)  // user feedback
    - NotFound (404)         // graceful degradation

  DataError
    - SyncConflict           // CRDT merge, auto-resolve
    - StaleData              // refresh cache
    - CorruptedData          // restore from backup

  PlatformError
    - YouTubeQuotaExceeded   // fallback to cache
    - StreamingUnavailable   // suggest alternatives
    - RegionRestricted       // notify user


STRUCTURE ErrorContext:
    errorType: ErrorType
    message: string
    code: string
    timestamp: timestamp
    retryable: boolean
    retryCount: integer
    maxRetries: integer
    metadata: Map<string, any>
```

### 2. Retry Strategy Engine

```pseudocode
STRUCTURE RetryConfig:
    maxRetries: integer
    baseDelay: integer          // milliseconds
    maxDelay: integer           // milliseconds
    backoffMultiplier: float
    jitterFactor: float         // 0.0 to 1.0


ALGORITHM: ExecuteWithRetry
INPUT: operation (Function), config (RetryConfig)
OUTPUT: Result<T, Error>

CONSTANTS:
    DEFAULT_MAX_RETRIES = 3
    DEFAULT_BASE_DELAY = 1000
    DEFAULT_MAX_DELAY = 30000
    DEFAULT_BACKOFF = 2.0
    DEFAULT_JITTER = 0.3

BEGIN
    config ← config OR GetDefaultRetryConfig()
    attempt ← 0

    LOOP
        attempt ← attempt + 1

        TRY
            result ← AWAIT operation()
            RETURN Success(result)

        CATCH error
            // Check if error is retryable
            IF NOT IsRetryableError(error) THEN
                RETURN Failure(error)
            END IF

            IF attempt >= config.maxRetries THEN
                LOG_ERROR("Max retries exceeded", {
                    attempts: attempt,
                    error: error.message
                })
                RETURN Failure(error)
            END IF

            // Calculate delay with exponential backoff and jitter
            baseDelay ← config.baseDelay * (config.backoffMultiplier ^ (attempt - 1))
            jitter ← baseDelay * config.jitterFactor * Random(-1, 1)
            delay ← MIN(baseDelay + jitter, config.maxDelay)

            LOG_WARNING("Retrying operation", {
                attempt: attempt,
                maxRetries: config.maxRetries,
                delay: delay,
                error: error.message
            })

            AWAIT Sleep(delay)
        END TRY
    END LOOP
END


FUNCTION: IsRetryableError
INPUT: error (Error)
OUTPUT: boolean

BEGIN
    MATCH error.type
        CASE NetworkError.ConnectionTimeout:
            RETURN true
        CASE NetworkError.DNSFailure:
            RETURN true
        CASE APIError.RateLimited:
            RETURN true
        CASE APIError.ServiceUnavailable:
            RETURN true
        CASE AuthError.TokenExpired:
            RETURN true
        CASE DataError.StaleData:
            RETURN true
        DEFAULT:
            RETURN false
    END MATCH
END
```

### 3. Circuit Breaker Pattern

```pseudocode
ENUM CircuitState:
    CLOSED      // Normal operation
    OPEN        // Failing, reject requests
    HALF_OPEN   // Testing if service recovered

STRUCTURE CircuitBreaker:
    state: CircuitState
    failureCount: integer
    successCount: integer
    lastFailureTime: timestamp
    failureThreshold: integer
    successThreshold: integer
    resetTimeout: integer         // seconds

CONSTANTS:
    DEFAULT_FAILURE_THRESHOLD = 5
    DEFAULT_SUCCESS_THRESHOLD = 2
    DEFAULT_RESET_TIMEOUT = 60


ALGORITHM: CircuitBreakerExecute
INPUT: breaker (CircuitBreaker), operation (Function)
OUTPUT: Result<T, Error>

BEGIN
    // Check if circuit is open
    IF breaker.state = CircuitState.OPEN THEN
        // Check if reset timeout has elapsed
        IF GetCurrentTime() - breaker.lastFailureTime > breaker.resetTimeout THEN
            breaker.state ← CircuitState.HALF_OPEN
            breaker.successCount ← 0
            LOG_INFO("Circuit breaker entering half-open state")
        ELSE
            THROW CircuitOpenError("Circuit breaker is open")
        END IF
    END IF

    TRY
        result ← AWAIT operation()

        // Record success
        RecordSuccess(breaker)

        RETURN Success(result)

    CATCH error
        // Record failure
        RecordFailure(breaker, error)

        THROW error
    END TRY
END


ALGORITHM: RecordSuccess
INPUT: breaker (CircuitBreaker)
OUTPUT: void

BEGIN
    IF breaker.state = CircuitState.HALF_OPEN THEN
        breaker.successCount ← breaker.successCount + 1

        IF breaker.successCount >= breaker.successThreshold THEN
            // Transition to closed
            breaker.state ← CircuitState.CLOSED
            breaker.failureCount ← 0
            breaker.successCount ← 0
            LOG_INFO("Circuit breaker closed after successful recovery")

            // Emit event
            EmitEvent("circuit_breaker.closed", { name: breaker.name })
        END IF
    ELSE
        // In closed state, reset failure count on success
        breaker.failureCount ← MAX(0, breaker.failureCount - 1)
    END IF
END


ALGORITHM: RecordFailure
INPUT: breaker (CircuitBreaker), error (Error)
OUTPUT: void

BEGIN
    breaker.failureCount ← breaker.failureCount + 1
    breaker.lastFailureTime ← GetCurrentTime()

    IF breaker.state = CircuitState.HALF_OPEN THEN
        // Failed during recovery test, back to open
        breaker.state ← CircuitState.OPEN
        LOG_WARNING("Circuit breaker reopened after failed recovery test")
        EmitEvent("circuit_breaker.reopened", { name: breaker.name, error: error.message })

    ELSE IF breaker.failureCount >= breaker.failureThreshold THEN
        // Too many failures, open the circuit
        breaker.state ← CircuitState.OPEN
        LOG_WARNING("Circuit breaker opened due to failures", {
            failures: breaker.failureCount,
            threshold: breaker.failureThreshold
        })
        EmitEvent("circuit_breaker.opened", { name: breaker.name, failures: breaker.failureCount })
    END IF
END
```

### 4. Graceful Degradation

```pseudocode
ENUM DegradationLevel:
    FULL        // All features available
    REDUCED     // Some features disabled
    MINIMAL     // Core features only
    OFFLINE     // Offline mode

STRUCTURE DegradationState:
    level: DegradationLevel
    disabledFeatures: Set<string>
    reason: string
    since: timestamp


ALGORITHM: DetermineServiceHealth
INPUT: healthChecks (List<HealthCheck>)
OUTPUT: DegradationState

BEGIN
    failedChecks ← []
    warningChecks ← []

    FOR EACH check IN healthChecks DO
        result ← AWAIT check.execute()

        IF result.status = FAILED THEN
            failedChecks.append(check)
        ELSE IF result.status = WARNING THEN
            warningChecks.append(check)
        END IF
    END FOR

    state ← NEW DegradationState()
    state.since ← GetCurrentTime()

    // Determine degradation level
    IF failedChecks.length = 0 AND warningChecks.length = 0 THEN
        state.level ← DegradationLevel.FULL
        state.disabledFeatures ← EmptySet
        state.reason ← "All systems operational"

    ELSE IF failedChecks.length = 0 THEN
        state.level ← DegradationLevel.REDUCED
        state.disabledFeatures ← warningChecks.map(c => c.affectedFeatures).flatten()
        state.reason ← "Some services degraded"

    ELSE IF failedChecks.any(c => c.critical) THEN
        state.level ← DegradationLevel.MINIMAL
        state.disabledFeatures ← GetNonCoreFeatures()
        state.reason ← "Critical services unavailable"

    ELSE
        state.level ← DegradationLevel.REDUCED
        state.disabledFeatures ← failedChecks.map(c => c.affectedFeatures).flatten()
        state.reason ← "Some services unavailable"
    END IF

    RETURN state
END


ALGORITHM: ExecuteWithFallback
INPUT: primary (Function), fallbacks (List<Fallback>)
OUTPUT: Result<T, Error>

BEGIN
    // Try primary operation
    TRY
        result ← AWAIT primary()
        RETURN Success(result)
    CATCH primaryError
        LOG_WARNING("Primary operation failed, trying fallbacks", {
            error: primaryError.message
        })
    END TRY

    // Try fallbacks in order
    FOR EACH fallback IN fallbacks DO
        IF fallback.isAvailable() THEN
            TRY
                result ← AWAIT fallback.execute()
                LOG_INFO("Fallback succeeded", { fallback: fallback.name })
                RETURN Success(result)
            CATCH fallbackError
                LOG_WARNING("Fallback failed", {
                    fallback: fallback.name,
                    error: fallbackError.message
                })
            END TRY
        END IF
    END FOR

    // All fallbacks exhausted
    RETURN Failure(NEW Error("All fallbacks exhausted"))
END


// Example: Search with fallbacks
ALGORITHM: SearchWithFallbacks
INPUT: query (SearchQuery)
OUTPUT: SearchResults

BEGIN
    primary ← () => ExecuteHybridSearch(query)

    fallbacks ← [
        Fallback("cached_results", () => GetCachedSearch(query)),
        Fallback("keyword_only", () => ExecuteKeywordSearch(query)),
        Fallback("trending", () => GetTrendingContent())
    ]

    result ← AWAIT ExecuteWithFallback(primary, fallbacks)

    IF result.isSuccess THEN
        RETURN result.value
    ELSE
        // Return empty results as last resort
        RETURN SearchResults(results: [], total_count: 0)
    END IF
END
```

---

## Complexity Summary

### Algorithm Complexity Table

| Algorithm | Time Complexity | Space Complexity | Notes |
|-----------|----------------|------------------|-------|
| **Authentication** |
| OAuth PKCE Flow | O(1) | O(1) | Per-request |
| Device Auth Poll | O(n) | O(1) | n = poll attempts |
| JWT Validation | O(1) | O(1) | Cached keys |
| Token Rotation | O(1) | O(f) | f = family size |
| Rate Limiting | O(1) | O(k) | k = unique keys |
| **CLI** |
| Argument Parsing | O(a) | O(a) | a = argument count |
| Interactive Search | O(p * r) | O(r) | p = pages, r = results |
| Device Selection | O(d) | O(d) | d = device count |
| **Error Handling** |
| Retry with Backoff | O(r) | O(1) | r = retry count |
| Circuit Breaker | O(1) | O(1) | Amortized |
| Health Check | O(c) | O(c) | c = check count |
| Fallback Chain | O(f) | O(1) | f = fallback count |

### Storage Requirements

| Component | Per-User Size | Total (1M users) |
|-----------|--------------|------------------|
| OAuth State | 200 bytes | 200 MB (active) |
| Refresh Tokens | 500 bytes | 500 MB |
| Rate Limit Buckets | 100 bytes | 100 MB |
| Circuit Breakers | 100 bytes | Negligible (shared) |

---

## Appendix: Error Codes

| Code | Name | Description | User Message |
|------|------|-------------|--------------|
| E1001 | AUTH_REQUIRED | Not authenticated | Please log in to continue |
| E1002 | TOKEN_EXPIRED | Access token expired | Your session expired. Refreshing... |
| E1003 | INVALID_TOKEN | Token validation failed | Please log in again |
| E2001 | RATE_LIMITED | Rate limit exceeded | Too many requests. Try again in {retry_after}s |
| E2002 | SERVICE_DOWN | Service unavailable | Service temporarily unavailable |
| E3001 | NOT_FOUND | Content not found | Content not found |
| E3002 | DEVICE_OFFLINE | Device not online | Device is offline |
| E4001 | SYNC_CONFLICT | Data conflict | Syncing your data... |
| E4002 | NETWORK_ERROR | Network failure | Check your internet connection |

---

**Document Status:** Complete
**Phase Complete:** SPARC Pseudocode Phase
**Total Documents:** 4 parts, ~200 pages
**Next Phase:** Architecture

---

END OF PART 4 AND SPARC PSEUDOCODE PHASE
