# Product Requirements Document: StreamSense AI

## 1. Executive Summary

**Problem**: Users spend an average of 45 minutes navigating 5+ streaming platforms before finding content, experiencing decision paralysis, subscription fatigue, and fragmented discovery across Netflix, Disney+, Amazon Prime, Apple TV+, and HBO Max.

**Solution**: StreamSense AI is an intent-driven unified discovery platform with self-learning preference models that understand natural language queries, learn from user behavior, and provide personalized recommendations across all platforms, reducing decision time by 94% (45 min → 2.5 min).

**Impact**: Leveraging RuVector's 150x faster semantic search, AgentDB's persistent learning memory, and Agentic Flow's specialized agents, StreamSense delivers continuously improving recommendations that adapt to user preferences, viewing context, and historical satisfaction patterns.

---

## 2. Problem Statement

### 2.1 Current State Analysis

**User Pain Points:**
- **45-minute average** decision time across multiple platforms
- **5.2 platform subscriptions** per household (average)
- **73% of users** report "choice paralysis" when selecting content
- **22 minutes wasted** on average starting content they don't finish
- **Zero learning** - recommendations don't improve based on actual viewing behavior

**Market Data:**
- $120B global streaming market (2024)
- 1.1B streaming subscribers worldwide
- 82% user satisfaction with content discovery rated "poor" or "fair"
- $2.4B annual cost of abandoned content (started but not finished)

**Technical Challenges:**
- No unified search across platforms
- Recommendations ignore cross-platform viewing history
- Static preference models (no learning from outcomes)
- Context-blind suggestions (time of day, mood, social setting ignored)

### 2.2 Root Cause Analysis

The fundamental problem is **lack of adaptive learning** in content discovery:
1. Platforms optimize for engagement metrics, not user satisfaction
2. No feedback loop from viewing outcomes to recommendations
3. Preference models are static snapshots, not evolving profiles
4. Cross-platform behavior patterns remain invisible

---

## 3. Solution Overview

### 3.1 Vision

StreamSense AI creates a **self-learning discovery layer** that sits above all streaming platforms, using reinforcement learning to continuously optimize recommendations based on what users actually watch, enjoy, and complete.

### 3.2 Core Innovation: Adaptive Learning Engine

```
User Query → Intent Understanding (Agentic Flow)
          → Preference Vector Lookup (AgentDB)
          → Semantic Content Search (RuVector 150x faster)
          → Recommendation (with confidence scores)
          → User Selection
          → Viewing Outcome Tracking
          → RL Update (Q-learning + Experience Replay)
          → Updated Preference Embeddings (RuVector)
```

**Self-Learning Capabilities:**
- Learns optimal content-to-intent mappings through experience replay
- Adapts preference embeddings based on viewing completion rates
- Discovers latent preferences through semantic vector clustering
- Improves query understanding through ReasoningBank trajectory analysis

---

## 4. User Stories

### 4.1 Core Discovery Flow

**As a user**, I want to describe what I'm in the mood for in natural language, so that I get relevant content without browsing multiple apps.

**Acceptance Criteria:**
- Support natural queries: "Something like Succession but funnier"
- Return results within 2 seconds
- Show availability across all platforms
- Learn from my selection (or non-selection)

**Learning Component:**
- Track query → recommendation → selection pathway
- Store experience in AgentDB replay buffer
- Update RuVector embeddings based on implicit feedback

---

**As a user**, I want recommendations to improve over time based on what I actually watch, not just what I click.

**Acceptance Criteria:**
- Track viewing completion rate (watched >70% = positive signal)
- Adjust preference vectors based on outcomes
- Prioritize content similar to completed shows
- Deprioritize patterns leading to abandonment

**Learning Component:**
```typescript
interface ViewingOutcome {
  contentId: string;
  queryContext: string;
  completionRate: number; // 0-100%
  rating?: number; // explicit feedback
  timestamp: number;
}

// Reward function
reward = (completionRate * 0.7) + (rating ?? 0) * 0.3;
```

---

**As a user**, I want the system to understand context (Friday night vs Sunday morning) and adjust recommendations accordingly.

**Acceptance Criteria:**
- Detect temporal patterns (weekday evening, weekend morning)
- Learn context-specific preferences
- Store context embeddings in RuVector
- Apply context-aware filtering

**Learning Component:**
- Context state space: {time, day, location, device, social}
- Context-conditional Q-tables in AgentDB
- ReasoningBank pattern recognition for contextual preferences

---

**As a power user**, I want to refine recommendations by providing feedback on why suggestions miss the mark.

**Acceptance Criteria:**
- "Not this" with reason: too dark, too slow, wrong genre
- Immediate re-ranking based on negative feedback
- Learn constraint patterns (user never watches X)
- Store constraint embeddings

**Learning Component:**
- Negative signal processing: `reward = -0.5`
- Constraint vector subtraction from preference embedding
- Hard constraint storage in AgentDB

---

**As a returning user**, I want my preferences to persist across devices and sessions.

**Acceptance Criteria:**
- AgentDB cross-session memory restoration
- Preference vector synchronization
- Viewing history merge across devices
- Context transfer (started on mobile, finish on TV)

**Learning Component:**
- Persistent state storage in AgentDB
- RuVector embedding synchronization
- Session continuity tracking

---

**As a user discovering new genres**, I want the system to detect and expand my taste boundaries.

**Acceptance Criteria:**
- Detect successful exploration (completed content outside usual preferences)
- Expand preference vector space
- Suggest similar "boundary content"
- Track genre evolution over time

**Learning Component:**
- Exploration vs exploitation balance (ε-greedy)
- Preference vector expansion (not just refinement)
- Novelty bonus in reward function
- Trajectory analysis via ReasoningBank

---

**As a user**, I want to see why recommendations were made and adjust the reasoning.

**Acceptance Criteria:**
- Explainability: "Because you enjoyed X and rated Y highly"
- Adjustable weights: "Care more about genre than actors"
- Transparency in learning progress
- Confidence scores on recommendations

**Learning Component:**
- ReasoningBank decision trajectory storage
- Feature importance attribution
- Weighted preference vectors
- Uncertainty quantification

---

## 5. Technical Architecture

### 5.1 System Architecture (ASCII Diagram)

