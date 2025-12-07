# Device Authorization Grant (RFC 8628) - Pseudocode Specification

## Overview
Device Authorization Grant enables OAuth 2.0 authorization on devices with limited input capabilities (smart TVs, IoT devices, CLI tools) by delegating user authentication to a secondary device with better input methods.

---

## Data Structures

```
STRUCTURE DeviceCode:
    device_code: String (32 bytes, cryptographically random)
    user_code: String (8 characters, XXXX-XXXX format)
    verification_uri: URI
    verification_uri_complete: URI (with user_code embedded)
    expires_in: Integer (900 seconds = 15 minutes)
    interval: Integer (5 seconds, polling interval)
    client_id: String
    scopes: Array<String>
    created_at: Timestamp
    expires_at: Timestamp
    status: Enum[PENDING, AUTHORIZED, DENIED, EXPIRED]
    user_id: String (null until authorized)

STRUCTURE DeviceTokenResponse:
    access_token: JWT
    refresh_token: String
    token_type: String = "Bearer"
    expires_in: Integer
    scope: String
```

---

## Algorithm 1: Generate Device Code

```
ALGORITHM: GenerateDeviceCode
INPUT: client_id (string), scopes (array of strings)
OUTPUT: device_code_response (DeviceCode) or error

CONSTANTS:
    DEVICE_CODE_LENGTH = 32
    USER_CODE_LENGTH = 8
    USER_CODE_CHARSET = [A-Z, 0-9] excluding [I, O, 0, 1] for clarity
    DEVICE_CODE_TTL = 900 seconds (15 minutes)
    POLLING_INTERVAL = 5 seconds
    VERIFICATION_URI = "https://example.com/device"

BEGIN
    // Step 1: Validate client
    client ← Database.findClientByID(client_id)

    IF client is null THEN
        AuditLog.record(
            event="device_flow_invalid_client",
            client_id=client_id,
            severity="warning"
        )
        RETURN error("invalid_client", "Unknown client_id")
    END IF

    // Step 2: Verify client supports device flow
    IF "urn:ietf:params:oauth:grant-type:device_code" NOT IN client.grant_types THEN
        AuditLog.record(
            event="device_flow_unauthorized_grant",
            client_id=client_id,
            severity="warning"
        )
        RETURN error("unauthorized_client", "Device flow not enabled for this client")
    END IF

    // Step 3: Validate scopes
    FOR EACH scope IN scopes DO
        IF scope NOT IN client.allowed_scopes THEN
            AuditLog.record(
                event="device_flow_invalid_scope",
                client_id=client_id,
                invalid_scope=scope,
                severity="warning"
            )
            RETURN error("invalid_scope", "Scope not allowed: " + scope)
        END IF
    END FOR

    // Step 4: Generate device code
    device_code ← GenerateCryptographicString(
        length=DEVICE_CODE_LENGTH,
        charset=ALLOWED_CHARACTERS
    )

    // Step 5: Generate user-friendly user code
    user_code ← GenerateUserCode()

    // Step 6: Build verification URIs
    verification_uri ← VERIFICATION_URI
    verification_uri_complete ← VERIFICATION_URI + "?user_code=" + user_code

    // Step 7: Create device code record
    device_record ← DeviceCode{
        device_code: device_code,
        user_code: user_code,
        verification_uri: verification_uri,
        verification_uri_complete: verification_uri_complete,
        expires_in: DEVICE_CODE_TTL,
        interval: POLLING_INTERVAL,
        client_id: client_id,
        scopes: scopes,
        created_at: GetCurrentTimestamp(),
        expires_at: GetCurrentTimestamp() + DEVICE_CODE_TTL,
        status: "PENDING",
        user_id: null
    }

    // Step 8: Store device code (indexed by both device_code and user_code)
    Cache.setWithTTL(
        key="device_code:" + device_code,
        value=device_record,
        ttl=DEVICE_CODE_TTL
    )

    Cache.setWithTTL(
        key="user_code:" + user_code,
        value=device_code,
        ttl=DEVICE_CODE_TTL
    )

    // Step 9: Audit log
    AuditLog.record(
        event="device_code_generated",
        client_id=client_id,
        user_code=user_code,
        device_code_hash=HashSHA256(device_code),
        scopes=scopes,
        severity="info"
    )

    // Step 10: Return response (RFC 8628 format)
    RETURN {
        device_code: device_code,
        user_code: user_code,
        verification_uri: verification_uri,
        verification_uri_complete: verification_uri_complete,
        expires_in: DEVICE_CODE_TTL,
        interval: POLLING_INTERVAL
    }
END

SUBROUTINE: GenerateUserCode
OUTPUT: user_code (string in XXXX-XXXX format)

CONSTANTS:
    USER_CODE_CHARSET = [A, B, C, D, E, F, G, H, J, K, L, M, N, P, Q, R, S, T, U, V, W, X, Y, Z, 2, 3, 4, 5, 6, 7, 8, 9]
    // Excludes I, O, 0, 1 to prevent visual confusion

BEGIN
    // Generate 8 random characters
    code ← ""
    FOR i = 1 TO 8 DO
        random_byte ← CSRNG.generateByte()
        index ← random_byte MOD USER_CODE_CHARSET.length
        code ← code + USER_CODE_CHARSET[index]
    END FOR

    // Format as XXXX-XXXX for readability
    formatted_code ← code[0:4] + "-" + code[4:8]

    // Ensure uniqueness (extremely rare collision)
    WHILE Cache.exists("user_code:" + formatted_code) DO
        // Regenerate if collision detected
        code ← GenerateUserCode()
    END WHILE

    RETURN formatted_code
END
```

