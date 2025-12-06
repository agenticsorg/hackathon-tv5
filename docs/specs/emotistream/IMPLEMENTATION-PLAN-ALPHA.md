# EmotiStream MVP Alpha - Implementation Plan

**Version**: 1.0
**Created**: 2025-12-06
**Target**: Alpha Release for User Testing
**Methodology**: Claude-Flow Swarm with Agentic QE

---

## Executive Summary

This plan addresses the gaps identified in the Brutal Honesty Review to deliver a fully functional EmotiStream MVP ready for alpha user testing.

### Current State
- **Compliance**: 37.5% of spec implemented
- **Integration**: 20% - modules exist but aren't connected
- **Production Readiness**: 15%

### Target State
- **Compliance**: 100% of MVP spec
- **Integration**: 100% - fully wired end-to-end
- **Production Readiness**: 85% (alpha-ready)

---

## Phase 1: Critical Integration (P0)

**Duration**: 4-6 hours
**Agents**: `backend-dev`, `coder`, `qe-integration-tester`

### 1.1 Wire API Endpoints to Modules

**Task**: Connect existing modules to API routes

**Files to Modify**:
```
src/api/routes/emotion.ts    → Import and use EmotionDetector
src/api/routes/recommend.ts  → Import and use RecommendationEngine
src/api/routes/feedback.ts   → Import and use FeedbackProcessor
```

**Implementation Steps**:

```typescript
// src/api/routes/emotion.ts
import { EmotionDetector } from '../../emotion/detector';
import { EmotionDetectorService } from '../../services/emotion-service';

// Create singleton service
const emotionService = new EmotionDetectorService();

router.post('/analyze', async (req, res, next) => {
  try {
    const { userId, text, desiredMood } = req.body;

    // Call real detector
    const result = await emotionService.analyze(userId, text);
    const desired = desiredMood
      ? await emotionService.parseDesiredState(desiredMood)
      : result.desiredState;

    res.json({
      success: true,
      data: {
        userId,
        currentState: result.currentState,
        desiredState: desired,
        stateHash: result.stateHash
      },
      error: null,
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    next(error);
  }
});
```

**Acceptance Criteria**:
- [ ] POST /api/v1/emotion/analyze returns real EmotionDetector output
- [ ] POST /api/v1/recommend returns real RecommendationEngine output
- [ ] POST /api/v1/feedback returns real FeedbackProcessor output
- [ ] All modules share consistent state

---

### 1.2 Create Service Layer

**Task**: Build service layer to manage module lifecycles and dependencies

**New Files**:
```
src/services/
├── emotion-service.ts       # EmotionDetector wrapper
├── recommendation-service.ts # RecommendationEngine wrapper
├── feedback-service.ts      # FeedbackProcessor wrapper
├── policy-service.ts        # RLPolicyEngine wrapper
└── index.ts                 # Service container/DI
```

**Implementation**:

```typescript
// src/services/index.ts
import { EmotionDetector } from '../emotion/detector';
import { RLPolicyEngine } from '../rl/policy-engine';
import { RecommendationEngine } from '../recommendations/engine';
import { FeedbackProcessor } from '../feedback/processor';
import { QTable } from '../rl/q-table';
import { ContentProfiler } from '../content/profiler';

export class ServiceContainer {
  private static instance: ServiceContainer;

  readonly emotionDetector: EmotionDetector;
  readonly policyEngine: RLPolicyEngine;
  readonly recommendationEngine: RecommendationEngine;
  readonly feedbackProcessor: FeedbackProcessor;
  readonly qTable: QTable;
  readonly contentProfiler: ContentProfiler;

  private constructor() {
    // Initialize in correct order
    this.qTable = new QTable();
    this.emotionDetector = new EmotionDetector();
    this.contentProfiler = new ContentProfiler();
    this.policyEngine = new RLPolicyEngine(this.qTable, ...);
    this.recommendationEngine = new RecommendationEngine(
      this.policyEngine,
      this.contentProfiler
    );
    this.feedbackProcessor = new FeedbackProcessor(
      this.policyEngine,
      this.qTable
    );
  }

  static getInstance(): ServiceContainer {
    if (!ServiceContainer.instance) {
      ServiceContainer.instance = new ServiceContainer();
    }
    return ServiceContainer.instance;
  }
}
```

