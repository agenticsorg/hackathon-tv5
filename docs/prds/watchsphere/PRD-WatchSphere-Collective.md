# Product Requirements Document: WatchSphere Collective

## 1. Executive Summary

**Problem**: Group entertainment decisions (family movie night, date night, friend gatherings) are frustrating multi-person negotiations taking 20-45 minutes with 67% of participants reporting dissatisfaction with final selection. Current platforms optimize for individuals, ignoring group dynamics, social contexts, and collective preferences.

**Solution**: WatchSphere Collective is a multi-agent AI system that learns group dynamics, facilitates consensus through intelligent voting, and optimizes for collective satisfaction. Using specialized agents for each group member plus a consensus coordinator, the system learns which voting strategies work for different social contexts and improves recommendations based on actual group enjoyment outcomes.

**Impact**: Reduce group decision time by 87% (45 min → 6 min), increase post-viewing satisfaction by 45%, and create a learning system that understands family dynamics, couple preferences, and friend group patterns. Powered by RuVector's semantic group preference matching, AgentDB's multi-profile learning, and Agentic Flow's consensus agents.

---

## 2. Problem Statement

### 2.1 Current State Analysis

**Group Decision Pain Points:**
- **45-minute average** group decision time for entertainment
- **67% dissatisfaction** with final selection (at least one person unhappy)
- **83% "veto fatigue"** - exhaustion from rejecting options
- **22% abandonment rate** - groups give up and watch nothing
- **Zero learning** - same arguments repeat every time

**Social Context Complexity:**
| Context | Avg Decision Time | Satisfaction | Key Challenge |
|---------|------------------|--------------|---------------|
| Family (kids 5-12) | 52 min | 58% | Age-appropriate content for all |
| Couple | 38 min | 71% | Mood alignment, genre compromise |
| Friends (3-5) | 41 min | 63% | Diverse taste negotiation |
| Family (teens) | 47 min | 55% | Generation gap preferences |

**Market Data:**
- 73% of streaming viewing is group viewing (2+ people)
- $87B annual market for group entertainment
- 91% of users want "better group decision tools"
- Only 12% satisfied with current group recommendation features

### 2.2 Root Cause Analysis

The fundamental problem is **lack of collective intelligence** in recommendation systems:
1. Individual recommendation engines don't understand group dynamics
2. No learning from group outcomes (only individual feedback)
3. Context-blind voting (family Sunday vs Friday date night treated same)
4. Static compromise strategies (simple averaging ignores social dynamics)
5. No conflict resolution learning (what strategies lead to satisfaction)

---

## 3. Solution Overview

### 3.1 Vision

WatchSphere Collective creates a **self-learning multi-agent consensus system** where each group member has a preference agent, and a meta-coordinator agent learns optimal voting strategies, conflict resolution patterns, and context-specific group dynamics.

### 3.2 Core Innovation: Multi-Agent Collective Learning

```
Group Context → Individual Preference Agents (N agents)
             → Candidate Content Retrieval (RuVector)
             → Preference Vectors per Member (AgentDB)
             → Consensus Voting Agent
             → Conflict Detection Agent
             → Social Context Agent (family/couple/friends)
             → Age-Appropriate Filter Agent
             → Final Recommendation
             → Group Viewing Outcome Tracking
             → Multi-Agent RL Update
             → Consensus Strategy Learning (ReasoningBank)
```

**Self-Learning Capabilities:**
- Learn optimal voting weights per social context (family vs friends vs couple)
- Discover conflict resolution patterns that maximize satisfaction
- Adapt to group composition changes (kids growing up, new members)
- Learn content-safety boundaries by age group
- Optimize for collective happiness (not average happiness)

### 3.3 Multi-Agent Architecture

**Agent Types:**
1. **Preference Agent (one per member)**: Learns individual tastes
2. **Consensus Coordinator**: Learns voting strategies
3. **Conflict Resolver**: Learns resolution patterns
4. **Social Context Agent**: Learns context-specific rules
5. **Safety Guardian**: Learns age-appropriate boundaries
6. **Outcome Tracker**: Learns from group satisfaction

---

## 4. User Stories

### 4.1 Group Setup & Management

**As a group organizer**, I want to create a "family" group with all household members, so that we can make quick decisions together.

**Acceptance Criteria:**
- Create group with name, type (family/friends/couple)
- Add members with age, relationship
- Each member gets a preference agent
- Group stored in AgentDB with unique ID

**Learning Component:**
```typescript
interface GroupProfile {
  groupId: string;
  groupType: 'family' | 'friends' | 'couple' | 'custom';
  members: Array<{
    memberId: string;
    name: string;
    age: number;
    preferenceAgentId: string;
    votingWeight: number; // learned over time
  }>;
  consensusStrategy: 'majority' | 'weighted' | 'veto' | 'learned'; // initially 'majority', evolves to 'learned'
  createdAt: number;
  totalSessions: number;
  avgSatisfaction: number;
}
```

---

**As a parent**, I want age-appropriate filtering to protect my kids while still giving them a voice in the decision.

**Acceptance Criteria:**
- Automatic content rating filtering based on youngest member
- Safety guardian agent blocks inappropriate content
- Kids still get to express preferences within safe bounds
- Parents can override safety settings per session

**Learning Component:**
- Safety agent learns age-specific boundaries (what 8yo can watch vs 12yo)
- Context-aware safety (more lenient on weekends, stricter on school nights)
- Parent override patterns inform future boundary adjustments

---

**As a couple**, I want the system to learn that my partner's mood matters more than mine on stressful days.

**Acceptance Criteria:**
- Detect stress signals (calendar integration, time of day)
- Adjust voting weights based on context
- Learn "care-taking" patterns (when to defer to partner)
- Store context-conditional weight adjustments

**Learning Component:**
```typescript
interface ContextualWeightAdjustment {
  contextSignal: 'partner-stressed' | 'user-tired' | 'celebration' | 'casual';
  memberWeights: Map<string, number>; // learned per member
  outcomeReward: number; // satisfaction after applying this strategy
}
```

---

**As a group member**, I want to veto options I've already seen or strongly dislike, without dominating the decision.

**Acceptance Criteria:**
- Soft veto: "Don't prefer" (reduces score)
- Hard veto: "Never show" (removes from candidates)
- Veto budget (limited vetos to prevent abuse)
- Veto reasons captured for learning

