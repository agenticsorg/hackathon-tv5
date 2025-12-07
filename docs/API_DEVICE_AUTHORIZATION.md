# Device Authorization Flow API Reference (RFC 8628)

## Overview

The Device Authorization Grant (RFC 8628) enables OAuth 2.0 authentication for input-constrained devices like Smart TVs, CLI tools, and IoT devices.

## Complete Flow

```
┌──────────┐                                      ┌─────────────┐
│  Device  │                                      │ Auth Server │
└─────┬────┘                                      └──────┬──────┘
      │                                                  │
      │  1. POST /auth/device                           │
      │─────────────────────────────────────────────────>│
      │                                                  │
      │  device_code, user_code, verification_uri       │
      │<─────────────────────────────────────────────────│
      │                                                  │
      │  [Display user_code to user]                    │
      │                                                  │
      │                                                  │
      │  (User visits verification_uri on another device)
      │                                                  │
      │                                  ┌───────┐       │
      │                                  │ User  │       │
      │                                  └───┬───┘       │
      │                                      │           │
      │                                      │ 2. POST   │
      │                                      │ /auth/    │
      │                                      │ device/   │
      │                                      │ approve   │
      │                                      │ + JWT     │
      │                                      │────────────>
      │                                      │           │
      │                                      │ Approved  │
      │                                      │<───────────│
      │                                      │           │
      │  3. GET /auth/device/poll                        │
      │  (Every 5 seconds)                               │
      │─────────────────────────────────────────────────>│
      │                                                  │
      │  authorization_pending (while waiting)           │
      │<─────────────────────────────────────────────────│
      │                                                  │
      │  4. GET /auth/device/poll (after approval)       │
      │─────────────────────────────────────────────────>│
      │                                                  │
      │  access_token, refresh_token                     │
      │<─────────────────────────────────────────────────│
      │                                                  │
```

## Endpoints

### 1. Device Authorization Request

**Endpoint:** `POST /auth/device`

**Description:** Device initiates the authorization flow and receives a user code.

**Request:**
```http
POST /auth/device HTTP/1.1
Content-Type: application/x-www-form-urlencoded

client_id=smart-tv-app&scope=read:content write:content
```

**Parameters:**
- `client_id` (required) - The client application identifier
- `scope` (optional) - Space-separated list of requested scopes

**Success Response (200 OK):**
```json
{
  "device_code": "GmRhmhcxhwAzkoEqiMEg9YBNVdxIr",
  "user_code": "WDJB-MJHT",
  "verification_uri": "https://auth.mediagateway.io/device",
  "verification_uri_complete": "https://auth.mediagateway.io/device?user_code=WDJB-MJHT",
  "expires_in": 900,
  "interval": 5
}
```

**Response Fields:**
- `device_code` - Code for polling (32 characters)
- `user_code` - Code for user to enter (format: XXXX-XXXX)
- `verification_uri` - URL where user approves the device
- `verification_uri_complete` - Pre-filled URL with user_code
- `expires_in` - Code lifetime in seconds (900 = 15 minutes)
- `interval` - Minimum seconds between polls (5 seconds)

**Example:**
```bash
curl -X POST http://localhost:8080/auth/device \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=smart-tv-app&scope=read:content"
```

---

### 2. Device Approval (New)

**Endpoint:** `POST /auth/device/approve`

**Description:** Authenticated user approves a device using the displayed user code.

**Request:**
```http
POST /auth/device/approve HTTP/1.1
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "user_code": "WDJB-MJHT"
}
```

**Headers:**
- `Authorization` (required) - Bearer token with valid access token
- `Content-Type` (required) - application/json

**Request Body:**
- `user_code` (required) - The code displayed on the device

**Success Response (200 OK):**
```json
{
  "message": "Device authorization approved",
  "user_code": "WDJB-MJHT"
}
```

**Error Responses:**

**400 Bad Request - Invalid User Code:**
```json
{
  "error": "invalid_grant",
  "error_description": "Invalid user code"
}
```

**400 Bad Request - Device Already Approved:**
```json
{
  "error": "invalid_grant",
  "error_description": "Device already approved"
}
```

**400 Bad Request - Device Expired:**
```json
{
  "error": "expired_token",
  "error_description": "Device code expired"
}
```

**401 Unauthorized - Missing/Invalid Token:**
```json
{
  "error": "unauthorized",
  "error_description": "Authentication required"
}
```

**Example:**
```bash
# Get user access token first (via login flow)
USER_TOKEN="eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."

# Approve device
curl -X POST http://localhost:8080/auth/device/approve \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"user_code":"WDJB-MJHT"}'
```

---

### 3. Token Polling

**Endpoint:** `GET /auth/device/poll`

**Description:** Device polls for tokens after user approval.

**Request:**
```http
GET /auth/device/poll?device_code=GmRhmhcxhwAzkoEqiMEg9YBNVdxIr HTTP/1.1
```

**Query Parameters:**
- `device_code` (required) - The device_code from step 1