**Acceptance Criteria**:
- [ ] Single service container manages all module instances
- [ ] Dependency injection pattern followed
- [ ] Services are singleton per server instance

---

## Phase 2: Persistence Layer (P0)

**Duration**: 4-6 hours
**Agents**: `backend-dev`, `coder`, `qe-test-generator`

### 2.1 AgentDB Integration

**Task**: Add AgentDB for Q-table and user data persistence

**Dependencies to Add**:
```bash
npm install agentdb better-sqlite3
npm install -D @types/better-sqlite3
```

**New Files**:
```
src/persistence/
├── agentdb-client.ts        # AgentDB wrapper
├── q-table-store.ts         # Q-table persistence
├── user-store.ts            # User profile persistence
├── experience-store.ts      # Experience replay persistence
└── migrations/
    └── 001-initial-schema.ts
```

**Key Patterns** (per spec):
```typescript
// Key patterns from API-EmotiStream-MVP.md
const keys = {
  user: (userId: string) => `user:${userId}`,
  userExperiences: (userId: string) => `user:${userId}:experiences`,
  emotionalState: (stateId: string) => `state:${stateId}`,
  experience: (expId: string) => `exp:${expId}`,
  qValue: (userId: string, stateHash: string, contentId: string) =>
    `q:${userId}:${stateHash}:${contentId}`,
  content: (contentId: string) => `content:${contentId}`,
};
```

**Implementation**:

```typescript
// src/persistence/q-table-store.ts
import { AgentDB } from 'agentdb';
import { QTableEntry } from '../rl/types';

export class QTableStore {
  constructor(private db: AgentDB) {}

  async get(userId: string, stateHash: string, contentId: string): Promise<QTableEntry | null> {
    const key = `q:${userId}:${stateHash}:${contentId}`;
    return this.db.get(key);
  }

  async set(userId: string, stateHash: string, contentId: string, entry: QTableEntry): Promise<void> {
    const key = `q:${userId}:${stateHash}:${contentId}`;
    await this.db.set(key, entry, {
      metadata: { userId, stateHash, contentId },
      ttl: 90 * 24 * 60 * 60 // 90 days
    });
  }

  async getStateActions(userId: string, stateHash: string): Promise<QTableEntry[]> {
    return this.db.query({
      metadata: { userId, stateHash }
    });
  }
}
```

**Acceptance Criteria**:
- [ ] Q-values persist across server restarts
- [ ] User profiles stored with exploration rate
- [ ] Experience replay buffer persisted
- [ ] TTL of 90 days on Q-table entries
- [ ] Data survives `npm run start` restart

---

### 2.2 Update QTable to Use Persistence

**Task**: Modify QTable class to use AgentDB store

**File**: `src/rl/q-table.ts`

**Changes**:
```typescript
// Before: In-memory Map
private entries: Map<string, QTableEntry>;

// After: AgentDB-backed
constructor(private store: QTableStore) {}

async get(stateHash: string, contentId: string): Promise<QTableEntry | null> {
  return this.store.get(this.userId, stateHash, contentId);
}

async updateQValue(stateHash: string, contentId: string, qValue: number): Promise<void> {
  const existing = await this.get(stateHash, contentId);
  const entry: QTableEntry = {
    ...existing,
    qValue,
    visitCount: (existing?.visitCount || 0) + 1,
    lastUpdated: Date.now()
  };
  await this.store.set(this.userId, stateHash, contentId, entry);
}
```

---

## Phase 3: Gemini Integration (P1)

**Duration**: 2-3 hours
**Agents**: `backend-dev`, `coder`, `qe-api-contract-validator`

### 3.1 Install Gemini SDK

```bash
npm install @google/generative-ai
```

**Environment**:
```
GEMINI_API_KEY=your_key_here
```

### 3.2 Implement Real EmotionDetector

**File**: `src/emotion/gemini-client.ts`