**Learning Component:**
- Track veto patterns → preference vector updates
- Learn "veto effectiveness" (did it improve satisfaction?)
- Adjust veto budgets based on group dynamics
- Detect serial vetoers and reduce their weights

---

**As a friend group**, I want the system to suggest content that sparks conversation, not just high ratings.

**Acceptance Criteria:**
- "Social value" scoring (controversial, thought-provoking)
- Learn friend group preferences vs solo preferences
- Optimize for engagement, not just completion
- Track post-viewing discussion signals

**Learning Component:**
- Reward function includes "engagement time after viewing"
- Learn content features that spark discussion
- Friend-specific preference vectors (different from solo)

---

**As a returning group**, I want the system to remember our previous sessions and improve over time.

**Acceptance Criteria:**
- AgentDB persistent memory per group
- Historical session outcomes inform future recommendations
- Preference vectors evolve with group dynamics
- Context patterns recognized (Friday night = comedy)

**Learning Component:**
```typescript
interface GroupSession {
  sessionId: string;
  groupId: string;
  context: SocialContext;

  // Decision process
  candidatesPresented: string[];
  votingRound: Array<{
    contentId: string;
    votes: Map<string, number>; // memberId → vote score
    vetoCount: number;
    consensusScore: number;
  }>;

  // Outcome
  finalSelection: string;
  viewingOutcome: {
    completionRate: number;
    individualSatisfaction: Map<string, number>; // per member
    collectiveSatisfaction: number; // aggregate metric
    socialEngagement: number; // conversation, laughs
  };

  // Learning
  reward: number; // collective satisfaction
  strategyUsed: string;
  timestamp: number;
}
```

---

**As a user**, I want to see why the group recommendation was made and adjust the logic.

**Acceptance Criteria:**
- Transparency: "Selected because Alice (weight 1.2) and Bob (0.9) both rated it 4/5"
- Adjustable: "Give Alice more weight tonight"
- Explainable: "Strategy used: weighted voting with conflict resolution"
- Feedback loop: "Did this work for you? Y/N"

**Learning Component:**
- ReasoningBank trajectory tracking per group decision
- Verdict judgment: success = collective satisfaction >4/5
- Pattern distillation: what strategies work for which contexts

---

## 5. Technical Architecture

### 5.1 System Architecture (ASCII Diagram)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     WatchSphere Collective Platform                      │
└─────────────────────────────────────────────────────────────────────────┘

┌────────────────┐         ┌────────────────────────────────────────────┐
│  Group Device  │────────▶│      API Gateway (GraphQL)                 │
│  (Shared TV)   │         │  - Multi-member auth                       │
└────────────────┘         │  - Group session management                │
                           │  - Real-time voting sync                   │
                           └────────────────────────────────────────────┘
                                             │
                      ┌──────────────────────┼──────────────────────────┐
                      ▼                      ▼                          ▼
          ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐
          │  Multi-Agent System │  │  Consensus Engine   │  │  Learning Engine    │
          │  (Agentic Flow)     │  │  (Voting & Resolve) │  │  (Multi-Agent RL)   │
          │                     │  │                     │  │                     │
          │ • Preference agents │  │ • Voting agent      │  │ • Strategy learning │
          │   (N per group)     │  │ • Conflict resolver │  │ • Weight optimization│
          │ • Context agent     │  │ • Social optimizer  │  │ • Pattern discovery │
          │ • Safety guardian   │  │ • Fairness monitor  │  │ • Outcome tracking  │
          └─────────────────────┘  └─────────────────────┘  └─────────────────────┘
                      │                      │                          │
                      └──────────────────────┼──────────────────────────┘
                                             ▼
                      ┌────────────────────────────────────────────────┐
                      │         RuVector Semantic Store                │
                      │                                                │
                      │  • Content embeddings (1536D)                 │
                      │  • Individual preference vectors              │
                      │  • Group consensus vectors                    │
                      │  • Context embeddings                         │
                      │  • Safety boundary vectors                    │
                      └────────────────────────────────────────────────┘
                                             │
                      ┌──────────────────────┼──────────────────────────┐
                      ▼                      ▼                          ▼
          ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐
          │      AgentDB        │  │   ReasoningBank     │  │   Platform APIs     │
          │                     │  │   (Agentic Flow)    │  │                     │
          │ • Group profiles    │  │ • Decision trees    │  │ • Netflix           │
          │ • Member profiles   │  │ • Consensus paths   │  │ • Disney+           │
          │ • Session history   │  │ • Conflict patterns │  │ • Prime Video       │
          │ • Voting weights    │  │ • Strategy verdicts │  │ • Apple TV+         │
          │ • Q-tables (multi)  │  │ • Pattern library   │  │ • HBO Max           │
          └─────────────────────┘  └─────────────────────┘  └─────────────────────┘
```

### 5.2 Multi-Agent Self-Learning Architecture

#### 5.2.1 Agent Network Design

```typescript
// Preference Agent (one per group member)
class PreferenceAgent {
  constructor(
    public memberId: string,
    private agentDB: AgentDB,
    private ruVector: RuVectorClient
  ) {}

  async getPreferenceVector(context: SocialContext): Promise<Float32Array> {
    // Try context-specific preference
    const contextKey = `${context.groupType}:${context.social}`;
    const contextPref = await this.ruVector.get(
      `member:${this.memberId}:context:${contextKey}`
    );

    if (contextPref && contextPref.metadata.sampleSize > 10) {
      return contextPref.vector;
    }

    // Fallback to general preference
    const generalPref = await this.ruVector.get(`member:${this.memberId}:preferences`);
    return generalPref?.vector ?? new Float32Array(1536);
  }

  async voteOnContent(
    contentId: string,
    context: SocialContext
  ): Promise<{ score: number; confidence: number; reasoning: string }> {
    // Get preference vector
    const prefVector = await this.getPreferenceVector(context);

    // Get content vector
    const contentResult = await this.ruVector.get(`content:${contentId}`);
    if (!contentResult) {
      return { score: 0, confidence: 0, reasoning: 'Content not found' };
    }

    // Calculate similarity score
    const score = this.cosineSimilarity(prefVector, contentResult.vector);

    // Get confidence from historical voting accuracy
    const confidence = await this.getVotingConfidence(context);

    return {
      score,
      confidence,
      reasoning: `Match: ${(score * 100).toFixed(0)}%, Confidence: ${(confidence * 100).toFixed(0)}%`
    };
  }

