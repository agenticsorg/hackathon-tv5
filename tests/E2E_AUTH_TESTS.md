# Media Gateway E2E Auth Tests

## Overview

Comprehensive end-to-end (E2E) test suite for the Media Gateway authentication system.

## Test Coverage

### 1. test_full_registration_flow
Complete user journey from registration to logout:
- Register new user
- Verify email with token from database
- Login with credentials
- Verify access token works
- Refresh access token
- Verify new token works
- Logout and verify token revocation

### 2. test_email_verification_flow
Email verification edge cases:
- Invalid verification token
- Valid verification token
- Double verification attempt

### 3. test_login_and_refresh_flow
Login scenarios:
- Wrong password
- Correct credentials
- Token refresh
- Invalid refresh token
- Unverified user cannot login

### 4. test_mfa_enrollment_and_verify
Multi-factor authentication:
- Enroll TOTP MFA
- Receive secret and QR code
- Get backup codes (10 codes)
- Verify MFA code
- Login with MFA challenge
- Use backup code for login

### 5. test_password_reset_flow
Password reset process:
- Request password reset
- Get reset token from database
- Reset password with valid token
- Old password no longer works
- New password works
- Used token cannot be reused
- Invalid token fails

### 6. test_oauth_login_flow
OAuth provider integration:
- Google OAuth initiation
- Apple OAuth initiation

### 7. test_admin_user_management
Admin functionality:
- List all users (paginated)
- Get user details
- Update user (verify, activate)
- Generate impersonation token
- Delete user
- Non-admin access forbidden

### 8. test_session_management
Multi-session handling:
- Create multiple sessions
- Verify all sessions work
- Logout from single session
- Other sessions remain active

## Running Tests

```bash
# Run all E2E auth tests
cargo test --test e2e_auth_flow_tests

# Run specific test
cargo test --test e2e_auth_flow_tests test_full_registration_flow

# Run with output
cargo test --test e2e_auth_flow_tests -- --nocapture
```

## Prerequisites

- PostgreSQL running on localhost:5432
- Redis running on localhost:6379
- Auth service running on localhost:8081

## Design Principles

1. Real database connections (no mocks)
2. TDD approach
3. Error path testing
4. Idempotent tests
5. Isolated test execution
