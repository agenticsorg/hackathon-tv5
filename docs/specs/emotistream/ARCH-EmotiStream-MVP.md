# EmotiStream Nexus MVP Architecture
**Hackathon Build: 70-Hour Implementation**

**Version**: 1.0
**Last Updated**: 2025-12-05
**Build Target**: Hackathon MVP Demo
**Architecture Status**: ✅ Optimized for Rapid Development

---

## 1. System Architecture Overview

### 1.1 High-Level Architecture (ASCII)

```
┌─────────────────────────────────────────────────────────────────────┐
│                    EmotiStream Nexus MVP                            │
│                   (Single Node.js Server)                           │
└─────────────────────────────────────────────────────────────────────┘

┌──────────────┐
│  CLI Client  │────────────┐
│  (Demo UI)   │            │
└──────────────┘            │
                            ▼
                    ┌───────────────┐
                    │  REST API     │
                    │  (Express)    │
                    └───────────────┘
                            │
            ┌───────────────┼───────────────┐
            ▼               ▼               ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │   Emotion    │ │  RL Policy   │ │ Recommend.   │
    │   Detector   │ │   Engine     │ │   Engine     │
    │              │ │              │ │              │
    │ • Gemini API │ │ • Q-Learning │ │ • RuVector   │
    │ • Text only  │ │ • AgentDB    │ │ • Semantic   │
    │ • 8 emotions │ │ • Reward fn  │ │ • Fusion     │
    └──────────────┘ └──────────────┘ └──────────────┘
            │               │               │
            └───────────────┼───────────────┘
                            ▼
                    ┌───────────────┐
                    │   Storage     │
                    │               │
                    │ • AgentDB     │
                    │ • RuVector    │
                    │ • In-memory   │
                    └───────────────┘
                            │
            ┌───────────────┼───────────────┐
            ▼               ▼               ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │  User State  │ │  Q-Tables    │ │  Content     │
    │  Profiles    │ │  Experience  │ │  Embeddings  │
    │              │ │  Replay      │ │              │
    └──────────────┘ └──────────────┘ └──────────────┘

External Services:
┌──────────────┐ ┌──────────────┐
│ Gemini API   │ │ Mock Content │
│ (Emotion)    │ │ Catalog      │
└──────────────┘ └──────────────┘
```

### 1.2 Data Flow Diagrams

#### Emotion Detection Flow
```
User Input (Text)
    │
    ▼
Express API Endpoint (/api/emotion/detect)
    │
    ▼
EmotionDetector.analyzeText()
    │
    ├──▶ Gemini API Request
    │       │
    │       ▼
    │   Emotion Analysis (JSON)
    │
    ▼
Map to EmotionalState (valence, arousal, emotion vector)
    │
    ▼
Store in AgentDB (user:${userId}:emotional-history)
    │
    ▼
Return EmotionalState + emotionalStateId
```

#### Recommendation Flow
```
Recommendation Request
    │
    ├──▶ emotionalStateId
    └──▶ userId
         │
         ▼
    Load EmotionalState from AgentDB
         │
         ▼
    Predict Desired State
         │
         ├──▶ Check historical patterns (AgentDB)
         └──▶ Apply heuristics (stressed → calm)
              │
              ▼
    RLPolicyEngine.selectAction()
         │
         ├──▶ ε-greedy (15% exploration)
         │
         ├──▶ Exploit: RuVector semantic search
         │       │
         │       ├──▶ Create transition embedding
         │       ├──▶ Search content by emotion
         │       └──▶ Re-rank with Q-values (AgentDB)
         │
         └──▶ Explore: UCB exploration
              │
              ▼
    Top 20 Recommendations
         │
         ▼
    Return with emotional predictions
```

#### Feedback & Learning Flow
```
Post-Viewing Feedback
    │
    ├──▶ experienceId
    ├──▶ Post-state emotional input
    └──▶ Explicit feedback (optional)
         │
         ▼
    Detect Post-Viewing Emotion (Gemini)
         │
         ▼
    Calculate Reward
         │
         ├──▶ Direction alignment (cosine similarity)
         ├──▶ Magnitude of improvement
         └──▶ Proximity bonus
              │
              ▼
    Update Q-Value (AgentDB)
         │
         ├──▶ Q(s,a) = Q(s,a) + α[r + γ·maxQ(s',a') - Q(s,a)]
         │
         ▼
    Add to Experience Replay Buffer (AgentDB)
         │
         ▼
    Update User Profile (AgentDB)
         │
         └──▶ Increment experience count
              Update avg reward
              Adjust exploration rate
         │
         ▼
    Return reward + Q-value update confirmation
```

---

## 2. Component Breakdown

### 2.1 Emotion Detector

**Component**: EmotionDetector
**Responsibility**: Analyze text input to extract emotional state using Gemini API
**Technology**: TypeScript, Gemini 2.0 Flash Exp API
**Interfaces**:
- Input: `{ text: string }`
- Output: `EmotionalState` (valence, arousal, 8D emotion vector)

**Dependencies**:
- Gemini API (`@google/generative-ai`)
- AgentDB (store emotional history)

**Estimated LOC**: 250
**Build Time**: 4 hours

**Key Methods**:
```typescript
class EmotionDetector {
  async analyzeText(text: string): Promise<EmotionalState>
  private mapToEmotionalState(analysis: GeminiEmotionResult): EmotionalState
  private emotionToVector(emotion: string): Float32Array
}
```