  async updateFromOutcome(
    contentId: string,
    individualSatisfaction: number,
    context: SocialContext
  ): Promise<void> {
    // Get current preference
    const prefVector = await this.getPreferenceVector(context);

    // Get content vector
    const contentResult = await this.ruVector.get(`content:${contentId}`);
    if (!contentResult) return;

    // Learning rate proportional to satisfaction
    const alpha = 0.1 * individualSatisfaction;
    const direction = individualSatisfaction > 0.5 ? 1 : -1;

    // Update preference vector
    const updatedPref = new Float32Array(1536);
    for (let i = 0; i < 1536; i++) {
      const delta = (contentResult.vector[i] - prefVector[i]) * alpha * direction;
      updatedPref[i] = prefVector[i] + delta;
    }

    // Store updated preference
    const contextKey = `${context.groupType}:${context.social}`;
    await this.ruVector.upsert({
      id: `member:${this.memberId}:context:${contextKey}`,
      vector: updatedPref,
      metadata: {
        memberId: this.memberId,
        contextKey,
        lastUpdate: Date.now(),
        sampleSize: (await this.getSampleSize(contextKey)) + 1
      }
    });
  }

  private cosineSimilarity(v1: Float32Array, v2: Float32Array): number {
    let dot = 0, norm1 = 0, norm2 = 0;
    for (let i = 0; i < v1.length; i++) {
      dot += v1[i] * v2[i];
      norm1 += v1[i] * v1[i];
      norm2 += v2[i] * v2[i];
    }
    return dot / (Math.sqrt(norm1) * Math.sqrt(norm2));
  }
}
```

```typescript
// Consensus Coordinator Agent
class ConsensusCoordinator {
  private learningRate = 0.1;
  private discountFactor = 0.95;

  constructor(
    private agentDB: AgentDB,
    private reasoningBank: ReasoningBankClient
  ) {}

  async determineConsensus(
    groupId: string,
    candidates: ContentCandidate[],
    votes: Map<string, Map<string, number>>, // memberId → contentId → score
    context: SocialContext
  ): Promise<ConsensusResult> {
    // Get group profile
    const group = await this.agentDB.get<GroupProfile>(`group:${groupId}`);
    if (!group) throw new Error('Group not found');

    // Get learned voting weights
    const weights = await this.getLearnedWeights(groupId, context);

    // Calculate weighted scores
    const weightedScores = new Map<string, number>();

    for (const candidate of candidates) {
      let totalScore = 0;
      let totalWeight = 0;

      for (const member of group.members) {
        const memberVotes = votes.get(member.memberId);
        if (!memberVotes) continue;

        const vote = memberVotes.get(candidate.contentId) ?? 0;
        const weight = weights.get(member.memberId) ?? 1.0;

        totalScore += vote * weight;
        totalWeight += weight;
      }

      const avgScore = totalWeight > 0 ? totalScore / totalWeight : 0;
      weightedScores.set(candidate.contentId, avgScore);
    }

    // Detect conflicts (high variance in votes)
    const conflicts = this.detectConflicts(candidates, votes);

    if (conflicts.length > 0) {
      // Use conflict resolver agent
      return await this.resolveConflicts(groupId, candidates, votes, weights, context);
    }

    // No conflicts: select highest weighted score
    const sorted = Array.from(weightedScores.entries())
      .sort((a, b) => b[1] - a[1]);

    return {
      selectedContentId: sorted[0][0],
      consensusScore: sorted[0][1],
      strategyUsed: 'weighted-voting',
      weights,
      conflicts: []
    };
  }

  private detectConflicts(
    candidates: ContentCandidate[],
    votes: Map<string, Map<string, number>>
  ): ConflictDescription[] {
    const conflicts: ConflictDescription[] = [];

    for (const candidate of candidates) {
      const scores: number[] = [];

      for (const [memberId, memberVotes] of votes.entries()) {
        const score = memberVotes.get(candidate.contentId);
        if (score !== undefined) scores.push(score);
      }

      if (scores.length < 2) continue;

      // Calculate variance
      const mean = scores.reduce((sum, s) => sum + s, 0) / scores.length;
      const variance = scores.reduce((sum, s) => sum + Math.pow(s - mean, 2), 0) / scores.length;

      // High variance = conflict
      if (variance > 0.3) {
        conflicts.push({
          contentId: candidate.contentId,
          variance,
          votes: scores,
          severity: variance > 0.5 ? 'high' : 'medium'
        });
      }
    }

    return conflicts;
  }

  private async resolveConflicts(
    groupId: string,
    candidates: ContentCandidate[],
    votes: Map<string, Map<string, number>>,
    weights: Map<string, number>,
    context: SocialContext
  ): Promise<ConsensusResult> {
    // Get historical conflict resolution strategies
    const historicalStrategies = await this.agentDB.get<ConflictStrategy[]>(
      `group:${groupId}:conflict-strategies`
    ) ?? [];

    // Find best strategy for this context
    const contextStrategies = historicalStrategies.filter(
      s => s.context.groupType === context.groupType && s.successRate > 0.6
    );

    let strategyToUse: ConflictResolutionStrategy;

    if (contextStrategies.length > 0) {
      // Use learned strategy
      strategyToUse = contextStrategies.sort((a, b) => b.successRate - a.successRate)[0].strategy;
    } else {
      // Default: round-robin fairness
      strategyToUse = 'round-robin';
    }

    // Apply strategy
    switch (strategyToUse) {
      case 'round-robin':
        return await this.applyRoundRobin(groupId, candidates, votes, weights);

      case 'highest-satisfaction':
        return await this.applyHighestSatisfaction(candidates, votes, weights);

      case 'veto-elimination':
        return await this.applyVetoElimination(groupId, candidates, votes);

      case 'compromise-search':
        return await this.applyCompromiseSearch(candidates, votes, weights);

      default:
        // Fallback to weighted voting
        return await this.determineConsensus(groupId, candidates, votes, context);
    }
  }

