# Cross-Solution Technical Reference

## Shared Self-Learning Components

All three hackathon solutions (StreamSense AI, WatchSphere Collective, EmotiStream Nexus) share core self-learning infrastructure powered by RuVector, AgentDB, and Agentic Flow.

---

## 1. RuVector Integration (Shared)

### 1.1 Installation & Setup

```bash
npm install ruvector
```

```typescript
import { RuVector } from 'ruvector';
import { ruvLLM } from 'ruvector/ruvLLM';

// Content embeddings (shared across all solutions)
const contentVectors = new RuVector({
  dimensions: 1536,
  indexType: 'hnsw',
  efConstruction: 200,
  M: 16,
  space: 'cosine'
});

// User preference vectors (solution-specific)
const preferenceVectors = new RuVector({
  dimensions: 1536,
  indexType: 'hnsw',
  efConstruction: 200,
  M: 16
});
```

### 1.2 Content Embedding Pipeline (Shared)

```typescript
async function embedContent(content: ContentMetadata): Promise<void> {
  // Create rich text representation
  const textRep = `
    Title: ${content.title}
    Description: ${content.description}
    Genres: ${content.genres.join(', ')}
    Cast: ${content.cast.join(', ')}
    Director: ${content.director}
    Mood: ${content.emotionalTags?.join(', ') ?? ''}
  `.trim();

  // Generate embedding with ruvLLM
  const embedding = await ruvLLM.embed(textRep);

  // Store in RuVector
  await contentVectors.upsert({
    id: `content:${content.contentId}`,
    vector: embedding,
    metadata: {
      contentId: content.contentId,
      title: content.title,
      platform: content.platform,
      genres: content.genres,
      rating: content.rating
    }
  });
}
```

### 1.3 Semantic Search (150x Faster HNSW)

```typescript
async function semanticSearch(
  queryEmbedding: Float32Array,
  topK: number = 50,
  filters?: Record<string, any>
): Promise<SearchResult[]> {
  const results = await contentVectors.search({
    vector: queryEmbedding,
    topK,
    filter: filters,
    includeMetadata: true
  });

  return results.map(r => ({
    contentId: r.id,
    title: r.metadata.title,
    similarity: r.similarity,
    metadata: r.metadata
  }));
}
```

### 1.4 Preference Vector Learning (Shared Pattern)

```typescript
async function updatePreferenceVector(
  userId: string,
  contentId: string,
  reward: number,
  learningRate: number = 0.1
): Promise<void> {
  // Get current preference
  const prefResult = await preferenceVectors.get(`user:${userId}:preferences`);
  const currentPref = prefResult?.vector ?? new Float32Array(1536);

  // Get content vector
  const contentResult = await contentVectors.get(`content:${contentId}`);
  if (!contentResult) return;

  const contentVector = contentResult.vector;

  // Learning update
  const alpha = learningRate * Math.abs(reward);
  const direction = reward > 0 ? 1 : -1;

  const updatedPref = new Float32Array(1536);
  for (let i = 0; i < 1536; i++) {
    const delta = (contentVector[i] - currentPref[i]) * alpha * direction;
    updatedPref[i] = currentPref[i] + delta;
  }

  // Normalize
  const norm = Math.sqrt(updatedPref.reduce((sum, v) => sum + v * v, 0));
  for (let i = 0; i < 1536; i++) {
    updatedPref[i] /= norm;
  }

  // Store
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

---

## 2. AgentDB Integration (Shared)

### 2.1 Installation & Setup

```bash
# AgentDB is part of Agentic Flow
npm install agentic-flow
```

```typescript
import { AgentDB } from 'agentic-flow/agentdb';

const agentDB = new AgentDB({
  persistPath: './data/memory',
  autoSave: true,
  saveInterval: 60000 // 1 minute
});
```

### 2.2 Q-Learning Q-Table Management (Shared)

```typescript
class QTableManager {
  constructor(private agentDB: AgentDB) {}

  async getQValue(stateHash: string, action: string): Promise<number> {
    return await this.agentDB.get(`q:${stateHash}:${action}`) ?? 0;
  }

  async setQValue(stateHash: string, action: string, value: number): Promise<void> {
    await this.agentDB.set(`q:${stateHash}:${action}`, value);
  }

  async updateQValue(
    stateHash: string,
    action: string,
    reward: number,
    nextStateHash: string,
    learningRate: number = 0.1,
    discountFactor: number = 0.95
  ): Promise<void> {
    const currentQ = await this.getQValue(stateHash, action);
    const maxNextQ = await this.getMaxQValue(nextStateHash);

    const newQ = currentQ + learningRate * (
      reward + discountFactor * maxNextQ - currentQ
    );

    await this.setQValue(stateHash, action, newQ);
  }