```
┌─────────────────────────────────────────────────────────────────────┐
│                         StreamSense AI Platform                      │
└─────────────────────────────────────────────────────────────────────┘

┌───────────────┐         ┌──────────────────────────────────────────┐
│  User Device  │────────▶│         API Gateway (GraphQL)            │
│  (Web/Mobile) │         │  - Query parsing                         │
└───────────────┘         │  - Authentication                        │
                          │  - Rate limiting                         │
                          └──────────────────────────────────────────┘
                                           │
                    ┌──────────────────────┼──────────────────────┐
                    ▼                      ▼                      ▼
         ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
         │  Intent Engine   │  │ Recommendation   │  │  Learning Engine │
         │  (Agentic Flow)  │  │     Engine       │  │  (RL Controller) │
         │                  │  │                  │  │                  │
         │ • Query agent    │  │ • Ranking agent  │  │ • Q-learning     │
         │ • Context agent  │  │ • Filtering      │  │ • Replay buffer  │
         │ • Refinement     │  │ • Diversity      │  │ • Policy update  │
         └──────────────────┘  └──────────────────┘  └──────────────────┘
                    │                      │                      │
                    └──────────────────────┼──────────────────────┘
                                           ▼
                    ┌──────────────────────────────────────────────┐
                    │         RuVector Semantic Store              │
                    │                                              │
                    │  • Content embeddings (1536D)                │
                    │  • User preference vectors                   │
                    │  • Context embeddings                        │
                    │  • HNSW indexing (150x faster)               │
                    │  • Similarity search                         │
                    └──────────────────────────────────────────────┘
                                           │
                    ┌──────────────────────┼──────────────────────┐
                    ▼                      ▼                      ▼
         ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
         │     AgentDB      │  │  ReasoningBank   │  │  Platform APIs   │
         │                  │  │  (Agentic Flow)  │  │                  │
         │ • User profiles  │  │ • Trajectories   │  │ • Netflix        │
         │ • Q-tables       │  │ • Verdicts       │  │ • Disney+        │
         │ • Replay buffer  │  │ • Patterns       │  │ • Prime Video    │
         │ • Session state  │  │ • Distillation   │  │ • Apple TV+      │
         └──────────────────┘  └──────────────────┘  └──────────────────┘
```

### 5.2 Self-Learning Architecture (Detailed)

#### 5.2.1 State Space Design

```typescript
interface UserState {
  // User identity
  userId: string;

  // Preference embedding (learned)
  preferenceVector: Float32Array; // 1536D from RuVector

  // Current context
  context: {
    timestamp: number;
    dayOfWeek: number;
    hourOfDay: number;
    device: 'mobile' | 'tablet' | 'tv' | 'desktop';
    location?: 'home' | 'commute' | 'travel';
    social: 'solo' | 'partner' | 'family' | 'friends';
  };

  // Query intent
  queryEmbedding: Float32Array; // 1536D from query

  // Historical features
  recentViewing: {
    contentIds: string[];
    genres: string[];
    moods: string[];
    completionRates: number[];
  };

  // Exploration state
  explorationRate: number; // ε for ε-greedy

  // Constraint vectors (learned dislikes)
  constraintVectors: Float32Array[];
}
```

#### 5.2.2 Action Space Design

```typescript
interface RecommendationAction {
  contentId: string;
  platform: string;

  // Ranking features
  relevanceScore: number; // cosine similarity to preference vector
  diversityScore: number; // distance from recent viewing
  popularityScore: number; // global engagement

  // Learning metadata
  confidence: number; // model uncertainty
  explorationBonus: number; // UCB bonus for exploration

  // Explanation
  reasoning: {
    primaryMatch: string; // "Similar to 'Succession' which you rated 5/5"
    secondaryFactors: string[];
  };
}
```

#### 5.2.3 Reward Function

```typescript
function calculateReward(outcome: ViewingOutcome): number {
  const { completionRate, explicitRating, sessionDuration, returnRate } = outcome;

  // Primary signal: completion rate
  const completionReward = (completionRate / 100) * 0.5;

  // Secondary signal: explicit rating
  const ratingReward = explicitRating ? (explicitRating / 5) * 0.3 : 0;

  // Tertiary signal: session duration vs expected
  const durationReward = Math.min(sessionDuration / outcome.contentDuration, 1.0) * 0.1;

  // Quaternary signal: return to similar content
  const returnReward = returnRate * 0.1;

  return completionReward + ratingReward + durationReward + returnReward;
}
```

#### 5.2.4 Learning Algorithm (Q-Learning)

```typescript
class StreamSenseLearner {
  private learningRate = 0.1;
  private discountFactor = 0.95;
  private explorationRate = 0.15; // ε-greedy

  constructor(
    private agentDB: AgentDBClient,
    private ruVector: RuVectorClient,
    private reasoningBank: ReasoningBankClient
  ) {}

  async selectAction(state: UserState): Promise<RecommendationAction> {
    // ε-greedy exploration
    if (Math.random() < this.explorationRate) {
      return this.exploreAction(state);
    }

    return this.exploitAction(state);
  }

  private async exploitAction(state: UserState): Promise<RecommendationAction> {
    // Get Q-values for all possible actions from current state
    const stateHash = this.hashState(state);
    const qTable = await this.agentDB.get<QTable>(`q:${stateHash}`);

    // Combine Q-values with semantic search
    const candidates = await this.ruVector.search({
      vector: state.queryEmbedding,
      topK: 50,
      filter: this.buildContextFilter(state.context)
    });

    // Rank by Q-value + relevance
    const rankedActions = candidates.map(content => ({
      contentId: content.id,
      qValue: qTable?.[content.id] ?? 0,
      relevance: content.similarity,
      score: (qTable?.[content.id] ?? 0) * 0.6 + content.similarity * 0.4
    }));

    rankedActions.sort((a, b) => b.score - a.score);

    return this.buildRecommendationAction(rankedActions[0], state);
  }

  private async exploreAction(state: UserState): Promise<RecommendationAction> {
    // UCB exploration: select actions with high uncertainty
    const candidates = await this.ruVector.search({
      vector: state.queryEmbedding,
      topK: 50,
      filter: this.buildContextFilter(state.context)
    });

    const explorationScores = await Promise.all(
      candidates.map(async (content) => {
        const visitCount = await this.agentDB.get<number>(`visit:${content.id}`) ?? 0;
        const ucbBonus = Math.sqrt(2 * Math.log(state.totalActions) / (visitCount + 1));

        return {
          contentId: content.id,
          ucbScore: content.similarity + ucbBonus,
          visitCount
        };
      })
    );

    explorationScores.sort((a, b) => b.ucbScore - a.ucbScore);

    return this.buildRecommendationAction(explorationScores[0], state);
  }

  async updateQValue(experience: Experience): Promise<void> {
    const { state, action, reward, nextState } = experience;

    // Get current Q-value
    const stateHash = this.hashState(state);
    const currentQ = await this.agentDB.get<number>(`q:${stateHash}:${action}`) ?? 0;

    // Get max Q-value for next state
    const nextStateHash = this.hashState(nextState);
    const nextQTable = await this.agentDB.get<QTable>(`q:${nextStateHash}`) ?? {};
    const maxNextQ = Math.max(...Object.values(nextQTable), 0);

    // Q-learning update
    const newQ = currentQ + this.learningRate * (
      reward + this.discountFactor * maxNextQ - currentQ
    );

    // Store updated Q-value
    await this.agentDB.set(`q:${stateHash}:${action}`, newQ);

    // Store experience in replay buffer
    await this.agentDB.lpush('replay_buffer', experience, 10000); // keep last 10k

    // Update visit count
    await this.agentDB.incr(`visit:${action}`);

    // Track trajectory in ReasoningBank
    await this.reasoningBank.addTrajectory({
      state: stateHash,
      action,
      reward,
      nextState: nextStateHash,
      timestamp: Date.now()
    });
  }

  async updatePreferenceVector(
    userId: string,
    contentId: string,
    reward: number
  ): Promise<void> {
    // Get current preference vector
    const prefVector = await this.ruVector.get(`user:${userId}:preferences`);

    // Get content vector
    const contentVector = await this.ruVector.get(`content:${contentId}`);

    // Update with learning rate proportional to reward
    const alpha = this.learningRate * reward;
    const updatedVector = this.vectorLerp(prefVector, contentVector, alpha);

    // Store updated preference
    await this.ruVector.upsert({
      id: `user:${userId}:preferences`,
      vector: updatedVector,
      metadata: {
        lastUpdate: Date.now(),
        updateCount: (prefVector.metadata?.updateCount ?? 0) + 1
      }
    });
  }

  private vectorLerp(
    v1: Float32Array,
    v2: Float32Array,
    alpha: number
  ): Float32Array {
    const result = new Float32Array(v1.length);
    for (let i = 0; i < v1.length; i++) {
      result[i] = v1[i] * (1 - alpha) + v2[i] * alpha;
    }
    return result;
  }

  private hashState(state: UserState): string {
    // Create compact state representation for Q-table lookup
    return `${state.userId}:${state.context.dayOfWeek}:${state.context.hourOfDay}:${state.context.social}`;
  }
}
```

