# Platform Token Management (YouTube OAuth) - Pseudocode Specification

## Overview
Platform token management handles external OAuth provider tokens (YouTube, Twitch, etc.) with secure storage, automatic refresh, and real-time revocation propagation via PubNub.

---

## Data Structures

```
STRUCTURE PlatformToken:
    id: UUID
    user_id: String
    platform: Enum["youtube", "twitch", "facebook", "tiktok"]
    access_token_encrypted: String (AES-256-GCM encrypted)
    refresh_token_encrypted: String (AES-256-GCM encrypted)
    token_type: String (typically "Bearer")
    scopes: Array<String>
    expires_at: Timestamp
    created_at: Timestamp
    updated_at: Timestamp
    last_refreshed_at: Timestamp
    revoked: Boolean
    revoked_at: Timestamp
    revoked_reason: String
    encryption_key_id: String (for key rotation)
    iv: String (initialization vector for AES-GCM)
    auth_tag: String (GCM authentication tag)

STRUCTURE PlatformTokenMetadata:
    platform_user_id: String (YouTube channel ID, etc.)
    platform_username: String
    platform_email: String
    channel_name: String
    profile_picture_url: String
    raw_token_response: Object (original OAuth response)

STRUCTURE TokenRefreshJob:
    token_id: UUID
    scheduled_at: Timestamp (expires_at - 5 minutes)
    attempts: Integer
    last_attempt_at: Timestamp
    status: Enum["pending", "in_progress", "completed", "failed"]
```

---

## Algorithm 1: Store Platform Token

```
ALGORITHM: StorePlatformToken
INPUT: user_id (string), platform (string), oauth_response (object)
OUTPUT: token_id (UUID) or error

CONSTANTS:
    ENCRYPTION_ALGORITHM = "AES-256-GCM"
    REFRESH_BUFFER = 300 seconds (5 minutes before expiry)

BEGIN
    // Step 1: Extract token data from OAuth response
    access_token ← oauth_response.access_token
    refresh_token ← oauth_response.refresh_token
    expires_in ← oauth_response.expires_in
    scopes ← oauth_response.scope.split(" ")
    token_type ← oauth_response.token_type

    IF access_token is empty THEN
        RETURN error("Missing access_token in OAuth response")
    END IF

    // Step 2: Get encryption key
    encryption_key ← GetCurrentEncryptionKey()

    // Step 3: Encrypt access token
    access_token_result ← EncryptToken(
        plaintext=access_token,
        key=encryption_key.key,
        algorithm=ENCRYPTION_ALGORITHM
    )

    // Step 4: Encrypt refresh token (if present)
    refresh_token_encrypted ← null
    refresh_token_iv ← null
    refresh_token_auth_tag ← null

    IF refresh_token is not null THEN
        refresh_token_result ← EncryptToken(
            plaintext=refresh_token,
            key=encryption_key.key,
            algorithm=ENCRYPTION_ALGORITHM
        )

        refresh_token_encrypted ← refresh_token_result.ciphertext
        refresh_token_iv ← refresh_token_result.iv
        refresh_token_auth_tag ← refresh_token_result.auth_tag
    END IF

    // Step 5: Calculate expiration
    expires_at ← GetCurrentTimestamp() + expires_in

    // Step 6: Check if user already has token for this platform
    existing_token ← Database.findOne("platform_tokens", {
        user_id: user_id,
        platform: platform
    })

    IF existing_token is not null THEN
        // Revoke old token
        RevokePlatformToken(
            token_id=existing_token.id,
            reason="Replaced by new OAuth token"
        )
    END IF

    // Step 7: Store encrypted token
    token_id ← GenerateUUID()

    Database.insert("platform_tokens", {
        id: token_id,
        user_id: user_id,
        platform: platform,
        access_token_encrypted: access_token_result.ciphertext,
        refresh_token_encrypted: refresh_token_encrypted,
        token_type: token_type,
        scopes: scopes,
        expires_at: expires_at,
        created_at: GetCurrentTimestamp(),
        updated_at: GetCurrentTimestamp(),
        last_refreshed_at: null,
        revoked: false,
        revoked_at: null,
        revoked_reason: null,
        encryption_key_id: encryption_key.id,
        iv: access_token_result.iv,
        auth_tag: access_token_result.auth_tag
    })

    // Step 8: Store metadata
    metadata ← ExtractPlatformMetadata(platform, oauth_response)

    Database.insert("platform_token_metadata", {
        token_id: token_id,
        user_id: user_id,
        platform: platform,
        platform_user_id: metadata.platform_user_id,
        platform_username: metadata.username,
        platform_email: metadata.email,
        channel_name: metadata.channel_name,
        profile_picture_url: metadata.profile_picture_url,
        raw_token_response: oauth_response
    })

    // Step 9: Schedule refresh job
    refresh_time ← expires_at - REFRESH_BUFFER

    ScheduleTokenRefresh(
        token_id=token_id,
        scheduled_at=refresh_time
    )

    // Step 10: Audit log
    AuditLog.record(
        event="platform_token_stored",
        token_id=token_id,
        user_id=user_id,
        platform=platform,
        scopes=scopes,
        expires_at=expires_at,
        severity="info"
    )

    RETURN token_id
END

SUBROUTINE: EncryptToken
INPUT: plaintext (string), key (bytes), algorithm (string)
OUTPUT: {ciphertext, iv, auth_tag}

BEGIN
    // Generate random IV (12 bytes for GCM)
    iv ← CSRNG.generateBytes(12)

    // Encrypt using AES-256-GCM
    cipher ← AES_GCM_Cipher(
        key=key,
        iv=iv
    )

    ciphertext ← cipher.encrypt(plaintext)
    auth_tag ← cipher.getAuthenticationTag()

    RETURN {
        ciphertext: Base64Encode(ciphertext),
        iv: Base64Encode(iv),
        auth_tag: Base64Encode(auth_tag)
    }
END
```

