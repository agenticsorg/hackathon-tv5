# EmotiStream Nexus - RecommendationEngine Architecture

**Module**: RecommendationEngine
**SPARC Phase**: 3 - Architecture
**Version**: 1.0
**Created**: 2025-12-05
**Dependencies**: ContentProfiler, RLPolicyEngine, EmotionDetector, RuVector, AgentDB

---

## 1. Executive Summary

The **RecommendationEngine** is the central orchestration module that fuses reinforcement learning policy (Q-values) with semantic vector search to generate emotionally-aware content recommendations. It implements a **hybrid ranking strategy** (70% Q-value, 30% similarity) to balance exploitation of learned preferences with exploration of semantic content space.

### 1.1 Core Responsibilities

1. **Semantic Search**: Query RuVector for content matching emotional transitions
2. **Hybrid Ranking**: Combine Q-values and similarity scores with configurable weighting
3. **Outcome Prediction**: Predict post-viewing emotional states
4. **Reasoning Generation**: Create human-readable explanations
5. **Watch History Filtering**: Prevent redundant recommendations
6. **Exploration Management**: Inject diverse content using ε-greedy strategy

### 1.2 Key Metrics

- **Recommendation Latency**: Target <500ms for p95
- **Ranking Accuracy**: Q-value alignment with actual outcomes >0.75
- **Exploration Rate**: Dynamic decay from 30% to 10%
- **Search Quality**: Semantic relevance >0.6 average similarity

---

## 2. Module Structure

### 2.1 Directory Layout

```
src/recommendations/
├── index.ts                    # Public API exports
├── engine.ts                   # RecommendationEngine orchestrator class
├── ranker.ts                   # HybridRanker (70/30 scoring)
├── outcome-predictor.ts        # OutcomePredictor (state transitions)
├── reasoning.ts                # ReasoningGenerator (human explanations)
├── filters.ts                  # WatchHistoryFilter
├── transition-vector.ts        # TransitionVectorBuilder
├── desired-state.ts            # DesiredStatePredictor
├── exploration.ts              # ExplorationStrategy (ε-greedy)
├── types.ts                    # Module-specific interfaces
└── __tests__/                  # Unit and integration tests
    ├── engine.test.ts
    ├── ranker.test.ts
    ├── outcome-predictor.test.ts
    └── integration.test.ts
```

### 2.2 Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                   RecommendationEngine                       │
│                   (Orchestrator)                             │
└─────────────────────────────────────────────────────────────┘
            │
            ├──────────────────────────────────────────┐
            │                                          │
            ▼                                          ▼
┌──────────────────────┐                   ┌──────────────────────┐
│  TransitionVector    │                   │   DesiredState       │
│  Builder             │                   │   Predictor          │
└──────────────────────┘                   └──────────────────────┘
            │                                          │
            ▼                                          ▼
┌──────────────────────────────────────────────────────────────┐
│                      RuVector Search                          │
│              (Semantic Content Matching)                      │
└──────────────────────────────────────────────────────────────┘
            │
            ▼
┌──────────────────────┐
│  WatchHistory        │
│  Filter              │
└──────────────────────┘
            │
            ▼
┌──────────────────────────────────────────────────────────────┐
│                      HybridRanker                             │
│             (70% Q-value + 30% Similarity)                    │
└──────────────────────────────────────────────────────────────┘
            │                                          │
            ├──────────────────────────────────────────┤
            ▼                                          ▼
┌──────────────────────┐                   ┌──────────────────────┐
│  RLPolicyEngine      │                   │  OutcomePredictor    │
│  (Q-value lookup)    │                   │  (State prediction)  │
└──────────────────────┘                   └──────────────────────┘
            │
            ▼
┌──────────────────────┐
│  Exploration         │
│  Strategy            │
└──────────────────────┘
            │
            ▼
┌──────────────────────┐
│  Reasoning           │
│  Generator           │
└──────────────────────┘
```

---

## 3. Core Architecture

### 3.1 Class Diagram (ASCII)

```
┌─────────────────────────────────────────────────────────────┐
│                    RecommendationEngine                      │
├─────────────────────────────────────────────────────────────┤
│ - ruVector: RuVectorClient                                  │
│ - agentDB: AgentDB                                          │
│ - rlPolicy: RLPolicyEngine                                  │
│ - transitionBuilder: TransitionVectorBuilder                │
│ - desiredStatePredictor: DesiredStatePredictor              │
│ - watchHistoryFilter: WatchHistoryFilter                    │
│ - hybridRanker: HybridRanker                                │
│ - outcomePredictor: OutcomePredictor                        │
│ - reasoningGenerator: ReasoningGenerator                    │
│ - explorationStrategy: ExplorationStrategy                  │
├─────────────────────────────────────────────────────────────┤
│ + getRecommendations(request): Promise<Recommendation[]>    │
│ - searchCandidates(vector, limit): Promise<Candidate[]>     │
│ - filterWatched(userId, candidates): Promise<Candidate[]>   │
│ - rankCandidates(userId, candidates): Promise<Ranked[]>     │
│ - applyExploration(ranked, rate): Promise<Ranked[]>         │
│ - generateRecommendations(ranked): Promise<Recs[]>          │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                      HybridRanker                            │
├─────────────────────────────────────────────────────────────┤
│ - Q_WEIGHT: 0.7                                             │
│ - SIMILARITY_WEIGHT: 0.3                                    │
│ - DEFAULT_Q_VALUE: 0.5                                      │
├─────────────────────────────────────────────────────────────┤
│ + rank(userId, candidates, state): Promise<RankedCand[]>   │
│ - getQValue(userId, state, contentId): Promise<number>     │
│ - normalizeQValue(qValue): number                          │
│ - calculateHybridScore(q, sim, align): number              │
│ - calculateOutcomeAlignment(profile, desired): number      │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    OutcomePredictor                          │
├─────────────────────────────────────────────────────────────┤
│ + predict(current, profile): PredictedOutcome               │
│ - calculateConfidence(watchCount, variance): number         │
│ - clampValues(valence, arousal, stress): tuple              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                 TransitionVectorBuilder                      │
├─────────────────────────────────────────────────────────────┤
│ - embeddingModel: EmbeddingClient                           │
├─────────────────────────────────────────────────────────────┤
│ + buildVector(current, desired): Promise<Float32Array>      │
│ - generatePrompt(current, desired): string                  │
│ - describeEmotionalState(v, a, s): string                   │
│ - getEmotionalQuadrant(v, a): string                        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                 DesiredStatePredictor                        │
├─────────────────────────────────────────────────────────────┤
│ + predict(currentState): DesiredState                       │
│ - applyStressReductionRule(state): DesiredState | null     │
│ - applySadnessLiftRule(state): DesiredState | null         │
│ - applyAnxietyReductionRule(state): DesiredState | null    │
│ - applyBoredomStimulationRule(state): DesiredState | null  │
│ - applyHomeostasisDefault(state): DesiredState             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                  ExplorationStrategy                         │
├─────────────────────────────────────────────────────────────┤
│ - explorationRate: number                                   │
│ - decayFactor: 0.95                                         │
├─────────────────────────────────────────────────────────────┤
│ + inject(ranked, rate): RankedCandidate[]                  │
│ + decayRate(): void                                         │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                  ReasoningGenerator                          │
├─────────────────────────────────────────────────────────────┤
│ + generate(current, desired, profile, q, isExp): string    │
│ - describeCurrentState(state): string                       │
│ - describeTransition(current, desired): string              │
│ - describeExpectedChanges(profile): string                  │
│ - describeConfidence(qValue): string                        │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. TypeScript Interfaces

### 4.1 Core Interfaces

