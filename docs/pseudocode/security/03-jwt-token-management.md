# JWT Token Generation and Validation - Pseudocode Specification

## Overview
JSON Web Tokens (JWT) provide stateless authentication with cryptographic signatures. This implementation uses RS256 (RSA signatures with SHA-256) with automatic key rotation for maximum security.

---

## Data Structures

```
STRUCTURE JWTHeader:
    alg: String = "RS256"
    typ: String = "JWT"
    kid: String (Key ID for rotation)

STRUCTURE JWTClaims:
    // Standard claims (RFC 7519)
    sub: String (Subject - user_id)
    aud: String (Audience - client_id)
    iss: String (Issuer - "https://auth.example.com")
    exp: Integer (Expiration timestamp)
    iat: Integer (Issued at timestamp)
    nbf: Integer (Not before timestamp, optional)
    jti: String (JWT ID, unique identifier)

    // Custom claims
    scope: String (Space-separated scopes)
    token_type: Enum["access", "refresh_hint"] // "refresh_hint" for refresh token metadata
    grant_type: String (e.g., "authorization_code", "device_code")
    user_role: String (e.g., "admin", "user")

STRUCTURE SigningKey:
    kid: String (Key ID)
    private_key: RSAPrivateKey (2048-bit minimum)
    public_key: RSAPublicKey
    algorithm: String = "RS256"
    created_at: Timestamp
    expires_at: Timestamp (created_at + 90 days)
    status: Enum["active", "rotating", "retired"]

STRUCTURE ValidationResult:
    valid: Boolean
    claims: JWTClaims (if valid)
    error: String (if invalid)
    error_code: Enum["EXPIRED", "INVALID_SIGNATURE", "INVALID_AUDIENCE", "NOT_YET_VALID", "MALFORMED"]
```

---

## Algorithm 1: JWT Generation

```
ALGORITHM: GenerateJWT
INPUT: claims (JWTClaims), algorithm (string), key (SigningKey)
OUTPUT: jwt_token (string) or error

CONSTANTS:
    ISSUER = "https://auth.example.com"
    MAX_TOKEN_SIZE = 8192 bytes // Prevent oversized tokens

BEGIN
    // Step 1: Validate inputs
    IF claims.sub is empty THEN
        RETURN error("Subject (sub) claim is required")
    END IF

    IF claims.aud is empty THEN
        RETURN error("Audience (aud) claim is required")
    END IF

    IF claims.exp is empty OR claims.exp <= GetCurrentTimestamp() THEN
        RETURN error("Expiration (exp) must be in the future")
    END IF

    // Step 2: Add standard claims
    claims.iss ← ISSUER
    claims.iat ← GetCurrentTimestamp()

    IF claims.jti is empty THEN
        claims.jti ← GenerateUUID()
    END IF

    // Optional: Not Before claim (nbf) for time-delayed activation
    IF claims.nbf is empty THEN
        claims.nbf ← claims.iat
    END IF

    // Step 3: Build header
    header ← JWTHeader{
        alg: algorithm,
        typ: "JWT",
        kid: key.kid
    }

    // Step 4: Encode header and claims
    header_json ← JSONStringify(header)
    claims_json ← JSONStringify(claims)

    header_b64 ← Base64URLEncode(header_json)
    claims_b64 ← Base64URLEncode(claims_json)

    // Step 5: Create signing input
    signing_input ← header_b64 + "." + claims_b64

    // Step 6: Generate signature
    SWITCH algorithm:
        CASE "RS256":
            signature ← RSA_Sign(
                data=signing_input,
                private_key=key.private_key,
                hash_algorithm="SHA256"
            )
        CASE "HS256":
            // HMAC SHA-256 (not recommended for distributed systems)
            signature ← HMAC_SHA256(
                data=signing_input,
                secret=key.secret
            )
        DEFAULT:
            RETURN error("Unsupported algorithm: " + algorithm)
    END SWITCH

    signature_b64 ← Base64URLEncode(signature)

    // Step 7: Assemble JWT
    jwt_token ← signing_input + "." + signature_b64

    // Step 8: Validate token size
    IF Length(jwt_token) > MAX_TOKEN_SIZE THEN
        AuditLog.record(
            event="jwt_oversized_token",
            token_size=Length(jwt_token),
            user_id=claims.sub,
            severity="warning"
        )
        RETURN error("Token size exceeds maximum allowed")
    END IF

    // Step 9: Audit log (do not log full token!)
    AuditLog.record(
        event="jwt_generated",
        jti=claims.jti,
        sub=claims.sub,
        aud=claims.aud,
        exp=claims.exp,
        kid=key.kid,
        severity="debug"
    )

    RETURN jwt_token
END
```