**Time Complexity**: O(n) where n = token length (for encryption)
**Space Complexity**: O(n)

---

## Algorithm 2: Retrieve Platform Token (Decrypted)

```
ALGORITHM: GetPlatformToken
INPUT: user_id (string), platform (string)
OUTPUT: decrypted_token (object) or error

BEGIN
    // Step 1: Retrieve encrypted token
    token_record ← Database.findOne("platform_tokens", {
        user_id: user_id,
        platform: platform,
        revoked: false
    })

    IF token_record is null THEN
        RETURN error("No active token found for platform: " + platform)
    END IF

    // Step 2: Check if expired
    IF GetCurrentTimestamp() > token_record.expires_at THEN
        // Attempt to refresh
        refresh_result ← RefreshPlatformToken(token_record.id)

        IF refresh_result is error THEN
            RETURN error("Token expired and refresh failed")
        END IF

        // Retrieve updated token
        token_record ← Database.findOne("platform_tokens", {
            id: token_record.id
        })
    END IF

    // Step 3: Get decryption key
    decryption_key ← GetEncryptionKeyByID(token_record.encryption_key_id)

    IF decryption_key is null THEN
        AuditLog.record(
            event="token_decryption_key_not_found",
            token_id=token_record.id,
            key_id=token_record.encryption_key_id,
            severity="critical"
        )
        RETURN error("Decryption key not found")
    END IF

    // Step 4: Decrypt access token
    access_token ← DecryptToken(
        ciphertext=Base64Decode(token_record.access_token_encrypted),
        key=decryption_key.key,
        iv=Base64Decode(token_record.iv),
        auth_tag=Base64Decode(token_record.auth_tag)
    )

    // Step 5: Decrypt refresh token (if present)
    refresh_token ← null

    IF token_record.refresh_token_encrypted is not null THEN
        refresh_token ← DecryptToken(
            ciphertext=Base64Decode(token_record.refresh_token_encrypted),
            key=decryption_key.key,
            iv=Base64Decode(token_record.iv),
            auth_tag=Base64Decode(token_record.auth_tag)
        )
    END IF

    // Step 6: Audit log (do not log decrypted tokens!)
    AuditLog.record(
        event="platform_token_retrieved",
        token_id=token_record.id,
        user_id=user_id,
        platform=platform,
        severity="debug"
    )

    // Step 7: Return decrypted token
    RETURN {
        access_token: access_token,
        refresh_token: refresh_token,
        token_type: token_record.token_type,
        scopes: token_record.scopes,
        expires_at: token_record.expires_at
    }
END

SUBROUTINE: DecryptToken
INPUT: ciphertext (bytes), key (bytes), iv (bytes), auth_tag (bytes)
OUTPUT: plaintext (string)

BEGIN
    TRY:
        cipher ← AES_GCM_Cipher(
            key=key,
            iv=iv
        )

        cipher.setAuthenticationTag(auth_tag)
        plaintext ← cipher.decrypt(ciphertext)

        RETURN plaintext
    CATCH exception:
        AuditLog.record(
            event="token_decryption_failed",
            error=exception.message,
            severity="critical"
        )
        THROW error("Token decryption failed")
    END TRY
END
```

