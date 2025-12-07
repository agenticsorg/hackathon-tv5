# Refresh Token Rotation - Pseudocode Specification

## Overview
Refresh token rotation implements one-time use refresh tokens with automatic rotation on each use. This prevents token replay attacks and enables detection of compromised tokens through token family tracking.

---

## Data Structures

```
STRUCTURE RefreshToken:
    id: UUID
    token_hash: String (SHA-256 hash of actual token)
    user_id: String
    client_id: String
    scopes: Array<String>
    family_id: UUID (tracks token lineage)
    parent_token_id: UUID (null for initial token)
    grant_type: String (e.g., "authorization_code", "device_code")
    created_at: Timestamp
    expires_at: Timestamp
    revoked: Boolean
    revoked_at: Timestamp (null if not revoked)
    revocation_reason: String
    last_used_at: Timestamp
    use_count: Integer (should always be 0 or 1)
    ip_address: String (last used IP)
    user_agent: String (last used user agent)

STRUCTURE TokenFamily:
    family_id: UUID
    user_id: String
    client_id: String
    root_token_id: UUID
    created_at: Timestamp
    last_rotation_at: Timestamp
    rotation_count: Integer
    status: Enum["active", "revoked", "expired"]
    revocation_reason: String
```

---

## Algorithm 1: Issue Initial Refresh Token

```
ALGORITHM: IssueInitialRefreshToken
INPUT: user_id (string), client_id (string), scopes (array), grant_type (string)
OUTPUT: refresh_token (string) and token_id (UUID)

CONSTANTS:
    REFRESH_TOKEN_LENGTH = 64
    REFRESH_TOKEN_TTL = 2592000 seconds (30 days)

BEGIN
    // Step 1: Generate refresh token
    refresh_token ← GenerateCryptographicString(
        length=REFRESH_TOKEN_LENGTH,
        charset=ALLOWED_CHARACTERS
    )

    token_hash ← SHA256(refresh_token)

    // Step 2: Create token family
    family_id ← GenerateUUID()
    token_id ← GenerateUUID()

    // Step 3: Store token family
    Database.insert("token_families", {
        family_id: family_id,
        user_id: user_id,
        client_id: client_id,
        root_token_id: token_id,
        created_at: GetCurrentTimestamp(),
        last_rotation_at: GetCurrentTimestamp(),
        rotation_count: 0,
        status: "active",
        revocation_reason: null
    })

    // Step 4: Store refresh token
    Database.insert("refresh_tokens", {
        id: token_id,
        token_hash: token_hash,
        user_id: user_id,
        client_id: client_id,
        scopes: scopes,
        family_id: family_id,
        parent_token_id: null, // Root token has no parent
        grant_type: grant_type,
        created_at: GetCurrentTimestamp(),
        expires_at: GetCurrentTimestamp() + REFRESH_TOKEN_TTL,
        revoked: false,
        revoked_at: null,
        revocation_reason: null,
        last_used_at: null,
        use_count: 0,
        ip_address: null,
        user_agent: null
    })

    // Step 5: Audit log
    AuditLog.record(
        event="refresh_token_issued",
        token_id=token_id,
        family_id=family_id,
        user_id=user_id,
        client_id=client_id,
        grant_type=grant_type,
        severity="info"
    )

    RETURN {
        refresh_token: refresh_token,
        token_id: token_id
    }
END
```

**Time Complexity**: O(1)
**Space Complexity**: O(1)

---

## Algorithm 2: Rotate Refresh Token