**Simplifications for MVP**:
- Text-only (no voice or biometric)
- Single Gemini API call (no multimodal fusion)
- Synchronous processing (no queue)
- 30s timeout with neutral fallback

---

### 2.2 RL Policy Engine

**Component**: RLPolicyEngine
**Responsibility**: Learn which content improves emotional states using Q-Learning
**Technology**: TypeScript, AgentDB for Q-tables
**Interfaces**:
- Input: `{ userId, emotionalState, desiredState }`
- Output: `EmotionalContentAction` (contentId, predicted outcome, Q-value)

**Dependencies**:
- AgentDB (Q-tables, user profiles)
- RuVector (content search)

**Estimated LOC**: 400
**Build Time**: 12 hours

**Key Methods**:
```typescript
class RLPolicyEngine {
  async selectAction(userId, emotionalState): Promise<EmotionalContentAction>
  private async exploit(): Promise<EmotionalContentAction> // Best Q-value
  private async explore(): Promise<EmotionalContentAction> // UCB exploration
  async updatePolicy(experience): Promise<void> // Q-learning update
  private calculateReward(before, after, desired): number
}
```

**RL Hyperparameters (MVP)**:
- Learning rate (α): 0.1
- Discount factor (γ): 0.95
- Exploration rate (ε): 0.15
- State discretization: 5×5×3 buckets (valence × arousal × stress)

**Simplifications for MVP**:
- Q-Learning only (no policy gradient or actor-critic)
- Synchronous updates (no batch training)
- Single-user optimization (no transfer learning)

---

### 2.3 Content Profiler

**Component**: ContentEmotionalProfiler
**Responsibility**: Generate emotional profiles for content catalog
**Technology**: TypeScript, Gemini API, RuVector embeddings
**Interfaces**:
- Input: `ContentMetadata` (title, description, genres)
- Output: `EmotionalContentProfile` (primaryTone, valenceDelta, arousalDelta)

**Dependencies**:
- Gemini API (emotional analysis)
- RuVector (store embeddings)

**Estimated LOC**: 300
**Build Time**: 8 hours

**Key Methods**:
```typescript
class ContentEmotionalProfiler {
  async profileContent(content: ContentMetadata): Promise<EmotionalContentProfile>
  private async createEmotionEmbedding(analysis): Promise<Float32Array>
  async batchProfile(contents: ContentMetadata[]): Promise<void> // For catalog init
}
```

**Simplifications for MVP**:
- Batch profile mock catalog during setup (500 items)
- Pre-generated embeddings (no runtime profiling)
- Manual validation only for demo items

---

### 2.4 Recommendation Engine

**Component**: RecommendationEngine
**Responsibility**: Fuse RL policy with semantic search for content recommendations
**Technology**: TypeScript, RuVector semantic search
**Interfaces**:
- Input: `{ userId, emotionalState, desiredState }`
- Output: `EmotionalRecommendation[]` (top 20)

**Dependencies**:
- RLPolicyEngine (Q-values)
- RuVector (semantic search)
- AgentDB (user history)

**Estimated LOC**: 200
**Build Time**: 6 hours

**Key Methods**:
```typescript
class RecommendationEngine {
  async recommend(userId, emotionalState, desiredState): Promise<EmotionalRecommendation[]>
  private createDesiredStateVector(current, desired): Float32Array
  private async searchByEmotionalTransition(): Promise<Content[]>
}
```

**Simplifications for MVP**:
- Top-20 only (no pagination)
- Single re-ranking pass (Q-value × semantic similarity)
- No diversity filtering

---

### 2.5 User Session Manager

**Component**: UserSessionManager
**Responsibility**: Manage user profiles, emotional history, and experiences
**Technology**: TypeScript, AgentDB
**Interfaces**:
- Input: `{ userId }`
- Output: `UserProfile`, `EmotionalHistory`, `Experience[]`

**Dependencies**:
- AgentDB (primary storage)

**Estimated LOC**: 150
**Build Time**: 4 hours

**Key Methods**:
```typescript
class UserSessionManager {
  async getOrCreateUser(userId: string): Promise<UserProfile>
  async getEmotionalHistory(userId, days): Promise<EmotionalState[]>
  async addExperience(experience: EmotionalExperience): Promise<void>
}
```

**Simplifications for MVP**:
- Single user (no auth)
- In-memory session (no JWT)
- Manual user ID input

---

### 2.6 API Layer

**Component**: REST API (Express)
**Responsibility**: HTTP endpoints for emotion detection, recommendations, feedback
**Technology**: Express.js, TypeScript
**Interfaces**: See Section 5 (API Contract)

**Dependencies**: All components above

**Estimated LOC**: 300
**Build Time**: 8 hours

**Simplifications for MVP**:
- REST only (no GraphQL)
- Synchronous responses (no streaming)
- No authentication
- No rate limiting

---

### 2.7 Demo UI

**Component**: CLI Demo
**Responsibility**: Interactive demo for hackathon presentation
**Technology**: Node.js CLI (inquirer.js)
**Interfaces**:
- Interactive prompts for emotional input
- Display recommendations with emotional predictions
- Post-viewing feedback collection

**Estimated LOC**: 200
**Build Time**: 6 hours