  private async applyRoundRobin(
    groupId: string,
    candidates: ContentCandidate[],
    votes: Map<string, Map<string, number>>,
    weights: Map<string, number>
  ): Promise<ConsensusResult> {
    // Get last chooser
    const lastChooser = await this.agentDB.get<string>(`group:${groupId}:last-chooser`);

    // Find next member in rotation
    const group = await this.agentDB.get<GroupProfile>(`group:${groupId}`);
    if (!group) throw new Error('Group not found');

    const memberIds = group.members.map(m => m.memberId);
    const lastIndex = lastChooser ? memberIds.indexOf(lastChooser) : -1;
    const nextIndex = (lastIndex + 1) % memberIds.length;
    const nextChooser = memberIds[nextIndex];

    // Get this member's top choice
    const chooserVotes = votes.get(nextChooser);
    if (!chooserVotes) throw new Error('Chooser votes not found');

    const topChoice = Array.from(chooserVotes.entries())
      .sort((a, b) => b[1] - a[1])[0];

    // Store next chooser
    await this.agentDB.set(`group:${groupId}:last-chooser`, nextChooser);

    return {
      selectedContentId: topChoice[0],
      consensusScore: topChoice[1],
      strategyUsed: 'round-robin',
      weights,
      conflicts: [],
      fairnessMetric: 1.0 // Perfect fairness
    };
  }

  private async applyCompromiseSearch(
    candidates: ContentCandidate[],
    votes: Map<string, Map<string, number>>,
    weights: Map<string, number>
  ): Promise<ConsensusResult> {
    // Find content that minimizes dissatisfaction (max-min optimization)
    let bestContent = '';
    let bestMinScore = -Infinity;

    for (const candidate of candidates) {
      const scores: number[] = [];

      for (const [memberId, memberVotes] of votes.entries()) {
        const score = memberVotes.get(candidate.contentId) ?? 0;
        scores.push(score);
      }

      const minScore = Math.min(...scores);

      if (minScore > bestMinScore) {
        bestMinScore = minScore;
        bestContent = candidate.contentId;
      }
    }

    return {
      selectedContentId: bestContent,
      consensusScore: bestMinScore,
      strategyUsed: 'compromise-search',
      weights,
      conflicts: [],
      fairnessMetric: 0.9 // High fairness
    };
  }

  async learnFromOutcome(
    groupId: string,
    sessionId: string,
    collectiveSatisfaction: number,
    individualSatisfaction: Map<string, number>,
    context: SocialContext
  ): Promise<void> {
    // Get session details
    const session = await this.agentDB.get<GroupSession>(`session:${sessionId}`);
    if (!session) return;

    // Calculate reward (collective satisfaction)
    const reward = collectiveSatisfaction;

    // Update voting weights based on outcome
    const currentWeights = await this.getLearnedWeights(groupId, context);

    for (const [memberId, satisfaction] of individualSatisfaction.entries()) {
      const currentWeight = currentWeights.get(memberId) ?? 1.0;

      // If member was dissatisfied, reduce their weight (paradoxically improves group outcomes)
      // If satisfied, increase weight slightly
      const weightDelta = (satisfaction - 0.5) * this.learningRate;
      const newWeight = Math.max(0.5, Math.min(2.0, currentWeight + weightDelta));

      await this.agentDB.set(
        `group:${groupId}:weight:${memberId}:${context.groupType}`,
        newWeight
      );
    }

    // Update strategy success rate
    const strategyUsed = session.strategyUsed;
    const strategies = await this.agentDB.get<ConflictStrategy[]>(
      `group:${groupId}:conflict-strategies`
    ) ?? [];

    const existingStrategy = strategies.find(
      s => s.strategy === strategyUsed && s.context.groupType === context.groupType
    );

    if (existingStrategy) {
      // Update success rate with exponential moving average
      existingStrategy.successRate = existingStrategy.successRate * 0.9 + reward * 0.1;
      existingStrategy.sampleSize += 1;
    } else {
      // New strategy
      strategies.push({
        strategy: strategyUsed as ConflictResolutionStrategy,
        context,
        successRate: reward,
        sampleSize: 1
      });
    }

    await this.agentDB.set(`group:${groupId}:conflict-strategies`, strategies);

    // Track trajectory in ReasoningBank
    await this.reasoningBank.addTrajectory({
      groupId,
      sessionId,
      strategyUsed,
      reward: collectiveSatisfaction,
      weights: Array.from(currentWeights.entries()),
      timestamp: Date.now()
    });
  }

  private async getLearnedWeights(
    groupId: string,
    context: SocialContext
  ): Promise<Map<string, number>> {
    const group = await this.agentDB.get<GroupProfile>(`group:${groupId}`);
    if (!group) return new Map();

    const weights = new Map<string, number>();

    for (const member of group.members) {
      const weight = await this.agentDB.get<number>(
        `group:${groupId}:weight:${member.memberId}:${context.groupType}`
      ) ?? 1.0;

      weights.set(member.memberId, weight);
    }

    return weights;
  }
}
```

```typescript
// Safety Guardian Agent
class SafetyGuardian {
  constructor(
    private agentDB: AgentDB,
    private ruVector: RuVectorClient
  ) {}

  async filterContent(
    candidates: ContentCandidate[],
    groupId: string
  ): Promise<ContentCandidate[]> {
    // Get group profile
    const group = await this.agentDB.get<GroupProfile>(`group:${groupId}`);
    if (!group) return candidates;

    // Find youngest member
    const youngestAge = Math.min(...group.members.map(m => m.age));

    // Get learned safety boundaries for this age
    const safetyBoundaries = await this.getLearnedBoundaries(youngestAge);

    // Filter candidates
    const safeContent = candidates.filter(candidate =>
      this.isSafe(candidate, safetyBoundaries)
    );

    return safeContent;
  }

  private async getLearnedBoundaries(age: number): Promise<SafetyBoundaries> {
    // Get learned boundaries from parent overrides
    const boundaries = await this.agentDB.get<SafetyBoundaries>(`safety:age:${age}`);

    if (boundaries && boundaries.sampleSize > 10) {
      return boundaries;
    }

    // Default boundaries based on age
    return this.getDefaultBoundaries(age);
  }