```
ALGORITHM: RotateRefreshToken
INPUT: refresh_token (string), client_id (string), ip_address (string), user_agent (string)
OUTPUT: new_tokens (TokenPair) or error

CONSTANTS:
    ACCESS_TOKEN_TTL = 3600 seconds (1 hour)
    REFRESH_TOKEN_TTL = 2592000 seconds (30 days)
    MAX_ROTATION_COUNT = 100 // Prevent infinite rotation chains

BEGIN
    // Step 1: Hash and lookup refresh token
    token_hash ← SHA256(refresh_token)

    old_token ← Database.findOne("refresh_tokens", {
        token_hash: token_hash
    })

    IF old_token is null THEN
        AuditLog.record(
            event="refresh_token_not_found",
            token_hash_prefix=token_hash[0:8],
            client_id=client_id,
            ip_address=ip_address,
            severity="warning"
        )
        RETURN error("invalid_grant", "Invalid refresh token")
    END IF

    // Step 2: Validate client_id matches
    IF old_token.client_id != client_id THEN
        AuditLog.record(
            event="refresh_token_client_mismatch",
            token_id=old_token.id,
            expected_client=old_token.client_id,
            received_client=client_id,
            severity="critical"
        )
        RETURN error("invalid_client", "Client ID mismatch")
    END IF

    // Step 3: Check token family status
    token_family ← Database.findOne("token_families", {
        family_id: old_token.family_id
    })

    IF token_family.status != "active" THEN
        AuditLog.record(
            event="refresh_token_revoked_family",
            family_id=old_token.family_id,
            token_id=old_token.id,
            reason=token_family.revocation_reason,
            severity="critical"
        )
        RETURN error("invalid_grant", "Token family revoked: " + token_family.revocation_reason)
    END IF

    // Step 4: CRITICAL - Detect token reuse (replay attack)
    IF old_token.use_count > 0 OR old_token.revoked is true THEN
        AuditLog.record(
            event="refresh_token_reuse_detected",
            token_id=old_token.id,
            family_id=old_token.family_id,
            user_id=old_token.user_id,
            client_id=client_id,
            ip_address=ip_address,
            previous_ip=old_token.ip_address,
            severity="critical"
        )

        // IMMEDIATELY revoke entire token family
        RevokeTokenFamily(
            family_id=old_token.family_id,
            reason="Token reuse detected (possible theft)",
            revoked_by="system"
        )

        // Notify user of potential compromise
        NotifyUser(
            user_id=old_token.user_id,
            event="security_alert_token_reuse",
            message="Potential security breach detected. All sessions revoked."
        )

        RETURN error("invalid_grant", "Token reuse detected. All tokens revoked for security.")
    END IF

    // Step 5: Validate expiration
    IF GetCurrentTimestamp() > old_token.expires_at THEN
        AuditLog.record(
            event="refresh_token_expired",
            token_id=old_token.id,
            expired_at=old_token.expires_at,
            severity="info"
        )
        RETURN error("invalid_grant", "Refresh token expired")
    END IF

    // Step 6: Validate rotation count (prevent infinite chains)
    IF token_family.rotation_count >= MAX_ROTATION_COUNT THEN
        AuditLog.record(
            event="refresh_token_max_rotations",
            family_id=old_token.family_id,
            rotation_count=token_family.rotation_count,
            severity="warning"
        )

        RevokeTokenFamily(
            family_id=old_token.family_id,
            reason="Maximum rotation count exceeded",
            revoked_by="system"
        )

        RETURN error("invalid_grant", "Token rotation limit reached. Please re-authenticate.")
    END IF

    // Step 7: Mark old token as used (atomic operation)
    update_result ← Database.updateOne(
        "refresh_tokens",
        filter={
            id: old_token.id,
            use_count: 0 // Ensure atomic update (race condition protection)
        },
        update={
            $set: {
                use_count: 1,
                last_used_at: GetCurrentTimestamp(),
                ip_address: ip_address,
                user_agent: user_agent,
                revoked: true,
                revoked_at: GetCurrentTimestamp(),
                revocation_reason: "Rotated to new token"
            }
        }
    )

    IF update_result.modified_count == 0 THEN
        // Race condition: token was used by another request
        AuditLog.record(
            event="refresh_token_race_condition",
            token_id=old_token.id,
            family_id=old_token.family_id,
            severity="critical"
        )

        RevokeTokenFamily(
            family_id=old_token.family_id,
            reason="Concurrent token use detected",
            revoked_by="system"
        )

        RETURN error("invalid_grant", "Token already used. All tokens revoked for security.")
    END IF

    // Step 8: Generate new refresh token
    new_refresh_token ← GenerateCryptographicString(64)
    new_token_hash ← SHA256(new_refresh_token)
    new_token_id ← GenerateUUID()

    // Step 9: Store new refresh token
    Database.insert("refresh_tokens", {
        id: new_token_id,
        token_hash: new_token_hash,
        user_id: old_token.user_id,
        client_id: old_token.client_id,
        scopes: old_token.scopes,
        family_id: old_token.family_id,
        parent_token_id: old_token.id, // Track lineage
        grant_type: old_token.grant_type,
        created_at: GetCurrentTimestamp(),
        expires_at: GetCurrentTimestamp() + REFRESH_TOKEN_TTL,
        revoked: false,
        revoked_at: null,
        revocation_reason: null,
        last_used_at: null,
        use_count: 0,
        ip_address: null,
        user_agent: null
    })

    // Step 10: Update token family
    Database.updateOne("token_families", {
        family_id: old_token.family_id
    }, {
        $set: {
            last_rotation_at: GetCurrentTimestamp()
        },
        $inc: {
            rotation_count: 1
        }
    })

    // Step 11: Generate new access token
    access_token ← GenerateJWT(
        claims={
            sub: old_token.user_id,
            aud: old_token.client_id,
            scope: join(old_token.scopes, " "),
            iat: GetCurrentTimestamp(),
            exp: GetCurrentTimestamp() + ACCESS_TOKEN_TTL,
            jti: GenerateUUID(),
            token_type: "access",
            grant_type: "refresh_token"
        },
        algorithm="RS256",
        key=GetCurrentSigningKey()
    )

    // Step 12: Audit log
    AuditLog.record(
        event="refresh_token_rotated",
        old_token_id=old_token.id,
        new_token_id=new_token_id,
        family_id=old_token.family_id,
        user_id=old_token.user_id,
        rotation_count=token_family.rotation_count + 1,
        ip_address=ip_address,
        severity="info"
    )

    // Step 13: Return new token pair
    RETURN TokenPair{
        access_token: access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_TTL,
        scope: join(old_token.scopes, " ")
    }
END
```