**Simplifications for MVP**:
- CLI only (no web UI)
- Text-based visualizations (no charts)
- Manual flow (no automation)

---

## 3. Technology Stack

### 3.1 Runtime & Framework

| Component | Technology | Version | Rationale |
|-----------|-----------|---------|-----------|
| Runtime | Node.js | 20+ | Native ESM, latest LTS |
| Language | TypeScript | 5.x | Type safety, productivity |
| Module System | ESM | Native | Modern Node.js standard |
| API Framework | Express | 4.x | Simple, fast REST API |
| CLI Framework | Inquirer.js | 9.x | Interactive CLI prompts |

### 3.2 AI/ML Stack

| Component | Technology | Purpose | API Key Required |
|-----------|-----------|---------|------------------|
| Emotion Analysis | Gemini 2.0 Flash Exp | Emotion detection from text | ✅ `GEMINI_API_KEY` |
| Content Profiling | Gemini 2.0 Flash Exp | Generate emotional profiles | ✅ Same key |
| Semantic Search | RuVector | Content-emotion matching | ❌ Local vector DB |
| Embeddings | RuVector (ruvLLM) | 1536D emotion embeddings | ❌ Local embedding model |

### 3.3 Storage Stack

| Component | Technology | Purpose | Persistence |
|-----------|-----------|---------|-------------|
| Primary DB | AgentDB | User profiles, Q-tables, experiences | ✅ SQLite file |
| Vector DB | RuVector | Content emotion embeddings | ✅ HNSW index file |
| Session Cache | In-memory Map | Current user session state | ❌ Runtime only |

**AgentDB Configuration (MVP)**:
```typescript
const agentDB = new AgentDB({
  dbPath: './data/emotistream.db',
  enableQuantization: false, // Not needed for MVP scale
  cacheSize: 1000
});
```

**RuVector Configuration (MVP)**:
```typescript
const ruVector = new RuVector({
  dimension: 1536,
  indexType: 'hnsw',
  hnswParams: { M: 16, efConstruction: 200, efSearch: 50 },
  persistPath: './data/content-embeddings.idx'
});
```

### 3.4 Dependencies (package.json)

```json
{
  "name": "emotistream-mvp",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "tsx watch src/server.ts",
    "build": "tsc",
    "start": "node dist/server.js",
    "cli": "tsx src/demo/cli.ts",
    "setup": "tsx scripts/setup-catalog.ts"
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
    "ora": "^8.0.1"
  },
  "devDependencies": {
    "@types/express": "^4.17.21",
    "@types/node": "^20.11.5",
    "typescript": "^5.3.3",
    "tsx": "^4.7.0"
  }
}
```

---

## 4. Data Models (MVP Simplified)

### 4.1 Core Types

```typescript
// Emotional State (RL State)
interface EmotionalState {
  emotionalStateId: string;       // UUID
  userId: string;

  // Russell's Circumplex
  valence: number;                // -1 to +1
  arousal: number;                // -1 to +1

  // Plutchik's 8 emotions (one-hot encoded)
  emotionVector: Float32Array;    // [joy, sadness, anger, fear, trust, disgust, surprise, anticipation]
  primaryEmotion: string;         // Dominant emotion

  // Context
  timestamp: number;
  stressLevel: number;            // 0-1 (derived from valence/arousal)

  // Desired outcome (predicted or explicit)
  desiredValence: number;
  desiredArousal: number;
  desiredStateConfidence: number; // How confident is the prediction?
}

// Content Metadata
interface ContentMetadata {
  contentId: string;
  title: string;
  description: string;
  platform: 'youtube' | 'netflix' | 'mock';
  genres: string[];
  duration: number; // seconds
}

// Emotional Content Profile (Learned)
interface EmotionalContentProfile {
  contentId: string;

  // Emotional characteristics (from Gemini)
  primaryTone: string;            // 'uplifting', 'melancholic', 'thrilling'
  valenceDelta: number;           // Expected change in valence
  arousalDelta: number;           // Expected change in arousal
  intensity: number;              // 0-1 (how intense is the emotion?)
  complexity: number;             // 0-1 (simple vs nuanced emotions)

  // Embedding
  embeddingId: string;            // RuVector ID

  // Learned effectiveness (updated with each experience)
  avgEmotionalImprovement: number;
  sampleSize: number;
}

// Emotional Experience (RL Experience for Replay)
interface EmotionalExperience {
  experienceId: string;
  userId: string;

  // State-action-reward-next_state (SARS)
  stateBefore: EmotionalState;
  contentId: string;
  stateAfter: EmotionalState;
  desiredState: { valence: number; arousal: number };

  // Reward
  reward: number;

  // Optional explicit feedback
  explicitRating?: number;        // 1-5

  timestamp: number;
}

// User Profile
interface UserProfile {
  userId: string;

  // Learning metadata
  totalExperiences: number;
  avgReward: number;
  explorationRate: number;

  // Baselines
  emotionalBaseline: {
    avgValence: number;
    avgArousal: number;
  };

  createdAt: number;
  lastActive: number;
}

// Recommendation Result
interface EmotionalRecommendation {
  contentId: string;
  title: string;
  platform: string;

  // Emotional prediction
  emotionalProfile: EmotionalContentProfile;
  predictedOutcome: {
    postViewingValence: number;
    postViewingArousal: number;
    expectedImprovement: number;
  };

  // RL metadata
  qValue: number;
  confidence: number;
  explorationFlag: boolean;      // Was this from exploration?

  // Explanation
  reasoning: string;
}
```

