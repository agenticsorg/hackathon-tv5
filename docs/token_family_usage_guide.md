# Token Family Manager - Usage Guide

## Quick Start

### Initialization

```rust
use auth::{TokenFamilyManager, JwtManager, SessionManager};
use std::sync::Arc;

// Initialize managers
let redis_url = "redis://127.0.0.1/";
let token_family_manager = Arc::new(TokenFamilyManager::new(redis_url)?);
let session_manager = Arc::new(SessionManager::new(redis_url)?);
let jwt_manager = Arc::new(JwtManager::new(
    private_key,
    public_key,
    issuer,
    audience,
)?);

// Pass to server
start_server(
    bind_address,
    jwt_manager,
    session_manager,
    token_family_manager,  // Add this parameter
    oauth_config,
    storage,
).await?;
```

---

## Initial Token Issuance

When issuing a refresh token for the first time (e.g., after OAuth authorization):

```rust
// 1. Create a new token family
let family_id = token_family_manager
    .create_family(user_id.clone())
    .await?;

// 2. Generate access and refresh tokens
let access_token = jwt_manager.create_access_token(
    user_id.clone(),
    email,
    roles,
    scopes,
)?;

let refresh_token = jwt_manager.create_refresh_token_with_family(
    user_id.clone(),
    email,
    roles,
    scopes,
    family_id,  // Include family ID
)?;

// 3. Add the refresh token to the family
let refresh_claims = jwt_manager.verify_refresh_token(&refresh_token)?;
token_family_manager
    .add_token_to_family(family_id, refresh_claims.jti.clone())
    .await?;

// 4. Create session
session_manager
    .create_session(user_id, refresh_claims.jti, device_id)
    .await?;
```

---

## Refresh Token Rotation (Normal Flow)

When a client presents a refresh token to get new tokens:

```rust
async fn refresh_tokens(
    refresh_token: &str,
    jwt_manager: &JwtManager,
    token_family_manager: &TokenFamilyManager,
    session_manager: &SessionManager,
) -> Result<(String, String)> {
    // 1. Verify the refresh token
    let claims = jwt_manager.verify_refresh_token(refresh_token)?;

    // 2. Check if token is revoked
    if session_manager.is_token_revoked(&claims.jti).await? {
        return Err(AuthError::InvalidToken("Token revoked".to_string()));
    }

    // 3. Extract and verify token family
    let family_id = claims.token_family_id
        .ok_or(AuthError::InvalidToken("Missing family ID".to_string()))?;

    // 4. CRITICAL: Check if token is still in its family
    let is_in_family = token_family_manager
        .is_token_in_family(family_id, &claims.jti)
        .await?;

    if !is_in_family {
        // 🚨 SECURITY ALERT: Token reuse detected
        tracing::error!(
            user_id = %claims.sub,
            family_id = %family_id,
            attempted_jti = %claims.jti,
            "Refresh token reuse detected - revoking entire family"
        );

        // Revoke all tokens in this family
        token_family_manager.revoke_family(family_id).await?;

        return Err(AuthError::InvalidToken(
            "Token reuse detected. All tokens have been revoked.".to_string()
        ));
    }

    // 5. Generate new tokens
    let new_access_token = jwt_manager.create_access_token(
        claims.sub.clone(),
        claims.email.clone(),
        claims.roles.clone(),
        claims.scopes.clone(),
    )?;

    let new_refresh_token = jwt_manager.create_refresh_token_with_family(
        claims.sub.clone(),
        claims.email.clone(),
        claims.roles.clone(),
        claims.scopes.clone(),
        family_id,  // Same family
    )?;

    // 6. Rotate the token in the family
    // Remove old token
    token_family_manager
        .remove_token_from_family(family_id, &claims.jti)
        .await?;
    session_manager.revoke_token(&claims.jti, 3600).await?;

    // Add new token
    let new_claims = jwt_manager.verify_refresh_token(&new_refresh_token)?;
    token_family_manager
        .add_token_to_family(family_id, new_claims.jti.clone())
        .await?;

    // 7. Create new session
    session_manager
        .create_session(claims.sub, new_claims.jti, None)
        .await?;

    tracing::debug!(
        user_id = %claims.sub,
        family_id = %family_id,
        old_jti = %claims.jti,
        new_jti = %new_claims.jti,
        "Successfully rotated refresh token"
    );

    Ok((new_access_token, new_refresh_token))
}
```

