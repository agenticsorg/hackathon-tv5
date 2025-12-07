# Media Gateway - Authentication Service

Secure OAuth 2.0 authentication and authorization service for the Media Gateway platform.

## Features

### OAuth 2.0 Flows

- **OAuth 2.0 + PKCE**: Web and mobile client authentication with Proof Key for Code Exchange
- **Device Authorization Grant (RFC 8628)**: Smart TV and CLI authentication
- **Token Refresh**: Automatic access token rotation with refresh tokens

### Security Features

- **RS256 JWT Tokens**: Asymmetric signing with 1-hour access tokens
- **Token Rotation**: Refresh tokens rotate on every use (7-day expiry)
- **Token Revocation**: Real-time revocation with Redis tracking
- **RBAC**: Role-based access control with resource-level permissions
- **OAuth Scopes**: Fine-grained permission scoping

### Supported Roles

- `anonymous`: Minimal public access
- `free_user`: Basic content access
- `premium_user`: Advanced features and unlimited access
- `admin`: Full system access
- `service_account`: API service access

### OAuth Scopes

**Read Scopes:**
- `read:content` - Browse content metadata
- `read:watchlist` - View watchlists
- `read:preferences` - Access user preferences
- `read:recommendations` - Get personalized recommendations
- `read:devices` - View registered devices

**Write Scopes:**
- `write:watchlist` - Manage watchlists
- `write:preferences` - Update preferences
- `write:ratings` - Submit content ratings
- `write:devices` - Register/deregister devices

**Special Scopes:**
- `playback:control` - Control device playback (requires consent)
- `admin:full` - Full administrative access

## API Endpoints

### Health Check
```
GET /health
```

### OAuth 2.0 Authorization
```
GET /auth/authorize?client_id={id}&redirect_uri={uri}&response_type=code&scope={scopes}&code_challenge={challenge}&code_challenge_method=S256
```

### Token Exchange
```
POST /auth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code
&code={auth_code}
&code_verifier={verifier}
&redirect_uri={uri}
&client_id={id}
```

### Token Refresh
```
POST /auth/token
Content-Type: application/x-www-form-urlencoded

grant_type=refresh_token
&refresh_token={token}
&client_id={id}
```

### Token Revocation
```
POST /auth/revoke
Content-Type: application/x-www-form-urlencoded

token={token}
&token_type_hint=access_token
```

### Device Authorization (RFC 8628)
```
POST /auth/device
Content-Type: application/x-www-form-urlencoded

client_id={id}
&scope={scopes}
```

Response:
```json
{
  "device_code": "GmRh...mJc",
  "user_code": "WDJB-MJHT",
  "verification_uri": "https://auth.mediagateway.io/device",
  "verification_uri_complete": "https://auth.mediagateway.io/device?user_code=WDJB-MJHT",
  "expires_in": 900,
  "interval": 5
}
```

### Device Token Polling
```
GET /auth/device/poll?device_code={code}
```

## Configuration

### Required Environment Variables

```bash
# Server
BIND_ADDRESS=0.0.0.0:8084

# Redis
REDIS_URL=redis://localhost:6379

# JWT Keys (RS256)
JWT_PRIVATE_KEY_PATH=/secrets/jwt_private_key.pem
JWT_PUBLIC_KEY_PATH=/secrets/jwt_public_key.pem
JWT_ISSUER=https://api.mediagateway.io
JWT_AUDIENCE=mediagateway-users

# OAuth Providers
GOOGLE_CLIENT_ID=xxx
GOOGLE_CLIENT_SECRET=xxx
GITHUB_CLIENT_ID=xxx
GITHUB_CLIENT_SECRET=xxx
```

## Performance Targets (SPARC)

- **Authentication latency p95**: <200ms
- **Authorization latency p95**: <10ms
- **Token validation**: <5ms
- **Token revocation propagation**: <5 seconds

## Security Guarantees

- **PKCE Required**: S256 challenge method mandatory
- **Token Rotation**: Refresh tokens single-use
- **Replay Protection**: Authorization code reuse triggers full revocation
- **Secure Storage**: Tokens hashed with SHA-256 in database
- **Redis Sessions**: Distributed session management with TTL
- **Audit Logging**: All authentication events logged

## Dependencies

- `actix-web` 4.x - HTTP server
- `jsonwebtoken` - JWT RS256 implementation
- `redis` - Session store
- `sqlx` - Database access
- `argon2` - Password hashing
- `oauth2` - OAuth 2.0 client

## Development

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
```

### Run
```bash
cargo run --bin media-gateway-auth
```

## Security Notes

1. **NEVER** commit private keys to version control
2. Load JWT keys from Google Secret Manager in production
3. Use TLS 1.3 for all connections
4. Enable CORS only for whitelisted domains
5. Implement rate limiting at API gateway level
6. Monitor for brute force attacks
7. Rotate JWT signing keys every 90 days

## License

Copyright 2025 Media Gateway
