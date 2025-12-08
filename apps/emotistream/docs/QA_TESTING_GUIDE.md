# EmotiStream QA & Testing Guide

This guide provides comprehensive testing procedures for QA engineers and developers to validate the EmotiStream application.

## Test Environment Setup

### Prerequisites
```bash
cd apps/emotistream
npm install
```

### Environment Configuration
```bash
cp .env.example .env
# Optional: Add GEMINI_API_KEY for integration tests with real AI
```

---

## Running Tests

### All Tests
```bash
npm test
```

### Unit Tests Only
```bash
npm run test:unit
```

### Integration Tests
```bash
npm run test:integration
```

### Watch Mode (Development)
```bash
npm run test:watch
```

### Coverage Report
```bash
npm run test:coverage
```

---

## Test Structure

```
tests/
├── unit/
│   ├── emotion/           # EmotionDetector tests
│   │   ├── detector.test.ts
│   │   ├── mappers.test.ts
│   │   └── state.test.ts
│   ├── rl/                # RLPolicyEngine tests
│   │   ├── q-table.test.ts
│   │   ├── policy-engine.test.ts
│   │   ├── reward-calculator.test.ts
│   │   ├── epsilon-greedy.test.ts
│   │   └── ucb.test.ts
│   ├── content/           # ContentProfiler tests
│   │   ├── profiler.test.ts
│   │   ├── vector-store.test.ts
│   │   ├── embedding-generator.test.ts
│   │   └── mock-catalog.test.ts
│   ├── recommendations/   # RecommendationEngine tests
│   │   ├── engine.test.ts
│   │   ├── ranker.test.ts
│   │   ├── outcome-predictor.test.ts
│   │   └── reasoning.test.ts
│   └── feedback/          # FeedbackProcessor tests
│       ├── processor.test.ts
│       ├── reward-calculator.test.ts
│       ├── experience-store.test.ts
│       └── user-profile.test.ts
└── integration/
    └── api/               # REST API integration tests
        ├── emotion.test.ts
        ├── recommend.test.ts
        └── feedback.test.ts
```

---

## Manual Testing Procedures

### 1. API Health Check

**Test**: Verify server is running and healthy.

```bash
# Start server
npm run start:api &

# Test health endpoint
curl http://localhost:3000/health
```

**Expected Response**:
```json
{
  "status": "ok",
  "version": "1.0.0"
}
```

---

### 2. Emotion Analysis Endpoint

**Test**: Analyze emotional state from text input.

```bash
curl -X POST http://localhost:3000/api/v1/emotion/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "test_user_001",
    "input": "I am feeling very stressed and anxious about my upcoming presentation",
    "desiredMood": "confident and calm"
  }'
```

**Expected Response Structure**:
```json
{
  "userId": "test_user_001",
  "currentState": {
    "valence": -0.4,      // -1 to 1 (negative emotion)
    "arousal": 0.6,       // -1 to 1 (high activation)
    "stress": 0.8,        // 0 to 1 (high stress)
    "dominantEmotion": "anxiety",
    "emotionVector": [...]
  },
  "desiredState": {
    "valence": 0.5,       // Positive target
    "arousal": 0.2        // Moderate activation
  }
}
```

**Validation Criteria**:
- [ ] Response includes `currentState` with valence, arousal, stress
- [ ] Values are within expected ranges
- [ ] `desiredState` is parsed from input
- [ ] Response time < 2 seconds

---

### 3. Recommendation Endpoint

**Test**: Get personalized content recommendations.

```bash
curl -X POST http://localhost:3000/api/v1/recommend \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "test_user_001",
    "currentState": {
      "valence": -0.4,
      "arousal": 0.6,
      "stress": 0.8
    },
    "desiredState": {
      "valence": 0.5,
      "arousal": -0.2
    },
    "limit": 5
  }'
```

**Expected Response Structure**:
```json
{
  "userId": "test_user_001",
  "recommendations": [
    {
      "contentId": "mock_meditation_001",
      "title": "Deep Relaxation",
      "category": "meditation",
      "score": 0.87,
      "qValue": 0.5,
      "similarity": 0.92,
      "predictedOutcome": {
        "valence": 0.4,
        "arousal": -0.3
      },
      "reasoning": "High stress reduction potential..."
    }
  ]
}
```

**Validation Criteria**:
- [ ] Returns requested number of recommendations (default 3)
- [ ] Each recommendation has score, qValue, similarity
- [ ] Recommendations are sorted by score (descending)
- [ ] `predictedOutcome` shows expected emotional state
- [ ] `reasoning` provides human-readable explanation

---

### 4. Feedback Endpoint

**Test**: Submit viewing feedback to train the model.

```bash
curl -X POST http://localhost:3000/api/v1/feedback \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "test_user_001",
    "contentId": "mock_meditation_001",
    "preState": {
      "valence": -0.4,
      "arousal": 0.6,
      "stress": 0.8
    },
    "postState": {
      "valence": 0.3,
      "arousal": -0.2,
      "stress": 0.2
    },
    "desiredState": {
      "valence": 0.5,
      "arousal": -0.2
    },
    "watchDuration": 900,
    "completed": true,
    "rating": 5
  }'
```

