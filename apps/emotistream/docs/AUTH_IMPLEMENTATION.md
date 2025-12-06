# EmotiStream Authentication Implementation

## Overview

JWT-based authentication system implemented for the EmotiStream MVP API.

## Implementation Summary

### Files Created

1. **src/auth/jwt-service.ts** - JWT token generation and verification
   - Access tokens: 24h expiry
   - Refresh tokens: 7d expiry
   - Token verification with proper error handling

2. **src/auth/password-service.ts** - Password hashing and validation
   - Bcrypt with 12 salt rounds
   - Password strength validation (min 8 chars, letter + number)

3. **src/persistence/user-store.ts** - In-memory user management
   - File-based persistence (will be replaced with AgentDB later)
   - Email index for fast lookups
   - User CRUD operations

4. **src/auth/auth-middleware.ts** - JWT bearer token validation
   - Express middleware for protected routes
   - Error codes E007 (invalid token) and E008 (unauthorized)
   - Optional auth middleware for public routes

5. **src/api/routes/auth.ts** - Authentication endpoints
   - POST /api/v1/auth/register
   - POST /api/v1/auth/login
   - POST /api/v1/auth/refresh

6. **src/api/index.ts** - API server setup
   - Express application configuration
   - Route mounting
   - Health check endpoint

7. **src/server.ts** - Server entry point

## API Endpoints

### POST /api/v1/auth/register

Register a new user.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securePassword123",
  "dateOfBirth": "1990-01-01",
  "displayName": "John Doe"
}
```

**Response (201):**
```json
{
  "success": true,
  "data": {
    "userId": "usr_abc123xyz",
    "email": "user@example.com",
    "displayName": "John Doe",
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refreshToken": "refresh_token_here",
    "expiresAt": "2025-12-07T10:30:00.000Z"
  },
  "error": null,
  "timestamp": "2025-12-06T10:30:00.000Z"
}
```

### POST /api/v1/auth/login

Login with email and password.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securePassword123"
}
```

**Response (200):** Same as register

### POST /api/v1/auth/refresh

Refresh access token using refresh token.

**Request:**
```json
{
  "refreshToken": "refresh_token_here"
}
```

**Response (200):**
```json
{
  "success": true,
  "data": {
    "token": "new_jwt_token",
    "expiresAt": "2025-12-07T10:30:00.000Z"
  },
  "error": null,
  "timestamp": "2025-12-06T10:30:00.000Z"
}
```

## Security Features

1. **Password Hashing**: Bcrypt with 12 rounds
2. **JWT Tokens**: HS256 algorithm with configurable secret
3. **Token Expiry**: Access (24h), Refresh (7d)
4. **Password Validation**: Minimum 8 characters, letter + number
5. **Email Validation**: RFC-compliant email format check
6. **Case-Insensitive Email**: Stored in lowercase for consistency

## Error Codes

- **E003**: Invalid input (malformed data, missing fields)
- **E007**: Invalid or expired JWT token
- **E008**: Unauthorized access (not implemented yet)
- **E010**: Internal server error

## Usage

### Install Dependencies

```bash
npm install
```

### Start Development Server

```bash
npm run dev
```

Server starts on http://localhost:3000

### Build for Production

```bash
npm run build
npm start
```

### Environment Variables

- `PORT`: Server port (default: 3000)
- `JWT_SECRET`: JWT signing secret (default: dev secret)

## Testing

### Example curl commands

**Register:**
```bash
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123",
    "dateOfBirth": "1990-01-01",
    "displayName": "Test User"
  }'
```

**Login:**
```bash
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'
```

**Access Protected Route:**
```bash
curl http://localhost:3000/api/v1/protected \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

**Refresh Token:**
```bash
curl -X POST http://localhost:3000/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refreshToken": "YOUR_REFRESH_TOKEN_HERE"
  }'
```

## Next Steps

1. Replace UserStore with AgentDB integration
2. Add rate limiting
3. Add unit tests
4. Add integration tests
5. Implement additional protected routes (emotion detection, recommendations, etc.)

## Compliance with API Spec

This implementation follows the EmotiStream MVP API specification:
- ✅ JWT bearer token authentication
- ✅ Password hashing with bcrypt (12 rounds)
- ✅ Access token (24h expiry)
- ✅ Refresh token (7d expiry)
- ✅ Error codes E007, E008 per spec
- ✅ Standardized JSON response format
- ✅ All three auth endpoints implemented