---

## 5. API Contract (Simplified)

### 5.1 REST Endpoints

#### POST /api/emotion/detect
**Detect emotional state from text input**

Request:
```typescript
{
  "userId": "user-001",
  "text": "I'm feeling exhausted and stressed after a long day"
}
```

Response:
```typescript
{
  "emotionalStateId": "state-abc123",
  "valence": -0.6,
  "arousal": 0.4,
  "primaryEmotion": "sadness",
  "emotionVector": [0, 1, 0, 0, 0, 0, 0, 0], // One-hot: sadness
  "stressLevel": 0.7,
  "desiredValence": 0.5,
  "desiredArousal": -0.3,
  "desiredStateConfidence": 0.65,
  "timestamp": 1701234567890
}
```

---

#### POST /api/recommend
**Get content recommendations based on emotional state**

Request:
```typescript
{
  "userId": "user-001",
  "emotionalStateId": "state-abc123",
  "limit": 20
}
```

Response:
```typescript
{
  "recommendations": [
    {
      "contentId": "content-001",
      "title": "Nature Sounds: Ocean Waves",
      "platform": "youtube",
      "emotionalProfile": {
        "primaryTone": "calm",
        "valenceDelta": 0.6,
        "arousalDelta": -0.7,
        "intensity": 0.3,
        "complexity": 0.2
      },
      "predictedOutcome": {
        "postViewingValence": 0.5,
        "postViewingArousal": -0.3,
        "expectedImprovement": 0.85
      },
      "qValue": 0.72,
      "confidence": 0.88,
      "explorationFlag": false,
      "reasoning": "Based on your stressed state, this calming content has a 88% chance of improving your mood to calm and positive."
    }
    // ... 19 more
  ],
  "learningMetrics": {
    "totalExperiences": 42,
    "avgReward": 0.64,
    "explorationRate": 0.15,
    "policyConvergence": 0.82
  }
}
```

---

#### POST /api/feedback
**Submit post-viewing emotional state and feedback**

Request:
```typescript
{
  "userId": "user-001",
  "experienceId": "exp-xyz789",
  "postViewingText": "I feel much calmer now",
  "explicitRating": 5
}
```

Response:
```typescript
{
  "success": true,
  "reward": 0.87,
  "qValueUpdated": true,
  "emotionalImprovement": 1.1,
  "postViewingState": {
    "valence": 0.6,
    "arousal": -0.2,
    "primaryEmotion": "joy"
  }
}
```

---

#### GET /api/insights/:userId
**Get user's emotional journey and insights**

Response:
```typescript
{
  "emotionalJourney": [
    {
      "date": "2024-12-01",
      "avgValence": -0.3,
      "avgArousal": 0.5,
      "topContent": ["content-001", "content-042"]
    }
    // ... 7 days
  ],
  "mostEffectiveContent": [
    {
      "contentId": "content-001",
      "title": "Nature Sounds: Ocean Waves",
      "avgEmotionalImprovement": 0.92,
      "timesWatched": 5,
      "emotionTransition": "stressed → calm"
    }
  ],
  "wellbeingScore": 0.45,
  "avgMoodImprovement": 0.68
}
```

---

## 6. Directory Structure

```
emotistream-mvp/
├── src/
│   ├── emotion/
│   │   ├── detector.ts              # Gemini emotion detection
│   │   ├── types.ts                 # EmotionalState interfaces
│   │   └── utils.ts                 # Emotion vector utilities
│   │
│   ├── rl/
│   │   ├── policy-engine.ts         # Q-learning policy
│   │   ├── reward.ts                # Reward function
│   │   ├── desired-state.ts         # Desired state predictor
│   │   └── types.ts                 # RL interfaces
│   │
│   ├── content/
│   │   ├── profiler.ts              # Content emotional profiling
│   │   ├── catalog.ts               # Mock content catalog
│   │   └── types.ts                 # Content interfaces
│   │
│   ├── recommend/
│   │   ├── engine.ts                # Recommendation fusion
│   │   └── types.ts                 # Recommendation interfaces
│   │
│   ├── storage/
│   │   ├── agentdb-client.ts        # AgentDB wrapper
│   │   ├── ruvector-client.ts       # RuVector wrapper
│   │   └── user-session.ts          # User session manager
│   │
│   ├── api/
│   │   ├── routes.ts                # Express routes
│   │   ├── middleware.ts            # Error handling, validation
│   │   └── types.ts                 # API request/response types
│   │
│   ├── demo/
│   │   └── cli.ts                   # CLI demo interface
│   │
│   ├── config/
│   │   └── env.ts                   # Environment configuration
│   │
│   └── server.ts                    # Express server entry point
│
├── scripts/
│   ├── setup-catalog.ts             # Initialize content catalog
│   └── seed-demo-data.ts            # Seed demo user data
│
├── data/
│   ├── content-catalog.json         # Mock content (500 items)
│   ├── emotistream.db               # AgentDB SQLite (created at runtime)
│   └── content-embeddings.idx       # RuVector index (created at runtime)
│
├── tests/
│   ├── emotion.test.ts              # Emotion detection tests
│   ├── rl.test.ts                   # RL policy tests
│   └── api.test.ts                  # API integration tests
│
├── docs/
│   ├── API.md                       # API documentation
│   └── DEMO.md                      # Demo script for presentation
│
├── .env.example                     # Environment variables template
├── package.json
├── tsconfig.json
└── README.md
```