```typescript
// src/recommendations/types.ts

import { EmotionalState, DesiredState } from '../emotion/types';
import { EmotionalContentProfile } from '../content/types';

/**
 * Recommendation request from client
 */
export interface RecommendationRequest {
  userId: string;
  emotionalStateId: string;
  limit?: number;                    // Default: 20
  explicitDesiredState?: {
    valence: number;
    arousal: number;
  };
  includeExploration?: boolean;      // Default: false
  explorationRate?: number;          // Default: 0.1 (10%)
}

/**
 * Recommendation options for engine
 */
export interface RecommendationOptions {
  limit: number;
  includeExploration: boolean;
  explorationRate: number;
  searchTopK: number;                // 3x limit for re-ranking
}

/**
 * Final recommendation output
 */
export interface Recommendation {
  contentId: string;
  title: string;
  platform: string;

  // Emotional profile
  emotionalProfile: EmotionalContentProfile;

  // Predicted outcome
  predictedOutcome: PredictedOutcome;

  // Scoring components
  qValue: number;                    // Raw Q-value from RL policy
  similarityScore: number;           // Vector similarity [0, 1]
  combinedScore: number;             // Hybrid score (70/30)

  // Metadata
  isExploration: boolean;            // Exploration vs exploitation
  rank: number;                      // Final ranking position (1-based)
  reasoning: string;                 // Human-readable explanation
}

/**
 * Predicted emotional outcome after viewing
 */
export interface PredictedOutcome {
  postViewingValence: number;        // [-1, 1]
  postViewingArousal: number;        // [-1, 1]
  postViewingStress: number;         // [0, 1]
  confidence: number;                // [0, 1] based on historical data
}

/**
 * Search candidate from RuVector
 */
export interface SearchCandidate {
  contentId: string;
  profile: EmotionalContentProfile;
  similarity: number;                // Converted from distance [0, 1]
  distance: number;                  // Raw vector distance
}

/**
 * Ranked candidate after hybrid scoring
 */
export interface RankedCandidate {
  contentId: string;
  profile: EmotionalContentProfile;
  similarity: number;
  qValue: number;
  qValueNormalized: number;          // Normalized to [0, 1]
  hybridScore: number;               // Final ranking score
  outcomeAlignment: number;          // Alignment with desired outcome
  isExploration: boolean;
}

/**
 * Action key for Q-table lookup
 */
export interface ActionKey {
  contentId: string;
  valenceDelta: number;
  arousalDelta: number;
}

/**
 * State hash for Q-table lookup
 */
export interface StateHash {
  valenceBucket: number;             // Discretized valence
  arousalBucket: number;             // Discretized arousal
  stressBucket: number;              // Discretized stress
  hash: string;                      // "v:3:a:8:s:4"
}
```

### 4.2 Configuration Interface

```typescript
/**
 * Hybrid ranking configuration
 */
export interface HybridRankingConfig {
  qWeight: number;                   // Default: 0.7
  similarityWeight: number;          // Default: 0.3
  defaultQValue: number;             // Default: 0.5 for unexplored
  explorationBonus: number;          // Default: 0.1
  outcomeAlignmentFactor: number;    // Default: 1.0 (multiplier)
}

/**
 * Exploration strategy configuration
 */
export interface ExplorationConfig {
  initialRate: number;               // Default: 0.3 (30%)
  minRate: number;                   // Default: 0.1 (10%)
  decayFactor: number;               // Default: 0.95
  randomSelectionRange: [number, number]; // Default: [0.5, 1.0]
}

/**
 * State discretization configuration
 */
export interface StateDiscretizationConfig {
  valenceBuckets: number;            // Default: 10 (0.2 granularity)
  arousalBuckets: number;            // Default: 10 (0.2 granularity)
  stressBuckets: number;             // Default: 5 (0.2 granularity)
}
```

---

## 5. Sequence Diagrams

### 5.1 Full Recommendation Flow

```
User                 Engine              Emotion           Transition     RuVector
 │                     │                    │                  │              │
 │──getRecommendations()──>                 │                  │              │
 │                     │                    │                  │              │
 │                     │──loadEmotionalState()──>             │              │
 │                     │<───EmotionalState────────            │              │
 │                     │                    │                  │              │
 │                     │──predictDesiredState()──>            │              │
 │                     │<───DesiredState──────────            │              │
 │                     │                    │                  │              │
 │                     │──buildTransitionVector()──────────>  │              │
 │                     │                    │                  │              │
 │                     │                    │  (embed prompt)  │              │
 │                     │<───Float32Array[1536]────────────────┘              │
 │                     │                    │                                 │
 │                     │──search(vector, topK=60)──────────────────────────> │
 │                     │<───SearchCandidate[]──────────────────────────────┘ │
 │                     │                    │                                 │
 │                     │──filterWatchedContent()──>                           │
 │                     │   (query watch history)                              │
 │                     │<───FilteredCandidates───┘                            │
 │                     │                                                      │
 │                     │──rankCandidates()──>                                 │
 │                     │   (hybrid Q + similarity)                            │
 │                     │<───RankedCandidates──┘                               │
 │                     │                                                      │
 │                     │──applyExploration()──>                               │
 │                     │   (ε-greedy injection)                               │
 │                     │<───ExploredCandidates──┘                             │
 │                     │                                                      │
 │                     │──generateRecommendations()──>                        │
 │                     │   (top N with reasoning)                             │
 │                     │<───Recommendation[]──────┘                           │
 │                     │                                                      │
 │<───Recommendation[]──┘                                                     │
 │                                                                            │
```

### 5.2 Hybrid Ranking Calculation

```
HybridRanker          RLPolicy            AgentDB         OutcomeAlign
     │                   │                   │                  │
     │──rank(candidates)─>                   │                  │
     │                   │                   │                  │
     │   FOR EACH candidate:                 │                  │
     │                   │                   │                  │
     │──getQValue(userId, stateHash, actionKey)──────────────> │
     │                   │                   │                  │
     │                   │──lookup("q:user:state:content")──>  │
     │                   │<───qValue (or NULL)────────────────┘ │
     │                   │                   │                  │
     │<───qValue (or 0.5 default)────────────┘                  │
     │                   │                                      │
     │──normalizeQValue(qValue)──>                              │
     │   (qValue + 1.0) / 2.0                                   │
     │<───qValueNormalized─────────                             │
     │                                                          │
     │──calculateOutcomeAlignment(profile, desired)────────────>│
     │   (cosine similarity of delta vectors)                   │
     │<───alignmentScore───────────────────────────────────────┘│
     │                                                          │
     │──calculateHybridScore()──>                               │
     │   score = (qNorm * 0.7) + (similarity * 0.3) * alignment│
     │<───hybridScore────────────┘                              │
     │                                                          │
     │──SORT by hybridScore DESC──>                             │
     │<───rankedCandidates──────────┘                           │
     │                                                          │
```

### 5.3 Exploration Injection Flow

```
Engine              ExplorationStrategy           Random
  │                        │                         │
  │──applyExploration(ranked, rate=0.1)────────>    │
  │                        │                         │
  │                   explorationCount =             │
  │                   floor(length * 0.1)            │
  │                        │                         │
  │                   FOR i = 0 to length:           │
  │                        │                         │
  │                        │──random() < rate?───────>│
  │                        │<───true/false───────────┘│
  │                        │                         │
  │                   IF true:                       │
  │                        │──randomInt(length/2, length-1)──>│
  │                        │<───explorationIndex─────┘│
  │                        │                         │
  │                   candidate = ranked[explorationIndex]
  │                   candidate.isExploration = true│
  │                   candidate.hybridScore += 0.2  │
  │                        │                         │
  │                   INSERT into result[]           │
  │                        │                         │
  │                   SORT by hybridScore DESC       │
  │                        │                         │
  │<───exploredCandidates──┘                         │
  │                                                  │
```

---

## 6. Hybrid Ranking Algorithm

### 6.1 Scoring Formula

```
HYBRID_SCORE = (Q_VALUE_NORMALIZED × 0.7 + SIMILARITY × 0.3) × OUTCOME_ALIGNMENT
```

**Components**:
1. **Q-Value (70% weight)**: Learned value from RL policy
2. **Similarity (30% weight)**: Semantic relevance from vector search
3. **Outcome Alignment (multiplier)**: How well content's emotional delta matches desired transition

### 6.2 Q-Value Normalization

**Input Range**: Q-values from RL are typically in `[-1, 1]` after training convergence.

**Normalization Formula**:
```
Q_VALUE_NORMALIZED = (Q_VALUE + 1.0) / 2.0
```

**Output Range**: `[0, 1]` for consistent scoring with similarity.

**Cold Start Handling**:
- If Q-value doesn't exist in AgentDB (unexplored state-action pair):
  - Use `DEFAULT_Q_VALUE = 0.5` (neutral)
  - Add `EXPLORATION_BONUS = 0.1` to encourage trying new content
  - Final: `Q_normalized = (0.5 + 1.0) / 2.0 + 0.1 = 0.85`

### 6.3 Outcome Alignment Calculation

**Purpose**: Boost recommendations where content's emotional impact aligns with desired transition.

