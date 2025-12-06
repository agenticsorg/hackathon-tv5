# EmotiStream RL Policy Engine - Architecture Specification

**Component**: Reinforcement Learning Policy Engine
**Version**: 1.0.0
**Date**: 2025-12-05
**SPARC Phase**: 3 - Architecture
**Dependencies**:
- MVP-001 (Emotion Detection)
- MVP-003 (Content Profiling)
- AgentDB (Q-table persistence)
- RuVector (Semantic search)

---

## Table of Contents

1. [Overview](#overview)
2. [Module Structure](#module-structure)
3. [Component Architecture](#component-architecture)
4. [Class Diagrams](#class-diagrams)
5. [Interface Specifications](#interface-specifications)
6. [Sequence Diagrams](#sequence-diagrams)
7. [State Space Design](#state-space-design)
8. [Data Architecture](#data-architecture)
9. [Integration Points](#integration-points)
10. [Hyperparameter Configuration](#hyperparameter-configuration)
11. [Performance Considerations](#performance-considerations)
12. [Security & Privacy](#security--privacy)
13. [Monitoring & Observability](#monitoring--observability)
14. [Deployment Architecture](#deployment-architecture)
15. [Testing Strategy](#testing-strategy)

---

## 1. Overview

### 1.1 Purpose

The **RLPolicyEngine** is the core machine learning component of EmotiStream Nexus, responsible for learning optimal content recommendation policies through reinforcement learning. It implements **Q-learning with temporal difference (TD) updates**, enabling the system to discover which content produces the best emotional outcomes for each user.

### 1.2 Key Capabilities

- **Action Selection**: Choose content recommendations via ε-greedy exploration
- **Policy Learning**: Update Q-values through TD-learning from emotional experiences
- **State Discretization**: Map continuous emotional states to discrete buckets (5×5×3)
- **Exploration Strategy**: Balance exploitation (learned Q-values) with exploration (UCB bonuses)
- **Experience Replay**: Sample past experiences for improved learning efficiency
- **Convergence Monitoring**: Track policy convergence through TD error analysis

### 1.3 Architecture Principles

1. **Separation of Concerns**: Distinct modules for Q-learning, exploration, reward calculation
2. **Persistence-First**: All Q-values immediately persisted to AgentDB
3. **Stateless Design**: No in-memory state; all state in AgentDB/RuVector
4. **Fail-Safe Defaults**: Return neutral recommendations on errors
5. **Observable Learning**: Extensive logging for debugging and monitoring

---

## 2. Module Structure

### 2.1 Directory Layout

```
src/rl/
├── index.ts                    # Public exports
├── policy-engine.ts            # RLPolicyEngine class (main)
├── q-table.ts                  # Q-table management
├── reward-calculator.ts        # Reward function
├── exploration/
│   ├── epsilon-greedy.ts       # ε-greedy strategy
│   ├── ucb.ts                  # UCB bonus calculation
│   └── exploration-strategy.ts # Strategy interface
├── replay-buffer.ts            # Experience replay
├── state-hasher.ts             # State discretization (5×5×3)
├── convergence-monitor.ts      # Policy convergence detection
├── types.ts                    # Module-specific types
└── __tests__/
    ├── policy-engine.test.ts
    ├── q-table.test.ts
    ├── reward-calculator.test.ts
    ├── state-hasher.test.ts
    └── integration.test.ts
```

### 2.2 File Responsibilities

| File | Responsibility | Lines | Complexity |
|------|----------------|-------|------------|
| `policy-engine.ts` | Main orchestration, action selection, policy updates | 300 | High |
| `q-table.ts` | AgentDB Q-value CRUD operations | 150 | Medium |
| `reward-calculator.ts` | Emotional reward computation (cosine similarity) | 100 | Medium |
| `epsilon-greedy.ts` | ε-greedy exploration with decay | 80 | Low |
| `ucb.ts` | Upper Confidence Bound calculations | 100 | Medium |
| `replay-buffer.ts` | Circular buffer for experience replay | 120 | Medium |
| `state-hasher.ts` | Continuous → discrete state mapping | 60 | Low |
| `convergence-monitor.ts` | TD error tracking, convergence detection | 100 | Medium |
| `types.ts` | TypeScript interfaces and types | 150 | Low |

### 2.3 Dependencies

```typescript
// External dependencies
import AgentDB from '@ruvnet/agentdb';
import RuVectorClient from '@ruvnet/ruvector';

// Internal dependencies
import { EmotionalState, DesiredState } from '../emotion/types';
import { ContentMetadata } from '../content/types';
import { Logger } from '../utils/logger';
```

---

## 3. Component Architecture

### 3.1 High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     RLPolicyEngine Module                        │
└─────────────────────────────────────────────────────────────────┘
                                │
                ┌───────────────┴───────────────┐
                ▼                               ▼
    ┌───────────────────────┐       ┌───────────────────────┐
    │   RLPolicyEngine      │       │   QTableManager       │
    │   (Main Class)        │◄──────│   (Persistence)       │
    │                       │       │                       │
    │ • selectAction()      │       │ • getQValue()         │
    │ • updatePolicy()      │       │ • setQValue()         │
    │ • exploit()           │       │ • getMaxQValue()      │
    │ • explore()           │       │ • getTotalVisits()    │
    └───────────────────────┘       └───────────────────────┘
            │       │                        │
            │       │                        ▼
            │       │              ┌───────────────────────┐
            │       │              │      AgentDB          │
            │       │              │   (Redis Backend)     │
            │       │              │                       │
            │       │              │ • Key-value store     │
            │       │              │ • Q-table persistence │
            │       │              │ • 90-day TTL          │
            │       │              └───────────────────────┘
            │       │
            │       └──────────────┐
            │                      ▼
            │            ┌───────────────────────┐
            │            │  ExplorationStrategy  │
            │            │   (Strategy Pattern)  │
            │            │                       │
            │            │ • EpsilonGreedy       │
            │            │ • UCBCalculator       │
            │            └───────────────────────┘
            │
            ▼
┌───────────────────────┐       ┌───────────────────────┐
│  RewardCalculator     │       │   StateHasher         │
│                       │       │                       │
│ • calculateReward()   │       │ • hashState()         │
│ • direction alignment │       │ • 5×5×3 buckets       │
│ • magnitude scoring   │       │ • clampToRange()      │
│ • proximity bonus     │       └───────────────────────┘
│ • stress penalty      │
└───────────────────────┘

            │
            ▼
┌───────────────────────┐       ┌───────────────────────┐
│   ReplayBuffer        │       │ ConvergenceMonitor    │
│                       │       │                       │
│ • addExperience()     │       │ • trackTDError()      │
│ • sampleBatch()       │       │ • checkConvergence()  │
│ • circular buffer     │       │ • meanAbsError()      │
│ • size: 1000          │       │ • stdDeviation()      │
└───────────────────────┘       └───────────────────────┘

            │
            ▼
┌───────────────────────┐
│    RuVectorClient     │
│   (Semantic Search)   │
│                       │
│ • search()            │
│ • 1536D embeddings    │
│ • HNSW indexing       │
└───────────────────────┘
```

### 3.2 Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Action Selection Flow                       │
└─────────────────────────────────────────────────────────────────┘

User Emotion Input
    │
    ▼
┌──────────────────┐
│  EmotionalState  │──┐
│  - valence       │  │
│  - arousal       │  │
│  - stress        │  │
└──────────────────┘  │
                      │
    ┌─────────────────┘
    │
    ▼
┌──────────────────┐    ┌──────────────────┐
│  StateHasher     │───▶│  stateHash       │
│  discretize()    │    │  "2:3:1"         │
└──────────────────┘    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ EpsilonGreedy    │
                    │ decide explore?  │
                    └──────────────────┘
                         │         │
        ┌────────────────┘         └────────────────┐
        │                                           │
        ▼ (exploit)                                 ▼ (explore)
┌──────────────────┐                      ┌──────────────────┐
│  RuVector        │                      │  RuVector        │
│  semantic search │                      │  semantic search │
└──────────────────┘                      └──────────────────┘
        │                                           │
        ▼                                           ▼
┌──────────────────┐                      ┌──────────────────┐
│  QTableManager   │                      │  UCBCalculator   │
│  getQValue()     │                      │  compute bonus   │
│  rank by Q       │                      │  select action   │
└──────────────────┘                      └──────────────────┘
        │                                           │
        └────────────────┬──────────────────────────┘
                         ▼
                ┌──────────────────┐
                │ ContentRecommend │
                │ - contentId      │
                │ - qValue         │
                │ - isExploration  │
                └──────────────────┘


┌─────────────────────────────────────────────────────────────────┐
│                      Policy Update Flow                          │
└─────────────────────────────────────────────────────────────────┘

Post-Viewing Feedback
    │
    ▼
┌──────────────────┐
│ EmotionalState   │
│ (before/after)   │
└──────────────────┘
    │
    ▼
┌──────────────────┐    ┌──────────────────┐
│ RewardCalculator │───▶│   reward         │
│ cosine similarity│    │   0.72           │
└──────────────────┘    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ QTableManager    │
                    │ getCurrentQ()    │
                    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ TD Learning      │
                    │ Q ← Q + α[r +    │
                    │   γ·max(Q') - Q] │
                    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ QTableManager    │
                    │ setQValue(newQ)  │
                    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ AgentDB          │
                    │ persist Q-value  │
                    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ ReplayBuffer     │
                    │ addExperience()  │
                    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ EpsilonGreedy    │
                    │ decayEpsilon()   │
                    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ ConvergenceMonit │
                    │ trackTDError()   │
                    └──────────────────┘
```

---

## 4. Class Diagrams

### 4.1 RLPolicyEngine Class (Core)

```
┌────────────────────────────────────────────────────────────────┐
│                        RLPolicyEngine                           │
├────────────────────────────────────────────────────────────────┤
│ - qTableManager: QTableManager                                 │
│ - rewardCalculator: RewardCalculator                           │
│ - explorationStrategy: IExplorationStrategy                    │
│ - replayBuffer: ReplayBuffer                                   │
│ - stateHasher: StateHasher                                     │
│ - convergenceMonitor: ConvergenceMonitor                       │
│ - ruVector: RuVectorClient                                     │
│ - logger: Logger                                               │
│ - learningRate: number = 0.1                                   │
│ - discountFactor: number = 0.95                                │
├────────────────────────────────────────────────────────────────┤
│ + constructor(agentDB, ruVector, logger)                       │
│                                                                │
│ # Action Selection                                             │
│ + selectAction(userId, state, desired, candidates)            │
│   → Promise<ActionSelection>                                   │
│ - exploit(userId, stateHash, candidates)                       │
│   → Promise<ContentRecommendation>                             │
│ - explore(userId, stateHash, candidates)                       │
│   → Promise<ContentRecommendation>                             │
│                                                                │
│ # Policy Learning                                              │
│ + updatePolicy(userId, experience)                             │
│   → Promise<PolicyUpdate>                                      │
│ - calculateTDTarget(reward, maxNextQ)                          │
│   → number                                                     │
│ - calculateTDError(target, currentQ)                           │
│   → number                                                     │
│                                                                │
│ # Q-Value Operations                                           │
│ - getQValue(userId, stateHash, contentId)                      │
│   → Promise<number>                                            │
│ - setQValue(userId, stateHash, contentId, value)               │
│   → Promise<void>                                              │
│ - getMaxQValue(userId, stateHash)                              │
│   → Promise<number>                                            │
│                                                                │
│ # Utilities                                                    │
│ - createTransitionVector(current, desired)                     │
│   → Float32Array                                               │
│ - logActionSelection(...)                                      │
│   → void                                                       │
│ - logPolicyUpdate(...)                                         │
│   → void                                                       │
└────────────────────────────────────────────────────────────────┘
```

### 4.2 QTableManager Class

```
┌────────────────────────────────────────────────────────────────┐
│                        QTableManager                            │
├────────────────────────────────────────────────────────────────┤
│ - agentDB: AgentDB                                             │
│ - ttl: number = 90 * 24 * 60 * 60  // 90 days                 │
│ - keyPrefix: string = "qtable"                                 │
├────────────────────────────────────────────────────────────────┤
│ + constructor(agentDB: AgentDB)                                │
│                                                                │
│ # Q-Value CRUD                                                 │
│ + getQValue(userId, stateHash, contentId)                      │
│   → Promise<number>                                            │
│ + setQValue(userId, stateHash, contentId, value)               │
│   → Promise<void>                                              │
│ + getQTableEntry(userId, stateHash, contentId)                 │
│   → Promise<QTableEntry | null>                                │
│ + setQTableEntry(entry: QTableEntry)                           │
│   → Promise<void>                                              │
│                                                                │
│ # Batch Operations                                             │
│ + getMaxQValue(userId, stateHash)                              │
│   → Promise<number>                                            │
│ + getTotalStateVisits(userId, stateHash)                       │
│   → Promise<number>                                            │
│ + getAllQValues(userId, stateHash)                             │
│   → Promise<Map<string, number>>                               │
│                                                                │
│ # Key Management                                               │
│ - buildKey(userId, stateHash, contentId)                       │
│   → string                                                     │
│ - parseKey(key: string)                                        │
│   → { userId, stateHash, contentId }                           │
│ - buildQueryMetadata(userId, stateHash)                        │
│   → object                                                     │
└────────────────────────────────────────────────────────────────┘
```

### 4.3 ExplorationStrategy Interface

```
┌────────────────────────────────────────────────────────────────┐
│                   IExplorationStrategy                          │
│                      «interface»                                │
├────────────────────────────────────────────────────────────────┤
│ + shouldExplore(userId: string): Promise<boolean>              │
│ + selectExplorationAction(                                     │
│     userId: string,                                            │
│     stateHash: string,                                         │
│     candidates: ContentCandidate[]                             │
│   ): Promise<ContentRecommendation>                            │
│ + decayExplorationRate(userId: string): Promise<void>          │
│ + getExplorationRate(userId: string): Promise<number>          │
└────────────────────────────────────────────────────────────────┘
                            △
                            │ implements
            ┌───────────────┴───────────────┐
            │                               │
┌───────────────────────┐       ┌───────────────────────┐
│  EpsilonGreedyStrategy│       │   UCBExploration      │
├───────────────────────┤       ├───────────────────────┤
│ - initialEpsilon: 0.15│       │ - ucbConstant: 2.0    │
│ - minEpsilon: 0.10    │       │                       │
│ - decayRate: 0.95     │       │ + calculateUCB(...)   │
├───────────────────────┤       │   → number            │
│ + shouldExplore(...)  │       │ + selectWithUCB(...)  │
│   → Promise<boolean>  │       │   → ContentRecommend  │
└───────────────────────┘       └───────────────────────┘
```

### 4.4 RewardCalculator Class

```
┌────────────────────────────────────────────────────────────────┐
│                      RewardCalculator                           │
├────────────────────────────────────────────────────────────────┤
│ - directionWeight: number = 0.6                                │
│ - magnitudeWeight: number = 0.4                                │
│ - proximityThreshold: number = 0.15                            │
│ - proximityBonus: number = 0.2                                 │
│ - stressThreshold: number = 0.2                                │
│ - stressPenalty: number = -0.15                                │
├────────────────────────────────────────────────────────────────┤
│ + calculateReward(                                             │
│     stateBefore: EmotionalState,                               │
│     stateAfter: EmotionalState,                                │
│     desired: DesiredState                                      │
│   ): number                                                    │
│                                                                │
│ - calculateDirectionAlignment(actual, desired)                 │
│   → number                                                     │
│ - calculateMagnitudeScore(actual, desired)                     │
│   → number                                                     │
│ - calculateProximityBonus(stateAfter, desired)                 │
│   → number                                                     │
│ - calculateStressPenalty(stateBefore, stateAfter)              │
│   → number                                                     │
│ - cosineSimilarity(vec1, vec2)                                 │
│   → number                                                     │
│ - vectorMagnitude(vec)                                         │
│   → number                                                     │
│ - euclideanDistance(point1, point2)                            │
│   → number                                                     │
│ - clampReward(reward: number)                                  │
│   → number                                                     │
└────────────────────────────────────────────────────────────────┘
```

### 4.5 StateHasher Class

```
┌────────────────────────────────────────────────────────────────┐
│                         StateHasher                             │
├────────────────────────────────────────────────────────────────┤
│ - valenceBuckets: number = 5                                   │
│ - arousalBuckets: number = 5                                   │
│ - stressBuckets: number = 3                                    │
├────────────────────────────────────────────────────────────────┤
│ + hashState(state: EmotionalState): string                     │
│   → "2:3:1" format                                             │
│                                                                │
│ - discretizeValence(valence: number): number                   │
│   → [0, 4]                                                     │
│ - discretizeArousal(arousal: number): number                   │
│   → [0, 4]                                                     │
│ - discretizeStress(stress: number): number                     │
│   → [0, 2]                                                     │
│ - clampToBucket(value, buckets): number                        │
│   → number                                                     │
│                                                                │
│ + unhashState(stateHash: string): EmotionalStateBucket         │
│   → { valenceBucket, arousalBucket, stressBucket }             │
│ + getStateSpaceSize(): number                                  │
│   → 75                                                         │
└────────────────────────────────────────────────────────────────┘
```

---

## 5. Interface Specifications

### 5.1 Core TypeScript Interfaces

```typescript
/**
 * Main RL Policy Engine interface
 */
export interface IRLPolicyEngine {
  /**
   * Select best action (content recommendation) for current emotional state
   * Uses ε-greedy exploration strategy
   */
  selectAction(
    userId: string,
    state: EmotionalState,
    desired: DesiredState,
    candidates: string[]
  ): Promise<ActionSelection>;

  /**
   * Update Q-values based on emotional experience feedback
   * Implements TD-learning update rule
   */
  updatePolicy(
    userId: string,
    experience: EmotionalExperience
  ): Promise<PolicyUpdate>;

  /**
   * Get Q-value for specific state-action pair
   */
  getQValue(
    userId: string,
    stateHash: string,
    contentId: string
  ): Promise<number>;
}

/**
 * Action selection result with metadata
 */
export interface ActionSelection {
  contentId: string;           // Selected content ID
  qValue: number;              // Q-value for this state-action pair
  isExploration: boolean;      // True if exploration action
  explorationBonus: number;    // UCB bonus (if exploring)
  confidence: number;          // Confidence [0, 1] based on visit count
  reasoning: string;           // Human-readable explanation
  stateHash: string;           // Discretized state ("2:3:1")
  candidateCount: number;      // Number of candidates considered
  timestamp: number;           // Selection timestamp
}

/**
 * Policy update result with learning metrics
 */
export interface PolicyUpdate {
  userId: string;
  stateHash: string;
  contentId: string;

  // Q-learning metrics
  oldQValue: number;
  newQValue: number;
  tdError: number;
  reward: number;

  // Visit tracking
  visitCount: number;
  totalStateVisits: number;

  // Exploration
  explorationRate: number;

  // Convergence
  convergenceStatus: ConvergenceStatus;

  timestamp: number;
}

/**
 * Q-table entry stored in AgentDB
 */
export interface QTableEntry {
  userId: string;
  stateHash: string;           // "v:a:s" format (e.g., "2:3:1")
  contentId: string;
  qValue: number;              // Expected cumulative reward
  visitCount: number;          // Times this (s,a) pair visited
  lastUpdated: number;         // Timestamp of last update
  createdAt: number;           // Timestamp of creation

  // Optional metadata for analysis
  metadata?: {
    averageReward?: number;
    rewardVariance?: number;
    successRate?: number;
  };
}

/**
 * Emotional experience for RL learning
 */
export interface EmotionalExperience {
  experienceId: string;
  userId: string;

  // Before viewing
  stateBefore: EmotionalState;
  desiredState: DesiredState;

  // Content
  contentId: string;

  // After viewing
  stateAfter: EmotionalState;

  // Reward
  reward: number;

  // Metadata
  timestamp: number;
  duration?: number;           // Viewing duration (seconds)
  explicitRating?: number;     // User rating 1-5 (optional)
}

/**
 * Emotional state (continuous values)
 */
export interface EmotionalState {
  valence: number;             // [-1.0, 1.0] negative to positive
  arousal: number;             // [-1.0, 1.0] calm to excited
  stress: number;              // [0.0, 1.0] relaxed to stressed
  confidence: number;          // [0.0, 1.0] prediction confidence
  primaryEmotion?: string;     // joy, sadness, anger, etc.
  timestamp?: number;
}

/**
 * Desired emotional state (target)
 */
export interface DesiredState {
  valence: number;             // Target valence [-1.0, 1.0]
  arousal: number;             // Target arousal [-1.0, 1.0]
  confidence: number;          // Prediction confidence [0.0, 1.0]
  reasoning?: string;          // Why this state was predicted
}

/**
 * Content recommendation with RL metadata
 */
export interface ContentRecommendation {
  contentId: string;
  title: string;
  description?: string;

  // RL metrics
  qValue: number;
  explorationBonus: number;
  isExploration: boolean;
  confidence: number;

  // Content metadata
  expectedValenceDelta?: number;
  expectedArousalDelta?: number;
  intensity?: number;

  reasoning: string;
}

/**
 * Convergence monitoring status
 */
export interface ConvergenceStatus {
  hasConverged: boolean;
  meanAbsTDError: number;      // Mean absolute TD error
  tdErrorStdDev: number;        // Standard deviation of TD errors
  totalUpdates: number;         // Total policy updates
  recentUpdates: number;        // Updates in analysis window
  message: string;              // Human-readable status
}

/**
 * User RL statistics
 */
export interface UserRLStats {
  userId: string;
  episodeCount: number;         // Total episodes completed
  currentEpsilon: number;       // Current exploration rate
  totalReward: number;          // Cumulative reward
  meanReward: number;           // Average reward per episode
  lastUpdated: number;          // Last update timestamp

  // Convergence tracking
  convergenceMetrics?: {
    recentTDErrors: number[];
    qValueVariance: number;
  };
}
```

### 5.2 Configuration Interfaces

```typescript
/**
 * RL hyperparameter configuration
 */
export interface RLConfig {
  // Q-learning parameters
  learningRate: number;        // α (alpha) default: 0.1
  discountFactor: number;      // γ (gamma) default: 0.95

  // Exploration parameters
  initialExplorationRate: number;  // ε₀ default: 0.15
  minExplorationRate: number;      // ε_min default: 0.10
  explorationDecay: number;        // default: 0.95
  ucbConstant: number;             // c default: 2.0

  // State discretization
  valenceBuckets: number;          // default: 5
  arousalBuckets: number;          // default: 5
  stressBuckets: number;           // default: 3

  // Replay buffer
  replayBufferSize: number;        // default: 1000
  batchSize: number;               // default: 32

  // Convergence detection
  convergenceWindow: number;       // default: 100
  convergenceTDErrorThreshold: number;  // default: 0.05
  convergenceStdDevThreshold: number;   // default: 0.1
  minUpdatesForConvergence: number;     // default: 200

  // Reward function weights
  rewardDirectionWeight: number;   // default: 0.6
  rewardMagnitudeWeight: number;   // default: 0.4
  rewardProximityBonus: number;    // default: 0.2
  rewardStressPenalty: number;     // default: -0.15
}
```

---

## 6. Sequence Diagrams

### 6.1 Action Selection (Exploitation Path)

```
┌──────┐   ┌──────────────┐   ┌──────────┐   ┌────────┐   ┌────────┐
│Client│   │RLPolicyEngine│   │StateHasher│  │RuVector│   │QTable  │
└──┬───┘   └──────┬───────┘   └─────┬────┘   └───┬────┘   └───┬────┘
   │              │                  │             │            │
   │selectAction()│                  │             │            │
   │─────────────>│                  │             │            │
   │              │                  │             │            │
   │              │ hashState(state) │             │            │
   │              │─────────────────>│             │            │
   │              │                  │             │            │
   │              │   "2:3:1"        │             │            │
   │              │<─────────────────│             │            │
   │              │                  │             │            │
   │              │ shouldExplore(userId)          │            │
   │              │───────────────────────────────>│            │
   │              │                                │            │
   │              │   false (exploit)              │            │
   │              │<───────────────────────────────│            │
   │              │                                │            │
   │              │ createTransitionVector(state, desired)      │
   │              │────────────────────────────────>            │
   │              │                                │            │
   │              │ search(vector, topK=20)        │            │
   │              │────────────────────────────────>            │
   │              │                                │            │
   │              │ candidates[20]                 │            │
   │              │<────────────────────────────────            │
   │              │                                │            │
   │              │ for each candidate:            │            │
   │              │   getQValue(userId, stateHash, contentId)   │
   │              │────────────────────────────────────────────>│
   │              │                                             │
   │              │   qValue                                    │
   │              │<────────────────────────────────────────────│
   │              │                                │            │
   │              │ rank by Q-value                │            │
   │              │ select best                    │            │
   │              │                  │             │            │
   │ ActionSelection                 │             │            │
   │<─────────────│                  │             │            │
   │              │                  │             │            │
```

### 6.2 Action Selection (Exploration Path with UCB)

```
┌──────┐   ┌──────────────┐   ┌──────────┐   ┌────────┐   ┌────────┐
│Client│   │RLPolicyEngine│   │StateHasher│  │RuVector│   │QTable  │
└──┬───┘   └──────┬───────┘   └─────┬────┘   └───┬────┘   └───┬────┘
   │              │                  │             │            │
   │selectAction()│                  │             │            │
   │─────────────>│                  │             │            │
   │              │                  │             │            │
   │              │ hashState(state) │             │            │
   │              │─────────────────>│             │            │
   │              │   "2:3:1"        │             │            │
   │              │<─────────────────│             │            │
   │              │                  │             │            │
   │              │ shouldExplore(userId)          │            │
   │              │───────────────────────────────>│            │
   │              │   true (explore) │             │            │
   │              │<───────────────────────────────│            │
   │              │                                │            │
   │              │ getTotalStateVisits(userId, stateHash)      │
   │              │────────────────────────────────────────────>│
   │              │   totalVisits = 45                          │
   │              │<────────────────────────────────────────────│
   │              │                                │            │
   │              │ for each candidate:            │            │
   │              │   getQValue(userId, stateHash, contentId)   │
   │              │────────────────────────────────────────────>│
   │              │   qValue, visitCount                        │
   │              │<────────────────────────────────────────────│
   │              │                                │            │
   │              │   calculateUCB(qValue, visitCount, totalVisits)
   │              │   ucb = Q + c*sqrt(ln(N)/n)    │            │
   │              │                                │            │
   │              │ select max UCB                 │            │
   │              │                  │             │            │
   │ ActionSelection                 │             │            │
   │ (isExploration=true)            │             │            │
   │<─────────────│                  │             │            │
```

### 6.3 Policy Update (TD-Learning)

```
┌──────┐   ┌──────────────┐   ┌────────────┐   ┌────────┐   ┌────────┐
│Client│   │RLPolicyEngine│   │RewardCalc  │   │QTable  │   │AgentDB │
└──┬───┘   └──────┬───────┘   └─────┬──────┘   └───┬────┘   └───┬────┘
   │              │                  │              │            │
   │updatePolicy(experience)         │              │            │
   │─────────────>│                  │              │            │
   │              │                  │              │            │
   │              │ calculateReward(stateBefore, stateAfter, desired)
   │              │─────────────────>│              │            │
   │              │                  │              │            │
   │              │ reward = 0.72    │              │            │
   │              │<─────────────────│              │            │
   │              │                  │              │            │
   │              │ hashState(stateBefore)          │            │
   │              │────────────────────────────────>│            │
   │              │   currentStateHash = "2:3:1"    │            │
   │              │<────────────────────────────────│            │
   │              │                  │              │            │
   │              │ hashState(stateAfter)           │            │
   │              │────────────────────────────────>│            │
   │              │   nextStateHash = "3:2:1"       │            │
   │              │<────────────────────────────────│            │
   │              │                  │              │            │
   │              │ getQValue(userId, currentStateHash, contentId)
   │              │────────────────────────────────────────────>│
   │              │   currentQ = 0.45                           │
   │              │<────────────────────────────────────────────│
   │              │                  │              │            │
   │              │ getMaxQValue(userId, nextStateHash)         │
   │              │────────────────────────────────────────────>│
   │              │   maxNextQ = 0.38                           │
   │              │<────────────────────────────────────────────│
   │              │                  │              │            │
   │              │ TD Update:       │              │            │
   │              │ target = r + γ·max(Q')         │            │
   │              │ target = 0.72 + 0.95*0.38 = 1.081          │
   │              │ error = 1.081 - 0.45 = 0.631   │            │
   │              │ newQ = 0.45 + 0.1*0.631 = 0.513│            │
   │              │                  │              │            │
   │              │ setQValue(userId, currentStateHash, contentId, 0.513)
   │              │────────────────────────────────────────────>│
   │              │                  │              │            │
   │              │                  │              │ persist    │
   │              │                  │              │───────────>│
   │              │                  │              │            │
   │              │ decayExplorationRate(userId)    │            │
   │              │────────────────────────────────────────────>│
   │              │                  │              │            │
   │              │ trackTDError(userId, tdError)   │            │
   │              │────────────────────────────────────────────>│
   │              │                  │              │            │
   │ PolicyUpdate │                  │              │            │
   │<─────────────│                  │              │            │
```

### 6.4 Batch Learning from Replay Buffer

```
┌──────────┐   ┌──────────────┐   ┌──────────────┐   ┌────────┐
│Scheduler │   │RLPolicyEngine│   │ReplayBuffer  │   │QTable  │
└────┬─────┘   └──────┬───────┘   └──────┬───────┘   └───┬────┘
     │                │                   │               │
     │ periodicBatchLearning()            │               │
     │───────────────>│                   │               │
     │                │                   │               │
     │                │ sampleBatch(32)   │               │
     │                │──────────────────>│               │
     │                │                   │               │
     │                │ experiences[32]   │               │
     │                │<──────────────────│               │
     │                │                   │               │
     │                │ for each experience:              │
     │                │   updatePolicy(experience)        │
     │                │──────────────────────────────────>│
     │                │   ... (TD update) ...             │
     │                │<──────────────────────────────────│
     │                │                   │               │
     │                │ (repeat 32 times) │               │
     │                │                   │               │
     │ complete       │                   │               │
     │<───────────────│                   │               │
```

---

## 7. State Space Design

### 7.1 Discretization Strategy

The RL policy uses **discrete state space** for tractability, mapping continuous emotional states to a 5×5×3 grid:

```
State Space: 5 (valence) × 5 (arousal) × 3 (stress) = 75 discrete states
```

#### Valence Discretization: [-1.0, 1.0] → [0, 4]

```
Bucket 0: [-1.0, -0.6)  Very Negative
Bucket 1: [-0.6, -0.2)  Negative
Bucket 2: [-0.2, +0.2)  Neutral
Bucket 3: [+0.2, +0.6)  Positive
Bucket 4: [+0.6, +1.0]  Very Positive

Formula: bucket = floor((valence + 1.0) / 0.4)
Clamped: min(max(bucket, 0), 4)
```

#### Arousal Discretization: [-1.0, 1.0] → [0, 4]

```
Bucket 0: [-1.0, -0.6)  Very Calm
Bucket 1: [-0.6, -0.2)  Calm
Bucket 2: [-0.2, +0.2)  Neutral
Bucket 3: [+0.2, +0.6)  Aroused
Bucket 4: [+0.6, +1.0]  Very Aroused/Excited

Formula: bucket = floor((arousal + 1.0) / 0.4)
Clamped: min(max(bucket, 0), 4)
```

#### Stress Discretization: [0.0, 1.0] → [0, 2]

```
Bucket 0: [0.0, 0.33)   Low Stress
Bucket 1: [0.33, 0.67)  Moderate Stress
Bucket 2: [0.67, 1.0]   High Stress

Formula: bucket = floor(stress / 0.34)
Clamped: min(max(bucket, 0), 2)
```

### 7.2 State Hash Format

States are encoded as strings in `"v:a:s"` format:

```typescript
// Example state hashes
"0:0:0" // Very negative, very calm, low stress (depressed)
"4:4:0" // Very positive, very aroused, low stress (euphoric)
"2:2:1" // Neutral valence/arousal, moderate stress (baseline)
"1:3:2" // Negative, aroused, high stress (anxious)
```

### 7.3 State Space Visualization

```
Valence-Arousal Grid (Stress Bucket 1 - Moderate Stress)

    Arousal
    ↑
+1  │ 0:4:1  1:4:1  2:4:1  3:4:1  4:4:1    Very Aroused
    │
+0.6│ 0:3:1  1:3:1  2:3:1  3:3:1  4:3:1    Aroused
    │
+0.2│ 0:2:1  1:2:1  2:2:1  3:2:1  4:2:1    Neutral
    │
-0.2│ 0:1:1  1:1:1  2:1:1  3:1:1  4:1:1    Calm
    │
-0.6│ 0:0:1  1:0:1  2:0:1  3:0:1  4:0:1    Very Calm
-1  │
    └──────────────────────────────────────> Valence
      -1   -0.6  -0.2  +0.2 +0.6  +1

     Very  Neg  Neut  Pos  Very
     Neg                   Pos
```

### 7.4 State Space Coverage Analysis

```typescript
// Typical user emotional state distribution (expected)
const stateDistribution = {
  // High-frequency states (60% of experiences)
  "1:1:1": 0.15,  // Slightly negative, calm, moderate stress (common)
  "2:2:1": 0.20,  // Neutral baseline (most common)
  "3:2:1": 0.12,  // Slightly positive, neutral arousal
  "2:3:2": 0.13,  // Neutral valence, aroused, high stress (work stress)

  // Medium-frequency states (30% of experiences)
  "0:1:2": 0.08,  // Very negative, calm, high stress (depression)
  "4:3:0": 0.07,  // Very positive, aroused, low stress (joy)
  "1:3:2": 0.10,  // Negative, aroused, high stress (anxiety)
  "3:1:0": 0.05,  // Positive, calm, low stress (contentment)

  // Low-frequency states (10% of experiences)
  // Extreme states: 0:0:0, 4:4:0, 0:4:2, etc.
  "other": 0.10
};
```

### 7.5 State Transition Examples

```
Stress Reduction Transition:
  Before: "1:3:2" (negative, aroused, high stress)
  After:  "2:2:1" (neutral, neutral, moderate stress)
  Direction: +valence, -arousal, -stress
  Expected Reward: +0.65

Mood Uplift Transition:
  Before: "1:1:1" (negative, calm, moderate stress)
  After:  "3:2:0" (positive, neutral, low stress)
  Direction: +valence, +arousal, -stress
  Expected Reward: +0.75

Energy Boost Transition:
  Before: "2:0:1" (neutral, very calm, moderate stress)
  After:  "3:3:0" (positive, aroused, low stress)
  Direction: +valence, +arousal, -stress
  Expected Reward: +0.70
```

---

## 8. Data Architecture

### 8.1 AgentDB Key Patterns

```typescript
// Q-Table Entry
Key:   "qtable:{userId}:{stateHash}:{contentId}"
Value: QTableEntry
TTL:   90 days
Example: "qtable:user-001:2:3:1:content-042"

// User RL Statistics
Key:   "rlstats:{userId}"
Value: UserRLStats
TTL:   None (persistent)
Example: "rlstats:user-001"

// Replay Buffer (per user)
Key:   "replay:{userId}"
Value: ReplayBuffer (circular buffer)
TTL:   30 days
Example: "replay:user-001"

// Convergence Metrics
Key:   "convergence:{userId}"
Value: ConvergenceStatus
TTL:   7 days
Example: "convergence:user-001"
```

### 8.2 AgentDB Metadata Indexing

For efficient querying, Q-table entries include metadata:

```typescript
{
  key: "qtable:user-001:2:3:1:content-042",
  value: {
    userId: "user-001",
    stateHash: "2:3:1",
    contentId: "content-042",
    qValue: 0.513,
    visitCount: 7,
    lastUpdated: 1701792000000,
    createdAt: 1701700000000
  },
  metadata: {
    userId: "user-001",
    stateHash: "2:3:1",
    contentId: "content-042",
    qValue: 0.513,
    visitCount: 7
  },
  ttl: 7776000  // 90 days in seconds
}
```

### 8.3 Query Patterns

```typescript
// Get all Q-values for a specific state
const entries = await agentDB.query({
  metadata: {
    userId: "user-001",
    stateHash: "2:3:1"
  }
}, { limit: 1000 });

// Get Q-values above threshold
const highQEntries = await agentDB.query({
  metadata: {
    userId: "user-001",
    qValue: { $gt: 0.5 }
  }
}, { limit: 100 });

// Get recently updated Q-values
const recentEntries = await agentDB.query({
  metadata: {
    userId: "user-001",
    lastUpdated: { $gt: Date.now() - 86400000 }  // Last 24h
  }
}, { limit: 100 });
```

### 8.4 RuVector Embedding Storage

```typescript
// Content emotional profile embedding
{
  id: "content-042",
  vector: Float32Array(1536),  // 1536D embedding
  metadata: {
    contentId: "content-042",
    title: "Nature Sounds: Ocean Waves",
    primaryTone: "calming",
    valenceDelta: 0.4,
    arousalDelta: -0.5,
    intensity: 0.3,
    targetStates: [
      { currentValence: -0.6, currentArousal: 0.5, description: "stressed" }
    ]
  }
}

// Semantic search query
const results = await ruVector.search({
  vector: transitionVector,  // 1536D transition vector
  topK: 20,
  filter: {
    intensity: { $lt: 0.6 }  // Not too intense
  }
});
```

---

## 9. Integration Points

### 9.1 Emotion Detection Integration

```typescript
// Input: User text → Emotion state
import { EmotionDetectionService } from '../emotion/emotion-detection';

const emotionService = new EmotionDetectionService(geminiClient);

// Get current emotional state
const emotionalState = await emotionService.analyze(userId, userText);
// → { valence: -0.6, arousal: 0.5, stress: 0.7, ... }

// Get desired state prediction
const desiredState = await emotionService.predictDesiredState(emotionalState);
// → { valence: 0.5, arousal: -0.3, confidence: 0.8, ... }

// Pass to RL engine
const recommendation = await rlEngine.selectAction(
  userId,
  emotionalState,
  desiredState,
  availableContentIds
);
```

### 9.2 Content Profiling Integration

```typescript
// Input: Content metadata → Emotional profile → RuVector embedding
import { ContentProfilingService } from '../content/content-profiling';

const profilingService = new ContentProfilingService(geminiClient, ruVector);

// Profile content batch
const contentItems = [...];  // 200 items
await profilingService.batchProfile(contentItems);

// Creates RuVector embeddings for semantic search
// RLPolicyEngine uses these embeddings for content matching
```

### 9.3 GraphQL API Integration

```typescript
// GraphQL resolvers for RL operations

const resolvers = {
  Query: {
    async getRecommendations(
      _,
      { userId, emotionText },
      { rlEngine, emotionService, contentStore }
    ) {
      // 1. Detect emotion
      const emotionalState = await emotionService.analyze(userId, emotionText);
      const desiredState = await emotionService.predictDesiredState(emotionalState);

      // 2. Get available content IDs
      const contentIds = await contentStore.getAllContentIds();

      // 3. Select action via RL
      const selection = await rlEngine.selectAction(
        userId,
        emotionalState,
        desiredState,
        contentIds
      );

      // 4. Fetch content metadata
      const content = await contentStore.getContent(selection.contentId);

      return {
        contentId: content.id,
        title: content.title,
        qValue: selection.qValue,
        confidence: selection.confidence,
        reasoning: selection.reasoning
      };
    }
  },

  Mutation: {
    async submitFeedback(
      _,
      { userId, experienceId, postViewingText },
      { rlEngine, emotionService }
    ) {
      // 1. Get stored experience
      const experience = await getExperience(experienceId);

      // 2. Analyze post-viewing emotion
      const stateAfter = await emotionService.analyze(userId, postViewingText);

      // 3. Calculate reward
      const reward = rlEngine.rewardCalculator.calculateReward(
        experience.stateBefore,
        stateAfter,
        experience.desiredState
      );

      // 4. Update policy
      const update = await rlEngine.updatePolicy(userId, {
        ...experience,
        stateAfter,
        reward
      });

      return {
        reward,
        newQValue: update.newQValue,
        message: `Reward: ${reward.toFixed(2)}. Q-value updated!`
      };
    }
  }
};
```

---

## 10. Hyperparameter Configuration

### 10.1 Default Configuration

```typescript
export const DEFAULT_RL_CONFIG: RLConfig = {
  // Q-learning parameters
  learningRate: 0.1,           // α: How much new info overrides old
  discountFactor: 0.95,        // γ: Importance of future rewards

  // Exploration parameters
  initialExplorationRate: 0.15,  // ε₀: Start with 15% exploration
  minExplorationRate: 0.10,      // ε_min: Never go below 10%
  explorationDecay: 0.95,        // Decay rate per episode
  ucbConstant: 2.0,              // c: UCB exploration bonus weight

  // State discretization
  valenceBuckets: 5,             // Valence: 5 buckets
  arousalBuckets: 5,             // Arousal: 5 buckets
  stressBuckets: 3,              // Stress: 3 buckets

  // Replay buffer
  replayBufferSize: 1000,        // Max experiences stored
  batchSize: 32,                 // Batch update size

  // Convergence detection
  convergenceWindow: 100,        // Recent TD errors to analyze
  convergenceTDErrorThreshold: 0.05,   // Mean absolute error < 0.05
  convergenceStdDevThreshold: 0.1,     // Std dev < 0.1
  minUpdatesForConvergence: 200,       // Minimum 200 updates

  // Reward function weights
  rewardDirectionWeight: 0.6,    // 60% direction alignment
  rewardMagnitudeWeight: 0.4,    // 40% magnitude
  rewardProximityBonus: 0.2,     // +0.2 if within 0.15
  rewardStressPenalty: -0.15     // -0.15 if stress increases >0.2
};
```

### 10.2 Hyperparameter Tuning Guide

| Parameter | Effect | Tuning Advice |
|-----------|--------|---------------|
| `learningRate` (α) | Higher = faster learning, more instability | Start 0.1, increase to 0.15 for faster convergence, decrease to 0.05 for stability |
| `discountFactor` (γ) | Higher = values long-term rewards more | Keep 0.90-0.95; higher for strategic outcomes, lower for immediate rewards |
| `initialExplorationRate` (ε₀) | Higher = more random exploration | Start 0.15-0.30 for cold start, decrease to 0.10 for exploitation |
| `explorationDecay` | Higher = slower decay | Use 0.95 for gradual shift, 0.90 for faster exploitation |
| `ucbConstant` (c) | Higher = more exploration bonus | Start 2.0, increase to 3.0 for more exploration, decrease to 1.0 for exploitation |
| `convergenceWindow` | Larger = slower convergence detection | Use 100 for stable detection, 50 for faster (less reliable) |

### 10.3 Environment-Specific Configurations

```typescript
// Development/Testing Configuration
export const DEV_RL_CONFIG: RLConfig = {
  ...DEFAULT_RL_CONFIG,
  learningRate: 0.15,              // Faster learning for testing
  initialExplorationRate: 0.30,    // More exploration
  convergenceWindow: 50,           // Faster convergence detection
  minUpdatesForConvergence: 100    // Lower threshold for testing
};

// Production Configuration
export const PROD_RL_CONFIG: RLConfig = {
  ...DEFAULT_RL_CONFIG,
  learningRate: 0.1,               // Stable learning
  initialExplorationRate: 0.15,    // Balanced exploration
  convergenceWindow: 100,          // Reliable convergence
  minUpdatesForConvergence: 200    // High confidence threshold
};
```

---

## 11. Performance Considerations

### 11.1 Time Complexity Analysis

| Operation | Complexity | Notes |
|-----------|------------|-------|
| `selectAction()` | O(n) | n = candidates, dominated by Q-value lookups |
| `exploit()` | O(n) | Linear scan for max Q-value |
| `explore()` | O(n) | UCB calculation for each candidate |
| `updatePolicy()` | O(1) | Single Q-value update |
| `getMaxQValue()` | O(m) | m = actions in state (~20-50) |
| `batchUpdate()` | O(k) | k = batch size (32) |
| `hashState()` | O(1) | Simple arithmetic |
| `calculateReward()` | O(1) | Vector operations |

### 11.2 Space Complexity Analysis

| Component | Complexity | Notes |
|-----------|------------|-------|
| Q-Table | O(S × A × U) | S=75 states, A=content count, U=users |
| Replay Buffer | O(B × U) | B=1000 experiences per user |
| User Stats | O(U) | U = user count |
| RuVector Index | O(A) | A = content catalog size |

### 11.3 Performance Optimizations

#### 1. Q-Value Lookup Caching

```typescript
class QTableManager {
  private cache = new LRU<string, number>({ max: 1000, ttl: 60000 });

  async getQValue(userId: string, stateHash: string, contentId: string): Promise<number> {
    const cacheKey = `${userId}:${stateHash}:${contentId}`;

    // Check cache first
    const cached = this.cache.get(cacheKey);
    if (cached !== undefined) return cached;

    // Fallback to AgentDB
    const qValue = await this.agentDB.get(this.buildKey(userId, stateHash, contentId)) ?? 0;
    this.cache.set(cacheKey, qValue);

    return qValue;
  }
}
```

#### 2. Batch Q-Value Updates

```typescript
async batchUpdateQValues(updates: Array<{ key: string, value: number }>): Promise<void> {
  const pipeline = this.agentDB.pipeline();

  for (const { key, value } of updates) {
    pipeline.set(key, value, { ttl: this.ttl });
  }

  await pipeline.exec();
}
```

#### 3. Parallel Candidate Evaluation

```typescript
async exploit(userId: string, stateHash: string, candidates: string[]): Promise<ContentRecommendation> {
  // Parallel Q-value lookups
  const qValues = await Promise.all(
    candidates.map(contentId =>
      this.qTableManager.getQValue(userId, stateHash, contentId)
    )
  );

  // Find max Q-value
  const maxIndex = qValues.reduce((maxIdx, val, idx, arr) =>
    val > arr[maxIdx] ? idx : maxIdx
  , 0);

  return {
    contentId: candidates[maxIndex],
    qValue: qValues[maxIndex],
    ...
  };
}
```

#### 4. State Hash Pre-computation

```typescript
// Pre-compute state hashes during emotion detection
const emotionalStateWithHash = {
  ...emotionalState,
  _stateHash: stateHasher.hashState(emotionalState)
};

// Reuse in RL engine
const stateHash = emotionalState._stateHash || stateHasher.hashState(emotionalState);
```

### 11.4 Scalability Considerations

#### Database Sharding Strategy

```typescript
// Shard Q-tables by user ID
const shardId = hashCode(userId) % NUM_SHARDS;
const agentDB = agentDBShards[shardId];

// Each shard handles subset of users
// Scales horizontally with user growth
```

#### Content Catalog Pagination

```typescript
// Don't load all content IDs at once
async selectAction(userId, state, desired, candidateLimit = 100): Promise<ActionSelection> {
  // 1. Semantic search returns top K candidates
  const candidates = await this.ruVector.search({
    vector: this.createTransitionVector(state, desired),
    topK: candidateLimit  // Limit candidates
  });

  // 2. Re-rank with Q-values (only top K)
  const rankedCandidates = await this.rankByQValues(userId, stateHash, candidates);

  return rankedCandidates[0];
}
```

---

## 12. Security & Privacy

### 12.1 Data Retention Policies

```typescript
// Q-Table entries: 90-day TTL
// Rationale: Emotional preferences may change over time
const Q_TABLE_TTL = 90 * 24 * 60 * 60;  // 90 days

// Replay buffer: 30-day TTL
// Rationale: Recent experiences most relevant
const REPLAY_BUFFER_TTL = 30 * 24 * 60 * 60;  // 30 days

// User stats: No TTL (persistent)
// Rationale: Convergence metrics needed long-term
```

### 12.2 Data Anonymization

```typescript
// Hash user IDs before storage
function anonymizeUserId(userId: string): string {
  return crypto.createHash('sha256')
    .update(userId + process.env.SALT)
    .digest('hex');
}

// Use anonymized IDs in Q-table keys
const anonUserId = anonymizeUserId(userId);
const key = `qtable:${anonUserId}:${stateHash}:${contentId}`;
```

### 12.3 Access Control

```typescript
// User can only access their own Q-values
async getQValue(requestingUserId: string, userId: string, ...): Promise<number> {
  if (requestingUserId !== userId) {
    throw new UnauthorizedError('Cannot access other user Q-values');
  }

  return await this.qTableManager.getQValue(userId, ...);
}
```

### 12.4 GDPR Compliance

```typescript
// Right to be forgotten: Delete all user data
async deleteUserData(userId: string): Promise<void> {
  // 1. Delete Q-table entries
  const pattern = `qtable:${userId}:*`;
  await this.agentDB.deletePattern(pattern);

  // 2. Delete user stats
  await this.agentDB.delete(`rlstats:${userId}`);

  // 3. Delete replay buffer
  await this.agentDB.delete(`replay:${userId}`);

  // 4. Delete convergence metrics
  await this.agentDB.delete(`convergence:${userId}`);

  this.logger.info(`Deleted all RL data for user ${userId}`);
}

// Data export: Export user Q-values
async exportUserData(userId: string): Promise<UserDataExport> {
  const qTableEntries = await this.qTableManager.getAllUserQValues(userId);
  const userStats = await this.getUserStats(userId);

  return {
    userId,
    qTableEntries,
    userStats,
    exportDate: new Date().toISOString()
  };
}
```

---

## 13. Monitoring & Observability

### 13.1 Key Metrics to Track

```typescript
// Learning metrics
export interface RLMetrics {
  // Policy performance
  meanReward: number;              // Average reward per episode
  rewardVariance: number;          // Reward variance
  explorationRate: number;         // Current ε value

  // Convergence
  meanAbsTDError: number;          // Mean absolute TD error
  tdErrorStdDev: number;           // TD error standard deviation
  convergenceStatus: boolean;      // Has policy converged?

  // Q-value statistics
  meanQValue: number;              // Average Q-value
  qValueVariance: number;          // Q-value variance
  maxQValue: number;               // Highest Q-value
  minQValue: number;               // Lowest Q-value

  // Visit statistics
  totalStateVisits: number;        // Total state visits
  uniqueStates: number;            // Unique states encountered
  stateEntropy: number;            // State distribution entropy

  // Action statistics
  explorationActions: number;      // Count of exploration actions
  exploitationActions: number;     // Count of exploitation actions
  explorationRatio: number;        // Explore / (explore + exploit)

  // Performance
  avgActionSelectionTime: number;  // ms
  avgPolicyUpdateTime: number;     // ms
  qTableSize: number;              // Total Q-table entries

  timestamp: number;
}
```

### 13.2 Logging Strategy

```typescript
class RLLogger {
  // Action selection logging
  logActionSelection(userId: string, selection: ActionSelection): void {
    this.logger.info('RL Action Selection', {
      userId,
      stateHash: selection.stateHash,
      contentId: selection.contentId,
      qValue: selection.qValue,
      isExploration: selection.isExploration,
      explorationBonus: selection.explorationBonus,
      confidence: selection.confidence,
      candidateCount: selection.candidateCount,
      timestamp: selection.timestamp
    });
  }

  // Policy update logging
  logPolicyUpdate(userId: string, update: PolicyUpdate): void {
    this.logger.info('RL Policy Update', {
      userId,
      stateHash: update.stateHash,
      contentId: update.contentId,
      oldQValue: update.oldQValue,
      newQValue: update.newQValue,
      tdError: update.tdError,
      reward: update.reward,
      visitCount: update.visitCount,
      explorationRate: update.explorationRate,
      convergenceStatus: update.convergenceStatus.hasConverged,
      timestamp: update.timestamp
    });
  }

  // Error logging
  logError(operation: string, error: Error, context: any): void {
    this.logger.error(`RL ${operation} Error`, {
      operation,
      error: error.message,
      stack: error.stack,
      context,
      timestamp: Date.now()
    });
  }
}
```

### 13.3 Monitoring Dashboard Metrics

```typescript
// Prometheus metrics
export const rlMetrics = {
  actionSelections: new prometheus.Counter({
    name: 'rl_action_selections_total',
    help: 'Total action selections',
    labelNames: ['userId', 'isExploration']
  }),

  policyUpdates: new prometheus.Counter({
    name: 'rl_policy_updates_total',
    help: 'Total policy updates',
    labelNames: ['userId']
  }),

  rewardHistogram: new prometheus.Histogram({
    name: 'rl_reward_distribution',
    help: 'Reward distribution',
    buckets: [-1, -0.5, 0, 0.5, 1],
    labelNames: ['userId']
  }),

  tdErrorGauge: new prometheus.Gauge({
    name: 'rl_td_error_current',
    help: 'Current TD error',
    labelNames: ['userId']
  }),

  qValueGauge: new prometheus.Gauge({
    name: 'rl_q_value_mean',
    help: 'Mean Q-value',
    labelNames: ['userId', 'stateHash']
  }),

  actionSelectionDuration: new prometheus.Histogram({
    name: 'rl_action_selection_duration_ms',
    help: 'Action selection duration',
    buckets: [10, 50, 100, 500, 1000],
    labelNames: ['userId']
  })
};
```

### 13.4 Alerting Rules

```yaml
# Prometheus alerting rules
groups:
  - name: rl_policy_alerts
    interval: 1m
    rules:
      # TD error too high (not converging)
      - alert: RLHighTDError
        expr: rl_td_error_current > 0.5
        for: 10m
        annotations:
          summary: "RL policy not converging for user {{ $labels.userId }}"
          description: "TD error {{ $value }} > 0.5 for 10 minutes"

      # Q-values all zero (cold start issue)
      - alert: RLZeroQValues
        expr: rl_q_value_mean == 0
        for: 5m
        annotations:
          summary: "RL Q-values stuck at zero for user {{ $labels.userId }}"
          description: "No learning progress detected"

      # Action selection too slow
      - alert: RLSlowActionSelection
        expr: histogram_quantile(0.95, rl_action_selection_duration_ms) > 3000
        for: 5m
        annotations:
          summary: "RL action selection latency > 3s (p95)"
          description: "Performance degradation detected"

      # Exploration rate stuck
      - alert: RLExplorationRateStuck
        expr: rate(rl_action_selections_total{isExploration="true"}[5m]) == 0
        for: 10m
        annotations:
          summary: "No exploration actions for user {{ $labels.userId }}"
          description: "Policy may be over-fitting"
```

---

## 14. Deployment Architecture

### 14.1 Service Deployment Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     Production Deployment                        │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────┐
│   API Gateway   │
│   (Kong/Nginx)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   GraphQL API   │   ┌─────────────────┐
│   (Node.js)     │◄──│  Load Balancer  │
│   Port: 3000    │   └─────────────────┘
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────────────┐
│         RLPolicyEngine Module               │
│    (Embedded in API, stateless)             │
└────────┬────────────────┬───────────────────┘
         │                │
         ▼                ▼
┌─────────────────┐  ┌─────────────────┐
│    AgentDB      │  │    RuVector     │
│  (Redis Cluster)│  │  (Vector Store) │
│                 │  │                 │
│ • 3 master nodes│  │ • HNSW index    │
│ • 3 replica     │  │ • 1536D vectors │
│ • Sharded by ID │  │ • Port: 8080    │
│ • Port: 6379    │  └─────────────────┘
└─────────────────┘
         │
         ▼
┌─────────────────┐
│   Monitoring    │
│  (Prometheus)   │
│  • Metrics      │
│  • Alerts       │
│  • Grafana UI   │
└─────────────────┘
```

### 14.2 Docker Compose Configuration

```yaml
# docker-compose.yml
version: '3.8'

services:
  # GraphQL API with RL Engine
  api:
    build: ./api
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - GEMINI_API_KEY=${GEMINI_API_KEY}
      - RUVECTOR_URL=http://ruvector:8080
      - AGENTDB_REDIS_URL=redis://agentdb-master:6379
      - RL_LEARNING_RATE=0.1
      - RL_DISCOUNT_FACTOR=0.95
      - RL_INITIAL_EXPLORATION_RATE=0.15
    depends_on:
      - ruvector
      - agentdb-master
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '1.0'
          memory: 1G

  # RuVector (Vector Database)
  ruvector:
    image: ruvector:latest
    ports:
      - "8080:8080"
    volumes:
      - ruvector-data:/data
    environment:
      - HNSW_M=16
      - HNSW_EF_CONSTRUCTION=200
      - VECTOR_DIM=1536
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G

  # AgentDB (Redis Master)
  agentdb-master:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - agentdb-master-data:/data
    command: redis-server --maxmemory 2gb --maxmemory-policy allkeys-lru
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 2G

  # AgentDB (Redis Replica)
  agentdb-replica:
    image: redis:7-alpine
    depends_on:
      - agentdb-master
    command: redis-server --replicaof agentdb-master 6379
    deploy:
      replicas: 2
      resources:
        limits:
          cpus: '0.5'
          memory: 2G

  # Prometheus (Monitoring)
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'

  # Grafana (Dashboard)
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    volumes:
      - grafana-data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin

volumes:
  ruvector-data:
  agentdb-master-data:
  prometheus-data:
  grafana-data:
```

### 14.3 Kubernetes Deployment (Production)

```yaml
# kubernetes/rl-policy-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rl-policy-api
spec:
  replicas: 5
  selector:
    matchLabels:
      app: rl-policy-api
  template:
    metadata:
      labels:
        app: rl-policy-api
    spec:
      containers:
      - name: api
        image: emotistream/api:latest
        ports:
        - containerPort: 3000
        env:
        - name: RL_LEARNING_RATE
          value: "0.1"
        - name: AGENTDB_REDIS_URL
          valueFrom:
            secretKeyRef:
              name: agentdb-secret
              key: redis-url
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: rl-policy-api-service
spec:
  selector:
    app: rl-policy-api
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

---

## 15. Testing Strategy

### 15.1 Unit Tests

```typescript
// tests/policy-engine.test.ts
describe('RLPolicyEngine', () => {
  let rlEngine: RLPolicyEngine;
  let mockAgentDB: jest.Mocked<AgentDB>;
  let mockRuVector: jest.Mocked<RuVectorClient>;

  beforeEach(() => {
    mockAgentDB = createMockAgentDB();
    mockRuVector = createMockRuVector();
    rlEngine = new RLPolicyEngine(mockAgentDB, mockRuVector, logger);
  });

  describe('selectAction', () => {
    it('should exploit (use max Q-value) when not exploring', async () => {
      // Arrange
      mockAgentDB.get.mockResolvedValue(0.72);  // Q-value for content-1
      mockRuVector.search.mockResolvedValue([
        { id: 'content-1', similarity: 0.9 },
        { id: 'content-2', similarity: 0.8 }
      ]);

      jest.spyOn(rlEngine.explorationStrategy, 'shouldExplore')
        .mockResolvedValue(false);

      // Act
      const result = await rlEngine.selectAction(
        'user-1',
        mockEmotionalState,
        mockDesiredState,
        ['content-1', 'content-2']
      );

      // Assert
      expect(result.contentId).toBe('content-1');
      expect(result.isExploration).toBe(false);
      expect(result.qValue).toBe(0.72);
    });

    it('should explore (use UCB) when exploring', async () => {
      // Arrange
      jest.spyOn(rlEngine.explorationStrategy, 'shouldExplore')
        .mockResolvedValue(true);

      mockAgentDB.query.mockResolvedValue([
        { metadata: { visitCount: 10 } },  // content-1
        { metadata: { visitCount: 2 } }    // content-2 (less visited)
      ]);

      // Act
      const result = await rlEngine.selectAction(
        'user-1',
        mockEmotionalState,
        mockDesiredState,
        ['content-1', 'content-2']
      );

      // Assert
      expect(result.isExploration).toBe(true);
      expect(result.explorationBonus).toBeGreaterThan(0);
    });
  });

  describe('updatePolicy', () => {
    it('should increase Q-value after positive reward', async () => {
      // Arrange
      const experience: EmotionalExperience = {
        experienceId: 'exp-1',
        userId: 'user-1',
        stateBefore: { valence: -0.6, arousal: 0.5, stress: 0.7, confidence: 0.8 },
        stateAfter: { valence: 0.2, arousal: 0.1, stress: 0.5, confidence: 0.8 },
        desiredState: { valence: 0.6, arousal: 0.3, confidence: 0.8 },
        contentId: 'content-1',
        reward: 0.72,
        timestamp: Date.now()
      };

      mockAgentDB.get.mockResolvedValue(0.45);  // Current Q-value

      // Act
      const update = await rlEngine.updatePolicy('user-1', experience);

      // Assert
      expect(update.newQValue).toBeGreaterThan(0.45);
      expect(update.tdError).toBeGreaterThan(0);
      expect(mockAgentDB.set).toHaveBeenCalled();
    });
  });
});
```

### 15.2 Integration Tests

```typescript
// tests/integration/rl-engine.integration.test.ts
describe('RLPolicyEngine Integration', () => {
  let rlEngine: RLPolicyEngine;
  let agentDB: AgentDB;
  let ruVector: RuVectorClient;

  beforeAll(async () => {
    agentDB = new AgentDB(process.env.TEST_REDIS_URL);
    ruVector = new RuVectorClient(process.env.TEST_RUVECTOR_URL);
    rlEngine = new RLPolicyEngine(agentDB, ruVector, logger);

    // Seed test data
    await seedTestContent(ruVector);
  });

  afterAll(async () => {
    await agentDB.flushAll();
    await agentDB.disconnect();
  });

  it('should complete full RL cycle: select → feedback → update', async () => {
    const userId = 'test-user-1';
    const emotionalState = {
      valence: -0.6,
      arousal: 0.5,
      stress: 0.7,
      confidence: 0.8
    };
    const desiredState = {
      valence: 0.6,
      arousal: 0.3,
      confidence: 0.8
    };

    // 1. Select action
    const selection = await rlEngine.selectAction(
      userId,
      emotionalState,
      desiredState,
      ['content-1', 'content-2', 'content-3']
    );

    expect(selection.contentId).toBeDefined();

    // 2. Simulate feedback
    const experience: EmotionalExperience = {
      experienceId: 'exp-1',
      userId,
      stateBefore: emotionalState,
      stateAfter: {
        valence: 0.2,
        arousal: 0.1,
        stress: 0.5,
        confidence: 0.8
      },
      desiredState,
      contentId: selection.contentId,
      reward: 0.72,
      timestamp: Date.now()
    };

    // 3. Update policy
    const update = await rlEngine.updatePolicy(userId, experience);

    expect(update.newQValue).toBeDefined();
    expect(update.tdError).toBeDefined();

    // 4. Verify persistence
    const storedQ = await rlEngine.getQValue(
      userId,
      selection.stateHash,
      selection.contentId
    );

    expect(storedQ).toBe(update.newQValue);
  });

  it('should improve Q-values over 50 experiences', async () => {
    const userId = 'test-user-2';
    const rewards: number[] = [];

    // Simulate 50 experiences
    for (let i = 0; i < 50; i++) {
      // Select action
      const selection = await rlEngine.selectAction(
        userId,
        mockEmotionalState,
        mockDesiredState,
        testContentIds
      );

      // Simulate positive outcome
      const experience = createMockExperience(userId, selection.contentId, 0.6 + Math.random() * 0.3);

      // Update policy
      const update = await rlEngine.updatePolicy(userId, experience);
      rewards.push(experience.reward);
    }

    // Analyze learning
    const first10 = rewards.slice(0, 10);
    const last10 = rewards.slice(40, 50);

    const meanFirst = first10.reduce((a, b) => a + b) / first10.length;
    const meanLast = last10.reduce((a, b) => a + b) / last10.length;

    // Expect improvement (later rewards should be higher)
    expect(meanLast).toBeGreaterThan(meanFirst * 1.2);  // At least 20% improvement
  });
});
```

### 15.3 Performance Tests

```typescript
// tests/performance/rl-engine.perf.test.ts
describe('RLPolicyEngine Performance', () => {
  it('should select action in <100ms for 100 candidates', async () => {
    const start = Date.now();

    await rlEngine.selectAction(
      'user-1',
      mockEmotionalState,
      mockDesiredState,
      generate100ContentIds()
    );

    const duration = Date.now() - start;
    expect(duration).toBeLessThan(100);
  });

  it('should update policy in <50ms', async () => {
    const start = Date.now();

    await rlEngine.updatePolicy('user-1', mockExperience);

    const duration = Date.now() - start;
    expect(duration).toBeLessThan(50);
  });

  it('should handle 100 concurrent action selections', async () => {
    const start = Date.now();

    const promises = Array.from({ length: 100 }, (_, i) =>
      rlEngine.selectAction(
        `user-${i}`,
        mockEmotionalState,
        mockDesiredState,
        testContentIds
      )
    );

    await Promise.all(promises);

    const duration = Date.now() - start;
    const avgDuration = duration / 100;

    expect(avgDuration).toBeLessThan(200);  // Avg <200ms per request
  });
});
```

---

## 16. Appendix: Example Usage

### 16.1 Complete Usage Example

```typescript
import { RLPolicyEngine } from './rl/policy-engine';
import AgentDB from '@ruvnet/agentdb';
import RuVectorClient from '@ruvnet/ruvector';

// Initialize dependencies
const agentDB = new AgentDB(process.env.AGENTDB_URL);
const ruVector = new RuVectorClient(process.env.RUVECTOR_URL);

// Create RL engine
const rlEngine = new RLPolicyEngine(agentDB, ruVector, logger);

// 1. User emotional input
const userId = 'user-001';
const emotionalState = {
  valence: -0.6,    // Negative mood
  arousal: 0.5,     // Moderately aroused
  stress: 0.7,      // High stress
  confidence: 0.82
};

const desiredState = {
  valence: 0.6,     // Want positive mood
  arousal: 0.3,     // Want calm
  confidence: 0.75
};

// 2. Get recommendation
const availableContent = ['content-001', 'content-002', ..., 'content-200'];

const selection = await rlEngine.selectAction(
  userId,
  emotionalState,
  desiredState,
  availableContent
);

console.log('Recommendation:', {
  contentId: selection.contentId,
  qValue: selection.qValue,
  isExploration: selection.isExploration,
  confidence: selection.confidence,
  reasoning: selection.reasoning
});

// 3. User watches content and provides feedback
const postViewingState = {
  valence: 0.2,     // Improved!
  arousal: 0.1,     // Calmer
  stress: 0.5,      // Reduced stress
  confidence: 0.78
};

// 4. Update policy
const experience: EmotionalExperience = {
  experienceId: uuidv4(),
  userId,
  stateBefore: emotionalState,
  stateAfter: postViewingState,
  desiredState,
  contentId: selection.contentId,
  reward: 0,  // Will be calculated
  timestamp: Date.now()
};

// Calculate reward
experience.reward = rlEngine.rewardCalculator.calculateReward(
  experience.stateBefore,
  experience.stateAfter,
  experience.desiredState
);

// Update Q-values
const update = await rlEngine.updatePolicy(userId, experience);

console.log('Policy Updated:', {
  oldQValue: update.oldQValue,
  newQValue: update.newQValue,
  tdError: update.tdError,
  reward: update.reward,
  explorationRate: update.explorationRate,
  hasConverged: update.convergenceStatus.hasConverged
});
```

---

## Document Status

**Status**: Complete
**Next Phase**: Refinement (SPARC Phase 4) - Implementation
**Implementation Target**: MVP v1.0.0
**Estimated Implementation Time**: 20 hours

---

**End of Architecture Specification**
