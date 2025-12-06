# EmotiStream Nexus MVP - SPARC Phase 3: Architecture

**Generated**: 2025-12-05
**SPARC Phase**: 3 - Architecture
**Status**: Complete - Ready for Refinement Phase

---

## Overview

This directory contains detailed architecture specifications for all 6 core modules of the EmotiStream Nexus MVP. Each document provides:

- Module structure and file organization
- TypeScript interfaces and type definitions
- Class diagrams (ASCII)
- Sequence diagrams (ASCII)
- Integration points
- Error handling strategies
- Testing approaches
- Performance considerations

---

## Architecture Documents

| Document | Module | Key Components | LOC Est. |
|----------|--------|----------------|----------|
| [ARCH-ProjectStructure.md](./ARCH-ProjectStructure.md) | **Project Setup** | Directory structure, shared types, DI container, config | ~1,500 |
| [ARCH-EmotionDetector.md](./ARCH-EmotionDetector.md) | **Emotion Detection** | Gemini client, Russell/Plutchik mappers, state hasher | ~800 |
| [ARCH-RLPolicyEngine.md](./ARCH-RLPolicyEngine.md) | **RL Policy Engine** | Q-learning, exploration strategies, reward calculator | ~1,000 |
| [ARCH-ContentProfiler.md](./ARCH-ContentProfiler.md) | **Content Profiler** | Batch profiling, embeddings, RuVector HNSW | ~600 |
| [ARCH-RecommendationEngine.md](./ARCH-RecommendationEngine.md) | **Recommendations** | Hybrid ranking, outcome prediction, reasoning | ~700 |
| [ARCH-FeedbackAPI-CLI.md](./ARCH-FeedbackAPI-CLI.md) | **Feedback/API/CLI** | Reward calculation, REST API, interactive demo | ~1,200 |

**Total Estimated LOC**: ~5,800 lines of TypeScript

---

## Module Dependency Graph

```
                    ┌─────────────────────────────────────┐
                    │           CLI DEMO                   │
                    │      (Interactive Interface)         │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │           REST API                   │
                    │    (Express + Middleware)            │
                    └──────────────┬──────────────────────┘
                                   │
          ┌────────────────────────┼────────────────────────┐
          │                        │                        │
          ▼                        ▼                        ▼
┌─────────────────┐    ┌─────────────────────┐    ┌─────────────────┐
│ EMOTION         │    │ RECOMMENDATION      │    │ FEEDBACK        │
│ DETECTOR        │◄──►│ ENGINE              │◄──►│ PROCESSOR       │
│                 │    │                     │    │                 │
│ • Gemini API    │    │ • Hybrid Ranking    │    │ • Reward Calc   │
│ • Mappers       │    │ • Outcome Predict   │    │ • Experience    │
│ • State Hash    │    │ • Reasoning         │    │ • User Profile  │
└────────┬────────┘    └──────────┬──────────┘    └────────┬────────┘
         │                        │                        │
         │             ┌──────────┴──────────┐             │
         │             │                     │             │
         │             ▼                     ▼             │
         │    ┌─────────────────┐   ┌─────────────────┐    │
         │    │ RL POLICY       │   │ CONTENT         │    │
         │    │ ENGINE          │   │ PROFILER        │    │
         │    │                 │   │                 │    │
         │    │ • Q-Learning    │   │ • Batch Profile │    │
         │    │ • Exploration   │   │ • Embeddings    │    │
         │    │ • Q-Table       │   │ • RuVector      │    │
         │    └────────┬────────┘   └────────┬────────┘    │
         │             │                     │             │
         └─────────────┴──────────┬──────────┴─────────────┘
                                  │
                    ┌─────────────▼─────────────┐
                    │      STORAGE LAYER        │
                    │                           │
                    │  AgentDB     RuVector     │
                    │  (Q-tables)  (Embeddings) │
                    └───────────────────────────┘
```

---

## Core TypeScript Interfaces

### Shared Types (`src/types/`)

```typescript
// Emotional State (Russell's Circumplex)
interface EmotionalState {
  valence: number;        // -1 (negative) to +1 (positive)
  arousal: number;        // -1 (calm) to +1 (excited)
  stressLevel: number;    // 0 to 1
  primaryEmotion: string; // joy, sadness, anger, fear, etc.
  emotionVector: Float32Array; // Plutchik 8D
  confidence: number;     // 0 to 1
  timestamp: number;
}

// Q-Table Entry (RL)
interface QTableEntry {
  userId: string;
  stateHash: string;      // "v:a:s" format (e.g., "2:3:1")
  contentId: string;
  qValue: number;
  visitCount: number;
  lastUpdated: number;
}

// Recommendation Result
interface Recommendation {
  contentId: string;
  title: string;
  qValue: number;
  similarityScore: number;
  combinedScore: number;  // Q*0.7 + Sim*0.3
  predictedOutcome: PredictedOutcome;
  reasoning: string;
  isExploration: boolean;
}
```

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **TypeScript + ESM** | Modern Node.js with full type safety |
| **InversifyJS DI** | Loose coupling, easy testing, clean initialization |
| **5×5×3 State Space** | 75 states balances tractability with granularity |
| **Q-Learning (not DQN)** | Simpler, sufficient for MVP, faster convergence |
| **70/30 Hybrid Ranking** | Balances learned preferences with content similarity |
| **AgentDB for Q-Tables** | Key-value with TTL, perfect for RL state |
| **RuVector HNSW** | Fast ANN search (M=16, ef=200), 95%+ recall |
| **Express REST API** | Familiar, fast to implement, good middleware |
| **Inquirer.js CLI** | Interactive prompts, great for demos |

---

## Hyperparameters Summary