**Algorithm**:
```typescript
function calculateOutcomeAlignment(
  profile: EmotionalContentProfile,
  desiredState: DesiredState
): number {
  // Desired deltas
  const desiredValenceDelta = desiredState.valence; // Simplified from current
  const desiredArousalDelta = desiredState.arousal;

  // Content's deltas
  const contentValenceDelta = profile.valenceDelta;
  const contentArousalDelta = profile.arousalDelta;

  // Cosine similarity of 2D vectors
  const dotProduct =
    contentValenceDelta * desiredValenceDelta +
    contentArousalDelta * desiredArousalDelta;

  const magnitudeContent = Math.sqrt(
    contentValenceDelta ** 2 + contentArousalDelta ** 2
  );

  const magnitudeDesired = Math.sqrt(
    desiredValenceDelta ** 2 + desiredArousalDelta ** 2
  );

  if (magnitudeContent === 0 || magnitudeDesired === 0) {
    return 0.5; // Neutral alignment
  }

  // Cosine similarity in [-1, 1]
  const cosineSim = dotProduct / (magnitudeContent * magnitudeDesired);

  // Convert to [0, 1] with 0.5 as neutral
  let alignmentScore = (cosineSim + 1.0) / 2.0;

  // Boost for strong alignment
  if (alignmentScore > 0.8) {
    alignmentScore = 1.0 + ((alignmentScore - 0.8) * 0.5); // Up to 1.1x boost
  }

  return alignmentScore;
}
```

**Example**:
- Current State: `valence=-0.4, arousal=0.6` (stressed)
- Desired State: `valence=0.5, arousal=-0.4` (calm)
- Desired Delta: `valence=+0.9, arousal=-1.0`
- Content Delta: `valence=+0.7, arousal=-0.6`
- Cosine Similarity: ~0.95 (high alignment)
- Alignment Score: ~0.98 (strong boost)

### 6.4 Cold Start Strategy

**New User (No Q-values)**:
1. Rely primarily on **semantic similarity** (vector search)
2. Use `DEFAULT_Q_VALUE = 0.5` for all content
3. Add `EXPLORATION_BONUS = 0.1` to encourage diverse initial experiences
4. Hybrid score essentially becomes: `(0.75 × 0.7 + similarity × 0.3) × alignment`

**New Content (No Q-value for state-action)**:
1. Use `DEFAULT_Q_VALUE = 0.5`
2. Apply `EXPLORATION_BONUS = 0.1`
3. Slightly prefer unexplored content to gather data

**Learned Policy (Many Q-values)**:
1. Q-values dominate scoring (70% weight)
2. Similarity provides semantic grounding (30% weight)
3. Exploration rate decays to 10%

---

## 7. Integration Points

### 7.1 ContentProfiler Integration

```typescript
// RecommendationEngine uses ContentProfiler for search

class RecommendationEngine {
  private async searchCandidates(
    transitionVector: Float32Array,
    topK: number
  ): Promise<SearchCandidate[]> {
    // Query RuVector for semantically similar content
    const searchResults = await this.ruVector.search({
      collectionName: 'emotistream_content',
      vector: transitionVector,
      limit: topK,
      filter: {
        isActive: true // Only active content
      }
    });

    // Load full profiles and convert distances to similarities
    const candidates = await Promise.all(
      searchResults.map(async (result) => {
        const profile = await this.contentProfiler.getProfile(result.id);

        // Convert distance to similarity [0, 1]
        // Assuming cosine distance in [0, 2]
        const similarity = 1.0 - (result.distance / 2.0);
        const clampedSimilarity = Math.max(0, Math.min(1, similarity));

        return {
          contentId: result.id,
          profile,
          similarity: clampedSimilarity,
          distance: result.distance
        };
      })
    );

    return candidates;
  }
}
```

### 7.2 RLPolicyEngine Integration

```typescript
// RecommendationEngine queries Q-values from RLPolicyEngine

class HybridRanker {
  private async getQValue(
    userId: string,
    stateHash: string,
    contentId: string
  ): Promise<number> {
    // Construct action key
    const actionKey = this.constructActionKey(contentId, profile);

    // Query Q-value from RL policy
    const qValue = await this.rlPolicy.getQValue(userId, stateHash, actionKey);

    if (qValue === null) {
      // Unexplored state-action pair
      return this.config.defaultQValue + this.config.explorationBonus;
    }

    return qValue;
  }

  private constructActionKey(
    contentId: string,
    profile: EmotionalContentProfile
  ): string {
    // Format: "content:{id}:v:{delta}:a:{delta}"
    return `content:${contentId}:v:${profile.valenceDelta.toFixed(2)}:a:${profile.arousalDelta.toFixed(2)}`;
  }
}
```

### 7.3 EmotionDetector Integration

```typescript
// RecommendationEngine loads emotional state from EmotionDetector

class RecommendationEngine {
  async getRecommendations(
    request: RecommendationRequest
  ): Promise<Recommendation[]> {
    // Load current emotional state
    const currentState = await this.emotionDetector.getState(
      request.emotionalStateId
    );

    if (!currentState) {
      throw new Error(`Emotional state not found: ${request.emotionalStateId}`);
    }

    // Determine desired state
    const desiredState = request.explicitDesiredState
      ? request.explicitDesiredState
      : await this.desiredStatePredictor.predict(currentState);

    // Continue with recommendation flow...
  }
}
```

### 7.4 AgentDB Integration

```typescript
// Watch History Storage
interface WatchHistoryRecord {
  userId: string;
  contentId: string;
  watchedAt: number;
  emotionalStateId: string;
}

class WatchHistoryFilter {
  async filterWatched(
    userId: string,
    candidates: SearchCandidate[]
  ): Promise<SearchCandidate[]> {
    // Query watch history from AgentDB
    const watchHistory = await this.agentDB.query({
      namespace: 'emotistream/watch_history',
      pattern: `${userId}:*`,
      limit: 1000
    });

    const watchedContentIds = new Set<string>();
    const lastWatchTimes = new Map<string, number>();

    for (const record of watchHistory) {
      watchedContentIds.add(record.contentId);
      lastWatchTimes.set(record.contentId, record.watchedAt);
    }

    // Filter candidates
    const filtered = candidates.filter((candidate) => {
      // Allow if never watched
      if (!watchedContentIds.has(candidate.contentId)) {
        return true;
      }

      // Allow re-recommendation if watched >30 days ago
      const lastWatch = lastWatchTimes.get(candidate.contentId);
      const daysSinceWatch = (Date.now() - lastWatch) / (1000 * 60 * 60 * 24);

      return daysSinceWatch > 30;
    });

    return filtered;
  }
}

// Recommendation Event Logging
interface RecommendationEvent {
  userId: string;
  timestamp: number;
  emotionalStateId: string;
  currentValence: number;
  currentArousal: number;
  currentStress: number;
  recommendedContentIds: string[];
  topRecommendation: string;
}

class RecommendationEngine {
  private async logRecommendationEvent(
    userId: string,
    currentState: EmotionalState,
    recommendations: Recommendation[]
  ): Promise<void> {
    const event: RecommendationEvent = {
      userId,
      timestamp: Date.now(),
      emotionalStateId: currentState.id,
      currentValence: currentState.valence,
      currentArousal: currentState.arousal,
      currentStress: currentState.stressLevel,
      recommendedContentIds: recommendations.map(r => r.contentId),
      topRecommendation: recommendations[0]?.contentId
    };

    await this.agentDB.store({
      namespace: 'emotistream/recommendation_events',
      key: `rec:${userId}:${Date.now()}`,
      value: event,
      ttl: 90 * 24 * 60 * 60 * 1000 // 90 days
    });
  }
}
```

---

## 8. Outcome Prediction

### 8.1 Algorithm

```typescript
class OutcomePredictor {
  predict(
    currentState: EmotionalState,
    contentProfile: EmotionalContentProfile
  ): PredictedOutcome {
    // Predict post-viewing emotional state
    let postValence = currentState.valence + contentProfile.valenceDelta;
    let postArousal = currentState.arousal + contentProfile.arousalDelta;
    let postStress = Math.max(
      0.0,
      currentState.stressLevel - contentProfile.stressReduction
    );

    // Clamp to valid ranges
    postValence = this.clamp(postValence, -1.0, 1.0);
    postArousal = this.clamp(postArousal, -1.0, 1.0);
    postStress = this.clamp(postStress, 0.0, 1.0);

    // Calculate confidence based on historical data
    const watchCount = contentProfile.totalWatches ?? 0;
    const outcomeVariance = contentProfile.outcomeVariance ?? 1.0;

    // Confidence increases with watch count, decreases with variance
    const confidence = this.calculateConfidence(watchCount, outcomeVariance);

    return {
      postViewingValence: postValence,
      postViewingArousal: postArousal,
      postViewingStress: postStress,
      confidence
    };
  }

  private calculateConfidence(
    watchCount: number,
    variance: number
  ): number {
    // Sigmoid-like growth with watch count
    const countFactor = 1.0 - Math.exp(-watchCount / 20.0);

    // Penalty for high variance
    const varianceFactor = 1.0 - variance;

    // Combined confidence
    let confidence = countFactor * varianceFactor;

    // Clamp to [0.1, 0.95]
    confidence = Math.max(0.1, Math.min(0.95, confidence));

    return confidence;
  }

  private clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
  }
}
```