**Success Response (200 OK) - Device Approved:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "read:content write:content"
}
```

**Response Fields:**
- `access_token` - JWT access token for API calls
- `refresh_token` - JWT refresh token for renewing access
- `token_type` - Always "Bearer"
- `expires_in` - Access token lifetime in seconds
- `scope` - Granted scopes (space-separated)

**Error Responses:**

**400 Bad Request - Authorization Pending:**
```json
{
  "error": "authorization_pending",
  "error_description": "User has not yet completed authorization"
}
```
*Device should wait `interval` seconds and poll again.*

**400 Bad Request - Device Code Not Found:**
```json
{
  "error": "server_error",
  "error_description": "Internal server error"
}
```
*Invalid device_code or code expired.*

**Example:**
```bash
# Poll every 5 seconds until approved
DEVICE_CODE="GmRhmhcxhwAzkoEqiMEg9YBNVdxIr"

while true; do
  curl "http://localhost:8080/auth/device/poll?device_code=$DEVICE_CODE"
  sleep 5
done
```

---

## Client Implementation Example

### Smart TV Application

```javascript
// Step 1: Request device authorization
async function startDeviceFlow() {
  const response = await fetch('https://auth.mediagateway.io/auth/device', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: 'client_id=smart-tv-app&scope=read:content',
  });

  const data = await response.json();

  // Display user_code on TV screen
  displayUserCode(data.user_code);
  displayVerificationUrl(data.verification_uri);

  // Start polling
  pollForTokens(data.device_code, data.interval);
}

// Step 2: Poll for tokens
async function pollForTokens(deviceCode, interval) {
  while (true) {
    await sleep(interval * 1000);

    const response = await fetch(
      `https://auth.mediagateway.io/auth/device/poll?device_code=${deviceCode}`
    );

    const data = await response.json();

    if (response.ok) {
      // Success! Got tokens
      saveTokens(data.access_token, data.refresh_token);
      return;
    }

    if (data.error === 'authorization_pending') {
      // Keep polling
      continue;
    }

    // Other error - stop polling
    handleError(data.error);
    return;
  }
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
```

### User Approval Web App

```javascript
// User enters code on web/mobile
async function approveDevice(userCode) {
  const accessToken = localStorage.getItem('access_token');

  const response = await fetch('https://auth.mediagateway.io/auth/device/approve', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      user_code: userCode,
    }),
  });

  if (response.ok) {
    showSuccess('Device approved!');
  } else {
    const error = await response.json();
    showError(error.error_description);
  }
}
```

---

## Security Considerations

### For Device Applications
1. **Store device_code securely** - Don't log or expose to users
2. **Respect polling interval** - Wait at least `interval` seconds between polls
3. **Handle timeouts** - Stop polling after `expires_in` seconds
4. **Validate tokens** - Verify JWT signature and expiration

### For User Approval
1. **Require authentication** - User must be logged in
2. **Validate user_code format** - Must match XXXX-XXXX pattern
3. **Show device info** - Display what the user is approving
4. **Confirm action** - Ask user to confirm approval

### For Auth Server
1. **JWT validation** - Verify access token signature and expiration
2. **Token revocation** - Check if token is revoked
3. **One-time use** - Delete device_code after token issuance
4. **TTL enforcement** - Expire codes after 15 minutes
5. **Rate limiting** - Prevent polling abuse

---

## Error Codes Summary

| Error Code | Description | Action |
|------------|-------------|--------|
| `authorization_pending` | User hasn't approved yet | Keep polling |
| `expired_token` | Device code expired (15 min) | Start new flow |
| `invalid_grant` | Invalid user_code or already approved | Show error |
| `unauthorized` | Missing/invalid JWT token | Redirect to login |
| `server_error` | Internal error | Retry or contact support |

---

## Rate Limits

- **Polling**: Minimum 5 seconds between requests
- **Device authorization**: 10 requests per minute per IP
- **Approval**: 5 requests per minute per user

---

## Testing

### Manual Testing Flow

```bash
# Terminal 1: Start auth server
cd crates/auth
cargo run

# Terminal 2: Device requests authorization
curl -X POST http://localhost:8080/auth/device \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=test-client&scope=read:content"

# Note the device_code and user_code

# Terminal 3: Get user token (simulate logged-in user)
# This would normally come from login flow
USER_TOKEN="<access_token_from_login>"

# User approves device
curl -X POST http://localhost:8080/auth/device/approve \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"user_code":"WDJB-MJHT"}'

# Back to Terminal 2: Poll for tokens
curl "http://localhost:8080/auth/device/poll?device_code=<device_code>"
```

---

## Best Practices

### For Developers

1. **Use verification_uri_complete** - Include QR code for easy mobile scanning
2. **Show clear instructions** - Tell users where to go and what to enter
3. **Handle errors gracefully** - Provide user-friendly error messages
4. **Implement exponential backoff** - If polling fails repeatedly
5. **Clear codes** - Delete displayed codes after success/failure

### For UX

1. **Large font for user_code** - Easy to read from distance (TV)
2. **QR code option** - Faster approval via mobile scan
3. **Progress indicator** - Show "Waiting for approval..."
4. **Timeout warning** - Notify when code is about to expire
5. **Success confirmation** - Clear feedback when approved

---

## Related Documentation

- [RFC 8628 - OAuth 2.0 Device Authorization Grant](https://datatracker.ietf.org/doc/html/rfc8628)
- [JWT Token Management](./JWT_TOKENS.md)
- [OAuth 2.0 Scopes](./OAUTH_SCOPES.md)
- [Error Handling Guide](./ERROR_HANDLING.md)

---

## Support

For issues or questions:
- GitHub Issues: https://github.com/media-gateway/issues
- Documentation: https://docs.mediagateway.io
- API Status: https://status.mediagateway.io