**Expected Response**:
```json
{
  "success": true,
  "reward": 0.85,
  "newQValue": 0.58,
  "learningProgress": {
    "totalExperiences": 1,
    "episodesCompleted": 1,
    "averageReward": 0.85
  }
}
```

**Validation Criteria**:
- [ ] `reward` is between -1 and 1
- [ ] `reward` is positive when postState moves toward desiredState
- [ ] `newQValue` reflects learning update
- [ ] `learningProgress` tracks cumulative stats

---

### 5. Learning Progress Endpoint

**Test**: Check user's learning history.

```bash
curl http://localhost:3000/api/v1/feedback/progress/test_user_001
```

**Expected Response**:
```json
{
  "userId": "test_user_001",
  "totalExperiences": 5,
  "averageReward": 0.72,
  "episodesCompleted": 4,
  "explorationRate": 0.12,
  "topCategories": ["meditation", "music"]
}
```

---

## Exploratory Testing Scenarios

### Scenario 1: Cold Start (New User)

**Objective**: Verify system handles users with no history.

1. Use a new userId that has never been seen
2. Request recommendations
3. Verify system returns diverse recommendations (exploration mode)
4. Submit feedback for first content
5. Verify learning progress shows 1 experience

**Expected Behavior**:
- Initial recommendations should be diverse (high exploration)
- Q-values start at default (0.5)
- First feedback should update Q-table

---

### Scenario 2: Learning Over Time

**Objective**: Verify recommendations improve with feedback.

1. Submit 5 feedbacks for meditation content with high ratings
2. Submit 3 feedbacks for action movies with low ratings
3. Request new recommendations for "want to relax"
4. Verify meditation ranks higher than action

**Expected Behavior**:
- Meditation Q-values should increase
- Action movie Q-values should decrease
- Recommendations should reflect learned preferences

---

### Scenario 3: Emotional State Transitions

**Objective**: Verify different emotional inputs produce appropriate recommendations.

| Input Mood | Expected Content Types |
|------------|----------------------|
| "stressed and anxious" | Meditation, calm music |
| "bored and tired" | Energetic shorts, comedy |
| "sad and lonely" | Feel-good movies, uplifting docs |
| "excited and happy" | Action, adventure, party music |

---

### Scenario 4: Edge Cases

**Test each scenario**:

1. **Empty input**: Send empty `input` string
   - Expected: Error response with validation message

2. **Invalid emotional values**: Send valence > 1 or arousal < -1
   - Expected: Values should be clamped or rejected

3. **Missing required fields**: Omit `userId` from request
   - Expected: 400 Bad Request with clear error

4. **Very long input**: Send 10,000 character mood description
   - Expected: Handle gracefully (truncate or reject)

5. **Concurrent requests**: Send 10 simultaneous recommendations
   - Expected: All return successfully

---

## Performance Testing

### Response Time Benchmarks

| Endpoint | Target | Max Acceptable |
|----------|--------|----------------|
| GET /health | < 50ms | 200ms |
| POST /emotion/analyze | < 500ms | 2000ms |
| POST /recommend | < 200ms | 1000ms |
| POST /feedback | < 100ms | 500ms |

### Load Testing

```bash
# Using Apache Bench (ab)
ab -n 100 -c 10 http://localhost:3000/health

# Using wrk
wrk -t4 -c100 -d30s http://localhost:3000/health
```

---

## Validation Checklists

### Emotion Detection Module
- [ ] Valence correctly maps positive/negative emotions
- [ ] Arousal correctly maps activation levels
- [ ] Stress detection works for stress-related keywords
- [ ] Plutchik emotions are extracted (joy, fear, anger, etc.)
- [ ] State hash produces consistent values for same input

### RL Policy Engine
- [ ] Q-table persists across server restarts
- [ ] Epsilon decreases over time (exploration decay)
- [ ] UCB exploration bonus works correctly
- [ ] Reward calculation matches formula
- [ ] Q-value updates follow Bellman equation

### Recommendation Engine
- [ ] Hybrid ranking uses 70% Q-value, 30% similarity
- [ ] Outcome predictor estimates post-state
- [ ] Reasoning generator produces coherent explanations
- [ ] Content filtering by category works

### Feedback Processor
- [ ] Positive feedback increases Q-value
- [ ] Negative feedback decreases Q-value
- [ ] Completion bonus applied for finished content
- [ ] Experience store persists feedback history

---

## Bug Reporting Template

When reporting bugs, include:

```markdown
**Title**: [Brief description]

**Environment**:
- Node version:
- OS:
- EmotiStream version:

**Steps to Reproduce**:
1.
2.
3.

**Expected Result**:

**Actual Result**:

**Request/Response** (if API):
```bash
# Request
curl ...

# Response
{...}
```

**Logs**:
```
[paste relevant logs]
```
```

---

## Continuous Integration

Tests run automatically on:
- Pull request creation
- Commits to main branch
- Nightly builds

### CI Test Command
```bash
npm run test:ci
```

This runs:
1. Linting
2. Type checking
3. Unit tests
4. Integration tests
5. Coverage report (minimum 70% required)

---

## Support

- **Test Failures**: Check `tests/` folder for test file locations
- **Coverage Gaps**: Run `npm run test:coverage` to identify
- **Flaky Tests**: Report in issues with `flaky-test` label
