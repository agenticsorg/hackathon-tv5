# FeedbackProcessor Module

**EmotiStream MVP - Feedback and Reward System**

## Overview

The FeedbackProcessor module implements a multi-factor reward calculation system for the EmotiStream emotion-aware recommendation engine. It processes user feedback after content consumption and updates the reinforcement learning policy.

## Architecture

### Components

1. **FeedbackProcessor** (`processor.ts`)
   - Main entry point for processing feedback
   - Orchestrates reward calculation, experience storage, and profile updates
   - Integrates all sub-components

2. **RewardCalculator** (`reward-calculator.ts`)
   - Multi-factor reward formula implementation
   - Cosine similarity for direction alignment
   - Proximity bonus for reaching desired state
   - Completion penalty for early abandonment

3. **ExperienceStore** (`experience-store.ts`)
   - In-memory storage for emotional experiences
   - FIFO buffer with 1000 experience limit per user
   - Provides analytics (average reward, experience count)

4. **UserProfileManager** (`user-profile.ts`)
   - Tracks user learning progress
   - Manages exploration rate decay
   - Calculates convergence score

## Reward Formula

```
reward = 0.6 × directionAlignment + 0.4 × magnitude + proximityBonus
```

### Components

1. **Direction Alignment (60% weight)**
   - Cosine similarity between actual and desired emotional movement
   - Range: [-1, 1]
   - 1.0 = perfect alignment (same direction)
   - 0.0 = perpendicular
   - -1.0 = opposite direction

2. **Magnitude (40% weight)**
   - Normalized Euclidean distance of emotional change
   - Range: [0, 1]
   - Measures how much emotional change occurred

3. **Proximity Bonus**
   - +0.1 bonus if distance to desired state < 0.3
   - Encourages reaching the target state

4. **Completion Penalty** (applied separately)
   - -0.2 for early abandonment (<20% watched)
   - -0.1 for mid abandonment (20-50% watched)
   - -0.05 for late abandonment (50-80% watched)
   - 0 for completion

## Usage

### Basic Example

```typescript
import { FeedbackProcessor } from './feedback';
import type { FeedbackRequest, EmotionalState, DesiredState } from './types';

const processor = new FeedbackProcessor();

// User's emotional state before watching
const stateBefore: EmotionalState = {
  valence: -0.6,
  arousal: 0.2,
  stressLevel: 0.7,
  primaryEmotion: 'sadness',
  emotionVector: new Float32Array([0.1, 0.1, 0.1, 0.1, 0.5, 0.1, 0.05, 0.05]),
  confidence: 0.8,
  timestamp: Date.now() - 1800000, // 30 min ago
};

// User's desired emotional state
const desiredState: DesiredState = {
  targetValence: 0.5,
  targetArousal: -0.2,
  targetStress: 0.3,
  intensity: 'moderate',
  reasoning: 'User wants to feel calm and positive',
};

// User's emotional state after watching
const actualPostState: EmotionalState = {
  valence: 0.3,
  arousal: -0.1,
  stressLevel: 0.4,
  primaryEmotion: 'joy',
  emotionVector: new Float32Array([0.6, 0.1, 0.05, 0.05, 0.1, 0.05, 0.05, 0.1]),
  confidence: 0.8,
  timestamp: Date.now(),
};

// Process feedback
const request: FeedbackRequest = {
  userId: 'user-001',
  contentId: 'content-123',
  actualPostState,
  watchDuration: 30,
  completed: true,
  explicitRating: 5,
};

const response = processor.process(request, stateBefore, desiredState);

console.log('Reward:', response.reward); // 0.7-0.9 (high reward)
console.log('Policy Updated:', response.policyUpdated); // true
console.log('Learning Progress:', response.learningProgress);
```

### Get Learning Progress

```typescript
const progress = processor.getLearningProgress('user-001');

console.log('Total Experiences:', progress.totalExperiences);
console.log('Average Reward:', progress.avgReward);
console.log('Exploration Rate:', progress.explorationRate);
console.log('Convergence Score:', progress.convergenceScore);
```

### Get Recent Experiences

```typescript
const experiences = processor.getRecentExperiences('user-001', 10);

experiences.forEach(exp => {
  console.log(`Content: ${exp.action}, Reward: ${exp.reward}`);
});
```