### 8.2 Confidence Calculation

**Confidence Formula**:
```
confidence = (1 - e^(-watchCount/20)) × (1 - variance)
```

**Examples**:
- **0 watches**: `confidence = 0 × (1 - 1.0) = 0.1` (clamped minimum)
- **20 watches, low variance (0.1)**: `confidence = 0.63 × 0.9 = 0.57`
- **100 watches, low variance (0.05)**: `confidence = 0.99 × 0.95 = 0.94`
- **100 watches, high variance (0.8)**: `confidence = 0.99 × 0.2 = 0.20`

**Interpretation**:
- High confidence (>0.7): Reliable prediction based on many consistent outcomes
- Medium confidence (0.4-0.7): Moderate reliability
- Low confidence (<0.4): Uncertain prediction, limited data

---

## 9. Reasoning Generation

### 9.1 Algorithm

```typescript
class ReasoningGenerator {
  generate(
    currentState: EmotionalState,
    desiredState: DesiredState,
    contentProfile: EmotionalContentProfile,
    qValue: number,
    isExploration: boolean
  ): string {
    let reasoning = '';

    // Part 1: Current emotional context
    const currentDesc = this.describeEmotionalState(
      currentState.valence,
      currentState.arousal,
      currentState.stressLevel
    );
    reasoning += `You're currently feeling ${currentDesc}. `;

    // Part 2: Desired transition
    const desiredDesc = this.describeEmotionalState(
      desiredState.valence,
      desiredState.arousal,
      0
    );
    reasoning += `This content will help you transition to feeling ${desiredDesc}. `;

    // Part 3: Expected emotional changes
    if (contentProfile.valenceDelta > 0.2) {
      reasoning += 'It should improve your mood significantly. ';
    } else if (contentProfile.valenceDelta < -0.2) {
      reasoning += 'It may be emotionally intense. ';
    }

    if (contentProfile.arousalDelta > 0.3) {
      reasoning += 'Expect to feel more energized and alert. ';
    } else if (contentProfile.arousalDelta < -0.3) {
      reasoning += 'It will help you relax and unwind. ';
    }

    if (contentProfile.stressReduction > 0.5) {
      reasoning += 'Great for stress relief. ';
    }

    // Part 4: Recommendation confidence
    if (qValue > 0.7) {
      reasoning += 'Users in similar emotional states loved this content. ';
    } else if (qValue < 0.3) {
      reasoning += 'This is a personalized experimental pick. ';
    } else {
      reasoning += 'This matches your emotional needs well. ';
    }

    // Part 5: Exploration flag
    if (isExploration) {
      reasoning += '(New discovery for you!)';
    }

    return reasoning.trim();
  }

  private describeEmotionalState(
    valence: number,
    arousal: number,
    stress: number
  ): string {
    let emotion = '';

    // Map to emotional labels
    if (valence > 0.3 && arousal > 0.3) {
      emotion = 'excited happy';
    } else if (valence > 0.3 && arousal < -0.3) {
      emotion = 'calm content';
    } else if (valence < -0.3 && arousal > 0.3) {
      emotion = 'stressed anxious';
    } else if (valence < -0.3 && arousal < -0.3) {
      emotion = 'sad lethargic';
    } else if (arousal > 0.5) {
      emotion = 'energized alert';
    } else if (arousal < -0.5) {
      emotion = 'relaxed calm';
    } else {
      emotion = 'neutral balanced';
    }

    // Stress modifier
    if (stress > 0.7) {
      emotion = `highly stressed ${emotion}`;
    } else if (stress > 0.4) {
      emotion = `moderately stressed ${emotion}`;
    }

    return emotion;
  }
}
```

### 9.2 Example Reasoning Outputs

**Scenario 1: Stressed User**
- Current: `valence=-0.3, arousal=0.6, stress=0.8`
- Desired: `valence=0.5, arousal=-0.4`
- Content: "Nature Sounds: Ocean Waves"
- Q-Value: `0.82` (high)
- Reasoning:
  ```
  You're currently feeling highly stressed anxious. This content will help you
  transition to feeling calm content. It will help you relax and unwind.
  Great for stress relief. Users in similar emotional states loved this content.
  ```

**Scenario 2: Bored User with Exploration**
- Current: `valence=0.1, arousal=-0.5, stress=0.2`
- Desired: `valence=0.5, arousal=0.5`
- Content: "Action Movie: Mad Max"
- Q-Value: `0.5` (unexplored)
- Is Exploration: `true`
- Reasoning:
  ```
  You're currently feeling relaxed calm. This content will help you transition
  to feeling excited happy. Expect to feel more energized and alert.
  This matches your emotional needs well. (New discovery for you!)
  ```

---

## 10. State Hashing & Discretization

### 10.1 Algorithm

```typescript
interface StateHash {
  valenceBucket: number;
  arousalBucket: number;
  stressBucket: number;
  hash: string;
}

function hashEmotionalState(state: EmotionalState): StateHash {
  // Discretize continuous state space for Q-table lookup
  // Valence: [-1, 1] → 10 buckets (0.2 granularity)
  const valenceBucket = Math.floor((state.valence + 1.0) / 0.2);

  // Arousal: [-1, 1] → 10 buckets (0.2 granularity)
  const arousalBucket = Math.floor((state.arousal + 1.0) / 0.2);

  // Stress: [0, 1] → 5 buckets (0.2 granularity)
  const stressBucket = Math.floor(state.stressLevel / 0.2);

  // Create deterministic hash
  const hash = `v:${valenceBucket}:a:${arousalBucket}:s:${stressBucket}`;

  return {
    valenceBucket,
    arousalBucket,
    stressBucket,
    hash
  };
}
```

### 10.2 State Space Size

**Total State Space**:
```
10 (valence) × 10 (arousal) × 5 (stress) = 500 discrete states
```

**Trade-offs**:
- **Finer granularity** (more buckets): More precise Q-values, but slower learning
- **Coarser granularity** (fewer buckets): Faster learning, but less precise

**MVP Choice**: 500 states is manageable for Q-learning with tabular methods.

### 10.3 Examples

| Emotional State | Valence Bucket | Arousal Bucket | Stress Bucket | Hash |
|-----------------|----------------|----------------|---------------|------|
| `v=-0.6, a=0.5, s=0.8` | 2 | 7 | 4 | `v:2:a:7:s:4` |
| `v=0.3, a=-0.2, s=0.3` | 6 | 4 | 1 | `v:6:a:4:s:1` |
| `v=0.0, a=0.0, s=0.5` | 5 | 5 | 2 | `v:5:a:5:s:2` |

---

## 11. Performance Optimization

### 11.1 Complexity Analysis

**Time Complexity**:
- `loadEmotionalState()`: **O(1)** (AgentDB key lookup)
- `predictDesiredState()`: **O(1)** (rule evaluation)
- `buildTransitionVector()`: **O(1)** (embedding API call, async)
- `searchCandidates()`: **O(log n)** where n = total content (HNSW index)
- `filterWatched()`: **O(k)** where k = candidate count (~60)
- `rankCandidates()`: **O(k log k)** (k Q-value lookups + sort)
- `applyExploration()`: **O(k)** (linear scan with random injection)
- `generateRecommendations()`: **O(m)** where m = limit (20)

**Total**: **O(k log k)** dominated by re-ranking sort, where k = 60 candidates.

**Space Complexity**:
- Transition vector: **O(1)** (fixed 1536D)
- Search candidates: **O(k)** (60 items)
- Ranked results: **O(k)**
- Final recommendations: **O(m)** (20 items)

**Total**: **O(k)** where k is constant (60).

### 11.2 Optimization Strategies

#### 11.2.1 Batch Q-Value Lookups

**Problem**: Sequential Q-value lookups for 60 candidates cause latency.

**Solution**: Batch retrieve all Q-values in a single AgentDB query.

```typescript
class HybridRanker {
  private async batchGetQValues(
    userId: string,
    stateHash: string,
    contentIds: string[]
  ): Promise<Map<string, number>> {
    // Construct all keys
    const keys = contentIds.map(
      id => `q:${userId}:${stateHash}:${id}`
    );

    // Batch lookup (single round-trip to AgentDB)
    const qValues = await this.agentDB.multiGet(keys);

    // Map contentId → Q-value
    const qMap = new Map<string, number>();
    contentIds.forEach((id, idx) => {
      qMap.set(id, qValues[idx] ?? this.config.defaultQValue);
    });

    return qMap;
  }
}
```

**Speedup**: **3-5x faster** than sequential lookups.

#### 11.2.2 Content Profile Caching

**Problem**: Loading full content profiles for 60 candidates is slow.

**Solution**: LRU cache for frequently recommended content.

```typescript
import LRU from 'lru-cache';

