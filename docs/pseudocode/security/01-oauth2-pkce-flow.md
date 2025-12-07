# OAuth 2.0 + PKCE Flow - Pseudocode Specification

## Overview
OAuth 2.0 with Proof Key for Code Exchange (PKCE) provides secure authorization for public clients without requiring client secrets. This implementation follows RFC 7636.

---

## Data Structures

```
STRUCTURE PKCESession:
    state: String (32 bytes, cryptographically random)
    code_verifier: String (64 bytes, base64url-encoded)
    code_challenge: String (SHA256 hash, base64url-encoded)
    client_id: String
    redirect_uri: URI
    scopes: Array<String>
    created_at: Timestamp
    expires_at: Timestamp (created_at + 10 minutes)

STRUCTURE AuthorizationCode:
    code: String (32 bytes, cryptographically random)
    client_id: String
    redirect_uri: URI
    scopes: Array<String>
    code_challenge: String
    user_id: String
    created_at: Timestamp
    expires_at: Timestamp (created_at + 10 minutes)
    used: Boolean (default: false)

STRUCTURE TokenPair:
    access_token: JWT (1 hour expiry)
    refresh_token: String (30 days expiry)
    token_type: String = "Bearer"
    expires_in: Integer = 3600
    scope: String
```

---

## Algorithm 1: Initiate Authorization Flow

```
ALGORITHM: InitiateOAuthFlow
INPUT: client_id (string), redirect_uri (URI), scopes (array of strings)
OUTPUT: authorization_url (URI) or error

CONSTANTS:
    CODE_VERIFIER_LENGTH = 64
    STATE_LENGTH = 32
    SESSION_TTL = 600 seconds (10 minutes)
    ALLOWED_CHARACTERS = [A-Z, a-z, 0-9, -, ., _, ~]

BEGIN
    // Step 1: Validate client application
    client ← Database.findClientByID(client_id)

    IF client is null THEN
        AuditLog.record(
            event="oauth_invalid_client",
            client_id=client_id,
            severity="warning"
        )
        RETURN error("Invalid client_id")
    END IF

    // Step 2: Validate redirect URI
    IF redirect_uri NOT IN client.allowed_redirect_uris THEN
        AuditLog.record(
            event="oauth_invalid_redirect_uri",
            client_id=client_id,
            attempted_uri=redirect_uri,
            severity="critical"
        )
        RETURN error("Invalid redirect_uri")
    END IF

    // Step 3: Validate scopes
    invalid_scopes ← []
    FOR EACH scope IN scopes DO
        IF scope NOT IN client.allowed_scopes THEN
            invalid_scopes.append(scope)
        END IF
    END FOR

    IF invalid_scopes is not empty THEN
        AuditLog.record(
            event="oauth_invalid_scopes",
            client_id=client_id,
            invalid_scopes=invalid_scopes,
            severity="warning"
        )
        RETURN error("Invalid scopes: " + join(invalid_scopes, ", "))
    END IF

    // Step 4: Generate PKCE parameters
    code_verifier ← GenerateCryptographicString(
        length=CODE_VERIFIER_LENGTH,
        charset=ALLOWED_CHARACTERS
    )

    code_challenge ← Base64URLEncode(
        SHA256(code_verifier)
    )

    state ← GenerateCryptographicString(
        length=STATE_LENGTH,
        charset=ALLOWED_CHARACTERS
    )

    // Step 5: Create session
    session ← PKCESession{
        state: state,
        code_verifier: code_verifier,
        code_challenge: code_challenge,
        client_id: client_id,
        redirect_uri: redirect_uri,
        scopes: scopes,
        created_at: GetCurrentTimestamp(),
        expires_at: GetCurrentTimestamp() + SESSION_TTL
    }

    // Step 6: Store session in Redis with TTL
    Cache.setWithTTL(
        key="pkce_session:" + state,
        value=session,
        ttl=SESSION_TTL
    )

    // Step 7: Build authorization URL
    authorization_url ← BuildURL(
        base="/oauth/authorize",
        params={
            client_id: client_id,
            redirect_uri: redirect_uri,
            response_type: "code",
            scope: join(scopes, " "),
            state: state,
            code_challenge: code_challenge,
            code_challenge_method: "S256"
        }
    )

    // Step 8: Audit log
    AuditLog.record(
        event="oauth_flow_initiated",
        client_id=client_id,
        scopes=scopes,
        state=state,
        severity="info"
    )

    RETURN authorization_url
END

SUBROUTINE: GenerateCryptographicString
INPUT: length (integer), charset (array of characters)
OUTPUT: random_string (string)

BEGIN
    // Use cryptographically secure random number generator (CSRNG)
    random_bytes ← CSRNG.generateBytes(length)

    result ← ""
    FOR EACH byte IN random_bytes DO
        // Map byte to charset
        index ← byte MOD charset.length
        result ← result + charset[index]
    END FOR

    RETURN result
END
```

**Time Complexity**: O(n) where n = length of code verifier
**Space Complexity**: O(1) for operations, O(n) for storage

---

## Algorithm 2: Authorization Grant (User Consent)

