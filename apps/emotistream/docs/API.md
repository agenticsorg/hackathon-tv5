# EmotiStream API Documentation

## Overview

REST API for the EmotiStream emotion-driven content recommendation system.

**Base URL**: `http://localhost:3000/api/v1`

## Authentication

Currently, the API does not require authentication. Future versions will implement JWT-based authentication.

## Rate Limits

- **General API**: 100 requests/minute per IP
- **Emotion Analysis**: 30 requests/minute per IP (more expensive operations)
- **Recommendations**: 60 requests/minute per IP

## Response Format

All API responses follow this structure:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "timestamp": "2025-12-05T22:30:00.000Z"
}
```

Error responses:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Description of the error",
    "details": { ... }
  },
  "timestamp": "2025-12-05T22:30:00.000Z"
}
```

## Endpoints

### Health Check

**GET** `/health`

Check if the API server is running.

**Response**:
```json
{
  "status": "ok",
  "version": "1.0.0",
  "timestamp": "2025-12-05T22:30:00.000Z"
}
```

---

### Emotion Analysis

#### Analyze Emotional State

**POST** `/api/v1/emotion/analyze`

Analyze text input to detect emotional state.

**Request Body**:
```json
{
  "userId": "user-123",
  "text": "I'm feeling really stressed about work and need to relax"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "userId": "user-123",
    "inputText": "I'm feeling really stressed about work and need to relax",
    "state": {
      "valence": -0.4,
      "arousal": 0.3,
      "stressLevel": 0.6,
      "primaryEmotion": "stress",
      "emotionVector": [0.1, 0.2, 0.3, 0.1, 0.5, 0.1, 0.4, 0.2],
      "confidence": 0.85,
      "timestamp": 1733439000000
    },
    "desired": {
      "targetValence": 0.5,
      "targetArousal": -0.2,
      "targetStress": 0.2,
      "intensity": "moderate",
      "reasoning": "Detected high stress. Suggesting calm, positive content."
    }
  },
  "error": null,
  "timestamp": "2025-12-05T22:30:00.000Z"
}
```

**Validation**:
- `userId`: Required, string
- `text`: Required, string, 10-1000 characters

**Rate Limit**: 30 requests/minute

---

#### Get Emotion History

**GET** `/api/v1/emotion/history/:userId`

Get emotional state history for a user.

**Parameters**:
- `userId` (path): User identifier
- `limit` (query, optional): Number of records to return (default: 10)

**Response**:
```json
{
  "success": true,
  "data": {
    "userId": "user-123",
    "history": [],
    "count": 0
  },
  "error": null,
  "timestamp": "2025-12-05T22:30:00.000Z"
}
```

---

### Recommendations

#### Get Content Recommendations

**POST** `/api/v1/recommend`

Get personalized content recommendations based on emotional state.

**Request Body**:
```json
{
  "userId": "user-123",
  "currentState": {
    "valence": -0.4,
    "arousal": 0.3,
    "stressLevel": 0.6,
    "primaryEmotion": "stress",
    "emotionVector": [0.1, 0.2, 0.3, 0.1, 0.5, 0.1, 0.4, 0.2],
    "confidence": 0.85,
    "timestamp": 1733439000000
  },
  "desiredState": {
    "targetValence": 0.5,
    "targetArousal": -0.2,
    "targetStress": 0.2,
    "intensity": "moderate",
    "reasoning": "User wants to relax"
  },
  "limit": 5
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "userId": "user-123",
    "recommendations": [
      {
        "contentId": "content-001",
        "title": "Calm Nature Documentary",
        "qValue": 0.85,
        "similarityScore": 0.92,
        "combinedScore": 0.88,
        "predictedOutcome": {
          "expectedValence": 0.5,
          "expectedArousal": -0.3,
          "expectedStress": 0.2,
          "confidence": 0.87
        },
        "reasoning": "High Q-value for stress reduction. Nature scenes promote relaxation.",
        "isExploration": false
      }
    ],
    "explorationRate": 0.15,
    "timestamp": 1733439000000
  },
  "error": null,
  "timestamp": "2025-12-05T22:30:00.000Z"
}
```

**Validation**:
- `userId`: Required, string
- `currentState`: Required, EmotionalState object
- `desiredState`: Required, DesiredState object
- `limit`: Optional, number, 1-20 (default: 5)

**Rate Limit**: 60 requests/minute

---

#### Get Recommendation History

**GET** `/api/v1/recommend/history/:userId`

Get recommendation history for a user.

**Parameters**:
- `userId` (path): User identifier
- `limit` (query, optional): Number of records to return (default: 10)

**Response**:
```json
{
  "success": true,
  "data": {
    "userId": "user-123",
    "history": [],
    "count": 0
  },
  "error": null,
  "timestamp": "2025-12-05T22:30:00.000Z"
}
```

---

### Feedback

#### Submit Feedback

**POST** `/api/v1/feedback`

Submit post-viewing feedback to update the RL policy.