---

## 7. Build Order (Critical Path)

### Hour 0-4: Project Setup ⏱️
- [ ] Initialize Node.js project with TypeScript + ESM
- [ ] Install dependencies (Express, Gemini SDK, AgentDB, RuVector)
- [ ] Configure `.env` with Gemini API key
- [ ] Create directory structure
- [ ] Set up AgentDB and RuVector clients
- [ ] **Deliverable**: Server starts, DB connections verified

### Hour 4-12: Emotion Detection + Content Profiling ⏱️
- [ ] Implement `EmotionDetector.analyzeText()`
- [ ] Test Gemini API emotion analysis
- [ ] Create mock content catalog (500 items)
- [ ] Implement `ContentEmotionalProfiler.profileContent()`
- [ ] Batch profile mock catalog → RuVector
- [ ] **Deliverable**: Emotion detection API works, content catalog profiled

### Hour 12-28: RL Engine + Recommendation Fusion ⏱️
- [ ] Implement Q-learning policy engine
- [ ] State discretization (valence × arousal buckets)
- [ ] Q-value storage in AgentDB
- [ ] Desired state predictor (heuristics + patterns)
- [ ] ε-greedy exploration
- [ ] Reward function implementation
- [ ] RuVector semantic search by emotional transition
- [ ] Fusion: Q-values × semantic similarity
- [ ] **Deliverable**: Recommendations API works with RL

### Hour 28-40: API Layer + Integration ⏱️
- [ ] Express API routes (`/detect`, `/recommend`, `/feedback`, `/insights`)
- [ ] Request validation (Zod schemas)
- [ ] Error handling middleware
- [ ] Experience tracking in AgentDB
- [ ] Q-value updates on feedback
- [ ] User profile management
- [ ] **Deliverable**: Full API functional, end-to-end RL loop

### Hour 40-55: Demo UI + Polish ⏱️
- [ ] CLI demo with Inquirer.js
- [ ] Interactive emotional input
- [ ] Display recommendations with explanations
- [ ] Post-viewing feedback flow
- [ ] Emotional journey visualization (text-based)
- [ ] Demo script for presentation
- [ ] **Deliverable**: Polished CLI demo ready

### Hour 55-70: Testing, Bug Fixes, Demo Prep ⏱️
- [ ] Unit tests for emotion detection
- [ ] Unit tests for reward function
- [ ] Integration tests for API
- [ ] Manual QA (happy path + edge cases)
- [ ] Performance testing (API latency)
- [ ] Bug fixes from testing
- [ ] Demo rehearsal
- [ ] **Deliverable**: MVP ready for presentation

---

## 8. Integration Points

### 8.1 Gemini API → EmotionDetector

```typescript
// src/emotion/detector.ts
import { GoogleGenerativeAI } from '@google/generative-ai';

const genAI = new GoogleGenerativeAI(process.env.GEMINI_API_KEY);
const model = genAI.getGenerativeModel({ model: 'gemini-2.0-flash-exp' });

class EmotionDetector {
  async analyzeText(text: string): Promise<EmotionalState> {
    const prompt = `Analyze emotional state from: "${text}". Return JSON: {"primaryEmotion": "...", "valence": 0.0, "arousal": 0.0, "stressLevel": 0.0}`;

    const result = await model.generateContent(prompt);
    const analysis = JSON.parse(result.response.text());

    return this.mapToEmotionalState(analysis);
  }
}
```

**Error Handling**:
- 30s timeout → fallback neutral state (valence: 0, arousal: 0)
- JSON parse error → retry once, then fallback
- Rate limit → queue request (max 3 retries)

---

### 8.2 EmotionDetector → RLPolicyEngine

```typescript
// src/rl/policy-engine.ts
class RLPolicyEngine {
  async selectAction(
    userId: string,
    emotionalState: EmotionalState
  ): Promise<EmotionalContentAction> {
    // Predict desired state
    const desiredState = await this.predictDesiredState(userId, emotionalState);

    // Store desired state back to emotionalState
    emotionalState.desiredValence = desiredState.valence;
    emotionalState.desiredArousal = desiredState.arousal;

    // ε-greedy
    if (Math.random() < this.explorationRate) {
      return await this.explore(userId, emotionalState, desiredState);
    }
    return await this.exploit(userId, emotionalState, desiredState);
  }
}
```

---

### 8.3 RLPolicyEngine → RecommendationEngine → RuVector

```typescript
// src/recommend/engine.ts
class RecommendationEngine {
  async recommend(
    userId: string,
    emotionalState: EmotionalState,
    limit: number = 20
  ): Promise<EmotionalRecommendation[]> {
    // Create transition embedding
    const transitionVector = this.createTransitionVector(
      emotionalState,
      { valence: emotionalState.desiredValence, arousal: emotionalState.desiredArousal }
    );

    // Semantic search in RuVector
    const candidates = await this.ruVector.search({
      vector: transitionVector,
      topK: 50
    });

    // Re-rank with Q-values
    const ranked = await Promise.all(
      candidates.map(async (c) => {
        const qValue = await this.rlPolicy.getQValue(userId, emotionalState, c.id);
        return {
          ...c,
          qValue,
          score: qValue * 0.7 + c.similarity * 0.3
        };
      })
    );

    return ranked.sort((a, b) => b.score - a.score).slice(0, limit);
  }
}
```