  async getMaxQValue(stateHash: string): Promise<number> {
    const pattern = `q:${stateHash}:*`;
    const keys = await this.agentDB.keys(pattern);

    if (keys.length === 0) return 0;

    const qValues = await Promise.all(
      keys.map(key => this.agentDB.get<number>(key))
    );

    return Math.max(...qValues.filter(v => v !== null) as number[]);
  }

  async getBestAction(stateHash: string): Promise<{ action: string; qValue: number } | null> {
    const pattern = `q:${stateHash}:*`;
    const keys = await this.agentDB.keys(pattern);

    if (keys.length === 0) return null;

    let bestAction = '';
    let bestValue = -Infinity;

    for (const key of keys) {
      const value = await this.agentDB.get<number>(key);
      if (value !== null && value > bestValue) {
        bestValue = value;
        bestAction = key.split(':')[2]; // Extract action from key
      }
    }

    return { action: bestAction, qValue: bestValue };
  }
}
```

### 2.3 Experience Replay Buffer (Shared)

```typescript
interface Experience {
  stateHash: string;
  action: string;
  reward: number;
  nextStateHash: string;
  timestamp: number;
  metadata?: any;
}

class ReplayBuffer {
  private maxSize = 10000;

  constructor(private agentDB: AgentDB) {}

  async addExperience(exp: Experience): Promise<void> {
    await this.agentDB.lpush('replay_buffer', exp);
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

  async prioritizedSample(batchSize: number, alpha: number = 0.6): Promise<Experience[]> {
    const bufferSize = await this.agentDB.llen('replay_buffer');
    if (bufferSize === 0) return [];

    // Get all experiences
    const allExperiences: Experience[] = [];
    for (let i = 0; i < bufferSize; i++) {
      const exp = await this.agentDB.lindex<Experience>('replay_buffer', i);
      if (exp) allExperiences.push(exp);
    }

    // Calculate priorities (proportional to |reward|)
    const priorities = allExperiences.map(exp => Math.abs(exp.reward) ** alpha);
    const totalPriority = priorities.reduce((sum, p) => sum + p, 0);

    // Sample based on priorities
    const samples: Experience[] = [];
    for (let i = 0; i < Math.min(batchSize, allExperiences.length); i++) {
      let randomValue = Math.random() * totalPriority;
      let selectedIndex = 0;

      for (let j = 0; j < priorities.length; j++) {
        randomValue -= priorities[j];
        if (randomValue <= 0) {
          selectedIndex = j;
          break;
        }
      }

      samples.push(allExperiences[selectedIndex]);
    }

    return samples;
  }

  async batchUpdate(
    qTableManager: QTableManager,
    batchSize: number = 32,
    prioritized: boolean = false
  ): Promise<void> {
    const batch = prioritized
      ? await this.prioritizedSample(batchSize)
      : await this.sampleBatch(batchSize);

    for (const exp of batch) {
      await qTableManager.updateQValue(
        exp.stateHash,
        exp.action,
        exp.reward,
        exp.nextStateHash
      );
    }
  }
}
```

### 2.4 User Profile Persistence (Shared)

```typescript
interface UserProfile {
  userId: string;
  createdAt: number;
  preferenceVectorId: string;
  totalActions: number;
  totalReward: number;
  explorationRate: number;
  metadata?: Record<string, any>;
}

class UserProfileManager {
  constructor(private agentDB: AgentDB) {}

  async getProfile(userId: string): Promise<UserProfile | null> {
    return await this.agentDB.get(`profile:${userId}`);
  }

  async createProfile(userId: string, metadata?: any): Promise<UserProfile> {
    const profile: UserProfile = {
      userId,
      createdAt: Date.now(),
      preferenceVectorId: `user:${userId}:preferences`,
      totalActions: 0,
      totalReward: 0,
      explorationRate: 0.15,
      metadata
    };

    await this.agentDB.set(`profile:${userId}`, profile);
    return profile;
  }

  async updateProfile(userId: string, updates: Partial<UserProfile>): Promise<void> {
    const profile = await this.getProfile(userId);
    if (!profile) throw new Error('Profile not found');

    const updated = { ...profile, ...updates };
    await this.agentDB.set(`profile:${userId}`, updated);
  }