  private getDefaultBoundaries(age: number): SafetyBoundaries {
    if (age < 7) {
      return {
        maxRating: 'G',
        blockedGenres: ['horror', 'thriller'],
        blockedThemes: ['violence', 'adult-themes'],
        sampleSize: 0
      };
    } else if (age < 13) {
      return {
        maxRating: 'PG',
        blockedGenres: ['horror'],
        blockedThemes: ['graphic-violence', 'sexual-content'],
        sampleSize: 0
      };
    } else if (age < 17) {
      return {
        maxRating: 'PG-13',
        blockedGenres: [],
        blockedThemes: ['graphic-sexual-content'],
        sampleSize: 0
      };
    } else {
      return {
        maxRating: 'R',
        blockedGenres: [],
        blockedThemes: [],
        sampleSize: 0
      };
    }
  }

  private isSafe(candidate: ContentCandidate, boundaries: SafetyBoundaries): boolean {
    // Check rating
    const ratingOrder = ['G', 'PG', 'PG-13', 'R', 'NC-17'];
    const maxRatingIndex = ratingOrder.indexOf(boundaries.maxRating);
    const candidateRatingIndex = ratingOrder.indexOf(candidate.rating);

    if (candidateRatingIndex > maxRatingIndex) {
      return false;
    }

    // Check genres
    for (const genre of candidate.genres) {
      if (boundaries.blockedGenres.includes(genre.toLowerCase())) {
        return false;
      }
    }

    // Check themes
    for (const theme of candidate.themes ?? []) {
      if (boundaries.blockedThemes.includes(theme.toLowerCase())) {
        return false;
      }
    }

    return true;
  }

  async learnFromOverride(
    age: number,
    contentId: string,
    allowed: boolean
  ): Promise<void> {
    // Get content metadata
    const content = await this.agentDB.get<ContentMetadata>(`content:${contentId}`);
    if (!content) return;

    // Get current boundaries
    const boundaries = await this.getLearnedBoundaries(age);

    // Update boundaries based on override
    if (allowed && content.rating) {
      // Parent allowed more lenient content
      const ratingOrder = ['G', 'PG', 'PG-13', 'R', 'NC-17'];
      const currentMaxIndex = ratingOrder.indexOf(boundaries.maxRating);
      const allowedIndex = ratingOrder.indexOf(content.rating);

      if (allowedIndex > currentMaxIndex) {
        boundaries.maxRating = content.rating as any;
      }

      // Remove genre blocks if applicable
      for (const genre of content.genres) {
        const index = boundaries.blockedGenres.indexOf(genre.toLowerCase());
        if (index > -1) {
          boundaries.blockedGenres.splice(index, 1);
        }
      }
    } else if (!allowed) {
      // Parent blocked content, add to restrictions
      for (const genre of content.genres) {
        if (!boundaries.blockedGenres.includes(genre.toLowerCase())) {
          boundaries.blockedGenres.push(genre.toLowerCase());
        }
      }
    }

    // Increment sample size
    boundaries.sampleSize += 1;

    // Store updated boundaries
    await this.agentDB.set(`safety:age:${age}`, boundaries);
  }
}
```

---

## 6. Data Models

### 6.1 Core Entities

```typescript
// Group Profile (AgentDB)
interface GroupProfile {
  groupId: string;
  groupName: string;
  groupType: 'family' | 'friends' | 'couple' | 'custom';

  members: Array<{
    memberId: string;
    name: string;
    age: number;
    relationshipToOrganizer: string;
    preferenceAgentId: string;
    votingWeight: number; // learned, starts at 1.0
    vetoCount: number; // track veto usage
    satisfactionHistory: number[]; // last N sessions
  }>;

  // Learning state
  consensusStrategy: ConflictResolutionStrategy;
  totalSessions: number;
  avgCollectiveSatisfaction: number;
  avgIndividualSatisfaction: Map<string, number>;

  // Context patterns
  contextProfiles: Map<string, {
    strategy: ConflictResolutionStrategy;
    weights: Map<string, number>;
    successRate: number;
  }>;

  createdAt: number;
  lastSessionAt: number;
}

// Group Session (AgentDB - Experience for Multi-Agent RL)
interface GroupSession {
  sessionId: string;
  groupId: string;

  // Context
  context: {
    groupType: 'family' | 'friends' | 'couple';
    social: string; // 'movie-night', 'date-night', 'casual'
    timestamp: number;
    dayOfWeek: number;
    hourOfDay: number;
    location: 'home' | 'theater' | 'other';
  };

  // State before
  stateBefore: {
    memberPreferences: Map<string, Float32Array>; // preference vectors
    memberWeights: Map<string, number>;
    recentViewingHistory: string[];
  };

  // Decision process
  candidatesPresented: ContentCandidate[];

  votingRounds: Array<{
    roundNumber: number;
    votes: Map<string, Map<string, number>>; // memberId → contentId → score
    vetoes: Map<string, string[]>; // memberId → contentIds vetoed
    consensusScore: number;
    conflictsDetected: ConflictDescription[];
  }>;

  strategyUsed: ConflictResolutionStrategy;

  // Outcome
  finalSelection: string;

  viewingOutcome: {
    started: boolean;
    startTime?: number;
    endTime?: number;
    completionRate: number; // 0-100%

    // Individual feedback
    individualSatisfaction: Map<string, number>; // memberId → satisfaction (0-1)
    individualRatings: Map<string, number>; // memberId → rating (1-5)

    // Collective metrics
    collectiveSatisfaction: number; // aggregate (0-1)
    socialEngagement: number; // conversation, laughs, interaction
    fairnessScore: number; // how fair was the process
  };

  // Reward
  reward: number; // = collectiveSatisfaction

  // State after
  stateAfter: {
    memberPreferences: Map<string, Float32Array>; // updated preferences
    memberWeights: Map<string, number>; // updated weights
  };

  timestamp: number;
}

// Consensus Result
interface ConsensusResult {
  selectedContentId: string;
  consensusScore: number; // 0-1 (how much agreement)
  strategyUsed: ConflictResolutionStrategy;
  weights: Map<string, number>; // weights used
  conflicts: ConflictDescription[];
  fairnessMetric?: number; // 0-1 (how fair was the process)
  explanation: string;
}

// Conflict Description
interface ConflictDescription {
  contentId: string;
  variance: number; // variance in votes
  votes: number[];
  severity: 'low' | 'medium' | 'high';
  involvedMembers: string[];
}