---

### 8.4 All → AgentDB

**Storage Keys**:
```typescript
// User profiles
`user:${userId}:profile` → UserProfile

// Emotional history (last 90 days)
`user:${userId}:emotional-history` → EmotionalState[]

// Q-tables
`q:${userId}:${stateHash}:${contentId}` → number (Q-value)

// Experience replay buffer
`user:${userId}:experiences` → EmotionalExperience[]

// Visit counts (for UCB exploration)
`user:${userId}:visit:${contentId}` → number
`user:${userId}:total-actions` → number
```

**AgentDB Operations**:
```typescript
// Store emotional state
await agentDB.set(`user:${userId}:emotional-history`, [...history, newState]);

// Get Q-value
const qValue = await agentDB.get(`q:${userId}:${stateHash}:${contentId}`) ?? 0;

// Update Q-value
await agentDB.set(`q:${userId}:${stateHash}:${contentId}`, newQValue);

// Increment visit count
await agentDB.incr(`user:${userId}:visit:${contentId}`);
```

---

## 9. Hackathon Simplifications

### ✂️ What We're Cutting for MVP

| Feature (Full PRD) | MVP Status | Rationale |
|--------------------|-----------|-----------|
| Voice emotion detection | ❌ Cut | 40+ hours implementation |
| Biometric fusion | ❌ Cut | Requires wearable integration |
| GraphQL API | ❌ Cut | REST is faster to implement |
| User authentication | ❌ Cut | Single demo user acceptable |
| Multiple users | ❌ Cut | One user demonstrates concept |
| Platform APIs (Netflix, YouTube) | ❌ Cut | Use mock catalog |
| Wellbeing crisis detection | ❌ Cut | 8+ hours, not core demo |
| Emotional journey charts | ❌ Cut | Text-based insights OK |
| Actor-Critic RL | ❌ Cut | Q-learning sufficient for demo |
| Prioritized replay buffer | ❌ Cut | Standard replay OK |
| Production deployment | ❌ Cut | Local demo only |
| Monitoring/logging | ❌ Cut | Console logs acceptable |
| A/B testing framework | ❌ Cut | Post-hackathon feature |

### ✅ What We're Keeping

| Core Feature | Included | Why Critical |
|--------------|----------|--------------|
| Gemini emotion detection | ✅ Yes | Core innovation |
| Q-learning RL policy | ✅ Yes | Core innovation |
| RuVector semantic search | ✅ Yes | Differentiator |
| Reward function | ✅ Yes | Shows RL effectiveness |
| Mock content catalog | ✅ Yes | Demonstrates recommendations |
| CLI demo | ✅ Yes | Interactive presentation |
| Post-viewing feedback | ✅ Yes | Closes RL loop |
| Emotional insights | ✅ Yes | Shows learning over time |

---

## 10. Performance Requirements (MVP Relaxed)

| Metric | Production Target | MVP Target | Notes |
|--------|------------------|------------|-------|
| Emotion detection latency | <2s | <5s | Acceptable for demo |
| Recommendation latency | <3s | <10s | First-time cold start OK |
| API response time | <1s | <3s | Synchronous acceptable |
| RuVector search | <500ms | <2s | 500-item catalog is small |
| Q-value update | <100ms | <500ms | Synchronous update OK |
| Concurrent users | 100 | 1 | Single user demo |
| Content catalog size | 10,000 | 500 | Proves concept |
| Total experiences | 200 | 20 | Enough to show learning |

---

## 11. Risk Mitigation (MVP-Specific)

### Risk: Gemini API quota exhaustion
**Mitigation**:
- Use Gemini 2.0 Flash (cheaper, faster)
- Cache emotion analyses for 5 minutes
- Fallback to neutral state on rate limit

**Fallback**:
```typescript
if (error.status === 429) {
  return {
    valence: 0,
    arousal: 0,
    primaryEmotion: 'neutral',
    confidence: 0.3
  };
}
```

---

### Risk: RuVector search slow on first run
**Mitigation**:
- Pre-build HNSW index during setup
- Use smaller efSearch (50 vs 100)
- Limit catalog to 500 items

**Optimization**:
```typescript
// Warm up index on server start
await ruVector.search({ vector: randomVector, topK: 1 });
```

---

### Risk: Q-values don't converge in 20 experiences
**Mitigation**:
- Seed Q-tables with content-based similarity
- Use optimistic initialization (Q₀ = 0.5)
- Higher learning rate (α = 0.2) for faster convergence

**Detection**:
```typescript
if (user.totalExperiences > 10 && user.avgReward < 0.4) {
  console.warn('Policy not converging, increasing exploration rate');
  user.explorationRate = 0.5; // Reset to high exploration
}
```

---

### Risk: Demo fails during presentation
**Mitigation**:
- Pre-seed demo user with 15 experiences
- Record video backup of successful run
- Prepare offline mode (pre-generated responses)

**Demo Script**:
1. Show emotional input: "I'm stressed from work"
2. Show recommendations with RL explanations
3. Simulate post-viewing: "I feel much calmer"
4. Show reward calculation and Q-value update
5. Show insights: improvement over 15 experiences