**Time Complexity**: O(n) where n = token length
**Space Complexity**: O(n)

---

## Algorithm 3: Refresh Platform Token

```
ALGORITHM: RefreshPlatformToken
INPUT: token_id (UUID)
OUTPUT: success or error

CONSTANTS:
    YOUTUBE_TOKEN_ENDPOINT = "https://oauth2.googleapis.com/token"
    MAX_REFRESH_ATTEMPTS = 3

BEGIN
    // Step 1: Retrieve token record
    token_record ← Database.findOne("platform_tokens", {
        id: token_id
    })

    IF token_record is null THEN
        RETURN error("Token not found")
    END IF

    IF token_record.revoked is true THEN
        RETURN error("Token is revoked")
    END IF

    // Step 2: Check if refresh token exists
    IF token_record.refresh_token_encrypted is null THEN
        AuditLog.record(
            event="platform_token_no_refresh_token",
            token_id=token_id,
            platform=token_record.platform,
            severity="error"
        )
        RETURN error("No refresh token available")
    END IF

    // Step 3: Decrypt refresh token
    decryption_key ← GetEncryptionKeyByID(token_record.encryption_key_id)
    refresh_token ← DecryptToken(
        ciphertext=Base64Decode(token_record.refresh_token_encrypted),
        key=decryption_key.key,
        iv=Base64Decode(token_record.iv),
        auth_tag=Base64Decode(token_record.auth_tag)
    )

    // Step 4: Get platform OAuth config
    oauth_config ← GetPlatformOAuthConfig(token_record.platform)

    // Step 5: Make refresh token request
    TRY:
        response ← HTTP_POST(
            url=oauth_config.token_endpoint,
            headers={
                "Content-Type": "application/x-www-form-urlencoded"
            },
            body={
                grant_type: "refresh_token",
                refresh_token: refresh_token,
                client_id: oauth_config.client_id,
                client_secret: oauth_config.client_secret
            }
        )

        IF response.status_code != 200 THEN
            AuditLog.record(
                event="platform_token_refresh_failed",
                token_id=token_id,
                platform=token_record.platform,
                status_code=response.status_code,
                error=response.body,
                severity="error"
            )

            // Check if token is permanently invalid
            IF response.status_code == 400 AND response.body.error == "invalid_grant" THEN
                RevokePlatformToken(
                    token_id=token_id,
                    reason="Refresh token invalid (revoked by user or expired)"
                )
            END IF

            RETURN error("Token refresh failed: " + response.body.error)
        END IF

        oauth_response ← ParseJSON(response.body)

    CATCH exception:
        AuditLog.record(
            event="platform_token_refresh_error",
            token_id=token_id,
            error=exception.message,
            severity="error"
        )
        RETURN error("Token refresh request failed")
    END TRY

    // Step 6: Encrypt new access token
    encryption_key ← GetCurrentEncryptionKey()

    access_token_result ← EncryptToken(
        plaintext=oauth_response.access_token,
        key=encryption_key.key,
        algorithm=ENCRYPTION_ALGORITHM
    )

    // Step 7: Update token in database
    new_expires_at ← GetCurrentTimestamp() + oauth_response.expires_in

    Database.updateOne("platform_tokens", {
        id: token_id
    }, {
        $set: {
            access_token_encrypted: access_token_result.ciphertext,
            expires_at: new_expires_at,
            updated_at: GetCurrentTimestamp(),
            last_refreshed_at: GetCurrentTimestamp(),
            encryption_key_id: encryption_key.id,
            iv: access_token_result.iv,
            auth_tag: access_token_result.auth_tag
        }
    })

    // Step 8: Update refresh token if rotated (some platforms rotate refresh tokens)
    IF oauth_response.refresh_token is not null THEN
        refresh_token_result ← EncryptToken(
            plaintext=oauth_response.refresh_token,
            key=encryption_key.key,
            algorithm=ENCRYPTION_ALGORITHM
        )

        Database.updateOne("platform_tokens", {
            id: token_id
        }, {
            $set: {
                refresh_token_encrypted: refresh_token_result.ciphertext
            }
        })
    END IF

    // Step 9: Schedule next refresh
    next_refresh_time ← new_expires_at - REFRESH_BUFFER

    ScheduleTokenRefresh(
        token_id=token_id,
        scheduled_at=next_refresh_time
    )

    // Step 10: Audit log
    AuditLog.record(
        event="platform_token_refreshed",
        token_id=token_id,
        user_id=token_record.user_id,
        platform=token_record.platform,
        new_expires_at=new_expires_at,
        severity="info"
    )

    RETURN success
END
```