**Time Complexity**: O(n) where n = size of claims object
**Space Complexity**: O(n)

---

## Algorithm 2: JWT Validation

```
ALGORITHM: ValidateJWT
INPUT: jwt_token (string), expected_audience (string)
OUTPUT: validation_result (ValidationResult)

CONSTANTS:
    CLOCK_SKEW_TOLERANCE = 60 seconds // Allow 60s time drift

BEGIN
    // Step 1: Parse JWT structure
    parts ← Split(jwt_token, ".")

    IF Length(parts) != 3 THEN
        AuditLog.record(
            event="jwt_malformed",
            severity="warning"
        )
        RETURN ValidationResult{
            valid: false,
            error: "Malformed JWT: expected 3 parts",
            error_code: "MALFORMED"
        }
    END IF

    header_b64 ← parts[0]
    claims_b64 ← parts[1]
    signature_b64 ← parts[2]

    // Step 2: Decode header and claims
    TRY:
        header_json ← Base64URLDecode(header_b64)
        header ← JSONParse(header_json)

        claims_json ← Base64URLDecode(claims_b64)
        claims ← JSONParse(claims_json)
    CATCH exception:
        AuditLog.record(
            event="jwt_decode_failed",
            error=exception.message,
            severity="warning"
        )
        RETURN ValidationResult{
            valid: false,
            error: "Failed to decode JWT: " + exception.message,
            error_code: "MALFORMED"
        }
    END TRY

    // Step 3: Validate header
    IF header.alg NOT IN ["RS256", "HS256"] THEN
        AuditLog.record(
            event="jwt_unsupported_algorithm",
            algorithm=header.alg,
            severity="warning"
        )
        RETURN ValidationResult{
            valid: false,
            error: "Unsupported algorithm: " + header.alg,
            error_code: "MALFORMED"
        }
    END IF

    // Step 4: Retrieve signing key by kid
    IF header.kid is empty THEN
        RETURN ValidationResult{
            valid: false,
            error: "Missing key ID (kid) in header",
            error_code: "MALFORMED"
        }
    END IF

    signing_key ← GetPublicKeyByKID(header.kid)

    IF signing_key is null THEN
        AuditLog.record(
            event="jwt_unknown_key_id",
            kid=header.kid,
            severity="warning"
        )
        RETURN ValidationResult{
            valid: false,
            error: "Unknown key ID: " + header.kid,
            error_code: "INVALID_SIGNATURE"
        }
    END IF

    // Step 5: Verify signature
    signing_input ← header_b64 + "." + claims_b64
    signature ← Base64URLDecode(signature_b64)

    signature_valid ← RSA_Verify(
        data=signing_input,
        signature=signature,
        public_key=signing_key.public_key,
        hash_algorithm="SHA256"
    )

    IF NOT signature_valid THEN
        AuditLog.record(
            event="jwt_invalid_signature",
            kid=header.kid,
            jti=claims.jti,
            severity="critical"
        )
        RETURN ValidationResult{
            valid: false,
            error: "Invalid signature",
            error_code: "INVALID_SIGNATURE"
        }
    END IF

    // Step 6: Validate standard claims
    current_time ← GetCurrentTimestamp()

    // Check expiration (exp)
    IF claims.exp is empty THEN
        RETURN ValidationResult{
            valid: false,
            error: "Missing expiration claim (exp)",
            error_code: "MALFORMED"
        }
    END IF

    IF current_time > (claims.exp + CLOCK_SKEW_TOLERANCE) THEN
        AuditLog.record(
            event="jwt_expired",
            jti=claims.jti,
            exp=claims.exp,
            current_time=current_time,
            severity="info"
        )
        RETURN ValidationResult{
            valid: false,
            error: "Token expired at " + FormatTimestamp(claims.exp),
            error_code: "EXPIRED"
        }
    END IF

    // Check not before (nbf)
    IF claims.nbf is not empty THEN
        IF current_time < (claims.nbf - CLOCK_SKEW_TOLERANCE) THEN
            AuditLog.record(
                event="jwt_not_yet_valid",
                jti=claims.jti,
                nbf=claims.nbf,
                current_time=current_time,
                severity="warning"
            )
            RETURN ValidationResult{
                valid: false,
                error: "Token not yet valid until " + FormatTimestamp(claims.nbf),
                error_code: "NOT_YET_VALID"
            }
        END IF
    END IF

    // Check audience (aud)
    IF claims.aud != expected_audience THEN
        AuditLog.record(
            event="jwt_invalid_audience",
            jti=claims.jti,
            expected=expected_audience,
            received=claims.aud,
            severity="critical"
        )
        RETURN ValidationResult{
            valid: false,
            error: "Invalid audience: expected " + expected_audience,
            error_code: "INVALID_AUDIENCE"
        }
    END IF

    // Check issuer (iss)
    IF claims.iss != ISSUER THEN
        AuditLog.record(
            event="jwt_invalid_issuer",
            jti=claims.jti,
            expected=ISSUER,
            received=claims.iss,
            severity="critical"
        )
        RETURN ValidationResult{
            valid: false,
            error: "Invalid issuer",
            error_code: "MALFORMED"
        }
    END IF

    // Step 7: Check token revocation (optional, for critical operations)
    IF IsTokenRevoked(claims.jti) THEN
        AuditLog.record(
            event="jwt_revoked_token_used",
            jti=claims.jti,
            sub=claims.sub,
            severity="critical"
        )
        RETURN ValidationResult{
            valid: false,
            error: "Token has been revoked",
            error_code: "REVOKED"
        }
    END IF

    // Step 8: Success
    AuditLog.record(
        event="jwt_validated",
        jti=claims.jti,
        sub=claims.sub,
        severity="debug"
    )

    RETURN ValidationResult{
        valid: true,
        claims: claims,
        error: null,
        error_code: null
    }
END

SUBROUTINE: IsTokenRevoked
INPUT: jti (string)
OUTPUT: revoked (boolean)

BEGIN
    // Check Redis blacklist for revoked JTIs
    revoked ← Cache.exists("revoked_jti:" + jti)
    RETURN revoked
END
```