---

## 6. Data Models

### 6.1 Core Entities

```typescript
// User Profile (AgentDB)
interface UserProfile {
  userId: string;
  createdAt: number;

  // Learning state
  preferenceVectorId: string; // RuVector reference
  explorationRate: number;
  totalActions: number;
  totalReward: number;

  // Demographics (for cold start)
  demographics?: {
    ageRange: string;
    location: string;
    subscriptions: string[];
  };

  // Constraint learning
  hardConstraints: {
    neverShow: string[]; // content IDs
    blockedGenres: string[];
    blockedActors: string[];
  };

  // Context patterns
  contextProfiles: {
    [contextKey: string]: {
      preferenceVectorId: string;
      performanceMetric: number;
    };
  };
}

// Content Metadata (RuVector + metadata store)
interface ContentMetadata {
  contentId: string;
  title: string;
  platform: string;

  // Embedding
  vectorId: string; // RuVector reference

  // Structured metadata
  genres: string[];
  releaseYear: number;
  runtime: number;
  rating: string;
  cast: string[];
  director: string;

  // Learning features
  globalEngagement: number; // 0-1 score
  completionRate: number; // average across users
  emotionalTags: string[]; // "uplifting", "dark", "funny"

  // Availability
  platforms: Array<{
    name: string;
    url: string;
    subscriptionRequired: boolean;
  }>;
}

// Viewing Outcome (AgentDB - Experience Replay)
interface ViewingOutcome {
  userId: string;
  contentId: string;
  queryContext: string;

  // State before
  stateBefore: UserState;

  // Action taken
  recommendationRank: number; // position in recommendation list

  // Outcome
  selected: boolean;
  startTime?: number;
  endTime?: number;
  completionRate: number; // 0-100%
  sessionDuration: number; // seconds
  explicitRating?: number; // 1-5 if provided

  // Reward
  reward: number;

  // State after
  stateAfter: UserState;

  timestamp: number;
}

// Q-Table Entry (AgentDB)
interface QTableEntry {
  stateHash: string;
  actionId: string; // contentId
  qValue: number;
  visitCount: number;
  lastUpdate: number;
}

// Preference Vector (RuVector)
interface PreferenceVector {
  id: string; // user:${userId}:preferences or user:${userId}:context:${contextKey}
  vector: Float32Array; // 1536D
  metadata: {
    userId: string;
    contextKey?: string;
    updateCount: number;
    lastUpdate: number;
    avgReward: number;
  };
}

// Decision Trajectory (ReasoningBank)
interface DecisionTrajectory {
  trajectoryId: string;
  userId: string;

  steps: Array<{
    state: string; // stateHash
    action: string; // contentId
    reward: number;
    timestamp: number;
  }>;

  // Verdict
  overallOutcome: 'success' | 'failure' | 'neutral';
  totalReward: number;

  // Patterns
  identifiedPatterns: string[];
}
```

---

## 7. API Specifications

### 7.1 GraphQL Schema

```graphql
type Query {
  # Main discovery endpoint
  discover(input: DiscoverInput!): DiscoveryResult!

  # User profile
  userProfile(userId: ID!): UserProfile!

  # Learning insights
  learningInsights(userId: ID!): LearningInsights!

  # Content details
  content(contentId: ID!): Content!
}

type Mutation {
  # Track viewing outcome
  trackViewing(input: ViewingOutcomeInput!): TrackingResult!

  # Explicit feedback
  rateContent(contentId: ID!, rating: Int!, context: String): RatingResult!

  # Refinement
  refineRecommendation(
    recommendationId: ID!,
    feedback: RefinementFeedback!
  ): DiscoveryResult!

  # Preference management
  updateConstraints(userId: ID!, constraints: ConstraintInput!): UserProfile!
}

input DiscoverInput {
  userId: ID!
  query: String!
  context: ContextInput
  limit: Int = 20
  includeExplanations: Boolean = true
}

input ContextInput {
  device: Device
  location: Location
  social: SocialContext
  # System will auto-detect timestamp, dayOfWeek, hourOfDay
}

enum Device {
  MOBILE
  TABLET
  TV
  DESKTOP
}

enum Location {
  HOME
  COMMUTE
  TRAVEL
}

enum SocialContext {
  SOLO
  PARTNER
  FAMILY
  FRIENDS
}

type DiscoveryResult {
  recommendations: [Recommendation!]!
  learningMetrics: LearningMetrics!
  explanations: [Explanation!]
}

type Recommendation {
  contentId: ID!
  title: String!
  platform: String!

  # Ranking
  rank: Int!
  relevanceScore: Float!
  confidence: Float!

  # Metadata
  metadata: ContentMetadata!

  # Learning
  explorationFlag: Boolean! # true if exploration action
  qValue: Float # Q-value for this state-action pair

  # Explanation
  reasoning: RecommendationReasoning!
}

type RecommendationReasoning {
  primaryMatch: String!
  secondaryFactors: [String!]!
  confidenceFactors: [ConfidenceFactor!]!
}

type ConfidenceFactor {
  factor: String!
  weight: Float!
  contribution: Float!
}

type LearningMetrics {
  explorationRate: Float!
  totalActions: Int!
  avgReward: Float!
  preferenceStability: Float! # how much pref vector is changing
  modelConfidence: Float!
}

input ViewingOutcomeInput {
  userId: ID!
  contentId: ID!
  recommendationId: ID!

  startTime: DateTime!
  endTime: DateTime
  completionRate: Float!
  explicitRating: Int # 1-5
}

type TrackingResult {
  success: Boolean!
  rewardCalculated: Float!
  learningUpdated: Boolean!
}

input RefinementFeedback {
  reason: RefinementReason!
  details: String
}

enum RefinementReason {
  TOO_DARK
  TOO_SLOW
  WRONG_GENRE
  ALREADY_SEEN
  NOT_INTERESTED
  TOO_LONG
  WRONG_MOOD
}

type LearningInsights {
  preferenceEvolution: [PreferenceSnapshot!]!
  topGenres: [GenreAffinity!]!
  contextPatterns: [ContextPattern!]!
  explorationHistory: [ExplorationEvent!]!
}

type PreferenceSnapshot {
  timestamp: DateTime!
  topAffinities: [String!]!
  avgReward: Float!
}

type GenreAffinity {
  genre: String!
  affinity: Float! # -1 to 1
  confidence: Float!
  sampleSize: Int!
}

type ContextPattern {
  context: String! # e.g., "Friday evening solo"
  preferredGenres: [String!]!
  avgCompletionRate: Float!
  sampleSize: Int!
}

type ExplorationEvent {
  timestamp: DateTime!
  contentId: ID!
  outcome: Float! # reward
  led_to_preference_expansion: Boolean!
}
```