**Time Complexity**: O(1) with indexed database queries
**Space Complexity**: O(1)

---

## Algorithm 3: Revoke Token Family

```
ALGORITHM: RevokeTokenFamily
INPUT: family_id (UUID), reason (string), revoked_by (string)
OUTPUT: revoked_count (integer)

BEGIN
    // Step 1: Mark token family as revoked
    Database.updateOne("token_families", {
        family_id: family_id
    }, {
        $set: {
            status: "revoked",
            revocation_reason: reason
        }
    })

    // Step 2: Revoke all tokens in family
    result ← Database.updateMany("refresh_tokens", {
        family_id: family_id,
        revoked: false
    }, {
        $set: {
            revoked: true,
            revoked_at: GetCurrentTimestamp(),
            revocation_reason: reason
        }
    })

    // Step 3: Revoke all active access tokens (add JTIs to blacklist)
    active_sessions ← Database.find("user_sessions", {
        family_id: family_id,
        expires_at: {$gt: GetCurrentTimestamp()}
    })

    FOR EACH session IN active_sessions DO
        // Add access token JTI to Redis blacklist
        ttl ← session.access_token_expires_at - GetCurrentTimestamp()

        IF ttl > 0 THEN
            Cache.setWithTTL(
                key="revoked_jti:" + session.access_token_jti,
                value=reason,
                ttl=ttl
            )
        END IF

        // Delete session
        Database.delete("user_sessions", {id: session.id})
    END FOR

    // Step 4: Audit log
    AuditLog.record(
        event="token_family_revoked",
        family_id=family_id,
        revoked_count=result.modified_count,
        reason=reason,
        revoked_by=revoked_by,
        severity="warning"
    )

    RETURN result.modified_count
END
```