// Conflict Resolution Strategy
type ConflictResolutionStrategy =
  | 'weighted-voting'     // Learned weights
  | 'round-robin'         // Take turns choosing
  | 'highest-satisfaction' // Maximize sum of satisfaction
  | 'compromise-search'   // Minimize dissatisfaction (max-min)
  | 'veto-elimination'    // Eliminate vetoed options
  | 'learned';            // Meta-learned strategy

// Conflict Strategy (learned)
interface ConflictStrategy {
  strategy: ConflictResolutionStrategy;
  context: SocialContext;
  successRate: number; // 0-1
  sampleSize: number;
  avgCollectiveSatisfaction: number;
  avgFairnessScore: number;
}

// Safety Boundaries (learned per age)
interface SafetyBoundaries {
  maxRating: 'G' | 'PG' | 'PG-13' | 'R' | 'NC-17';
  blockedGenres: string[];
  blockedThemes: string[];
  sampleSize: number; // how many parent overrides
}

// Social Context
interface SocialContext {
  groupType: 'family' | 'friends' | 'couple' | 'custom';
  social: 'movie-night' | 'date-night' | 'casual' | 'celebration' | 'kids-bedtime';
  timestamp: number;
  dayOfWeek: number;
  hourOfDay: number;
}
```

---

## 7. API Specifications

### 7.1 GraphQL Schema

```graphql
type Query {
  # Group management
  group(groupId: ID!): Group!
  myGroups: [Group!]!

  # Discovery for group
  groupDiscover(input: GroupDiscoverInput!): GroupDiscoveryResult!

  # Session history
  groupSessions(groupId: ID!, limit: Int = 10): [GroupSession!]!

  # Learning insights
  groupInsights(groupId: ID!): GroupLearningInsights!
}

type Mutation {
  # Group CRUD
  createGroup(input: CreateGroupInput!): Group!
  addGroupMember(groupId: ID!, member: GroupMemberInput!): Group!
  updateGroupMember(groupId: ID!, memberId: ID!, updates: GroupMemberUpdateInput!): Group!

  # Voting
  submitVote(sessionId: ID!, memberId: ID!, votes: [VoteInput!]!): VotingResult!
  submitVeto(sessionId: ID!, memberId: ID!, contentId: ID!, reason: String): VotingResult!

  # Outcome tracking
  trackGroupViewing(input: GroupViewingOutcomeInput!): TrackingResult!
  submitIndividualFeedback(
    sessionId: ID!,
    memberId: ID!,
    satisfaction: Float!,
    rating: Int
  ): FeedbackResult!

  # Safety overrides
  overrideSafety(
    groupId: ID!,
    contentId: ID!,
    allowed: Boolean!,
    reason: String
  ): Group!
}

input CreateGroupInput {
  groupName: String!
  groupType: GroupType!
  members: [GroupMemberInput!]!
}

input GroupMemberInput {
  name: String!
  age: Int!
  relationshipToOrganizer: String
}

enum GroupType {
  FAMILY
  FRIENDS
  COUPLE
  CUSTOM
}

input GroupDiscoverInput {
  groupId: ID!
  query: String!
  context: GroupContextInput
  limit: Int = 20
}

input GroupContextInput {
  social: SocialContextType
  location: Location
}

enum SocialContextType {
  MOVIE_NIGHT
  DATE_NIGHT
  CASUAL
  CELEBRATION
  KIDS_BEDTIME
}

type GroupDiscoveryResult {
  candidates: [ContentCandidate!]!
  votingSessionId: ID! # Start voting session
  safetyFiltered: Int # How many filtered for safety

  # Pre-voting predictions
  predictedConsensus: ContentCandidate
  predictedConflicts: [ConflictPrediction!]!

  # Recommendations
  recommendedStrategy: ConflictResolutionStrategy!
}

type ContentCandidate {
  contentId: ID!
  title: String!
  platform: String!
  metadata: ContentMetadata!

  # Pre-voting scores (based on individual preferences)
  memberScores: [MemberScore!]!
  predictedConsensusScore: Float!

  # Safety
  safeForGroup: Boolean!
  ageAppropriate: Boolean!
}

type MemberScore {
  memberId: ID!
  memberName: String!
  score: Float! # 0-1 preference match
  confidence: Float!
}

type ConflictPrediction {
  contentId: ID!
  conflictSeverity: ConflictSeverity!
  involvedMembers: [ID!]!
  reason: String!
}

enum ConflictSeverity {
  LOW
  MEDIUM
  HIGH
}

enum ConflictResolutionStrategy {
  WEIGHTED_VOTING
  ROUND_ROBIN
  HIGHEST_SATISFACTION
  COMPROMISE_SEARCH
  VETO_ELIMINATION
  LEARNED
}

input VoteInput {
  contentId: ID!
  score: Float! # 0-1
}

type VotingResult {
  votesRecorded: Int!
  allVotesIn: Boolean!
  consensusReached: Boolean!

  # If consensus reached
  consensus: ConsensusResult
}

type ConsensusResult {
  selectedContentId: ID!
  consensusScore: Float!
  strategyUsed: ConflictResolutionStrategy!

  # Transparency
  weights: [MemberWeight!]!
  conflicts: [ConflictDescription!]!
  fairnessMetric: Float

  explanation: String!
}

type MemberWeight {
  memberId: ID!
  memberName: String!
  weight: Float!
  reasoning: String
}

input GroupViewingOutcomeInput {
  sessionId: ID!
  started: Boolean!
  startTime: DateTime
  endTime: DateTime
  completionRate: Float!
  socialEngagement: Float # 0-1 (how much interaction)
}

type GroupLearningInsights {
  # Group dynamics
  avgCollectiveSatisfaction: Float!
  totalSessions: Int!

  # Member dynamics
  memberStats: [MemberStats!]!

  # Strategy effectiveness
  strategyPerformance: [StrategyPerformance!]!

  # Preference evolution
  groupPreferenceEvolution: [PreferenceSnapshot!]!

  # Context patterns
  contextPatterns: [ContextPattern!]!
}

type MemberStats {
  memberId: ID!
  memberName: String!

  avgSatisfaction: Float!
  currentWeight: Float!
  vetoCount: Int!

  preferredGenres: [String!]!
  contextSpecificPreferences: [ContextPreference!]!
}

type StrategyPerformance {
  strategy: ConflictResolutionStrategy!
  successRate: Float!
  avgSatisfaction: Float!
  avgFairness: Float!
  timesUsed: Int!
}