---

## 12. Testing Strategy (Minimal Viable)

### Unit Tests (8 hours)

```typescript
// tests/emotion.test.ts
describe('EmotionDetector', () => {
  it('should detect stressed state from text', async () => {
    const result = await detector.analyzeText('I am so stressed');
    expect(result.valence).toBeLessThan(-0.3);
    expect(result.arousal).toBeGreaterThan(0.3);
    expect(result.primaryEmotion).toMatch(/stress|anger|fear/);
  });
});

// tests/rl.test.ts
describe('Reward Function', () => {
  it('should give positive reward for improvement', () => {
    const before = { valence: -0.6, arousal: 0.5 };
    const after = { valence: 0.4, arousal: -0.2 };
    const desired = { valence: 0.5, arousal: -0.3 };

    const reward = calculateReward(before, after, desired);
    expect(reward).toBeGreaterThan(0.7); // Strong improvement
  });
});

// tests/api.test.ts
describe('API Integration', () => {
  it('should return recommendations from POST /api/recommend', async () => {
    const res = await request(app)
      .post('/api/recommend')
      .send({ userId: 'test-user', emotionalStateId: 'state-123' });

    expect(res.status).toBe(200);
    expect(res.body.recommendations).toHaveLength(20);
    expect(res.body.recommendations[0].qValue).toBeDefined();
  });
});
```

### Manual QA Checklist (2 hours)

- [ ] Emotion detection returns valid emotional state
- [ ] Recommendations API returns 20 items with Q-values
- [ ] Feedback API updates Q-values in AgentDB
- [ ] Insights API shows emotional journey
- [ ] CLI demo runs without errors
- [ ] Reward function gives expected values for test cases
- [ ] RuVector search returns relevant content
- [ ] AgentDB persists data across server restarts

---

## 13. Demo Script (Presentation Guide)

### Scene 1: Problem Statement (2 min)
**Presenter**: "67% of people experience 'binge regret' after watching content. Current recommendations optimize for engagement, not emotional wellbeing. EmotiStream uses reinforcement learning to learn which content actually improves your mood."

### Scene 2: Emotional Input (1 min)
**CLI Demo**:
```
EmotiStream> How are you feeling right now?
You: "I'm exhausted and stressed from work"

[Analyzing emotion with Gemini...]

Detected emotional state:
  Valence: -0.6 (negative)
  Arousal: 0.4 (moderate)
  Primary emotion: sadness/stress
  Stress level: 70%

Predicted desired state:
  Valence: 0.5 (positive)
  Arousal: -0.3 (calm)
  Confidence: 68%
```

### Scene 3: Personalized Recommendations (2 min)
**CLI Demo**:
```
Top recommendations to improve your mood:

1. "Nature Sounds: Ocean Waves" [YouTube]
   Emotional effect: Calming (valenceDelta: +0.6, arousalDelta: -0.7)
   Q-value: 0.72 (learned from 5 similar experiences)
   Predicted outcome: 88% chance you'll feel calm and positive
   Why: Based on your stressed state, this has consistently helped you relax.

2. "The Great British Bake Off" [Netflix]
   Emotional effect: Gentle uplift (valenceDelta: +0.4, arousalDelta: -0.2)
   Q-value: 0.65
   Predicted outcome: 74% chance you'll feel better
   Why: Wholesome content that reduces stress without overstimulation.

[... 18 more]
```

### Scene 4: Post-Viewing Feedback & Learning (1 min)
**CLI Demo**:
```
How do you feel after watching "Ocean Waves"?
You: "I feel much calmer and more positive"

[Analyzing post-viewing emotion...]

Emotional improvement:
  Before: valence -0.6, arousal 0.4
  After: valence 0.6, arousal -0.2
  Reward: 0.87 (strong improvement!)

Q-value updated: 0.72 → 0.78
Your RL policy is learning what works for you!
```

### Scene 5: Learning Over Time (1 min)
**CLI Demo**:
```
Your emotional journey (last 7 days):

Day       | Avg Mood | Sessions | Improvement
----------|----------|----------|------------
Dec 1     | -0.4     | 2        | +0.52
Dec 2     | -0.2     | 3        | +0.61
Dec 3     | 0.1      | 2        | +0.74
...
Dec 7     | 0.3      | 3        | +0.82

Most effective content for you:
1. "Ocean Waves" - 92% improvement (stressed → calm)
2. "Bob Ross" - 88% improvement (sad → uplifted)
3. "Planet Earth" - 85% improvement (anxious → grounded)

Your RL policy has learned what improves your mood!
```

---

## 14. Post-Hackathon Roadmap

### Week 1-2: Core Improvements
- [ ] Add voice emotion detection (Gemini multimodal)
- [ ] Implement wellbeing crisis detection
- [ ] Add user authentication (JWT)
- [ ] Expand content catalog to 5,000 items

### Week 3-4: Advanced RL
- [ ] Implement Actor-Critic algorithm
- [ ] Add prioritized experience replay
- [ ] Batch policy updates (gradient descent)
- [ ] Cross-user transfer learning

