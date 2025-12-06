# EmotiStream Nexus MVP - API & Data Model Specification

**Version**: 1.0
**Last Updated**: 2025-12-05
**Target Implementation**: ~70-hour Hackathon MVP
**Base URL**: `http://localhost:3000/api/v1`

---

## Table of Contents

1. [API Overview](#1-api-overview)
2. [Authentication](#2-authentication)
3. [Core Endpoints](#3-core-endpoints)
4. [Data Models](#4-data-models)
5. [AgentDB Key Patterns](#5-agentdb-key-patterns)
6. [RuVector Collections](#6-ruvector-collections)
7. [Error Handling](#7-error-handling)
8. [Example API Calls](#8-example-api-calls)
9. [Rate Limits & Performance](#9-rate-limits--performance)

---

## 1. API Overview

### 1.1 Architecture

- **Protocol**: REST with JSON payloads
- **Authentication**: JWT bearer tokens
- **Rate Limiting**: 100 requests/minute per user
- **Versioning**: URL-based (`/api/v1`)
- **Content Type**: `application/json`

### 1.2 Response Format

All responses follow this structure:

```json
{
  "success": true,
  "data": { /* response payload */ },
  "error": null,
  "timestamp": "2025-12-05T10:30:00.000Z"
}
```

Error responses:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "E001",
    "message": "Gemini API timeout",
    "details": { /* optional context */ }
  },
  "timestamp": "2025-12-05T10:30:00.000Z"
}
```

---

## 2. Authentication

### 2.1 Register User

```http
POST /api/v1/auth/register
Content-Type: application/json
```

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securePassword123",
  "dateOfBirth": "1990-01-01",
  "displayName": "John Doe"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "userId": "usr_abc123xyz",
    "email": "user@example.com",
    "displayName": "John Doe",
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refreshToken": "refresh_token_here",
    "expiresAt": "2025-12-06T10:30:00.000Z"
  },
  "error": null,
  "timestamp": "2025-12-05T10:30:00.000Z"
}
```

### 2.2 Login

```http
POST /api/v1/auth/login
Content-Type: application/json
```

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securePassword123"
}
```

**Response:** Same as register response

### 2.3 Refresh Token

```http
POST /api/v1/auth/refresh
Content-Type: application/json
```

**Request:**
```json
{
  "refreshToken": "refresh_token_here"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "token": "new_jwt_token",
    "expiresAt": "2025-12-06T10:30:00.000Z"
  },
  "error": null,
  "timestamp": "2025-12-05T10:30:00.000Z"
}
```

---

## 3. Core Endpoints

### 3.1 Emotion Detection

#### POST /api/v1/emotion/detect

Detect emotional state from text input (voice/biometric in future iterations).

**Request:**
```json
{
  "userId": "usr_abc123xyz",
  "text": "I'm feeling exhausted and stressed after a long day",
  "context": {
    "dayOfWeek": 5,
    "hourOfDay": 18,
    "socialContext": "solo"
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "emotionalStateId": "state_xyz789",
    "primaryEmotion": "sadness",
    "valence": -0.6,
    "arousal": 0.2,
    "stressLevel": 0.8,
    "confidence": 0.85,
    "predictedDesiredState": {
      "valence": 0.5,
      "arousal": -0.2,
      "confidence": 0.7
    },
    "timestamp": "2025-12-05T18:30:00.000Z"
  },
  "error": null,
  "timestamp": "2025-12-05T18:30:00.000Z"
}
```

**Emotional State Fields:**

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `valence` | `number` | -1 to +1 | Emotional positivity (-1=very negative, +1=very positive) |
| `arousal` | `number` | -1 to +1 | Energy level (-1=very calm, +1=very excited) |
| `stressLevel` | `number` | 0 to 1 | Stress intensity (0=relaxed, 1=extremely stressed) |
| `confidence` | `number` | 0 to 1 | Detection confidence (0=uncertain, 1=very confident) |

**Error Codes:**
- `E001`: Gemini API timeout
- `E002`: Gemini rate limit exceeded
- `E003`: Invalid input text (empty or too long)

---

### 3.2 Get Recommendations

#### POST /api/v1/recommend

Get content recommendations based on emotional state using RL policy.

**Request:**
```json
{
  "userId": "usr_abc123xyz",
  "emotionalStateId": "state_xyz789",
  "limit": 10,
  "explicitDesiredState": {
    "valence": 0.7,
    "arousal": -0.3
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "recommendations": [
      {
        "contentId": "content_123",
        "title": "Nature Sounds: Ocean Waves",
        "platform": "youtube",
        "emotionalProfile": {
          "primaryTone": "calm",
          "valenceDelta": 0.4,
          "arousalDelta": -0.5,
          "intensity": 0.3
        },
        "predictedOutcome": {
          "postViewingValence": 0.2,
          "postViewingArousal": -0.3,
          "confidence": 0.78
        },
        "qValue": 0.82,
        "isExploration": false,
        "rank": 1
      },
      {
        "contentId": "content_456",
        "title": "The Grand Budapest Hotel",
        "platform": "netflix",
        "emotionalProfile": {
          "primaryTone": "uplifting",
          "valenceDelta": 0.6,
          "arousalDelta": 0.1,
          "intensity": 0.5
        },
        "predictedOutcome": {
          "postViewingValence": 0.4,
          "postViewingArousal": 0.3,
          "confidence": 0.72
        },
        "qValue": 0.75,
        "isExploration": false,
        "rank": 2
      }
    ],
    "explorationRate": 0.15,
    "totalCandidates": 234
  },
  "error": null,
  "timestamp": "2025-12-05T18:31:00.000Z"
}
```

**Recommendation Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `qValue` | `number` | Q-learning value (0-1, higher = better predicted outcome) |
| `isExploration` | `boolean` | Whether this is an exploratory recommendation |
| `rank` | `number` | Position in recommendation list (1 = best) |
| `valenceDelta` | `number` | Expected change in valence after viewing |
| `arousalDelta` | `number` | Expected change in arousal after viewing |

**Error Codes:**
- `E004`: User not found
- `E005`: Content not found
- `E006`: RL policy error

---

### 3.3 Submit Feedback

#### POST /api/v1/feedback

Submit post-viewing emotional feedback to update RL policy.

**Request:**
```json
{
  "userId": "usr_abc123xyz",
  "contentId": "content_123",
  "emotionalStateId": "state_xyz789",
  "postViewingState": {
    "text": "I feel much calmer now",
    "explicitRating": 4,
    "explicitEmoji": "😊"
  },
  "viewingDetails": {
    "completionRate": 0.95,
    "durationSeconds": 1800
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "experienceId": "exp_abc789",
    "reward": 0.78,
    "emotionalImprovement": 0.65,
    "qValueBefore": 0.82,
    "qValueAfter": 0.85,
    "policyUpdated": true,
    "message": "Thank you for your feedback! Your recommendations are getting better."
  },
  "error": null,
  "timestamp": "2025-12-05T19:00:00.000Z"
}
```

**Feedback Processing:**

1. Analyzes post-viewing text using Gemini
2. Calculates emotional state change (reward)
3. Updates Q-values using Q-learning algorithm
4. Stores experience in replay buffer for batch updates

**Reward Calculation:**

```
reward = directionAlignment * 0.6 + improvement * 0.4 + proximityBonus * 0.2

where:
  directionAlignment = cosine similarity between actual and desired emotional change
  improvement = magnitude of emotional improvement
  proximityBonus = bonus for reaching desired state
```

---

### 3.4 Get User Insights

#### GET /api/v1/insights/:userId

Get emotional journey and learning insights for a user.

**Request:**
```http
GET /api/v1/insights/usr_abc123xyz
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response:**
```json
{
  "success": true,
  "data": {
    "userId": "usr_abc123xyz",
    "totalExperiences": 45,
    "avgReward": 0.68,
    "explorationRate": 0.12,
    "policyConvergence": 0.88,
    "emotionalJourney": [
      {
        "timestamp": "2025-12-01T18:00:00.000Z",
        "valence": -0.6,
        "arousal": 0.3,
        "primaryEmotion": "stressed"
      },
      {
        "timestamp": "2025-12-02T19:00:00.000Z",
        "valence": 0.2,
        "arousal": -0.1,
        "primaryEmotion": "calm"
      }
    ],
    "mostEffectiveContent": [
      {
        "contentId": "content_123",
        "title": "Nature Sounds: Ocean Waves",
        "avgReward": 0.82,
        "timesRecommended": 8
      }
    ],
    "learningProgress": {
      "experiencesUntilConvergence": 10,
      "currentQValueVariance": 0.03,
      "isConverged": true
    }
  },
  "error": null,
  "timestamp": "2025-12-05T19:30:00.000Z"
}
```

**Learning Metrics:**

| Metric | Meaning | Target |
|--------|---------|--------|
| `avgReward` | Average emotional improvement per session | ≥0.60 |
| `explorationRate` | % of recommendations that are exploratory | 0.10-0.30 |
| `policyConvergence` | How stable the RL policy is (0-1) | ≥0.85 |
| `qValueVariance` | Variance in Q-values (lower = more stable) | <0.05 |

---

### 3.5 Content Profiling (Internal)

#### POST /api/v1/content/profile

Profile content emotional characteristics (admin/batch processing only).

**Request:**
```json
{
  "contentId": "content_new123",
  "title": "Peaceful Forest Walk - 4K Nature Video",
  "description": "Immerse yourself in the tranquility of a peaceful forest walk. Perfect for relaxation and stress relief.",
  "genres": ["nature", "relaxation", "ambient"],
  "platform": "youtube"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "contentId": "content_new123",
    "emotionalProfile": {
      "primaryTone": "calm",
      "valenceDelta": 0.35,
      "arousalDelta": -0.45,
      "intensity": 0.2,
      "complexity": 0.1,
      "targetStates": [
        {
          "currentValence": -0.5,
          "currentArousal": 0.5,
          "description": "stressed and anxious"
        },
        {
          "currentValence": -0.3,
          "currentArousal": 0.2,
          "description": "moderately stressed"
        }
      ]
    },
    "embeddingId": "vec_forest_walk_123",
    "profiledAt": "2025-12-05T20:00:00.000Z"
  },
  "error": null,
  "timestamp": "2025-12-05T20:00:00.000Z"
}
```

**Profiling Process:**

1. Gemini analyzes title + description
2. Generates emotional profile (tone, deltas, intensity)
3. Creates 1536D embedding using ruvLLM
4. Stores embedding in RuVector with HNSW index
5. Stores metadata in AgentDB

---

### 3.6 Wellbeing Check

#### GET /api/v1/wellbeing/:userId

Check user's wellbeing status and get alerts/recommendations.

**Request:**
```http
GET /api/v1/wellbeing/usr_abc123xyz
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Response:**
```json
{
  "success": true,
  "data": {
    "userId": "usr_abc123xyz",
    "overallTrend": 0.15,
    "recentMoodAvg": -0.25,
    "emotionalVariability": 0.45,
    "sustainedNegativeMoodDays": 3,
    "alerts": [
      {
        "type": "sustained-negative-mood",
        "severity": "medium",
        "message": "We noticed you've been feeling down lately. Would you like some resources?",
        "resources": [
          {
            "type": "crisis-line",
            "name": "988 Suicide & Crisis Lifeline",
            "url": "tel:988",
            "description": "24/7 free and confidential support"
          },
          {
            "type": "therapy",
            "name": "Find a therapist",
            "url": "https://www.psychologytoday.com/us/therapists",
            "description": "Connect with licensed mental health professionals"
          }
        ],
        "triggeredAt": "2025-12-05T08:00:00.000Z"
      }
    ],
    "recommendations": [
      {
        "type": "self-care",
        "message": "Try incorporating daily mindfulness exercises",
        "actionUrl": "/app/mindfulness"
      }
    ]
  },
  "error": null,
  "timestamp": "2025-12-05T20:30:00.000Z"
}
```

**Alert Thresholds:**

| Alert Type | Trigger | Severity |
|------------|---------|----------|
| `sustained-negative-mood` | Avg valence < -0.5 for 7+ days | high |
| `emotional-dysregulation` | Valence variance > 0.7 over 7 days | medium |
| `crisis-detected` | Valence < -0.8 for 3+ consecutive sessions | critical |

---

## 4. Data Models

### 4.1 EmotionalState

Core emotional state representation (AgentDB).

```typescript
interface EmotionalState {
  id: string;                    // state_xyz789
  userId: string;                // usr_abc123xyz
  valence: number;               // -1 to +1
  arousal: number;               // -1 to +1
  primaryEmotion: string;        // "joy", "sadness", "anger", etc.
  emotionVector: number[];       // 8D Plutchik [joy, sadness, anger, fear, trust, disgust, surprise, anticipation]
  stressLevel: number;           // 0 to 1
  confidence: number;            // 0 to 1
  context: {
    dayOfWeek: number;           // 0-6 (Sunday=0)
    hourOfDay: number;           // 0-23
    socialContext: string;       // "solo" | "partner" | "family" | "friends"
  };
  desiredValence: number;        // -1 to +1 (predicted or explicit)
  desiredArousal: number;        // -1 to +1
  timestamp: number;             // Unix timestamp (ms)
}
```

**AgentDB Key:** `state:{stateId}`

**Example:**
```json
{
  "id": "state_xyz789",
  "userId": "usr_abc123xyz",
  "valence": -0.6,
  "arousal": 0.2,
  "primaryEmotion": "sadness",
  "emotionVector": [0, 0.8, 0.2, 0.3, 0.1, 0, 0, 0],
  "stressLevel": 0.8,
  "confidence": 0.85,
  "context": {
    "dayOfWeek": 5,
    "hourOfDay": 18,
    "socialContext": "solo"
  },
  "desiredValence": 0.5,
  "desiredArousal": -0.2,
  "timestamp": 1733421000000
}
```

---

### 4.2 Content

Content with emotional profile (AgentDB + RuVector).

> **Note**: This MVP uses a **mock content catalog** (200 items) rather than live
> streaming APIs. Real-world integrations with Netflix, YouTube, etc. require
> contractual relationships and are deferred to Phase 2. The mock catalog allows
> us to prove the RL algorithm without external API dependencies.

```typescript
interface Content {
  id: string;                    // content_123
  title: string;                 // "Nature Sounds: Ocean Waves"
  description: string;           // Full description
  platform: string;              // "mock" | "youtube" | "netflix" | "prime"
  genres: string[];              // ["nature", "relaxation"]

  // Content categorization (for improved search)
  category: 'movie' | 'series' | 'documentary' | 'music' | 'meditation' | 'short';
  tags: string[];                // ['feel-good', 'nature', 'slow-paced', etc.]

  duration: number;              // Duration in seconds
  emotionalProfile: {
    primaryTone: string;         // "calm", "uplifting", "thrilling", etc.
    valenceDelta: number;        // Expected change in valence (-1 to +1)
    arousalDelta: number;        // Expected change in arousal (-1 to +1)
    intensity: number;           // 0-1 (subtle to intense)
    complexity: number;          // 0-1 (simple to nuanced)
    targetStates: TargetState[]; // Best for which emotional states
  };
  embeddingId: string;           // RuVector embedding ID
  createdAt: number;             // Unix timestamp (ms)
}

interface TargetState {
  currentValence: number;        // -1 to +1
  currentArousal: number;        // -1 to +1
  description: string;           // "stressed and anxious"
}
```

**AgentDB Key:** `content:{contentId}`

**RuVector Collection:** `content_emotions`

**Example:**
```json
{
  "id": "content_123",
  "title": "Nature Sounds: Ocean Waves",
  "description": "Relaxing ocean waves for stress relief and sleep",
  "platform": "mock",
  "genres": ["nature", "relaxation", "ambient"],
  "category": "meditation",
  "tags": ["calming", "nature-sounds", "sleep-aid", "stress-relief"],
  "duration": 3600,
  "emotionalProfile": {
    "primaryTone": "calm",
    "valenceDelta": 0.4,
    "arousalDelta": -0.5,
    "intensity": 0.3,
    "complexity": 0.1,
    "targetStates": [
      {
        "currentValence": -0.5,
        "currentArousal": 0.5,
        "description": "stressed and anxious"
      }
    ]
  },
  "embeddingId": "vec_ocean_waves_123",
  "createdAt": 1733420000000
}
```

---

### 4.3 UserProfile

User profile with RL learning metrics (AgentDB).

```typescript
interface UserProfile {
  id: string;                    // usr_abc123xyz
  email: string;                 // user@example.com
  displayName: string;           // "John Doe"
  emotionalBaseline: {
    avgValence: number;          // Average valence over all sessions
    avgArousal: number;          // Average arousal
    variability: number;         // Emotional variability (std dev)
  };
  totalExperiences: number;      // Total content viewing experiences
  avgReward: number;             // Average RL reward (0-1)
  explorationRate: number;       // Current ε-greedy exploration rate
  createdAt: number;             // Unix timestamp (ms)
  lastActive: number;            // Unix timestamp (ms)
}
```

**AgentDB Key:** `user:{userId}`

**Example:**
```json
{
  "id": "usr_abc123xyz",
  "email": "user@example.com",
  "displayName": "John Doe",
  "emotionalBaseline": {
    "avgValence": 0.15,
    "avgArousal": 0.05,
    "variability": 0.35
  },
  "totalExperiences": 45,
  "avgReward": 0.68,
  "explorationRate": 0.12,
  "createdAt": 1733000000000,
  "lastActive": 1733421000000
}
```

---

### 4.4 Experience

Emotional experience for RL training (AgentDB).

```typescript
interface Experience {
  id: string;                    // exp_abc789
  userId: string;                // usr_abc123xyz
  stateBefore: EmotionalState;   // Emotional state before viewing
  stateAfter: EmotionalState;    // Emotional state after viewing
  contentId: string;             // content_123
  desiredState: {
    valence: number;             // -1 to +1
    arousal: number;             // -1 to +1
  };
  reward: number;                // -1 to +1 (RL reward)
  timestamp: number;             // Unix timestamp (ms)
}
```

**AgentDB Key:** `exp:{experienceId}`

**User Experience List Key:** `user:{userId}:experiences` (sorted set by timestamp)

**Example:**
```json
{
  "id": "exp_abc789",
  "userId": "usr_abc123xyz",
  "stateBefore": {
    "valence": -0.6,
    "arousal": 0.2,
    "primaryEmotion": "sadness",
    "stressLevel": 0.8
  },
  "stateAfter": {
    "valence": 0.2,
    "arousal": -0.3,
    "primaryEmotion": "calm",
    "stressLevel": 0.3
  },
  "contentId": "content_123",
  "desiredState": {
    "valence": 0.5,
    "arousal": -0.2
  },
  "reward": 0.78,
  "timestamp": 1733421000000
}
```

---

### 4.5 QTableEntry

Q-learning table entry (AgentDB).

```typescript
interface QTableEntry {
  userId: string;                // usr_abc123xyz
  stateHash: string;             // Discretized state hash (e.g., "2:3:2:solo")
  contentId: string;             // content_123
  qValue: number;                // Q-value (0-1)
  visitCount: number;            // Number of times this state-action pair was visited
  lastUpdated: number;           // Unix timestamp (ms)
}
```

**AgentDB Key:** `q:{userId}:{stateHash}:{contentId}`

**State Hash Format:** `{valenceBucket}:{arousalBucket}:{stressBucket}:{socialContext}`

- `valenceBucket`: 0-4 (5 buckets, each 0.4 wide)
- `arousalBucket`: 0-4 (5 buckets, each 0.4 wide)
- `stressBucket`: 0-2 (3 buckets, each 0.33 wide)
- `socialContext`: "solo" | "partner" | "family" | "friends"

**Example:**
```json
{
  "userId": "usr_abc123xyz",
  "stateHash": "1:3:2:solo",
  "contentId": "content_123",
  "qValue": 0.82,
  "visitCount": 5,
  "lastUpdated": 1733421000000
}
```

---

## 5. AgentDB Key Patterns

All data is stored in AgentDB using these key naming conventions:

```typescript
const keys = {
  // User data
  user: (userId: string) => `user:${userId}`,
  userExperiences: (userId: string) => `user:${userId}:experiences`,
  userVisitCount: (userId: string, contentId: string) => `user:${userId}:visit:${contentId}`,
  userTotalActions: (userId: string) => `user:${userId}:total-actions`,

  // Emotional states
  emotionalState: (stateId: string) => `state:${stateId}`,

  // Experiences
  experience: (expId: string) => `exp:${expId}`,

  // Q-values
  qValue: (userId: string, stateHash: string, contentId: string) =>
    `q:${userId}:${stateHash}:${contentId}`,

  // Content
  content: (contentId: string) => `content:${contentId}`,

  // Wellbeing
  wellbeingAlert: (userId: string, alertId: string) =>
    `wellbeing:${userId}:alert:${alertId}`,
};
```

**Example Usage:**

```typescript
// Store user profile
await agentDB.set('user:usr_abc123xyz', userProfile);

// Get user experiences (sorted set)
await agentDB.zrange('user:usr_abc123xyz:experiences', 0, -1);

// Update Q-value
await agentDB.set('q:usr_abc123xyz:1:3:2:solo:content_123', 0.82);

// Increment visit count
await agentDB.incr('user:usr_abc123xyz:visit:content_123');
```

---

## 6. RuVector Collections

### 6.1 Content Emotions Collection

**Collection Name:** `content_emotions`

**Vector Dimensions:** 1536 (Gemini embedding size)

**Index:** HNSW (M=16, efConstruction=200)

**Metadata Schema:**
```typescript
interface ContentEmotionMetadata {
  contentId: string;
  title: string;
  platform: string;
  primaryTone: string;
  valenceDelta: number;
  arousalDelta: number;
  intensity: number;
  complexity: number;
}
```

**Search Example:**
```typescript
// Search for content matching emotional transition
const results = await ruVector.search({
  collection: 'content_emotions',
  vector: transitionEmbedding,  // 1536D float array
  topK: 30,
  filter: {
    valenceDelta: { $gte: 0.3 },  // Only positive content
    arousalDelta: { $lte: -0.2 }  // Only calming content
  }
});
```

---

### 6.2 Emotional Transitions Collection

**Collection Name:** `emotional_transitions`

**Vector Dimensions:** 1536

**Purpose:** Store successful emotional transitions for pattern matching.

**Metadata Schema:**
```typescript
interface TransitionMetadata {
  userId: string;
  startValence: number;
  startArousal: number;
  endValence: number;
  endArousal: number;
  contentId: string;
  reward: number;
  timestamp: number;
}
```

**Usage:**
- Find similar past transitions
- Recommend content based on similar users' successful transitions
- Collaborative filtering in emotion space

---

## 7. Error Handling

### 7.1 Error Codes

| Code | Error | HTTP Status | Retry | Description |
|------|-------|-------------|-------|-------------|
| `E001` | `GEMINI_TIMEOUT` | 504 | No | Gemini API timeout after 30s |
| `E002` | `GEMINI_RATE_LIMIT` | 429 | Yes (60s) | Gemini rate limit exceeded |
| `E003` | `INVALID_INPUT` | 400 | No | Invalid input (empty text, malformed JSON) |
| `E004` | `USER_NOT_FOUND` | 404 | No | User ID not found |
| `E005` | `CONTENT_NOT_FOUND` | 404 | No | Content ID not found |
| `E006` | `RL_POLICY_ERROR` | 500 | No | RL policy computation error |
| `E007` | `AUTH_INVALID_TOKEN` | 401 | No | Invalid or expired JWT token |
| `E008` | `AUTH_UNAUTHORIZED` | 403 | No | User not authorized for this resource |
| `E009` | `RATE_LIMIT_EXCEEDED` | 429 | Yes (60s) | API rate limit exceeded |
| `E010` | `INTERNAL_ERROR` | 500 | No | Unexpected server error |

### 7.2 Error Response Format

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "E001",
    "message": "Gemini API timeout",
    "details": {
      "timeout": 30000,
      "attemptedAt": "2025-12-05T10:30:00.000Z"
    },
    "fallback": {
      "emotionalState": {
        "valence": 0,
        "arousal": 0,
        "confidence": 0.3
      },
      "message": "Emotion detection temporarily unavailable, please try again"
    }
  },
  "timestamp": "2025-12-05T10:30:00.000Z"
}
```

### 7.3 Fallback Behavior

#### Gemini Timeout (E001)
- Return neutral emotional state (valence=0, arousal=0, confidence=0.3)
- Log error for monitoring
- User message: "Emotion detection temporarily unavailable"

#### Gemini Rate Limit (E002)
- Queue request for retry after 60 seconds
- Return 429 with `Retry-After: 60` header
- User message: "Processing... please wait"

#### No Q-values for State (E006)
- Fallback to content-based filtering using RuVector semantic search
- Set exploration rate to 0.5 (high exploration)
- User receives recommendations but from vector search, not RL

#### User Not Found (E004)
- For new users, create default profile with exploration rate 0.3
- Use population-based recommendations (top 20 most effective content)

---

## 8. Example API Calls

### 8.1 Complete User Flow (curl)

#### Step 1: Register User

```bash
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "jane@example.com",
    "password": "securePass123",
    "dateOfBirth": "1995-05-15",
    "displayName": "Jane Doe"
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "userId": "usr_jane123",
    "email": "jane@example.com",
    "displayName": "Jane Doe",
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOiJ1c3JfamFuZTEyMyIsImlhdCI6MTczMzQyMTAwMCwiZXhwIjoxNzMzNTA3NDAwfQ.signature",
    "refreshToken": "refresh_abc123xyz",
    "expiresAt": "2025-12-06T10:30:00.000Z"
  },
  "error": null,
  "timestamp": "2025-12-05T10:30:00.000Z"
}
```

---

#### Step 2: Detect Emotional State

```bash
curl -X POST http://localhost:3000/api/v1/emotion/detect \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -d '{
    "userId": "usr_jane123",
    "text": "I had a really stressful day at work and I just want to relax",
    "context": {
      "dayOfWeek": 3,
      "hourOfDay": 19,
      "socialContext": "solo"
    }
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "emotionalStateId": "state_stress_evening",
    "primaryEmotion": "sadness",
    "valence": -0.5,
    "arousal": 0.4,
    "stressLevel": 0.75,
    "confidence": 0.82,
    "predictedDesiredState": {
      "valence": 0.6,
      "arousal": -0.3,
      "confidence": 0.7
    },
    "timestamp": "2025-12-05T19:00:00.000Z"
  },
  "error": null,
  "timestamp": "2025-12-05T19:00:00.000Z"
}
```

---

#### Step 3: Get Recommendations

```bash
curl -X POST http://localhost:3000/api/v1/recommend \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -d '{
    "userId": "usr_jane123",
    "emotionalStateId": "state_stress_evening",
    "limit": 5
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "recommendations": [
      {
        "contentId": "content_ocean_waves",
        "title": "Ocean Waves - 1 Hour Relaxation",
        "platform": "youtube",
        "emotionalProfile": {
          "primaryTone": "calm",
          "valenceDelta": 0.45,
          "arousalDelta": -0.6,
          "intensity": 0.2
        },
        "predictedOutcome": {
          "postViewingValence": 0.15,
          "postViewingArousal": -0.2,
          "confidence": 0.78
        },
        "qValue": 0.85,
        "isExploration": false,
        "rank": 1
      },
      {
        "contentId": "content_studio_ghibli",
        "title": "My Neighbor Totoro",
        "platform": "netflix",
        "emotionalProfile": {
          "primaryTone": "uplifting",
          "valenceDelta": 0.55,
          "arousalDelta": -0.1,
          "intensity": 0.4
        },
        "predictedOutcome": {
          "postViewingValence": 0.25,
          "postViewingArousal": 0.3,
          "confidence": 0.72
        },
        "qValue": 0.76,
        "isExploration": false,
        "rank": 2
      }
    ],
    "explorationRate": 0.15,
    "totalCandidates": 187
  },
  "error": null,
  "timestamp": "2025-12-05T19:01:00.000Z"
}
```

---

#### Step 4: Submit Feedback After Viewing

```bash
curl -X POST http://localhost:3000/api/v1/feedback \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -d '{
    "userId": "usr_jane123",
    "contentId": "content_ocean_waves",
    "emotionalStateId": "state_stress_evening",
    "postViewingState": {
      "text": "I feel so much calmer now, that was exactly what I needed",
      "explicitRating": 5,
      "explicitEmoji": "😊"
    },
    "viewingDetails": {
      "completionRate": 1.0,
      "durationSeconds": 3600
    }
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "experienceId": "exp_jane_ocean_1",
    "reward": 0.87,
    "emotionalImprovement": 0.72,
    "qValueBefore": 0.85,
    "qValueAfter": 0.88,
    "policyUpdated": true,
    "message": "Thank you for your feedback! Your recommendations are getting better."
  },
  "error": null,
  "timestamp": "2025-12-05T20:00:00.000Z"
}
```

---

#### Step 5: Get Insights (After 50+ Experiences)

```bash
curl -X GET http://localhost:3000/api/v1/insights/usr_jane123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

**Response:**
```json
{
  "success": true,
  "data": {
    "userId": "usr_jane123",
    "totalExperiences": 52,
    "avgReward": 0.71,
    "explorationRate": 0.11,
    "policyConvergence": 0.92,
    "emotionalJourney": [
      {
        "timestamp": "2025-12-01T19:00:00.000Z",
        "valence": -0.5,
        "arousal": 0.4,
        "primaryEmotion": "stressed"
      },
      {
        "timestamp": "2025-12-02T19:30:00.000Z",
        "valence": 0.3,
        "arousal": -0.1,
        "primaryEmotion": "calm"
      },
      {
        "timestamp": "2025-12-03T20:00:00.000Z",
        "valence": 0.5,
        "arousal": 0.2,
        "primaryEmotion": "content"
      }
    ],
    "mostEffectiveContent": [
      {
        "contentId": "content_ocean_waves",
        "title": "Ocean Waves - 1 Hour Relaxation",
        "avgReward": 0.86,
        "timesRecommended": 12
      },
      {
        "contentId": "content_studio_ghibli",
        "title": "My Neighbor Totoro",
        "avgReward": 0.79,
        "timesRecommended": 6
      }
    ],
    "learningProgress": {
      "experiencesUntilConvergence": 8,
      "currentQValueVariance": 0.028,
      "isConverged": true
    }
  },
  "error": null,
  "timestamp": "2025-12-05T20:30:00.000Z"
}
```

---

#### Step 6: Check Wellbeing Status

```bash
curl -X GET http://localhost:3000/api/v1/wellbeing/usr_jane123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

**Response:**
```json
{
  "success": true,
  "data": {
    "userId": "usr_jane123",
    "overallTrend": 0.35,
    "recentMoodAvg": 0.25,
    "emotionalVariability": 0.32,
    "sustainedNegativeMoodDays": 0,
    "alerts": [],
    "recommendations": [
      {
        "type": "positive-reinforcement",
        "message": "Great progress! Your emotional wellbeing is improving.",
        "actionUrl": "/app/insights"
      }
    ]
  },
  "error": null,
  "timestamp": "2025-12-05T20:35:00.000Z"
}
```

---

### 8.2 Batch Content Profiling (Admin)

```bash
curl -X POST http://localhost:3000/api/v1/content/profile \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer admin_token..." \
  -d '{
    "contentId": "content_planet_earth",
    "title": "Planet Earth II - Forests",
    "description": "Experience the breathtaking beauty of Earth'\''s forests with stunning 4K footage and David Attenborough'\''s narration.",
    "genres": ["nature", "documentary", "educational"],
    "platform": "netflix"
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "contentId": "content_planet_earth",
    "emotionalProfile": {
      "primaryTone": "awe-inspiring",
      "valenceDelta": 0.5,
      "arousalDelta": 0.2,
      "intensity": 0.6,
      "complexity": 0.4,
      "targetStates": [
        {
          "currentValence": -0.3,
          "currentArousal": 0.1,
          "description": "mildly stressed, seeking inspiration"
        },
        {
          "currentValence": 0.2,
          "currentArousal": -0.2,
          "description": "calm but seeking engagement"
        }
      ]
    },
    "embeddingId": "vec_planet_earth_forests",
    "profiledAt": "2025-12-05T21:00:00.000Z"
  },
  "error": null,
  "timestamp": "2025-12-05T21:00:00.000Z"
}
```

---

## 9. Rate Limits & Performance

### 9.1 Rate Limits

| Endpoint | Limit | Window | Bucket |
|----------|-------|--------|--------|
| `/api/v1/emotion/detect` | 30 requests | 1 minute | Per user |
| `/api/v1/recommend` | 60 requests | 1 minute | Per user |
| `/api/v1/feedback` | 60 requests | 1 minute | Per user |
| `/api/v1/insights/:userId` | 10 requests | 1 minute | Per user |
| `/api/v1/content/profile` | 1000 requests | 1 hour | Per API key (admin) |

**Rate Limit Headers:**
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1733421060
```

**Rate Limit Exceeded Response:**
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "E009",
    "message": "Rate limit exceeded",
    "details": {
      "limit": 60,
      "window": "1 minute",
      "resetAt": "2025-12-05T19:01:00.000Z"
    }
  },
  "timestamp": "2025-12-05T19:00:30.000Z"
}
```

---

### 9.2 Performance Targets (MVP)

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Emotion Detection (text)** | <2s (p95) | Time from request to response |
| **Content Recommendations** | <3s (p95) | Including RL policy + vector search |
| **Feedback Submission** | <100ms (p95) | Q-value update latency |
| **Insights Query** | <1s (p95) | Aggregation over user history |
| **RuVector Search** | <500ms (p95) | HNSW semantic search (30 candidates) |
| **AgentDB Read** | <10ms (p95) | Single key lookup |
| **AgentDB Write** | <20ms (p95) | Single key write |

---

### 9.3 Scalability Considerations

**AgentDB:**
- Keys are partitioned by userId for horizontal scaling
- Q-tables use state hashing to limit key space
- Experience replay buffer uses sorted sets with TTL (30 days)

**RuVector:**
- HNSW index allows O(log n) search complexity
- Batch upsert for content profiling (1000 items/batch)
- Index rebuild schedule: weekly (off-peak hours)

**Gemini API:**
- Request batching for content profiling (10 items/batch)
- Circuit breaker after 3 consecutive timeouts
- Fallback to cached emotional profiles if available

---

## 10. Testing Endpoints

For development/testing, these endpoints are available:

### 10.1 Health Check

```bash
curl -X GET http://localhost:3000/api/v1/health
```

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "services": {
      "agentdb": "connected",
      "ruvector": "connected",
      "gemini": "available"
    },
    "uptime": 86400,
    "version": "1.0.0"
  },
  "error": null,
  "timestamp": "2025-12-05T22:00:00.000Z"
}
```

---

### 10.2 Reset User Data (Dev Only)

```bash
curl -X DELETE http://localhost:3000/api/v1/dev/user/:userId/reset \
  -H "Authorization: Bearer dev_token..."
```

**Response:**
```json
{
  "success": true,
  "data": {
    "deletedKeys": [
      "user:usr_jane123",
      "user:usr_jane123:experiences",
      "q:usr_jane123:*"
    ],
    "message": "User data reset successfully"
  },
  "error": null,
  "timestamp": "2025-12-05T22:05:00.000Z"
}
```

---

## Appendix A: RL Algorithm Details

### Q-Learning Update Rule

```typescript
function updateQValue(
  userId: string,
  stateHash: string,
  contentId: string,
  reward: number,
  nextStateHash: string
): void {
  const learningRate = 0.1;
  const discountFactor = 0.95;

  // Current Q-value
  const currentQ = await getQValue(userId, stateHash, contentId);

  // Max Q-value for next state
  const maxNextQ = await getMaxQValue(userId, nextStateHash);

  // Q-learning update: Q(s,a) ← Q(s,a) + α[r + γ max Q(s',a') - Q(s,a)]
  const newQ = currentQ + learningRate * (
    reward + discountFactor * maxNextQ - currentQ
  );

  await setQValue(userId, stateHash, contentId, newQ);
}
```

### State Discretization

```typescript
function hashEmotionalState(state: EmotionalState): string {
  // Discretize continuous state space into buckets
  const valenceBucket = Math.floor((state.valence + 1) / 0.4); // 0-4
  const arousalBucket = Math.floor((state.arousal + 1) / 0.4); // 0-4
  const stressBucket = Math.floor(state.stressLevel / 0.33);   // 0-2

  return `${valenceBucket}:${arousalBucket}:${stressBucket}:${state.socialContext}`;
}

// Example:
// State: { valence: -0.5, arousal: 0.3, stressLevel: 0.8, socialContext: "solo" }
// Hash: "1:3:2:solo"
//   valenceBucket: (-0.5 + 1) / 0.4 = 1.25 → floor = 1
//   arousalBucket: (0.3 + 1) / 0.4 = 3.25 → floor = 3
//   stressBucket: 0.8 / 0.33 = 2.42 → floor = 2
```

### Exploration Strategy (ε-greedy)

```typescript
async function selectAction(
  userId: string,
  emotionalState: EmotionalState,
  explorationRate: number = 0.15
): Promise<Content> {
  if (Math.random() < explorationRate) {
    // Explore: UCB-based selection
    return await exploreContent(userId, emotionalState);
  } else {
    // Exploit: Select highest Q-value
    return await exploitContent(userId, emotionalState);
  }
}
```

---

## Appendix B: Gemini Prompts

### Emotion Detection Prompt (Text)

```
Analyze the emotional state from this text: "{user_text}"

Provide:
1. Primary emotion (joy, sadness, anger, fear, trust, disgust, surprise, anticipation)
2. Valence: -1 (very negative) to +1 (very positive)
3. Arousal: -1 (very calm) to +1 (very excited)
4. Stress level: 0 (relaxed) to 1 (extremely stressed)
5. Confidence: 0 to 1

Format as JSON:
{
  "primaryEmotion": "...",
  "valence": 0.0,
  "arousal": 0.0,
  "stressLevel": 0.0,
  "confidence": 0.0,
  "reasoning": "..."
}
```

### Content Profiling Prompt

```
Analyze the emotional impact of this content:

Title: {title}
Description: {description}
Genres: {genres}

Provide:
1. Primary emotional tone (joy, sadness, anger, fear, etc.)
2. Valence delta: expected change in viewer's valence (-1 to +1)
3. Arousal delta: expected change in viewer's arousal (-1 to +1)
4. Emotional intensity: 0 (subtle) to 1 (intense)
5. Emotional complexity: 0 (simple) to 1 (nuanced, mixed emotions)
6. Target viewer emotions: which emotional states is this content good for?

Format as JSON:
{
  "primaryTone": "...",
  "valenceDelta": 0.0,
  "arousalDelta": 0.0,
  "intensity": 0.0,
  "complexity": 0.0,
  "targetStates": [
    {"currentValence": 0.0, "currentArousal": 0.0, "description": "..."}
  ]
}
```

---

## Appendix C: Database Schema

### AgentDB Schema (Key-Value Store)

```typescript
// User Profile
type UserKey = `user:${string}`;  // user:usr_abc123xyz
interface UserValue {
  id: string;
  email: string;
  displayName: string;
  emotionalBaseline: { avgValence: number; avgArousal: number; variability: number };
  totalExperiences: number;
  avgReward: number;
  explorationRate: number;
  createdAt: number;
  lastActive: number;
}

// Emotional State
type StateKey = `state:${string}`;  // state:xyz789
interface StateValue extends EmotionalState {}

// Experience
type ExpKey = `exp:${string}`;  // exp:abc789
interface ExpValue extends Experience {}

// Q-Table Entry
type QKey = `q:${string}:${string}:${string}`;  // q:userId:stateHash:contentId
type QValue = number;  // Q-value (0-1)

// Content
type ContentKey = `content:${string}`;  // content:123
interface ContentValue extends Content {}

// User Experience List (Sorted Set)
type UserExpKey = `user:${string}:experiences`;  // user:usr_abc123xyz:experiences
// Members: experienceId (exp_abc789)
// Scores: timestamp (for chronological ordering)

// Visit Count
type VisitKey = `user:${string}:visit:${string}`;  // user:userId:visit:contentId
type VisitValue = number;  // visit count
```

### RuVector Schema

```typescript
// Collection: content_emotions
interface ContentEmotionVector {
  id: string;              // content_123
  vector: Float32Array;    // 1536D embedding
  metadata: {
    contentId: string;
    title: string;
    platform: string;
    primaryTone: string;
    valenceDelta: number;
    arousalDelta: number;
    intensity: number;
    complexity: number;
  };
}

// Collection: emotional_transitions
interface TransitionVector {
  id: string;              // transition_abc123
  vector: Float32Array;    // 1536D embedding (transition representation)
  metadata: {
    userId: string;
    startValence: number;
    startArousal: number;
    endValence: number;
    endArousal: number;
    contentId: string;
    reward: number;
    timestamp: number;
  };
}
```

---

**End of API Specification**

This document provides complete API contracts for the EmotiStream Nexus MVP implementation. Developers can use this as a reference for both frontend integration and backend implementation.