### 7.2 REST API Endpoints (Alternative)

```typescript
// POST /api/v1/discover
interface DiscoverRequest {
  userId: string;
  query: string;
  context?: {
    device?: 'mobile' | 'tablet' | 'tv' | 'desktop';
    location?: 'home' | 'commute' | 'travel';
    social?: 'solo' | 'partner' | 'family' | 'friends';
  };
  limit?: number;
  includeExplanations?: boolean;
}

interface DiscoverResponse {
  recommendations: Array<{
    contentId: string;
    title: string;
    platform: string;
    rank: number;
    relevanceScore: number;
    confidence: number;
    metadata: ContentMetadata;
    reasoning: {
      primaryMatch: string;
      secondaryFactors: string[];
    };
  }>;
  learningMetrics: {
    explorationRate: number;
    totalActions: number;
    avgReward: number;
    modelConfidence: number;
  };
}

// POST /api/v1/track
interface TrackViewingRequest {
  userId: string;
  contentId: string;
  recommendationId: string;
  startTime: string; // ISO 8601
  endTime?: string;
  completionRate: number; // 0-100
  explicitRating?: number; // 1-5
}

interface TrackViewingResponse {
  success: boolean;
  reward: number;
  learningUpdated: boolean;
  newQValue?: number;
  preferenceVectorUpdated: boolean;
}

// GET /api/v1/insights/:userId
interface InsightsResponse {
  preferenceEvolution: Array<{
    timestamp: string;
    topAffinities: string[];
    avgReward: number;
  }>;
  topGenres: Array<{
    genre: string;
    affinity: number; // -1 to 1
    confidence: number;
  }>;
  contextPatterns: Array<{
    context: string;
    preferredGenres: string[];
    avgCompletionRate: number;
  }>;
}
```

---

## 8. RuVector Integration Patterns

### 8.1 Initialization

```typescript
import { RuVector } from 'ruvector';

const contentVectors = new RuVector({
  dimensions: 1536,
  indexType: 'hnsw',
  efConstruction: 200,
  M: 16,
  space: 'cosine'
});

const preferenceVectors = new RuVector({
  dimensions: 1536,
  indexType: 'hnsw',
  efConstruction: 200,
  M: 16,
  space: 'cosine'
});
```

### 8.2 Content Embedding Pipeline

```typescript
import { ruvLLM } from 'ruvector/ruvLLM';

async function embedContent(content: ContentMetadata): Promise<void> {
  // Create rich text representation
  const textRepresentation = `
    Title: ${content.title}
    Genres: ${content.genres.join(', ')}
    Description: ${content.description}
    Mood: ${content.emotionalTags.join(', ')}
    Cast: ${content.cast.slice(0, 5).join(', ')}
    Director: ${content.director}
  `.trim();

  // Generate embedding using ruvLLM
  const embedding = await ruvLLM.embed(textRepresentation);

  // Store in RuVector
  await contentVectors.upsert({
    id: `content:${content.contentId}`,
    vector: embedding,
    metadata: {
      contentId: content.contentId,
      platform: content.platform,
      genres: content.genres,
      releaseYear: content.releaseYear,
      globalEngagement: content.globalEngagement
    }
  });
}
```

### 8.3 Query Understanding with ruvLLM

```typescript
async function processQuery(
  query: string,
  userId: string
): Promise<RecommendationAction[]> {
  // Step 1: Embed query
  const queryEmbedding = await ruvLLM.embed(query);

  // Step 2: Get user preference vector
  const preferenceResult = await preferenceVectors.get(`user:${userId}:preferences`);
  const preferenceVector = preferenceResult?.vector;

  // Step 3: Combine query + preference (weighted average)
  const combinedVector = preferenceVector
    ? weightedAverage(queryEmbedding, preferenceVector, 0.6, 0.4)
    : queryEmbedding;

  // Step 4: Semantic search with RuVector (150x faster HNSW)
  const searchResults = await contentVectors.search({
    vector: combinedVector,
    topK: 50,
    includeMetadata: true
  });

  // Step 5: Re-rank with Q-values
  const rankedResults = await reRankWithQLearning(userId, searchResults);

  return rankedResults;
}

function weightedAverage(
  v1: Float32Array,
  v2: Float32Array,
  w1: number,
  w2: number
): Float32Array {
  const result = new Float32Array(v1.length);
  for (let i = 0; i < v1.length; i++) {
    result[i] = v1[i] * w1 + v2[i] * w2;
  }
  return result;
}
```

### 8.4 Preference Learning Update

```typescript
async function updatePreferencesFromViewing(
  userId: string,
  contentId: string,
  reward: number
): Promise<void> {
  // Get current preference vector
  const prefResult = await preferenceVectors.get(`user:${userId}:preferences`);
  const currentPref = prefResult?.vector ?? new Float32Array(1536);

  // Get content vector
  const contentResult = await contentVectors.get(`content:${contentId}`);
  if (!contentResult) return;

  const contentVector = contentResult.vector;

  // Learning rate proportional to reward
  const learningRate = 0.1 * Math.abs(reward);

  // Update direction: toward content if positive, away if negative
  const direction = reward > 0 ? 1 : -1;

  // Vector update
  const updatedPref = new Float32Array(1536);
  for (let i = 0; i < 1536; i++) {
    const delta = (contentVector[i] - currentPref[i]) * learningRate * direction;
    updatedPref[i] = currentPref[i] + delta;
  }

  // Normalize
  const norm = Math.sqrt(updatedPref.reduce((sum, val) => sum + val * val, 0));
  for (let i = 0; i < 1536; i++) {
    updatedPref[i] /= norm;
  }

  // Store updated preference
  await preferenceVectors.upsert({
    id: `user:${userId}:preferences`,
    vector: updatedPref,
    metadata: {
      userId,
      lastUpdate: Date.now(),
      updateCount: (prefResult?.metadata?.updateCount ?? 0) + 1
    }
  });
}
```