```typescript
import { GoogleGenerativeAI, GenerativeModel } from '@google/generative-ai';
import { GeminiEmotionResponse } from './types';

const EMOTION_PROMPT = `Analyze the emotional state from this text: "{text}"

You are an expert emotion analyst. Extract the following emotional dimensions:

1. **Primary Emotion**: Choose ONE from [joy, sadness, anger, fear, trust, disgust, surprise, anticipation]

2. **Valence**: Emotional pleasantness
   - Range: -1.0 (very negative) to +1.0 (very positive)

3. **Arousal**: Emotional activation/energy level
   - Range: -1.0 (very calm/sleepy) to +1.0 (very excited/agitated)

4. **Stress Level**: Psychological stress
   - Range: 0.0 (completely relaxed) to 1.0 (extremely stressed)

5. **Confidence**: How certain are you about this analysis?
   - Range: 0.0 (very uncertain) to 1.0 (very certain)

Respond ONLY with valid JSON:
{
  "primaryEmotion": "...",
  "valence": 0.0,
  "arousal": 0.0,
  "stressLevel": 0.0,
  "confidence": 0.0,
  "reasoning": "Brief explanation (max 50 words)"
}`;

export class GeminiClient {
  private model: GenerativeModel;
  private readonly maxRetries = 3;
  private readonly timeout = 30000;

  constructor(apiKey: string) {
    const genAI = new GoogleGenerativeAI(apiKey);
    this.model = genAI.getGenerativeModel({
      model: 'gemini-2.0-flash-exp',
      generationConfig: {
        temperature: 0.3,
        topP: 0.8,
        maxOutputTokens: 256
      }
    });
  }

  async analyzeEmotion(text: string): Promise<GeminiEmotionResponse> {
    const prompt = EMOTION_PROMPT.replace('{text}', text);

    for (let attempt = 1; attempt <= this.maxRetries; attempt++) {
      try {
        const result = await Promise.race([
          this.model.generateContent(prompt),
          this.createTimeout()
        ]);

        const response = await result.response;
        const jsonText = response.text();
        return JSON.parse(jsonText);

      } catch (error) {
        if (attempt === this.maxRetries) throw error;
        await this.sleep(1000 * attempt); // Exponential backoff
      }
    }

    throw new Error('Gemini API failed after retries');
  }

  private createTimeout(): Promise<never> {
    return new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Timeout')), this.timeout)
    );
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
```

### 3.3 Update EmotionDetector to Use Gemini

**File**: `src/emotion/detector.ts`

```typescript
import { GeminiClient } from './gemini-client';

export class EmotionDetector {
  private geminiClient: GeminiClient | null;

  constructor() {
    const apiKey = process.env.GEMINI_API_KEY;
    this.geminiClient = apiKey ? new GeminiClient(apiKey) : null;
  }

  async analyzeText(text: string): Promise<EmotionResult> {
    // Use Gemini if available, fallback to mock
    const geminiResponse = this.geminiClient
      ? await this.geminiClient.analyzeEmotion(text)
      : this.mockGeminiAPI(text);

    // Rest of processing...
  }
}
```

**Acceptance Criteria**:
- [ ] Real Gemini API called when GEMINI_API_KEY set
- [ ] Fallback to mock when no API key
- [ ] 30s timeout implemented
- [ ] 3 retry attempts with backoff
- [ ] Rate limit handling (429 → wait and retry)

---

## Phase 4: Authentication (P1)

**Duration**: 4-6 hours
**Agents**: `backend-dev`, `qe-security-scanner`, `qe-test-generator`

### 4.1 Install Auth Dependencies

```bash
npm install jsonwebtoken bcryptjs
npm install -D @types/jsonwebtoken @types/bcryptjs
```

### 4.2 Create Auth Module

**New Files**:
```
src/auth/
├── jwt-service.ts           # JWT generation/validation
├── password-service.ts      # Password hashing
├── auth-middleware.ts       # Express middleware
└── routes.ts                # Auth endpoints
```

**Implementation** (`src/auth/routes.ts`):