**Time Complexity**: O(1) with indexed key lookup
**Space Complexity**: O(1)

---

## Algorithm 3: Key Rotation

```
ALGORITHM: RotateSigningKeys
INPUT: none
OUTPUT: new_key (SigningKey)

CONSTANTS:
    KEY_SIZE = 2048 bits // RSA key size
    KEY_LIFETIME = 7776000 seconds (90 days)
    ROTATION_OVERLAP = 86400 seconds (24 hours) // Grace period for old keys

BEGIN
    // Step 1: Get current active key
    current_key ← Database.findOne("signing_keys", {status: "active"})

    // Step 2: Check if rotation needed
    IF current_key is not null THEN
        time_until_expiry ← current_key.expires_at - GetCurrentTimestamp()

        IF time_until_expiry > ROTATION_OVERLAP THEN
            // No rotation needed yet
            RETURN current_key
        END IF
    END IF

    // Step 3: Generate new RSA key pair
    key_pair ← RSA_GenerateKeyPair(
        key_size=KEY_SIZE,
        public_exponent=65537
    )

    // Step 4: Create new signing key
    new_kid ← GenerateUUID()

    new_key ← SigningKey{
        kid: new_kid,
        private_key: key_pair.private_key,
        public_key: key_pair.public_key,
        algorithm: "RS256",
        created_at: GetCurrentTimestamp(),
        expires_at: GetCurrentTimestamp() + KEY_LIFETIME,
        status: "active"
    }

    // Step 5: Store new key
    Database.insert("signing_keys", new_key)

    // Step 6: Mark old key as rotating (keep for verification during overlap)
    IF current_key is not null THEN
        current_key.status ← "rotating"
        Database.update("signing_keys", current_key)
    END IF

    // Step 7: Schedule cleanup of old key after overlap period
    Schedule(
        delay=ROTATION_OVERLAP,
        task=RetireSigningKey(current_key.kid)
    )

    // Step 8: Audit log
    AuditLog.record(
        event="signing_key_rotated",
        new_kid=new_kid,
        old_kid=current_key?.kid,
        severity="info"
    )

    // Step 9: Invalidate public key cache
    Cache.delete("jwks") // Force refresh of JSON Web Key Set

    RETURN new_key
END

SUBROUTINE: RetireSigningKey
INPUT: kid (string)
OUTPUT: success

BEGIN
    key ← Database.findOne("signing_keys", {kid: kid})

    IF key is null THEN
        RETURN error("Key not found")
    END IF

    key.status ← "retired"
    Database.update("signing_keys", key)

    AuditLog.record(
        event="signing_key_retired",
        kid=kid,
        severity="info"
    )

    RETURN success
END
```