**Time Complexity**: O(n) where n = token length (encryption overhead)
**Space Complexity**: O(n)

---

## Algorithm 4: Revoke Platform Token with PubNub Propagation

```
ALGORITHM: RevokePlatformToken
INPUT: token_id (UUID), reason (string)
OUTPUT: success

CONSTANTS:
    PUBNUB_CHANNEL_PREFIX = "platform_token_revocation"

BEGIN
    // Step 1: Retrieve token
    token_record ← Database.findOne("platform_tokens", {
        id: token_id
    })

    IF token_record is null THEN
        RETURN error("Token not found")
    END IF

    // Step 2: Mark as revoked in database
    Database.updateOne("platform_tokens", {
        id: token_id
    }, {
        $set: {
            revoked: true,
            revoked_at: GetCurrentTimestamp(),
            revoked_reason: reason,
            updated_at: GetCurrentTimestamp()
        }
    })

    // Step 3: Publish revocation event to PubNub
    pubnub_channel ← PUBNUB_CHANNEL_PREFIX + "." + token_record.user_id

    pubnub_message ← {
        event: "platform_token_revoked",
        token_id: token_id,
        user_id: token_record.user_id,
        platform: token_record.platform,
        reason: reason,
        revoked_at: GetCurrentTimestamp()
    }

    TRY:
        PubNub.publish(
            channel=pubnub_channel,
            message=pubnub_message
        )
    CATCH exception:
        AuditLog.record(
            event="pubnub_publish_failed",
            error=exception.message,
            severity="warning"
        )
        // Continue even if PubNub fails (DB is source of truth)
    END TRY

    // Step 4: Cancel scheduled refresh job
    Database.delete("token_refresh_jobs", {
        token_id: token_id
    })

    // Step 5: Audit log
    AuditLog.record(
        event="platform_token_revoked",
        token_id=token_id,
        user_id=token_record.user_id,
        platform=token_record.platform,
        reason=reason,
        severity="warning"
    )

    RETURN success
END
```

**Time Complexity**: O(1)
**Space Complexity**: O(1)

---

## Algorithm 5: Background Token Refresh Job

```
ALGORITHM: ProcessTokenRefreshQueue
INPUT: none
OUTPUT: processed_count (integer)

CONSTANTS:
    BATCH_SIZE = 50
    RETRY_DELAY = 300 seconds (5 minutes)

BEGIN
    current_time ← GetCurrentTimestamp()

    // Step 1: Get tokens scheduled for refresh
    jobs ← Database.find("token_refresh_jobs", {
        scheduled_at: {$lte: current_time},
        status: {$in: ["pending", "failed"]},
        attempts: {$lt: MAX_REFRESH_ATTEMPTS}
    }).limit(BATCH_SIZE)

    processed_count ← 0

    FOR EACH job IN jobs DO
        // Step 2: Mark as in progress
        Database.updateOne("token_refresh_jobs", {
            id: job.id
        }, {
            $set: {
                status: "in_progress",
                last_attempt_at: current_time
            },
            $inc: {
                attempts: 1
            }
        })

        // Step 3: Attempt refresh
        result ← RefreshPlatformToken(job.token_id)

        IF result is success THEN
            // Step 4a: Mark job as completed
            Database.updateOne("token_refresh_jobs", {
                id: job.id
            }, {
                $set: {
                    status: "completed"
                }
            })

            processed_count ← processed_count + 1
        ELSE
            // Step 4b: Mark as failed
            Database.updateOne("token_refresh_jobs", {
                id: job.id
            }, {
                $set: {
                    status: "failed"
                }
            })

            // Step 5: Schedule retry if attempts remaining
            IF job.attempts + 1 < MAX_REFRESH_ATTEMPTS THEN
                Database.updateOne("token_refresh_jobs", {
                    id: job.id
                }, {
                    $set: {
                        scheduled_at: current_time + RETRY_DELAY,
                        status: "pending"
                    }
                })
            ELSE
                // Max attempts reached - notify user
                NotifyUser(
                    user_id=job.user_id,
                    event="platform_token_refresh_failed",
                    message="Unable to refresh " + job.platform + " token. Please re-authenticate."
                )
            END IF
        END IF
    END FOR

    RETURN processed_count
END
```

**Time Complexity**: O(b) where b = batch size
**Space Complexity**: O(b)