type ContextPreference {
  context: String!
  preferredGenres: [String!]!
  avgSatisfaction: Float!
}
```

---

## 8. RuVector Integration Patterns

### 8.1 Multi-User Preference Embeddings

```typescript
import { RuVector } from 'ruvector';
import { ruvLLM } from 'ruvector/ruvLLM';

const memberPreferences = new RuVector({
  dimensions: 1536,
  indexType: 'hnsw',
  efConstruction: 200,
  M: 16
});

const groupConsensusVectors = new RuVector({
  dimensions: 1536,
  indexType: 'hnsw',
  efConstruction: 200,
  M: 16
});

// Create group consensus vector from member preferences
async function createGroupConsensusVector(
  groupId: string,
  memberIds: string[],
  weights: Map<string, number>
): Promise<Float32Array> {
  // Get all member preference vectors
  const memberVectors: Array<{ vector: Float32Array; weight: number }> = [];

  for (const memberId of memberIds) {
    const prefResult = await memberPreferences.get(`member:${memberId}:preferences`);
    if (!prefResult) continue;

    const weight = weights.get(memberId) ?? 1.0;
    memberVectors.push({ vector: prefResult.vector, weight });
  }

  // Weighted average
  const consensusVector = new Float32Array(1536);
  let totalWeight = 0;

  for (const { vector, weight } of memberVectors) {
    for (let i = 0; i < 1536; i++) {
      consensusVector[i] += vector[i] * weight;
    }
    totalWeight += weight;
  }

  // Normalize
  for (let i = 0; i < 1536; i++) {
    consensusVector[i] /= totalWeight;
  }

  // Store group consensus vector
  await groupConsensusVectors.upsert({
    id: `group:${groupId}:consensus`,
    vector: consensusVector,
    metadata: {
      groupId,
      memberCount: memberIds.length,
      lastUpdate: Date.now()
    }
  });

  return consensusVector;
}

// Search with group consensus
async function searchForGroup(
  groupId: string,
  query: string,
  memberIds: string[],
  weights: Map<string, number>
): Promise<ContentCandidate[]> {
  // Embed query
  const queryEmbedding = await ruvLLM.embed(query);

  // Get group consensus vector
  const consensusVector = await createGroupConsensusVector(groupId, memberIds, weights);

  // Combine query + consensus
  const combinedVector = weightedAverage(queryEmbedding, consensusVector, 0.5, 0.5);

  // Search
  const results = await contentVectors.search({
    vector: combinedVector,
    topK: 30,
    includeMetadata: true
  });

  // For each result, get individual member scores
  const candidates = await Promise.all(
    results.map(async (result) => {
      const memberScores = await Promise.all(
        memberIds.map(async (memberId) => {
          const prefResult = await memberPreferences.get(`member:${memberId}:preferences`);
          if (!prefResult) return { memberId, score: 0, confidence: 0 };

          const score = cosineSimilarity(prefResult.vector, result.vector);
          const confidence = await getMemberConfidence(memberId);

          return { memberId, score, confidence };
        })
      );

      return {
        contentId: result.id,
        relevanceScore: result.similarity,
        memberScores,
        metadata: result.metadata
      };
    })
  );

  return candidates;
}
```

---

## 9. AgentDB Integration Patterns

### 9.1 Multi-Agent Q-Tables

```typescript
// Separate Q-tables for each agent
class MultiAgentQLearning {
  constructor(private agentDB: AgentDB) {}

  // Consensus Coordinator Q-table: state → strategy → Q-value
  async getCoordinatorQValue(
    groupStateHash: string,
    strategy: ConflictResolutionStrategy
  ): Promise<number> {
    const key = `q:coordinator:${groupStateHash}:${strategy}`;
    return await this.agentDB.get(key) ?? 0;
  }

  async updateCoordinatorQValue(
    groupStateHash: string,
    strategy: ConflictResolutionStrategy,
    reward: number,
    nextStateHash: string
  ): Promise<void> {
    const currentQ = await this.getCoordinatorQValue(groupStateHash, strategy);

    // Get max Q for next state
    const strategies: ConflictResolutionStrategy[] = [
      'weighted-voting',
      'round-robin',
      'highest-satisfaction',
      'compromise-search',
      'veto-elimination'
    ];

    const nextQValues = await Promise.all(
      strategies.map(s => this.getCoordinatorQValue(nextStateHash, s))
    );

    const maxNextQ = Math.max(...nextQValues);

    // Q-learning update
    const learningRate = 0.1;
    const discountFactor = 0.95;
    const newQ = currentQ + learningRate * (reward + discountFactor * maxNextQ - currentQ);

    await this.agentDB.set(`q:coordinator:${groupStateHash}:${strategy}`, newQ);
  }

  // Preference Agent Q-table: memberId → state → content → Q-value
  async getPreferenceQValue(
    memberId: string,
    stateHash: string,
    contentId: string
  ): Promise<number> {
    const key = `q:preference:${memberId}:${stateHash}:${contentId}`;
    return await this.agentDB.get(key) ?? 0;
  }

  async updatePreferenceQValue(
    memberId: string,
    stateHash: string,
    contentId: string,
    individualSatisfaction: number
  ): Promise<void> {
    const currentQ = await this.getPreferenceQValue(memberId, stateHash, contentId);

    // Simple update: move toward satisfaction
    const learningRate = 0.1;
    const newQ = currentQ + learningRate * (individualSatisfaction - currentQ);

    await this.agentDB.set(`q:preference:${memberId}:${stateHash}:${contentId}`, newQ);
  }
}
```

---

## 10. Agentic Flow Integration

### 10.1 Multi-Agent Orchestrator

```typescript
class GroupOrchestrator {
  private preferenceAgents: Map<string, PreferenceAgent> = new Map();
  private consensusCoordinator: ConsensusCoordinator;
  private safetyGuardian: SafetyGuardian;

  constructor(
    private agentDB: AgentDB,
    private ruVector: RuVectorClient,
    private reasoningBank: ReasoningBankClient
  ) {
    this.consensusCoordinator = new ConsensusCoordinator(agentDB, reasoningBank);
    this.safetyGuardian = new SafetyGuardian(agentDB, ruVector);
  }