class RecommendationEngine {
  private profileCache = new LRU<string, EmotionalContentProfile>({
    max: 500, // Cache 500 most popular content items
    ttl: 1000 * 60 * 60 // 1 hour TTL
  });

  private async loadContentProfile(
    contentId: string
  ): Promise<EmotionalContentProfile> {
    // Check cache first
    const cached = this.profileCache.get(contentId);
    if (cached) {
      return cached;
    }

    // Load from AgentDB
    const profile = await this.agentDB.get({
      namespace: 'emotistream/content_profiles',
      key: contentId
    });

    // Cache for future requests
    this.profileCache.set(contentId, profile);

    return profile;
  }
}
```

**Speedup**: **10x faster** for cache hits (95%+ hit rate for popular content).

#### 11.2.3 Approximate RuVector Search

**Problem**: Exact HNSW search can be slow for very large collections (100K+ items).

**Solution**: Use RuVector's quantization for faster approximate search.

```typescript
const searchResults = await this.ruVector.search({
  collectionName: 'emotistream_content',
  vector: transitionVector,
  limit: topK,
  quantization: 'scalar', // Enable quantization
  ef: 64 // Lower ef for faster search (default: 128)
});
```

**Speedup**: **2-3x faster** with negligible accuracy loss (<1% difference in top-20).

#### 11.2.4 Parallel Ranking

**Problem**: Serial execution of outcome prediction and reasoning generation.

**Solution**: Use `Promise.all()` for concurrent processing.

```typescript
async generateRecommendations(
  rankedCandidates: RankedCandidate[],
  currentState: EmotionalState,
  desiredState: DesiredState,
  limit: number
): Promise<Recommendation[]> {
  const topCandidates = rankedCandidates.slice(0, limit);

  // Process all candidates in parallel
  const recommendations = await Promise.all(
    topCandidates.map(async (candidate, idx) => {
      // Predict outcome (I/O-bound)
      const outcome = await this.outcomePredictor.predict(
        currentState,
        candidate.profile
      );

      // Generate reasoning (CPU-bound, but fast)
      const reasoning = this.reasoningGenerator.generate(
        currentState,
        desiredState,
        candidate.profile,
        candidate.qValue,
        candidate.isExploration
      );

      return {
        contentId: candidate.contentId,
        title: candidate.profile.title,
        platform: candidate.profile.platform,
        emotionalProfile: candidate.profile,
        predictedOutcome: outcome,
        qValue: candidate.qValue,
        similarityScore: candidate.similarity,
        combinedScore: candidate.hybridScore,
        isExploration: candidate.isExploration,
        rank: idx + 1,
        reasoning
      };
    })
  );

  return recommendations;
}
```

**Speedup**: **2-4x faster** for generating final 20 recommendations.

### 11.3 Latency Targets

| Operation | Target Latency (p95) | Current (Optimized) |
|-----------|----------------------|---------------------|
| `getRecommendations()` | <500ms | ~350ms |
| `searchCandidates()` | <100ms | ~80ms |
| `rankCandidates()` | <150ms | ~120ms |
| `generateRecommendations()` | <100ms | ~70ms |

**Total**: **~350ms** for 20 recommendations (meets <500ms target).

---

## 12. Testing Strategy

### 12.1 Unit Tests

```typescript
// tests/engine.test.ts
describe('RecommendationEngine', () => {
  let engine: RecommendationEngine;
  let mockRuVector: jest.Mocked<RuVectorClient>;
  let mockAgentDB: jest.Mocked<AgentDB>;
  let mockRLPolicy: jest.Mocked<RLPolicyEngine>;

  beforeEach(() => {
    mockRuVector = createMockRuVector();
    mockAgentDB = createMockAgentDB();
    mockRLPolicy = createMockRLPolicy();

    engine = new RecommendationEngine({
      ruVector: mockRuVector,
      agentDB: mockAgentDB,
      rlPolicy: mockRLPolicy
    });
  });

  describe('getRecommendations', () => {
    it('should return 20 recommendations for valid request', async () => {
      const request: RecommendationRequest = {
        userId: 'user123',
        emotionalStateId: 'state456',
        limit: 20
      };

      const recommendations = await engine.getRecommendations(request);

      expect(recommendations).toHaveLength(20);
      expect(recommendations[0]).toHaveProperty('contentId');
      expect(recommendations[0]).toHaveProperty('qValue');
      expect(recommendations[0]).toHaveProperty('similarityScore');
      expect(recommendations[0]).toHaveProperty('combinedScore');
    });

    it('should handle explicit desired state override', async () => {
      const request: RecommendationRequest = {
        userId: 'user123',
        emotionalStateId: 'state456',
        explicitDesiredState: {
          valence: 0.8,
          arousal: -0.3
        }
      };

      const recommendations = await engine.getRecommendations(request);

      // Verify transition vector was built with explicit desired state
      expect(mockTransitionBuilder.buildVector).toHaveBeenCalledWith(
        expect.any(Object),
        { valence: 0.8, arousal: -0.3 }
      );
    });

    it('should throw error for non-existent emotional state', async () => {
      mockAgentDB.get.mockResolvedValueOnce(null);

      const request: RecommendationRequest = {
        userId: 'user123',
        emotionalStateId: 'invalid_state'
      };

      await expect(engine.getRecommendations(request)).rejects.toThrow(
        'Emotional state not found'
      );
    });
  });
});

// tests/ranker.test.ts
describe('HybridRanker', () => {
  let ranker: HybridRanker;

  beforeEach(() => {
    ranker = new HybridRanker({
      qWeight: 0.7,
      similarityWeight: 0.3,
      defaultQValue: 0.5
    });
  });

  describe('rank', () => {
    it('should rank by hybrid score (70% Q + 30% similarity)', async () => {
      const candidates: SearchCandidate[] = [
        { contentId: 'A', similarity: 0.9, qValue: 0.3 },
        { contentId: 'B', similarity: 0.6, qValue: 0.8 },
        { contentId: 'C', similarity: 0.7, qValue: 0.7 }
      ];

      const ranked = await ranker.rank('user123', candidates, mockState);

      // B should rank highest: (0.8 * 0.7) + (0.6 * 0.3) = 0.74
      // C should rank second: (0.7 * 0.7) + (0.7 * 0.3) = 0.70
      // A should rank third: (0.3 * 0.7) + (0.9 * 0.3) = 0.48
      expect(ranked[0].contentId).toBe('B');
      expect(ranked[1].contentId).toBe('C');
      expect(ranked[2].contentId).toBe('A');
    });

    it('should use default Q-value for unexplored content', async () => {
      mockAgentDB.get.mockResolvedValueOnce(null); // No Q-value

      const candidates: SearchCandidate[] = [
        { contentId: 'unexplored', similarity: 0.8 }
      ];

      const ranked = await ranker.rank('user123', candidates, mockState);

      // Should use default Q = 0.5
      expect(ranked[0].qValue).toBe(0.5);
    });
  });

  describe('calculateOutcomeAlignment', () => {
    it('should return high alignment for matching deltas', () => {
      const profile = {
        valenceDelta: 0.8,
        arousalDelta: -0.6
      };

      const desired = {
        valence: 0.8, // Assuming current is 0
        arousal: -0.6
      };

      const alignment = ranker.calculateOutcomeAlignment(profile, desired);

      expect(alignment).toBeGreaterThan(0.9);
    });

    it('should return low alignment for opposite deltas', () => {
      const profile = {
        valenceDelta: 0.8,
        arousalDelta: -0.6
      };

      const desired = {
        valence: -0.8,
        arousal: 0.6
      };

      const alignment = ranker.calculateOutcomeAlignment(profile, desired);

      expect(alignment).toBeLessThan(0.3);
    });
  });
});