### 8.5 Context-Specific Preferences

```typescript
async function getContextualPreference(
  userId: string,
  context: ContextInput
): Promise<Float32Array> {
  // Build context key
  const contextKey = `${context.social}:${context.device}`;

  // Try to get context-specific preference
  const contextPrefId = `user:${userId}:context:${contextKey}`;
  const contextResult = await preferenceVectors.get(contextPrefId);

  if (contextResult && contextResult.metadata.updateCount > 5) {
    // Sufficient data for context-specific preference
    return contextResult.vector;
  }

  // Fall back to general preference
  const generalResult = await preferenceVectors.get(`user:${userId}:preferences`);
  return generalResult?.vector ?? new Float32Array(1536);
}
```

---

## 9. AgentDB Integration Patterns

### 9.1 Initialization

```typescript
import { AgentDB } from 'agentic-flow/agentdb';

const agentDB = new AgentDB({
  persistPath: './data/streamsense-memory',
  autoSave: true,
  saveInterval: 60000 // 1 minute
});
```

### 9.2 Q-Table Management

```typescript
class QTableManager {
  constructor(private agentDB: AgentDB) {}

  async getQValue(stateHash: string, action: string): Promise<number> {
    const key = `q:${stateHash}:${action}`;
    return await this.agentDB.get(key) ?? 0;
  }

  async setQValue(stateHash: string, action: string, value: number): Promise<void> {
    const key = `q:${stateHash}:${action}`;
    await this.agentDB.set(key, value);
  }

  async getAllActionsForState(stateHash: string): Promise<Map<string, number>> {
    const pattern = `q:${stateHash}:*`;
    const keys = await this.agentDB.keys(pattern);

    const qValues = new Map<string, number>();
    for (const key of keys) {
      const action = key.split(':')[2];
      const value = await this.agentDB.get<number>(key);
      if (value !== null) {
        qValues.set(action, value);
      }
    }

    return qValues;
  }

  async getBestAction(stateHash: string): Promise<{ action: string; qValue: number } | null> {
    const qValues = await this.getAllActionsForState(stateHash);

    if (qValues.size === 0) return null;

    let bestAction = '';
    let bestValue = -Infinity;

    for (const [action, value] of qValues.entries()) {
      if (value > bestValue) {
        bestValue = value;
        bestAction = action;
      }
    }

    return { action: bestAction, qValue: bestValue };
  }
}
```

### 9.3 Experience Replay Buffer

```typescript
interface Experience {
  userId: string;
  stateHash: string;
  action: string;
  reward: number;
  nextStateHash: string;
  timestamp: number;
}

class ReplayBuffer {
  private maxSize = 10000;

  constructor(private agentDB: AgentDB) {}

  async addExperience(exp: Experience): Promise<void> {
    // Add to list
    await this.agentDB.lpush('replay_buffer', exp);

    // Trim to max size
    await this.agentDB.ltrim('replay_buffer', 0, this.maxSize - 1);
  }

  async sampleBatch(batchSize: number): Promise<Experience[]> {
    const bufferSize = await this.agentDB.llen('replay_buffer');
    if (bufferSize === 0) return [];

    const samples: Experience[] = [];
    const sampleSize = Math.min(batchSize, bufferSize);

    for (let i = 0; i < sampleSize; i++) {
      const randomIndex = Math.floor(Math.random() * bufferSize);
      const exp = await this.agentDB.lindex<Experience>('replay_buffer', randomIndex);
      if (exp) samples.push(exp);
    }

    return samples;
  }

  async batchUpdate(batchSize: number = 32): Promise<void> {
    const batch = await this.sampleBatch(batchSize);
    const qTableManager = new QTableManager(this.agentDB);

    for (const exp of batch) {
      // Get current Q-value
      const currentQ = await qTableManager.getQValue(exp.stateHash, exp.action);

      // Get max Q-value for next state
      const nextBest = await qTableManager.getBestAction(exp.nextStateHash);
      const maxNextQ = nextBest?.qValue ?? 0;

      // Q-learning update
      const learningRate = 0.1;
      const discountFactor = 0.95;
      const newQ = currentQ + learningRate * (
        exp.reward + discountFactor * maxNextQ - currentQ
      );

      // Update Q-table
      await qTableManager.setQValue(exp.stateHash, exp.action, newQ);
    }
  }
}
```

### 9.4 User Profile Persistence

```typescript
interface UserProfileData {
  userId: string;
  createdAt: number;
  preferenceVectorId: string;
  explorationRate: number;
  totalActions: number;
  totalReward: number;
  hardConstraints: {
    neverShow: string[];
    blockedGenres: string[];
  };
}

class UserProfileManager {
  constructor(private agentDB: AgentDB) {}

  async getProfile(userId: string): Promise<UserProfileData | null> {
    return await this.agentDB.get<UserProfileData>(`profile:${userId}`);
  }

  async createProfile(userId: string): Promise<UserProfileData> {
    const profile: UserProfileData = {
      userId,
      createdAt: Date.now(),
      preferenceVectorId: `user:${userId}:preferences`,
      explorationRate: 0.15, // Initial exploration rate
      totalActions: 0,
      totalReward: 0,
      hardConstraints: {
        neverShow: [],
        blockedGenres: []
      }
    };

    await this.agentDB.set(`profile:${userId}`, profile);
    return profile;
  }

  async updateProfile(userId: string, updates: Partial<UserProfileData>): Promise<void> {
    const profile = await this.getProfile(userId);
    if (!profile) throw new Error('Profile not found');

    const updated = { ...profile, ...updates };
    await this.agentDB.set(`profile:${userId}`, updated);
  }

  async incrementActions(userId: string, reward: number): Promise<void> {
    const profile = await this.getProfile(userId);
    if (!profile) return;

    await this.updateProfile(userId, {
      totalActions: profile.totalActions + 1,
      totalReward: profile.totalReward + reward
    });
  }

  async addConstraint(
    userId: string,
    type: 'neverShow' | 'blockedGenres',
    value: string
  ): Promise<void> {
    const profile = await this.getProfile(userId);
    if (!profile) return;

    const constraints = { ...profile.hardConstraints };
    if (!constraints[type].includes(value)) {
      constraints[type].push(value);
    }

    await this.updateProfile(userId, { hardConstraints: constraints });
  }
}
```

---

## 10. Agentic Flow Integration

### 10.1 Agent Definitions

