# EmotiStream Nexus MVP - Project Structure Architecture

**SPARC Phase**: 3 - Architecture
**Version**: 1.0
**Last Updated**: 2025-12-05
**Status**: ✅ Ready for Implementation

---

## Table of Contents

1. [Directory Structure](#1-directory-structure)
2. [Core TypeScript Interfaces](#2-core-typescript-interfaces)
3. [Module Contracts](#3-module-contracts)
4. [Configuration Structure](#4-configuration-structure)
5. [Dependency Injection Pattern](#5-dependency-injection-pattern)
6. [Error Handling Architecture](#6-error-handling-architecture)
7. [Testing Strategy](#7-testing-strategy)
8. [Build and Development Workflow](#8-build-and-development-workflow)

---

## 1. Directory Structure

```
emotistream-mvp/
├── src/
│   ├── types/                      # Shared TypeScript interfaces
│   │   ├── index.ts                # Re-exports all types
│   │   ├── emotional-state.ts      # EmotionalState, DesiredState
│   │   ├── content.ts              # ContentMetadata, EmotionalContentProfile
│   │   ├── experience.ts           # EmotionalExperience, QTableEntry
│   │   ├── recommendation.ts       # Recommendation, RankedContent
│   │   ├── user.ts                 # UserProfile, UserSession
│   │   └── api.ts                  # API request/response types
│   │
│   ├── emotion/                    # EmotionDetector module
│   │   ├── index.ts                # Exports EmotionDetector class
│   │   ├── detector.ts             # Main emotion detection logic
│   │   ├── gemini-client.ts        # Gemini API wrapper
│   │   ├── emotion-mapper.ts       # Maps Gemini output to EmotionalState
│   │   ├── desired-state-predictor.ts # Predicts desired emotional state
│   │   └── utils.ts                # Emotion vector utilities
│   │
│   ├── rl/                         # RLPolicyEngine module
│   │   ├── index.ts                # Exports RLPolicyEngine class
│   │   ├── policy-engine.ts        # Main Q-learning policy engine
│   │   ├── q-table.ts              # Q-value storage and retrieval
│   │   ├── state-hasher.ts         # State discretization (5×5×3 buckets)
│   │   ├── exploration.ts          # ε-greedy and UCB exploration
│   │   ├── reward.ts               # Reward function calculation
│   │   └── experience-replay.ts    # Experience buffer management
│   │
│   ├── content/                    # ContentProfiler module
│   │   ├── index.ts                # Exports ContentProfiler class
│   │   ├── profiler.ts             # Content emotional profiling
│   │   ├── catalog.ts              # Mock content catalog (200 items)
│   │   ├── embedding-generator.ts  # Generate 1536D embeddings
│   │   └── target-state-matcher.ts # Match content to emotional states
│   │
│   ├── recommendations/            # RecommendationEngine module
│   │   ├── index.ts                # Exports RecommendationEngine class
│   │   ├── engine.ts               # Hybrid ranking (Q + semantic)
│   │   ├── fusion.ts               # Q-value (70%) + similarity (30%)
│   │   ├── transition-vector.ts    # Create emotional transition embeddings
│   │   └── ranker.ts               # Final ranking and filtering
│   │
│   ├── feedback/                   # FeedbackReward module
│   │   ├── index.ts                # Exports FeedbackManager class
│   │   ├── feedback-manager.ts     # Post-viewing feedback processing
│   │   ├── reward-calculator.ts    # Reward function implementation
│   │   ├── q-updater.ts            # Q-value TD updates
│   │   └── profile-updater.ts      # User profile synchronization
│   │
│   ├── db/                         # Storage layer
│   │   ├── index.ts                # Exports DB clients
│   │   ├── agentdb-client.ts       # AgentDB wrapper
│   │   ├── ruvector-client.ts      # RuVector wrapper
│   │   ├── keys.ts                 # AgentDB key patterns
│   │   └── migrations.ts           # Data migration utilities
│   │
│   ├── api/                        # REST API layer
│   │   ├── index.ts                # Express app export
│   │   ├── server.ts               # Express server setup
│   │   ├── routes/
│   │   │   ├── index.ts            # Route aggregator
│   │   │   ├── emotion.routes.ts   # POST /emotion/detect
│   │   │   ├── recommend.routes.ts # POST /recommend
│   │   │   ├── feedback.routes.ts  # POST /feedback
│   │   │   ├── insights.routes.ts  # GET /insights/:userId
│   │   │   └── health.routes.ts    # GET /health
│   │   ├── middleware/
│   │   │   ├── error-handler.ts    # Global error handler
│   │   │   ├── validator.ts        # Request validation (Zod)
│   │   │   ├── logger.ts           # Request logging
│   │   │   └── rate-limiter.ts     # Rate limiting
│   │   └── controllers/
│   │       ├── emotion.controller.ts
│   │       ├── recommend.controller.ts
│   │       ├── feedback.controller.ts
│   │       └── insights.controller.ts
│   │
│   ├── cli/                        # CLI demo interface
│   │   ├── index.ts                # CLI entry point
│   │   ├── demo.ts                 # Interactive demo flow
│   │   ├── prompts.ts              # Inquirer.js prompts
│   │   ├── display.ts              # Chalk visualization
│   │   └── demo-script.ts          # Pre-configured demo scenarios
│   │
│   ├── config/                     # Configuration
│   │   ├── index.ts                # Config aggregator
│   │   ├── env.ts                  # Environment variable loader
│   │   ├── hyperparameters.ts      # RL hyperparameters
│   │   └── constants.ts            # Application constants
│   │
│   ├── utils/                      # Shared utilities
│   │   ├── logger.ts               # Logging utility
│   │   ├── validators.ts           # Common validators
│   │   ├── math.ts                 # Math utilities (cosine similarity, etc.)
│   │   └── time.ts                 # Time utilities
│   │
│   └── index.ts                    # Main application entry point
│
├── tests/                          # Test files
│   ├── unit/
│   │   ├── emotion/
│   │   │   ├── detector.test.ts
│   │   │   ├── emotion-mapper.test.ts
│   │   │   └── desired-state-predictor.test.ts
│   │   ├── rl/
│   │   │   ├── policy-engine.test.ts
│   │   │   ├── reward.test.ts
│   │   │   ├── state-hasher.test.ts
│   │   │   └── exploration.test.ts
│   │   ├── content/
│   │   │   ├── profiler.test.ts
│   │   │   └── embedding-generator.test.ts
│   │   ├── recommendations/
│   │   │   ├── engine.test.ts
│   │   │   └── fusion.test.ts
│   │   └── feedback/
│   │       ├── reward-calculator.test.ts
│   │       └── q-updater.test.ts
│   ├── integration/
│   │   ├── api/
│   │   │   ├── emotion.integration.test.ts
│   │   │   ├── recommend.integration.test.ts
│   │   │   └── feedback.integration.test.ts
│   │   ├── db/
│   │   │   ├── agentdb.integration.test.ts
│   │   │   └── ruvector.integration.test.ts
│   │   └── end-to-end/
│   │       └── full-flow.e2e.test.ts
│   ├── fixtures/
│   │   ├── emotional-states.json
│   │   ├── content-catalog.json
│   │   └── test-users.json
│   └── helpers/
│       ├── setup.ts                # Test environment setup
│       ├── teardown.ts             # Test cleanup
│       └── mocks.ts                # Mock data generators
│
├── scripts/                        # Build and setup scripts
│   ├── setup-catalog.ts            # Initialize mock content catalog
│   ├── profile-content.ts          # Batch profile content emotions
│   ├── seed-demo-data.ts           # Seed demo user data
│   ├── init-db.ts                  # Initialize AgentDB
│   ├── init-vector.ts              # Initialize RuVector index
│   └── reset-data.ts               # Reset all data (dev only)
│
├── data/                           # Runtime data (gitignored)
│   ├── content-catalog.json        # Mock content (200 items)
│   ├── emotistream.db              # AgentDB SQLite (created at runtime)
│   └── content-embeddings.idx      # RuVector HNSW index
│
├── docs/                           # Documentation
│   ├── API.md                      # API documentation
│   ├── DEMO.md                     # Demo script
│   ├── ARCHITECTURE.md             # This document
│   └── DEPLOYMENT.md               # Deployment guide
│
├── .env.example                    # Environment variables template
├── .gitignore                      # Git ignore patterns
├── package.json                    # Node.js dependencies
├── tsconfig.json                   # TypeScript configuration
├── jest.config.js                  # Jest test configuration
├── README.md                       # Project README
└── LICENSE                         # MIT License
```

---

## 2. Core TypeScript Interfaces

### 2.1 Emotional State Types (`src/types/emotional-state.ts`)

```typescript
/**
 * Core emotional state representation based on Russell's Circumplex Model
 * and Plutchik's 8 basic emotions.
 */
export interface EmotionalState {
  // Identifiers
  emotionalStateId: string;           // UUID
  userId: string;                     // User identifier

  // Russell's Circumplex (2D emotional space)
  valence: number;                    // -1 (negative) to +1 (positive)
  arousal: number;                    // -1 (calm) to +1 (excited)

  // Plutchik's 8 basic emotions (one-hot encoded)
  emotionVector: Float32Array;        // [joy, sadness, anger, fear, trust, disgust, surprise, anticipation]
  primaryEmotion: EmotionLabel;       // Dominant emotion

  // Derived metrics
  stressLevel: number;                // 0-1 (derived from valence/arousal)
  confidence: number;                 // 0-1 (detection confidence)

  // Context (for state discretization)
  context: EmotionalContext;

  // Desired outcome (predicted or explicit)
  desiredValence: number;             // -1 to +1
  desiredArousal: number;             // -1 to +1
  desiredStateConfidence: number;     // 0-1 (confidence in prediction)

  // Metadata
  timestamp: number;                  // Unix timestamp (ms)
  detectionSource: DetectionSource;   // "text" | "voice" | "biometric"
}

export type EmotionLabel =
  | "joy"
  | "sadness"
  | "anger"
  | "fear"
  | "trust"
  | "disgust"
  | "surprise"
  | "anticipation"
  | "neutral";

export interface EmotionalContext {
  dayOfWeek: number;                  // 0-6 (Sunday=0)
  hourOfDay: number;                  // 0-23
  socialContext: SocialContext;       // "solo" | "partner" | "family" | "friends"
}

export type SocialContext = "solo" | "partner" | "family" | "friends";

export type DetectionSource = "text" | "voice" | "biometric" | "explicit";

/**
 * Desired emotional state (prediction or explicit goal)
 */
export interface DesiredState {
  valence: number;                    // -1 to +1
  arousal: number;                    // -1 to +1
  confidence: number;                 // 0-1 (confidence in prediction)
  reasoning: string;                  // Explanation for predicted state
}
```

---

### 2.2 Content Types (`src/types/content.ts`)

```typescript
/**
 * Content metadata (movies, shows, music, etc.)
 */
export interface ContentMetadata {
  contentId: string;                  // Unique content identifier
  title: string;
  description: string;
  platform: Platform;                 // "mock" | "youtube" | "netflix" | "prime"
  genres: string[];                   // ["nature", "relaxation", "comedy"]

  // Content categorization
  category: ContentCategory;
  tags: string[];                     // ["feel-good", "nature", "slow-paced"]

  // Duration
  duration: number;                   // Seconds

  // Timestamps
  createdAt: number;                  // Unix timestamp (ms)
  lastProfiledAt?: number;            // Last emotional profiling timestamp
}

export type Platform = "mock" | "youtube" | "netflix" | "prime" | "spotify";

export type ContentCategory =
  | "movie"
  | "series"
  | "documentary"
  | "music"
  | "meditation"
  | "short";

/**
 * Emotional profile of content (learned from Gemini + user experiences)
 */
export interface EmotionalContentProfile {
  contentId: string;

  // Emotional characteristics (from Gemini analysis)
  primaryTone: string;                // "calm", "uplifting", "thrilling", "melancholic"
  valenceDelta: number;               // Expected change in valence (-1 to +1)
  arousalDelta: number;               // Expected change in arousal (-1 to +1)
  intensity: number;                  // 0-1 (subtle to intense)
  complexity: number;                 // 0-1 (simple to nuanced emotions)

  // Target emotional states (which states is this content good for?)
  targetStates: TargetEmotionalState[];

  // Embedding
  embeddingId: string;                // RuVector embedding ID

  // Learned effectiveness (updated with each experience)
  avgEmotionalImprovement: number;    // Average reward received
  sampleSize: number;                 // Number of experiences
}

export interface TargetEmotionalState {
  currentValence: number;             // -1 to +1
  currentArousal: number;             // -1 to +1
  description: string;                // "stressed and anxious"
}
```

---

### 2.3 Experience and Q-Learning Types (`src/types/experience.ts`)

```typescript
/**
 * Emotional experience for RL training (SARS: State-Action-Reward-State')
 */
export interface EmotionalExperience {
  experienceId: string;               // UUID
  userId: string;

  // RL experience components
  stateBefore: EmotionalState;        // Initial emotional state (S)
  contentId: string;                  // Action taken (A)
  stateAfter: EmotionalState;         // Resulting emotional state (S')
  desiredState: DesiredState;         // Goal state

  // Reward
  reward: number;                     // -1 to +1 (RL reward signal)

  // Optional explicit feedback
  explicitRating?: number;            // 1-5 star rating
  explicitEmoji?: string;             // Emoji feedback

  // Viewing details
  viewingDetails?: ViewingDetails;

  // Metadata
  timestamp: number;                  // Unix timestamp (ms)
}

export interface ViewingDetails {
  completionRate: number;             // 0-1 (% of content watched)
  durationSeconds: number;            // Actual viewing time
  interactions?: number;              // Pauses, rewinds, etc.
}

/**
 * Q-table entry (state-action-value)
 */
export interface QTableEntry {
  userId: string;
  stateHash: string;                  // Discretized state hash
  contentId: string;                  // Action (content ID)
  qValue: number;                     // Q-value (0-1, higher = better)
  visitCount: number;                 // Number of visits (for UCB)
  lastUpdated: number;                // Unix timestamp (ms)
}

/**
 * State hash format: "v:a:s:c"
 * - v: valence bucket (0-4)
 * - a: arousal bucket (0-4)
 * - s: stress bucket (0-2)
 * - c: social context ("solo", "partner", "family", "friends")
 */
export type StateHash = string;       // e.g., "2:3:1:solo"
```

---

### 2.4 Recommendation Types (`src/types/recommendation.ts`)

```typescript
/**
 * Recommendation result with emotional predictions
 */
export interface Recommendation {
  // Content details
  contentId: string;
  title: string;
  platform: string;
  duration: number;

  // Emotional profile
  emotionalProfile: EmotionalContentProfile;

  // Predicted outcome
  predictedOutcome: PredictedOutcome;

  // RL metadata
  qValue: number;                     // Q-value (0-1)
  confidence: number;                 // Overall confidence (0-1)
  explorationFlag: boolean;           // Was this from exploration?

  // Ranking
  rank: number;                       // Position in recommendation list (1 = best)
  score: number;                      // Combined score (Q × 0.7 + similarity × 0.3)

  // Explanation
  reasoning: string;                  // Human-readable explanation
}

export interface PredictedOutcome {
  postViewingValence: number;         // Predicted valence after viewing
  postViewingArousal: number;         // Predicted arousal after viewing
  expectedImprovement: number;        // Expected reward (0-1)
  confidence: number;                 // Confidence in prediction (0-1)
}

/**
 * Ranked content list (with hybrid scoring)
 */
export interface RankedContent {
  contentId: string;
  qScore: number;                     // Q-value component (0-1)
  similarityScore: number;            // Semantic similarity component (0-1)
  totalScore: number;                 // Weighted sum (Q × 0.7 + sim × 0.3)
}
```

---

### 2.5 User Types (`src/types/user.ts`)

```typescript
/**
 * User profile with RL learning metrics
 */
export interface UserProfile {
  userId: string;
  email: string;
  displayName: string;

  // Emotional baseline
  emotionalBaseline: EmotionalBaseline;

  // Learning metrics
  totalExperiences: number;           // Total content viewing experiences
  avgReward: number;                  // Average RL reward (0-1)
  explorationRate: number;            // Current ε-greedy exploration rate

  // Timestamps
  createdAt: number;                  // Unix timestamp (ms)
  lastActive: number;                 // Unix timestamp (ms)
}

export interface EmotionalBaseline {
  avgValence: number;                 // Average valence over all sessions
  avgArousal: number;                 // Average arousal
  variability: number;                // Emotional variability (std dev)
}

/**
 * User session state (in-memory only)
 */
export interface UserSession {
  userId: string;
  currentEmotionalState?: EmotionalState;
  lastRecommendationTime?: number;
  activeExperienceId?: string;
  sessionStartTime: number;
}
```

---

### 2.6 API Types (`src/types/api.ts`)

```typescript
/**
 * Standard API response wrapper
 */
export interface ApiResponse<T = unknown> {
  success: boolean;
  data: T | null;
  error: ApiError | null;
  timestamp: string;                  // ISO 8601
}

export interface ApiError {
  code: ErrorCode;
  message: string;
  details?: Record<string, unknown>;
  fallback?: unknown;                 // Fallback response if available
}

export type ErrorCode =
  | "E001"  // GEMINI_TIMEOUT
  | "E002"  // GEMINI_RATE_LIMIT
  | "E003"  // INVALID_INPUT
  | "E004"  // USER_NOT_FOUND
  | "E005"  // CONTENT_NOT_FOUND
  | "E006"  // RL_POLICY_ERROR
  | "E007"  // AUTH_INVALID_TOKEN
  | "E008"  // AUTH_UNAUTHORIZED
  | "E009"  // RATE_LIMIT_EXCEEDED
  | "E010"; // INTERNAL_ERROR

/**
 * API request types
 */
export interface EmotionDetectionRequest {
  userId: string;
  text: string;
  context?: Partial<EmotionalContext>;
}

export interface RecommendationRequest {
  userId: string;
  emotionalStateId: string;
  limit?: number;                     // Default: 20
  explicitDesiredState?: Partial<DesiredState>;
}

export interface FeedbackRequest {
  userId: string;
  contentId: string;
  emotionalStateId: string;
  postViewingState: PostViewingFeedback;
  viewingDetails?: ViewingDetails;
}

export interface PostViewingFeedback {
  text?: string;                      // Post-viewing text input
  explicitRating?: number;            // 1-5
  explicitEmoji?: string;             // Emoji feedback
}
```

---

## 3. Module Contracts

### 3.1 EmotionDetector (`src/emotion/`)

**Public Interface:**

```typescript
export class EmotionDetector {
  constructor(
    private geminiClient: GeminiClient,
    private agentDB: AgentDBClient
  ) {}

  /**
   * Analyze text input to extract emotional state
   * @param userId - User identifier
   * @param text - Text input from user
   * @param context - Optional emotional context
   * @returns Emotional state with predicted desired state
   * @throws GeminiTimeoutError, GeminiRateLimitError
   */
  async analyzeText(
    userId: string,
    text: string,
    context?: Partial<EmotionalContext>
  ): Promise<EmotionalState>;

  /**
   * Predict desired emotional state based on current state
   * @param userId - User identifier
   * @param currentState - Current emotional state
   * @returns Predicted desired state with confidence
   */
  async predictDesiredState(
    userId: string,
    currentState: EmotionalState
  ): Promise<DesiredState>;
}
```

**Dependencies:**
- `GeminiClient` (internal) - Gemini API wrapper
- `AgentDBClient` - Store emotional history
- `EmotionMapper` (internal) - Maps Gemini output to `EmotionalState`

**Error Types:**
- `GeminiTimeoutError` - Gemini API timeout (30s)
- `GeminiRateLimitError` - Rate limit exceeded
- `InvalidInputError` - Empty or invalid text input

---

### 3.2 RLPolicyEngine (`src/rl/`)

**Public Interface:**

```typescript
export class RLPolicyEngine {
  constructor(
    private agentDB: AgentDBClient,
    private config: RLConfig
  ) {}

  /**
   * Select content action using ε-greedy Q-learning policy
   * @param userId - User identifier
   * @param emotionalState - Current emotional state
   * @param desiredState - Desired emotional state
   * @param availableContent - Content IDs to choose from
   * @returns Selected content ID with Q-value
   */
  async selectAction(
    userId: string,
    emotionalState: EmotionalState,
    desiredState: DesiredState,
    availableContent: string[]
  ): Promise<{ contentId: string; qValue: number; isExploration: boolean }>;

  /**
   * Update Q-value based on experience (TD update)
   * @param experience - Emotional experience with reward
   */
  async updatePolicy(experience: EmotionalExperience): Promise<void>;

  /**
   * Get Q-value for state-action pair
   * @param userId - User identifier
   * @param stateHash - Discretized state hash
   * @param contentId - Content ID (action)
   * @returns Q-value (0 if not found)
   */
  async getQValue(
    userId: string,
    stateHash: StateHash,
    contentId: string
  ): Promise<number>;
}
```

**Dependencies:**
- `AgentDBClient` - Q-table storage
- `StateHasher` (internal) - State discretization (5×5×3 buckets)
- `ExplorationStrategy` (internal) - ε-greedy + UCB

**Configuration:**
```typescript
export interface RLConfig {
  learningRate: number;               // α (default: 0.1)
  discountFactor: number;             // γ (default: 0.95)
  explorationRate: number;            // ε (default: 0.15)
  explorationDecay: number;           // ε decay per episode (default: 0.95)
  ucbConstant: number;                // c for UCB exploration (default: 2.0)
}
```

---

### 3.3 ContentProfiler (`src/content/`)

**Public Interface:**

```typescript
export class ContentProfiler {
  constructor(
    private geminiClient: GeminiClient,
    private ruVectorClient: RuVectorClient,
    private agentDB: AgentDBClient
  ) {}

  /**
   * Profile content emotional characteristics using Gemini
   * @param content - Content metadata
   * @returns Emotional content profile
   */
  async profileContent(
    content: ContentMetadata
  ): Promise<EmotionalContentProfile>;

  /**
   * Batch profile multiple content items (for catalog initialization)
   * @param contents - Array of content metadata
   * @param batchSize - Items per batch (default: 10)
   * @returns Array of emotional profiles
   */
  async batchProfile(
    contents: ContentMetadata[],
    batchSize?: number
  ): Promise<EmotionalContentProfile[]>;

  /**
   * Load mock content catalog (200 items)
   * @returns Array of content metadata
   */
  async loadMockCatalog(): Promise<ContentMetadata[]>;
}
```

**Dependencies:**
- `GeminiClient` - Content emotional analysis
- `RuVectorClient` - Store 1536D embeddings
- `AgentDBClient` - Store content profiles
- `EmbeddingGenerator` (internal) - Generate embeddings from Gemini output

---

### 3.4 RecommendationEngine (`src/recommendations/`)

**Public Interface:**

```typescript
export class RecommendationEngine {
  constructor(
    private rlPolicyEngine: RLPolicyEngine,
    private ruVectorClient: RuVectorClient,
    private agentDB: AgentDBClient
  ) {}

  /**
   * Generate content recommendations using hybrid ranking
   * @param userId - User identifier
   * @param emotionalState - Current emotional state
   * @param desiredState - Desired emotional state
   * @param limit - Number of recommendations (default: 20)
   * @returns Ranked recommendations
   */
  async recommend(
    userId: string,
    emotionalState: EmotionalState,
    desiredState: DesiredState,
    limit?: number
  ): Promise<Recommendation[]>;

  /**
   * Search content by emotional transition using RuVector
   * @param emotionalState - Current state
   * @param desiredState - Desired state
   * @param topK - Number of candidates (default: 50)
   * @returns Content IDs with similarity scores
   */
  async searchByEmotionalTransition(
    emotionalState: EmotionalState,
    desiredState: DesiredState,
    topK?: number
  ): Promise<RankedContent[]>;
}
```

**Dependencies:**
- `RLPolicyEngine` - Q-values for re-ranking
- `RuVectorClient` - Semantic search
- `AgentDBClient` - Content metadata
- `TransitionVectorGenerator` (internal) - Create transition embeddings
- `HybridRanker` (internal) - Fusion algorithm (Q × 0.7 + sim × 0.3)

---

### 3.5 FeedbackManager (`src/feedback/`)

**Public Interface:**

```typescript
export class FeedbackManager {
  constructor(
    private emotionDetector: EmotionDetector,
    private rlPolicyEngine: RLPolicyEngine,
    private agentDB: AgentDBClient,
    private rewardCalculator: RewardCalculator
  ) {}

  /**
   * Process post-viewing feedback and update RL policy
   * @param userId - User identifier
   * @param contentId - Content that was viewed
   * @param emotionalStateId - Pre-viewing emotional state ID
   * @param feedback - Post-viewing feedback
   * @returns Experience with reward and updated Q-value
   */
  async processFeedback(
    userId: string,
    contentId: string,
    emotionalStateId: string,
    feedback: PostViewingFeedback,
    viewingDetails?: ViewingDetails
  ): Promise<{
    experienceId: string;
    reward: number;
    emotionalImprovement: number;
    qValueBefore: number;
    qValueAfter: number;
  }>;

  /**
   * Calculate reward based on emotional state change
   * @param stateBefore - Pre-viewing emotional state
   * @param stateAfter - Post-viewing emotional state
   * @param desiredState - Desired emotional state
   * @returns Reward value (-1 to +1)
   */
  calculateReward(
    stateBefore: EmotionalState,
    stateAfter: EmotionalState,
    desiredState: DesiredState
  ): number;
}
```

**Dependencies:**
- `EmotionDetector` - Detect post-viewing emotional state
- `RLPolicyEngine` - Update Q-values
- `AgentDBClient` - Store experiences
- `RewardCalculator` (internal) - Reward function implementation

---

### 3.6 Storage Layer (`src/db/`)

**AgentDB Client:**

```typescript
export class AgentDBClient {
  constructor(private config: AgentDBConfig) {}

  async init(): Promise<void>;

  // User operations
  async getUser(userId: string): Promise<UserProfile | null>;
  async setUser(userId: string, profile: UserProfile): Promise<void>;

  // Emotional state operations
  async getEmotionalState(stateId: string): Promise<EmotionalState | null>;
  async setEmotionalState(state: EmotionalState): Promise<void>;
  async getUserEmotionalHistory(userId: string, limit?: number): Promise<EmotionalState[]>;

  // Q-table operations
  async getQValue(userId: string, stateHash: StateHash, contentId: string): Promise<number>;
  async setQValue(userId: string, stateHash: StateHash, contentId: string, qValue: number): Promise<void>;
  async getAllQValues(userId: string, stateHash: StateHash): Promise<Map<string, number>>;

  // Experience operations
  async addExperience(experience: EmotionalExperience): Promise<void>;
  async getUserExperiences(userId: string, limit?: number): Promise<EmotionalExperience[]>;

  // Content operations
  async getContent(contentId: string): Promise<ContentMetadata | null>;
  async setContent(content: ContentMetadata): Promise<void>;

  // Visit count (for UCB exploration)
  async incrementVisitCount(userId: string, contentId: string): Promise<number>;
  async getVisitCount(userId: string, contentId: string): Promise<number>;
  async getTotalActions(userId: string): Promise<number>;
}
```

**RuVector Client:**

```typescript
export class RuVectorClient {
  constructor(private config: RuVectorConfig) {}

  async init(): Promise<void>;

  // Collection operations
  async createCollection(name: string, dimension: number): Promise<void>;

  // Embedding operations
  async upsertEmbedding(
    collection: string,
    id: string,
    vector: Float32Array,
    metadata?: Record<string, unknown>
  ): Promise<void>;

  async batchUpsertEmbeddings(
    collection: string,
    embeddings: Array<{
      id: string;
      vector: Float32Array;
      metadata?: Record<string, unknown>;
    }>
  ): Promise<void>;

  // Search operations
  async search(
    collection: string,
    queryVector: Float32Array,
    topK: number,
    filter?: Record<string, unknown>
  ): Promise<Array<{ id: string; score: number; metadata?: Record<string, unknown> }>>;

  // Utility operations
  async getEmbedding(collection: string, id: string): Promise<Float32Array | null>;
  async deleteEmbedding(collection: string, id: string): Promise<void>;
}
```

---

## 4. Configuration Structure

### 4.1 Environment Variables (`src/config/env.ts`)

```typescript
import { z } from "zod";
import dotenv from "dotenv";

dotenv.config();

const envSchema = z.object({
  // Server
  NODE_ENV: z.enum(["development", "production", "test"]).default("development"),
  PORT: z.string().default("3000"),

  // Gemini API
  GEMINI_API_KEY: z.string().min(1, "GEMINI_API_KEY is required"),
  GEMINI_MODEL: z.string().default("gemini-2.0-flash-exp"),
  GEMINI_TIMEOUT_MS: z.string().default("30000"),

  // Storage
  AGENTDB_PATH: z.string().default("./data/emotistream.db"),
  RUVECTOR_INDEX_PATH: z.string().default("./data/content-embeddings.idx"),

  // RL Hyperparameters
  RL_LEARNING_RATE: z.string().default("0.1"),
  RL_DISCOUNT_FACTOR: z.string().default("0.95"),
  RL_EXPLORATION_RATE: z.string().default("0.15"),
  RL_EXPLORATION_DECAY: z.string().default("0.95"),
  RL_UCB_CONSTANT: z.string().default("2.0"),

  // API
  API_RATE_LIMIT: z.string().default("100"),
  API_RATE_WINDOW_MS: z.string().default("60000"),

  // Logging
  LOG_LEVEL: z.enum(["debug", "info", "warn", "error"]).default("info"),
});

export type Env = z.infer<typeof envSchema>;

export const env = envSchema.parse(process.env);
```

---

### 4.2 Hyperparameters (`src/config/hyperparameters.ts`)

```typescript
import { env } from "./env.js";

export interface RLHyperparameters {
  learningRate: number;               // α
  discountFactor: number;             // γ
  explorationRate: number;            // ε
  explorationDecay: number;           // ε decay
  ucbConstant: number;                // c for UCB
  minExplorationRate: number;         // Minimum ε
  maxExplorationRate: number;         // Maximum ε
}

export const rlHyperparameters: RLHyperparameters = {
  learningRate: parseFloat(env.RL_LEARNING_RATE),
  discountFactor: parseFloat(env.RL_DISCOUNT_FACTOR),
  explorationRate: parseFloat(env.RL_EXPLORATION_RATE),
  explorationDecay: parseFloat(env.RL_EXPLORATION_DECAY),
  ucbConstant: parseFloat(env.RL_UCB_CONSTANT),
  minExplorationRate: 0.05,
  maxExplorationRate: 0.5,
};

export interface StateDiscretization {
  valenceBuckets: number;             // 5 buckets
  arousalBuckets: number;             // 5 buckets
  stressBuckets: number;              // 3 buckets
  valenceBucketSize: number;          // 0.4
  arousalBucketSize: number;          // 0.4
  stressBucketSize: number;           // 0.33
}

export const stateDiscretization: StateDiscretization = {
  valenceBuckets: 5,
  arousalBuckets: 5,
  stressBuckets: 3,
  valenceBucketSize: 0.4,
  arousalBucketSize: 0.4,
  stressBucketSize: 0.33,
};

export interface RewardWeights {
  directionAlignment: number;         // 0.6
  magnitudeImprovement: number;       // 0.4
  proximityBonus: number;             // 0.2
  proximityThreshold: number;         // 0.3
}

export const rewardWeights: RewardWeights = {
  directionAlignment: 0.6,
  magnitudeImprovement: 0.4,
  proximityBonus: 0.2,
  proximityThreshold: 0.3,
};

export interface HybridRankingWeights {
  qValueWeight: number;               // 0.7
  similarityWeight: number;           // 0.3
}

export const hybridRankingWeights: HybridRankingWeights = {
  qValueWeight: 0.7,
  similarityWeight: 0.3,
};
```

---

### 4.3 Constants (`src/config/constants.ts`)

```typescript
export const EMOTION_LABELS = [
  "joy",
  "sadness",
  "anger",
  "fear",
  "trust",
  "disgust",
  "surprise",
  "anticipation",
] as const;

export const SOCIAL_CONTEXTS = ["solo", "partner", "family", "friends"] as const;

export const PLATFORMS = ["mock", "youtube", "netflix", "prime", "spotify"] as const;

export const CONTENT_CATEGORIES = [
  "movie",
  "series",
  "documentary",
  "music",
  "meditation",
  "short",
] as const;

export const VECTOR_DIMENSIONS = 1536;  // Gemini embedding size

export const HNSW_PARAMS = {
  M: 16,
  efConstruction: 200,
  efSearch: 50,
};

export const RECOMMENDATION_LIMITS = {
  default: 20,
  min: 1,
  max: 50,
};

export const EXPERIENCE_REPLAY_BUFFER_SIZE = 1000;
export const EXPERIENCE_TTL_DAYS = 90;

export const TIMEOUT_MS = {
  gemini: 30000,
  database: 5000,
  vectorSearch: 10000,
};
```

---

## 5. Dependency Injection Pattern

### 5.1 Container Setup (`src/di-container.ts`)

```typescript
import { Container } from "inversify";
import { AgentDBClient } from "./db/agentdb-client.js";
import { RuVectorClient } from "./db/ruvector-client.js";
import { GeminiClient } from "./emotion/gemini-client.js";
import { EmotionDetector } from "./emotion/detector.js";
import { RLPolicyEngine } from "./rl/policy-engine.js";
import { ContentProfiler } from "./content/profiler.js";
import { RecommendationEngine } from "./recommendations/engine.js";
import { FeedbackManager } from "./feedback/feedback-manager.js";
import { env } from "./config/env.js";
import { rlHyperparameters } from "./config/hyperparameters.js";

export const container = new Container();

// Bind storage clients
container.bind(AgentDBClient).toSelf().inSingletonScope();
container.bind(RuVectorClient).toSelf().inSingletonScope();

// Bind external API clients
container.bind(GeminiClient).toSelf().inSingletonScope();

// Bind core modules
container.bind(EmotionDetector).toSelf().inSingletonScope();
container.bind(RLPolicyEngine).toSelf().inSingletonScope();
container.bind(ContentProfiler).toSelf().inSingletonScope();
container.bind(RecommendationEngine).toSelf().inSingletonScope();
container.bind(FeedbackManager).toSelf().inSingletonScope();

// Initialize all clients
export async function initializeContainer(): Promise<void> {
  const agentDB = container.get(AgentDBClient);
  await agentDB.init();

  const ruVector = container.get(RuVectorClient);
  await ruVector.init();

  console.log("✅ Dependency injection container initialized");
}

// Cleanup
export async function cleanupContainer(): Promise<void> {
  // Close database connections
  const agentDB = container.get(AgentDBClient);
  await agentDB.close();

  const ruVector = container.get(RuVectorClient);
  await ruVector.close();

  console.log("✅ Dependency injection container cleaned up");
}
```

---

### 5.2 Usage in Controllers (`src/api/controllers/recommend.controller.ts`)

```typescript
import { Request, Response, NextFunction } from "express";
import { container } from "../../di-container.js";
import { RecommendationEngine } from "../../recommendations/engine.js";
import { AgentDBClient } from "../../db/agentdb-client.js";
import { RecommendationRequest } from "../../types/api.js";
import { validateRecommendationRequest } from "../middleware/validator.js";

export class RecommendController {
  private recommendationEngine: RecommendationEngine;
  private agentDB: AgentDBClient;

  constructor() {
    this.recommendationEngine = container.get(RecommendationEngine);
    this.agentDB = container.get(AgentDBClient);
  }

  async getRecommendations(
    req: Request,
    res: Response,
    next: NextFunction
  ): Promise<void> {
    try {
      const request = validateRecommendationRequest(req.body);

      // Load emotional state
      const emotionalState = await this.agentDB.getEmotionalState(
        request.emotionalStateId
      );

      if (!emotionalState) {
        res.status(404).json({
          success: false,
          data: null,
          error: { code: "E004", message: "Emotional state not found" },
          timestamp: new Date().toISOString(),
        });
        return;
      }

      // Generate recommendations
      const recommendations = await this.recommendationEngine.recommend(
        request.userId,
        emotionalState,
        {
          valence: emotionalState.desiredValence,
          arousal: emotionalState.desiredArousal,
          confidence: emotionalState.desiredStateConfidence,
          reasoning: "Predicted from current state",
        },
        request.limit
      );

      res.json({
        success: true,
        data: { recommendations },
        error: null,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      next(error);
    }
  }
}
```

---

## 6. Error Handling Architecture

### 6.1 Custom Error Types (`src/utils/errors.ts`)

```typescript
export class EmotiStreamError extends Error {
  constructor(
    public code: string,
    message: string,
    public statusCode: number = 500,
    public details?: Record<string, unknown>
  ) {
    super(message);
    this.name = this.constructor.name;
    Error.captureStackTrace(this, this.constructor);
  }
}

export class GeminiTimeoutError extends EmotiStreamError {
  constructor(details?: Record<string, unknown>) {
    super("E001", "Gemini API timeout", 504, details);
  }
}

export class GeminiRateLimitError extends EmotiStreamError {
  constructor(details?: Record<string, unknown>) {
    super("E002", "Gemini rate limit exceeded", 429, details);
  }
}

export class InvalidInputError extends EmotiStreamError {
  constructor(message: string, details?: Record<string, unknown>) {
    super("E003", message, 400, details);
  }
}

export class UserNotFoundError extends EmotiStreamError {
  constructor(userId: string) {
    super("E004", `User not found: ${userId}`, 404, { userId });
  }
}

export class ContentNotFoundError extends EmotiStreamError {
  constructor(contentId: string) {
    super("E005", `Content not found: ${contentId}`, 404, { contentId });
  }
}

export class RLPolicyError extends EmotiStreamError {
  constructor(message: string, details?: Record<string, unknown>) {
    super("E006", `RL policy error: ${message}`, 500, details);
  }
}
```

---

### 6.2 Global Error Handler (`src/api/middleware/error-handler.ts`)

```typescript
import { Request, Response, NextFunction } from "express";
import { EmotiStreamError, GeminiTimeoutError } from "../../utils/errors.js";
import { ApiResponse } from "../../types/api.js";
import { logger } from "../../utils/logger.js";

export function errorHandler(
  error: Error,
  req: Request,
  res: Response,
  next: NextFunction
): void {
  logger.error("Error occurred", {
    error: error.message,
    stack: error.stack,
    path: req.path,
  });

  if (error instanceof EmotiStreamError) {
    const response: ApiResponse = {
      success: false,
      data: null,
      error: {
        code: error.code,
        message: error.message,
        details: error.details,
      },
      timestamp: new Date().toISOString(),
    };

    // Add fallback for certain errors
    if (error instanceof GeminiTimeoutError) {
      response.error!.fallback = {
        emotionalState: {
          valence: 0,
          arousal: 0,
          confidence: 0.3,
          primaryEmotion: "neutral",
        },
        message: "Emotion detection temporarily unavailable",
      };
    }

    res.status(error.statusCode).json(response);
  } else {
    // Unexpected error
    const response: ApiResponse = {
      success: false,
      data: null,
      error: {
        code: "E010",
        message: "Internal server error",
      },
      timestamp: new Date().toISOString(),
    };

    res.status(500).json(response);
  }
}
```

---

## 7. Testing Strategy

### 7.1 Unit Testing Setup (`tests/unit/`)

**Jest Configuration (`jest.config.js`):**

```javascript
export default {
  preset: "ts-jest/presets/default-esm",
  testEnvironment: "node",
  extensionsToTreatAsEsm: [".ts"],
  moduleNameMapper: {
    "^(\\.{1,2}/.*)\\.js$": "$1",
  },
  transform: {
    "^.+\\.tsx?$": [
      "ts-jest",
      {
        useESM: true,
      },
    ],
  },
  testMatch: ["**/tests/**/*.test.ts"],
  collectCoverageFrom: ["src/**/*.ts", "!src/**/*.d.ts"],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80,
    },
  },
};
```

**Example Unit Test (`tests/unit/rl/reward.test.ts`):**

```typescript
import { describe, it, expect } from "@jest/globals";
import { RewardCalculator } from "../../../src/feedback/reward-calculator.js";
import { EmotionalState, DesiredState } from "../../../src/types/index.js";

describe("RewardCalculator", () => {
  const calculator = new RewardCalculator();

  describe("calculateReward", () => {
    it("should give positive reward for improvement toward desired state", () => {
      const before: EmotionalState = {
        valence: -0.6,
        arousal: 0.5,
        stressLevel: 0.8,
        // ... other fields
      };

      const after: EmotionalState = {
        valence: 0.4,
        arousal: -0.2,
        stressLevel: 0.3,
        // ... other fields
      };

      const desired: DesiredState = {
        valence: 0.5,
        arousal: -0.3,
        confidence: 0.7,
        reasoning: "Test",
      };

      const reward = calculator.calculateReward(before, after, desired);

      expect(reward).toBeGreaterThan(0.7);
      expect(reward).toBeLessThanOrEqual(1.0);
    });

    it("should give negative reward for movement away from desired state", () => {
      const before: EmotionalState = {
        valence: 0.2,
        arousal: 0.1,
        stressLevel: 0.3,
      };

      const after: EmotionalState = {
        valence: -0.5,
        arousal: 0.6,
        stressLevel: 0.8,
      };

      const desired: DesiredState = {
        valence: 0.6,
        arousal: -0.3,
        confidence: 0.7,
        reasoning: "Test",
      };

      const reward = calculator.calculateReward(before, after, desired);

      expect(reward).toBeLessThan(0);
    });

    it("should apply proximity bonus when within threshold", () => {
      const before: EmotionalState = {
        valence: -0.3,
        arousal: 0.2,
        stressLevel: 0.5,
      };

      const after: EmotionalState = {
        valence: 0.48,
        arousal: -0.28,
        stressLevel: 0.2,
      };

      const desired: DesiredState = {
        valence: 0.5,
        arousal: -0.3,
        confidence: 0.8,
        reasoning: "Test",
      };

      const reward = calculator.calculateReward(before, after, desired);

      // Should include proximity bonus
      expect(reward).toBeGreaterThan(0.8);
    });
  });
});
```

---

### 7.2 Integration Testing (`tests/integration/`)

**Example Integration Test (`tests/integration/api/recommend.integration.test.ts`):**

```typescript
import { describe, it, expect, beforeAll, afterAll } from "@jest/globals";
import request from "supertest";
import { app } from "../../../src/api/index.js";
import { container, initializeContainer, cleanupContainer } from "../../../src/di-container.js";
import { AgentDBClient } from "../../../src/db/agentdb-client.js";

describe("POST /api/recommend - Integration", () => {
  let agentDB: AgentDBClient;

  beforeAll(async () => {
    await initializeContainer();
    agentDB = container.get(AgentDBClient);

    // Seed test data
    await seedTestData(agentDB);
  });

  afterAll(async () => {
    await cleanupContainer();
  });

  it("should return recommendations for valid request", async () => {
    const response = await request(app)
      .post("/api/recommend")
      .send({
        userId: "test-user-001",
        emotionalStateId: "test-state-001",
        limit: 5,
      });

    expect(response.status).toBe(200);
    expect(response.body.success).toBe(true);
    expect(response.body.data.recommendations).toHaveLength(5);
    expect(response.body.data.recommendations[0]).toMatchObject({
      contentId: expect.any(String),
      qValue: expect.any(Number),
      rank: 1,
    });
  });

  it("should return 404 for non-existent emotional state", async () => {
    const response = await request(app)
      .post("/api/recommend")
      .send({
        userId: "test-user-001",
        emotionalStateId: "non-existent",
        limit: 5,
      });

    expect(response.status).toBe(404);
    expect(response.body.success).toBe(false);
    expect(response.body.error.code).toBe("E004");
  });
});

async function seedTestData(agentDB: AgentDBClient): Promise<void> {
  // Seed test user, emotional states, Q-values, etc.
  // ...
}
```

---

## 8. Build and Development Workflow

### 8.1 Package.json Scripts

```json
{
  "name": "emotistream-mvp",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "tsx watch src/index.ts",
    "build": "tsc",
    "start": "node dist/index.js",
    "cli": "tsx src/cli/index.ts",
    "test": "node --experimental-vm-modules node_modules/jest/bin/jest.js",
    "test:watch": "node --experimental-vm-modules node_modules/jest/bin/jest.js --watch",
    "test:coverage": "node --experimental-vm-modules node_modules/jest/bin/jest.js --coverage",
    "lint": "eslint src --ext .ts",
    "format": "prettier --write \"src/**/*.ts\"",
    "typecheck": "tsc --noEmit",
    "setup": "tsx scripts/setup-catalog.ts && tsx scripts/init-db.ts && tsx scripts/init-vector.ts",
    "profile-content": "tsx scripts/profile-content.ts",
    "seed-demo": "tsx scripts/seed-demo-data.ts",
    "reset": "tsx scripts/reset-data.ts"
  },
  "dependencies": {
    "@google/generative-ai": "^0.21.0",
    "express": "^4.19.2",
    "agentdb": "latest",
    "ruvector": "latest",
    "zod": "^3.22.4",
    "dotenv": "^16.4.5",
    "inquirer": "^9.2.12",
    "chalk": "^5.3.0",
    "ora": "^8.0.1",
    "inversify": "^6.0.2",
    "reflect-metadata": "^0.2.1"
  },
  "devDependencies": {
    "@types/express": "^4.17.21",
    "@types/node": "^20.11.5",
    "@types/inquirer": "^9.0.7",
    "typescript": "^5.3.3",
    "tsx": "^4.7.0",
    "@jest/globals": "^29.7.0",
    "jest": "^29.7.0",
    "ts-jest": "^29.1.2",
    "supertest": "^6.3.4",
    "@types/supertest": "^6.0.2",
    "eslint": "^8.56.0",
    "@typescript-eslint/eslint-plugin": "^6.19.0",
    "@typescript-eslint/parser": "^6.19.0",
    "prettier": "^3.2.4"
  }
}
```

---

### 8.2 TypeScript Configuration (`tsconfig.json`)

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ES2022",
    "lib": ["ES2022"],
    "moduleResolution": "node",
    "resolveJsonModule": true,
    "allowJs": true,
    "outDir": "./dist",
    "rootDir": "./src",
    "removeComments": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "strictPropertyInitialization": true,
    "noImplicitThis": true,
    "alwaysStrict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "skipLibCheck": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "experimentalDecorators": true,
    "emitDecoratorMetadata": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "tests"]
}
```

---

### 8.3 Development Workflow

**Step 1: Setup Environment**
```bash
# Clone repository
git clone <repo-url>
cd emotistream-mvp

# Install dependencies
npm install

# Create .env file
cp .env.example .env
# Add GEMINI_API_KEY to .env

# Initialize databases and content catalog
npm run setup
```

**Step 2: Development**
```bash
# Start development server with hot reload
npm run dev

# In another terminal, run CLI demo
npm run cli
```

**Step 3: Testing**
```bash
# Run all tests
npm test

# Run tests with coverage
npm test:coverage

# Run tests in watch mode
npm run test:watch

# Type checking
npm run typecheck

# Linting
npm run lint
```

**Step 4: Build for Production**
```bash
# Build TypeScript to JavaScript
npm run build

# Start production server
npm start
```

---

## Summary

This project structure provides:

1. **Clear Module Boundaries** - Each module has well-defined responsibilities and interfaces
2. **Type Safety** - Comprehensive TypeScript interfaces for all data structures
3. **Dependency Injection** - Loose coupling via container-based DI
4. **Error Handling** - Structured error types with fallback behaviors
5. **Testing** - Unit and integration tests with high coverage targets
6. **Configuration** - Environment-based configuration with validation
7. **Scalability** - Modular architecture allows for easy extension

**Next Steps:**
1. Implement core modules following these interfaces
2. Write unit tests for each module
3. Build integration tests for API endpoints
4. Create CLI demo interface
5. Profile content catalog
6. Test end-to-end RL learning loop

---

**Status**: ✅ Architecture complete, ready for SPARC Phase 4 (Refinement/Implementation)