```typescript
import { Router } from 'express';
import { JWTService } from './jwt-service';
import { PasswordService } from './password-service';
import { UserStore } from '../persistence/user-store';

const router = Router();
const jwtService = new JWTService(process.env.JWT_SECRET!);
const passwordService = new PasswordService();

// POST /api/v1/auth/register
router.post('/register', async (req, res, next) => {
  try {
    const { email, password, displayName, dateOfBirth } = req.body;

    // Validate
    if (!email || !password || !displayName) {
      return res.status(400).json({
        success: false,
        error: { code: 'E003', message: 'Missing required fields' }
      });
    }

    // Check existing
    const existing = await UserStore.findByEmail(email);
    if (existing) {
      return res.status(409).json({
        success: false,
        error: { code: 'E003', message: 'Email already registered' }
      });
    }

    // Create user
    const hashedPassword = await passwordService.hash(password);
    const user = await UserStore.create({
      email,
      password: hashedPassword,
      displayName,
      dateOfBirth
    });

    // Generate tokens
    const token = jwtService.generateAccessToken(user.id);
    const refreshToken = jwtService.generateRefreshToken(user.id);

    res.status(201).json({
      success: true,
      data: {
        userId: user.id,
        email: user.email,
        displayName: user.displayName,
        token,
        refreshToken,
        expiresAt: jwtService.getExpiry(token)
      },
      error: null,
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    next(error);
  }
});

// POST /api/v1/auth/login
router.post('/login', async (req, res, next) => {
  // ... similar implementation
});

// POST /api/v1/auth/refresh
router.post('/refresh', async (req, res, next) => {
  // ... implementation
});

export default router;
```

### 4.3 Add Auth Middleware

```typescript
// src/auth/auth-middleware.ts
import { Request, Response, NextFunction } from 'express';
import { JWTService } from './jwt-service';

const jwtService = new JWTService(process.env.JWT_SECRET!);

export function authMiddleware(req: Request, res: Response, next: NextFunction) {
  const authHeader = req.headers.authorization;

  if (!authHeader?.startsWith('Bearer ')) {
    return res.status(401).json({
      success: false,
      error: { code: 'E007', message: 'Invalid or missing token' }
    });
  }

  const token = authHeader.substring(7);

  try {
    const payload = jwtService.verify(token);
    req.userId = payload.userId;
    next();
  } catch (error) {
    return res.status(401).json({
      success: false,
      error: { code: 'E007', message: 'Token expired or invalid' }
    });
  }
}
```

**Acceptance Criteria**:
- [ ] POST /api/v1/auth/register creates user and returns JWT
- [ ] POST /api/v1/auth/login validates credentials and returns JWT
- [ ] POST /api/v1/auth/refresh exchanges refresh token
- [ ] Protected endpoints require valid JWT
- [ ] Passwords hashed with bcrypt (12 rounds)

---

## Phase 5: Missing Endpoints (P2)

**Duration**: 4-6 hours
**Agents**: `backend-dev`, `coder`, `qe-api-contract-validator`

### 5.1 Insights Endpoint

**File**: `src/api/routes/insights.ts`

```typescript
// GET /api/v1/insights/:userId
router.get('/:userId', authMiddleware, async (req, res, next) => {
  const { userId } = req.params;

  // Get user experiences
  const experiences = await ExperienceStore.getByUser(userId);
  const userStats = await UserStore.getStats(userId);

  // Calculate insights
  const avgReward = experiences.length > 0
    ? experiences.reduce((sum, e) => sum + e.reward, 0) / experiences.length
    : 0;

  const emotionalJourney = experiences.slice(-10).map(e => ({
    timestamp: new Date(e.timestamp).toISOString(),
    valence: e.stateAfter.valence,
    arousal: e.stateAfter.arousal,
    primaryEmotion: e.stateAfter.primaryEmotion
  }));

  const mostEffective = await ContentStore.getTopByReward(userId, 5);

  res.json({
    success: true,
    data: {
      userId,
      totalExperiences: experiences.length,
      avgReward,
      explorationRate: userStats.explorationRate,
      policyConvergence: calculateConvergence(experiences),
      emotionalJourney,
      mostEffectiveContent: mostEffective,
      learningProgress: {
        experiencesUntilConvergence: Math.max(0, 50 - experiences.length),
        currentQValueVariance: calculateQVariance(userId),
        isConverged: experiences.length >= 50
      }
    }
  });
});
```