```typescript
// Intent Understanding Agent
const intentAgent = {
  name: 'intent-analyzer',
  type: 'analyst',
  capabilities: [
    'natural-language-understanding',
    'query-embedding',
    'intent-classification'
  ],

  async process(query: string, userId: string): Promise<IntentAnalysis> {
    // Use ruvLLM for intent understanding
    const embedding = await ruvLLM.embed(query);

    // Classify intent type
    const intentType = await this.classifyIntent(query);

    // Extract entities
    const entities = await this.extractEntities(query);

    return {
      queryEmbedding: embedding,
      intentType,
      entities,
      confidence: this.calculateConfidence(query)
    };
  },

  async classifyIntent(query: string): Promise<IntentType> {
    // Classification logic
    const lowerQuery = query.toLowerCase();

    if (lowerQuery.includes('like') || lowerQuery.includes('similar')) {
      return 'similarity-search';
    }
    if (lowerQuery.includes('mood') || lowerQuery.includes('feel')) {
      return 'mood-based';
    }
    if (lowerQuery.includes('tonight') || lowerQuery.includes('watch')) {
      return 'general-discovery';
    }

    return 'general-discovery';
  }
};

// Recommendation Ranking Agent
const rankingAgent = {
  name: 'recommendation-ranker',
  type: 'optimizer',
  capabilities: [
    'multi-criteria-ranking',
    'diversity-optimization',
    'explanation-generation'
  ],

  async rank(
    candidates: ContentCandidate[],
    userState: UserState,
    qTableManager: QTableManager
  ): Promise<RankedRecommendation[]> {
    const ranked = await Promise.all(
      candidates.map(async (candidate) => {
        // Get Q-value
        const qValue = await qTableManager.getQValue(
          hashState(userState),
          candidate.contentId
        );

        // Calculate diversity score
        const diversityScore = this.calculateDiversity(
          candidate,
          userState.recentViewing
        );

        // Combined score
        const score = (
          qValue * 0.5 +
          candidate.relevanceScore * 0.3 +
          diversityScore * 0.2
        );

        return {
          ...candidate,
          qValue,
          diversityScore,
          finalScore: score,
          explanation: this.generateExplanation(candidate, qValue, diversityScore)
        };
      })
    );

    // Sort by final score
    ranked.sort((a, b) => b.finalScore - a.finalScore);

    return ranked;
  },

  calculateDiversity(
    candidate: ContentCandidate,
    recentViewing: string[]
  ): number {
    // Calculate how different this is from recent viewing
    // Higher score = more diverse
    let diversityScore = 1.0;

    for (const recentId of recentViewing.slice(0, 5)) {
      if (candidate.contentId === recentId) {
        return 0; // Already watched
      }

      // Genre overlap penalty
      const genreOverlap = this.calculateGenreOverlap(candidate, recentId);
      diversityScore *= (1 - genreOverlap * 0.3);
    }

    return diversityScore;
  },

  generateExplanation(
    candidate: ContentCandidate,
    qValue: number,
    diversityScore: number
  ): string {
    if (qValue > 0.8) {
      return `Highly recommended based on your viewing history`;
    }
    if (diversityScore > 0.7) {
      return `Something different from your usual preferences`;
    }
    return `Matches your current search`;
  }
};

// Learning Coordinator Agent
const learningAgent = {
  name: 'learning-coordinator',
  type: 'coordinator',
  capabilities: [
    'q-learning-updates',
    'experience-replay',
    'preference-vector-updates',
    'reasoning-bank-integration'
  ],

  async processOutcome(outcome: ViewingOutcome): Promise<void> {
    // Calculate reward
    const reward = calculateReward(outcome);

    // Create experience
    const experience: Experience = {
      userId: outcome.userId,
      stateHash: hashState(outcome.stateBefore),
      action: outcome.contentId,
      reward,
      nextStateHash: hashState(outcome.stateAfter),
      timestamp: outcome.timestamp
    };

    // Add to replay buffer
    const replayBuffer = new ReplayBuffer(agentDB);
    await replayBuffer.addExperience(experience);

    // Update Q-value
    const qTableManager = new QTableManager(agentDB);
    const currentQ = await qTableManager.getQValue(experience.stateHash, experience.action);
    const nextBest = await qTableManager.getBestAction(experience.nextStateHash);
    const maxNextQ = nextBest?.qValue ?? 0;

    const newQ = currentQ + 0.1 * (reward + 0.95 * maxNextQ - currentQ);
    await qTableManager.setQValue(experience.stateHash, experience.action, newQ);

    // Update preference vector
    await updatePreferencesFromViewing(outcome.userId, outcome.contentId, reward);

    // Track in ReasoningBank
    await reasoningBank.addTrajectory({
      userId: outcome.userId,
      state: experience.stateHash,
      action: experience.action,
      reward,
      timestamp: experience.timestamp
    });

    // Trigger batch update periodically
    if (outcome.timestamp % 100 === 0) {
      await replayBuffer.batchUpdate(32);
    }
  }
};
```

### 10.2 Agent Orchestration

```typescript
class StreamSenseOrchestrator {
  private intentAgent: typeof intentAgent;
  private rankingAgent: typeof rankingAgent;
  private learningAgent: typeof learningAgent;

  async processDiscoveryRequest(
    userId: string,
    query: string,
    context: ContextInput
  ): Promise<DiscoveryResult> {
    // Step 1: Intent analysis
    const intentAnalysis = await this.intentAgent.process(query, userId);

    // Step 2: Get user state
    const userState = await this.buildUserState(userId, context, intentAnalysis);

    // Step 3: Semantic search with RuVector
    const candidates = await this.searchContent(userState, intentAnalysis.queryEmbedding);

    // Step 4: Rank with Q-learning
    const qTableManager = new QTableManager(agentDB);
    const ranked = await this.rankingAgent.rank(candidates, userState, qTableManager);

    // Step 5: Build response
    return {
      recommendations: ranked.slice(0, 20),
      learningMetrics: await this.getLearningMetrics(userId),
      explanations: ranked.map(r => r.explanation)
    };
  }

  async trackViewingOutcome(outcome: ViewingOutcome): Promise<void> {
    await this.learningAgent.processOutcome(outcome);
  }

  private async buildUserState(
    userId: string,
    context: ContextInput,
    intentAnalysis: IntentAnalysis
  ): Promise<UserState> {
    // Get profile
    const profileManager = new UserProfileManager(agentDB);
    const profile = await profileManager.getProfile(userId);

    if (!profile) throw new Error('User profile not found');

    // Get preference vector
    const preferenceVector = await getContextualPreference(userId, context);

    // Build state
    return {
      userId,
      preferenceVector,
      context: {
        timestamp: Date.now(),
        dayOfWeek: new Date().getDay(),
        hourOfDay: new Date().getHours(),
        device: context.device ?? 'desktop',
        location: context.location,
        social: context.social ?? 'solo'
      },
      queryEmbedding: intentAnalysis.queryEmbedding,
      recentViewing: await this.getRecentViewing(userId),
      explorationRate: profile.explorationRate
    };
  }

  private async searchContent(
    userState: UserState,
    queryEmbedding: Float32Array
  ): Promise<ContentCandidate[]> {
    // Combine query + preference
    const combinedVector = weightedAverage(
      queryEmbedding,
      userState.preferenceVector,
      0.6,
      0.4
    );

    // Search with RuVector
    const results = await contentVectors.search({
      vector: combinedVector,
      topK: 50,
      includeMetadata: true
    });

    return results.map(r => ({
      contentId: r.id,
      relevanceScore: r.similarity,
      metadata: r.metadata
    }));
  }
}
```

