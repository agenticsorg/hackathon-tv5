# Feedback Collection and Learning Progress API

## Overview

This API provides comprehensive feedback collection, watch tracking, and learning progress analytics for EmotiStream.

## Architecture

```
┌─────────────────────────────────────────────────┐
│          Feedback & Analytics System            │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌──────────────┐  ┌─────────────────────┐    │
│  │ WatchTracker │  │ RewardCalculator    │    │
│  │              │  │ - Emotional align   │    │
│  │ - Start      │  │ - Completion bonus  │    │
│  │ - Pause      │  │ - Rating bonus      │    │
│  │ - Resume     │  └─────────────────────┘    │
│  │ - End        │                              │
│  └──────────────┘  ┌─────────────────────┐    │
│                    │ ProgressAnalytics   │    │
│  ┌──────────────┐  │ - Convergence       │    │
│  │ FeedbackStore│  │ - Journey map       │    │
│  │              │  │ - Performance       │    │
│  │ - Save       │  │ - Trends            │    │
│  │ - Query      │  └─────────────────────┘    │
│  │ - Index      │                              │
│  └──────────────┘                              │
│                                                 │
└─────────────────────────────────────────────────┘
```

## API Endpoints

### 1. Watch Tracking

#### Start Watch Session
```http
POST /api/v1/watch/start
```

**Request:**
```json
{
  "userId": "usr_123",
  "contentId": "content_456",
  "contentTitle": "The Matrix"
}
```

**Response (201):**
```json
{
  "success": true,
  "data": {
    "session": {
      "sessionId": "watch_abc123",
      "userId": "usr_123",
      "contentId": "content_456",
      "contentTitle": "The Matrix",
      "startTime": "2025-12-06T10:00:00.000Z",
      "status": "active"
    }
  },
  "error": null,
  "timestamp": "2025-12-06T10:00:00.000Z"
}
```

#### Pause Session
```http
POST /api/v1/watch/pause
```

**Request:**
```json
{
  "sessionId": "watch_abc123"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "session": {
      "sessionId": "watch_abc123",
      "status": "paused",
      "duration": 1800000,
      "pauseCount": 1
    }
  }
}
```

#### Resume Session
```http
POST /api/v1/watch/resume
```

#### End Session
```http
POST /api/v1/watch/end
```

**Request:**
```json
{
  "sessionId": "watch_abc123",
  "completed": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "session": {
      "sessionId": "watch_abc123",
      "status": "ended",
      "duration": 7200000,
      "completed": true,
      "startTime": "2025-12-06T10:00:00.000Z",
      "endTime": "2025-12-06T12:00:00.000Z"
    }
  }
}
```

#### Get Session Details
```http
GET /api/v1/watch/:sessionId
```

### 2. Feedback Submission

#### Submit Comprehensive Feedback
```http
POST /api/v1/feedback/submit
```

**Request:**
```json
{
  "userId": "usr_123",
  "contentId": "content_456",
  "contentTitle": "The Matrix",
  "sessionId": "watch_abc123",

  "emotionBefore": {
    "valence": -0.3,
    "arousal": -0.2,
    "stress": 0.6
  },

  "emotionAfter": {
    "valence": 0.5,
    "arousal": 0.3,
    "stress": 0.2
  },

  "desiredState": {
    "valence": 0.6,
    "arousal": 0.0,
    "stress": 0.1
  },

  "starRating": 5,
  "completed": true,
  "totalDuration": 8160000
}
```

**Response (201):**
```json
{
  "success": true,
  "data": {
    "feedbackId": "fbk_xyz789",
    "reward": {
      "value": 0.75,
      "components": {
        "emotionalAlignment": 0.82,
        "completionBonus": 1.0,
        "ratingBonus": 1.0
      },
      "explanation": "✨ Great choice! You moved significantly closer to your desired emotional state (+82%). You completed the content. You gave a high rating."
    },
    "emotionComparison": {
      "before": {
        "valence": -0.3,
        "arousal": -0.2,
        "stress": 0.6
      },
      "after": {
        "valence": 0.5,
        "arousal": 0.3,
        "stress": 0.2
      },
      "delta": {
        "valence": 0.8,
        "arousal": 0.5,
        "stress": -0.4
      },
      "improvement": 0.875
    },
    "message": "🎉 Excellent choice! You felt significantly better!",
    "confetti": true
  }
}
```