**Request Body**:
```json
{
  "userId": "user-123",
  "contentId": "content-001",
  "actualPostState": {
    "valence": 0.6,
    "arousal": -0.2,
    "stressLevel": 0.2,
    "primaryEmotion": "relaxed",
    "emotionVector": [0.7, 0.3, 0.1, 0.05, 0.1, 0.05, 0.1, 0.2],
    "confidence": 0.88,
    "timestamp": 1733439000000
  },
  "watchDuration": 45,
  "completed": true,
  "explicitRating": 5
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "reward": 0.75,
    "policyUpdated": true,
    "newQValue": 0.82,
    "learningProgress": {
      "totalExperiences": 15,
      "avgReward": 0.68,
      "explorationRate": 0.12,
      "convergenceScore": 0.45
    }
  },
  "error": null,
  "timestamp": "2025-12-05T22:30:00.000Z"
}
```

**Validation**:
- `userId`: Required, string
- `contentId`: Required, string
- `actualPostState`: Required, EmotionalState object
- `watchDuration`: Required, number (minutes, >= 0)
- `completed`: Required, boolean
- `explicitRating`: Optional, number (1-5)

---

#### Get Learning Progress

**GET** `/api/v1/feedback/progress/:userId`

Get learning progress metrics for a user.

**Parameters**:
- `userId` (path): User identifier

**Response**:
```json
{
  "success": true,
  "data": {
    "userId": "user-123",
    "totalExperiences": 15,
    "avgReward": 0.68,
    "explorationRate": 0.12,
    "convergenceScore": 0.45,
    "recentRewards": [0.75, 0.82, 0.65, 0.71, 0.88]
  },
  "error": null,
  "timestamp": "2025-12-05T22:30:00.000Z"
}
```

---

#### Get Feedback Experiences

**GET** `/api/v1/feedback/experiences/:userId`

Get feedback experiences for a user.

**Parameters**:
- `userId` (path): User identifier
- `limit` (query, optional): Number of records to return (1-100, default: 10)

**Response**:
```json
{
  "success": true,
  "data": {
    "userId": "user-123",
    "experiences": [],
    "count": 0
  },
  "error": null,
  "timestamp": "2025-12-05T22:30:00.000Z"
}
```

---

## Error Codes

| Code | Description |
|------|-------------|
| `VALIDATION_ERROR` | Request validation failed |
| `NOT_FOUND` | Resource not found |
| `RATE_LIMIT_EXCEEDED` | Rate limit exceeded |
| `EMOTION_RATE_LIMIT` | Emotion analysis rate limit exceeded |
| `INTERNAL_ERROR` | Internal server error |

## HTTP Status Codes

- `200 OK`: Successful request
- `400 Bad Request`: Validation error
- `404 Not Found`: Resource not found
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

## Development

### Start the server

```bash
npm run dev
```

### Build for production

```bash
npm run build
npm start
```

### Environment Variables

See `.env.example` for configuration options.

## Testing

### Test with curl

```bash
# Health check
curl http://localhost:3000/health

# Analyze emotion
curl -X POST http://localhost:3000/api/v1/emotion/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "user-123",
    "text": "I feel stressed and need to relax"
  }'

# Get recommendations
curl -X POST http://localhost:3000/api/v1/recommend \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "user-123",
    "currentState": {
      "valence": -0.4,
      "arousal": 0.3,
      "stressLevel": 0.6,
      "primaryEmotion": "stress",
      "emotionVector": [0.1, 0.2, 0.3, 0.1, 0.5, 0.1, 0.4, 0.2],
      "confidence": 0.85,
      "timestamp": 1733439000000
    },
    "desiredState": {
      "targetValence": 0.5,
      "targetArousal": -0.2,
      "targetStress": 0.2,
      "intensity": "moderate",
      "reasoning": "User wants to relax"
    },
    "limit": 5
  }'

# Submit feedback
curl -X POST http://localhost:3000/api/v1/feedback \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "user-123",
    "contentId": "content-001",
    "actualPostState": {
      "valence": 0.6,
      "arousal": -0.2,
      "stressLevel": 0.2,
      "primaryEmotion": "relaxed",
      "emotionVector": [0.7, 0.3, 0.1, 0.05, 0.1, 0.05, 0.1, 0.2],
      "confidence": 0.88,
      "timestamp": 1733439000000
    },
    "watchDuration": 45,
    "completed": true,
    "explicitRating": 5
  }'
```

## Architecture

```
src/api/
├── index.ts                 # Express app setup
├── middleware/
│   ├── error-handler.ts     # Global error handling
│   ├── logger.ts            # Request logging
│   └── rate-limiter.ts      # Rate limiting
└── routes/
    ├── emotion.ts           # Emotion detection endpoints
    ├── recommend.ts         # Recommendation endpoints
    └── feedback.ts          # Feedback endpoints
```

## Future Enhancements

- [ ] JWT authentication
- [ ] WebSocket support for real-time updates
- [ ] OpenAPI/Swagger documentation
- [ ] Request caching
- [ ] Database persistence
- [ ] Integration with EmotionDetector module
- [ ] Integration with RecommendationEngine module
- [ ] Integration with FeedbackProcessor module