**Time Complexity**: O(1) for database operations, O(n) for RSA key generation where n = key size
**Space Complexity**: O(1)

---

## Algorithm 4: Public Key Discovery (JWKS Endpoint)

```
ALGORITHM: GetJSONWebKeySet
INPUT: none
OUTPUT: jwks (JSON Web Key Set)

BEGIN
    // Step 1: Check cache
    cached_jwks ← Cache.get("jwks")

    IF cached_jwks is not null THEN
        RETURN cached_jwks
    END IF

    // Step 2: Retrieve all active and rotating keys
    keys ← Database.find("signing_keys", {
        status: {$in: ["active", "rotating"]}
    })

    // Step 3: Build JWKS
    jwks ← {
        keys: []
    }

    FOR EACH key IN keys DO
        // Export public key in JWK format
        jwk ← {
            kty: "RSA",
            use: "sig",
            kid: key.kid,
            alg: key.algorithm,
            n: Base64URLEncode(key.public_key.modulus),
            e: Base64URLEncode(key.public_key.exponent)
        }

        jwks.keys.append(jwk)
    END FOR

    // Step 4: Cache JWKS for 1 hour
    Cache.setWithTTL(
        key="jwks",
        value=jwks,
        ttl=3600
    )

    RETURN jwks
END
```

**Time Complexity**: O(k) where k = number of active keys (typically 1-2)
**Space Complexity**: O(k)

---

## Security Best Practices

### 1. Algorithm Selection
- **RS256 (RSA + SHA-256)**: Recommended for distributed systems
- **HS256 (HMAC + SHA-256)**: Only for single-server deployments
- **NEVER use "none" algorithm**: Always validate `alg` header

### 2. Key Management
- **Key Size**: Minimum 2048 bits for RSA (3072+ recommended for high security)
- **Rotation**: Every 90 days with 24-hour overlap
- **Storage**: Private keys MUST be stored in HSM or encrypted key vault
- **Separation**: Different keys for different environments (dev, staging, prod)

### 3. Token Lifetime
- **Access Tokens**: 1 hour (short-lived)
- **Refresh Tokens**: 30 days (long-lived, stored in database)
- **ID Tokens**: 5 minutes (OpenID Connect)

### 4. Claims Validation
Always validate:
- `exp` (expiration) - CRITICAL
- `aud` (audience) - CRITICAL
- `iss` (issuer) - CRITICAL
- `nbf` (not before) - If present
- Signature - CRITICAL

### 5. Token Revocation
```
ALGORITHM: RevokeToken
INPUT: jti (string), reason (string)
OUTPUT: success

BEGIN
    // Add to Redis blacklist with TTL matching token expiration
    token_claims ← ParseJWT(jti) // Get from request context

    ttl ← token_claims.exp - GetCurrentTimestamp()

    IF ttl > 0 THEN
        Cache.setWithTTL(
            key="revoked_jti:" + jti,
            value=reason,
            ttl=ttl
        )
    END IF

    AuditLog.record(
        event="jwt_revoked",
        jti=jti,
        reason=reason,
        severity="warning"
    )

    RETURN success
END
```

### 6. Clock Skew Tolerance
- Allow 60 seconds of clock drift between servers
- Use NTP to synchronize server clocks
- Apply tolerance to both `exp` and `nbf` checks

---

## Complexity Analysis

### Time Complexity
- **GenerateJWT**: O(n) where n = claims size
- **ValidateJWT**: O(1) with cached public keys
- **RotateSigningKeys**: O(k) where k = key size (2048 bits)
- **GetJSONWebKeySet**: O(m) where m = number of active keys

### Space Complexity
- **JWT Storage**: O(n) where n = claims size (typically 500-2000 bytes)
- **Key Storage**: O(k) where k = number of active keys (1-2)
- **Revocation List**: O(r) where r = number of revoked tokens

### Database Indexes
```sql
CREATE UNIQUE INDEX idx_signing_keys_kid ON signing_keys(kid);
CREATE INDEX idx_signing_keys_status ON signing_keys(status);
CREATE INDEX idx_signing_keys_expires_at ON signing_keys(expires_at);
```

---

**Algorithm Designed By**: Security Algorithm Design Agent
**SPARC Phase**: Pseudocode
**Standards**: RFC 7519 (JWT), RFC 7517 (JWK), RFC 7518 (JWA)
**Last Updated**: 2025-12-06
