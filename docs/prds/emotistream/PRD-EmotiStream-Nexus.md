# Product Requirements Document: EmotiStream Nexus

**Version**: 2.0 (Validated)
**Last Updated**: 2025-12-05
**Validation Status**: ✅ Requirements Validated via QE Agent

---

## 1. Executive Summary

**Problem**: Current recommendation systems optimize for engagement (watch time), not user wellbeing. Users consume content that keeps them watching but leaves them feeling worse. 67% of users report "binge regret", 43% use entertainment as emotional escape that backfires, and $12B annually is spent on content that negatively impacts mental health. Recommendations are content-centric, not outcome-centric.

**Solution**: EmotiStream Nexus is an emotion-driven recommendation system using multimodal AI (voice, text, biometric) to predict desired emotional outcomes and learn which content actually delivers psychological benefit. Powered by Gemini's emotion analysis, reinforcement learning optimizes for emotional state improvement, and ReasoningBank tracks long-term wellbeing trajectories.

**Impact Targets** (Validated & Measurable):
| Metric | Baseline | Target | Measurement Method |
|--------|----------|--------|-------------------|
| Binge regret | 67% (industry survey) | <30% | Post-30-day survey |
| Emotional improvement | 30% (random baseline) | 70% mean reward | RL reward function |
| Prediction accuracy | 25% (random 4-quadrant) | 78% | Desired state vs actual |
| Decision time | 45 minutes | <5 minutes | Session analytics |

---

## 2. Problem Statement

### 2.1 Current State Analysis

**Emotional Wellbeing Crisis:**
- **67% "binge regret"** - feeling worse after watching
- **43% emotional escape** that backfires (numbing → worse mood)
- **$12B annual cost** of mental health-negative content consumption
- **22% stress increase** from choice overload + poor outcomes

**Emotion-Content Mismatch:**
| Starting Emotion | Common Selection | Actual Outcome | Desired Outcome |
|-----------------|------------------|----------------|-----------------|
| Stressed | "Relax with thriller" | More stressed | Calming |
| Sad | "Cheer up with drama" | More sad | Uplifting |
| Anxious | "Distract with news" | More anxious | Grounding |
| Lonely | "Binge comedy" | Still lonely | Connection |