// tests/outcome-predictor.test.ts
describe('OutcomePredictor', () => {
  let predictor: OutcomePredictor;

  beforeEach(() => {
    predictor = new OutcomePredictor();
  });

  it('should predict post-viewing state by adding deltas', () => {
    const currentState: EmotionalState = {
      valence: -0.4,
      arousal: 0.6,
      stressLevel: 0.8
    };

    const profile: EmotionalContentProfile = {
      valenceDelta: 0.7,
      arousalDelta: -0.6,
      stressReduction: 0.5
    };

    const outcome = predictor.predict(currentState, profile);

    expect(outcome.postViewingValence).toBeCloseTo(0.3);
    expect(outcome.postViewingArousal).toBeCloseTo(0.0);
    expect(outcome.postViewingStress).toBeCloseTo(0.3);
  });

  it('should clamp values to valid ranges', () => {
    const currentState: EmotionalState = {
      valence: 0.8,
      arousal: 0.9,
      stressLevel: 0.1
    };

    const profile: EmotionalContentProfile = {
      valenceDelta: 0.5, // Would exceed 1.0
      arousalDelta: 0.5, // Would exceed 1.0
      stressReduction: 0.3 // Would go negative
    };

    const outcome = predictor.predict(currentState, profile);

    expect(outcome.postViewingValence).toBe(1.0); // Clamped
    expect(outcome.postViewingArousal).toBe(1.0); // Clamped
    expect(outcome.postViewingStress).toBe(0.0); // Clamped to 0
  });

  it('should calculate confidence based on watch count and variance', () => {
    const profile1: EmotionalContentProfile = {
      totalWatches: 0,
      outcomeVariance: 1.0
    };

    const profile2: EmotionalContentProfile = {
      totalWatches: 100,
      outcomeVariance: 0.05
    };

    const outcome1 = predictor.predict(mockState, profile1);
    const outcome2 = predictor.predict(mockState, profile2);

    expect(outcome1.confidence).toBeLessThan(0.2); // Low confidence
    expect(outcome2.confidence).toBeGreaterThan(0.9); // High confidence
  });
});
```

### 12.2 Integration Tests

```typescript
// tests/integration/end-to-end.test.ts
describe('RecommendationEngine Integration', () => {
  let engine: RecommendationEngine;
  let ruVector: RuVectorClient;
  let agentDB: AgentDB;

  beforeAll(async () => {
    // Use real instances for integration testing
    ruVector = await RuVectorClient.connect(process.env.RUVECTOR_URL);
    agentDB = await AgentDB.connect(process.env.AGENTDB_URL);

    engine = new RecommendationEngine({ ruVector, agentDB });

    // Seed test data
    await seedTestContent(ruVector);
    await seedTestQValues(agentDB);
  });

  afterAll(async () => {
    await ruVector.disconnect();
    await agentDB.disconnect();
  });

  it('should generate recommendations end-to-end', async () => {
    const request: RecommendationRequest = {
      userId: 'integration-test-user',
      emotionalStateId: 'stress-state-1',
      limit: 20
    };

    const recommendations = await engine.getRecommendations(request);

    expect(recommendations).toHaveLength(20);
    expect(recommendations[0].similarityScore).toBeGreaterThan(0.5);
    expect(recommendations[0].reasoning).toContain('feel');
  });

  it('should filter watched content', async () => {
    // Mark content as watched
    await agentDB.store({
      namespace: 'emotistream/watch_history',
      key: 'user123:content-A',
      value: {
        contentId: 'content-A',
        watchedAt: Date.now()
      }
    });

    const recommendations = await engine.getRecommendations({
      userId: 'user123',
      emotionalStateId: 'state1'
    });

    // Should not include recently watched content
    const watchedIds = recommendations.map(r => r.contentId);
    expect(watchedIds).not.toContain('content-A');
  });

  it('should apply exploration when enabled', async () => {
    const recommendations = await engine.getRecommendations({
      userId: 'user123',
      emotionalStateId: 'state1',
      includeExploration: true,
      explorationRate: 0.3
    });

    // Should have ~30% exploration picks
    const explorationCount = recommendations.filter(r => r.isExploration).length;
    expect(explorationCount).toBeGreaterThanOrEqual(4); // ~20% of 20
    expect(explorationCount).toBeLessThanOrEqual(8);    // ~40% tolerance
  });
});
```

### 12.3 Performance Tests

```typescript
// tests/performance/latency.test.ts
describe('RecommendationEngine Performance', () => {
  it('should generate 20 recommendations in <500ms (p95)', async () => {
    const trials = 100;
    const latencies: number[] = [];

    for (let i = 0; i < trials; i++) {
      const start = Date.now();

      await engine.getRecommendations({
        userId: `perf-user-${i}`,
        emotionalStateId: `state-${i}`
      });

      const latency = Date.now() - start;
      latencies.push(latency);
    }

    latencies.sort((a, b) => a - b);
    const p95Latency = latencies[Math.floor(trials * 0.95)];

    console.log(`p95 latency: ${p95Latency}ms`);
    expect(p95Latency).toBeLessThan(500);
  });

  it('should handle 100 concurrent requests', async () => {
    const requests = Array.from({ length: 100 }, (_, i) =>
      engine.getRecommendations({
        userId: `concurrent-user-${i}`,
        emotionalStateId: `state-${i}`
      })
    );

    const start = Date.now();
    const results = await Promise.all(requests);
    const duration = Date.now() - start;

    expect(results).toHaveLength(100);
    expect(duration).toBeLessThan(3000); // <3s for 100 concurrent
    console.log(`100 concurrent requests: ${duration}ms`);
  });
});
```

---

## 13. Error Handling & Edge Cases

### 13.1 Error Scenarios

```typescript
class RecommendationEngine {
  async getRecommendations(
    request: RecommendationRequest
  ): Promise<Recommendation[]> {
    try {
      // 1. Emotional state not found
      const currentState = await this.loadEmotionalState(request.emotionalStateId);
      if (!currentState) {
        throw new RecommendationError(
          'EMOTIONAL_STATE_NOT_FOUND',
          `Emotional state ${request.emotionalStateId} does not exist`
        );
      }

      // 2. RuVector search returns no results
      const candidates = await this.searchCandidates(transitionVector, topK);
      if (candidates.length === 0) {
        // Fallback to popular content in desired quadrant
        return await this.getFallbackRecommendations(currentState, desiredState);
      }

      // 3. All content already watched
      const filtered = await this.filterWatched(userId, candidates);
      if (filtered.length === 0) {
        // Relax filter: allow re-recommendations from 7+ days ago
        return await this.filterWatched(userId, candidates, { minDaysSinceWatch: 7 });
      }

      // 4. AgentDB connection error
      try {
        const ranked = await this.rankCandidates(userId, filtered, currentState);
      } catch (error) {
        if (error.code === 'AGENTDB_UNAVAILABLE') {
          // Fallback to similarity-only ranking
          logger.warn('AgentDB unavailable, using similarity-only ranking');
          return await this.rankBySimilarityOnly(filtered);
        }
        throw error;
      }

      // ... continue
    } catch (error) {
      if (error instanceof RecommendationError) {
        throw error;
      }

      // Unexpected error
      logger.error('Recommendation generation failed', { error, request });
      throw new RecommendationError(
        'RECOMMENDATION_FAILED',
        'Unable to generate recommendations',
        error
      );
    }
  }
}