```
ALGORITHM: GrantAuthorization
INPUT: state (string), user_id (string), approved_scopes (array of strings)
OUTPUT: authorization_code (string) or error

CONSTANTS:
    AUTH_CODE_LENGTH = 32
    AUTH_CODE_TTL = 600 seconds (10 minutes)

BEGIN
    // Step 1: Retrieve and validate PKCE session
    session ← Cache.get("pkce_session:" + state)

    IF session is null THEN
        AuditLog.record(
            event="oauth_invalid_state",
            state=state,
            severity="warning"
        )
        RETURN error("Invalid or expired state parameter")
    END IF

    IF GetCurrentTimestamp() > session.expires_at THEN
        Cache.delete("pkce_session:" + state)
        AuditLog.record(
            event="oauth_session_expired",
            state=state,
            severity="info"
        )
        RETURN error("Session expired")
    END IF

    // Step 2: Validate approved scopes are subset of requested
    FOR EACH scope IN approved_scopes DO
        IF scope NOT IN session.scopes THEN
            AuditLog.record(
                event="oauth_scope_escalation_attempt",
                state=state,
                user_id=user_id,
                invalid_scope=scope,
                severity="critical"
            )
            RETURN error("Scope escalation detected")
        END IF
    END FOR

    // Step 3: Generate authorization code
    auth_code ← GenerateCryptographicString(
        length=AUTH_CODE_LENGTH,
        charset=ALLOWED_CHARACTERS
    )

    // Step 4: Create authorization code record
    code_record ← AuthorizationCode{
        code: auth_code,
        client_id: session.client_id,
        redirect_uri: session.redirect_uri,
        scopes: approved_scopes,
        code_challenge: session.code_challenge,
        user_id: user_id,
        created_at: GetCurrentTimestamp(),
        expires_at: GetCurrentTimestamp() + AUTH_CODE_TTL,
        used: false
    }

    // Step 5: Store authorization code
    Cache.setWithTTL(
        key="auth_code:" + auth_code,
        value=code_record,
        ttl=AUTH_CODE_TTL
    )

    // Step 6: Delete PKCE session (one-time use)
    Cache.delete("pkce_session:" + state)

    // Step 7: Audit log
    AuditLog.record(
        event="oauth_authorization_granted",
        client_id=session.client_id,
        user_id=user_id,
        scopes=approved_scopes,
        severity="info"
    )

    // Step 8: Redirect to client with code
    redirect_url ← BuildURL(
        base=session.redirect_uri,
        params={
            code: auth_code,
            state: state
        }
    )

    RETURN redirect_url
END
```

**Time Complexity**: O(n) where n = number of scopes
**Space Complexity**: O(1)

---

## Algorithm 3: Token Exchange

```
ALGORITHM: ExchangeCodeForTokens
INPUT: code (string), code_verifier (string), client_id (string), redirect_uri (URI)
OUTPUT: token_pair (TokenPair) or error

CONSTANTS:
    ACCESS_TOKEN_TTL = 3600 seconds (1 hour)
    REFRESH_TOKEN_TTL = 2592000 seconds (30 days)
    MAX_CODE_EXCHANGE_ATTEMPTS = 3

BEGIN
    // Step 1: Retrieve authorization code
    code_record ← Cache.get("auth_code:" + code)

    IF code_record is null THEN
        AuditLog.record(
            event="oauth_invalid_code",
            code=HashSHA256(code),
            client_id=client_id,
            severity="warning"
        )
        RETURN error("Invalid or expired authorization code")
    END IF

    // Step 2: Validate code not already used (replay attack prevention)
    IF code_record.used is true THEN
        // CRITICAL: Authorization code reuse detected
        AuditLog.record(
            event="oauth_code_reuse_detected",
            code=HashSHA256(code),
            client_id=client_id,
            user_id=code_record.user_id,
            severity="critical"
        )

        // Revoke all tokens for this user + client
        RevokeAllTokens(code_record.user_id, client_id)

        Cache.delete("auth_code:" + code)
        RETURN error("Authorization code already used")
    END IF

    // Step 3: Validate code verifier (PKCE)
    computed_challenge ← Base64URLEncode(SHA256(code_verifier))

    IF computed_challenge != code_record.code_challenge THEN
        AuditLog.record(
            event="oauth_pkce_validation_failed",
            code=HashSHA256(code),
            client_id=client_id,
            severity="critical"
        )

        // Mark code as used to prevent further attempts
        code_record.used ← true
        Cache.set("auth_code:" + code, code_record)

        RETURN error("Invalid code verifier")
    END IF

    // Step 4: Validate client_id and redirect_uri match
    IF code_record.client_id != client_id THEN
        AuditLog.record(
            event="oauth_client_mismatch",
            expected=code_record.client_id,
            received=client_id,
            severity="critical"
        )
        RETURN error("Client ID mismatch")
    END IF

    IF code_record.redirect_uri != redirect_uri THEN
        AuditLog.record(
            event="oauth_redirect_uri_mismatch",
            expected=code_record.redirect_uri,
            received=redirect_uri,
            severity="critical"
        )
        RETURN error("Redirect URI mismatch")
    END IF

    // Step 5: Validate expiration
    IF GetCurrentTimestamp() > code_record.expires_at THEN
        Cache.delete("auth_code:" + code)
        AuditLog.record(
            event="oauth_code_expired",
            code=HashSHA256(code),
            severity="info"
        )
        RETURN error("Authorization code expired")
    END IF

    // Step 6: Generate access token (JWT)
    access_token ← GenerateJWT(
        claims={
            sub: code_record.user_id,
            aud: client_id,
            scope: join(code_record.scopes, " "),
            iat: GetCurrentTimestamp(),
            exp: GetCurrentTimestamp() + ACCESS_TOKEN_TTL,
            jti: GenerateUUID(),
            token_type: "access"
        },
        algorithm="RS256",
        key=GetCurrentSigningKey()
    )

    // Step 7: Generate refresh token
    refresh_token_id ← GenerateUUID()
    refresh_token ← GenerateCryptographicString(64)

    // Step 8: Store refresh token in database
    Database.insert("refresh_tokens", {
        id: refresh_token_id,
        token_hash: HashSHA256(refresh_token),
        user_id: code_record.user_id,
        client_id: client_id,
        scopes: code_record.scopes,
        family_id: GenerateUUID(), // For token rotation tracking
        created_at: GetCurrentTimestamp(),
        expires_at: GetCurrentTimestamp() + REFRESH_TOKEN_TTL,
        revoked: false,
        parent_token_id: null
    })

    // Step 9: Mark authorization code as used
    code_record.used ← true
    Cache.set("auth_code:" + code, code_record)

    // Step 10: Audit log
    AuditLog.record(
        event="oauth_tokens_issued",
        user_id=code_record.user_id,
        client_id=client_id,
        scopes=code_record.scopes,
        access_token_jti=access_token.claims.jti,
        refresh_token_id=refresh_token_id,
        severity="info"
    )

    // Step 11: Return token pair
    RETURN TokenPair{
        access_token: access_token,
        refresh_token: refresh_token,
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_TTL,
        scope: join(code_record.scopes, " ")
    }
END
```