**Time Complexity**: O(1) average case, O(n) worst case with collision retries
**Space Complexity**: O(1)

---

## Algorithm 2: User Authorization (Secondary Device)

```
ALGORITHM: AuthorizeDeviceCode
INPUT: user_code (string), user_id (string), approved_scopes (array of strings)
OUTPUT: success or error

BEGIN
    // Step 1: Normalize user code (remove dashes, uppercase)
    normalized_code ← RemoveDashes(ToUpperCase(user_code))
    formatted_code ← normalized_code[0:4] + "-" + normalized_code[4:8]

    // Step 2: Retrieve device code from user code
    device_code_key ← Cache.get("user_code:" + formatted_code)

    IF device_code_key is null THEN
        AuditLog.record(
            event="device_flow_invalid_user_code",
            user_code=formatted_code,
            user_id=user_id,
            severity="warning"
        )
        RETURN error("Invalid or expired user code")
    END IF

    // Step 3: Retrieve device code record
    device_record ← Cache.get("device_code:" + device_code_key)

    IF device_record is null THEN
        RETURN error("Device code expired")
    END IF

    // Step 4: Validate device code not already processed
    IF device_record.status != "PENDING" THEN
        AuditLog.record(
            event="device_flow_code_already_processed",
            user_code=formatted_code,
            status=device_record.status,
            severity="warning"
        )
        RETURN error("Device code already " + device_record.status)
    END IF

    // Step 5: Validate expiration
    IF GetCurrentTimestamp() > device_record.expires_at THEN
        device_record.status ← "EXPIRED"
        Cache.set("device_code:" + device_code_key, device_record)

        AuditLog.record(
            event="device_flow_code_expired",
            user_code=formatted_code,
            severity="info"
        )
        RETURN error("Device code expired")
    END IF

    // Step 6: Validate approved scopes are subset of requested
    FOR EACH scope IN approved_scopes DO
        IF scope NOT IN device_record.scopes THEN
            AuditLog.record(
                event="device_flow_scope_escalation",
                user_code=formatted_code,
                user_id=user_id,
                invalid_scope=scope,
                severity="critical"
            )
            RETURN error("Scope escalation detected")
        END IF
    END FOR

    // Step 7: Update device code with authorization
    device_record.status ← "AUTHORIZED"
    device_record.user_id ← user_id
    device_record.scopes ← approved_scopes // Update with approved subset

    Cache.set("device_code:" + device_code_key, device_record)

    // Step 8: Audit log
    AuditLog.record(
        event="device_code_authorized",
        user_code=formatted_code,
        user_id=user_id,
        client_id=device_record.client_id,
        scopes=approved_scopes,
        severity="info"
    )

    RETURN success("Device authorized successfully")
END
```