### 5.2 Wellbeing Endpoint

**File**: `src/api/routes/wellbeing.ts`

```typescript
// GET /api/v1/wellbeing/:userId
router.get('/:userId', authMiddleware, async (req, res, next) => {
  const { userId } = req.params;

  // Get recent emotional states (last 7 days)
  const recentStates = await EmotionalStateStore.getRecent(userId, 7);

  // Calculate wellbeing metrics
  const recentMoodAvg = calculateAverageValence(recentStates);
  const overallTrend = calculateTrend(recentStates);
  const emotionalVariability = calculateVariability(recentStates);

  // Check for alerts
  const alerts = [];

  // Sustained negative mood (7+ days below -0.5)
  const sustainedNegativeDays = countConsecutiveNegativeDays(recentStates);
  if (sustainedNegativeDays >= 7) {
    alerts.push({
      type: 'sustained-negative-mood',
      severity: 'high',
      message: 'We noticed you\'ve been feeling down lately. Would you like some resources?',
      resources: [
        {
          type: 'crisis-line',
          name: '988 Suicide & Crisis Lifeline',
          url: 'tel:988',
          description: '24/7 free and confidential support'
        }
      ],
      triggeredAt: new Date().toISOString()
    });
  }

  res.json({
    success: true,
    data: {
      userId,
      overallTrend,
      recentMoodAvg,
      emotionalVariability,
      sustainedNegativeMoodDays: sustainedNegativeDays,
      alerts,
      recommendations: generateWellbeingRecommendations(recentMoodAvg, overallTrend)
    }
  });
});
```

### 5.3 Content Profiling Endpoint

**File**: `src/api/routes/content.ts`

```typescript
// POST /api/v1/content/profile
router.post('/profile', authMiddleware, async (req, res, next) => {
  const { contentId, title, description, genres, platform } = req.body;

  // Generate emotional profile using Gemini
  const profile = await contentProfiler.profileContent({
    title,
    description,
    genres
  });

  // Generate embedding
  const embedding = await embeddingGenerator.generate(
    `${title} ${description} ${genres.join(' ')}`
  );

  // Store in content database
  await ContentStore.save({
    id: contentId,
    title,
    description,
    platform,
    genres,
    emotionalProfile: profile,
    embeddingId: embedding.id
  });

  res.json({
    success: true,
    data: {
      contentId,
      emotionalProfile: profile,
      embeddingId: embedding.id,
      profiledAt: new Date().toISOString()
    }
  });
});
```

---

## Phase 6: Vector Search (P2)

**Duration**: 6-8 hours
**Agents**: `backend-dev`, `ml-developer`, `qe-performance-tester`

### 6.1 Install Vector Dependencies

```bash
npm install hnswlib-node
```

Or use AgentDB's built-in vector support:
```bash
npm install agentdb@latest  # Includes vector support
```

### 6.2 Implement Vector Store

**File**: `src/content/vector-store.ts`

```typescript
import HNSWLib from 'hnswlib-node';

export class ContentVectorStore {
  private index: HNSWLib.HierarchicalNSW;
  private readonly dimensions = 1536;
  private readonly maxElements = 10000;

  constructor() {
    this.index = new HNSWLib.HierarchicalNSW('cosine', this.dimensions);
    this.index.initIndex(this.maxElements, 16, 200); // M=16, efConstruction=200
  }

  async add(contentId: string, embedding: number[]): Promise<void> {
    const id = this.getNumericId(contentId);
    this.index.addPoint(embedding, id);
    // Also store mapping in AgentDB
    await ContentStore.saveEmbeddingMapping(contentId, id);
  }

  async search(queryEmbedding: number[], topK: number = 30): Promise<string[]> {
    this.index.setEf(100);
    const result = this.index.searchKnn(queryEmbedding, topK);

    // Convert numeric IDs back to content IDs
    const contentIds = await Promise.all(
      result.neighbors.map(id => ContentStore.getContentIdByEmbeddingId(id))
    );

    return contentIds;
  }

  async save(path: string): Promise<void> {
    this.index.writeIndex(path);
  }

  async load(path: string): Promise<void> {
    this.index.readIndex(path, this.maxElements);
  }
}
```