---

## Manual Token Revocation

When you need to manually revoke tokens (e.g., user logout, security event):

### Revoke Entire Family

```rust
// Revoke all refresh tokens for a specific family
token_family_manager.revoke_family(family_id).await?;
```

This will:
1. Mark all JTIs in the family as revoked in Redis
2. Delete the family record
3. Force the user to re-authenticate

### Revoke All User Sessions

```rust
// This revokes all sessions but doesn't automatically revoke families
// You may want to also find and revoke all families for this user
session_manager.revoke_all_user_sessions(user_id).await?;
```

---

## Monitoring and Debugging

### Check Family Status

```rust
// Get family details
let family = token_family_manager.get_family(family_id).await?;

println!("Family ID: {}", family.family_id);
println!("User ID: {}", family.user_id);
println!("Created: {}", family.created_at);
println!("Active tokens: {}", family.active_jtis.len());

for jti in &family.active_jtis {
    println!("  - {}", jti);
}
```

### Check Token Status

```rust
// Check if a specific token is in a family
let is_valid = token_family_manager
    .is_token_in_family(family_id, jti)
    .await?;

if is_valid {
    println!("Token is valid and in family");
} else {
    println!("Token is NOT in family (revoked or rotated)");
}

// Check if token is globally revoked
let is_revoked = session_manager
    .is_token_revoked(jti)
    .await?;

if is_revoked {
    println!("Token is revoked");
}
```

---

## Error Handling

### Common Errors

```rust
match token_family_manager.is_token_in_family(family_id, jti).await {
    Ok(true) => {
        // Token is valid and in family
    }
    Ok(false) => {
        // Token not in family - possible reuse attack
        token_family_manager.revoke_family(family_id).await?;
        return Err(AuthError::InvalidToken("Token reuse detected".to_string()));
    }
    Err(AuthError::Redis(e)) => {
        // Redis connection error
        tracing::error!("Redis error: {}", e);
        return Err(AuthError::Internal("Storage unavailable".to_string()));
    }
    Err(AuthError::InvalidToken(msg)) => {
        // Token family not found or invalid
        tracing::warn!("Invalid token family: {}", msg);
        return Err(AuthError::InvalidToken(msg));
    }
    Err(e) => {
        // Other errors
        return Err(e);
    }
}
```

---

## Security Best Practices

### 1. Always Verify Family Membership

```rust
// ✅ CORRECT: Check before rotating
if !token_family_manager.is_token_in_family(family_id, jti).await? {
    token_family_manager.revoke_family(family_id).await?;
    return Err(AuthError::InvalidToken("Token reuse detected".to_string()));
}

// ❌ WRONG: Skip family check
// This defeats the security purpose!
```

### 2. Rotate, Don't Reuse

```rust
// ✅ CORRECT: Always rotate on refresh
token_family_manager.remove_token_from_family(family_id, old_jti).await?;
let new_refresh_token = jwt_manager.create_refresh_token_with_family(...)?;
let new_claims = jwt_manager.verify_refresh_token(&new_refresh_token)?;
token_family_manager.add_token_to_family(family_id, new_claims.jti).await?;

// ❌ WRONG: Return same refresh token
// return old_refresh_token;  // Don't do this!
```

### 3. Log Security Events

```rust
if !is_in_family {
    // Always log with structured fields
    tracing::error!(
        event = "token_reuse_detected",
        user_id = %claims.sub,
        family_id = %family_id,
        attempted_jti = %claims.jti,
        timestamp = %chrono::Utc::now(),
        "Refresh token reuse detected - revoking entire family"
    );

    // Optionally trigger additional alerts
    alert_security_team(SecurityEvent::TokenReuse {
        user_id: claims.sub.clone(),
        family_id,
        timestamp: chrono::Utc::now(),
    }).await;
}
```

### 4. Handle Legacy Tokens