**Time Complexity**: O(n) where n = number of scopes
**Space Complexity**: O(1)

---

## Algorithm 3: Token Polling (Device)

```
ALGORITHM: PollForToken
INPUT: device_code (string), client_id (string)
OUTPUT: token_pair (DeviceTokenResponse) or pending/error

CONSTANTS:
    ACCESS_TOKEN_TTL = 3600 seconds (1 hour)
    REFRESH_TOKEN_TTL = 2592000 seconds (30 days)

BEGIN
    // Step 1: Retrieve device code record
    device_record ← Cache.get("device_code:" + device_code)

    IF device_record is null THEN
        AuditLog.record(
            event="device_flow_invalid_device_code",
            device_code_hash=HashSHA256(device_code),
            client_id=client_id,
            severity="warning"
        )
        RETURN error("expired_token", "Device code expired or invalid")
    END IF

    // Step 2: Validate client_id matches
    IF device_record.client_id != client_id THEN
        AuditLog.record(
            event="device_flow_client_mismatch",
            expected=device_record.client_id,
            received=client_id,
            severity="critical"
        )
        RETURN error("invalid_client", "Client ID mismatch")
    END IF

    // Step 3: Check for slow down (rate limiting)
    rate_limit_key ← "device_poll_rate:" + device_code
    last_poll_time ← Cache.get(rate_limit_key)

    IF last_poll_time is not null THEN
        time_since_last_poll ← GetCurrentTimestamp() - last_poll_time

        IF time_since_last_poll < device_record.interval THEN
            // Client polling too fast
            AuditLog.record(
                event="device_flow_slow_down",
                device_code_hash=HashSHA256(device_code),
                interval_violation=device_record.interval - time_since_last_poll,
                severity="info"
            )
            RETURN error("slow_down", "Polling too frequently")
        END IF
    END IF

    // Step 4: Update last poll time
    Cache.setWithTTL(
        key=rate_limit_key,
        value=GetCurrentTimestamp(),
        ttl=device_record.interval * 2
    )

    // Step 5: Check device code status
    SWITCH device_record.status:
        CASE "PENDING":
            // Still waiting for user authorization
            RETURN {
                error: "authorization_pending",
                error_description: "User has not yet authorized the device"
            }

        CASE "DENIED":
            // User explicitly denied authorization
            Cache.delete("device_code:" + device_code)
            Cache.delete("user_code:" + device_record.user_code)

            AuditLog.record(
                event="device_flow_access_denied",
                device_code_hash=HashSHA256(device_code),
                user_id=device_record.user_id,
                severity="info"
            )

            RETURN error("access_denied", "User denied authorization")

        CASE "EXPIRED":
            Cache.delete("device_code:" + device_code)
            Cache.delete("user_code:" + device_record.user_code)

            RETURN error("expired_token", "Device code expired")

        CASE "AUTHORIZED":
            // Proceed to token generation
            BREAK
    END SWITCH

    // Step 6: Generate access token (JWT)
    access_token ← GenerateJWT(
        claims={
            sub: device_record.user_id,
            aud: client_id,
            scope: join(device_record.scopes, " "),
            iat: GetCurrentTimestamp(),
            exp: GetCurrentTimestamp() + ACCESS_TOKEN_TTL,
            jti: GenerateUUID(),
            token_type: "access",
            grant_type: "device_code"
        },
        algorithm="RS256",
        key=GetCurrentSigningKey()
    )

    // Step 7: Generate refresh token
    refresh_token_id ← GenerateUUID()
    refresh_token ← GenerateCryptographicString(64)

    // Step 8: Store refresh token
    Database.insert("refresh_tokens", {
        id: refresh_token_id,
        token_hash: HashSHA256(refresh_token),
        user_id: device_record.user_id,
        client_id: client_id,
        scopes: device_record.scopes,
        family_id: GenerateUUID(),
        grant_type: "device_code",
        created_at: GetCurrentTimestamp(),
        expires_at: GetCurrentTimestamp() + REFRESH_TOKEN_TTL,
        revoked: false,
        parent_token_id: null
    })

    // Step 9: Delete device code (one-time use)
    Cache.delete("device_code:" + device_code)
    Cache.delete("user_code:" + device_record.user_code)
    Cache.delete(rate_limit_key)

    // Step 10: Audit log
    AuditLog.record(
        event="device_flow_tokens_issued",
        user_id=device_record.user_id,
        client_id=client_id,
        scopes=device_record.scopes,
        access_token_jti=access_token.claims.jti,
        refresh_token_id=refresh_token_id,
        severity="info"
    )

    // Step 11: Return token pair
    RETURN DeviceTokenResponse{
        access_token: access_token,
        refresh_token: refresh_token,
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_TTL,
        scope: join(device_record.scopes, " ")
    }
END
```

