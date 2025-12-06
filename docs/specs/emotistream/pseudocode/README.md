# EmotiStream Nexus MVP - SPARC Phase 2: Pseudocode

**Generated**: 2025-12-05
**SPARC Phase**: 2 - Pseudocode
**Status**: Complete - Ready for Architecture Phase

---

## Overview

This directory contains implementation-ready pseudocode for all 6 core components of the EmotiStream Nexus MVP. Each document includes:

- Data structures with type definitions
- Core algorithms with step-by-step logic
- Complexity analysis (time/space)
- Error handling patterns
- Integration notes and dependencies
- Example scenarios with worked calculations

---

## Pseudocode Documents

| Document | Component | Key Algorithms | Lines |
|----------|-----------|----------------|-------|
| [PSEUDO-EmotionDetector.md](./PSEUDO-EmotionDetector.md) | Emotion Detection | Gemini API integration, Russell's Circumplex mapping, Plutchik 8D vectors, stress calculation | ~800 |
| [PSEUDO-RLPolicyEngine.md](./PSEUDO-RLPolicyEngine.md) | RL Policy Engine | Q-learning TD updates, UCB exploration, ε-greedy decay, state hashing (5×5×3 buckets) | ~700 |
| [PSEUDO-ContentProfiler.md](./PSEUDO-ContentProfiler.md) | Content Profiler | Batch Gemini profiling, 1536D embedding generation, RuVector HNSW indexing | ~650 |
| [PSEUDO-RecommendationEngine.md](./PSEUDO-RecommendationEngine.md) | Recommendation Engine | Hybrid ranking (Q 70% + similarity 30%), transition vectors, desired state prediction | ~700 |
| [PSEUDO-FeedbackReward.md](./PSEUDO-FeedbackReward.md) | Feedback & Reward | Reward formula (direction 60% + magnitude 40%), Q-value updates, profile sync | ~800 |
| [PSEUDO-CLIDemo.md](./PSEUDO-CLIDemo.md) | CLI Demo | 7-step demo flow, Inquirer.js prompts, Chalk visualization, 3-minute script | ~750 |

---

## Component Dependencies

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CLI DEMO (Interactive UI)                     │
│                    Uses: Inquirer.js, Chalk                          │
└──────────┬─────────────────────────────────────┬────────────────────┘
           │                                     │
           ▼                                     ▼
┌─────────────────────┐              ┌─────────────────────┐
│  EMOTION DETECTOR   │              │ RECOMMENDATION ENG  │
│  Gemini → Emotion   │◀────────────▶│  Hybrid Ranking     │
└──────────┬──────────┘              └──────────┬──────────┘
           │                                     │
           ▼                                     ▼
┌─────────────────────┐              ┌─────────────────────┐
│  RL POLICY ENGINE   │◀────────────▶│  CONTENT PROFILER   │
│  Q-Learning Core    │              │  Embeddings/Vector  │
└──────────┬──────────┘              └──────────┬──────────┘
           │                                     │
           └─────────────────┬───────────────────┘
                             ▼
                  ┌─────────────────────┐
                  │  FEEDBACK/REWARD    │
                  │  Learning Signal    │
                  └─────────────────────┘
```

---

## Key Algorithms Summary

### 1. Emotion Detection
```
analyzeText(text) → EmotionalState
  1. Call Gemini API with emotion prompt
  2. Parse valence (-1 to +1) and arousal (-1 to +1)
  3. Generate Plutchik 8D vector
  4. Calculate stress = clamp((arousal + (1 - valence)) / 2, 0, 1)
  5. Return EmotionalState with confidence score
```

### 2. Q-Learning Core
```
selectAction(state, availableContent) → contentId
  1. Hash state: "v_bucket:a_bucket:s_bucket"
  2. If random() < ε: return random(availableContent)
  3. For each content: qValue + UCB_bonus
  4. Return argmax(adjusted Q-values)

updateQValue(experience)
  1. Q_old = getQValue(state, action)
  2. maxQ_next = max(Q(next_state, all_actions))
  3. Q_new = Q_old + α * (reward + γ * maxQ_next - Q_old)
  4. Store Q_new in AgentDB
```

### 3. Reward Calculation
```
calculateReward(before, after, target) → float
  1. direction = normalize(target - before)
  2. movement = after - before
  3. directionAlignment = cosine(direction, movement) * 0.6
  4. magnitude = ||movement|| / ||target - before|| * 0.4
  5. proximityBonus = 0.1 if ||after - target|| < 0.3
  6. Return clamp(directionAlignment + magnitude + proximityBonus, -1, 1)
```

### 4. Hybrid Ranking
```
rankContent(emotionalState, candidates) → rankedList
  1. For each candidate:
     - qScore = getQValue(state, candidate.contentId) * 0.70
     - simScore = cosineSimilarity(state.embedding, candidate.embedding) * 0.30
     - totalScore = qScore + simScore
  2. Sort by totalScore descending
  3. Return top K recommendations