---

## Algorithm 6: Encryption Key Rotation

```
ALGORITHM: RotateEncryptionKeys
INPUT: none
OUTPUT: new_key_id (string)

CONSTANTS:
    KEY_SIZE = 32 bytes (256 bits for AES-256)
    KEY_LIFETIME = 7776000 seconds (90 days)

BEGIN
    // Step 1: Generate new encryption key
    new_key ← CSRNG.generateBytes(KEY_SIZE)
    key_id ← GenerateUUID()

    // Step 2: Store new key in secure key vault
    KeyVault.store(
        id=key_id,
        key=new_key,
        algorithm="AES-256-GCM",
        created_at=GetCurrentTimestamp(),
        expires_at=GetCurrentTimestamp() + KEY_LIFETIME,
        status="active"
    )

    // Step 3: Mark old key as rotating (keep for decryption)
    current_key ← KeyVault.getCurrentKey()

    IF current_key is not null THEN
        KeyVault.updateStatus(
            id=current_key.id,
            status="rotating"
        )

        // Step 4: Schedule background re-encryption of tokens with old key
        ScheduleTokenReEncryption(
            old_key_id=current_key.id,
            new_key_id=key_id
        )
    END IF

    // Step 5: Audit log
    AuditLog.record(
        event="encryption_key_rotated",
        new_key_id=key_id,
        old_key_id=current_key?.id,
        severity="info"
    )

    RETURN key_id
END
```

**Time Complexity**: O(1)
**Space Complexity**: O(1)

---

## Platform-Specific OAuth Endpoints

### YouTube (Google OAuth)
```
OAUTH_CONFIG: youtube
    authorization_endpoint: "https://accounts.google.com/o/oauth2/v2/auth"
    token_endpoint: "https://oauth2.googleapis.com/token"
    revocation_endpoint: "https://oauth2.googleapis.com/revoke"
    scopes: [
        "https://www.googleapis.com/auth/youtube.readonly",
        "https://www.googleapis.com/auth/youtube.upload",
        "https://www.googleapis.com/auth/youtube.force-ssl"
    ]
    token_lifetime: 3600 seconds (1 hour)
    supports_refresh: true
```

---

## Security Best Practices

### 1. Encryption
- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Storage**: Hardware Security Module (HSM) or managed key vault
- **Key Rotation**: Every 90 days
- **IV**: Unique random IV for each encryption operation (12 bytes for GCM)

### 2. Token Refresh
- **Buffer Time**: Refresh 5 minutes before expiration
- **Retry Logic**: 3 attempts with 5-minute delay
- **Background Jobs**: Scheduled task every 1 minute
- **Failure Handling**: Notify user after max retries

### 3. Revocation Propagation
- **Real-time**: PubNub for instant notification to connected clients
- **Channels**: Per-user channels (`platform_token_revocation.{user_id}`)
- **Fallback**: Database is source of truth if PubNub fails

### 4. Audit Logging
- `platform_token_stored` - New token encrypted and stored
- `platform_token_retrieved` - Token decrypted for use
- `platform_token_refreshed` - Access token refreshed
- `platform_token_revoked` - Token revoked
- `token_decryption_failed` - CRITICAL: Decryption error
- `platform_token_refresh_failed` - Refresh attempt failed

---

## Complexity Analysis

### Time Complexity
- **StorePlatformToken**: O(n) where n = token length
- **GetPlatformToken**: O(n) with automatic refresh if expired
- **RefreshPlatformToken**: O(n) + O(HTTP request)
- **RevokePlatformToken**: O(1)
- **ProcessTokenRefreshQueue**: O(b × n) where b = batch size

### Space Complexity
- **Encrypted Storage**: O(n + overhead) where overhead ≈ 64 bytes (IV + auth tag)
- **Key Vault**: O(k) where k = number of active encryption keys (typically 1-2)

### Database Indexes
```sql
CREATE INDEX idx_platform_tokens_user_platform ON platform_tokens(user_id, platform);
CREATE INDEX idx_platform_tokens_expires_at ON platform_tokens(expires_at);
CREATE INDEX idx_platform_tokens_revoked ON platform_tokens(revoked);

CREATE INDEX idx_token_refresh_jobs_scheduled ON token_refresh_jobs(scheduled_at, status);
```

---

**Algorithm Designed By**: Security Algorithm Design Agent
**SPARC Phase**: Pseudocode
**Encryption**: AES-256-GCM with key rotation
**Last Updated**: 2025-12-06