**Time Complexity**: O(1)
**Space Complexity**: O(1)

---

## User Code Display Format

### Visual Design for Device Display
```
┌─────────────────────────────────────┐
│  Authorize This Device              │
├─────────────────────────────────────┤
│                                     │
│  1. Visit: example.com/device       │
│                                     │
│  2. Enter code:                     │
│                                     │
│     ┌──────┬──────┐                │
│     │ ABCD │ EF12 │                │
│     └──────┴──────┘                │
│                                     │
│  Code expires in 14:35              │
│                                     │
└─────────────────────────────────────┘
```

### QR Code Alternative
```
ALGORITHM: GenerateQRCode
INPUT: verification_uri_complete (URI)
OUTPUT: qr_code_image (binary)

BEGIN
    // Generate QR code containing the complete verification URI
    qr_code ← QRCodeLibrary.encode(
        data=verification_uri_complete,
        error_correction="M", // Medium (15% recovery)
        version="auto",
        size=200 // pixels
    )

    RETURN qr_code
END
```

---

## Security Best Practices

### 1. User Code Design
- **Length**: 8 characters minimum
- **Character Set**: Exclude visually ambiguous characters (I/1, O/0)
- **Format**: XXXX-XXXX with hyphen for readability
- **Case Insensitive**: Normalize to uppercase
- **Collision Handling**: Regenerate on conflict (extremely rare)

### 2. Polling Behavior
- **Interval**: 5 seconds (RFC 8628 recommendation)
- **Slow Down**: Return `slow_down` error if polling < interval
- **Rate Limiting**: Track last poll time per device_code
- **Max Duration**: 15 minutes before expiration

### 3. Expiration Handling
- **Device Code TTL**: 15 minutes (900 seconds)
- **User Code TTL**: Same as device code
- **Cleanup**: Automatic deletion on expiry or token issuance

### 4. Status Transitions
```
PENDING → AUTHORIZED → (tokens issued)
PENDING → DENIED → (deleted)
PENDING → EXPIRED → (deleted)
```

### 5. Audit Events
- `device_code_generated` - Device flow initiated
- `device_code_authorized` - User approved on secondary device
- `device_flow_tokens_issued` - Tokens successfully generated
- `device_flow_access_denied` - User denied authorization
- `device_flow_slow_down` - Client polling too fast
- `device_flow_code_expired` - Code expired before authorization

---

## Error Codes (RFC 8628)

### Polling Errors
- `authorization_pending` - User has not yet authorized
- `slow_down` - Client polling too frequently (add 5s to interval)
- `access_denied` - User denied authorization
- `expired_token` - Device code expired

### Request Errors
- `invalid_request` - Malformed request
- `invalid_client` - Client authentication failed
- `invalid_grant` - Device code invalid
- `unauthorized_client` - Client not authorized for device flow

---

## Complexity Analysis

### Time Complexity
- **GenerateDeviceCode**: O(1) average, O(k) worst case with collisions
- **AuthorizeDeviceCode**: O(n) where n = number of scopes
- **PollForToken**: O(1)

### Space Complexity
- **Cache Storage**: O(2) per device code (device_code + user_code mappings)
- **Rate Limiting**: O(1) per active polling session

### Database Indexes
```sql
CREATE INDEX idx_refresh_tokens_device_grant ON refresh_tokens(grant_type, client_id) WHERE grant_type = 'device_code';
```

---

**Algorithm Designed By**: Security Algorithm Design Agent
**SPARC Phase**: Pseudocode
**RFC Reference**: RFC 8628 (OAuth 2.0 Device Authorization Grant)
**Last Updated**: 2025-12-06
