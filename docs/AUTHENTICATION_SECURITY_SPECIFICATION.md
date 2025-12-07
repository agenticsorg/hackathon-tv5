# Media Gateway - Authentication and Security Specification

**Document Version:** 1.0.0
**Last Updated:** 2025-12-06
**Status:** SPARC Specification Phase
**Classification:** Architecture Constraints

---

## Executive Summary

This specification defines the authentication, authorization, and security constraints for the Media Gateway platform. The system implements a privacy-first, multi-tenant architecture supporting web, mobile, CLI, and smart TV clients with OAuth 2.0-based authentication and comprehensive security controls.

**Key Security Principles:**
- Privacy-first design with on-device data storage
- Zero platform credential storage (deep-link based integration)
- Multi-layer security boundaries with defense in depth
- OAuth 2.0 + PKCE for all client authentication
- Workload Identity for service-to-service authentication
- Comprehensive audit logging and monitoring

---

## Table of Contents

1. [Authentication Constraints](#1-authentication-constraints)
2. [Authorization Model](#2-authorization-model)
3. [Security Requirements](#3-security-requirements)
4. [Platform-Specific Authentication](#4-platform-specific-authentication)
5. [Multi-Tenant Architecture](#5-multi-tenant-architecture)
6. [Compliance and Audit Requirements](#6-compliance-and-audit-requirements)

---

## 1. Authentication Constraints

### 1.1 User Authentication Mechanisms

#### 1.1.1 OAuth 2.0 + PKCE (Web & Mobile)

**Protocol:** OAuth 2.0 with Proof Key for Code Exchange (PKCE)
**RFC Reference:** RFC 7636, RFC 9700 (2025 Best Practices)
**Use Case:** Web applications, mobile applications (iOS, Android)

**Authentication Flow:**

```
┌──────────┐                                           ┌──────────────┐
│  Client  │                                           │ Auth Server  │
└────┬─────┘                                           └──────┬───────┘
     │                                                         │
     │ 1. Generate code_verifier (random 43-128 chars)       │
     │    code_challenge = SHA256(code_verifier)             │
     │                                                         │
     │ 2. Authorization Request                               │
     │    GET /authorize?                                     │
     │        client_id={client_id}&                         │
     │        redirect_uri={redirect}&                       │
     │        response_type=code&                            │
     │        code_challenge={challenge}&                    │
     │        code_challenge_method=S256                     │
     ├────────────────────────────────────────────────────────>│
     │                                                         │
     │ 3. User Authentication & Consent                       │
     │<────────────────────────────────────────────────────────┤
     │                                                         │
     │ 4. Authorization Code                                  │
     │    Redirect: {redirect_uri}?code={auth_code}          │
     │<────────────────────────────────────────────────────────┤
     │                                                         │
     │ 5. Token Request                                       │
     │    POST /token                                         │
     │        grant_type=authorization_code&                  │
     │        code={auth_code}&                               │
     │        redirect_uri={redirect}&                        │
     │        client_id={client_id}&                         │
     │        code_verifier={verifier}                       │
     ├────────────────────────────────────────────────────────>│
     │                                                         │
     │ 6. Access + Refresh Tokens                            │
     │    {                                                   │
     │      "access_token": "...",                           │
     │      "refresh_token": "...",                          │
     │      "expires_in": 3600,                              │
     │      "token_type": "Bearer"                           │
     │    }                                                   │
     │<────────────────────────────────────────────────────────┤
     │                                                         │
```

**Security Constraints:**

- **Code Verifier Length:** 43-128 characters (cryptographically random)
- **Code Challenge Method:** SHA256 (S256) REQUIRED
- **State Parameter:** REQUIRED for CSRF protection
- **Redirect URI:** MUST be pre-registered and validated
- **Authorization Code Lifetime:** Maximum 10 minutes
- **HTTPS Enforcement:** All OAuth endpoints MUST use TLS 1.3

**DEPRECATED FLOWS (MUST NOT USE):**
- ❌ Implicit Grant (RFC 9700 deprecated)
- ❌ Resource Owner Password Credentials (ROPC)
- ❌ Client Credentials for user authentication

#### 1.1.2 Device Authorization Grant (Smart TV & CLI)

**Protocol:** OAuth 2.0 Device Authorization Grant
**RFC Reference:** RFC 8628
**Use Case:** Smart TVs (Samsung Tizen, LG webOS, Roku), CLI tools, headless devices

**Authentication Flow:**

```
┌──────────┐                                           ┌──────────────┐
│  Device  │                                           │ Auth Server  │
└────┬─────┘                                           └──────┬───────┘
     │                                                         │
     │ 1. Device Authorization Request                        │
     │    POST /device/authorize                              │
     │        client_id={client_id}&                         │
     │        scope={requested_scopes}                       │
     ├────────────────────────────────────────────────────────>│
     │                                                         │
     │ 2. Device Code Response                                │
     │    {                                                   │
     │      "device_code": "GmRh...mJc",                     │
     │      "user_code": "WDJB-MJHT",                        │
     │      "verification_uri": "https://auth.../device",    │
     │      "verification_uri_complete": "https://...?c=...", │
     │      "expires_in": 1800,                              │
     │      "interval": 5                                    │
     │    }                                                   │
     │<────────────────────────────────────────────────────────┤
     │                                                         │
     │ 3. Display User Code & QR Code                        │
     │    ┌─────────────────────────────┐                    │
     │    │  Visit: auth.../device      │                    │
     │    │  Enter Code: WDJB-MJHT      │                    │
     │    │  [QR Code displayed here]   │                    │
     │    └─────────────────────────────┘                    │
     │                                                         │
     │ 4. Polling Loop (every 5 seconds)                     │
     │    POST /token                                         │
     │        grant_type=urn:ietf:params:oauth:...device&    │
     │        device_code={device_code}&                     │
     │        client_id={client_id}                          │
     ├────────────────────────────────────────────────────────>│
     │                                                         │
     │ 5a. Authorization Pending                              │
     │     {"error": "authorization_pending"}                │
     │<────────────────────────────────────────────────────────┤
     │                                                         │
     │    [User completes auth on separate device]            │
     │                                                         │
     │ 5b. Tokens Issued                                      │
     │     {                                                  │
     │       "access_token": "...",                          │
     │       "refresh_token": "...",                         │
     │       "expires_in": 3600                              │
     │     }                                                  │
     │<────────────────────────────────────────────────────────┤
     │                                                         │
```

**Security Constraints:**

- **User Code Format:** 6-8 uppercase alphanumeric characters (human-readable)
- **Device Code Lifetime:** 15-30 minutes maximum
- **Polling Interval:** 5 seconds minimum (exponential backoff on errors)
- **Rate Limiting:** Maximum 12 attempts per minute per device_code
- **QR Code Generation:** MUST encode verification_uri_complete
- **Error Handling:**
  - `authorization_pending` - Continue polling
  - `slow_down` - Increase polling interval by 5 seconds
  - `access_denied` - User rejected authorization
  - `expired_token` - Device code expired, restart flow

#### 1.1.3 Service-to-Service Authentication

**Protocol:** Mutual TLS (mTLS) + Workload Identity
**Use Case:** Microservice communication within GKE cluster

**GCP Workload Identity Pattern:**

```yaml
# Kubernetes ServiceAccount binding
apiVersion: v1
kind: ServiceAccount
metadata:
  name: recommendation-service
  namespace: media-gateway
  annotations:
    iam.gke.io/gcp-service-account: recommendation-sa@project.iam.gserviceaccount.com

---
# GCP IAM Binding
gcloud iam service-accounts add-iam-policy-binding \
  recommendation-sa@project.iam.gserviceaccount.com \
  --role roles/iam.workloadIdentityUser \
  --member "serviceAccount:project.svc.id.goog[media-gateway/recommendation-service]"
```

**mTLS Configuration:**

```rust
// gRPC client with mTLS
let tls_config = ClientTlsConfig::new()
    .ca_certificate(Certificate::from_pem(ca_cert))
    .identity(Identity::from_pem(client_cert, client_key))
    .domain_name("recommendation-service.media-gateway.svc.cluster.local");

let channel = Channel::from_static("https://recommendation-service:50051")
    .tls_config(tls_config)?
    .connect()
    .await?;
```

**Security Constraints:**

- **Certificate Authority:** Private CA managed by cert-manager or GCP Certificate Authority Service
- **Certificate Rotation:** Automated 90-day rotation via cert-manager
- **Peer Verification:** REQUIRED - all services MUST verify peer certificates
- **Domain Validation:** MUST match Kubernetes service DNS names
- **Cipher Suites:** TLS 1.3 with ECDHE-RSA-AES256-GCM-SHA384 minimum
- **Workload Identity Binding:** REQUIRED for all GKE pods accessing GCP APIs

### 1.2 Token Management and Refresh

#### 1.2.1 Token Specifications

**Access Token:**
- **Format:** JWT (JSON Web Token) signed with RS256
- **Lifetime:** 3600 seconds (1 hour) - CONFIGURABLE
- **Payload:**
```json
{
  "iss": "https://auth.media-gateway.io",
  "sub": "user-uuid-v4",
  "aud": ["api.media-gateway.io"],
  "exp": 1733453765,
  "iat": 1733450165,
  "scope": "read:content write:preferences",
  "device_id": "device-uuid-v4",
  "region": "US"
}
```

**Refresh Token:**
- **Format:** Opaque token (cryptographically secure random, 256-bit)
- **Lifetime:** 90 days (rolling window with rotation)
- **Storage:** Server-side database with indexed lookup
- **Rotation:** REQUIRED - new refresh token issued on each refresh request

#### 1.2.2 Token Refresh Flow

```
Client                                              Token Service
  │                                                       │
  │ POST /token                                          │
  │   grant_type=refresh_token                          │
  │   refresh_token={current_refresh_token}             │
  │   client_id={client_id}                             │
  ├──────────────────────────────────────────────────────>│
  │                                                       │
  │                                    ┌──────────────────┤
  │                                    │ 1. Validate token│
  │                                    │ 2. Check revoked │
  │                                    │ 3. Verify client │
  │                                    └──────────────────┤
  │                                                       │
  │ Response:                                            │
  │   {                                                  │
  │     "access_token": "new_access_token",             │
  │     "refresh_token": "new_refresh_token",           │
  │     "expires_in": 3600                              │
  │   }                                                  │
  │<──────────────────────────────────────────────────────┤
  │                                                       │
  │                                    ┌──────────────────┤
  │                                    │ 4. Revoke old    │
  │                                    │    refresh token │
  │                                    │ 5. Store new     │
  │                                    └──────────────────┤
```

**Security Constraints:**

- **Refresh Window:** Access tokens refreshable 5 minutes before expiry
- **Single-Use:** Refresh tokens MUST be invalidated after use
- **Replay Detection:** Attempted reuse of revoked refresh token triggers:
  - Immediate revocation of all tokens for user+device
  - Security audit event logged
  - Optional user notification
- **Concurrent Refresh:** Last-writer-wins with 30-second grace period
- **Family Tracking:** Maintain refresh token family chains for audit

#### 1.2.3 Token Storage by Platform

| Platform | Storage Mechanism | Encryption | Accessibility |
|----------|------------------|------------|---------------|
| **Web** | HTTP-only Cookies | TLS in-transit | Same-origin only |
| **Mobile (iOS)** | iOS Keychain | Hardware-backed (Secure Enclave) | App-specific |
| **Mobile (Android)** | Android Keystore | Hardware-backed (TEE) | App-specific |
| **CLI** | OS Keyring (keyring-rs) | OS-managed | User-specific |
| **Smart TV (Tizen)** | Encrypted Local Storage | AES-256-GCM | App sandbox |
| **Smart TV (webOS)** | Encrypted Local Storage | AES-256-GCM | App sandbox |

**Security Requirements:**

- **Web Cookies:**
  - `HttpOnly` flag REQUIRED
  - `Secure` flag REQUIRED (HTTPS only)
  - `SameSite=Strict` for CSRF protection
  - Domain scoping to `*.media-gateway.io`

- **Mobile Storage:**
  - Biometric authentication protection (optional)
  - Background app lock (token access restricted)
  - Jailbreak/root detection with warnings

- **CLI Storage:**
  - macOS: Keychain Services API
  - Linux: Secret Service API (libsecret)
  - Windows: Credential Manager API
  - Fallback: Encrypted file with user password

- **Smart TV Storage:**
  - Application-specific encrypted storage
  - Device binding (tokens tied to device ID)
  - Factory reset clears all tokens

### 1.3 Token Revocation

#### 1.3.1 Revocation Triggers

**User-Initiated:**
- Explicit logout action
- "Revoke all devices" security action
- Account deletion request

**System-Initiated:**
- Suspicious activity detection
- Maximum device limit exceeded
- Token replay attempt detected
- Account compromise indicators

**Scheduled:**
- 90-day refresh token expiration
- Inactive device cleanup (180 days)

#### 1.3.2 Revocation Propagation

**Architecture:** Real-time revocation via PubNub

```
Token Service                    PubNub                    Client Devices
     │                              │                            │
     │ 1. Revoke Token              │                            │
     │    (Database update)         │                            │
     │                              │                            │
     │ 2. Publish Revocation Event  │                            │
     │────────────────────────────>│                            │
     │                              │                            │
     │                              │ 3. Broadcast to Subscribed │
     │                              │    Devices                 │
     │                              ├───────────────────────────>│
     │                              │                            │
     │                              │                            │ 4. Clear Local
     │                              │                            │    Tokens
     │                              │                            │ 5. Redirect to
     │                              │                            │    Login
```

**PubNub Channel Structure:**
```
user.{userId}.tokens - Per-user token revocation events
user.{userId}.devices - Device presence and registration
system.security.global - Global security events (breach, maintenance)
```

**Event Payload:**
```json
{
  "event_type": "token_revoked",
  "user_id": "user-uuid",
  "device_id": "device-uuid",
  "revoked_at": "2025-12-06T00:00:00Z",
  "reason": "user_logout",
  "action": "redirect_login"
}
```

**Security Constraints:**

- **Revocation Latency:** Maximum 5 seconds for connected devices
- **Offline Handling:** Token validation on next API request
- **Grace Period:** 30 seconds for in-flight requests to complete
- **Audit Logging:** All revocations logged with reason and timestamp

---

## 2. Authorization Model

### 2.1 Role-Based Access Control (RBAC)

#### 2.1.1 User Roles

| Role | Capabilities | Resource Access |
|------|-------------|----------------|
| **user** | Read content, manage personal preferences, create watchlists | Own user data only |
| **premium** | All user capabilities + advanced filters, export data | Own user data + premium features |
| **family_admin** | Manage family account, parental controls, sub-profiles | Family account scope |
| **partner** | Content metadata submission, analytics access | Partner-specific content |
| **admin** | User management, system configuration, audit logs | All resources |

#### 2.1.2 Permission Scopes

OAuth scopes control granular access:

**Read Scopes:**
- `read:content` - Search and browse content metadata
- `read:preferences` - Access user viewing preferences
- `read:watchlist` - View saved watchlists
- `read:recommendations` - Get personalized recommendations

**Write Scopes:**
- `write:preferences` - Update viewing preferences
- `write:watchlist` - Manage watchlists
- `write:ratings` - Submit content ratings
- `write:devices` - Register/deregister devices

**Administrative Scopes:**
- `admin:users` - User account management
- `admin:system` - System configuration
- `admin:analytics` - Access aggregated analytics

**Scope Validation:**
```rust
fn validate_scopes(required: &[&str], token_scopes: &[String]) -> Result<()> {
    for scope in required {
        if !token_scopes.iter().any(|s| s == scope) {
            return Err(Error::InsufficientPermissions);
        }
    }
    Ok(())
}
```

### 2.2 Resource-Level Permissions

#### 2.2.1 User Data Isolation

**Database Query Pattern:**
```sql
-- All user data queries MUST include user_id filter
SELECT * FROM preferences
WHERE user_id = $1
AND region = $2;

-- Multi-tenant watchlist access
SELECT w.* FROM watchlists w
INNER JOIN watchlist_permissions wp ON w.id = wp.watchlist_id
WHERE wp.user_id = $1
AND wp.permission IN ('owner', 'editor', 'viewer');
```

**API Middleware:**
```rust
async fn user_scope_middleware(
    req: Request,
    next: Next,
) -> Result<Response> {
    let token = extract_token(&req)?;
    let user_id = validate_token(&token)?;

    // Inject user_id into request context
    req.extensions_mut().insert(UserId(user_id));

    // Verify resource ownership
    if let Some(resource_user_id) = extract_resource_user_id(&req) {
        if resource_user_id != user_id {
            return Err(Error::Forbidden);
        }
    }

    next.run(req).await
}
```

#### 2.2.2 Content Rights Validation

**Availability Check:**
```rust
pub struct RightsValidator {
    region: String,
    user_subscriptions: Vec<PlatformSubscription>,
}

impl RightsValidator {
    pub async fn check_availability(
        &self,
        content_id: &str,
    ) -> Result<AvailabilityResult> {
        // 1. Query rights database
        let rights = self.db.get_rights(content_id, &self.region).await?;

        // 2. Filter by active licensing windows
        let available_platforms = rights.into_iter()
            .filter(|r| r.is_active_in_region(&self.region))
            .filter(|r| r.end_date > Utc::now())
            .collect();

        // 3. Match with user subscriptions
        let accessible_platforms = self.match_subscriptions(available_platforms);

        Ok(AvailabilityResult {
            available: !accessible_platforms.is_empty(),
            platforms: accessible_platforms,
        })
    }
}
```

### 2.3 Multi-Tenant Considerations

#### 2.3.1 Tenant Isolation Levels

**Level 1: Database Schema Separation**
- Separate PostgreSQL schemas per major tenant
- Shared infrastructure, isolated data
- Use case: Enterprise partners with high data volumes

**Level 2: Row-Level Security (RLS)**
- Shared tables with `tenant_id` column
- PostgreSQL RLS policies enforce isolation
- Use case: Individual users and small organizations

**Level 3: Application-Level Enforcement**
- Middleware validates tenant context
- All queries include tenant scoping
- Use case: All API requests

**RLS Policy Example:**
```sql
CREATE POLICY user_isolation_policy ON preferences
FOR ALL
TO application_role
USING (user_id = current_setting('app.user_id')::uuid);

-- Set context in application
SET LOCAL app.user_id = 'user-uuid-value';
```

#### 2.3.2 Cross-Tenant Resource Sharing

**Watchlist Sharing:**
```rust
pub struct WatchlistPermission {
    watchlist_id: Uuid,
    user_id: Uuid,
    permission: PermissionLevel, // Owner, Editor, Viewer
    granted_by: Uuid,
    granted_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

pub enum PermissionLevel {
    Owner,   // Full control
    Editor,  // Add/remove items
    Viewer,  // Read-only
}
```

**Permission Validation:**
```rust
async fn validate_watchlist_access(
    user_id: Uuid,
    watchlist_id: Uuid,
    required: PermissionLevel,
) -> Result<()> {
    let permission = db.get_permission(user_id, watchlist_id).await?;

    match (permission.permission, required) {
        (PermissionLevel::Owner, _) => Ok(()),
        (PermissionLevel::Editor, PermissionLevel::Viewer) => Ok(()),
        (PermissionLevel::Editor, PermissionLevel::Editor) => Ok(()),
        (PermissionLevel::Viewer, PermissionLevel::Viewer) => Ok(()),
        _ => Err(Error::InsufficientPermissions),
    }
}
```

### 2.4 API Key Scoping

#### 2.4.1 API Key Types

**User API Keys:**
- Tied to specific user account
- Inherit user's role and permissions
- Use case: Personal automation, CLI access

**Service API Keys:**
- Machine-to-machine authentication
- Scoped to specific services/endpoints
- Use case: Partner integrations, external services

**Developer API Keys:**
- Sandbox environment access
- Rate-limited and feature-restricted
- Use case: Third-party app development

#### 2.4.2 API Key Structure

```json
{
  "key_id": "mgw_live_1a2b3c4d5e6f",
  "key_secret": "sk_live_9x8y7z6w5v4u3t2s1r",
  "type": "service",
  "scopes": ["read:content", "read:availability"],
  "rate_limit": {
    "requests_per_minute": 100,
    "requests_per_day": 10000
  },
  "allowed_ips": ["203.0.113.0/24"],
  "created_at": "2025-12-06T00:00:00Z",
  "expires_at": "2026-12-06T00:00:00Z",
  "metadata": {
    "partner_id": "partner-uuid",
    "environment": "production"
  }
}
```

**Key Prefix Convention:**
- `mgw_test_` - Sandbox/development environment
- `mgw_live_` - Production environment
- `sk_` - Secret key (never logged or displayed)

#### 2.4.3 API Key Rotation

**Rotation Policy:**
- **Manual Rotation:** User-initiated key regeneration
- **Scheduled Rotation:** 365-day automatic rotation for service keys
- **Emergency Rotation:** Immediate revocation on compromise detection

**Rotation Process:**
```
1. Generate new API key
2. Return both old and new keys
3. Grace period: 30 days (both keys valid)
4. Warning notifications: 7 days before old key expiration
5. Revoke old key after grace period
6. Audit log rotation event
```

---

## 3. Security Requirements

### 3.1 Data Encryption

#### 3.1.1 Encryption at Rest

**Google Cloud SQL (PostgreSQL):**
- **Method:** Automatic encryption using Google-managed keys
- **Algorithm:** AES-256 encryption
- **Key Management:** Google Cloud KMS with automatic rotation
- **Scope:** Database files, backups, snapshots

**Memorystore (Valkey/Redis):**
- **Method:** Encryption using Google-managed keys
- **Algorithm:** AES-256 encryption
- **In-transit:** TLS 1.3 for all client connections

**User Data (On-Device):**
- **Watch History:** Device-specific AES-256-GCM encryption
- **Key Derivation:** PBKDF2 with 100,000 iterations
- **Storage:** Platform-specific secure storage (Keychain, Keystore)

**Federated Learning Gradients:**
- **Algorithm:** AES-256-GCM with per-user ephemeral keys
- **Differential Privacy:** Laplace noise (ε=1.0, δ=1e-5) added before encryption
- **Aggregation:** Homomorphic encryption for secure aggregation

#### 3.1.2 Encryption in Transit

**External Communications:**
- **Protocol:** TLS 1.3 (minimum 1.2 for legacy clients)
- **Cipher Suites:**
  - TLS_AES_256_GCM_SHA384 (preferred)
  - TLS_CHACHA20_POLY1305_SHA256
  - TLS_AES_128_GCM_SHA256
- **Certificate Validation:** REQUIRED with certificate pinning for mobile apps
- **HSTS:** Enabled with max-age=31536000; includeSubDomains; preload

**Internal Service Mesh (Istio):**
- **Protocol:** Mutual TLS (mTLS) with automatic rotation
- **Certificate Authority:** Istio CA or cert-manager
- **Certificate Lifetime:** 90 days with automatic renewal
- **Cipher Suite:** TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384

**PubNub Real-Time Channels:**
- **Protocol:** TLS 1.2+ for all connections
- **Channel Encryption:** AES-256 for message payloads (optional)
- **Message Signing:** HMAC-SHA256 for integrity verification

### 3.2 Credential Storage Patterns

#### 3.2.1 Google Secret Manager

**Secret Types:**
- Platform API keys (YouTube Data API, streaming aggregators)
- Database credentials
- OAuth client secrets
- Service account keys (exception only)
- Encryption keys for application-level encryption

**Secret Naming Convention:**
```
{environment}/{service}/{secret-type}/{version}

Examples:
- production/recommendation-service/api-key/v1
- staging/auth-service/oauth-client-secret/v2
- production/global/youtube-api-key/v3
```

**Access Control (IAM):**
```yaml
# Service-specific secret access
roles/secretmanager.secretAccessor:
  - serviceAccount:recommendation-sa@project.iam.gserviceaccount.com
  conditions:
    - title: "Production secrets only"
      expression: "resource.name.startsWith('projects/PROJECT_ID/secrets/production/')"
```

**Rotation Policy:**
```json
{
  "rotation_schedule": {
    "api_keys": "90_days",
    "database_passwords": "30_days",
    "oauth_client_secrets": "180_days",
    "encryption_keys": "365_days"
  },
  "rotation_automation": {
    "enabled": true,
    "notification_days_before": 7,
    "grace_period_days": 14
  }
}
```

#### 3.2.2 Credential Injection Patterns

**Kubernetes Secrets (for non-GCP secrets):**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
  namespace: media-gateway
type: Opaque
data:
  pubnub-publish-key: <base64-encoded>
  pubnub-subscribe-key: <base64-encoded>

---
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
      - name: api-server
        env:
        - name: PUBNUB_PUBLISH_KEY
          valueFrom:
            secretKeyRef:
              name: app-secrets
              key: pubnub-publish-key
```

**Secret Manager Integration:**
```yaml
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: gcp-secrets
spec:
  provider: gcp
  parameters:
    secrets: |
      - resourceName: "projects/PROJECT_ID/secrets/youtube-api-key/versions/latest"
        path: "youtube-api-key"
      - resourceName: "projects/PROJECT_ID/secrets/db-password/versions/latest"
        path: "db-password"
---
volumes:
- name: secrets-store
  csi:
    driver: secrets-store.csi.k8s.io
    readOnly: true
    volumeAttributes:
      secretProviderClass: "gcp-secrets"
```

**Application Code (Rust):**
```rust
use std::env;
use std::fs;

pub struct SecretManager;

impl SecretManager {
    pub fn get_secret(key: &str) -> Result<String, Error> {
        // Priority order:
        // 1. Environment variable (for local dev)
        // 2. Mounted secret file (for production)
        // 3. GCP Secret Manager API (fallback)

        if let Ok(value) = env::var(key) {
            return Ok(value);
        }

        let secret_path = format!("/mnt/secrets/{}", key);
        if let Ok(value) = fs::read_to_string(&secret_path) {
            return Ok(value.trim().to_string());
        }

        // Fallback to API call (requires proper IAM permissions)
        Self::fetch_from_gcp(key)
    }
}
```

### 3.3 Audit Logging Requirements

#### 3.3.1 Security Event Logging

**Event Categories:**
- Authentication events (login, logout, token refresh)
- Authorization failures (insufficient permissions)
- Data access (PII queries, preference updates)
- Administrative actions (user management, config changes)
- Security incidents (brute force, token replay)

**Log Structure:**
```json
{
  "timestamp": "2025-12-06T00:00:00.123Z",
  "event_type": "authentication_success",
  "severity": "INFO",
  "user_id": "user-uuid",
  "device_id": "device-uuid",
  "ip_address": "203.0.113.42",
  "user_agent": "MediaGateway-iOS/1.2.3",
  "geo_location": {
    "country": "US",
    "region": "CA",
    "city": "San Francisco"
  },
  "metadata": {
    "auth_method": "oauth2_pkce",
    "scopes_granted": ["read:content", "write:preferences"],
    "session_id": "session-uuid"
  },
  "trace_id": "trace-uuid",
  "span_id": "span-uuid"
}
```

**Critical Events (Always Logged):**
```yaml
AUTHENTICATION_FAILURE:
  severity: WARNING
  retention: 90_days
  fields: [user_id, ip_address, reason, attempt_count]

TOKEN_REPLAY_DETECTED:
  severity: CRITICAL
  retention: 365_days
  fields: [user_id, device_id, token_id, original_issue_time]
  actions: [revoke_all_tokens, notify_user, alert_security_team]

PRIVILEGE_ESCALATION_ATTEMPT:
  severity: CRITICAL
  retention: 365_days
  fields: [user_id, requested_resource, required_permission, actual_permission]
  actions: [block_request, alert_security_team]

DATA_EXPORT_REQUEST:
  severity: INFO
  retention: 180_days
  fields: [user_id, data_types, record_count, export_format]

ADMIN_ACTION:
  severity: INFO
  retention: 365_days
  fields: [admin_user_id, action_type, target_resource, changes]
```

#### 3.3.2 Logging Infrastructure

**Technology Stack:**
- **Collection:** Fluentd/Fluent Bit on GKE nodes
- **Transport:** gRPC to Google Cloud Logging
- **Storage:** Cloud Logging with log sinks to BigQuery
- **Analysis:** BigQuery SQL queries + Looker dashboards
- **Alerting:** Cloud Monitoring with PagerDuty integration

**Log Retention Policy:**
```yaml
security_events:
  critical: 365_days
  warning: 90_days
  info: 30_days

audit_trail:
  authentication: 90_days
  authorization: 90_days
  data_access: 180_days
  admin_actions: 365_days

application_logs:
  error: 30_days
  info: 7_days
  debug: 1_day
```

**Compliance Export:**
```sql
-- BigQuery export for compliance audits
CREATE OR REPLACE TABLE compliance_exports.auth_events_2025 AS
SELECT
  timestamp,
  event_type,
  user_id,
  ip_address,
  metadata
FROM `project.logs.security_events`
WHERE event_type IN (
  'authentication_success',
  'authentication_failure',
  'token_issued',
  'token_revoked'
)
AND timestamp BETWEEN '2025-01-01' AND '2025-12-31'
ORDER BY timestamp DESC;
```

### 3.4 Rate Limiting and Abuse Prevention

#### 3.4.1 Rate Limit Tiers

**User API Endpoints:**
```yaml
tier_free:
  requests_per_minute: 60
  requests_per_hour: 1000
  requests_per_day: 10000
  burst_capacity: 10

tier_premium:
  requests_per_minute: 300
  requests_per_hour: 10000
  requests_per_day: 100000
  burst_capacity: 50

tier_partner:
  requests_per_minute: 1000
  requests_per_hour: 50000
  requests_per_day: 500000
  burst_capacity: 200
```

**MCP Tool-Specific Limits:**
```yaml
semantic_search:
  capacity: 100
  refill_rate: 10_per_second

recommendations:
  capacity: 50
  refill_rate: 5_per_second

availability_check:
  capacity: 200
  refill_rate: 20_per_second

rights_validation:
  capacity: 200
  refill_rate: 20_per_second
```

#### 3.4.2 Rate Limit Implementation

**Token Bucket Algorithm (Rust):**
```rust
use std::time::{Duration, Instant};

pub struct TokenBucket {
    capacity: u32,
    tokens: u32,
    refill_rate: u32, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn consume(&mut self, tokens: u32) -> Result<(), RateLimitError> {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            Ok(())
        } else {
            Err(RateLimitError {
                retry_after: self.time_until_available(tokens),
                limit: self.capacity,
                remaining: self.tokens,
            })
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = (elapsed * self.refill_rate as f64) as u32;

        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }
}
```

**HTTP Headers:**
```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1733453400
X-RateLimit-Bucket: user:{user-id}:api
Retry-After: 15
```

**Rate Limit Error Response:**
```json
{
  "error": "rate_limit_exceeded",
  "message": "API rate limit exceeded for tier: free",
  "details": {
    "limit": 60,
    "remaining": 0,
    "reset_at": "2025-12-06T01:00:00Z",
    "retry_after_seconds": 45
  },
  "upgrade_url": "https://media-gateway.io/pricing"
}
```

#### 3.4.3 Cloud Armor WAF Rules

**OWASP Top 10 Protection:**
```yaml
security_policy:
  name: media-gateway-waf
  rules:
    - priority: 1
      description: "Block SQL injection attempts"
      match:
        expr: "evaluatePreconfiguredExpr('sqli-v33-stable')"
      action: deny(403)

    - priority: 2
      description: "Block XSS attempts"
      match:
        expr: "evaluatePreconfiguredExpr('xss-v33-stable')"
      action: deny(403)

    - priority: 3
      description: "Block remote code execution"
      match:
        expr: "evaluatePreconfiguredExpr('rce-v33-stable')"
      action: deny(403)

    - priority: 10
      description: "Rate limit by IP"
      match:
        versioned_expr: SRC_IPS_V1
        config:
          src_ip_ranges: ["0.0.0.0/0"]
      rate_limit_options:
        conform_action: allow
        exceed_action: deny(429)
        enforce_on_key: IP
        rate_limit_threshold:
          count: 1000
          interval_sec: 60
```

**Adaptive Protection:**
```yaml
adaptive_protection:
  enabled: true
  layer_7_ddos_defense:
    enabled: true
    rule_visibility: STANDARD
  auto_deploy_confidence_threshold: 0.5
```

**Geographic Restrictions (Optional):**
```yaml
# Block high-risk regions for admin endpoints
- priority: 5
  description: "Block admin access from restricted regions"
  match:
    expr: |
      request.path.startsWith('/admin/') &&
      origin.region_code in ['XX', 'YY']
  action: deny(403)
```

---

## 4. Platform-Specific Authentication

### 4.1 YouTube API Authentication

#### 4.1.1 OAuth 2.0 Configuration

**Client Registration:**
- **Console:** Google Cloud Console → APIs & Services → Credentials
- **Application Type:** Web application / TV and Limited Input devices
- **Authorized Redirect URIs:**
  - Web: `https://media-gateway.io/auth/youtube/callback`
  - Device: Not applicable (uses device flow)

**OAuth Scopes:**
```yaml
required_scopes:
  - https://www.googleapis.com/auth/youtube.readonly
  - https://www.googleapis.com/auth/youtube.force-ssl

optional_scopes:
  - https://www.googleapis.com/auth/youtube # Full access (not recommended)
  - https://www.googleapis.com/auth/youtubepartner # Partner access
```

#### 4.1.2 YouTube OAuth Flow

**Web/Mobile (PKCE):**
```
Authorization Endpoint:
https://accounts.google.com/o/oauth2/v2/auth
  ?client_id={CLIENT_ID}
  &redirect_uri={REDIRECT_URI}
  &response_type=code
  &scope=https://www.googleapis.com/auth/youtube.readonly
  &code_challenge={CODE_CHALLENGE}
  &code_challenge_method=S256
  &state={CSRF_TOKEN}
  &access_type=offline  # Request refresh token
  &prompt=consent        # Force consent screen

Token Endpoint:
POST https://oauth2.googleapis.com/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code
&code={AUTHORIZATION_CODE}
&redirect_uri={REDIRECT_URI}
&client_id={CLIENT_ID}
&client_secret={CLIENT_SECRET}
&code_verifier={CODE_VERIFIER}
```

**Smart TV (Device Flow):**
```
1. Device Code Request:
POST https://oauth2.googleapis.com/device/code
Content-Type: application/x-www-form-urlencoded

client_id={CLIENT_ID}
&scope=https://www.googleapis.com/auth/youtube.readonly

2. Response:
{
  "device_code": "AH-1Ng3cJW8...",
  "user_code": "GQVQ-JKEC",
  "verification_url": "https://www.google.com/device",
  "expires_in": 1800,
  "interval": 5
}

3. Token Polling:
POST https://oauth2.googleapis.com/token
Content-Type: application/x-www-form-urlencoded

client_id={CLIENT_ID}
&client_secret={CLIENT_SECRET}
&device_code={DEVICE_CODE}
&grant_type=urn:ietf:params:oauth:grant-type:device_code
```

#### 4.1.3 API Key Management

**YouTube Data API v3:**
- **API Key Type:** Server key or browser key (for public data only)
- **Quota:** 10,000 units/day (free tier)
- **Rate Limit:** 100 queries/100 seconds/user
- **Restrictions:**
  - HTTP referrer restrictions (for web)
  - IP address restrictions (for server)
  - API restrictions (YouTube Data API v3 only)

**Quota Management:**
```rust
pub struct YouTubeQuotaManager {
    daily_quota: u32,
    remaining_quota: u32,
    reset_time: DateTime<Utc>,
}

impl YouTubeQuotaManager {
    pub async fn check_quota(&mut self, cost: u32) -> Result<()> {
        if Utc::now() > self.reset_time {
            self.remaining_quota = self.daily_quota;
            self.reset_time = Utc::now().date_naive().succ_opt()
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
        }

        if self.remaining_quota >= cost {
            self.remaining_quota -= cost;
            Ok(())
        } else {
            Err(Error::QuotaExceeded {
                reset_at: self.reset_time,
            })
        }
    }
}
```

### 4.2 Twitch OAuth Requirements

#### 4.2.1 OAuth 2.0 Configuration

**Client Registration:**
- **Console:** Twitch Developer Console → Applications
- **OAuth Redirect URLs:** `https://media-gateway.io/auth/twitch/callback`
- **Application Category:** Website Integration

**OAuth Flows:**
```yaml
authorization_code_flow:
  endpoint: https://id.twitch.tv/oauth2/authorize
  scopes:
    - user:read:email
    - user:read:follows
    - user:read:subscriptions

client_credentials_flow:
  endpoint: https://id.twitch.tv/oauth2/token
  use_case: App access token (no user context)
  scopes: []  # App access tokens have no scopes
```

#### 4.2.2 Twitch Authentication Flow

**Authorization Code Flow:**
```
1. Authorization Request:
https://id.twitch.tv/oauth2/authorize
  ?client_id={CLIENT_ID}
  &redirect_uri={REDIRECT_URI}
  &response_type=code
  &scope=user:read:email+user:read:follows
  &state={CSRF_TOKEN}
  &force_verify=false

2. Token Exchange:
POST https://id.twitch.tv/oauth2/token
Content-Type: application/x-www-form-urlencoded

client_id={CLIENT_ID}
&client_secret={CLIENT_SECRET}
&code={AUTHORIZATION_CODE}
&grant_type=authorization_code
&redirect_uri={REDIRECT_URI}

3. Response:
{
  "access_token": "rfx2uswqe8l4g1mkagrvg5tv0ks3",
  "expires_in": 14124,
  "refresh_token": "5b93chm6hdve3mycz05zfzatkfdenfspp1h1ar2xxdalen01",
  "scope": ["user:read:email", "user:read:follows"],
  "token_type": "bearer"
}
```

**Client Credentials Flow (App Access Token):**
```
POST https://id.twitch.tv/oauth2/token
Content-Type: application/x-www-form-urlencoded

client_id={CLIENT_ID}
&client_secret={CLIENT_SECRET}
&grant_type=client_credentials
&scope=  # Optional scopes for app access

Response:
{
  "access_token": "jostpf5q0puzmxmkba9iyug38kjtgh",
  "expires_in": 5011271,
  "token_type": "bearer"
}
```

#### 4.2.3 Twitch API Headers

**Required Headers:**
```http
Authorization: Bearer {ACCESS_TOKEN}
Client-Id: {CLIENT_ID}
Content-Type: application/json
```

**Token Validation:**
```
GET https://id.twitch.tv/oauth2/validate
Authorization: Bearer {ACCESS_TOKEN}

Response:
{
  "client_id": "wbmytr93xzw8zbg0p1izqyzzc5mbiz",
  "login": "twitchuser",
  "scopes": ["user:read:email"],
  "user_id": "44322889",
  "expires_in": 5520838
}
```

### 4.3 Social Platform Token Management

#### 4.3.1 Aggregator API Authentication

**Streaming Availability API:**
```yaml
authentication:
  method: RapidAPI Key
  header: X-RapidAPI-Key
  endpoint: https://streaming-availability.p.rapidapi.com/
  rate_limit: 100 requests/day (free tier)

example_request:
  curl -X GET "https://streaming-availability.p.rapidapi.com/get" \
    -H "X-RapidAPI-Key: {API_KEY}" \
    -H "X-RapidAPI-Host: streaming-availability.p.rapidapi.com"
```

**Watchmode API:**
```yaml
authentication:
  method: API Key (query parameter)
  parameter: apiKey
  endpoint: https://api.watchmode.com/v1/
  rate_limit: 1000 requests/day (free tier)

example_request:
  curl "https://api.watchmode.com/v1/title/{title_id}/details/?apiKey={API_KEY}"
```

**JustWatch API:**
```yaml
authentication:
  method: None (public API)
  endpoint: https://apis.justwatch.com/
  rate_limit: Unspecified (fair use)
  note: Unofficial API, may change without notice
```

#### 4.3.2 Token Storage and Rotation

**Database Schema:**
```sql
CREATE TABLE platform_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform VARCHAR(50) NOT NULL, -- 'youtube', 'twitch', 'rapidapi'
    credential_type VARCHAR(20) NOT NULL, -- 'oauth_token', 'api_key'
    user_id UUID REFERENCES users(id), -- NULL for service credentials
    access_token TEXT,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ,
    scopes TEXT[],
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Constraints
    CONSTRAINT valid_expiry CHECK (expires_at > created_at)
);

CREATE INDEX idx_platform_creds_user ON platform_credentials(user_id, platform);
CREATE INDEX idx_platform_creds_expiry ON platform_credentials(expires_at)
    WHERE expires_at IS NOT NULL;
```

**Automatic Refresh Job:**
```rust
pub struct TokenRefreshJob;

#[async_trait]
impl Job for TokenRefreshJob {
    async fn run(&self, ctx: &JobContext) -> Result<()> {
        let expiring_tokens = ctx.db
            .query(
                "SELECT * FROM platform_credentials
                 WHERE expires_at < NOW() + INTERVAL '1 hour'
                 AND refresh_token IS NOT NULL"
            )
            .await?;

        for token in expiring_tokens {
            match self.refresh_token(&token).await {
                Ok(new_token) => {
                    ctx.db.update_token(token.id, &new_token).await?;
                    info!("Refreshed token for user {}", token.user_id);
                }
                Err(e) => {
                    error!("Failed to refresh token: {:?}", e);
                    // Notify user to re-authenticate
                    self.notify_user_reauth_required(token.user_id).await?;
                }
            }
        }

        Ok(())
    }
}
```

### 4.4 Stream Key Security

#### 4.4.1 Stream Key Generation

**IMPORTANT:** Media Gateway is a content discovery platform, NOT a streaming platform. Users do NOT stream content through the gateway. This section documents stream key security for reference only, in case future features involve user-generated content or live interaction.

**Stream Key Format:**
```
{platform}-{user_id}-{random_suffix}-{checksum}

Example: mgw-550e8400-e29b-41d4-a716-446655440000-x7k2p9-a3f1
```

**Generation Algorithm:**
```rust
use rand::{Rng, thread_rng};
use sha2::{Sha256, Digest};

pub fn generate_stream_key(user_id: &Uuid) -> String {
    let mut rng = thread_rng();
    let random_suffix: String = (0..6)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();

    let payload = format!("mgw-{}-{}", user_id, random_suffix);

    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let checksum = format!("{:x}", hasher.finalize())[..4].to_string();

    format!("{}-{}", payload, checksum)
}
```

#### 4.4.2 Stream Key Storage

**Encrypted Storage:**
```sql
CREATE TABLE stream_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    encrypted_key TEXT NOT NULL, -- AES-256-GCM encrypted
    key_hash TEXT NOT NULL, -- SHA-256 hash for lookup
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,

    UNIQUE(key_hash)
);

CREATE INDEX idx_stream_keys_user ON stream_keys(user_id) WHERE revoked_at IS NULL;
```

**Encryption Implementation:**
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct StreamKeyManager {
    cipher: Aes256Gcm,
}

impl StreamKeyManager {
    pub fn encrypt_key(&self, plain_key: &str) -> Result<String> {
        let nonce = Nonce::from_slice(b"unique nonce"); // Use random nonce
        let ciphertext = self.cipher
            .encrypt(nonce, plain_key.as_bytes())
            .map_err(|e| Error::EncryptionFailed)?;

        Ok(base64::encode(ciphertext))
    }

    pub fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
```

#### 4.4.3 Stream Key Validation

**Validation Middleware:**
```rust
pub async fn validate_stream_key(
    key: &str,
    db: &Database,
) -> Result<UserId> {
    // 1. Hash the provided key
    let key_hash = hash_key(key);

    // 2. Lookup in database
    let stream_key = db.query_one(
        "SELECT user_id, revoked_at FROM stream_keys
         WHERE key_hash = $1",
        &[&key_hash]
    ).await?;

    // 3. Check revocation status
    if stream_key.revoked_at.is_some() {
        return Err(Error::StreamKeyRevoked);
    }

    // 4. Update last_used_at timestamp
    db.execute(
        "UPDATE stream_keys SET last_used_at = NOW()
         WHERE key_hash = $1",
        &[&key_hash]
    ).await?;

    // 5. Return user_id
    Ok(stream_key.user_id)
}
```

**Rate Limiting Per Stream Key:**
```rust
pub struct StreamKeyRateLimiter {
    redis: RedisClient,
}

impl StreamKeyRateLimiter {
    pub async fn check_limit(&self, key: &str) -> Result<()> {
        let redis_key = format!("stream_key:{}:rate_limit", hash_key(key));

        let count: i64 = self.redis
            .incr(&redis_key, 1)
            .await?;

        if count == 1 {
            self.redis.expire(&redis_key, 60).await?; // 1-minute window
        }

        if count > 10 {
            Err(Error::RateLimitExceeded)
        } else {
            Ok(())
        }
    }
}
```

---

## 5. Multi-Tenant Architecture

### 5.1 Tenant Isolation Strategy

#### 5.1.1 Isolation Levels

**Level 1: Network Isolation**
- Separate GKE namespaces per tenant tier (free, premium, enterprise)
- Network policies restrict inter-namespace communication
- Istio virtual services route traffic by tenant

**Level 2: Data Isolation**
- Row-Level Security (RLS) in PostgreSQL
- Tenant-scoped partitions for high-volume tables
- Separate encryption keys per enterprise tenant

**Level 3: Compute Isolation**
- Resource quotas per namespace
- Dedicated node pools for enterprise tenants
- CPU/memory limits per tenant tier

#### 5.1.2 Database Multi-Tenancy

**Shared Database with RLS:**
```sql
-- Enable RLS on user data tables
ALTER TABLE preferences ENABLE ROW LEVEL SECURITY;

-- Policy: Users can only access their own data
CREATE POLICY user_preferences_isolation ON preferences
FOR ALL
TO application_role
USING (user_id = current_setting('app.current_user_id')::uuid);

-- Application sets user context
BEGIN;
SET LOCAL app.current_user_id = 'user-uuid';
SELECT * FROM preferences; -- Only returns current user's data
COMMIT;
```

**Partitioned Tables (Enterprise):**
```sql
-- Partition by enterprise tenant
CREATE TABLE enterprise_analytics (
    tenant_id UUID NOT NULL,
    event_time TIMESTAMPTZ NOT NULL,
    event_data JSONB,
    ...
) PARTITION BY HASH (tenant_id);

-- Create partitions per tenant
CREATE TABLE enterprise_analytics_tenant_1
    PARTITION OF enterprise_analytics
    FOR VALUES WITH (MODULUS 10, REMAINDER 0);
```

### 5.2 Cross-Tenant Communication

#### 5.2.1 Shared Content Catalog

**Public Content Metadata:**
- Shared across all tenants (no isolation)
- Read-only access for all users
- Single source of truth for content data

**Tenant-Specific Annotations:**
```sql
CREATE TABLE content_metadata (
    content_id UUID PRIMARY KEY,
    title VARCHAR(500),
    description TEXT,
    release_year INT,
    -- Shared metadata (no tenant_id)
);

CREATE TABLE tenant_content_annotations (
    id UUID PRIMARY KEY,
    content_id UUID REFERENCES content_metadata(id),
    tenant_id UUID NOT NULL,
    custom_tags TEXT[],
    internal_notes TEXT,
    -- Tenant-specific data
    UNIQUE(content_id, tenant_id)
);
```

#### 5.2.2 Watchlist Sharing Across Tenants

**Permission Model:**
```sql
CREATE TABLE watchlist_shares (
    id UUID PRIMARY KEY,
    watchlist_id UUID REFERENCES watchlists(id),
    owner_user_id UUID NOT NULL,
    shared_with_user_id UUID NOT NULL,
    permission_level VARCHAR(20) NOT NULL, -- 'viewer', 'editor'
    shared_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,

    -- Prevent sharing with self
    CONSTRAINT no_self_share CHECK (owner_user_id != shared_with_user_id)
);
```

**Cross-Tenant Share Validation:**
```rust
pub async fn share_watchlist(
    owner_id: Uuid,
    target_user_id: Uuid,
    watchlist_id: Uuid,
    permission: PermissionLevel,
) -> Result<()> {
    // 1. Verify owner has permission to share
    let watchlist = db.get_watchlist(watchlist_id).await?;
    if watchlist.user_id != owner_id {
        return Err(Error::Forbidden);
    }

    // 2. Verify target user exists (cross-tenant check)
    let target_user = db.get_user(target_user_id).await?;

    // 3. Create share record
    db.create_watchlist_share(WatchlistShare {
        watchlist_id,
        owner_user_id: owner_id,
        shared_with_user_id: target_user_id,
        permission_level: permission,
        expires_at: Some(Utc::now() + Duration::days(30)),
    }).await?;

    Ok(())
}
```

### 5.3 Tenant-Specific Configuration

#### 5.3.1 Feature Flags

**Database Schema:**
```sql
CREATE TABLE tenant_features (
    tenant_id UUID NOT NULL,
    feature_key VARCHAR(100) NOT NULL,
    enabled BOOLEAN DEFAULT false,
    config JSONB,
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    PRIMARY KEY (tenant_id, feature_key)
);

-- Example feature flags
INSERT INTO tenant_features (tenant_id, feature_key, enabled, config) VALUES
('enterprise-tenant-1', 'advanced_analytics', true, '{"retention_days": 365}'),
('enterprise-tenant-1', 'custom_branding', true, '{"logo_url": "..."}'),
('free-tenant-2', 'advanced_analytics', false, NULL);
```

**Feature Flag Middleware:**
```rust
pub struct FeatureGuard {
    feature_key: String,
    required_tiers: Vec<TenantTier>,
}

impl FeatureGuard {
    pub async fn check(&self, tenant_id: Uuid) -> Result<()> {
        let feature = db.get_tenant_feature(tenant_id, &self.feature_key).await?;

        if !feature.enabled {
            return Err(Error::FeatureNotAvailable {
                feature: self.feature_key.clone(),
                upgrade_url: "/pricing",
            });
        }

        Ok(())
    }
}
```

#### 5.3.2 Tenant Resource Quotas

**Kubernetes ResourceQuota:**
```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: tenant-free-quota
  namespace: tenant-free
spec:
  hard:
    requests.cpu: "10"
    requests.memory: 20Gi
    limits.cpu: "20"
    limits.memory: 40Gi
    persistentvolumeclaims: "5"
    services.loadbalancers: "0"

---
apiVersion: v1
kind: ResourceQuota
metadata:
  name: tenant-enterprise-quota
  namespace: tenant-enterprise-1
spec:
  hard:
    requests.cpu: "100"
    requests.memory: 200Gi
    limits.cpu: "200"
    limits.memory: 400Gi
    persistentvolumeclaims: "50"
    services.loadbalancers: "5"
```

**Application-Level Quotas:**
```sql
CREATE TABLE tenant_quotas (
    tenant_id UUID PRIMARY KEY,
    max_users INT,
    max_watchlists_per_user INT,
    max_api_requests_per_day INT,
    max_storage_gb INT,
    current_users INT DEFAULT 0,
    current_storage_gb DECIMAL(10,2) DEFAULT 0,

    CONSTRAINT quota_limits CHECK (
        current_users <= max_users AND
        current_storage_gb <= max_storage_gb
    )
);
```

---

## 6. Compliance and Audit Requirements

### 6.1 Regulatory Compliance

#### 6.1.1 GDPR Compliance

**Data Subject Rights:**
```yaml
right_to_access:
  endpoint: POST /api/v1/users/{user_id}/data-export
  response_time: 30_days
  format: JSON, CSV
  includes: [profile, preferences, watch_history, ratings]

right_to_erasure:
  endpoint: DELETE /api/v1/users/{user_id}
  response_time: 30_days
  process:
    - anonymize_user_data
    - revoke_all_tokens
    - delete_pii
    - retain_audit_logs (legal hold)

right_to_rectification:
  endpoint: PATCH /api/v1/users/{user_id}
  validation: user_verified
  audit: true
```

**Consent Management:**
```sql
CREATE TABLE user_consents (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    consent_type VARCHAR(50) NOT NULL, -- 'marketing', 'analytics', 'personalization'
    granted BOOLEAN NOT NULL,
    ip_address INET,
    user_agent TEXT,
    consented_at TIMESTAMPTZ DEFAULT NOW(),
    withdrawn_at TIMESTAMPTZ,

    -- Audit trail
    version INT NOT NULL, -- Consent policy version
    policy_url TEXT NOT NULL
);
```

#### 6.1.2 CCPA Compliance

**Do Not Sell Preference:**
```sql
CREATE TABLE privacy_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    do_not_sell BOOLEAN DEFAULT false,
    opt_out_of_targeted_ads BOOLEAN DEFAULT false,
    limit_use_of_sensitive_data BOOLEAN DEFAULT false,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Data Sale Disclosure:**
```yaml
data_sharing_categories:
  personal_identifiers: false # Email, user_id - NOT sold
  commercial_information: false # Purchase history - NOT applicable
  internet_activity: true # Anonymized viewing patterns - Shared with partners
  geolocation: false # Coarse region only - NOT sold
  inferences: true # Anonymized preferences - Used for recommendations
```

### 6.2 Audit Trail Implementation

#### 6.2.1 Comprehensive Audit Logging

**Audit Event Schema:**
```sql
CREATE TABLE audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_time TIMESTAMPTZ DEFAULT NOW(),
    event_type VARCHAR(100) NOT NULL,
    actor_type VARCHAR(20) NOT NULL, -- 'user', 'service', 'admin'
    actor_id UUID,
    target_type VARCHAR(50), -- 'user', 'watchlist', 'preference'
    target_id UUID,
    action VARCHAR(50) NOT NULL, -- 'create', 'read', 'update', 'delete'
    result VARCHAR(20) NOT NULL, -- 'success', 'failure', 'denied'
    metadata JSONB,
    ip_address INET,
    user_agent TEXT,

    -- Partitioned by month for performance
    CONSTRAINT audit_time_check CHECK (event_time >= '2025-01-01')
) PARTITION BY RANGE (event_time);

-- Create monthly partitions
CREATE TABLE audit_events_2025_12 PARTITION OF audit_events
FOR VALUES FROM ('2025-12-01') TO ('2026-01-01');
```

**Critical Events Always Audited:**
```yaml
authentication_events:
  - login_success
  - login_failure
  - logout
  - token_issued
  - token_refreshed
  - token_revoked

authorization_events:
  - permission_granted
  - permission_denied
  - role_changed

data_access_events:
  - pii_accessed
  - bulk_export_requested
  - search_query_executed

administrative_events:
  - user_created
  - user_deleted
  - config_changed
  - feature_flag_toggled
```

#### 6.2.2 Tamper-Proof Audit Logs

**Write-Once Storage:**
```sql
-- Prevent updates and deletes on audit table
CREATE POLICY audit_append_only ON audit_events
FOR ALL
TO application_role
USING (false)
WITH CHECK (true);

-- Only allow inserts
GRANT INSERT ON audit_events TO application_role;
REVOKE UPDATE, DELETE ON audit_events FROM application_role;
```

**Cryptographic Hashing (Merkle Tree):**
```rust
pub struct AuditHashChain {
    previous_hash: String,
}

impl AuditHashChain {
    pub fn hash_event(&self, event: &AuditEvent) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(serde_json::to_string(event).unwrap().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub async fn verify_chain(&self, db: &Database) -> Result<bool> {
        let events = db.query("SELECT * FROM audit_events ORDER BY event_time ASC").await?;

        let mut computed_hash = "genesis".to_string();
        for event in events {
            let expected_hash = event.hash_chain;
            let actual_hash = self.hash_event(&event);

            if expected_hash != actual_hash {
                return Ok(false); // Tampering detected
            }

            computed_hash = actual_hash;
        }

        Ok(true)
    }
}
```

### 6.3 Security Monitoring

#### 6.3.1 Real-Time Alerting

**Cloud Monitoring Alert Policies:**
```yaml
authentication_anomalies:
  condition: failed_login_attempts > 5 in 5 minutes
  severity: WARNING
  notification: security_team@media-gateway.io

token_replay_detected:
  condition: token_replay_count > 0
  severity: CRITICAL
  notification: pagerduty
  actions:
    - revoke_all_user_tokens
    - block_ip_address
    - require_password_reset

privilege_escalation:
  condition: unauthorized_admin_access_attempt
  severity: CRITICAL
  notification: pagerduty, security_team
  actions:
    - lock_account
    - create_incident_ticket

unusual_api_patterns:
  condition: api_requests_per_minute > 1000
  severity: WARNING
  notification: devops_team
  actions:
    - enable_aggressive_rate_limiting
```

#### 6.3.2 Security Dashboards

**Looker Dashboard Metrics:**
```yaml
authentication_metrics:
  - total_logins_24h
  - failed_login_rate
  - new_device_registrations
  - token_refresh_rate
  - geographic_login_distribution

authorization_metrics:
  - permission_denied_count
  - role_escalation_attempts
  - cross_tenant_access_attempts

threat_detection:
  - suspicious_ip_addresses
  - brute_force_attempts
  - token_replay_incidents
  - sql_injection_attempts (WAF blocked)
```

**BigQuery Analysis Queries:**
```sql
-- Detect unusual login patterns
SELECT
  user_id,
  COUNT(DISTINCT ip_address) as unique_ips,
  COUNT(DISTINCT country) as unique_countries,
  COUNT(*) as login_count
FROM audit_events
WHERE event_type = 'login_success'
  AND event_time > TIMESTAMP_SUB(CURRENT_TIMESTAMP(), INTERVAL 24 HOUR)
GROUP BY user_id
HAVING unique_countries > 2  -- Logins from 3+ countries in 24h
ORDER BY login_count DESC;
```

---

## Appendix A: Security Checklist

### Pre-Production Security Audit

- [ ] OAuth 2.0 flows tested with PKCE enabled
- [ ] Device authorization grant tested on all TV platforms
- [ ] mTLS certificates generated and rotated
- [ ] Workload Identity bindings verified for all services
- [ ] Secret Manager secrets created and permissions assigned
- [ ] Token refresh automation tested with expiry scenarios
- [ ] Token revocation propagates within 5 seconds
- [ ] Rate limiting enforced at API gateway and application layers
- [ ] Cloud Armor WAF rules deployed and tested
- [ ] GDPR data export/erasure endpoints functional
- [ ] Audit logging enabled for all critical events
- [ ] Monitoring alerts configured and tested
- [ ] Security incident response playbook documented
- [ ] Penetration testing completed by third party
- [ ] Dependency vulnerability scans passing

---

## Appendix B: Security Contacts

**Security Team:**
- Security Lead: security@media-gateway.io
- PagerDuty: Critical alerts only
- Bug Bounty: [HackerOne Program URL]

**Incident Response:**
- Severity 1 (Critical): PagerDuty + Slack #security-incidents
- Severity 2 (High): Email security team + JIRA ticket
- Severity 3 (Medium): JIRA ticket only

---

## Appendix C: Credential Rotation Schedule

| Credential Type | Rotation Frequency | Automation | Notification |
|----------------|-------------------|------------|--------------|
| OAuth Client Secrets | 180 days | Manual | 14 days before |
| API Keys (YouTube, Twitch) | 90 days | Automated | 7 days before |
| Database Passwords | 30 days | Automated | N/A |
| TLS Certificates | 90 days | Automated (cert-manager) | N/A |
| Service Account Keys | 365 days | Manual | 30 days before |
| User Access Tokens | 1 hour | Automatic refresh | N/A |
| User Refresh Tokens | 90 days (rolling) | Automatic rotation | N/A |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-12-06 | Research Agent | Initial specification based on repository analysis |

---

**End of Specification**