## Type Definitions

### FeedbackRequest

```typescript
interface FeedbackRequest {
  userId: string;
  contentId: string;
  actualPostState: EmotionalState;
  watchDuration: number;
  completed: boolean;
  explicitRating?: number; // 1-5 star rating
}
```

### FeedbackResponse

```typescript
interface FeedbackResponse {
  reward: number;
  policyUpdated: boolean;
  newQValue: number;
  learningProgress: LearningProgress;
}
```

### LearningProgress

```typescript
interface LearningProgress {
  totalExperiences: number;
  avgReward: number;
  explorationRate: number;
  convergenceScore: number;
}
```

## Implementation Details

### Reward Calculation

The `RewardCalculator.calculateComponents()` method returns a detailed breakdown:

```typescript
interface RewardComponents {
  directionAlignment: number; // [-1, 1]
  magnitude: number; // [0, 1]
  proximityBonus: number; // 0 or 0.1
  completionPenalty: number; // [-0.2, 0]
  totalReward: number; // [-1, 1]
}
```

### Exploration Rate Decay

Exploration rate decays exponentially:

```
explorationRate(t) = max(0.05, 0.3 × 0.995^t)
```

Where:
- Initial rate: 30%
- Minimum rate: 5%
- Decay factor: 0.995 per experience

### Convergence Score

Convergence score is a weighted average of three components:

```
convergence = 0.4 × experienceScore + 0.4 × rewardScore + 0.2 × explorationScore
```

Where:
- `experienceScore = min(1, totalExperiences / 100)`
- `rewardScore = (avgReward + 1) / 2`
- `explorationScore = 1 - normalized(explorationRate)`

## Testing

Run tests:

```bash
npm test -- src/feedback/__tests__/feedback.test.ts
```

Test coverage: **91.93%** (statements)

### Test Suites

1. **FeedbackProcessor Tests**
   - Positive reward for aligned movement
   - Negative reward for misaligned movement
   - Learning progress tracking

2. **RewardCalculator Tests**
   - Direction alignment calculation
   - Proximity bonus
   - Completion penalty
   - Edge cases

3. **ExperienceStore Tests**
   - Store and retrieve experiences
   - Average reward calculation
   - FIFO buffer enforcement

4. **UserProfileManager Tests**
   - Profile initialization
   - Stats updates
   - Exploration rate decay
   - Convergence score calculation

## Performance Characteristics

- **Time Complexity**: O(1) for all operations
- **Space Complexity**: O(n) where n = number of users
- **Memory**: ~1KB per user (1000 experiences)

## Integration Points

### With EmotionDetector

```typescript
import { EmotionDetector } from '../emotion';

const detector = new EmotionDetector();
const actualPostState = await detector.analyze(userText);
```

### With RLPolicyEngine (Future)

```typescript
import { RLPolicyEngine } from '../rl';

const rlEngine = new RLPolicyEngine();
const newQValue = await rlEngine.updateQValue(state, contentId, reward);
```

## Future Enhancements

1. **Persistent Storage**
   - Replace in-memory store with AgentDB
   - Enable cross-session learning

2. **Batch Learning**
   - Experience replay from stored experiences
   - Off-policy learning

3. **Advanced Reward Shaping**
   - Time-based decay
   - User preference weighting
   - Content quality signals

4. **Multi-Objective Optimization**
   - Balance exploration vs exploitation
   - Diversity in recommendations
   - Fairness constraints

## API Endpoints (Future)

```typescript
// POST /api/v1/feedback
{
  "userId": "user-001",
  "contentId": "content-123",
  "actualPostState": { ... },
  "watchDuration": 30,
  "completed": true
}

// Response
{
  "reward": 0.75,
  "policyUpdated": true,
  "newQValue": 0.68,
  "learningProgress": {
    "totalExperiences": 42,
    "avgReward": 0.63,
    "explorationRate": 0.15,
    "convergenceScore": 0.72
  }
}
```

## References

- Architecture Spec: `/docs/specs/emotistream/architecture/ARCH-FeedbackAPI-CLI.md`
- Emotion Types: `/src/emotion/types.ts`
- RL Types: `/src/rl/types.ts`

## License

MIT