| Parameter | Value | Module |
|-----------|-------|--------|
| Learning rate (α) | 0.1 | RLPolicyEngine |
| Discount factor (γ) | 0.95 | RLPolicyEngine |
| Exploration rate (ε) | 0.15 → 0.10 | RLPolicyEngine |
| Exploration decay | 0.95 per episode | RLPolicyEngine |
| UCB constant (c) | 2.0 | RLPolicyEngine |
| State buckets | 5×5×3 (V×A×S) | RLPolicyEngine |
| Q-value weight | 70% | RecommendationEngine |
| Similarity weight | 30% | RecommendationEngine |
| Direction weight | 60% | FeedbackReward |
| Magnitude weight | 40% | FeedbackReward |
| Proximity bonus | +0.1 (if dist < 0.3) | FeedbackReward |
| Embedding dimensions | 1536 | ContentProfiler |
| HNSW M | 16 | ContentProfiler |
| HNSW efConstruction | 200 | ContentProfiler |
| Batch size | 10 items | ContentProfiler |

---

## File Structure Overview

```
src/
├── types/
│   ├── index.ts                 # Re-exports
│   ├── emotional-state.ts       # EmotionalState, DesiredState
│   ├── content.ts               # ContentMetadata, EmotionalContentProfile
│   ├── rl.ts                    # QTableEntry, EmotionalExperience
│   ├── recommendation.ts        # Recommendation, PredictedOutcome
│   └── api.ts                   # Request/Response types
├── emotion/
│   ├── index.ts                 # Public exports
│   ├── detector.ts              # EmotionDetector class
│   ├── gemini-client.ts         # Gemini API wrapper
│   ├── mappers/
│   │   ├── valence-arousal.ts   # Russell's Circumplex
│   │   ├── plutchik.ts          # 8D emotion vectors
│   │   └── stress.ts            # Stress calculation
│   ├── state-hasher.ts          # State discretization
│   └── desired-state.ts         # Desired state prediction
├── rl/
│   ├── index.ts                 # Public exports
│   ├── policy-engine.ts         # RLPolicyEngine class
│   ├── q-table.ts               # Q-table with AgentDB
│   ├── reward-calculator.ts     # Reward function
│   ├── exploration/
│   │   ├── epsilon-greedy.ts    # ε-greedy strategy
│   │   └── ucb.ts               # UCB bonus
│   └── replay-buffer.ts         # Experience replay
├── content/
│   ├── index.ts                 # Public exports
│   ├── profiler.ts              # ContentProfiler class
│   ├── batch-processor.ts       # Batch Gemini profiling
│   ├── embedding-generator.ts   # 1536D embeddings
│   ├── ruvector-client.ts       # RuVector HNSW
│   └── mock-catalog.ts          # Mock content (200 items)
├── recommendations/
│   ├── index.ts                 # Public exports
│   ├── engine.ts                # RecommendationEngine class
│   ├── ranker.ts                # Hybrid ranking
│   ├── outcome-predictor.ts     # Outcome prediction
│   └── reasoning.ts             # Explanation generation
├── feedback/
│   ├── index.ts                 # Public exports
│   ├── processor.ts             # FeedbackProcessor class
│   ├── reward-calculator.ts     # Multi-factor reward
│   └── experience-store.ts      # Experience persistence
├── api/
│   ├── index.ts                 # Express app
│   ├── routes/
│   │   ├── emotion.ts           # /api/v1/emotion/*
│   │   ├── recommend.ts         # /api/v1/recommend
│   │   └── feedback.ts          # /api/v1/feedback
│   └── middleware/
│       ├── error-handler.ts
│       └── rate-limiter.ts
├── cli/
│   ├── index.ts                 # CLI entry point
│   ├── demo.ts                  # Demo flow
│   ├── prompts.ts               # Inquirer prompts
│   └── display/
│       ├── emotion.ts           # Emotion display
│       ├── recommendations.ts   # Recommendation table
│       └── learning.ts          # Learning progress
├── db/
│   ├── agentdb-client.ts        # AgentDB wrapper
│   └── ruvector-client.ts       # RuVector wrapper
└── utils/
    ├── logger.ts                # Structured logging
    ├── config.ts                # Configuration
    └── errors.ts                # Custom error types
```

---

## Performance Targets

| Metric | Target | Module |
|--------|--------|--------|
| Emotion detection | <3s (p95) | EmotionDetector |
| Recommendation generation | <500ms (p95) | RecommendationEngine |
| Q-value lookup | <10ms | RLPolicyEngine |
| Feedback processing | <200ms | FeedbackProcessor |
| Content search | <100ms | ContentProfiler |
| Full demo cycle | <5s | End-to-end |

---

## Testing Strategy

### Unit Tests (Jest)
- Each module has `*.test.ts` files
- Mock external dependencies (Gemini, AgentDB, RuVector)
- 80%+ code coverage target

### Integration Tests
- API endpoint tests with Supertest
- Full recommendation flow tests
- Q-value update verification

### Demo Tests
- CLI flow automation
- 5-minute stability tests
- Visual output verification

---

## Next Phase: Refinement (SPARC Phase 4)

With architecture complete, the next phase involves:

1. **Project Setup**: Initialize TypeScript project, install dependencies
2. **TDD Implementation**: Write tests first, then implement modules
3. **Integration**: Wire modules together via DI container
4. **API Development**: Build REST endpoints
5. **CLI Demo**: Create interactive demo interface
6. **Testing**: Achieve 80%+ coverage
7. **Demo Rehearsal**: Practice 3-minute demo flow

See [PLAN-EmotiStream-MVP.md](../PLAN-EmotiStream-MVP.md) for hour-by-hour implementation schedule.

---

**SPARC Phase 3 Complete** - 6 architecture documents ready for implementation.