### 3. Learning Progress

#### Get Overall Progress
```http
GET /api/v1/progress/:userId
```

**Response:**
```json
{
  "success": true,
  "data": {
    "progress": {
      "totalExperiences": 42,
      "completedContent": 35,
      "completionRate": "83.3",

      "averageReward": "0.682",
      "rewardTrend": "improving",
      "recentRewards": [0.75, 0.82, 0.65, 0.71, 0.88, 0.79, 0.84, 0.91, 0.73, 0.80],

      "explorationRate": "12.0%",
      "explorationCount": 5,
      "exploitationCount": 37,

      "convergence": {
        "score": 72.5,
        "stage": "learning",
        "percentage": 73
      },

      "summary": {
        "level": "learning",
        "description": "Building a solid understanding",
        "nextMilestone": {
          "count": 50,
          "description": "Achieve 50 experiences for expert-level personalization"
        }
      }
    }
  }
}
```

#### Get Convergence Analysis
```http
GET /api/v1/progress/:userId/convergence
```

**Response:**
```json
{
  "success": true,
  "data": {
    "convergence": {
      "score": 73,
      "stage": "learning",
      "explanation": "Good progress! (73% confident). After 42 experiences, the system is developing a solid understanding of what you enjoy.",

      "metrics": {
        "qValueStability": "78.5%",
        "rewardVariance": "0.042",
        "explorationRate": "12.0%",
        "policyChanges": 8
      },

      "recommendations": [
        "Your profile is developing well",
        "Keep completing content you start",
        "Rate content honestly to refine recommendations"
      ],

      "progressBar": {
        "percentage": 73,
        "color": "#3b82f6",
        "label": "Learning preferences"
      }
    }
  }
}
```

#### Get Emotional Journey
```http
GET /api/v1/progress/:userId/journey?limit=20
```

**Response:**
```json
{
  "success": true,
  "data": {
    "journey": [
      {
        "experienceNumber": 1,
        "timestamp": "2025-12-01T10:00:00.000Z",
        "contentId": "content_123",
        "contentTitle": "Inception",
        "emotionBefore": { "valence": -0.2, "arousal": 0.1, "stress": 0.5 },
        "emotionAfter": { "valence": 0.6, "arousal": 0.4, "stress": 0.2 },
        "delta": { "valence": 0.8, "arousal": 0.3, "stress": -0.3 },
        "reward": 0.75,
        "completed": true,
        "quadrant": "Excited"
      }
    ],
    "totalPoints": 42
  }
}
```

#### Get Reward Timeline
```http
GET /api/v1/progress/:userId/rewards
```

**Response:**
```json
{
  "success": true,
  "data": {
    "timeline": [
      {
        "experienceNumber": 1,
        "timestamp": "2025-12-01T10:00:00.000Z",
        "reward": 0.75,
        "contentTitle": "Inception",
        "contentId": "content_123",
        "completed": true,
        "starRating": 5
      }
    ],
    "trendLine": [0.65, 0.68, 0.71, 0.72, 0.75],
    "statistics": {
      "average": 0.682,
      "highest": 0.91,
      "lowest": 0.42,
      "recent": [0.75, 0.82, 0.65, 0.71, 0.88, 0.79, 0.84, 0.91, 0.73, 0.80]
    }
  }
}
```

#### Get Content Performance
```http
GET /api/v1/progress/:userId/content
```