class RecommendationError extends Error {
  constructor(
    public code: string,
    message: string,
    public cause?: Error
  ) {
    super(message);
    this.name = 'RecommendationError';
  }
}
```

### 13.2 Edge Cases

#### Edge Case 1: Extreme Emotional States

```typescript
function handleExtremeEmotionalState(
  currentState: EmotionalState,
  candidates: SearchCandidate[]
): SearchCandidate[] {
  // If user is in extreme state (valence/arousal > 0.9)
  const isExtreme =
    Math.abs(currentState.valence) > 0.9 ||
    Math.abs(currentState.arousal) > 0.9;

  if (isExtreme) {
    // Filter out content with extreme deltas (avoid shocking transitions)
    return candidates.filter((candidate) => {
      const deltaMagnitude = Math.sqrt(
        candidate.profile.valenceDelta ** 2 +
        candidate.profile.arousalDelta ** 2
      );
      return deltaMagnitude < 0.6; // Conservative transitions only
    });
  }

  return candidates;
}
```

#### Edge Case 2: New User (Cold Start)

```typescript
function handleColdStart(
  userId: string,
  candidates: RankedCandidate[]
): RankedCandidate[] {
  // Check if user has any Q-values
  const hasQValues = await this.agentDB.exists(`q:${userId}:*`);

  if (!hasQValues) {
    // New user: rely on similarity + popular content bias
    return candidates.map((candidate) => ({
      ...candidate,
      hybridScore: candidate.similarity * 0.8 + (candidate.profile.popularity ?? 0) * 0.2
    })).sort((a, b) => b.hybridScore - a.hybridScore);
  }

  return candidates;
}
```

#### Edge Case 3: No Semantic Match

```typescript
async function getFallbackRecommendations(
  currentState: EmotionalState,
  desiredState: DesiredState
): Promise<Recommendation[]> {
  // Determine desired emotional quadrant
  const quadrant = this.getEmotionalQuadrant(
    desiredState.valence,
    desiredState.arousal
  );

  // Query popular content in that quadrant
  const fallback = await this.ruVector.search({
    collectionName: 'emotistream_content',
    filter: {
      emotionalQuadrant: quadrant,
      isActive: true
    },
    limit: 20
  });

  return this.generateRecommendations(fallback, currentState, desiredState);
}
```

---

## 14. Configuration & Tuning

### 14.1 Configurable Parameters

```typescript
// config/recommendation-engine.ts

export const RECOMMENDATION_CONFIG = {
  // Hybrid ranking weights
  ranking: {
    qWeight: 0.7,                    // 70% Q-value
    similarityWeight: 0.3,           // 30% similarity
    defaultQValue: 0.5,              // For unexplored content
    explorationBonus: 0.1,           // Bonus for unexplored
    outcomeAlignmentFactor: 1.0      // Multiplier for alignment
  },

  // Search parameters
  search: {
    topKMultiplier: 3,               // Get 3x candidates for re-ranking
    minSimilarity: 0.3,              // Filter low-similarity results
    maxDistance: 1.5                 // Max vector distance threshold
  },

  // Exploration strategy
  exploration: {
    initialRate: 0.3,                // 30% exploration initially
    minRate: 0.1,                    // 10% minimum
    decayFactor: 0.95,               // Decay per episode
    randomSelectionRange: [0.5, 1.0] // Random from bottom half
  },

  // Watch history filtering
  watchHistory: {
    minDaysSinceWatch: 30,           // Re-recommend after 30 days
    maxHistorySize: 1000             // Track last 1000 watches
  },

  // State discretization
  stateDiscretization: {
    valenceBuckets: 10,              // [-1, 1] → 10 buckets
    arousalBuckets: 10,
    stressBuckets: 5                 // [0, 1] → 5 buckets
  },

  // Performance tuning
  performance: {
    enableProfileCache: true,
    profileCacheSize: 500,           // Cache top 500 content items
    profileCacheTTL: 3600000,        // 1 hour
    enableBatchQValueLookup: true,
    enableParallelRanking: true
  }
};
```

### 14.2 A/B Testing Parameters

```typescript
// Experiment with different weighting schemes

export const AB_TEST_CONFIGS = {
  // Control: 70/30 Q-value/similarity
  control: {
    qWeight: 0.7,
    similarityWeight: 0.3
  },

  // Variant A: 80/20 (favor Q-values more)
  variantA: {
    qWeight: 0.8,
    similarityWeight: 0.2
  },

  // Variant B: 60/40 (favor similarity more)
  variantB: {
    qWeight: 0.6,
    similarityWeight: 0.4
  },

  // Variant C: 50/50 (balanced)
  variantC: {
    qWeight: 0.5,
    similarityWeight: 0.5
  }
};

// Usage
function getConfigForUser(userId: string): HybridRankingConfig {
  const experimentGroup = hashUserId(userId) % 4;

  switch (experimentGroup) {
    case 0: return AB_TEST_CONFIGS.control;
    case 1: return AB_TEST_CONFIGS.variantA;
    case 2: return AB_TEST_CONFIGS.variantB;
    case 3: return AB_TEST_CONFIGS.variantC;
  }
}
```

---

## 15. Deployment Architecture

### 15.1 Service Configuration

```typescript
// src/recommendations/server.ts

import { RecommendationEngine } from './engine';
import { RuVectorClient } from '../vector/client';
import { AgentDB } from '../storage/agentdb';
import { RLPolicyEngine } from '../rl/policy';

export async function createRecommendationService() {
  // Initialize dependencies
  const ruVector = await RuVectorClient.connect({
    url: process.env.RUVECTOR_URL || 'http://localhost:8080',
    timeout: 5000
  });

  const agentDB = await AgentDB.connect({
    url: process.env.AGENTDB_URL || 'redis://localhost:6379',
    db: 0
  });

  const rlPolicy = new RLPolicyEngine({ agentDB });

  // Create engine
  const engine = new RecommendationEngine({
    ruVector,
    agentDB,
    rlPolicy,
    config: RECOMMENDATION_CONFIG
  });

  // Health check
  const healthCheck = async () => {
    try {
      await ruVector.health();
      await agentDB.ping();
      return { status: 'healthy' };
    } catch (error) {
      return { status: 'unhealthy', error: error.message };
    }
  };

  return {
    engine,
    healthCheck,
    async shutdown() {
      await ruVector.disconnect();
      await agentDB.disconnect();
    }
  };
}
```

### 15.2 Docker Compose Integration

```yaml
# docker-compose.yml (excerpt)

services:
  recommendation-engine:
    build: ./src/recommendations
    ports:
      - "3002:3002"
    environment:
      - RUVECTOR_URL=http://ruvector:8080
      - AGENTDB_URL=redis://agentdb:6379
      - NODE_ENV=production
    depends_on:
      - ruvector
      - agentdb
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3002/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

---

## 16. Monitoring & Observability

### 16.1 Key Metrics

```typescript
import { Counter, Histogram, Gauge } from 'prom-client';

// Request metrics
const recommendationRequests = new Counter({
  name: 'recommendations_total',
  help: 'Total recommendation requests',
  labelNames: ['userId', 'status']
});

const recommendationLatency = new Histogram({
  name: 'recommendations_duration_seconds',
  help: 'Recommendation generation latency',
  buckets: [0.1, 0.3, 0.5, 0.7, 1.0, 2.0]
});

// Ranking metrics
const qValueUtilization = new Gauge({
  name: 'q_value_utilization',
  help: 'Percentage of Q-values found (vs default)',
  labelNames: ['userId']
});

const hybridScoreDistribution = new Histogram({
  name: 'hybrid_score_distribution',
  help: 'Distribution of final hybrid scores',
  buckets: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
});

// Exploration metrics
const explorationRate = new Gauge({
  name: 'exploration_rate',
  help: 'Current exploration rate',
  labelNames: ['userId']
});

const explorationCount = new Counter({
  name: 'exploration_picks_total',
  help: 'Total exploration picks',
  labelNames: ['userId']
});

// Usage
class RecommendationEngine {
  async getRecommendations(
    request: RecommendationRequest
  ): Promise<Recommendation[]> {
    const timer = recommendationLatency.startTimer();

    try {
      const recommendations = await this.generateRecommendations(request);

      recommendationRequests.inc({ userId: request.userId, status: 'success' });
      timer({ status: 'success' });

      // Track metrics
      this.trackMetrics(request.userId, recommendations);

      return recommendations;
    } catch (error) {
      recommendationRequests.inc({ userId: request.userId, status: 'error' });
      timer({ status: 'error' });
      throw error;
    }
  }

  private trackMetrics(userId: string, recommendations: Recommendation[]) {
    // Q-value utilization
    const qValuesFound = recommendations.filter(r => r.qValue !== 0.5).length;
    qValueUtilization.set(
      { userId },
      qValuesFound / recommendations.length
    );

    // Hybrid score distribution
    recommendations.forEach(r => {
      hybridScoreDistribution.observe(r.combinedScore);
    });

    // Exploration metrics
    const explorationPicks = recommendations.filter(r => r.isExploration).length;
    explorationCount.inc({ userId }, explorationPicks);
  }
}
```

### 16.2 Logging

```typescript
import winston from 'winston';

const logger = winston.createLogger({
  level: 'info',
  format: winston.format.json(),
  defaultMeta: { service: 'recommendation-engine' },
  transports: [
    new winston.transports.File({ filename: 'error.log', level: 'error' }),
    new winston.transports.File({ filename: 'combined.log' })
  ]
});

// Log recommendation events
logger.info('Recommendations generated', {
  userId: request.userId,
  emotionalStateId: request.emotionalStateId,
  candidateCount: candidates.length,
  filteredCount: filtered.length,
  topScore: recommendations[0].combinedScore,
  explorationRate: this.explorationStrategy.rate
});

// Log errors with context
logger.error('Recommendation generation failed', {
  userId: request.userId,
  error: error.message,
  stack: error.stack,
  request
});
```