  async recordAction(userId: string, reward: number): Promise<void> {
    const profile = await this.getProfile(userId);
    if (!profile) return;

    await this.updateProfile(userId, {
      totalActions: profile.totalActions + 1,
      totalReward: profile.totalReward + reward
    });
  }
}
```

---

## 3. Agentic Flow Integration (Shared)

### 3.1 Installation & Setup

```bash
npm install agentic-flow@alpha
```

### 3.2 ReasoningBank Trajectory Tracking (Shared)

```typescript
import { ReasoningBank } from 'agentic-flow/reasoningbank';

const reasoningBank = new ReasoningBank(agentDB);

// Track decision trajectory
async function trackDecision(
  userId: string,
  state: any,
  action: string,
  reward: number,
  metadata?: any
): Promise<void> {
  await reasoningBank.addTrajectory({
    userId,
    state: JSON.stringify(state),
    action,
    reward,
    timestamp: Date.now(),
    metadata
  });

  // Trigger pattern distillation periodically
  const trajectoryCount = await reasoningBank.getTrajectoryCount(userId);
  if (trajectoryCount % 100 === 0) {
    await reasoningBank.distillPatterns(userId);
  }
}

// Get learned patterns
async function getLearnedPatterns(userId: string): Promise<any[]> {
  return await reasoningBank.getPatterns(userId);
}

// Verdict judgment (classify trajectory as success/failure)
async function judgeTrajectory(
  trajectoryId: string,
  outcome: 'success' | 'failure' | 'neutral'
): Promise<void> {
  await reasoningBank.setVerdict(trajectoryId, outcome);
}
```

### 3.3 Agent Spawning via Claude Code Task Tool

```typescript
// This is conceptual - actual execution via Claude Code's Task tool

// Example: Spawn multiple agents concurrently for StreamSense
/*
[Single Message - Parallel Execution]:
  Task("Intent Analyzer", "Analyze user query and extract intent", "analyst")
  Task("Content Ranker", "Rank content candidates by Q-values", "optimizer")
  Task("Learning Coordinator", "Update Q-values and preferences from outcome", "coordinator")

  TodoWrite({ todos: [
    { content: "Analyze user intent", status: "in_progress", activeForm: "Analyzing user intent" },
    { content: "Rank content candidates", status: "pending", activeForm: "Ranking content candidates" },
    { content: "Update learning models", status: "pending", activeForm: "Updating learning models" }
  ]})
*/
```

### 3.4 Memory Coordination (Shared)

```typescript
// Use Agentic Flow memory for cross-agent coordination
async function storeCoordinationData(
  namespace: string,
  key: string,
  value: any,
  ttl?: number
): Promise<void> {
  await agentDB.set(`${namespace}:${key}`, value);

  if (ttl) {
    await agentDB.expire(`${namespace}:${key}`, ttl);
  }
}

async function getCoordinationData(namespace: string, key: string): Promise<any> {
  return await agentDB.get(`${namespace}:${key}`);
}

// Example: Share recommendation state across agents
await storeCoordinationData('streamsense', 'current-query', {
  userId: 'user123',
  query: 'Something like Succession',
  timestamp: Date.now()
});
```

---

## 4. Solution-Specific Adaptations

### 4.1 StreamSense AI

**RuVector Usage:**
- Content embeddings (title + description + genres)
- User preference vectors (single user)
- Query embeddings

**AgentDB Usage:**
- Q-tables: `q:${userId}:${stateHash}:${contentId}`
- User profiles
- Experience replay buffer

**Agentic Flow:**
- Intent analyzer agent
- Recommendation ranker agent
- Learning coordinator agent

### 4.2 WatchSphere Collective

**RuVector Usage:**
- Content embeddings (same as StreamSense)
- Individual member preference vectors (multi-user)
- Group consensus vectors (weighted average)

**AgentDB Usage:**
- Multi-agent Q-tables:
  - Consensus coordinator: `q:coordinator:${groupStateHash}:${strategy}`
  - Preference agents: `q:preference:${memberId}:${stateHash}:${contentId}`
- Group profiles
- Member profiles
- Session history

**Agentic Flow:**
- Preference agent (one per member)
- Consensus coordinator agent
- Conflict resolver agent
- Social context agent
- Safety guardian agent

### 4.3 EmotiStream Nexus

**RuVector Usage:**
- Content emotion embeddings (emotional tone + impact)
- User emotional preference vectors
- Emotional transition vectors (current → desired state)

**AgentDB Usage:**
- Emotional Q-tables: `q:emotion:${userId}:${emotionalStateHash}:${contentId}`
- Emotional history (time-series)
- Experience replay buffer (prioritized)
- Wellbeing metrics

**Agentic Flow:**
- Emotion detector agent (Gemini integration)
- Desired state predictor agent
- RL policy agent (deep RL)
- Outcome tracker agent
- Wellbeing monitor agent

---

## 5. Shared Reward Functions

### 5.1 Basic Completion Reward (StreamSense)

```typescript
function calculateBasicReward(outcome: ViewingOutcome): number {
  const completionReward = (outcome.completionRate / 100) * 0.7;
  const ratingReward = outcome.explicitRating ? (outcome.explicitRating / 5) * 0.3 : 0;

  return completionReward + ratingReward;
}
```

### 5.2 Collective Satisfaction Reward (WatchSphere)

```typescript
function calculateCollectiveReward(
  individualSatisfaction: Map<string, number>
): number {
  const satisfactionValues = Array.from(individualSatisfaction.values());

  // Avoid simple averaging (can hide low satisfaction)
  const minSatisfaction = Math.min(...satisfactionValues);
  const avgSatisfaction = satisfactionValues.reduce((sum, s) => sum + s, 0) / satisfactionValues.length;

  // Weighted: prioritize minimum (fairness) but consider average
  return minSatisfaction * 0.6 + avgSatisfaction * 0.4;
}
```

### 5.3 Emotional Improvement Reward (EmotiStream)

```typescript
function calculateEmotionalReward(
  stateBefore: EmotionalState,
  stateAfter: EmotionalState,
  desired: { valence: number; arousal: number }
): number {
  const valenceDelta = stateAfter.valence - stateBefore.valence;
  const arousalDelta = stateAfter.arousal - stateBefore.arousal;

  const desiredValenceDelta = desired.valence - stateBefore.valence;
  const desiredArousalDelta = desired.arousal - stateBefore.arousal;

  // Direction alignment
  const actualVector = [valenceDelta, arousalDelta];
  const desiredVector = [desiredValenceDelta, desiredArousalDelta];

  const dotProduct = actualVector[0] * desiredVector[0] + actualVector[1] * desiredVector[1];
  const magnitudeActual = Math.sqrt(actualVector[0]**2 + actualVector[1]**2);
  const magnitudeDesired = Math.sqrt(desiredVector[0]**2 + desiredVector[1]**2);

  const directionAlignment = magnitudeDesired > 0
    ? dotProduct / (magnitudeActual * magnitudeDesired)
    : 0;

  // Magnitude
  const improvement = magnitudeActual;

  return directionAlignment * 0.6 + improvement * 0.4;
}
```

---

## 6. Shared Learning Metrics

### 6.1 Q-Value Convergence

```typescript
async function calculateQValueConvergence(
  userId: string,
  timeWindow: number = 7 * 24 * 60 * 60 * 1000
): Promise<number> {
  // Get Q-value update history
  const updates = await agentDB.get<number[]>(`${userId}:qvalue-updates`);
  if (!updates || updates.length < 10) return 0;

  // Calculate variance
  const recent = updates.slice(-100);
  const mean = recent.reduce((sum, v) => sum + v, 0) / recent.length;
  const variance = recent.reduce((sum, v) => sum + (v - mean) ** 2, 0) / recent.length;

  // Convergence = 1 - normalized variance
  return 1 - Math.min(variance, 1);
}
```

### 6.2 Preference Vector Stability

```typescript
async function calculatePreferenceStability(
  userId: string,
  timeWindow: number = 7 * 24 * 60 * 60 * 1000
): Promise<number> {
  // Get preference vector update history
  const history = await agentDB.get<Array<{ timestamp: number; vector: Float32Array }>>(
    `${userId}:preference-history`
  );

  if (!history || history.length < 2) return 0;

  // Calculate cosine similarity between consecutive vectors
  let totalSimilarity = 0;
  for (let i = 1; i < history.length; i++) {
    const similarity = cosineSimilarity(history[i-1].vector, history[i].vector);
    totalSimilarity += similarity;
  }

  return totalSimilarity / (history.length - 1);
}

function cosineSimilarity(v1: Float32Array, v2: Float32Array): number {
  let dot = 0, norm1 = 0, norm2 = 0;
  for (let i = 0; i < v1.length; i++) {
    dot += v1[i] * v2[i];
    norm1 += v1[i] * v1[i];
    norm2 += v2[i] * v2[i];
  }
  return dot / (Math.sqrt(norm1) * Math.sqrt(norm2));
}
```

---

## 7. Shared Exploration Strategies

### 7.1 ε-Greedy Exploration

```typescript
async function selectActionEpsilonGreedy(
  userId: string,
  state: any,
  candidates: any[],
  epsilon: number = 0.15
): Promise<any> {
  if (Math.random() < epsilon) {
    // Explore: random selection
    return candidates[Math.floor(Math.random() * candidates.length)];
  }

  // Exploit: select best Q-value
  const stateHash = hashState(state);
  const qTableManager = new QTableManager(agentDB);

  let bestAction = null;
  let bestQValue = -Infinity;

  for (const candidate of candidates) {
    const qValue = await qTableManager.getQValue(stateHash, candidate.id);
    if (qValue > bestQValue) {
      bestQValue = qValue;
      bestAction = candidate;
    }
  }

  return bestAction ?? candidates[0];
}
```

### 7.2 UCB Exploration

```typescript
async function selectActionUCB(
  userId: string,
  state: any,
  candidates: any[]
): Promise<any> {
  const stateHash = hashState(state);
  const qTableManager = new QTableManager(agentDB);

  const totalActions = await agentDB.get<number>(`${userId}:total-actions`) ?? 1;

  let bestAction = null;
  let bestUCB = -Infinity;

  for (const candidate of candidates) {
    const qValue = await qTableManager.getQValue(stateHash, candidate.id);
    const visitCount = await agentDB.get<number>(`${userId}:visit:${candidate.id}`) ?? 0;

    // UCB formula
    const ucb = qValue + Math.sqrt(2 * Math.log(totalActions) / (visitCount + 1));

    if (ucb > bestUCB) {
      bestUCB = ucb;
      bestAction = candidate;
    }
  }

  return bestAction ?? candidates[0];
}
```

---

## 8. Deployment Checklist

### 8.1 RuVector Setup

- [ ] Install RuVector: `npm install ruvector`
- [ ] Initialize content vectors (1536D, HNSW)
- [ ] Initialize preference vectors (1536D, HNSW)
- [ ] Embed initial content library (1000+ items)
- [ ] Test semantic search performance (<100ms)

### 8.2 AgentDB Setup

- [ ] Install Agentic Flow: `npm install agentic-flow@alpha`
- [ ] Initialize AgentDB with persist path
- [ ] Set up Q-table schema
- [ ] Set up experience replay buffer
- [ ] Set up user profile schema
- [ ] Enable auto-save (60s interval)

### 8.3 Agentic Flow Setup

- [ ] Initialize ReasoningBank
- [ ] Define agent types (solution-specific)
- [ ] Set up memory namespaces
- [ ] Configure hooks (pre/post-task)
- [ ] Test agent coordination

### 8.4 Learning System

- [ ] Implement reward function (solution-specific)
- [ ] Implement Q-learning updates
- [ ] Implement experience replay
- [ ] Implement preference vector updates
- [ ] Set learning rate (0.1)
- [ ] Set discount factor (0.95)
- [ ] Set exploration rate (0.15)

### 8.5 Monitoring

- [ ] Track Q-value convergence
- [ ] Track preference stability
- [ ] Track reward trends
- [ ] Track exploration vs exploitation ratio
- [ ] Set up alerts for learning failures

---

## 9. Performance Targets

| Metric | StreamSense | WatchSphere | EmotiStream |
|--------|-------------|-------------|-------------|
| Query latency | <2s | <3s | <2.5s |
| Recommendation acceptance | 70% | 65% | 60% |
| Avg reward | >0.7 | >0.75 | >0.6 |
| Q-value convergence | >85% | >80% | >75% |
| Preference stability | >85% | >80% | >70% |
| Learning improvement (week 1→2) | +30% | +25% | +35% |

---

## 10. Common Utilities

### 10.1 State Hashing

```typescript
function hashState(state: any): string {
  // Create compact state representation for Q-table lookup
  // Solution-specific implementation

  // Example for StreamSense:
  // return `${userId}:${dayOfWeek}:${hourOfDay}:${socialContext}`;

  // Example for WatchSphere:
  // return `${groupId}:${groupType}:${context}`;

  // Example for EmotiStream:
  // return `${valenceBucket}:${arousalBucket}:${stressBucket}:${socialContext}`;

  return JSON.stringify(state);
}
```

### 10.2 Vector Operations

```typescript
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

function normalize(v: Float32Array): Float32Array {
  const norm = Math.sqrt(v.reduce((sum, val) => sum + val * val, 0));
  const result = new Float32Array(v.length);
  for (let i = 0; i < v.length; i++) {
    result[i] = v[i] / norm;
  }
  return result;
}
```

---

**End of Cross-Solution Reference**