### 6.3 Integrate with Recommendations

```typescript
// src/recommendations/engine.ts
async getRecommendations(
  userId: string,
  currentState: EmotionalState,
  desiredState: DesiredState,
  limit: number
): Promise<Recommendation[]> {
  // Step 1: Get semantically similar content via vector search
  const transitionEmbedding = await this.embeddingGenerator.generate(
    `transition from ${currentState.primaryEmotion} to ${this.describeState(desiredState)}`
  );

  const candidates = await this.vectorStore.search(transitionEmbedding, 50);

  // Step 2: Rank by Q-values
  const ranked = await this.ranker.rank(userId, currentState, desiredState, candidates);

  // Step 3: Apply exploration/exploitation
  const selected = await this.applyExploration(ranked, limit);

  return selected;
}
```

---

## Phase 7: QE Verification & Testing

**Duration**: 6-8 hours
**Agents**: `qe-test-generator`, `qe-integration-tester`, `qe-coverage-analyzer`, `qe-performance-tester`

### 7.1 Integration Test Suite

**File**: `tests/integration/api-flow.test.ts`

```typescript
describe('EmotiStream E2E Flow', () => {
  let authToken: string;
  let userId: string;

  beforeAll(async () => {
    // Register user
    const res = await request(app)
      .post('/api/v1/auth/register')
      .send({
        email: 'test@example.com',
        password: 'TestPass123!',
        displayName: 'Test User'
      });

    authToken = res.body.data.token;
    userId = res.body.data.userId;
  });

  test('Complete emotion → recommend → feedback cycle', async () => {
    // Step 1: Analyze emotion
    const emotionRes = await request(app)
      .post('/api/v1/emotion/analyze')
      .set('Authorization', `Bearer ${authToken}`)
      .send({
        userId,
        text: 'I am feeling stressed and anxious about work'
      });

    expect(emotionRes.body.success).toBe(true);
    expect(emotionRes.body.data.currentState.valence).toBeLessThan(0);
    expect(emotionRes.body.data.currentState.stressLevel).toBeGreaterThan(0.5);

    // Step 2: Get recommendations
    const recommendRes = await request(app)
      .post('/api/v1/recommend')
      .set('Authorization', `Bearer ${authToken}`)
      .send({
        userId,
        currentState: emotionRes.body.data.currentState,
        desiredState: emotionRes.body.data.desiredState,
        limit: 3
      });

    expect(recommendRes.body.success).toBe(true);
    expect(recommendRes.body.data.recommendations).toHaveLength(3);
    expect(recommendRes.body.data.recommendations[0].qValue).toBeDefined();

    const selectedContent = recommendRes.body.data.recommendations[0];

    // Step 3: Submit feedback
    const feedbackRes = await request(app)
      .post('/api/v1/feedback')
      .set('Authorization', `Bearer ${authToken}`)
      .send({
        userId,
        contentId: selectedContent.contentId,
        actualPostState: {
          valence: 0.3,
          arousal: -0.2,
          stressLevel: 0.3,
          primaryEmotion: 'trust'
        },
        watchDuration: 1800,
        completed: true,
        explicitRating: 4
      });

    expect(feedbackRes.body.success).toBe(true);
    expect(feedbackRes.body.data.reward).toBeGreaterThan(0);
    expect(feedbackRes.body.data.policyUpdated).toBe(true);

    // Step 4: Verify Q-value was updated
    const secondRecommendRes = await request(app)
      .post('/api/v1/recommend')
      .set('Authorization', `Bearer ${authToken}`)
      .send({
        userId,
        currentState: emotionRes.body.data.currentState,
        desiredState: emotionRes.body.data.desiredState,
        limit: 3
      });

    // Same content should now have higher Q-value (learning happened)
    const sameContent = secondRecommendRes.body.data.recommendations
      .find(r => r.contentId === selectedContent.contentId);

    expect(sameContent.qValue).toBeGreaterThan(selectedContent.qValue);
  });

  test('Q-values persist across server restart', async () => {
    // Get current Q-value
    const before = await request(app)
      .post('/api/v1/recommend')
      .set('Authorization', `Bearer ${authToken}`)
      .send({ ... });

    // Simulate restart by reinitializing services
    await services.reinitialize();

    // Q-value should be same
    const after = await request(app)
      .post('/api/v1/recommend')
      .set('Authorization', `Bearer ${authToken}`)
      .send({ ... });

    expect(after.body.data.recommendations[0].qValue)
      .toBe(before.body.data.recommendations[0].qValue);
  });
});
```