---

## 11. Learning Metrics & KPIs

### 11.1 Model Performance Metrics

```typescript
interface LearningMetrics {
  // Recommendation quality
  recommendationAcceptanceRate: number; // % of recommendations clicked
  completionRate: number; // % of started content finished
  avgReward: number; // Average reward per recommendation

  // Learning progress
  qValueConvergence: number; // How stable Q-values are
  preferenceVectorStability: number; // How much pref vector changes
  explorationRate: number; // Current ε value

  // User satisfaction
  explicitRatingAvg: number; // Average user rating
  returnRate: number; // % users returning to app
  timeToDecision: number; // Avg seconds from query to selection

  // Model confidence
  avgConfidence: number; // Average prediction confidence
  uncertaintyReduction: number; // How much uncertainty decreased
}

class MetricsTracker {
  constructor(private agentDB: AgentDB) {}

  async calculateMetrics(userId: string, timeWindow: number = 7 * 24 * 60 * 60 * 1000): Promise<LearningMetrics> {
    const now = Date.now();
    const startTime = now - timeWindow;

    // Get all experiences in time window
    const experiences = await this.getExperiences(userId, startTime, now);

    if (experiences.length === 0) {
      return this.getDefaultMetrics();
    }

    // Calculate metrics
    const totalReward = experiences.reduce((sum, exp) => sum + exp.reward, 0);
    const avgReward = totalReward / experiences.length;

    const completedExperiences = experiences.filter(exp => exp.reward > 0.7);
    const completionRate = completedExperiences.length / experiences.length;

    // Q-value convergence: variance of Q-value updates
    const qValueChanges = await this.getQValueChanges(userId, startTime, now);
    const qValueConvergence = 1 - this.calculateVariance(qValueChanges);

    // Preference vector stability
    const vectorChanges = await this.getVectorChanges(userId, startTime, now);
    const preferenceVectorStability = 1 - this.calculateVectorDistance(vectorChanges);

    // Get current exploration rate
    const profile = await this.agentDB.get<UserProfileData>(`profile:${userId}`);
    const explorationRate = profile?.explorationRate ?? 0.15;

    return {
      recommendationAcceptanceRate: completionRate,
      completionRate,
      avgReward,
      qValueConvergence,
      preferenceVectorStability,
      explorationRate,
      explicitRatingAvg: this.calculateAvgRating(experiences),
      returnRate: await this.calculateReturnRate(userId),
      timeToDecision: await this.calculateAvgDecisionTime(userId, startTime, now),
      avgConfidence: this.calculateAvgConfidence(experiences),
      uncertaintyReduction: this.calculateUncertaintyReduction(experiences)
    };
  }

  private calculateVariance(values: number[]): number {
    if (values.length === 0) return 0;

    const mean = values.reduce((sum, val) => sum + val, 0) / values.length;
    const squaredDiffs = values.map(val => Math.pow(val - mean, 2));
    return squaredDiffs.reduce((sum, val) => sum + val, 0) / values.length;
  }

  private calculateVectorDistance(vectors: Float32Array[]): number {
    if (vectors.length < 2) return 0;

    let totalDistance = 0;
    for (let i = 1; i < vectors.length; i++) {
      totalDistance += this.cosineSimilarity(vectors[i-1], vectors[i]);
    }

    return 1 - (totalDistance / (vectors.length - 1));
  }

  private cosineSimilarity(v1: Float32Array, v2: Float32Array): number {
    let dotProduct = 0;
    let norm1 = 0;
    let norm2 = 0;

    for (let i = 0; i < v1.length; i++) {
      dotProduct += v1[i] * v2[i];
      norm1 += v1[i] * v1[i];
      norm2 += v2[i] * v2[i];
    }

    return dotProduct / (Math.sqrt(norm1) * Math.sqrt(norm2));
  }
}
```

### 11.2 Business KPIs

```typescript
interface BusinessKPIs {
  // User engagement
  dailyActiveUsers: number;
  weeklyActiveUsers: number;
  avgSessionDuration: number; // minutes
  avgSessionsPerUser: number;

  // Conversion
  queryToClickRate: number; // % queries leading to clicks
  clickToWatchRate: number; // % clicks leading to watch >5min
  watchToCompleteRate: number; // % watches completing >70%

  // Retention
  day1Retention: number;
  day7Retention: number;
  day30Retention: number;

  // Discovery efficiency
  avgTimeToDecision: number; // seconds from query to watch
  avgQueriesPerSession: number;
  crossPlatformDiscoveryRate: number; // % discoveries across platforms

  // Learning impact
  recommendationImprovementRate: number; // % improvement in acceptance over time
  personalizationScore: number; // How different user profiles are
}
```

---

## 12. MVP Scope (Week 1)

### 12.1 Core Features

**Must Have:**
1. ✅ Natural language query processing
2. ✅ Unified search across 3 platforms (Netflix, Disney+, Prime)
3. ✅ Basic Q-learning implementation
4. ✅ RuVector content embeddings
5. ✅ AgentDB user profiles
6. ✅ Viewing outcome tracking
7. ✅ Simple preference vector updates

**Technical Deliverables:**
- RuVector initialized with 1000 content embeddings
- AgentDB schema for users, Q-tables, experiences
- 3 Agentic Flow agents: intent, ranking, learning
- GraphQL API (5 core endpoints)
- Basic web interface (search + results)

### 12.2 Learning Capabilities (Week 1)

**Simplified RL:**
- ε-greedy exploration (ε = 0.2)
- Q-learning with experience replay (buffer size: 1000)
- Preference vector updates on explicit ratings only
- No context-specific learning yet

**Success Criteria:**
- 2-second query response time
- 50% recommendation acceptance rate (baseline)
- Q-values converging after 100 user interactions
- Preference vectors updating correctly

---

## 13. Enhanced Scope (Week 2)

### 13.1 Advanced Features

**Add:**
1. ✅ Context-aware recommendations (time, device, social)
2. ✅ Constraint learning (never show, blocked genres)
3. ✅ Exploration strategy (UCB instead of ε-greedy)
4. ✅ ReasoningBank trajectory analysis
5. ✅ Batch Q-learning updates
6. ✅ Preference vector clustering (discover user segments)
7. ✅ Explanation generation

**Technical Enhancements:**
- Context-specific preference vectors
- UCB exploration with confidence bounds
- ReasoningBank pattern distillation
- Batch replay buffer processing
- K-means clustering on preference vectors

### 13.2 Learning Enhancements (Week 2)

**Advanced RL:**
- Context-conditional Q-tables
- Prioritized experience replay (sample high-reward experiences more)
- Dual learning rates (fast for new users, slow for converged users)
- Curiosity-driven exploration bonus
- Meta-learning across users (transfer learning)

**Success Criteria:**
- 70% recommendation acceptance rate
- 30% reduction in time to decision
- 85% preference vector stability
- Context-aware recommendations working