**Time Complexity**: O(1) for token generation and validation
**Space Complexity**: O(1)

---

## Security Best Practices

### 1. PKCE Parameters
- **Code Verifier**: 64 bytes minimum (RFC 7636 recommends 43-128 characters)
- **Code Challenge Method**: Always use S256 (SHA-256), never plain
- **Character Set**: Unreserved characters only [A-Z, a-z, 0-9, -, ., _, ~]

### 2. State Parameter
- **Length**: Minimum 32 bytes of cryptographic randomness
- **Purpose**: CSRF protection
- **Storage**: Server-side session with short TTL (10 minutes)

### 3. Authorization Code Security
- **One-Time Use**: Must be invalidated after first use
- **Replay Attack Detection**: Revoke all tokens if reuse detected
- **Short Expiry**: Maximum 10 minutes (RFC 6749 recommends < 10 minutes)
- **Binding**: Tied to client_id, redirect_uri, and code_challenge

### 4. Audit Logging
Log the following events:
- `oauth_flow_initiated` - Authorization started
- `oauth_authorization_granted` - User approved scopes
- `oauth_tokens_issued` - Tokens successfully generated
- `oauth_code_reuse_detected` - CRITICAL: Replay attack
- `oauth_pkce_validation_failed` - CRITICAL: Invalid code verifier
- `oauth_invalid_client` - Unknown client_id
- `oauth_scope_escalation_attempt` - CRITICAL: Scope manipulation

### 5. Rate Limiting
Apply rate limits to:
- Authorization endpoint: 10 requests/minute per IP
- Token endpoint: 5 requests/minute per client_id
- Failed PKCE validation: 3 failures locks client for 15 minutes

---

## Complexity Analysis

### Space Complexity
- **Session Storage**: O(1) per authorization flow
- **Code Storage**: O(1) per authorization code
- **Token Storage**: O(n) where n = number of active refresh tokens per user

### Time Complexity
- **InitiateOAuthFlow**: O(n) where n = code_verifier length
- **GrantAuthorization**: O(s) where s = number of scopes
- **ExchangeCodeForTokens**: O(1) with indexed database lookups

### Database Indexes Required
```sql
CREATE INDEX idx_refresh_tokens_user_client ON refresh_tokens(user_id, client_id);
CREATE INDEX idx_refresh_tokens_family ON refresh_tokens(family_id);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
```

---

## Error Handling

### Error Response Format (RFC 6749)
```json
{
  "error": "invalid_request",
  "error_description": "Code verifier does not match code challenge",
  "error_uri": "https://docs.example.com/oauth/errors#invalid_request"
}
```

### Standard Error Codes
- `invalid_request` - Malformed request
- `invalid_client` - Client authentication failed
- `invalid_grant` - Authorization code invalid/expired/revoked
- `unauthorized_client` - Client not authorized for grant type
- `unsupported_grant_type` - Grant type not supported
- `invalid_scope` - Requested scope invalid/unknown/malformed

---

**Algorithm Designed By**: Security Algorithm Design Agent
**SPARC Phase**: Pseudocode
**Last Updated**: 2025-12-06