**Time Complexity**: O(n) where n = number of tokens in family
**Space Complexity**: O(1)

---

## Algorithm 4: Token Family Lineage Tracking

```
ALGORITHM: GetTokenFamilyLineage
INPUT: token_id (UUID)
OUTPUT: lineage (array of RefreshToken)

BEGIN
    lineage ← []
    current_token_id ← token_id

    // Traverse up to root token
    WHILE current_token_id is not null DO
        token ← Database.findOne("refresh_tokens", {
            id: current_token_id
        })

        IF token is null THEN
            BREAK
        END IF

        lineage.append(token)
        current_token_id ← token.parent_token_id

        // Safety check: prevent infinite loops
        IF lineage.length > 1000 THEN
            AuditLog.record(
                event="token_lineage_too_deep",
                token_id=token_id,
                severity="critical"
            )
            BREAK
        END IF
    END WHILE

    // Reverse to show root-to-current order
    lineage.reverse()

    RETURN lineage
END
```

**Time Complexity**: O(d) where d = depth of token chain
**Space Complexity**: O(d)

---

## Algorithm 5: Automatic Token Cleanup

```
ALGORITHM: CleanupExpiredTokens
INPUT: batch_size (integer)
OUTPUT: deleted_count (integer)

BEGIN
    current_time ← GetCurrentTimestamp()

    // Step 1: Find expired tokens
    expired_tokens ← Database.find(
        "refresh_tokens",
        filter={
            expires_at: {$lt: current_time}
        },
        limit=batch_size
    )

    deleted_count ← 0

    FOR EACH token IN expired_tokens DO
        // Step 2: Check if entire family is expired
        family ← Database.findOne("token_families", {
            family_id: token.family_id
        })

        family_tokens ← Database.count("refresh_tokens", {
            family_id: token.family_id,
            expires_at: {$gte: current_time}
        })

        // Step 3: If all tokens in family expired, delete family
        IF family_tokens == 0 THEN
            Database.updateOne("token_families", {
                family_id: token.family_id
            }, {
                $set: {
                    status: "expired"
                }
            })
        END IF

        // Step 4: Delete expired token
        Database.delete("refresh_tokens", {
            id: token.id
        })

        deleted_count ← deleted_count + 1
    END FOR

    // Step 5: Audit log
    IF deleted_count > 0 THEN
        AuditLog.record(
            event="refresh_tokens_cleaned",
            deleted_count=deleted_count,
            severity="info"
        )
    END IF

    RETURN deleted_count
END
```

**Time Complexity**: O(n) where n = batch_size
**Space Complexity**: O(n)

---

## Security Best Practices

### 1. Token Reuse Detection
- **Critical**: ALWAYS check `use_count` before rotating
- **Atomic Update**: Use database atomic operations to prevent race conditions
- **Immediate Revocation**: Revoke entire token family on reuse detection
- **User Notification**: Alert user of potential security breach

### 2. Token Family Tracking
- **Lineage**: Every token knows its parent (`parent_token_id`)
- **Family ID**: All tokens in rotation chain share same `family_id`
- **Root Token**: First token in family has `parent_token_id = null`
- **Forensics**: Complete audit trail for security investigations

### 3. Rotation Limits
- **Max Rotations**: 100 rotations per family (prevents infinite chains)
- **Expiration**: 30 days for each token (forces periodic re-authentication)
- **Grace Period**: No grace period for expired tokens

### 4. Concurrent Use Protection
```
// Use database atomic operations for race condition prevention
UPDATE refresh_tokens
SET use_count = 1, revoked = true
WHERE id = ? AND use_count = 0
```

### 5. Storage Security
- **Hash Tokens**: NEVER store plaintext refresh tokens
- **SHA-256**: Use SHA-256 for token hashing
- **Indexed Hashes**: Index `token_hash` for fast lookups