### 7.2 Performance Benchmarks

**File**: `tests/performance/benchmarks.test.ts`

```typescript
describe('Performance Benchmarks', () => {
  test('Emotion analysis < 2s (p95)', async () => {
    const times: number[] = [];

    for (let i = 0; i < 100; i++) {
      const start = Date.now();
      await request(app)
        .post('/api/v1/emotion/analyze')
        .set('Authorization', `Bearer ${authToken}`)
        .send({ userId, text: 'I feel happy today' });
      times.push(Date.now() - start);
    }

    times.sort((a, b) => a - b);
    const p95 = times[Math.floor(times.length * 0.95)];

    expect(p95).toBeLessThan(2000);
  });

  test('Recommendations < 3s (p95)', async () => {
    // Similar benchmark
  });

  test('Feedback < 100ms (p95)', async () => {
    // Similar benchmark
  });
});
```

---

## Phase 8: Optimization & Polish

**Duration**: 4-6 hours
**Agents**: `perf-analyzer`, `code-analyzer`, `qe-code-complexity`

### 8.1 Caching Layer

```typescript
// src/cache/redis-cache.ts or in-memory
class RecommendationCache {
  private cache: Map<string, { data: Recommendation[], expires: number }>;
  private readonly ttl = 60 * 1000; // 1 minute

  get(userId: string, stateHash: string): Recommendation[] | null {
    const key = `${userId}:${stateHash}`;
    const entry = this.cache.get(key);

    if (entry && entry.expires > Date.now()) {
      return entry.data;
    }

    return null;
  }

  set(userId: string, stateHash: string, recommendations: Recommendation[]): void {
    const key = `${userId}:${stateHash}`;
    this.cache.set(key, {
      data: recommendations,
      expires: Date.now() + this.ttl
    });
  }
}
```

### 8.2 Error Response Standardization

```typescript
// src/utils/errors.ts
const ERROR_CODES = {
  E001: { status: 504, message: 'Gemini API timeout' },
  E002: { status: 429, message: 'Gemini rate limit exceeded' },
  E003: { status: 400, message: 'Invalid input' },
  E004: { status: 404, message: 'User not found' },
  E005: { status: 404, message: 'Content not found' },
  E006: { status: 500, message: 'RL policy error' },
  E007: { status: 401, message: 'Invalid or expired token' },
  E008: { status: 403, message: 'Unauthorized' },
  E009: { status: 429, message: 'Rate limit exceeded' },
  E010: { status: 500, message: 'Internal error' }
};

export class AppError extends Error {
  constructor(
    public readonly code: keyof typeof ERROR_CODES,
    public readonly details?: Record<string, any>
  ) {
    super(ERROR_CODES[code].message);
  }
}
```

---

## Swarm Configuration

### Claude-Flow Swarm Setup