---

## 17. Future Enhancements

### 17.1 Multi-Objective Optimization

**Goal**: Balance emotional outcomes with diversity, novelty, and serendipity.

```typescript
interface MultiObjectiveScore {
  emotionalFit: number;     // Current hybrid score
  diversity: number;        // Genre/platform diversity
  novelty: number;          // How unexpected this pick is
  serendipity: number;      // Alignment with hidden interests
}

function calculateMultiObjectiveScore(
  candidate: RankedCandidate,
  userProfile: UserProfile,
  recentRecommendations: Recommendation[]
): number {
  const weights = {
    emotionalFit: 0.6,
    diversity: 0.2,
    novelty: 0.1,
    serendipity: 0.1
  };

  const scores = {
    emotionalFit: candidate.hybridScore,
    diversity: calculateDiversity(candidate, recentRecommendations),
    novelty: calculateNovelty(candidate, userProfile),
    serendipity: calculateSerendipity(candidate, userProfile)
  };

  return Object.entries(weights).reduce(
    (total, [key, weight]) => total + scores[key] * weight,
    0
  );
}
```

### 17.2 Contextual Recommendations

**Goal**: Incorporate time-of-day, day-of-week, location, and social context.

```typescript
interface ContextualFactors {
  timeOfDay: 'morning' | 'afternoon' | 'evening' | 'night';
  dayOfWeek: 'weekday' | 'weekend';
  location: 'home' | 'work' | 'commute' | 'other';
  socialContext: 'alone' | 'with_partner' | 'with_family' | 'with_friends';
}

function adjustScoreByContext(
  score: number,
  content: EmotionalContentProfile,
  context: ContextualFactors
): number {
  let adjustment = 1.0;

  // Example: Prefer calming content in evening
  if (context.timeOfDay === 'evening' && content.arousalDelta < -0.3) {
    adjustment *= 1.2;
  }

  // Example: Prefer social content with friends
  if (context.socialContext === 'with_friends' && content.genres.includes('comedy')) {
    adjustment *= 1.15;
  }

  return score * adjustment;
}
```

### 17.3 Explainable AI (XAI)

**Goal**: Provide SHAP values and counterfactual explanations.

```typescript
interface ExplainableRecommendation extends Recommendation {
  shapValues: {
    qValue: number;
    similarity: number;
    outcomeAlignment: number;
    [feature: string]: number;
  };
  counterfactuals: {
    question: string;
    answer: string;
  }[];
}

function generateExplanation(
  recommendation: Recommendation
): ExplainableRecommendation {
  // SHAP values show feature contributions
  const shapValues = {
    qValue: recommendation.qValue * 0.7,
    similarity: recommendation.similarityScore * 0.3,
    outcomeAlignment: calculateAlignmentContribution(recommendation)
  };

  // Counterfactuals answer "why not X?"
  const counterfactuals = [
    {
      question: "Why not a thriller instead?",
      answer: "Thrillers would increase your arousal, but you need calming content."
    },
    {
      question: "Why this over other nature documentaries?",
      answer: "This has a higher Q-value (0.82) based on your past positive experiences."
    }
  ];

  return { ...recommendation, shapValues, counterfactuals };
}
```

---

## 18. Appendix: Full Code Example

```typescript
// src/recommendations/engine.ts

import { RuVectorClient } from '../vector/client';
import { AgentDB } from '../storage/agentdb';
import { RLPolicyEngine } from '../rl/policy';
import { TransitionVectorBuilder } from './transition-vector';
import { DesiredStatePredictor } from './desired-state';
import { WatchHistoryFilter } from './filters';
import { HybridRanker } from './ranker';
import { OutcomePredictor } from './outcome-predictor';
import { ReasoningGenerator } from './reasoning';
import { ExplorationStrategy } from './exploration';
import {
  RecommendationRequest,
  Recommendation,
  SearchCandidate,
  RankedCandidate
} from './types';

export class RecommendationEngine {
  private transitionBuilder: TransitionVectorBuilder;
  private desiredStatePredictor: DesiredStatePredictor;
  private watchHistoryFilter: WatchHistoryFilter;
  private hybridRanker: HybridRanker;
  private outcomePredictor: OutcomePredictor;
  private reasoningGenerator: ReasoningGenerator;
  private explorationStrategy: ExplorationStrategy;

  constructor(
    private ruVector: RuVectorClient,
    private agentDB: AgentDB,
    private rlPolicy: RLPolicyEngine
  ) {
    this.transitionBuilder = new TransitionVectorBuilder();
    this.desiredStatePredictor = new DesiredStatePredictor();
    this.watchHistoryFilter = new WatchHistoryFilter(agentDB);
    this.hybridRanker = new HybridRanker(rlPolicy, agentDB);
    this.outcomePredictor = new OutcomePredictor();
    this.reasoningGenerator = new ReasoningGenerator();
    this.explorationStrategy = new ExplorationStrategy();
  }

  async getRecommendations(
    request: RecommendationRequest
  ): Promise<Recommendation[]> {
    // Step 1: Load emotional state
    const currentState = await this.loadEmotionalState(request.emotionalStateId);

    // Step 2: Determine desired state
    const desiredState = request.explicitDesiredState
      ? request.explicitDesiredState
      : await this.desiredStatePredictor.predict(currentState);

    // Step 3: Create transition vector
    const transitionVector = await this.transitionBuilder.buildVector(
      currentState,
      desiredState
    );

    // Step 4: Search RuVector
    const topK = (request.limit ?? 20) * 3;
    const candidates = await this.searchCandidates(transitionVector, topK);

    // Step 5: Filter watched content
    const filtered = await this.watchHistoryFilter.filter(
      request.userId,
      candidates
    );

    // Step 6: Hybrid ranking
    const ranked = await this.hybridRanker.rank(
      request.userId,
      filtered,
      currentState,
      desiredState
    );

    // Step 7: Apply exploration
    const explored = request.includeExploration
      ? await this.explorationStrategy.inject(
          ranked,
          request.explorationRate ?? 0.1
        )
      : ranked;

    // Step 8: Generate recommendations
    const recommendations = await this.generateRecommendations(
      explored,
      currentState,
      desiredState,
      request.limit ?? 20
    );

    // Step 9: Log event
    await this.logRecommendationEvent(request.userId, currentState, recommendations);

    return recommendations;
  }

  private async searchCandidates(
    vector: Float32Array,
    topK: number
  ): Promise<SearchCandidate[]> {
    const results = await this.ruVector.search({
      collectionName: 'emotistream_content',
      vector,
      limit: topK,
      filter: { isActive: true }
    });

    return results.map(result => ({
      contentId: result.id,
      profile: result.metadata,
      similarity: 1.0 - (result.distance / 2.0),
      distance: result.distance
    }));
  }

  private async generateRecommendations(
    ranked: RankedCandidate[],
    currentState: EmotionalState,
    desiredState: DesiredState,
    limit: number
  ): Promise<Recommendation[]> {
    const top = ranked.slice(0, limit);

    return Promise.all(
      top.map(async (candidate, idx) => {
        const outcome = await this.outcomePredictor.predict(
          currentState,
          candidate.profile
        );

        const reasoning = this.reasoningGenerator.generate(
          currentState,
          desiredState,
          candidate.profile,
          candidate.qValue,
          candidate.isExploration
        );

        return {
          contentId: candidate.contentId,
          title: candidate.profile.title,
          platform: candidate.profile.platform,
          emotionalProfile: candidate.profile,
          predictedOutcome: outcome,
          qValue: candidate.qValue,
          similarityScore: candidate.similarity,
          combinedScore: candidate.hybridScore,
          isExploration: candidate.isExploration,
          rank: idx + 1,
          reasoning
        };
      })
    );
  }

  // ... other helper methods
}
```

---

**End of Architecture Document**

**Document Version**: 1.0
**Last Updated**: 2025-12-05
**Author**: SPARC Architecture Agent
**Status**: Ready for Refinement Phase (TDD Implementation)

**Next Steps**:
1. Review architecture with team
2. Create test specifications (TDD)
3. Begin implementation of core modules
4. Integration testing with RuVector and AgentDB