**Market Opportunity:**
- Mental health tech market: $5.3B (2024)
- Wellness apps: 87% retention issues (users don't feel results)
- Emotional AI market: $37B by 2027
- **Untapped**: Entertainment for emotional regulation

### 2.2 Root Cause Analysis

Recommendation systems fail emotionally because:
1. **Content-centric, not outcome-centric** - optimize for clicks, not feelings
2. **No emotional state input** - can't recommend without knowing current state
3. **No outcome tracking** - don't learn if content helped
4. **No emotional intelligence** - don't understand content's emotional impact
5. **No long-term wellbeing** - optimize for session, not life satisfaction

---

## 3. Solution Overview

### 3.1 Vision

EmotiStream Nexus creates a **self-learning emotional outcome prediction system** that:
- Detects emotional state via multimodal input (voice, text, biometric)
- Predicts desired emotional state (not just content preferences)
- Recommends content optimized for emotional journey
- Tracks actual emotional outcomes (not just engagement)
- Learns which content → emotion transitions work
- Optimizes for long-term wellbeing, not just immediate gratification

### 3.2 Core Innovation: Emotion-Reinforcement Learning

```
Emotional Input (Voice/Text/Bio) → Gemini Emotion Analysis
                                 → Current State Vector (RuVector)
                                 → Historical Emotional Patterns (AgentDB)
                                 → Desired State Prediction (ML)
                                 → Content-Emotion Mapping (RuVector)
                                 → RL Policy (AgentDB Q-tables)
                                 → Content Recommendation
                                 → Viewing + Outcome Tracking
                                 → Post-Viewing Emotion Analysis
                                 → Reward Calculation
                                 → RL Update (Q-learning + Policy Gradient)
                                 → Preference Vector Update (RuVector)
                                 → Trajectory Logging (ReasoningBank)
```

**Self-Learning Architecture:**

### Reinforcement Learning Components:

**State Space (Emotional Context):**
```typescript
interface EmotionalState {
  // Primary emotions (Russell's Circumplex Model)
  valence: number;        // -1 (negative) to +1 (positive)
  arousal: number;        // -1 (calm) to +1 (excited)

  // Secondary emotions (Plutchik's Wheel)
  emotionVector: Float32Array; // 8D: joy, sadness, anger, fear, trust, disgust, surprise, anticipation

  // Context
  timestamp: number;
  dayOfWeek: number;
  hourOfDay: number;
  stressLevel: number;    // 0-1 from calendar, location, etc.
  socialContext: 'solo' | 'partner' | 'family' | 'friends';

  // Recent history
  recentEmotionalTrajectory: Array<{
    timestamp: number;
    valence: number;
    arousal: number;
  }>;

  // Desired outcome
  desiredValence: number;   // predicted or explicit
  desiredArousal: number;
}
```

**Action Space (Content Recommendations):**
```typescript
interface EmotionalContentAction {
  contentId: string;
  platform: string;

  // Emotional features (learned)
  emotionalProfile: {
    primaryEmotion: string;
    valenceDelta: number;    // expected change in valence
    arousalDelta: number;    // expected change in arousal
    emotionalIntensity: number; // 0-1
    emotionalComplexity: number; // simple vs nuanced
  };

  // Predicted outcome
  predictedPostViewing: {
    valence: number;
    arousal: number;
    emotionVector: Float32Array;
    confidence: number;
  };
}
```

**Reward Function (Emotional Improvement):**
```typescript
function calculateEmotionalReward(
  stateBefore: EmotionalState,
  stateAfter: EmotionalState,
  desired: { valence: number; arousal: number }
): number {
  // Primary: movement toward desired state
  const valenceDelta = stateAfter.valence - stateBefore.valence;
  const arousalDelta = stateAfter.arousal - stateBefore.arousal;

  const desiredValenceDelta = desired.valence - stateBefore.valence;
  const desiredArousalDelta = desired.arousal - stateBefore.arousal;

  // Cosine similarity in 2D emotion space
  const actualVector = [valenceDelta, arousalDelta];
  const desiredVector = [desiredValenceDelta, desiredArousalDelta];

  const dotProduct = actualVector[0] * desiredVector[0] + actualVector[1] * desiredVector[1];
  const magnitudeActual = Math.sqrt(actualVector[0]**2 + actualVector[1]**2);
  const magnitudeDesired = Math.sqrt(desiredVector[0]**2 + desiredVector[1]**2);

  const directionAlignment = magnitudeDesired > 0
    ? dotProduct / (magnitudeActual * magnitudeDesired)
    : 0;

  // Magnitude of improvement
  const improvement = Math.sqrt(valenceDelta**2 + arousalDelta**2);

  // Combined reward
  const reward = directionAlignment * 0.6 + improvement * 0.4;

  // Bonus for reaching desired state
  const desiredProximity = Math.sqrt(
    (stateAfter.valence - desired.valence)**2 +
    (stateAfter.arousal - desired.arousal)**2
  );

  const proximityBonus = Math.max(0, 1 - desiredProximity) * 0.2;

  return Math.max(-1, Math.min(1, reward + proximityBonus));
}
```

**Policy Learning (Deep RL):**
- Q-Learning for discrete content selection
- Policy Gradient for continuous emotion space navigation
- Actor-Critic for balancing exploration vs exploitation
- Experience Replay for sample efficiency
- Prioritized Replay for high-reward experiences

---

## 4. User Stories

### 4.1 Emotional Input & Detection

**As a stressed user**, I want to tell the system "I had a rough day" in natural language, and have it understand my emotional state.

**Acceptance Criteria:**
- Accept text input: "I'm exhausted and stressed"
- Accept voice input with tone analysis
- Optional biometric integration (heart rate from wearable)
- Gemini analyzes and extracts emotional state
- Map to valence-arousal space

**Learning Component:**
```typescript
interface EmotionalInputAnalysis {
  rawInput: {
    text?: string;
    voiceAudio?: Blob;
    biometricData?: {
      heartRate: number;
      heartRateVariability: number;
    };
  };

  // Gemini analysis
  geminiAnalysis: {
    primaryEmotion: string;
    emotionScores: Map<string, number>; // emotion → confidence
    valence: number;
    arousal: number;
    stressLevel: number;
    sentimentPolarity: number;
  };

  // Historical calibration
  userEmotionalBaseline: {
    avgValence: number;
    avgArousal: number;
    emotionalVariability: number;
  };

  // Final state
  emotionalState: EmotionalState;
}
```

---

**As a user**, I want the system to predict my desired emotional outcome (e.g., "calm and positive") without me explicitly stating it.

**Acceptance Criteria:**
- Learn patterns: when stressed → usually wants calm
- Contextual prediction: Friday evening → wants excitement
- Historical trajectory: track what user typically seeks
- Explicit override: "Actually, I want to laugh"

**Learning Component:**
```typescript
class DesiredStatePredictor {
  async predictDesiredState(
    currentState: EmotionalState,
    userId: string
  ): Promise<{ valence: number; arousal: number; confidence: number }> {
    // Get historical patterns
    const patterns = await this.agentDB.get<EmotionalPattern[]>(
      `user:${userId}:emotion-patterns`
    );

    // Find matching patterns
    const matchingPatterns = patterns.filter(p =>
      this.isStateSimilar(p.startState, currentState)
    );

    if (matchingPatterns.length > 0) {
      // Use most successful pattern
      const bestPattern = matchingPatterns.sort((a, b) => b.successRate - a.successRate)[0];

      return {
        valence: bestPattern.desiredState.valence,
        arousal: bestPattern.desiredState.arousal,
        confidence: bestPattern.successRate
      };
    }

    // Default heuristics
    if (currentState.valence < -0.3) {
      // Negative → want positive
      return { valence: 0.6, arousal: 0.3, confidence: 0.5 };
    } else if (currentState.arousal > 0.5) {
      // High arousal → want calm
      return { valence: 0.5, arousal: -0.3, confidence: 0.5 };
    }

    // Maintain state
    return {
      valence: currentState.valence,
      arousal: currentState.arousal,
      confidence: 0.3
    };
  }
}
```

---

**As a user experiencing anxiety**, I want content that grounds me, not distracts me.

**Acceptance Criteria:**
- Detect anxiety signals (high arousal, negative valence)
- Learn that distraction doesn't help (negative reward)
- Recommend grounding content (nature docs, slow dramas)
- Track anxiety reduction as reward

**Learning Component:**
- Emotion-specific policies (anxiety policy vs sadness policy)
- Learn ineffective patterns (distraction → worse anxiety)
- Discover effective transitions (anxiety → grounded → calm)

---

**As a user**, I want post-viewing emotional check-in to teach the system what works.

**Acceptance Criteria:**
- Quick "How do you feel now?" (1-5 scale + emoji)
- Optional voice check-in for deeper analysis
- Automatic biometric tracking if available
- Learn from implicit signals (watched to completion = good)

**Learning Component:**
```typescript
interface EmotionalOutcome {
  // Pre-viewing
  stateBefore: EmotionalState;

  // Viewing
  contentId: string;
  completionRate: number;
  sessionDuration: number;

  // Post-viewing
  stateAfter: EmotionalState;

  // Explicit feedback
  explicitRating?: number; // 1-5
  explicitEmoji?: string; // '😊', '😢', '😐', etc.

  // Implicit signals
  returnedImmediately: boolean; // watched again?
  recommendedToFriend: boolean;

  // Reward
  reward: number;

  timestamp: number;
}
```

---

**As a user with depression**, I want the system to detect patterns and recommend professional help resources.

**Acceptance Criteria:**
- Detect sustained negative valence (<-0.5 for 7+ days)
- Detect emotional dysregulation (high variability)
- Surface mental health resources
- Partner with crisis services

**Safety Component:**
```typescript
class WellbeingMonitor {
  async checkWellbeing(userId: string): Promise<WellbeingAlert | null> {
    // Get last 7 days of emotional states
    const recentStates = await this.getRecentEmotionalHistory(userId, 7 * 24 * 60 * 60 * 1000);

    // Calculate metrics
    const avgValence = recentStates.reduce((sum, s) => sum + s.valence, 0) / recentStates.length;
    const variability = this.calculateStandardDeviation(recentStates.map(s => s.valence));

    // Detection thresholds
    const DEPRESSION_THRESHOLD = -0.5;
    const HIGH_VARIABILITY = 0.7;

    if (avgValence < DEPRESSION_THRESHOLD) {
      return {
        type: 'sustained-negative-mood',
        severity: 'high',
        message: 'We noticed you\'ve been feeling down. Would you like resources?',
        resources: [
          { type: 'crisis-line', name: '988 Suicide & Crisis Lifeline', url: 'tel:988' },
          { type: 'therapy', name: 'Find a therapist', url: 'https://...' }
        ]
      };
    }

    if (variability > HIGH_VARIABILITY) {
      return {
        type: 'emotional-dysregulation',
        severity: 'medium',
        message: 'Your emotions have been fluctuating. Self-care resources?',
        resources: [
          { type: 'mindfulness', name: 'Guided meditation', url: '...' },
          { type: 'journaling', name: 'Mood tracking', url: '...' }
        ]
      };
    }

    return null;
  }
}
```

---

**As a user**, I want to see my emotional journey over time and understand patterns.

**Acceptance Criteria:**
- Visualize valence-arousal trajectory over weeks
- Identify content that consistently improves mood
- Discover emotional triggers (time of day, day of week)
- Export data for personal reflection or therapy

**Learning Insights:**
```typescript
interface EmotionalInsights {
  // Trajectory
  emotionalJourney: Array<{
    date: string;
    avgValence: number;
    avgArousal: number;
    topContent: string[];
  }>;

  // Content effectiveness
  mostEffectiveContent: Array<{
    contentId: string;
    title: string;
    avgEmotionalImprovement: number;
    timesWatched: number;
    emotionTransition: string; // "stressed → calm"
  }>;

  // Patterns
  identifiedPatterns: Array<{
    pattern: string; // "Sunday evenings: sad → uplifted with comedy"
    frequency: number;
    successRate: number;
  }>;

  // Wellbeing score
  overallWellbeingTrend: number; // -1 to +1 (improving vs declining)
  avgMoodImprovement: number; // avg reward per session
}
```

---

## 5. Technical Architecture

### 5.1 System Architecture (ASCII Diagram)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         EmotiStream Nexus Platform                           │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────────┐         ┌──────────────────────────────────────────────┐
│   User Device    │────────▶│         API Gateway (GraphQL)                │
│  (Mobile/Web)    │         │  - Authentication                            │
│  + Wearables     │         │  - Voice upload                              │
│                  │         │  - Biometric sync                            │
└──────────────────┘         └──────────────────────────────────────────────┘
                                              │
                       ┌──────────────────────┼────────────────────────┐
                       ▼                      ▼                        ▼
            ┌────────────────────┐ ┌────────────────────┐ ┌────────────────────┐
            │  Emotion Engine    │ │ RL Recommendation  │ │  Outcome Tracker   │
            │  (Gemini Multi.)   │ │     Engine         │ │  (Wellbeing Mon.)  │
            │                    │ │                    │ │                    │
            │ • Voice analysis   │ │ • Q-learning       │ │ • Post-view check  │
            │ • Text sentiment   │ │ • Policy gradient  │ │ • Emotion analysis │
            │ • Biometric fusion │ │ • Actor-Critic     │ │ • Crisis detection │
            │ • State mapping    │ │ • Experience replay│ │ • Trajectory log   │
            └────────────────────┘ └────────────────────┘ └────────────────────┘
                       │                      │                        │
                       └──────────────────────┼────────────────────────┘
                                              ▼
                       ┌──────────────────────────────────────────────────┐
                       │         RuVector Emotional Semantic Store        │
                       │                                                  │
                       │  • Content emotion embeddings (1536D)           │
                       │  • User emotional preference vectors            │
                       │  • Emotion transition vectors                   │
                       │  • Desired state embeddings                     │
                       │  • HNSW indexing (150x faster)                  │
                       └──────────────────────────────────────────────────┘
                                              │
                       ┌──────────────────────┼────────────────────────┐
                       ▼                      ▼                        ▼
            ┌────────────────────┐ ┌────────────────────┐ ┌────────────────────┐
            │      AgentDB       │ │  ReasoningBank     │ │  External APIs     │
            │                    │ │  (Agentic Flow)    │ │                    │
            │ • User profiles    │ │ • Trajectories     │ │ • Gemini API       │
            │ • Emotion history  │ │ • Verdicts         │ │ • Platforms        │
            │ • Q-tables         │ │ • Pattern lib      │ │ • Wearables        │
            │ • Policy params    │ │ • Wellbeing trends │ │ • Crisis services  │
            │ • Replay buffer    │ │ • Meta-learning    │ │                    │
            └────────────────────┘ └────────────────────┘ └────────────────────┘
```

### 5.2 Reinforcement Learning Architecture (Deep Dive)

#### 5.2.1 Multi-Modal Emotion Detection

```typescript
import { GoogleGenerativeAI } from '@google/generative-ai';

const genAI = new GoogleGenerativeAI(process.env.GEMINI_API_KEY);

class EmotionDetector {
  private model = genAI.getGenerativeModel({ model: 'gemini-2.0-flash-exp' });

  async analyzeEmotionalState(input: {
    text?: string;
    voiceAudio?: Blob;
    biometric?: BiometricData;
  }): Promise<EmotionalState> {
    let geminiAnalysis: GeminiEmotionResult;

    if (input.voiceAudio) {
      // Voice + tone analysis
      geminiAnalysis = await this.analyzeVoice(input.voiceAudio);
    } else if (input.text) {
      // Text sentiment
      geminiAnalysis = await this.analyzeText(input.text);
    } else {
      throw new Error('No input provided');
    }

    // Fuse with biometric if available
    if (input.biometric) {
      geminiAnalysis = this.fuseBiometric(geminiAnalysis, input.biometric);
    }

    // Map to valence-arousal space
    return this.mapToEmotionalState(geminiAnalysis);
  }

  private async analyzeVoice(audio: Blob): Promise<GeminiEmotionResult> {
    const prompt = `
Analyze the emotional state from this voice recording.

Provide:
1. Primary emotion (joy, sadness, anger, fear, trust, disgust, surprise, anticipation)
2. Valence: -1 (very negative) to +1 (very positive)
3. Arousal: -1 (very calm) to +1 (very excited)
4. Stress level: 0 (relaxed) to 1 (extremely stressed)
5. Confidence: 0 to 1

Format as JSON:
{
  "primaryEmotion": "...",
  "valence": 0.0,
  "arousal": 0.0,
  "stressLevel": 0.0,
  "confidence": 0.0,
  "reasoning": "..."
}
    `.trim();

    const audioBase64 = await this.blobToBase64(audio);

    const result = await this.model.generateContent([
      prompt,
      {
        inlineData: {
          mimeType: 'audio/wav',
          data: audioBase64
        }
      }
    ]);

    const response = result.response.text();
    return JSON.parse(this.extractJSON(response));
  }

  private async analyzeText(text: string): Promise<GeminiEmotionResult> {
    const prompt = `
Analyze the emotional state from this text: "${text}"

Provide:
1. Primary emotion (joy, sadness, anger, fear, trust, disgust, surprise, anticipation)
2. Valence: -1 (very negative) to +1 (very positive)
3. Arousal: -1 (very calm) to +1 (very excited)
4. Stress level: 0 (relaxed) to 1 (extremely stressed)
5. Confidence: 0 to 1

Format as JSON:
{
  "primaryEmotion": "...",
  "valence": 0.0,
  "arousal": 0.0,
  "stressLevel": 0.0,
  "confidence": 0.0,
  "reasoning": "..."
}
    `.trim();

    const result = await this.model.generateContent(prompt);
    const response = result.response.text();
    return JSON.parse(this.extractJSON(response));
  }

  private fuseBiometric(
    geminiAnalysis: GeminiEmotionResult,
    biometric: BiometricData
  ): GeminiEmotionResult {
    // Heart rate variability indicates stress
    const hrvStress = biometric.heartRateVariability < 50 ? 0.8 : 0.2;

    // Fuse stress levels
    const fusedStressLevel = (geminiAnalysis.stressLevel * 0.7) + (hrvStress * 0.3);

    // Heart rate indicates arousal
    const hrArousal = (biometric.heartRate - 70) / 50; // normalize around resting HR

    // Fuse arousal
    const fusedArousal = (geminiAnalysis.arousal * 0.7) + (hrArousal * 0.3);

    return {
      ...geminiAnalysis,
      stressLevel: fusedStressLevel,
      arousal: Math.max(-1, Math.min(1, fusedArousal)),
      confidence: geminiAnalysis.confidence * 0.9 // higher confidence with biometric
    };
  }

  private mapToEmotionalState(analysis: GeminiEmotionResult): EmotionalState {
    // Convert primary emotion to 8D emotion vector (Plutchik)
    const emotionVector = this.emotionToVector(analysis.primaryEmotion);

    return {
      valence: analysis.valence,
      arousal: analysis.arousal,
      emotionVector,
      timestamp: Date.now(),
      dayOfWeek: new Date().getDay(),
      hourOfDay: new Date().getHours(),
      stressLevel: analysis.stressLevel,
      socialContext: 'solo', // detect from context
      recentEmotionalTrajectory: [], // populate from history
      desiredValence: 0, // predict next
      desiredArousal: 0
    };
  }

  private emotionToVector(emotion: string): Float32Array {
    const emotions = ['joy', 'sadness', 'anger', 'fear', 'trust', 'disgust', 'surprise', 'anticipation'];
    const vector = new Float32Array(8);

    const index = emotions.indexOf(emotion.toLowerCase());
    if (index >= 0) {
      vector[index] = 1.0;
    }

    return vector;
  }
}
```

#### 5.2.2 Content Emotional Profiling

```typescript
class ContentEmotionalProfiler {
  async profileContent(content: ContentMetadata): Promise<EmotionalContentProfile> {
    // Use Gemini to analyze emotional impact
    const prompt = `
Analyze the emotional impact of this content:

Title: ${content.title}
Description: ${content.description}
Genres: ${content.genres.join(', ')}

Provide:
1. Primary emotional tone (joy, sadness, anger, fear, etc.)
2. Valence delta: expected change in viewer's valence (-1 to +1)
3. Arousal delta: expected change in viewer's arousal (-1 to +1)
4. Emotional intensity: 0 (subtle) to 1 (intense)
5. Emotional complexity: 0 (simple) to 1 (nuanced, mixed emotions)
6. Target viewer emotions: which emotional states is this content good for?

Format as JSON:
{
  "primaryTone": "...",
  "valenceDelta": 0.0,
  "arousalDelta": 0.0,
  "intensity": 0.0,
  "complexity": 0.0,
  "targetStates": [
    {"currentValence": 0.0, "currentArousal": 0.0, "description": "..."}
  ]
}
    `.trim();

    const result = await this.model.generateContent(prompt);
    const analysis = JSON.parse(this.extractJSON(result.response.text()));

    // Create emotion embedding
    const emotionEmbedding = await this.createEmotionEmbedding(content, analysis);

    // Store in RuVector
    await this.ruVector.upsert({
      id: `content:emotion:${content.contentId}`,
      vector: emotionEmbedding,
      metadata: {
        contentId: content.contentId,
        ...analysis
      }
    });

    return {
      contentId: content.contentId,
      primaryTone: analysis.primaryTone,
      valenceDelta: analysis.valenceDelta,
      arousalDelta: analysis.arousalDelta,
      intensity: analysis.intensity,
      complexity: analysis.complexity,
      targetStates: analysis.targetStates,
      emotionEmbedding
    };
  }

  private async createEmotionEmbedding(
    content: ContentMetadata,
    analysis: any
  ): Promise<Float32Array> {
    // Create rich emotional description
    const emotionDescription = `
Emotional tone: ${analysis.primaryTone}
Effect: moves viewer from [baseline] toward ${analysis.valenceDelta > 0 ? 'positive' : 'negative'} valence, ${analysis.arousalDelta > 0 ? 'excited' : 'calm'} arousal
Intensity: ${analysis.intensity > 0.7 ? 'intense' : analysis.intensity > 0.4 ? 'moderate' : 'subtle'}
Best for: ${analysis.targetStates.map(s => s.description).join(', ')}
    `.trim();

    // Embed using ruvLLM
    return await ruvLLM.embed(emotionDescription);
  }
}
```

#### 5.2.3 RL Policy Implementation

```typescript
class EmotionalRLPolicy {
  private learningRate = 0.1;
  private discountFactor = 0.95;
  private explorationRate = 0.15;

  constructor(
    private agentDB: AgentDB,
    private ruVector: RuVectorClient,
    private reasoningBank: ReasoningBankClient
  ) {}

  async selectAction(
    userId: string,
    emotionalState: EmotionalState
  ): Promise<EmotionalContentAction> {
    // Predict desired state
    const desiredState = await this.predictDesiredState(userId, emotionalState);

    // ε-greedy exploration
    if (Math.random() < this.explorationRate) {
      return await this.explore(userId, emotionalState, desiredState);
    }

    return await this.exploit(userId, emotionalState, desiredState);
  }

  private async exploit(
    userId: string,
    currentState: EmotionalState,
    desiredState: { valence: number; arousal: number }
  ): Promise<EmotionalContentAction> {
    // Create desired state embedding
    const desiredStateVector = this.createDesiredStateVector(currentState, desiredState);

    // Search for content that produces desired emotional transition
    const candidates = await this.ruVector.search({
      vector: desiredStateVector,
      topK: 30,
      filter: {
        // Only content that moves in desired direction
        valenceDelta: desiredState.valence > currentState.valence ? { $gt: 0 } : { $lt: 0 },
        arousalDelta: desiredState.arousal > currentState.arousal ? { $gt: 0 } : { $lt: 0 }
      }
    });

    // Re-rank with Q-values
    const stateHash = this.hashEmotionalState(currentState);

    const rankedActions = await Promise.all(
      candidates.map(async (candidate) => {
        const qValue = await this.getQValue(userId, stateHash, candidate.id);

        return {
          contentId: candidate.id,
          emotionalProfile: candidate.metadata,
          predictedOutcome: this.predictOutcome(currentState, candidate.metadata),
          qValue,
          score: qValue * 0.7 + candidate.similarity * 0.3
        };
      })
    );

    rankedActions.sort((a, b) => b.score - a.score);

    return rankedActions[0];
  }

  private async explore(
    userId: string,
    currentState: EmotionalState,
    desiredState: { valence: number; arousal: number }
  ): Promise<EmotionalContentAction> {
    // UCB exploration: select actions with high uncertainty
    const desiredStateVector = this.createDesiredStateVector(currentState, desiredState);

    const candidates = await this.ruVector.search({
      vector: desiredStateVector,
      topK: 30
    });

    const stateHash = this.hashEmotionalState(currentState);
    const totalActions = await this.agentDB.get<number>(`user:${userId}:total-actions`) ?? 1;

    const explorationScores = await Promise.all(
      candidates.map(async (candidate) => {
        const visitCount = await this.agentDB.get<number>(
          `user:${userId}:visit:${candidate.id}`
        ) ?? 0;

        const qValue = await this.getQValue(userId, stateHash, candidate.id);

        // UCB formula
        const ucbBonus = Math.sqrt(2 * Math.log(totalActions) / (visitCount + 1));

        return {
          contentId: candidate.id,
          emotionalProfile: candidate.metadata,
          predictedOutcome: this.predictOutcome(currentState, candidate.metadata),
          ucbScore: qValue + ucbBonus,
          visitCount
        };
      })
    );

    explorationScores.sort((a, b) => b.ucbScore - a.ucbScore);

    return explorationScores[0];
  }

  async updatePolicy(
    userId: string,
    experience: EmotionalExperience
  ): Promise<void> {
    const { stateBefore, contentId, stateAfter, desiredState } = experience;

    // Calculate reward
    const reward = calculateEmotionalReward(stateBefore, stateAfter, desiredState);

    // Update Q-value
    const stateHash = this.hashEmotionalState(stateBefore);
    const nextStateHash = this.hashEmotionalState(stateAfter);

    const currentQ = await this.getQValue(userId, stateHash, contentId);
    const maxNextQ = await this.getMaxQValue(userId, nextStateHash);

    const newQ = currentQ + this.learningRate * (
      reward + this.discountFactor * maxNextQ - currentQ
    );

    await this.setQValue(userId, stateHash, contentId, newQ);

    // Add to experience replay
    await this.addExperience(userId, experience, reward);

    // Update visit count
    await this.agentDB.incr(`user:${userId}:visit:${contentId}`);
    await this.agentDB.incr(`user:${userId}:total-actions`);

    // Track trajectory in ReasoningBank
    await this.reasoningBank.addTrajectory({
      userId,
      experienceId: experience.experienceId,
      emotionalTransition: {
        before: { valence: stateBefore.valence, arousal: stateBefore.arousal },
        after: { valence: stateAfter.valence, arousal: stateAfter.arousal },
        desired: desiredState
      },
      contentId,
      reward,
      timestamp: experience.timestamp
    });

    // Batch update (policy gradient)
    if (await this.shouldTriggerBatchUpdate(userId)) {
      await this.batchPolicyUpdate(userId);
    }
  }

  private async batchPolicyUpdate(userId: string, batchSize: number = 32): Promise<void> {
    // Sample from replay buffer
    const experiences = await this.sampleReplayBuffer(userId, batchSize);

    // Prioritize high-reward experiences
    const prioritized = experiences.sort((a, b) => b.reward - a.reward);

    // Update Q-values with batch
    for (const exp of prioritized) {
      const stateHash = this.hashEmotionalState(exp.stateBefore);
      const currentQ = await this.getQValue(userId, stateHash, exp.contentId);

      // TD-learning update
      const target = exp.reward + this.discountFactor * await this.getMaxQValue(
        userId,
        this.hashEmotionalState(exp.stateAfter)
      );

      const newQ = currentQ + this.learningRate * (target - currentQ);
      await this.setQValue(userId, stateHash, exp.contentId, newQ);
    }
  }

  private createDesiredStateVector(
    current: EmotionalState,
    desired: { valence: number; arousal: number }
  ): Float32Array {
    // Create embedding that represents desired emotional transition
    const transition = new Float32Array(1536);

    // Encode current state (first 768 dimensions)
    transition[0] = current.valence;
    transition[1] = current.arousal;
    transition.set(current.emotionVector, 2);

    // Encode desired state (next 768 dimensions)
    transition[768] = desired.valence;
    transition[769] = desired.arousal;

    // Encode delta (what we want to achieve)
    transition[770] = desired.valence - current.valence;
    transition[771] = desired.arousal - current.arousal;

    return transition;
  }

  private hashEmotionalState(state: EmotionalState): string {
    // Discretize continuous state space for Q-table lookup
    const valenceBucket = Math.floor((state.valence + 1) / 0.4); // 5 buckets
    const arousalBucket = Math.floor((state.arousal + 1) / 0.4); // 5 buckets
    const stressBucket = Math.floor(state.stressLevel / 0.33); // 3 buckets

    return `${valenceBucket}:${arousalBucket}:${stressBucket}:${state.socialContext}`;
  }

  private async getQValue(userId: string, stateHash: string, contentId: string): Promise<number> {
    const key = `q:emotion:${userId}:${stateHash}:${contentId}`;
    return await this.agentDB.get(key) ?? 0;
  }

  private async setQValue(
    userId: string,
    stateHash: string,
    contentId: string,
    value: number
  ): Promise<void> {
    const key = `q:emotion:${userId}:${stateHash}:${contentId}`;
    await this.agentDB.set(key, value);
  }

  private async getMaxQValue(userId: string, stateHash: string): Promise<number> {
    // Get all Q-values for this state
    const pattern = `q:emotion:${userId}:${stateHash}:*`;
    const keys = await this.agentDB.keys(pattern);

    if (keys.length === 0) return 0;

    const qValues = await Promise.all(
      keys.map(key => this.agentDB.get<number>(key))
    );

    return Math.max(...qValues.filter(v => v !== null) as number[]);
  }
}
```

### 5.3 Performance Requirements

**Latency (p95)**:
| Operation | MVP Target | Production Target |
|-----------|------------|-------------------|
| Emotion detection (text) | <2s | <1.5s |
| Emotion detection (voice) | <5s | <3s |
| Content recommendations | <3s | <2s |
| GraphQL API response | <1s | <500ms |
| RuVector semantic search | <500ms | <200ms |
| RL policy update | <100ms | <50ms |

**Throughput**:
| Metric | MVP | Production | 6-Month |
|--------|-----|------------|---------|
| Concurrent users | 100 | 1,000 | 10,000 |
| Emotion analyses/sec | 10 | 100 | 500 |
| RL policy updates/sec | 5 | 50 | 200 |

**Scale**:
| Resource | MVP | Production | 6-Month |
|----------|-----|------------|---------|
| Total users | 50 | 500 | 50,000 |
| Content catalog | 1,000 | 10,000 | 100,000 |
| Emotional experiences/month | 3,000 | 30,000 | 300,000 |
| RuVector embeddings | 10K | 100K | 1M |

### 5.4 Error Handling & Fallback Specifications

#### 5.4.1 Gemini API Error Handling

```typescript
interface GeminiErrorHandling {
  timeout: {
    threshold: 30000; // 30 seconds
    action: 'return_fallback';
    fallback: {
      emotionalState: { valence: 0, arousal: 0, confidence: 0.3 };
      message: 'Emotion detection temporarily unavailable, please try again';
    };
    logging: 'ERROR: gemini_timeout';
    retry: { enabled: false }; // Don't block user
  };

  rateLimit: {
    threshold: 429; // HTTP status
    action: 'queue_and_retry';
    retryDelay: 60000; // 60 seconds
    maxRetries: 3;
    userMessage: 'Processing... please wait';
  };

  invalidResponse: {
    action: 'log_and_fallback';
    fallback: {
      emotionalState: { valence: 0, arousal: 0, confidence: 0.2 };
      message: 'Could not analyze emotion, please rephrase';
    };
    logging: 'ERROR: gemini_invalid_json';
  };
}
```

#### 5.4.2 RL Policy Error Handling

```typescript
interface RLPolicyErrorHandling {
  noQValuesForState: {
    trigger: 'state_hash_not_in_q_table';
    action: 'content_based_fallback';
    fallback: 'Use RuVector semantic search for emotional transition';
    explorationRate: 0.5; // High exploration for unknown state
  };

  consecutiveNegativeRewards: {
    trigger: 'reward < 0 for 5 consecutive experiences';
    action: 'increase_exploration';
    newExplorationRate: 0.5; // Reset to 50%
    notification: 'User policy not converging, increasing exploration';
  };

  userProfileNotFound: {
    trigger: 'user_id not in database';
    action: 'population_recommendations';
    fallback: 'Use top 20 most effective content across all users';
  };

  policyDivergence: {
    trigger: 'Q-value variance > 1.0';
    action: 'reset_policy';
    fallback: 'Clear Q-table, restart with content-based filtering';
    notification: 'Policy reset due to instability';
  };
}
```

#### 5.4.3 Content API Error Handling

```typescript
interface ContentAPIErrorHandling {
  platformUnavailable: {
    trigger: 'platform_api_error';
    action: 'filter_recommendations';
    behavior: 'Only recommend content from available platforms';
    userMessage: 'Some platforms unavailable, showing available content';
  };

  metadataMissing: {
    trigger: 'title or description is null';
    action: 'skip_or_fallback';
    behavior: 'Skip content OR use title-only profiling with lower confidence';
    minMetadata: ['title', 'platform'];
  };

  embeddingGenerationFailed: {
    trigger: 'ruvector_embed_error';
    action: 'text_similarity_fallback';
    behavior: 'Use TF-IDF similarity instead of semantic embeddings';
    confidencePenalty: 0.2; // Lower confidence for fallback search
  };
}
```

---

## 6. Data Models

### 6.1 Core Entities

```typescript
// Emotional State (core RL state)
interface EmotionalState {
  // Russell's Circumplex
  valence: number;        // -1 to +1
  arousal: number;        // -1 to +1

  // Plutchik's emotions
  emotionVector: Float32Array; // 8D

  // Context
  timestamp: number;
  dayOfWeek: number;
  hourOfDay: number;
  stressLevel: number;    // 0-1
  socialContext: 'solo' | 'partner' | 'family' | 'friends';

  // History
  recentEmotionalTrajectory: Array<{
    timestamp: number;
    valence: number;
    arousal: number;
  }>;

  // Desired outcome
  desiredValence: number;
  desiredArousal: number;
}

// Emotional Experience (RL experience for replay buffer)
interface EmotionalExperience {
  experienceId: string;
  userId: string;

  // State before viewing
  stateBefore: EmotionalState;

  // Action (content selection)
  contentId: string;
  contentEmotionalProfile: EmotionalContentProfile;

  // Viewing details
  viewingDetails: {
    startTime: number;
    endTime?: number;
    completionRate: number;
    pauseCount: number;
    skipCount: number;
  };

  // State after viewing
  stateAfter: EmotionalState;

  // Desired state (predicted or explicit)
  desiredState: {
    valence: number;
    arousal: number;
  };

  // Feedback
  explicitFeedback?: {
    rating: number; // 1-5
    emoji: string;
    textFeedback?: string;
  };

  // Reward
  reward: number;

  timestamp: number;
}

// Content Emotional Profile
interface EmotionalContentProfile {
  contentId: string;

  // Emotional characteristics
  primaryTone: string; // 'uplifting', 'melancholic', 'thrilling', etc.
  valenceDelta: number; // expected change in valence
  arousalDelta: number; // expected change in arousal

  emotionalIntensity: number; // 0-1
  emotionalComplexity: number; // 0-1 (simple vs nuanced)

  // Target states (when is this content effective?)
  targetStates: Array<{
    currentValence: number;
    currentArousal: number;
    description: string; // "stressed and anxious"
    effectiveness: number; // 0-1 (learned)
  }>;

  // Embedding
  emotionEmbedding: Float32Array; // 1536D

  // Learned effectiveness
  avgEmotionalImprovement: number;
  sampleSize: number;
}

// User Emotional Profile (AgentDB)
interface UserEmotionalProfile {
  userId: string;

  // Baselines
  emotionalBaseline: {
    avgValence: number;
    avgArousal: number;
    emotionalVariability: number;
  };

  // Patterns
  emotionalPatterns: Array<{
    startState: { valence: number; arousal: number };
    desiredState: { valence: number; arousal: number };
    successfulTransitions: Array<{
      contentId: string;
      successRate: number;
      avgReward: number;
    }>;
    frequency: number;
  }>;

  // Learning state
  totalExperiences: number;
  avgReward: number;
  explorationRate: number;

  // Wellbeing metrics
  wellbeingTrend: number; // -1 to +1
  sustainedNegativeMoodDays: number;
  emotionalDysregulationScore: number;

  createdAt: number;
  lastActive: number;
}

// Wellbeing Alert
interface WellbeingAlert {
  type: 'sustained-negative-mood' | 'emotional-dysregulation' | 'crisis-detected';
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  resources: Array<{
    type: string;
    name: string;
    url: string;
  }>;
  triggered: number;
}
```

---

## 7. API Specifications

### 7.1 GraphQL Schema

```graphql
type Query {
  # Emotional state
  currentEmotionalState(userId: ID!): EmotionalState!

  # Recommendations
  emotionalDiscover(input: EmotionalDiscoverInput!): EmotionalDiscoveryResult!

  # Insights
  emotionalJourney(userId: ID!, timeRange: TimeRange!): EmotionalJourney!

  # Wellbeing
  wellbeingStatus(userId: ID!): WellbeingStatus!
}

type Mutation {
  # Input emotional state
  submitEmotionalInput(input: EmotionalInputSubmission!): EmotionalState!

  # Track outcome
  trackEmotionalOutcome(input: EmotionalOutcomeInput!): OutcomeResult!

  # Explicit desired state
  setDesiredState(userId: ID!, valence: Float!, arousal: Float!): EmotionalState!
}

input EmotionalInputSubmission {
  userId: ID!
  text: String
  voiceAudio: Upload
  biometricData: BiometricInput
}

input BiometricInput {
  heartRate: Float
  heartRateVariability: Float
  # Future: EEG, skin conductance, etc.
}

type EmotionalState {
  valence: Float!        # -1 to +1
  arousal: Float!        # -1 to +1
  primaryEmotion: String!
  stressLevel: Float!    # 0-1
  confidence: Float!

  # Predicted desired state
  predictedDesiredState: DesiredState!

  timestamp: DateTime!
}

type DesiredState {
  valence: Float!
  arousal: Float!
  confidence: Float!
  reasoning: String!
}

input EmotionalDiscoverInput {
  userId: ID!
  emotionalStateId: ID! # from submitEmotionalInput
  explicitDesiredState: DesiredStateInput
  limit: Int = 20
}

input DesiredStateInput {
  valence: Float!
  arousal: Float!
}

type EmotionalDiscoveryResult {
  recommendations: [EmotionalRecommendation!]!
  learningMetrics: EmotionalLearningMetrics!
}

type EmotionalRecommendation {
  contentId: ID!
  title: String!
  platform: String!

  # Emotional prediction
  emotionalProfile: EmotionalContentProfile!
  predictedOutcome: PredictedEmotionalOutcome!

  # RL metadata
  qValue: Float!
  confidence: Float!
  explorationFlag: Boolean!

  # Explanation
  reasoning: String!
}

type EmotionalContentProfile {
  primaryTone: String!
  valenceDelta: Float!
  arousalDelta: Float!
  intensity: Float!
  complexity: Float!
}

type PredictedEmotionalOutcome {
  postViewingValence: Float!
  postViewingArousal: Float!
  expectedImprovement: Float!
  confidence: Float!
}

input EmotionalOutcomeInput {
  userId: ID!
  experienceId: ID!
  postViewingEmotionalState: EmotionalStateInput!
  explicitFeedback: ExplicitFeedbackInput
}

input EmotionalStateInput {
  text: String
  voiceAudio: Upload
  biometricData: BiometricInput
}

input ExplicitFeedbackInput {
  rating: Int! # 1-5
  emoji: String
  textFeedback: String
}

type OutcomeResult {
  success: Boolean!
  reward: Float!
  qValueUpdated: Boolean!
  emotionalImprovement: Float!
}

type EmotionalJourney {
  timeRange: TimeRange!

  emotionalTrajectory: [EmotionalDataPoint!]!

  mostEffectiveContent: [ContentEffectiveness!]!

  identifiedPatterns: [EmotionalPattern!]!

  wellbeingScore: Float! # -1 to +1
  avgMoodImprovement: Float!
}

type EmotionalDataPoint {
  timestamp: DateTime!
  valence: Float!
  arousal: Float!
  primaryEmotion: String!
}

type ContentEffectiveness {
  contentId: ID!
  title: String!
  avgEmotionalImprovement: Float!
  timesWatched: Int!
  emotionTransition: String! # "stressed → calm"
}

type EmotionalPattern {
  pattern: String! # "Sunday evenings: sad → uplifted"
  frequency: Int!
  successRate: Float!
  avgReward: Float!
}

type WellbeingStatus {
  overallTrend: Float! # -1 to +1
  recentMoodAvg: Float!
  emotionalVariability: Float!

  alerts: [WellbeingAlert!]!
  recommendations: [WellbeingRecommendation!]!
}

type WellbeingAlert {
  type: WellbeingAlertType!
  severity: Severity!
  message: String!
  resources: [Resource!]!
}

enum WellbeingAlertType {
  SUSTAINED_NEGATIVE_MOOD
  EMOTIONAL_DYSREGULATION
  CRISIS_DETECTED
}

type WellbeingRecommendation {
  type: String!
  message: String!
  actionUrl: String
}

type EmotionalLearningMetrics {
  totalExperiences: Int!
  avgReward: Float!
  explorationRate: Float!
  policyConvergence: Float!
  predictionAccuracy: Float!
}
```

---

## 8. RuVector Integration Patterns

### 8.1 Emotion-Content Mapping

```typescript
// Emotional transition search
async function searchByEmotionalTransition(
  currentState: EmotionalState,
  desiredState: { valence: number; arousal: number }
): Promise<EmotionalContentAction[]> {
  // Create transition vector
  const transitionVector = new Float32Array(1536);

  // Encode current state
  transitionVector[0] = currentState.valence;
  transitionVector[1] = currentState.arousal;
  transitionVector.set(currentState.emotionVector, 2);

  // Encode desired transition
  transitionVector[768] = desiredState.valence - currentState.valence;
  transitionVector[769] = desiredState.arousal - currentState.arousal;

  // Search for content that produces this transition
  const results = await contentEmotionVectors.search({
    vector: transitionVector,
    topK: 30,
    includeMetadata: true
  });

  return results.map(r => ({
    contentId: r.id,
    emotionalProfile: r.metadata,
    relevanceScore: r.similarity
  }));
}
```

---

## 9. Success Criteria (SMART)

### 9.1 Emotion Detection Accuracy

| Metric | Baseline | Target | Measurement | Threshold |
|--------|----------|--------|-------------|-----------|
| Text emotion classification | N/A | ≥70% | IEMOCAP 1000-sample test set | 8-class Plutchik emotions |
| Voice tone analysis | N/A | ≥65% | IEMOCAP voice samples | Audio + transcript fusion |
| Multimodal fusion accuracy | Text-only | ≥75% | Text + biometric fusion | +5% vs text-only |
| Valence detection | 50% (binary) | ≥80% | Positive/negative classification | Binary accuracy |
| Confusion tolerance | N/A | <15% | Per emotion-pair misclassification | Joy↔Sadness: <5% |

### 9.2 RL Policy Convergence

| Metric | Definition | Target | Measurement Method |
|--------|------------|--------|-------------------|
| Q-value stability | Variance over 100 consecutive updates | <0.05 | `var(Q_updates[-100:])` |
| Sample efficiency | Experiences to reach 60% reward | ≤100 per user | Mean reward after N experiences |
| Mean reward (RL) | Average reward across experiences | ≥0.65 | `mean(rewards[-50:])` |
| Baseline comparison | RL vs random recommendations | +35% reward | Random baseline: ~0.30 reward |
| Policy consistency | Top 5 recommendations overlap | ≥80% | Same state queried 10 times |
| Exploration decay | ε-greedy decay curve | 0.30 → 0.10 | After 500 experiences |

### 9.3 MVP Success (Week 8)

| Metric | Target | Measurement Method | Statistical Significance |
|--------|--------|-------------------|-------------------------|
| Beta users enrolled | 50 | User registration count | N/A |
| Total experiences | 200 | Emotional experience records | N/A |
| Mean reward (RL users) | ≥0.60 | RL reward function output | p<0.05 vs random (0.30) |
| Desired state prediction | 70% accuracy | Predicted vs actual post-state | 4-quadrant baseline: 25% |
| Users with converged Q-values | ≥30 (60%) | Q-variance <0.05 | N/A |
| Post-viewing "felt better" | ≥50% | Post-viewing survey (1-5 scale) | Rating ≥4 |

### 9.4 Production Success (Week 16)

| Metric | Target | Measurement Method | Baseline Comparison |
|--------|--------|-------------------|---------------------|
| Active users | 500 | DAU over 7 days | 10x MVP growth |
| Total experiences | 2,000 | Experience records | 10x MVP |
| Mean reward (RL users) | ≥0.70 | RL reward function | +10% over MVP |
| Desired state prediction | 78% accuracy | Predicted vs actual | +8% over MVP |
| Binge regret reduction | <30% | 30-day post-launch survey | Industry: 67% |
| Post-viewing wellbeing | ≥60% positive | "Felt better" after viewing | MVP: 50% |

### 9.5 Binge Regret Measurement Protocol

```typescript
interface BingeRegretMetric {
  measurement: 'post-viewing-survey';
  timing: 'immediately after viewing AND 30-day follow-up';

  immediateQuestion: 'How do you feel after watching this content?';
  immediateScale: [
    { value: 1, label: 'Much worse than before' },
    { value: 2, label: 'Somewhat worse' },
    { value: 3, label: 'About the same' },
    { value: 4, label: 'Somewhat better' },
    { value: 5, label: 'Much better than before' }
  ];
  bingeRegretThreshold: 'rating < 3';

  followUpQuestion: 'Looking back at your viewing this month, did it improve your wellbeing?';
  followUpScale: 'same 1-5 scale';

  baseline: '67% of sessions rated <3 (industry survey)';
  target: '<30% of sessions rated <3 (55% reduction)';
  sampleSize: 'minimum 500 sessions for statistical significance (p<0.05)';
}

---

## 10. Risk Mitigation

**Risk: Emotion detection inaccuracy**
- Mitigation: Multi-modal fusion (voice + text + biometric)
- Fallback: Explicit emotion selection by user

**Risk: RL policy overfits to short-term pleasure**
- Mitigation: Long-term wellbeing reward component
- Fallback: Wellbeing monitor overrides recommendations

**Risk: Privacy concerns with emotional data**
- Mitigation: Local-first processing, encrypted storage
- Fallback: Anonymous mode with no learning

**Risk: Mental health crisis detection false positives**
- Mitigation: High thresholds, human review
- Fallback: Always provide resources, never diagnose

---

## 11. Privacy & Security Requirements

### 11.1 Data Encryption

| Data Type | At Rest | In Transit | Notes |
|-----------|---------|------------|-------|
| Emotional state data | AES-256 | TLS 1.3 | User-encrypted keys |
| User profiles | AES-256 | TLS 1.3 | Per-user encryption |
| Q-tables | AES-256 | TLS 1.3 | Deleted on account deletion |
| Voice recordings | Not stored | TLS 1.3 | Processed in-memory only |
| Biometric data | Not stored | TLS 1.3 | Never persisted to disk |
| Wellbeing alerts | AES-256 | TLS 1.3 | Encrypted, user-only access |

### 11.2 Data Retention Policy

| Data Category | Retention Period | Deletion Trigger | GDPR Basis |
|---------------|------------------|------------------|------------|
| Emotional state history | 90 days | Auto-delete after 90 days | Consent + Legitimate interest |
| Anonymous aggregates | Indefinite | Never (anonymized) | Legitimate interest |
| User Q-tables | Until account deletion | User request or deletion | Consent |
| Experience replay buffer | 30 days | Rolling window | Consent |
| Wellbeing alerts | 7 days after dismissal | User dismissal | Consent |

### 11.3 Access Control

```typescript
interface AccessControlPolicy {
  userData: {
    access: ['user_only'];
    adminAccess: false;
    supportAccess: false; // No support can view emotional data
    exportable: true; // GDPR portability
  };

  wellbeingAlerts: {
    access: ['user_only'];
    crisisServiceAccess: 'with_explicit_consent_only';
    encryption: 'AES-256';
    logging: 'access_logged_for_audit';
  };

  aggregateAnalytics: {
    access: ['system_analytics'];
    anonymization: 'k-anonymity >= 50';
    personalIdentifiers: 'never_included';
  };
}
```

### 11.4 Compliance Requirements

| Regulation | Requirement | Implementation |
|------------|-------------|----------------|
| **GDPR** | Right to access | Export all emotional data as JSON |
| **GDPR** | Right to erasure | Delete user data within 72 hours |
| **GDPR** | Data portability | Download in machine-readable format |
| **COPPA** | Age verification | Users must be 18+ (date of birth verification) |
| **HIPAA** | N/A | Not medical diagnosis system |
| **ADA** | Accessibility | WCAG 2.1 AA compliance |

---

## 12. Content Catalog Requirements

### 12.1 Content Sources (MVP)

| Platform | Initial Titles | Integration Method | Priority |
|----------|---------------|-------------------|----------|
| YouTube | 5,000 videos | YouTube Data API v3 | P0 |
| Netflix | 3,000 titles | Unofficial API (JustWatch) | P1 |
| Prime Video | 2,000 titles | JustWatch integration | P1 |
| Manual curation | 500 items | Admin panel | P0 |
| **Total MVP** | **10,500 items** | | |

### 12.2 Content Profiling Pipeline

```typescript
interface ContentProfilingPipeline {
  automated: {
    method: 'Gemini batch profiling';
    rate: '1,000 items/day';
    fields: [
      'primaryTone',
      'valenceDelta',
      'arousalDelta',
      'intensity',
      'complexity',
      'targetStates'
    ];
  };

  qualityControl: {
    method: 'Human validation';
    coverage: 'Top 100 most-recommended items';
    frequency: 'Weekly';
    validators: '2 validators per item';
    interRaterReliability: '>0.8 Cohen\'s kappa';
  };

  updateSchedule: {
    reprocessing: 'Every 30 days';
    newContent: 'Daily batch at 2am UTC';
    versionDrift: 'Alert if accuracy drops >5% on validation set';
  };

  storage: {
    metadata: 'PostgreSQL (content table)';
    embeddings: 'RuVector (1536D emotion embeddings)';
    index: 'HNSW (M=16, efConstruction=200)';
  };
}
```

### 12.3 Content Metadata Schema

```typescript
interface ContentMetadata {
  // Required fields
  contentId: string;          // UUID
  title: string;              // Content title
  platform: Platform;         // youtube | netflix | prime | manual
  duration: number;           // Duration in seconds
  genres: string[];           // Genre tags

  // Emotional profiling (generated)
  emotionalProfile: {
    primaryTone: string;
    valenceDelta: number;
    arousalDelta: number;
    intensity: number;        // 0-1
    complexity: number;       // 0-1
    targetStates: TargetState[];
    confidence: number;
    profiledAt: Date;
    geminiVersion: string;
  };

  // Vector embedding
  embeddingId: string;        // RuVector ID
  embeddingVersion: string;   // For cache invalidation

  // Licensing
  availableRegions: string[]; // ISO country codes
  ageRating: string;          // G, PG, PG-13, R
  expiresAt?: Date;           // License expiration

  createdAt: Date;
  updatedAt: Date;
}
```

---

## 13. A/B Testing Framework

### 13.1 Experiment Design

| Experiment | Baseline (Control) | Treatment | Primary Metric |
|------------|-------------------|-----------|----------------|
| **RL vs Random** | Random content recommendations | RL-optimized recommendations | Mean reward |
| **Multimodal vs Text-only** | Text emotion detection | Text + voice + biometric | Detection accuracy |
| **Exploration rates** | ε=0.15 | ε=0.30 | Long-term reward |
| **Reward function** | Direction + magnitude | Direction + magnitude + proximity bonus | Desired state accuracy |

### 13.2 Sample Size & Duration

```typescript
interface ExperimentConfig {
  rlVsRandom: {
    sampleSize: 500; // 250 control, 250 treatment
    duration: '4 weeks';
    minExperiencesPerUser: 10;
    primaryMetric: 'mean_reward';
    successThreshold: 'treatment.reward > control.reward + 0.2';
    statisticalPower: 0.8;
    significanceLevel: 0.05;
  };

  multimodalVsTextOnly: {
    sampleSize: 200; // 100 per group
    duration: '2 weeks';
    primaryMetric: 'emotion_detection_accuracy';
    successThreshold: 'multimodal.accuracy > text.accuracy + 0.05';
    validationSet: 'IEMOCAP subset (200 samples)';
  };
}
```

### 13.3 Metrics & Monitoring

| Metric | Measurement | Frequency | Alert Threshold |
|--------|-------------|-----------|-----------------|
| Mean reward (treatment) | RL reward function | Hourly | <0.50 for 24h |
| Mean reward (control) | RL reward function | Hourly | >0.60 (polluted) |
| User retention | DAU/MAU | Daily | <10% treatment |
| Completion rate | Views completed / started | Daily | <50% |
| Experiment validity | Sample ratio mismatch | Daily | >10% imbalance |

### 13.4 Guardrails

```typescript
interface ExperimentGuardrails {
  earlyStop: {
    minSampleSize: 100; // Minimum before checking
    significantHarm: 'p<0.01 AND treatment.reward < control.reward - 0.15';
    significantSuccess: 'p<0.01 AND treatment.reward > control.reward + 0.25';
  };

  rollback: {
    trigger: 'Mean reward < 0.3 for 48 hours';
    action: 'Disable RL, switch to content-based filtering';
    notification: 'Engineering on-call alert';
  };

  exclusions: {
    newUsers: 'First 3 experiences use treatment (RL for learning)';
    wellbeingAlerts: 'Users with active alerts excluded from experiments';
  };
}
```

---

## Appendix A: BDD Scenarios

### A.1 Emotion Detection Scenarios

```gherkin
Feature: Multimodal Emotion Detection
  As an EmotiStream user
  I want to input my emotional state via text, voice, or biometric
  So that the system understands my current mood accurately

  Background:
    Given the Gemini API is available and responsive
    And the user has granted necessary permissions

  Scenario: Text-based emotion detection with high confidence
    Given I enter the text "I'm feeling exhausted after a stressful day at work"
    When the system analyzes my emotional state
    Then the detected primary emotion should be "sadness" or "anger"
    And the valence should be between -0.8 and -0.4 (negative)
    And the arousal should be between -0.5 and 0.2 (low to moderate)
    And the stress level should be ≥0.6 (stressed)
    And the confidence should be ≥0.7 (high confidence)
    And the processing time should be <2 seconds

  Scenario: Fallback to text when biometric unavailable
    Given I enter the text "I need something calming"
    And no wearable data is available
    When the system analyzes my emotional state
    Then the system should use text-only analysis without error
    And the confidence should reflect text-only accuracy (≥0.7)

  Scenario: Error handling for Gemini API timeout
    Given I enter the text "I'm feeling stressed"
    And the Gemini API times out after 30 seconds
    When the system attempts emotion detection
    Then the system should return a fallback neutral emotion
    And the user should receive a message "Emotion detection temporarily unavailable"
```

### A.2 RL Policy Scenarios

```gherkin
Feature: RL Policy Convergence and Effectiveness
  As an EmotiStream system
  I want to learn which content improves each user's emotional state
  So that recommendations become more effective over time

  Background:
    Given a new user with no prior emotional history
    And a content catalog of 1,000 profiled items

  Scenario: Initial cold-start recommendations use content-based filtering
    Given the user has completed 0 emotional experiences
    When the user requests recommendations for "stressed" (valence: -0.5, arousal: 0.6)
    Then the system should use content-based filtering
    And the exploration rate should be 30% (high exploration)

  Scenario: Q-values converge after 100 experiences
    Given the user has completed 100 content viewing experiences
    When I calculate the Q-value variance over the last 100 policy updates
    Then the variance should be <0.05 (Q-values stabilized)
    And the mean reward over the last 50 experiences should be ≥0.6

  Scenario: RL policy outperforms baseline after 50 experiences
    Given the user has completed 50 experiences with RL recommendations
    And a baseline user with 50 experiences using random recommendations
    When I compare the mean reward between RL and baseline
    Then the RL policy mean reward should be ≥0.65
    And the baseline random policy mean reward should be ≤0.45
```

### A.3 Wellbeing Monitoring Scenarios

```gherkin
Feature: Wellbeing Monitoring and Crisis Detection
  As an EmotiStream system
  I want to detect sustained negative mood or crisis signals
  So that I can surface mental health resources proactively

  Background:
    Given the wellbeing monitor runs every 24 hours
    And crisis thresholds are set at (valence <-0.5 for 7+ days)

  Scenario: Detect sustained negative mood over 7 days
    Given I have logged emotional states for 7 consecutive days
    And the average valence over these 7 days is -0.6 (sustained negative)
    When the wellbeing monitor analyzes my recent history
    Then a wellbeing alert should be triggered
    And the alert type should be "sustained-negative-mood"
    And the alert severity should be "high"
    And the alert should include crisis resources

  Scenario: No alert for normal emotional fluctuation
    Given I have logged emotional states for 7 consecutive days
    And my average valence is 0.2 (slightly positive)
    And my valence variability is 0.3 (normal fluctuation)
    When the wellbeing monitor analyzes my recent history
    Then no wellbeing alert should be triggered
```

### A.4 Content Profiling Scenarios

```gherkin
Feature: Content Emotional Profiling
  As an EmotiStream system
  I want to profile the emotional impact of content at scale
  So that I can match content to desired emotional transitions

  Scenario: Profile a single content item for emotional impact
    Given a content item with title "Nature Sounds: Ocean Waves"
    And description "Relaxing ocean waves for stress relief and sleep"
    When the system profiles the content using Gemini
    Then the primary emotional tone should be "calm" or "peaceful"
    And the valence delta should be between 0.2 and 0.5 (positive)
    And the arousal delta should be between -0.6 and -0.3 (calming)

  Scenario: Search content by emotional transition
    Given I am in a "stressed" state (valence: -0.5, arousal: 0.6)
    And I want to reach a "calm" state (valence: 0.5, arousal: -0.3)
    When the system searches for content matching this transition
    Then the top 20 results should have valenceDelta ≥0.5
    And the search latency should be <3 seconds
```

---

## Appendix B: Implementation Roadmap

### Phase 0: Foundation (Weeks 1-4)
- [ ] Set up development environment
- [ ] Implement Gemini emotion detection pipeline
- [ ] Create content profiling batch processor
- [ ] Initialize RuVector with HNSW index
- [ ] Build GraphQL API skeleton

### Phase 1: MVP (Weeks 5-8)
- [ ] Implement RL policy engine with Q-learning
- [ ] Build post-viewing emotional check-in
- [ ] Create desired state predictor
- [ ] Deploy wellbeing monitor
- [ ] Recruit 50 beta users
- [ ] Collect 200 emotional experiences

### Phase 2: Optimization (Weeks 9-16)
- [ ] A/B test RL vs random recommendations
- [ ] Tune hyperparameters (learning rate, exploration)
- [ ] Scale to 500 users
- [ ] Implement multimodal fusion (voice + biometric)
- [ ] Add emotional journey visualization

### Phase 3: Scale (Weeks 17-24)
- [ ] Scale to 5,000 users
- [ ] Expand content catalog to 50,000 items
- [ ] Add more streaming platform integrations
- [ ] Implement advanced RL (actor-critic, prioritized replay)
- [ ] Launch mobile app

---

**End of EmotiStream Nexus PRD**

**Validation Status**: ✅ Requirements validated and updated per QE Agent recommendations
**Last Validated**: 2025-12-05
**Validator**: Agentic QE Requirements Validator