```

---

## RL Hyperparameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| α (learning rate) | 0.1 | Q-value update step size |
| γ (discount factor) | 0.95 | Future reward weighting |
| ε (exploration) | 0.15 → 0.10 | Random action probability |
| ε decay | 0.95 | Per-episode decay rate |
| UCB constant (c) | 2.0 | Exploration bonus weight |
| State buckets | 5×5×3 | Valence × Arousal × Stress |
| Replay buffer | 1000 | Experience storage |
| Batch size | 32 | Mini-batch for updates |

---

## Data Flow Summary

```
User Input: "I'm feeling stressed"
       │
       ▼
┌──────────────────┐
│ EMOTION DETECTOR │
│ Gemini API Call  │
└────────┬─────────┘
         │ EmotionalState{valence:-0.5, arousal:0.6, stress:0.7}
         ▼
┌──────────────────┐
│  RL POLICY ENG   │
│ Q-table lookup   │──────────┐
└────────┬─────────┘          │
         │ State hash: "1:3:2" │
         ▼                     ▼
┌──────────────────┐  ┌────────────────┐
│ RECOMMENDATION   │◀─│ CONTENT PROF   │
│ Hybrid ranking   │  │ RuVector search│
└────────┬─────────┘  └────────────────┘
         │ Top 5 content IDs + scores
         ▼
┌──────────────────┐
│    CLI DEMO      │
│ Display options  │
└────────┬─────────┘
         │ User selects "Ocean Waves"
         ▼
┌──────────────────┐
│ FEEDBACK/REWARD  │
│ Calculate reward │
└────────┬─────────┘
         │ reward=0.72, Q-value update
         ▼
    Loop continues...
```

---

## State Bucket Mapping

### Valence Buckets (5)
```
Bucket 0: [-1.0, -0.6)  Very Negative
Bucket 1: [-0.6, -0.2)  Negative
Bucket 2: [-0.2, +0.2)  Neutral
Bucket 3: [+0.2, +0.6)  Positive
Bucket 4: [+0.6, +1.0]  Very Positive
```

### Arousal Buckets (5)
```
Bucket 0: [-1.0, -0.6)  Very Low
Bucket 1: [-0.6, -0.2)  Low
Bucket 2: [-0.2, +0.2)  Neutral
Bucket 3: [+0.2, +0.6)  High
Bucket 4: [+0.6, +1.0]  Very High
```

### Stress Buckets (3)
```
Bucket 0: [0.0, 0.33)   Low Stress
Bucket 1: [0.33, 0.67)  Medium Stress
Bucket 2: [0.67, 1.0]   High Stress
```

**Total State Space**: 5 × 5 × 3 = **75 unique states**

---

## Implementation Order

Based on dependencies and critical path:

1. **Hour 0-8**: EmotionDetector (Gemini setup + basic detection)
2. **Hour 8-16**: ContentProfiler (batch profiling + RuVector)
3. **Hour 16-28**: RLPolicyEngine (Q-learning core)
4. **Hour 28-40**: FeedbackReward (reward calculation + updates)
5. **Hour 40-52**: RecommendationEngine (hybrid ranking)
6. **Hour 52-65**: CLIDemo (interactive flow)
7. **Hour 65-70**: Integration + Demo rehearsal

---

## Testing Checklist

Each component should pass these tests before integration:

### EmotionDetector
- [ ] Gemini API responds within 30s
- [ ] Valence/arousal in [-1, +1] range
- [ ] Stress calculation correct
- [ ] Retry logic works on failures

### RLPolicyEngine
- [ ] State hashing produces valid bucket combinations
- [ ] Q-values persist to AgentDB
- [ ] ε-greedy selects random action ~15% of time
- [ ] Q-value updates converge

### ContentProfiler
- [ ] Batch profiles 200 items successfully
- [ ] Embeddings are 1536D Float32Array
- [ ] RuVector HNSW search returns results

### RecommendationEngine
- [ ] Returns 5 recommendations
- [ ] Q-value component is 70% of score
- [ ] Similarity component is 30% of score

### FeedbackReward
- [ ] Reward in [-1, +1] range
- [ ] Direction alignment correctly calculated
- [ ] Proximity bonus triggers at distance < 0.3

### CLIDemo
- [ ] Full flow completes in <3 minutes
- [ ] No crashes during 5-minute run
- [ ] Q-value improvement visible

---

## Next Phase: Architecture

With pseudocode complete, the next SPARC phase involves:

1. **File structure** - Define TypeScript module organization
2. **Interface contracts** - TypeScript interfaces for each component
3. **Dependency injection** - Wiring components together
4. **Error boundaries** - Try/catch patterns
5. **Test scaffolding** - Jest test file structure

See [ARCH-EmotiStream-MVP.md](../ARCH-EmotiStream-MVP.md) for architecture details.

---

**SPARC Phase 2 Complete** - 6 pseudocode documents ready for implementation.