---

## 14. Success Criteria

### 14.1 MVP Success (Week 1)

**User Metrics:**
- ✅ 50 beta users
- ✅ 500 total queries
- ✅ 50% recommendation acceptance
- ✅ 60% completion rate
- ✅ <5s time to decision

**Technical Metrics:**
- ✅ 99% API uptime
- ✅ <2s query latency
- ✅ Q-values converging
- ✅ Preference vectors updating
- ✅ Zero data loss

### 14.2 Production Success (Week 2)

**User Metrics:**
- ✅ 500 active users
- ✅ 5,000 total queries
- ✅ 70% recommendation acceptance
- ✅ 75% completion rate
- ✅ <3s time to decision
- ✅ 80% day-7 retention

**Learning Metrics:**
- ✅ 30% improvement in recommendation quality (week 1 vs week 2)
- ✅ Preference vectors stable (>85%)
- ✅ Context patterns identified (>5 distinct patterns)
- ✅ User segments discovered (>3 clusters)

**Business Metrics:**
- ✅ 40% reduction in decision time vs baseline (45min → 27min)
- ✅ 20% increase in cross-platform discovery
- ✅ NPS score >50

---

## 15. Risk Mitigation

### 15.1 Technical Risks

**Risk: RuVector performance degrades with 100k+ embeddings**
- Mitigation: Benchmark at 10k, 50k, 100k embeddings in week 1
- Fallback: Use hierarchical indexing or sharding

**Risk: Q-learning doesn't converge**
- Mitigation: Monitor convergence metrics daily
- Fallback: Use simpler collaborative filtering

**Risk: Preference vectors overfit to recent behavior**
- Mitigation: Add L2 regularization, limit update magnitude
- Fallback: Use exponential moving average

**Risk: Cold start problem (new users)**
- Mitigation: Demographic-based initialization, popular content bootstrapping
- Fallback: Fallback to trending content

### 15.2 Product Risks

**Risk: Users don't provide enough feedback for learning**
- Mitigation: Implicit feedback (completion rate) as primary signal
- Fallback: Reduce exploration rate, use more exploitation

**Risk: Learning converges to local optimum (filter bubble)**
- Mitigation: Forced exploration (10% random recommendations)
- Fallback: Periodic preference vector perturbation

**Risk: Privacy concerns with tracking**
- Mitigation: Local-first storage, clear consent, data deletion
- Fallback: Anonymous mode with no learning

---

## 16. Implementation Timeline

### Week 1: MVP
- **Day 1-2**: RuVector + AgentDB setup, content embedding pipeline
- **Day 3-4**: Basic Q-learning, intent agent, ranking agent
- **Day 5**: GraphQL API, basic UI
- **Day 6**: Testing, debugging
- **Day 7**: Beta launch

### Week 2: Enhancement
- **Day 8-9**: Context-aware learning, ReasoningBank integration
- **Day 10-11**: Advanced RL (UCB, prioritized replay)
- **Day 12-13**: User clustering, pattern discovery
- **Day 14**: Production launch

---

## Appendix A: Code Snippets

### A.1 Complete End-to-End Flow

```typescript
import { RuVector } from 'ruvector';
import { AgentDB } from 'agentic-flow/agentdb';
import { ReasoningBank } from 'agentic-flow/reasoningbank';
import { ruvLLM } from 'ruvector/ruvLLM';

// Initialize systems
const contentVectors = new RuVector({ dimensions: 1536, indexType: 'hnsw' });
const preferenceVectors = new RuVector({ dimensions: 1536, indexType: 'hnsw' });
const agentDB = new AgentDB({ persistPath: './data/streamsense' });
const reasoningBank = new ReasoningBank(agentDB);

// Main discovery flow
async function discover(userId: string, query: string, context: ContextInput): Promise<DiscoveryResult> {
  // 1. Embed query
  const queryEmbedding = await ruvLLM.embed(query);

  // 2. Get user preference
  const prefResult = await preferenceVectors.get(`user:${userId}:preferences`);
  const preferenceVector = prefResult?.vector ?? new Float32Array(1536);

  // 3. Combine query + preference
  const combinedVector = weightedAverage(queryEmbedding, preferenceVector, 0.6, 0.4);

  // 4. Semantic search
  const candidates = await contentVectors.search({
    vector: combinedVector,
    topK: 50
  });

  // 5. Re-rank with Q-values
  const stateHash = hashState({ userId, context });
  const qTableManager = new QTableManager(agentDB);

  const ranked = await Promise.all(
    candidates.map(async (c) => {
      const qValue = await qTableManager.getQValue(stateHash, c.id);
      return {
        ...c,
        qValue,
        score: qValue * 0.5 + c.similarity * 0.5
      };
    })
  );

  ranked.sort((a, b) => b.score - a.score);

  // 6. Return top recommendations
  return {
    recommendations: ranked.slice(0, 20),
    learningMetrics: await new MetricsTracker(agentDB).calculateMetrics(userId)
  };
}

// Track viewing outcome
async function trackViewing(outcome: ViewingOutcome): Promise<void> {
  // 1. Calculate reward
  const reward = (outcome.completionRate / 100) * 0.7 + (outcome.explicitRating ?? 0) / 5 * 0.3;

  // 2. Create experience
  const experience: Experience = {
    userId: outcome.userId,
    stateHash: hashState(outcome.stateBefore),
    action: outcome.contentId,
    reward,
    nextStateHash: hashState(outcome.stateAfter),
    timestamp: Date.now()
  };

  // 3. Update Q-value
  const qTableManager = new QTableManager(agentDB);
  const currentQ = await qTableManager.getQValue(experience.stateHash, experience.action);
  const nextBest = await qTableManager.getBestAction(experience.nextStateHash);
  const maxNextQ = nextBest?.qValue ?? 0;

  const newQ = currentQ + 0.1 * (reward + 0.95 * maxNextQ - currentQ);
  await qTableManager.setQValue(experience.stateHash, experience.action, newQ);

  // 4. Update preference vector
  const prefResult = await preferenceVectors.get(`user:${outcome.userId}:preferences`);
  const currentPref = prefResult?.vector ?? new Float32Array(1536);

  const contentResult = await contentVectors.get(`content:${outcome.contentId}`);
  const contentVector = contentResult.vector;

  const alpha = 0.1 * Math.abs(reward);
  const direction = reward > 0 ? 1 : -1;
  const updatedPref = new Float32Array(1536);

  for (let i = 0; i < 1536; i++) {
    updatedPref[i] = currentPref[i] + (contentVector[i] - currentPref[i]) * alpha * direction;
  }

  await preferenceVectors.upsert({
    id: `user:${outcome.userId}:preferences`,
    vector: updatedPref
  });

  // 5. Add to replay buffer
  await new ReplayBuffer(agentDB).addExperience(experience);

  // 6. Track trajectory
  await reasoningBank.addTrajectory(experience);
}
```

---

**End of StreamSense AI PRD**