**Response:**
```json
{
  "success": true,
  "data": {
    "bestContent": [
      {
        "contentId": "content_456",
        "contentTitle": "The Matrix",
        "timesWatched": 2,
        "averageReward": "0.875",
        "completionRate": "100%",
        "averageRating": "5.0",
        "lastWatched": "2025-12-06T12:00:00.000Z"
      }
    ],
    "worstContent": [
      {
        "contentId": "content_789",
        "contentTitle": "Boring Documentary",
        "timesWatched": 1,
        "averageReward": "0.125",
        "completionRate": "50%",
        "averageRating": "2.0",
        "lastWatched": "2025-12-05T14:00:00.000Z"
      }
    ]
  }
}
```

#### Get Recent Experiences
```http
GET /api/v1/progress/:userId/experiences?limit=10
```

**Response:**
```json
{
  "success": true,
  "data": {
    "experiences": [
      {
        "experienceId": "fbk_abc123",
        "experienceNumber": 42,
        "timestamp": "2025-12-06T12:00:00.000Z",
        "contentId": "content_456",
        "contentTitle": "The Matrix",
        "emotionChange": {
          "before": { "valence": -0.3, "arousal": -0.2, "stress": 0.6 },
          "after": { "valence": 0.5, "arousal": 0.3, "stress": 0.2 },
          "delta": { "valence": 0.8, "arousal": 0.5, "stress": -0.4 },
          "improvement": 0.75
        },
        "reward": 0.75,
        "starRating": 5,
        "completed": true,
        "watchDuration": 7200000,
        "completionPercentage": "88"
      }
    ],
    "total": 42,
    "showing": 10
  }
}
```

## Reward Calculation

Rewards are calculated using a weighted combination:

```
reward = 0.6 * emotionalAlignment + 0.25 * completionBonus + 0.15 * ratingBonus
```

### Components:

1. **Emotional Alignment** (-1 to 1):
   - Measures how well emotions moved toward desired state
   - Based on Euclidean distance in 3D emotional space
   - Improvement = distance_before - distance_after

2. **Completion Bonus** (-1 to 1):
   - 1.0 if completed
   - Partial credit based on percentage watched:
     - < 10%: -0.5
     - < 25%: -0.2
     - < 50%: 0.0
     - < 75%: 0.3
     - < 100%: 0.6

3. **Rating Bonus** (-1 to 1):
   - Maps 1-5 star rating to -1 to 1
   - 3 stars = neutral (0)
   - 5 stars = +1
   - 1 star = -1

## Convergence Analysis

Convergence score (0-100) combines:

- **Reward variance** (40%): Low variance = consistent preferences
- **Q-value stability** (30%): Stable Q-values = converged policy
- **Recent rewards** (15%): High rewards = good fit
- **Experience count** (15%): More experience = more confident

### Stages:

- **Exploring** (0-30): Just getting started
- **Learning** (30-70): Developing understanding
- **Confident** (70-100): Well-established preferences

## Emotional Journey Quadrants

Based on Russell's Circumplex Model:

- **Excited**: valence > 0, arousal > 0
- **Calm**: valence > 0, arousal < 0
- **Sad**: valence < 0, arousal < 0
- **Stressed**: valence < 0, arousal > 0
- **Neutral**: valence ≈ 0, arousal ≈ 0

## Error Codes

- **E003**: Invalid request data (validation error)
- **E004**: Resource not found (session, feedback, etc.)
- **E010**: Internal server error

## Data Persistence

Currently uses file-based persistence (`data/feedback.json`). Will be migrated to AgentDB vector database in future iterations.

## Testing

See `/tests/unit/services/` for comprehensive unit tests:
- `watch-tracker.test.ts`
- `reward-calculator.test.ts`
- `progress-analytics.test.ts`

See `/tests/integration/api/` for integration tests:
- `watch.test.ts`
- `feedback-enhanced.test.ts`
- `progress.test.ts`

## Next Steps

1. Integration with RL policy engine for real Q-values
2. Migration to AgentDB for vector-based storage
3. Real-time progress updates via WebSockets
4. Export/import of learning data
5. A/B testing of reward calculation formulas