```rust
// Support tokens issued before family tracking
let family_id = match claims.token_family_id {
    Some(id) => id,
    None => {
        // Legacy token without family
        tracing::warn!(
            user_id = %claims.sub,
            jti = %claims.jti,
            "Legacy refresh token detected - creating new family"
        );

        // Option 1: Force re-authentication
        return Err(AuthError::InvalidToken(
            "Please re-authenticate to get updated tokens".to_string()
        ));

        // Option 2: Create family retroactively (less secure)
        // let new_family_id = token_family_manager.create_family(claims.sub.clone()).await?;
        // new_family_id
    }
};
```

---

## Performance Optimization

### Batch Operations

If you need to check multiple tokens:

```rust
// Instead of sequential checks:
// for jti in jtis {
//     token_family_manager.is_token_in_family(family_id, jti).await?;
// }

// Consider fetching family once:
let family = token_family_manager.get_family(family_id).await?;
for jti in jtis {
    if family.active_jtis.contains(jti) {
        // Token is valid
    }
}
```

### Redis Connection Pooling

The `TokenFamilyManager` uses multiplexed connections. Ensure you:

1. Reuse the same manager instance (Arc)
2. Don't create new managers per request
3. Monitor connection pool exhaustion

```rust
// ✅ CORRECT: Shared instance
let token_family_manager = Arc::new(TokenFamilyManager::new(redis_url)?);
let app_state = AppState {
    token_family_manager: token_family_manager.clone(),
    // ...
};

// ❌ WRONG: New instance per request
// let mgr = TokenFamilyManager::new(redis_url)?;  // Don't do this!
```

---

## Testing

### Unit Test Example

```rust
#[tokio::test]
#[ignore] // Requires Redis
async fn test_token_rotation() {
    let manager = TokenFamilyManager::new("redis://127.0.0.1/").unwrap();
    let family_id = manager.create_family("user123".to_string()).await.unwrap();

    // Add first token
    manager.add_token_to_family(family_id, "jti1".to_string()).await.unwrap();
    assert!(manager.is_token_in_family(family_id, "jti1").await.unwrap());

    // Rotate to second token
    manager.remove_token_from_family(family_id, "jti1").await.unwrap();
    manager.add_token_to_family(family_id, "jti2".to_string()).await.unwrap();

    // Verify rotation
    assert!(!manager.is_token_in_family(family_id, "jti1").await.unwrap());
    assert!(manager.is_token_in_family(family_id, "jti2").await.unwrap());
}
```

### Integration Test Example

See `/workspaces/media-gateway/crates/auth/tests/token_family_integration_test.rs` for comprehensive examples.

---

## Troubleshooting

### "Token missing family ID (legacy token)"

**Cause**: Client is using old refresh token issued before family tracking was implemented.

**Solution**: Force user to re-authenticate to get new tokens with family tracking.

### "Token reuse detected"

**Cause**: Either:
1. Actual security attack (token theft)
2. Client bug (using old token by mistake)
3. Race condition (concurrent refresh requests)

**Solution**:
1. Log details for security analysis
2. Revoke family (already done automatically)
3. Force user to re-authenticate
4. Investigate client implementation

### High Redis Latency

**Cause**: Redis overloaded or network issues.

**Solution**:
1. Monitor Redis metrics
2. Scale Redis if needed
3. Check network latency
4. Consider Redis cluster for high load

### Family Size Growing Unexpectedly

**Cause**: Tokens not being removed during rotation.

**Solution**:
1. Verify `remove_token_from_family` is called
2. Check for error handling issues
3. Monitor family sizes in Redis

---

## Migration Checklist

When deploying to existing system:

- [ ] Backup Redis data
- [ ] Update server initialization to include `TokenFamilyManager`
- [ ] Deploy new code
- [ ] Monitor for "legacy token" errors
- [ ] Set grace period for users to refresh tokens naturally
- [ ] After grace period, consider forcing global token refresh
- [ ] Monitor security event logs for unusual patterns
- [ ] Set up alerts for token reuse detection

---

## Further Reading

- [BATCH_004_TASK_005_IMPLEMENTATION.md](./BATCH_004_TASK_005_IMPLEMENTATION.md) - Complete implementation details
- [OAuth 2.0 Security Best Current Practice](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-security-topics)
- [Refresh Token Rotation](https://auth0.com/docs/secure/tokens/refresh-tokens/refresh-token-rotation)

---

**Last Updated**: 2025-12-06
**Maintained By**: Media Gateway Team
