# EmotionDetector Module Architecture

**Component**: Emotion Detection System
**Version**: 1.0.0
**SPARC Phase**: Architecture
**Last Updated**: 2025-12-05
**Status**: Ready for Implementation

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Module Structure](#module-structure)
3. [Class Diagrams](#class-diagrams)
4. [TypeScript Interfaces](#typescript-interfaces)
5. [Sequence Diagrams](#sequence-diagrams)
6. [Component Architecture](#component-architecture)
7. [Error Handling Strategy](#error-handling-strategy)
8. [Testing Strategy](#testing-strategy)
9. [Performance Considerations](#performance-considerations)
10. [Deployment Architecture](#deployment-architecture)

---

## Executive Summary

The **EmotionDetector** module is the core emotion analysis component of EmotiStream Nexus, responsible for:

- **Text-based emotion detection** via Gemini API (MVP-001)
- **Valence-arousal mapping** using Russell's Circumplex Model
- **8D emotion vector generation** based on Plutchik's Wheel
- **Stress level calculation** from emotional dimensions
- **Desired state prediction** using rule-based heuristics (MVP-002)
- **Persistent storage** in AgentDB for emotional history tracking

### Key Metrics

| Metric | Target | Implementation Strategy |
|--------|--------|------------------------|
| Average Response Time | < 3 seconds | Async API calls, timeout handling |
| P95 Response Time | < 5 seconds | Retry logic with exponential backoff |
| API Success Rate | > 98% | Fallback to neutral state on failure |
| Confidence Score | > 0.8 average | Multi-factor confidence calculation |
| Fallback Rate | < 2% | Robust error handling and validation |

---

## Module Structure

```
src/emotion/
├── index.ts                    # Public exports and module entry point
├── detector.ts                 # EmotionDetector main class
├── gemini-client.ts            # Gemini API integration with retry logic
├── types.ts                    # Module-specific TypeScript types
│
├── mappers/
│   ├── index.ts                # Mapper exports
│   ├── valence-arousal.ts      # Russell's Circumplex mapping
│   ├── plutchik.ts             # 8D Plutchik emotion vectors
│   └── stress.ts               # Stress level calculation
│
├── state/
│   ├── state-hasher.ts         # Discretize continuous state space
│   └── desired-state.ts        # Desired state prediction (MVP-002)
│
├── utils/
│   ├── validators.ts           # Input/response validation
│   ├── fallback.ts             # Fallback state generation
│   └── logger.ts               # Structured logging
│
└── __tests__/
    ├── detector.test.ts        # EmotionDetector unit tests
    ├── gemini-client.test.ts   # API client tests
    ├── mappers/                # Mapper unit tests
    │   ├── valence-arousal.test.ts
    │   ├── plutchik.test.ts
    │   └── stress.test.ts
    ├── state/                  # State management tests
    │   ├── state-hasher.test.ts
    │   └── desired-state.test.ts
    └── integration/            # End-to-end integration tests
        └── full-flow.test.ts
```

### File Responsibilities

| File | Primary Responsibility | Lines of Code (Est.) |
|------|------------------------|----------------------|
| `detector.ts` | Main orchestration, API coordination | 250-300 |
| `gemini-client.ts` | Gemini API communication, retry logic | 200-250 |
| `valence-arousal.ts` | Russell's Circumplex mapping | 100-150 |
| `plutchik.ts` | 8D emotion vector generation | 150-200 |
| `stress.ts` | Stress level calculation | 100-150 |
| `state-hasher.ts` | State discretization for Q-learning | 80-100 |
| `desired-state.ts` | Desired state prediction heuristics | 150-200 |
| `validators.ts` | Input/response validation | 100-150 |
| `fallback.ts` | Fallback state generation | 50-80 |

---

## Class Diagrams

### Core Classes (ASCII Diagram)

```
┌─────────────────────────────────────────────────────────────┐
│                      EmotionDetector                         │
├─────────────────────────────────────────────────────────────┤
│ - geminiClient: GeminiClient                                 │
│ - agentDBClient: AgentDBClient                               │
│ - logger: Logger                                             │
│ - valenceMappeer: ValenceArousalMapper                       │
│ - plutchikMapper: PlutchikMapper                             │
│ - stressCalculator: StressCalculator                         │
│ - stateHasher: StateHasher                                   │
│ - desiredStatePredictor: DesiredStatePredictor               │
├─────────────────────────────────────────────────────────────┤
│ + analyzeText(text: string, userId: string):                 │
│     Promise<EmotionAnalysisResult>                           │
│ - callGeminiAPI(text: string, attempt: number):              │
│     Promise<GeminiResponse>                                  │
│ - validateInput(text: string): boolean                       │
│ - createFallbackState(userId: string): EmotionalState        │
│ - saveToAgentDB(state: EmotionalState): Promise<void>        │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ uses
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                      GeminiClient                            │
├─────────────────────────────────────────────────────────────┤
│ - apiKey: string                                             │
│ - timeout: number = 30000                                    │
│ - maxRetries: number = 3                                     │
│ - baseDelay: number = 1000                                   │
├─────────────────────────────────────────────────────────────┤
│ + generateContent(request: GeminiRequest):                   │
│     Promise<GeminiResponse>                                  │
│ - buildPrompt(text: string): string                          │
│ - parseResponse(raw: any): GeminiResponse                    │
│ - retryWithBackoff<T>(fn: () => Promise<T>, attempt: number):│
│     Promise<T>                                               │
│ - createTimeoutPromise(ms: number): Promise<never>           │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ uses
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  ValenceArousalMapper                        │
├─────────────────────────────────────────────────────────────┤
│ - NEUTRAL_VALENCE: number = 0.0                              │
│ - NEUTRAL_AROUSAL: number = 0.0                              │
│ - MAX_MAGNITUDE: number = 1.414                              │
├─────────────────────────────────────────────────────────────┤
│ + map(geminiResponse: GeminiResponse):                       │
│     {valence: number, arousal: number}                       │
│ - validateRange(value: number, min: number, max: number):    │
│     boolean                                                  │
│ - normalizeToCircumplex(v: number, a: number):               │
│     {valence: number, arousal: number}                       │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                     PlutchikMapper                           │
├─────────────────────────────────────────────────────────────┤
│ - PLUTCHIK_EMOTIONS: string[] = [                            │
│     "joy", "sadness", "anger", "fear",                       │
│     "trust", "disgust", "surprise", "anticipation"           │
│   ]                                                          │
│ - OPPOSITE_PAIRS: Map<string, string>                        │
│ - ADJACENT_EMOTIONS: Map<string, string[]>                   │
├─────────────────────────────────────────────────────────────┤
│ + generateVector(primaryEmotion: string, intensity: number): │
│     Float32Array                                             │
│ - getEmotionIndex(emotion: string): number                   │
│ - getAdjacentEmotions(emotion: string): string[]             │
│ - getOppositeEmotion(emotion: string): string                │
│ - normalizeVector(vector: Float32Array): Float32Array        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    StressCalculator                          │
├─────────────────────────────────────────────────────────────┤
│ - Q1_WEIGHT: number = 0.3  // High arousal + Positive       │
│ - Q2_WEIGHT: number = 0.9  // High arousal + Negative       │
│ - Q3_WEIGHT: number = 0.6  // Low arousal + Negative        │
│ - Q4_WEIGHT: number = 0.1  // Low arousal + Positive        │
├─────────────────────────────────────────────────────────────┤
│ + calculate(valence: number, arousal: number): number        │
│ - getQuadrantWeight(v: number, a: number): number            │
│ - calculateEmotionalIntensity(v: number, a: number): number  │
│ - applyNegativeBoost(stress: number, valence: number): number│
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                   DesiredStatePredictor                      │
├─────────────────────────────────────────────────────────────┤
│ - STRESS_THRESHOLD: number = 0.6                             │
│ - LOW_MOOD_THRESHOLD: number = -0.3                          │
│ - HIGH_AROUSAL_THRESHOLD: number = 0.5                       │
├─────────────────────────────────────────────────────────────┤
│ + predict(currentState: EmotionalState): DesiredState        │
│ - applyStressRule(state: EmotionalState): DesiredState | null│
│ - applyLowMoodRule(state: EmotionalState): DesiredState | null│
│ - applyAnxiousRule(state: EmotionalState): DesiredState | null│
│ - getDefaultDesiredState(state: EmotionalState): DesiredState│
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                       StateHasher                            │
├─────────────────────────────────────────────────────────────┤
│ - VALENCE_BUCKETS: number = 5                                │
│ - AROUSAL_BUCKETS: number = 5                                │
│ - STRESS_BUCKETS: number = 3                                 │
├─────────────────────────────────────────────────────────────┤
│ + hashState(state: EmotionalState): string                   │
│ - discretizeValue(value: number, buckets: number): number    │
└─────────────────────────────────────────────────────────────┘
```

### Class Relationships

```
EmotionDetector
    │
    ├── uses ──> GeminiClient (API communication)
    ├── uses ──> ValenceArousalMapper (emotion mapping)
    ├── uses ──> PlutchikMapper (vector generation)
    ├── uses ──> StressCalculator (stress computation)
    ├── uses ──> StateHasher (state discretization)
    ├── uses ──> DesiredStatePredictor (prediction logic)
    └── uses ──> AgentDBClient (persistence)

GeminiClient
    │
    └── uses ──> axios/fetch (HTTP client)

All components
    │
    └── uses ──> Logger (structured logging)
```

---

## TypeScript Interfaces

### Core Types

```typescript
/**
 * Emotional state derived from text analysis
 */
export interface EmotionalState {
  /** Unique identifier for this emotional state (UUID v4) */
  emotionalStateId: string;

  /** User identifier */
  userId: string;

  /** Valence: emotional pleasantness (-1.0 to +1.0) */
  valence: number;

  /** Arousal: emotional activation level (-1.0 to +1.0) */
  arousal: number;

  /** Primary emotion from Plutchik's 8 basic emotions */
  primaryEmotion: PlutchikEmotion;

  /** 8D emotion vector (normalized to sum to 1.0) */
  emotionVector: Float32Array;

  /** Stress level (0.0 to 1.0) */
  stressLevel: number;

  /** Confidence in this analysis (0.0 to 1.0) */
  confidence: number;

  /** Unix timestamp in milliseconds */
  timestamp: number;

  /** Original input text */
  rawText: string;
}

/**
 * Desired emotional state predicted from current state
 */
export interface DesiredState {
  /** Target valence (-1.0 to +1.0) */
  valence: number;

  /** Target arousal (-1.0 to +1.0) */
  arousal: number;

  /** Confidence in this prediction (0.0 to 1.0) */
  confidence: number;

  /** Human-readable reasoning for this prediction */
  reasoning: string;
}

/**
 * Complete emotion analysis result
 */
export interface EmotionAnalysisResult {
  /** Current emotional state */
  emotionalState: EmotionalState;

  /** Predicted desired state */
  desiredState: DesiredState;

  /** Total processing time in milliseconds */
  processingTime: number;
}

/**
 * Plutchik's 8 basic emotions
 */
export type PlutchikEmotion =
  | 'joy'
  | 'sadness'
  | 'anger'
  | 'fear'
  | 'trust'
  | 'disgust'
  | 'surprise'
  | 'anticipation';

/**
 * Gemini API response structure
 */
export interface GeminiResponse {
  /** Primary emotion detected */
  primaryEmotion: PlutchikEmotion;

  /** Valence value from Gemini */
  valence: number;

  /** Arousal value from Gemini */
  arousal: number;

  /** Stress level from Gemini */
  stressLevel: number;

  /** Gemini's confidence in this analysis */
  confidence: number;

  /** Gemini's explanation */
  reasoning: string;

  /** Full raw API response */
  rawResponse: any;
}

/**
 * Gemini API request configuration
 */
export interface GeminiRequest {
  /** Model to use (default: gemini-2.0-flash-exp) */
  model: string;

  /** Input text to analyze */
  text: string;

  /** Temperature (0.0-1.0, default: 0.3) */
  temperature?: number;

  /** Max output tokens (default: 256) */
  maxOutputTokens?: number;

  /** Response MIME type (default: application/json) */
  responseMimeType?: string;
}

/**
 * Error types for emotion detection
 */
export class EmotionDetectionError extends Error {
  constructor(message: string, public code: EmotionErrorCode) {
    super(message);
    this.name = 'EmotionDetectionError';
  }
}

export enum EmotionErrorCode {
  INVALID_INPUT = 'INVALID_INPUT',
  API_TIMEOUT = 'API_TIMEOUT',
  API_RATE_LIMIT = 'API_RATE_LIMIT',
  API_ERROR = 'API_ERROR',
  PARSE_ERROR = 'PARSE_ERROR',
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  AGENTDB_ERROR = 'AGENTDB_ERROR',
}

/**
 * Configuration for EmotionDetector
 */
export interface EmotionDetectorConfig {
  /** Gemini API key */
  geminiApiKey: string;

  /** API timeout in milliseconds (default: 30000) */
  timeout?: number;

  /** Max retry attempts (default: 3) */
  maxRetries?: number;

  /** Base retry delay in milliseconds (default: 1000) */
  retryDelay?: number;

  /** AgentDB connection URL */
  agentDBUrl: string;

  /** Enable debug logging (default: false) */
  debug?: boolean;
}
```

### Service Interfaces

```typescript
/**
 * Main emotion detection interface
 */
export interface IEmotionDetector {
  /**
   * Analyze text and return emotional state with desired state prediction
   * @param text - Input text to analyze (3-5000 characters)
   * @param userId - User identifier
   * @returns Complete emotion analysis result
   * @throws EmotionDetectionError on failure
   */
  analyzeText(text: string, userId: string): Promise<EmotionAnalysisResult>;

  /**
   * Get emotional history for a user
   * @param userId - User identifier
   * @param limit - Max number of results (default: 10)
   * @param fromTimestamp - Filter results after this timestamp
   * @returns Array of historical emotional states
   */
  getEmotionalHistory(
    userId: string,
    limit?: number,
    fromTimestamp?: number
  ): Promise<EmotionalState[]>;

  /**
   * Find similar emotional states using vector similarity
   * @param targetState - Target emotional state
   * @param topK - Number of similar states to return (default: 5)
   * @returns Array of similar states with similarity scores
   */
  findSimilarStates(
    targetState: EmotionalState,
    topK?: number
  ): Promise<Array<{ state: EmotionalState; similarity: number }>>;
}

/**
 * Gemini API client interface
 */
export interface IGeminiClient {
  /**
   * Generate content using Gemini API with retry logic
   * @param request - Gemini API request
   * @returns Parsed Gemini response
   * @throws EmotionDetectionError on failure
   */
  generateContent(request: GeminiRequest): Promise<GeminiResponse>;
}

/**
 * Valence-arousal mapper interface
 */
export interface IValenceArousalMapper {
  /**
   * Map Gemini response to Russell's Circumplex coordinates
   * @param response - Gemini API response
   * @returns Valence and arousal values
   */
  map(response: GeminiResponse): { valence: number; arousal: number };
}

/**
 * Plutchik emotion vector mapper interface
 */
export interface IPlutchikMapper {
  /**
   * Generate 8D emotion vector based on primary emotion
   * @param primaryEmotion - Primary emotion (one of Plutchik's 8)
   * @param intensity - Intensity (0.0 to 1.0)
   * @returns Normalized 8D emotion vector
   */
  generateVector(primaryEmotion: PlutchikEmotion, intensity: number): Float32Array;
}

/**
 * Stress calculator interface
 */
export interface IStressCalculator {
  /**
   * Calculate stress level from valence and arousal
   * @param valence - Valence value (-1.0 to +1.0)
   * @param arousal - Arousal value (-1.0 to +1.0)
   * @returns Stress level (0.0 to 1.0)
   */
  calculate(valence: number, arousal: number): number;
}

/**
 * Desired state predictor interface
 */
export interface IDesiredStatePredictor {
  /**
   * Predict desired emotional state from current state
   * @param currentState - Current emotional state
   * @returns Predicted desired state
   */
  predict(currentState: EmotionalState): DesiredState;
}

/**
 * State hasher interface
 */
export interface IStateHasher {
  /**
   * Hash emotional state for Q-learning state space
   * @param state - Emotional state to hash
   * @returns State hash string (e.g., "2:3:1")
   */
  hashState(state: EmotionalState): string;
}
```

---

## Sequence Diagrams

### Primary Flow: Text Analysis (Happy Path)

```
User → EmotionDetector → GeminiClient → Gemini API
  │           │                │              │
  │  analyze  │                │              │
  │  Text()   │                │              │
  ├──────────>│                │              │
  │           │ validate       │              │
  │           │ Input()        │              │
  │           │───────┐        │              │
  │           │       │        │              │
  │           │<──────┘        │              │
  │           │                │              │
  │           │ generateContent()             │
  │           ├───────────────>│              │
  │           │                │  POST /v1/   │
  │           │                │  models/     │
  │           │                │  gemini:     │
  │           │                │  generate    │
  │           │                ├─────────────>│
  │           │                │              │
  │           │                │ JSON response│
  │           │                │<─────────────│
  │           │                │              │
  │           │ GeminiResponse │              │
  │           │<───────────────┤              │
  │           │                │              │
  │           │ map()          │              │
  │           ├───> ValenceArousalMapper      │
  │           │<───┤            │              │
  │           │                │              │
  │           │ generateVector()│              │
  │           ├───> PlutchikMapper             │
  │           │<───┤            │              │
  │           │                │              │
  │           │ calculate()    │              │
  │           ├───> StressCalculator           │
  │           │<───┤            │              │
  │           │                │              │
  │           │ predict()      │              │
  │           ├───> DesiredStatePredictor      │
  │           │<───┤            │              │
  │           │                │              │
  │           │ saveToAgentDB()│              │
  │           ├───> AgentDB    │              │
  │           │    (async)     │              │
  │           │                │              │
  │ EmotionAnalysisResult      │              │
  │<──────────┤                │              │
  │           │                │              │
```

### Error Flow: API Timeout with Retry

```
EmotionDetector → GeminiClient → Gemini API
      │                 │              │
      │ generateContent()│              │
      ├────────────────>│              │
      │                 │  POST /v1/   │
      │                 │  (Attempt 1) │
      │                 ├─────────────>│
      │                 │              │
      │                 │  (30s timeout│
      │                 │   exceeded)  │
      │                 │      ✗       │
      │                 │              │
      │                 │ SLEEP(1000ms)│
      │                 │──────┐       │
      │                 │      │       │
      │                 │<─────┘       │
      │                 │              │
      │                 │  POST /v1/   │
      │                 │  (Attempt 2) │
      │                 ├─────────────>│
      │                 │              │
      │                 │  (30s timeout│
      │                 │   exceeded)  │
      │                 │      ✗       │
      │                 │              │
      │                 │ SLEEP(2000ms)│
      │                 │──────┐       │
      │                 │      │       │
      │                 │<─────┘       │
      │                 │              │
      │                 │  POST /v1/   │
      │                 │  (Attempt 3) │
      │                 ├─────────────>│
      │                 │              │
      │                 │ JSON response│
      │                 │<─────────────│
      │                 │              │
      │ GeminiResponse  │              │
      │<────────────────┤              │
      │                 │              │
```

### Error Flow: All Retries Failed (Fallback)

```
EmotionDetector → GeminiClient → Gemini API
      │                 │              │
      │ generateContent()│              │
      ├────────────────>│              │
      │                 │  POST /v1/   │
      │                 │  (Attempt 1) │
      │                 ├─────────────>│
      │                 │      ✗       │
      │                 │              │
      │                 │  (Attempt 2) │
      │                 ├─────────────>│
      │                 │      ✗       │
      │                 │              │
      │                 │  (Attempt 3) │
      │                 ├─────────────>│
      │                 │      ✗       │
      │                 │              │
      │ EmotionDetectionError          │
      │<────────────────┤              │
      │                 │              │
      │ createFallback  │              │
      │ State()         │              │
      │───────┐         │              │
      │       │         │              │
      │<──────┘         │              │
      │                 │              │
      │ EmotionAnalysisResult          │
      │ (neutral state, │              │
      │  confidence=0)  │              │
      │                 │              │
      │ [Logged Warning:│              │
      │  "Fallback state│              │
      │   created"]     │              │
```

### Desired State Prediction Flow

```
EmotionDetector → DesiredStatePredictor
      │                   │
      │ predict(          │
      │  currentState)    │
      ├──────────────────>│
      │                   │
      │                   │ Check stress
      │                   │ level > 0.6?
      │                   │───────┐
      │                   │       │
      │                   │<──────┘
      │                   │  YES
      │                   │
      │                   │ Return:
      │                   │ valence=0.5
      │                   │ arousal=-0.4
      │                   │ (calming)
      │                   │
      │ DesiredState      │
      │ (calming content) │
      │<──────────────────┤
      │                   │
```

---

## Component Architecture

### Dependency Injection

```typescript
/**
 * Dependency injection container for EmotionDetector
 */
export class EmotionDetectorFactory {
  private static instance: EmotionDetector | null = null;

  static create(config: EmotionDetectorConfig): EmotionDetector {
    // Create dependencies
    const logger = new Logger({ debug: config.debug });

    const geminiClient = new GeminiClient({
      apiKey: config.geminiApiKey,
      timeout: config.timeout ?? 30000,
      maxRetries: config.maxRetries ?? 3,
      retryDelay: config.retryDelay ?? 1000,
      logger,
    });

    const agentDBClient = new AgentDBClient({
      url: config.agentDBUrl,
      logger,
    });

    const valenceMappeer = new ValenceArousalMapper(logger);
    const plutchikMapper = new PlutchikMapper(logger);
    const stressCalculator = new StressCalculator(logger);
    const stateHasher = new StateHasher();
    const desiredStatePredictor = new DesiredStatePredictor(logger);

    // Create and return EmotionDetector
    return new EmotionDetector({
      geminiClient,
      agentDBClient,
      logger,
      valenceMappeer,
      plutchikMapper,
      stressCalculator,
      stateHasher,
      desiredStatePredictor,
    });
  }

  /**
   * Get singleton instance (useful for testing)
   */
  static getInstance(config?: EmotionDetectorConfig): EmotionDetector {
    if (!this.instance && !config) {
      throw new Error('EmotionDetector not initialized. Provide config.');
    }

    if (config) {
      this.instance = this.create(config);
    }

    return this.instance!;
  }

  /**
   * Reset singleton (useful for testing)
   */
  static reset(): void {
    this.instance = null;
  }
}
```

### Module Exports

```typescript
// src/emotion/index.ts

/**
 * Main EmotionDetector module entry point
 */

// Export main class
export { EmotionDetector } from './detector';
export { EmotionDetectorFactory } from './factory';

// Export types
export type {
  EmotionalState,
  DesiredState,
  EmotionAnalysisResult,
  PlutchikEmotion,
  GeminiResponse,
  GeminiRequest,
  EmotionDetectorConfig,
  IEmotionDetector,
  IGeminiClient,
  IValenceArousalMapper,
  IPlutchikMapper,
  IStressCalculator,
  IDesiredStatePredictor,
  IStateHasher,
} from './types';

// Export errors
export { EmotionDetectionError, EmotionErrorCode } from './types';

// Export utilities (optional, for advanced usage)
export { validateText, validateGeminiResponse } from './utils/validators';
export { createFallbackState } from './utils/fallback';

/**
 * Example usage:
 *
 * ```typescript
 * import { EmotionDetectorFactory } from './emotion';
 *
 * const detector = EmotionDetectorFactory.create({
 *   geminiApiKey: process.env.GEMINI_API_KEY,
 *   agentDBUrl: process.env.AGENTDB_URL,
 *   debug: true,
 * });
 *
 * const result = await detector.analyzeText(
 *   "I'm feeling stressed and anxious",
 *   "user_12345"
 * );
 *
 * console.log(result.emotionalState);
 * console.log(result.desiredState);
 * ```
 */
```

### Configuration Management

```typescript
/**
 * Configuration loader with environment variable support
 */
export class EmotionDetectorConfigLoader {
  static loadFromEnv(): EmotionDetectorConfig {
    const geminiApiKey = process.env.GEMINI_API_KEY;
    const agentDBUrl = process.env.AGENTDB_URL || 'redis://localhost:6379';

    if (!geminiApiKey) {
      throw new Error('GEMINI_API_KEY environment variable is required');
    }

    return {
      geminiApiKey,
      agentDBUrl,
      timeout: parseInt(process.env.EMOTION_API_TIMEOUT || '30000', 10),
      maxRetries: parseInt(process.env.EMOTION_MAX_RETRIES || '3', 10),
      retryDelay: parseInt(process.env.EMOTION_RETRY_DELAY || '1000', 10),
      debug: process.env.NODE_ENV === 'development',
    };
  }

  static loadFromFile(configPath: string): EmotionDetectorConfig {
    const fs = require('fs');
    const path = require('path');

    const absolutePath = path.resolve(configPath);
    const configJson = fs.readFileSync(absolutePath, 'utf-8');
    const config = JSON.parse(configJson);

    this.validate(config);

    return config;
  }

  private static validate(config: any): void {
    if (!config.geminiApiKey) {
      throw new Error('geminiApiKey is required in config');
    }

    if (!config.agentDBUrl) {
      throw new Error('agentDBUrl is required in config');
    }

    if (config.timeout && (config.timeout < 1000 || config.timeout > 60000)) {
      throw new Error('timeout must be between 1000ms and 60000ms');
    }

    if (config.maxRetries && (config.maxRetries < 1 || config.maxRetries > 10)) {
      throw new Error('maxRetries must be between 1 and 10');
    }
  }
}
```

---

## Error Handling Strategy

### Error Hierarchy

```
Error
  │
  └── EmotionDetectionError
        │
        ├── InvalidInputError (INVALID_INPUT)
        │     • Text too short (<3 chars)
        │     • Text too long (>5000 chars)
        │     • No alphanumeric characters
        │     • Empty/null/undefined input
        │
        ├── GeminiAPIError (API_ERROR)
        │     • Invalid API key
        │     • Service unavailable
        │     • Unexpected API response
        │
        ├── GeminiTimeoutError (API_TIMEOUT)
        │     • Request exceeded 30s timeout
        │     • All retry attempts failed
        │
        ├── GeminiRateLimitError (API_RATE_LIMIT)
        │     • 429 Too Many Requests
        │     • Quota exceeded
        │
        ├── ParseError (PARSE_ERROR)
        │     • Invalid JSON in Gemini response
        │     • Missing required fields
        │
        ├── ValidationError (VALIDATION_ERROR)
        │     • Invalid valence/arousal range
        │     • Invalid emotion type
        │     • Inconsistent response data
        │
        └── AgentDBError (AGENTDB_ERROR)
              • Connection failure
              • Save operation failed
              • Query failure
```

### Error Handling Implementation

```typescript
/**
 * Custom error classes
 */
export class GeminiTimeoutError extends EmotionDetectionError {
  constructor(message: string = 'Gemini API request timed out') {
    super(message, EmotionErrorCode.API_TIMEOUT);
    this.name = 'GeminiTimeoutError';
  }
}

export class GeminiRateLimitError extends EmotionDetectionError {
  constructor(message: string = 'Gemini API rate limit exceeded') {
    super(message, EmotionErrorCode.API_RATE_LIMIT);
    this.name = 'GeminiRateLimitError';
  }
}

export class InvalidInputError extends EmotionDetectionError {
  constructor(message: string = 'Invalid input text') {
    super(message, EmotionErrorCode.INVALID_INPUT);
    this.name = 'InvalidInputError';
  }
}

export class ParseError extends EmotionDetectionError {
  constructor(message: string = 'Failed to parse Gemini response') {
    super(message, EmotionErrorCode.PARSE_ERROR);
    this.name = 'ParseError';
  }
}

/**
 * Error handler with logging and fallback
 */
export class EmotionDetectorErrorHandler {
  constructor(private logger: Logger) {}

  handle(error: Error, userId: string): EmotionalState {
    // Log error with context
    this.logger.error('EmotionDetector error', {
      error: error.message,
      stack: error.stack,
      userId,
      timestamp: Date.now(),
    });

    // Check if we should retry
    if (error instanceof GeminiTimeoutError) {
      this.logger.warn('Gemini API timeout, returning fallback state');
    } else if (error instanceof GeminiRateLimitError) {
      this.logger.warn('Rate limit exceeded, returning fallback state');
    } else if (error instanceof InvalidInputError) {
      this.logger.warn('Invalid input, returning fallback state');
    }

    // Return fallback state
    return createFallbackState(userId);
  }

  shouldRetry(error: Error, attemptNumber: number, maxAttempts: number): boolean {
    if (attemptNumber >= maxAttempts) {
      return false;
    }

    // Retry on timeout
    if (error instanceof GeminiTimeoutError) {
      return true;
    }

    // Retry on rate limit
    if (error instanceof GeminiRateLimitError) {
      return true;
    }

    // Don't retry on input errors
    if (error instanceof InvalidInputError) {
      return false;
    }

    // Don't retry on parse errors
    if (error instanceof ParseError) {
      return false;
    }

    // Retry on generic API errors
    return true;
  }
}
```

### Retry Policy

```typescript
/**
 * Retry configuration
 */
export interface RetryConfig {
  maxAttempts: number;
  baseDelay: number;
  maxDelay: number;
  exponentialBackoff: boolean;
  jitter: boolean;
}

export const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxAttempts: 3,
  baseDelay: 1000,        // 1 second
  maxDelay: 10000,        // 10 seconds
  exponentialBackoff: true,
  jitter: true,
};

/**
 * Calculate retry delay with exponential backoff and jitter
 */
export function calculateRetryDelay(
  attemptNumber: number,
  config: RetryConfig = DEFAULT_RETRY_CONFIG
): number {
  let delay = config.baseDelay;

  if (config.exponentialBackoff) {
    // Exponential backoff: delay = baseDelay * 2^(attempt - 1)
    delay = config.baseDelay * Math.pow(2, attemptNumber - 1);
  } else {
    // Linear backoff: delay = baseDelay * attempt
    delay = config.baseDelay * attemptNumber;
  }

  // Cap at maxDelay
  delay = Math.min(delay, config.maxDelay);

  // Add jitter (0-20% of delay)
  if (config.jitter) {
    const jitterAmount = Math.random() * 0.2 * delay;
    delay += jitterAmount;
  }

  return Math.floor(delay);
}

/**
 * Example retry delays:
 *
 * Attempt 1: ~1000ms  (1s + jitter)
 * Attempt 2: ~2000ms  (2s + jitter)
 * Attempt 3: ~4000ms  (4s + jitter)
 *
 * Total worst case: ~7 seconds for 3 retries
 */
```

---

## Testing Strategy

### Unit Tests

```typescript
/**
 * EmotionDetector unit tests
 */
describe('EmotionDetector', () => {
  let detector: EmotionDetector;
  let mockGeminiClient: jest.Mocked<IGeminiClient>;
  let mockAgentDB: jest.Mocked<AgentDBClient>;

  beforeEach(() => {
    mockGeminiClient = {
      generateContent: jest.fn(),
    } as any;

    mockAgentDB = {
      insert: jest.fn(),
      query: jest.fn(),
    } as any;

    detector = new EmotionDetector({
      geminiClient: mockGeminiClient,
      agentDBClient: mockAgentDB,
      logger: new Logger({ debug: false }),
      valenceMappeer: new ValenceArousalMapper(),
      plutchikMapper: new PlutchikMapper(),
      stressCalculator: new StressCalculator(),
      stateHasher: new StateHasher(),
      desiredStatePredictor: new DesiredStatePredictor(),
    });
  });

  describe('analyzeText()', () => {
    it('should analyze happy emotion correctly', async () => {
      // Mock Gemini response
      mockGeminiClient.generateContent.mockResolvedValue({
        primaryEmotion: 'joy',
        valence: 0.8,
        arousal: 0.6,
        stressLevel: 0.3,
        confidence: 0.9,
        reasoning: 'User expressed excitement',
        rawResponse: {},
      });

      const result = await detector.analyzeText(
        "I'm so excited about my promotion!",
        'user_123'
      );

      expect(result.emotionalState.primaryEmotion).toBe('joy');
      expect(result.emotionalState.valence).toBeGreaterThan(0.7);
      expect(result.emotionalState.arousal).toBeGreaterThan(0.5);
      expect(result.emotionalState.stressLevel).toBeLessThan(0.4);
      expect(result.desiredState).toBeDefined();
    });

    it('should handle API timeout with fallback', async () => {
      // Mock API timeout
      mockGeminiClient.generateContent.mockRejectedValue(
        new GeminiTimeoutError('Timeout after 30s')
      );

      const result = await detector.analyzeText(
        'Test text',
        'user_123'
      );

      expect(result.emotionalState.valence).toBe(0.0);
      expect(result.emotionalState.arousal).toBe(0.0);
      expect(result.emotionalState.confidence).toBe(0.0);
      expect(result.emotionalState.primaryEmotion).toBe('trust');
    });

    it('should reject invalid input', async () => {
      await expect(
        detector.analyzeText('ab', 'user_123')  // Too short
      ).rejects.toThrow(InvalidInputError);

      await expect(
        detector.analyzeText('', 'user_123')  // Empty
      ).rejects.toThrow(InvalidInputError);
    });
  });
});

/**
 * ValenceArousalMapper unit tests
 */
describe('ValenceArousalMapper', () => {
  let mapper: ValenceArousalMapper;

  beforeEach(() => {
    mapper = new ValenceArousalMapper();
  });

  it('should map valid Gemini response', () => {
    const response: GeminiResponse = {
      primaryEmotion: 'joy',
      valence: 0.8,
      arousal: 0.6,
      stressLevel: 0.3,
      confidence: 0.9,
      reasoning: 'Test',
      rawResponse: {},
    };

    const result = mapper.map(response);

    expect(result.valence).toBe(0.8);
    expect(result.arousal).toBe(0.6);
  });

  it('should normalize values outside circumplex', () => {
    const response: GeminiResponse = {
      primaryEmotion: 'joy',
      valence: 1.2,   // Out of range
      arousal: 1.0,
      stressLevel: 0.3,
      confidence: 0.9,
      reasoning: 'Test',
      rawResponse: {},
    };

    const result = mapper.map(response);

    // Should be normalized to unit circle
    const magnitude = Math.sqrt(result.valence ** 2 + result.arousal ** 2);
    expect(magnitude).toBeLessThanOrEqual(1.414);  // √2
  });
});

/**
 * PlutchikMapper unit tests
 */
describe('PlutchikMapper', () => {
  let mapper: PlutchikMapper;

  beforeEach(() => {
    mapper = new PlutchikMapper();
  });

  it('should generate normalized vector for joy', () => {
    const vector = mapper.generateVector('joy', 0.8);

    // Check vector is normalized (sums to 1.0)
    const sum = Array.from(vector).reduce((a, b) => a + b, 0);
    expect(sum).toBeCloseTo(1.0, 2);

    // Joy should be dominant
    expect(vector[0]).toBeGreaterThan(0.5);

    // Sadness (opposite) should be suppressed
    expect(vector[1]).toBe(0.0);

    // Adjacent emotions should have some weight
    expect(vector[4]).toBeGreaterThan(0.0);  // trust
    expect(vector[7]).toBeGreaterThan(0.0);  // anticipation
  });

  it('should handle all 8 emotions', () => {
    const emotions: PlutchikEmotion[] = [
      'joy', 'sadness', 'anger', 'fear',
      'trust', 'disgust', 'surprise', 'anticipation'
    ];

    emotions.forEach(emotion => {
      const vector = mapper.generateVector(emotion, 0.7);

      const sum = Array.from(vector).reduce((a, b) => a + b, 0);
      expect(sum).toBeCloseTo(1.0, 2);
    });
  });
});

/**
 * StressCalculator unit tests
 */
describe('StressCalculator', () => {
  let calculator: StressCalculator;

  beforeEach(() => {
    calculator = new StressCalculator();
  });

  it('should calculate high stress for Q2 (negative + high arousal)', () => {
    const stress = calculator.calculate(-0.8, 0.7);
    expect(stress).toBeGreaterThan(0.8);
  });

  it('should calculate low stress for Q4 (positive + low arousal)', () => {
    const stress = calculator.calculate(0.7, -0.4);
    expect(stress).toBeLessThan(0.2);
  });

  it('should boost stress for extreme negative valence', () => {
    const stress1 = calculator.calculate(-0.5, 0.5);
    const stress2 = calculator.calculate(-0.9, 0.5);

    expect(stress2).toBeGreaterThan(stress1);
  });
});

/**
 * DesiredStatePredictor unit tests
 */
describe('DesiredStatePredictor', () => {
  let predictor: DesiredStatePredictor;

  beforeEach(() => {
    predictor = new DesiredStatePredictor();
  });

  it('should predict calming state for high stress', () => {
    const currentState: EmotionalState = {
      emotionalStateId: 'test',
      userId: 'user_123',
      valence: -0.6,
      arousal: 0.7,
      primaryEmotion: 'fear',
      emotionVector: new Float32Array(8),
      stressLevel: 0.85,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'Test',
    };

    const desired = predictor.predict(currentState);

    expect(desired.arousal).toBeLessThan(0.0);  // Want calm
    expect(desired.valence).toBeGreaterThan(0.0);  // Want positive
    expect(desired.reasoning).toContain('stress');
  });

  it('should predict uplifting state for low mood', () => {
    const currentState: EmotionalState = {
      emotionalStateId: 'test',
      userId: 'user_123',
      valence: -0.7,
      arousal: -0.3,
      primaryEmotion: 'sadness',
      emotionVector: new Float32Array(8),
      stressLevel: 0.5,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'Test',
    };

    const desired = predictor.predict(currentState);

    expect(desired.valence).toBeGreaterThan(0.5);  // Want positive
    expect(desired.arousal).toBeGreaterThan(0.0);  // Want energizing
  });
});
```

### Integration Tests

```typescript
/**
 * Integration tests with real Gemini API
 */
describe('EmotionDetector Integration', () => {
  let detector: EmotionDetector;

  beforeAll(() => {
    const config = EmotionDetectorConfigLoader.loadFromEnv();
    detector = EmotionDetectorFactory.create(config);
  });

  it('should analyze real text end-to-end', async () => {
    const result = await detector.analyzeText(
      "I'm feeling stressed and anxious about my deadline tomorrow",
      'integration_test_user'
    );

    expect(result.emotionalState.valence).toBeLessThan(0.0);
    expect(result.emotionalState.stressLevel).toBeGreaterThan(0.6);
    expect(result.emotionalState.confidence).toBeGreaterThan(0.7);
    expect(result.desiredState.arousal).toBeLessThan(0.0);  // Want calming
    expect(result.processingTime).toBeLessThan(5000);  // Under 5s
  }, 10000);  // 10s timeout

  it('should persist to AgentDB', async () => {
    const result = await detector.analyzeText(
      'Test text for persistence',
      'persistence_test_user'
    );

    // Wait for async save
    await new Promise(resolve => setTimeout(resolve, 1000));

    const history = await detector.getEmotionalHistory(
      'persistence_test_user',
      1
    );

    expect(history.length).toBeGreaterThan(0);
    expect(history[0].emotionalStateId).toBe(
      result.emotionalState.emotionalStateId
    );
  }, 15000);
});
```

### Test Coverage Targets

| Component | Target Coverage | Critical Paths |
|-----------|----------------|----------------|
| `detector.ts` | 95% | API error handling, fallback logic |
| `gemini-client.ts` | 90% | Retry logic, timeout handling |
| `valence-arousal.ts` | 95% | Circumplex normalization |
| `plutchik.ts` | 95% | Vector generation, normalization |
| `stress.ts` | 95% | Quadrant calculations |
| `desired-state.ts` | 90% | Rule-based heuristics |
| `state-hasher.ts` | 100% | Discretization logic |
| `validators.ts` | 100% | All validation rules |
| `fallback.ts` | 100% | Fallback state generation |

---

## Performance Considerations

### Bottleneck Analysis

```
Performance Bottlenecks (Ranked by Impact):

1. Gemini API Latency (CRITICAL)
   - Average: 2-3 seconds per request
   - P95: 4-5 seconds
   - Mitigation: Caching, timeout enforcement

2. Network Retries (HIGH)
   - Worst case: 3 retries × 30s = 90s total
   - Mitigation: Exponential backoff, circuit breaker

3. AgentDB Write Operations (MEDIUM)
   - Average: 50-100ms per write
   - Mitigation: Async, non-blocking writes

4. Emotion Vector Computation (LOW)
   - Average: <1ms
   - Negligible impact
```

### Optimization Strategies

```typescript
/**
 * Caching layer for emotion detection
 */
export class EmotionDetectorCache {
  private cache: Map<string, { result: EmotionAnalysisResult; timestamp: number }>;
  private ttl: number;

  constructor(ttl: number = 5 * 60 * 1000) {  // 5 minutes default
    this.cache = new Map();
    this.ttl = ttl;
  }

  /**
   * Generate cache key from text
   */
  private getCacheKey(text: string, userId: string): string {
    const crypto = require('crypto');
    const hash = crypto.createHash('sha256')
      .update(`${userId}:${text}`)
      .digest('hex');
    return hash;
  }

  /**
   * Get cached result if available and not expired
   */
  get(text: string, userId: string): EmotionAnalysisResult | null {
    const key = this.getCacheKey(text, userId);
    const cached = this.cache.get(key);

    if (!cached) {
      return null;
    }

    const age = Date.now() - cached.timestamp;
    if (age > this.ttl) {
      this.cache.delete(key);
      return null;
    }

    return cached.result;
  }

  /**
   * Store result in cache
   */
  set(text: string, userId: string, result: EmotionAnalysisResult): void {
    const key = this.getCacheKey(text, userId);
    this.cache.set(key, {
      result,
      timestamp: Date.now(),
    });
  }

  /**
   * Clear expired entries
   */
  cleanup(): void {
    const now = Date.now();
    for (const [key, value] of this.cache.entries()) {
      if (now - value.timestamp > this.ttl) {
        this.cache.delete(key);
      }
    }
  }
}

/**
 * EmotionDetector with caching
 */
export class CachedEmotionDetector implements IEmotionDetector {
  private cache: EmotionDetectorCache;

  constructor(
    private detector: EmotionDetector,
    cacheTTL?: number
  ) {
    this.cache = new EmotionDetectorCache(cacheTTL);

    // Cleanup expired cache entries every 5 minutes
    setInterval(() => this.cache.cleanup(), 5 * 60 * 1000);
  }

  async analyzeText(text: string, userId: string): Promise<EmotionAnalysisResult> {
    // Check cache first
    const cached = this.cache.get(text, userId);
    if (cached) {
      return cached;
    }

    // Call underlying detector
    const result = await this.detector.analyzeText(text, userId);

    // Store in cache
    this.cache.set(text, userId, result);

    return result;
  }

  // Delegate other methods to underlying detector
  getEmotionalHistory(userId: string, limit?: number, fromTimestamp?: number) {
    return this.detector.getEmotionalHistory(userId, limit, fromTimestamp);
  }

  findSimilarStates(targetState: EmotionalState, topK?: number) {
    return this.detector.findSimilarStates(targetState, topK);
  }
}
```

### Performance Metrics

```typescript
/**
 * Performance monitoring
 */
export class EmotionDetectorMetrics {
  private metrics: {
    totalRequests: number;
    successfulRequests: number;
    failedRequests: number;
    fallbackRequests: number;
    totalLatency: number;
    apiLatency: number;
    cacheHits: number;
    cacheMisses: number;
  };

  constructor() {
    this.metrics = {
      totalRequests: 0,
      successfulRequests: 0,
      failedRequests: 0,
      fallbackRequests: 0,
      totalLatency: 0,
      apiLatency: 0,
      cacheHits: 0,
      cacheMisses: 0,
    };
  }

  recordRequest(
    success: boolean,
    isFallback: boolean,
    latency: number,
    apiLatency: number,
    cacheHit: boolean
  ): void {
    this.metrics.totalRequests++;

    if (success) {
      this.metrics.successfulRequests++;
    } else {
      this.metrics.failedRequests++;
    }

    if (isFallback) {
      this.metrics.fallbackRequests++;
    }

    this.metrics.totalLatency += latency;
    this.metrics.apiLatency += apiLatency;

    if (cacheHit) {
      this.metrics.cacheHits++;
    } else {
      this.metrics.cacheMisses++;
    }
  }

  getReport() {
    const avgLatency = this.metrics.totalLatency / this.metrics.totalRequests;
    const avgApiLatency = this.metrics.apiLatency / this.metrics.totalRequests;
    const successRate = this.metrics.successfulRequests / this.metrics.totalRequests;
    const fallbackRate = this.metrics.fallbackRequests / this.metrics.totalRequests;
    const cacheHitRate = this.metrics.cacheHits / (this.metrics.cacheHits + this.metrics.cacheMisses);

    return {
      totalRequests: this.metrics.totalRequests,
      successRate: (successRate * 100).toFixed(2) + '%',
      fallbackRate: (fallbackRate * 100).toFixed(2) + '%',
      cacheHitRate: (cacheHitRate * 100).toFixed(2) + '%',
      avgLatency: avgLatency.toFixed(2) + 'ms',
      avgApiLatency: avgApiLatency.toFixed(2) + 'ms',
    };
  }
}
```

---

## Deployment Architecture

### Docker Container

```dockerfile
# Dockerfile for EmotionDetector service

FROM node:18-alpine

# Install dependencies
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

# Copy source code
COPY dist/ ./dist/

# Environment variables
ENV NODE_ENV=production
ENV GEMINI_API_KEY=""
ENV AGENTDB_URL="redis://agentdb:6379"
ENV EMOTION_API_TIMEOUT=30000
ENV EMOTION_MAX_RETRIES=3

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD node dist/health-check.js

# Run service
EXPOSE 3001
CMD ["node", "dist/index.js"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: emotion-detector
  labels:
    app: emotion-detector
spec:
  replicas: 3
  selector:
    matchLabels:
      app: emotion-detector
  template:
    metadata:
      labels:
        app: emotion-detector
    spec:
      containers:
      - name: emotion-detector
        image: emotistream/emotion-detector:1.0.0
        ports:
        - containerPort: 3001
        env:
        - name: NODE_ENV
          value: "production"
        - name: GEMINI_API_KEY
          valueFrom:
            secretKeyRef:
              name: gemini-secret
              key: api-key
        - name: AGENTDB_URL
          value: "redis://agentdb:6379"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3001
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3001
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: emotion-detector
spec:
  selector:
    app: emotion-detector
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3001
  type: ClusterIP
```

### Service Mesh Integration

```yaml
# Istio VirtualService for traffic management
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: emotion-detector
spec:
  hosts:
  - emotion-detector
  http:
  - timeout: 35s  # Slightly higher than API timeout
    retries:
      attempts: 3
      perTryTimeout: 12s
      retryOn: 5xx,reset,connect-failure,refused-stream
    route:
    - destination:
        host: emotion-detector
        subset: v1
      weight: 100
```

---

## Summary

The EmotionDetector module architecture provides:

✅ **Modular Design**: Clear separation of concerns with single-responsibility classes
✅ **Robust Error Handling**: Comprehensive error hierarchy with retry logic and fallbacks
✅ **Performance Optimization**: Caching, async operations, timeout enforcement
✅ **Testability**: Dependency injection, mock-friendly interfaces, 95%+ coverage target
✅ **Scalability**: Stateless design, horizontal scaling, containerized deployment
✅ **Maintainability**: Well-documented interfaces, structured logging, metrics tracking

### Next Steps

1. **Implementation Phase**: Begin coding based on this architecture
2. **Unit Testing**: Achieve 95% code coverage
3. **Integration Testing**: Test with real Gemini API
4. **Performance Testing**: Validate latency and throughput targets
5. **Documentation**: Update API docs and usage examples

---

**Document Status**: Complete
**Review Status**: Ready for implementation
**Next Phase**: SPARC Refinement (TDD implementation)