### Week 5-8: Production Ready
- [ ] GraphQL API migration
- [ ] Web UI (React + visualization)
- [ ] Platform integrations (YouTube Data API, JustWatch)
- [ ] Deploy to cloud (Railway/Render)
- [ ] A/B testing framework
- [ ] Monitoring (Prometheus + Grafana)

---

## 15. Success Criteria (MVP Demo)

| Metric | Target | Measurement |
|--------|--------|-------------|
| Emotion detection accuracy | ≥60% | Manual validation on 20 test inputs |
| Recommendations returned | 20 items | API response |
| Q-value convergence | After 15 experiences | Variance <0.1 |
| Mean reward (demo user) | ≥0.55 | After 15 experiences |
| API latency | <10s | All endpoints |
| Demo success | No errors | Live presentation |
| Audience understanding | ≥80% get concept | Post-demo survey |

---

## Appendix A: Environment Setup

### .env.example
```bash
# Gemini API
GEMINI_API_KEY=your-api-key-here

# Server
PORT=3000
NODE_ENV=development

# Storage
AGENTDB_PATH=./data/emotistream.db
RUVECTOR_INDEX_PATH=./data/content-embeddings.idx

# RL Hyperparameters
LEARNING_RATE=0.1
DISCOUNT_FACTOR=0.95
EXPLORATION_RATE=0.15
```

### Setup Commands
```bash
# Install dependencies
npm install

# Create data directory
mkdir -p data

# Generate mock content catalog and profile emotions
npm run setup

# Start development server
npm run dev

# Run CLI demo
npm run cli
```

---

## Appendix B: Mock Content Catalog Schema

```json
{
  "catalog": [
    {
      "contentId": "content-001",
      "title": "Nature Sounds: Ocean Waves",
      "description": "Relaxing ocean waves for stress relief and sleep",
      "platform": "youtube",
      "genres": ["relaxation", "nature", "meditation"],
      "duration": 3600,
      "emotionalProfile": {
        "primaryTone": "calm",
        "valenceDelta": 0.6,
        "arousalDelta": -0.7,
        "intensity": 0.3,
        "complexity": 0.2,
        "targetStates": [
          { "currentValence": -0.5, "currentArousal": 0.5, "description": "stressed" }
        ]
      }
    }
    // ... 499 more items
  ]
}
```

**Content Categories**:
- Relaxation (100 items): Nature sounds, meditation, ASMR
- Uplifting (100 items): Wholesome shows, feel-good movies
- Grounding (100 items): Documentaries, educational content
- Cathartic (100 items): Emotional dramas, sad movies
- Exciting (100 items): Action, thrillers, sports

---

## Appendix C: Key Code Snippets

### Reward Function Implementation
```typescript
export function calculateEmotionalReward(
  stateBefore: EmotionalState,
  stateAfter: EmotionalState,
  desired: { valence: number; arousal: number }
): number {
  const valenceDelta = stateAfter.valence - stateBefore.valence;
  const arousalDelta = stateAfter.arousal - stateBefore.arousal;

  const desiredValenceDelta = desired.valence - stateBefore.valence;
  const desiredArousalDelta = desired.arousal - stateBefore.arousal;

  // Direction alignment (cosine similarity)
  const actualVector = [valenceDelta, arousalDelta];
  const desiredVector = [desiredValenceDelta, desiredArousalDelta];

  const dotProduct = actualVector[0] * desiredVector[0] + actualVector[1] * desiredVector[1];
  const magnitudeActual = Math.sqrt(actualVector[0]**2 + actualVector[1]**2);
  const magnitudeDesired = Math.sqrt(desiredVector[0]**2 + desiredVector[1]**2);

  const directionAlignment = magnitudeDesired > 0
    ? dotProduct / (magnitudeActual * magnitudeDesired + 1e-6)
    : 0;

  // Magnitude of improvement
  const improvement = Math.sqrt(valenceDelta**2 + arousalDelta**2);

  // Combined reward
  const reward = directionAlignment * 0.6 + improvement * 0.4;

  // Proximity bonus
  const desiredProximity = Math.sqrt(
    (stateAfter.valence - desired.valence)**2 +
    (stateAfter.arousal - desired.arousal)**2
  );
  const proximityBonus = Math.max(0, 1 - desiredProximity) * 0.2;

  return Math.max(-1, Math.min(1, reward + proximityBonus));
}
```

### State Discretization
```typescript
function hashEmotionalState(state: EmotionalState): string {
  // 5 valence buckets: [-1, -0.6), [-0.6, -0.2), [-0.2, 0.2), [0.2, 0.6), [0.6, 1]
  const valenceBucket = Math.floor((state.valence + 1) / 0.4);

  // 5 arousal buckets: same ranges
  const arousalBucket = Math.floor((state.arousal + 1) / 0.4);

  // 3 stress buckets: [0, 0.33), [0.33, 0.67), [0.67, 1]
  const stressBucket = Math.floor(state.stressLevel / 0.33);

  return `${valenceBucket}:${arousalBucket}:${stressBucket}`;
}
```

---

**End of MVP Architecture Document**

**Status**: ✅ Ready for Hackathon Implementation
**Estimated Build Time**: 70 hours
**Critical Path Dependencies**: Gemini API key, Node.js 20+, 8GB RAM
**Demo Readiness**: Hour 55+ (CLI demo functional)

---

*This architecture prioritizes speed, simplicity, and demonstrable RL learning. All non-essential features are deferred to post-hackathon development.*