  async processGroupDiscovery(
    groupId: string,
    query: string,
    context: GroupContextInput
  ): Promise<GroupDiscoveryResult> {
    // 1. Get group
    const group = await this.agentDB.get<GroupProfile>(`group:${groupId}`);
    if (!group) throw new Error('Group not found');

    // 2. Initialize preference agents for each member
    for (const member of group.members) {
      if (!this.preferenceAgents.has(member.memberId)) {
        this.preferenceAgents.set(
          member.memberId,
          new PreferenceAgent(member.memberId, this.agentDB, this.ruVector)
        );
      }
    }

    // 3. Search for candidates
    const weights = await this.consensusCoordinator.getLearnedWeights(
      groupId,
      this.buildSocialContext(context)
    );

    const candidates = await searchForGroup(
      groupId,
      query,
      group.members.map(m => m.memberId),
      weights
    );

    // 4. Safety filtering
    const safeCandidates = await this.safetyGuardian.filterContent(candidates, groupId);

    // 5. Get individual votes
    const votes = new Map<string, Map<string, number>>();

    for (const member of group.members) {
      const agent = this.preferenceAgents.get(member.memberId)!;
      const memberVotes = new Map<string, number>();

      for (const candidate of safeCandidates) {
        const vote = await agent.voteOnContent(
          candidate.contentId,
          this.buildSocialContext(context)
        );
        memberVotes.set(candidate.contentId, vote.score);
      }

      votes.set(member.memberId, memberVotes);
    }

    // 6. Determine consensus
    const consensus = await this.consensusCoordinator.determineConsensus(
      groupId,
      safeCandidates,
      votes,
      this.buildSocialContext(context)
    );

    // 7. Create voting session
    const sessionId = await this.createVotingSession(groupId, safeCandidates, context);

    return {
      candidates: safeCandidates,
      votingSessionId: sessionId,
      safetyFiltered: candidates.length - safeCandidates.length,
      predictedConsensus: safeCandidates.find(c => c.contentId === consensus.selectedContentId),
      predictedConflicts: consensus.conflicts.map(this.toPrediction),
      recommendedStrategy: consensus.strategyUsed
    };
  }

  async trackGroupOutcome(sessionId: string, outcome: GroupViewingOutcome): Promise<void> {
    // Get session
    const session = await this.agentDB.get<GroupSession>(`session:${sessionId}`);
    if (!session) return;

    // Calculate collective satisfaction
    const collectiveSatisfaction = outcome.collectiveSatisfaction;

    // Update consensus coordinator
    await this.consensusCoordinator.learnFromOutcome(
      session.groupId,
      sessionId,
      collectiveSatisfaction,
      outcome.individualSatisfaction,
      session.context
    );

    // Update individual preference agents
    for (const [memberId, satisfaction] of outcome.individualSatisfaction.entries()) {
      const agent = this.preferenceAgents.get(memberId);
      if (!agent) continue;

      await agent.updateFromOutcome(
        session.finalSelection,
        satisfaction,
        session.context
      );
    }

    // Track in ReasoningBank
    await this.reasoningBank.addTrajectory({
      groupId: session.groupId,
      sessionId,
      strategyUsed: session.strategyUsed,
      reward: collectiveSatisfaction,
      individualRewards: Array.from(outcome.individualSatisfaction.entries()),
      timestamp: Date.now()
    });
  }
}
```

---

## 11. Learning Metrics & KPIs

### 11.1 Group Learning Metrics

```typescript
interface GroupLearningMetrics {
  // Collective performance
  avgCollectiveSatisfaction: number; // 0-1
  collectiveSatisfactionTrend: number; // improvement rate

  // Individual fairness
  satisfactionVariance: number; // how equal satisfaction is
  fairnessScore: number; // Gini coefficient (0=perfectly fair, 1=unfair)

  // Decision efficiency
  avgDecisionTime: number; // seconds
  decisionTimeImprovement: number; // % improvement

  // Strategy learning
  strategyConvergence: number; // how stable strategy selection is
  strategyDiversity: number; // how many different strategies tried

  // Conflict resolution
  conflictRate: number; // % sessions with conflicts
  conflictResolutionSuccess: number; // % conflicts resolved successfully

  // Weight learning
  weightStability: Map<string, number>; // per member
  weightFairness: number; // how equal weights are
}
```

---

## 12. MVP Scope (Week 1)

### 12.1 Core Features

**Must Have:**
1. ✅ Create family/couple/friend groups (3 member max for MVP)
2. ✅ Basic preference agents (one per member)
3. ✅ Simple weighted voting (equal weights initially)
4. ✅ Safety filtering (age-based)
5. ✅ Viewing outcome tracking
6. ✅ Basic weight learning from outcomes

**Simplified Learning:**
- Equal weights (no context-specific learning yet)
- Simple weighted average voting (no conflict resolution)
- Individual preference vectors update on feedback
- Group satisfaction = average of individual satisfaction

---

## 13. Enhanced Scope (Week 2)

### 13.1 Advanced Features

**Add:**
1. ✅ Context-aware voting weights
2. ✅ Conflict detection & resolution strategies
3. ✅ Round-robin fairness
4. ✅ Veto system with learning
5. ✅ ReasoningBank trajectory analysis
6. ✅ Meta-learning across groups

---

## 14. Success Criteria

### 14.1 MVP Success (Week 1)

- ✅ 30 beta groups
- ✅ 200 group sessions
- ✅ 65% avg collective satisfaction
- ✅ <10min avg decision time
- ✅ Weights learning from outcomes

### 14.2 Production Success (Week 2)

- ✅ 500 active groups
- ✅ 3,000 group sessions
- ✅ 75% avg collective satisfaction
- ✅ <6min avg decision time
- ✅ 80% conflict resolution success rate
- ✅ Fairness score >0.7

---

## 15. Risk Mitigation

**Risk: Groups don't provide individual feedback**
- Mitigation: Use completion rate as proxy
- Fallback: Explicit "thumbs up/down" per member

**Risk: Weights diverge (one person dominates)**
- Mitigation: Cap weights at 2.0, floor at 0.5
- Fallback: Periodic weight reset to 1.0

**Risk: Kids game the system**
- Mitigation: Veto budgets, parent override controls
- Fallback: Manual weight adjustments by parents

---

**End of WatchSphere Collective PRD**