```yaml
# .claude/swarm-config.yaml
topology: hierarchical
maxAgents: 12

phases:
  - name: integration
    agents:
      - type: backend-dev
        task: "Wire API endpoints to modules"
        memory: aqe/integration/*
      - type: coder
        task: "Create service layer"
        memory: aqe/services/*
      - type: qe-integration-tester
        task: "Write integration tests"
        memory: aqe/tests/integration/*

  - name: persistence
    agents:
      - type: backend-dev
        task: "Implement AgentDB persistence"
        memory: aqe/persistence/*
      - type: qe-test-generator
        task: "Generate persistence tests"
        memory: aqe/tests/persistence/*

  - name: gemini
    agents:
      - type: backend-dev
        task: "Integrate Gemini API"
        memory: aqe/gemini/*
      - type: qe-api-contract-validator
        task: "Validate Gemini responses"
        memory: aqe/contracts/*

  - name: auth
    agents:
      - type: backend-dev
        task: "Implement JWT authentication"
        memory: aqe/auth/*
      - type: qe-security-scanner
        task: "Security audit auth system"
        memory: aqe/security/*

  - name: endpoints
    agents:
      - type: backend-dev
        task: "Implement missing endpoints"
        memory: aqe/endpoints/*
      - type: coder
        task: "Add wellbeing and insights"
        memory: aqe/features/*

  - name: vectors
    agents:
      - type: ml-developer
        task: "Implement vector search"
        memory: aqe/vectors/*
      - type: qe-performance-tester
        task: "Benchmark vector search"
        memory: aqe/performance/*

  - name: verification
    agents:
      - type: qe-test-executor
        task: "Run full test suite"
        memory: aqe/results/*
      - type: qe-coverage-analyzer
        task: "Analyze coverage gaps"
        memory: aqe/coverage/*
      - type: qe-performance-tester
        task: "Run performance benchmarks"
        memory: aqe/benchmarks/*

  - name: optimization
    agents:
      - type: perf-analyzer
        task: "Identify bottlenecks"
        memory: aqe/optimization/*
      - type: code-analyzer
        task: "Code quality review"
        memory: aqe/quality/*
```

### Slash Command

```bash
# Run the full implementation swarm
/parallel_subagents "Complete EmotiStream MVP per IMPLEMENTATION-PLAN-ALPHA.md" 12
```

---

## Success Criteria for Alpha Release

### Functional Requirements

| Requirement | Metric | Target |
|-------------|--------|--------|
| API Endpoints | All spec endpoints working | 100% |
| Authentication | JWT auth flow | Complete |
| Gemini Integration | Real emotion detection | With fallback |
| Persistence | Q-values survive restart | Verified |
| Learning | Q-values update on feedback | Demonstrated |

### Performance Requirements

| Metric | Target |
|--------|--------|
| Emotion Analysis (p95) | < 2s |
| Recommendations (p95) | < 3s |
| Feedback (p95) | < 100ms |
| Vector Search (p95) | < 500ms |

### Quality Requirements

| Metric | Target |
|--------|--------|
| Test Coverage | > 80% |
| Integration Tests | All passing |
| Security Scan | No critical issues |
| API Spec Compliance | 100% |

---

## Timeline

| Phase | Duration | Agents | Dependencies |
|-------|----------|--------|--------------|
| 1. Integration | 4-6h | 3 | None |
| 2. Persistence | 4-6h | 2 | Phase 1 |
| 3. Gemini | 2-3h | 2 | Phase 1 |
| 4. Auth | 4-6h | 3 | Phase 2 |
| 5. Endpoints | 4-6h | 2 | Phase 2, 3 |
| 6. Vectors | 6-8h | 2 | Phase 3 |
| 7. Verification | 6-8h | 4 | Phase 1-6 |
| 8. Optimization | 4-6h | 2 | Phase 7 |

**Total Estimated**: 34-49 hours with parallel execution

**Parallel Execution**: Phases 3, 4, 5 can run in parallel after Phase 2.

**Critical Path**: 1 → 2 → 4 → 7 → 8 (~22-32 hours)

---

## Post-Alpha Checklist

- [ ] All endpoints return real data (no mocks)
- [ ] Q-learning actually learns from feedback
- [ ] Data persists across restarts
- [ ] Authentication protects all endpoints
- [ ] Gemini integration works with fallback
- [ ] Performance meets SLAs
- [ ] Test coverage > 80%
- [ ] Security scan passed
- [ ] User documentation updated
- [ ] Deployment script ready

---

**Document Status**: Ready for Execution
**Next Action**: Initialize claude-flow swarm with Phase 1 agents