### 6. Audit Events
Log the following:
- `refresh_token_issued` - New token created
- `refresh_token_rotated` - Successful rotation
- `refresh_token_reuse_detected` - CRITICAL: Replay attack
- `refresh_token_race_condition` - CRITICAL: Concurrent use
- `token_family_revoked` - Entire family revoked
- `refresh_tokens_cleaned` - Expired tokens deleted

---

## Revocation Scenarios

### Scenario 1: User Logout
```
ALGORITHM: LogoutUser
INPUT: user_id, client_id
OUTPUT: success

BEGIN
    // Revoke all token families for user + client
    families ← Database.find("token_families", {
        user_id: user_id,
        client_id: client_id,
        status: "active"
    })

    FOR EACH family IN families DO
        RevokeTokenFamily(
            family_id=family.family_id,
            reason="User logout",
            revoked_by=user_id
        )
    END FOR

    RETURN success
END
```

### Scenario 2: Account Compromise
```
ALGORITHM: RevokeAllUserTokens
INPUT: user_id, reason
OUTPUT: revoked_count

BEGIN
    // Revoke ALL token families for user (all clients)
    families ← Database.find("token_families", {
        user_id: user_id,
        status: "active"
    })

    revoked_count ← 0

    FOR EACH family IN families DO
        RevokeTokenFamily(
            family_id=family.family_id,
            reason=reason,
            revoked_by="admin"
        )
        revoked_count ← revoked_count + 1
    END FOR

    // Force password reset
    Database.update("users", {id: user_id}, {
        $set: {
            password_reset_required: true,
            password_reset_reason: reason
        }
    })

    RETURN revoked_count
END
```

### Scenario 3: Client Revocation
```
ALGORITHM: RevokeClientTokens
INPUT: client_id, reason
OUTPUT: revoked_count

BEGIN
    // Revoke all token families for client (all users)
    families ← Database.find("token_families", {
        client_id: client_id,
        status: "active"
    })

    revoked_count ← 0

    FOR EACH family IN families DO
        RevokeTokenFamily(
            family_id=family.family_id,
            reason=reason,
            revoked_by="admin"
        )
        revoked_count ← revoked_count + 1
    END FOR

    RETURN revoked_count
END
```

---

## Complexity Analysis

### Time Complexity
- **IssueInitialRefreshToken**: O(1)
- **RotateRefreshToken**: O(1) with indexed queries
- **RevokeTokenFamily**: O(n) where n = tokens in family
- **GetTokenFamilyLineage**: O(d) where d = chain depth
- **CleanupExpiredTokens**: O(b) where b = batch size

### Space Complexity
- **Per Token**: O(1) - Fixed size record
- **Per Family**: O(n) where n = number of rotations
- **Lineage Tracking**: O(d) where d = chain depth

### Database Indexes
```sql
-- Critical indexes for performance
CREATE UNIQUE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_family ON refresh_tokens(family_id);
CREATE INDEX idx_refresh_tokens_user_client ON refresh_tokens(user_id, client_id);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);

CREATE INDEX idx_token_families_user ON token_families(user_id);
CREATE INDEX idx_token_families_client ON token_families(client_id);
CREATE INDEX idx_token_families_status ON token_families(status);
```

---

## Monitoring and Alerts

### Critical Alerts
1. **Token Reuse Detection** - Immediate alert to security team
2. **High Rotation Count** - Alert at 80% of max (80 rotations)
3. **Multiple Family Revocations** - Possible attack pattern
4. **Race Condition Detection** - Database integrity issue

### Metrics to Track
- Average rotation count per family
- Token reuse detection rate
- Time between rotations
- Family revocation rate
- Cleanup efficiency

---

**Algorithm Designed By**: Security Algorithm Design Agent
**SPARC Phase**: Pseudocode
**Security Model**: Zero-trust with token rotation
**Last Updated**: 2025-12-06
