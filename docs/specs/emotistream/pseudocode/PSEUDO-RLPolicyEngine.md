# EmotiStream RL Policy Engine - Pseudocode Specification

**Component**: Reinforcement Learning Policy Engine
**Version**: 1.0.0
**Date**: 2025-12-05
**SPARC Phase**: 2 - Pseudocode

---

## Table of Contents
1. [Overview](#overview)
2. [Data Structures](#data-structures)
3. [Core Algorithms](#core-algorithms)
4. [Reward Calculation](#reward-calculation)
5. [Exploration Strategies](#exploration-strategies)
6. [Experience Replay](#experience-replay)
7. [Complexity Analysis](#complexity-analysis)
8. [Example Scenarios](#example-scenarios)

---

## Overview

The RL Policy Engine implements Q-learning with temporal difference (TD) learning to optimize content recommendations for emotional state transitions. It uses epsilon-greedy exploration with UCB bonuses and experience replay for improved sample efficiency.

### Key Parameters
```
LEARNING_RATE (α) = 0.1
DISCOUNT_FACTOR (γ) = 0.95
INITIAL_EXPLORATION_RATE (ε₀) = 0.15
MINIMUM_EXPLORATION_RATE (ε_min) = 0.10
EXPLORATION_DECAY = 0.95 per episode
UCB_CONSTANT (c) = 2.0
VALENCE_BUCKETS = 5    // [-1.0, 1.0] → [0, 4]
AROUSAL_BUCKETS = 5    // [-1.0, 1.0] → [0, 4]
STRESS_BUCKETS = 3     // [0.0, 1.0] → [0, 2]
REPLAY_BUFFER_SIZE = 1000
BATCH_SIZE = 32
```

---

## Data Structures

### QTableEntry
```
STRUCTURE QTableEntry:
    userId: STRING              // User identifier
    stateHash: STRING          // "v:a:s" format (e.g., "2:3:1")
    contentId: STRING          // Content identifier
    qValue: FLOAT              // Expected cumulative reward
    visitCount: INTEGER        // Number of times (s,a) visited
    lastUpdated: TIMESTAMP     // Last update time
    createdAt: TIMESTAMP       // Creation time
END STRUCTURE
```

### EmotionalState
```
STRUCTURE EmotionalState:
    valence: FLOAT     // [-1.0, 1.0] negative to positive
    arousal: FLOAT     // [-1.0, 1.0] calm to excited
    stress: FLOAT      // [0.0, 1.0] relaxed to stressed
    confidence: FLOAT  // [0.0, 1.0] prediction confidence
END STRUCTURE
```

### EmotionalExperience
```
STRUCTURE EmotionalExperience:
    experienceId: STRING
    userId: STRING
    stateBefore: EmotionalState
    stateAfter: EmotionalState
    contentId: STRING
    desiredState: {
        valence: FLOAT,
        arousal: FLOAT
    }
    reward: FLOAT
    timestamp: TIMESTAMP
END STRUCTURE
```

### ContentRecommendation
```
STRUCTURE ContentRecommendation:
    contentId: STRING
    title: STRING
    expectedReward: FLOAT      // Q-value
    explorationBonus: FLOAT    // UCB bonus if exploring
    isExploration: BOOLEAN     // True if ε-greedy exploration
    confidence: FLOAT          // Based on visit count
END STRUCTURE
```

### ReplayBuffer
```
STRUCTURE ReplayBuffer:
    experiences: CIRCULAR_BUFFER<EmotionalExperience>
    maxSize: INTEGER = 1000
    currentSize: INTEGER
    insertIndex: INTEGER
END STRUCTURE
```

---

## Core Algorithms

### 1. Action Selection (Main Entry Point)

```
ALGORITHM: selectAction
INPUT:
    userId: STRING
    emotionalState: EmotionalState
    desiredState: {valence: FLOAT, arousal: FLOAT}
    availableContent: ARRAY<STRING>  // Array of contentIds
OUTPUT:
    ContentRecommendation

BEGIN
    // Step 1: Discretize current emotional state
    stateHash ← hashState(emotionalState)

    // Step 2: Get current exploration rate for user
    userEpsilon ← getUserExplorationRate(userId)

    // Step 3: Decide exploration vs exploitation
    randomValue ← RANDOM(0, 1)

    IF randomValue < userEpsilon THEN
        // EXPLORE: Use UCB to select action
        recommendation ← explore(userId, stateHash, availableContent)
        recommendation.isExploration ← TRUE
    ELSE
        // EXPLOIT: Use best known Q-value
        recommendation ← exploit(userId, stateHash, availableContent)
        recommendation.isExploration ← FALSE
    END IF

    // Step 4: Log selection for monitoring
    logActionSelection(userId, stateHash, recommendation, userEpsilon)

    RETURN recommendation
END
```

**Time Complexity**: O(n) where n = availableContent.length
**Space Complexity**: O(1) excluding database queries

---

### 2. Exploitation Strategy

```
ALGORITHM: exploit
INPUT:
    userId: STRING
    stateHash: STRING
    availableContent: ARRAY<STRING>
OUTPUT:
    ContentRecommendation

BEGIN
    maxQValue ← -INFINITY
    bestContentId ← NULL
    bestMetadata ← NULL

    // Iterate through all available content
    FOR EACH contentId IN availableContent DO
        // Retrieve Q-value for this state-action pair
        qValue ← getQValue(userId, stateHash, contentId)

        IF qValue > maxQValue THEN
            maxQValue ← qValue
            bestContentId ← contentId

            // Get metadata for confidence calculation
            entry ← getQTableEntry(userId, stateHash, contentId)
            bestMetadata ← entry
        END IF
    END FOR

    // If no Q-values exist, return random content
    IF bestContentId IS NULL THEN
        bestContentId ← RANDOM_CHOICE(availableContent)
        maxQValue ← 0.0
        confidence ← 0.0
    ELSE
        // Calculate confidence based on visit count
        // confidence = 1 - exp(-visitCount / 10)
        confidence ← 1.0 - EXP(-bestMetadata.visitCount / 10.0)
    END IF

    // Retrieve content metadata
    contentInfo ← getContentInfo(bestContentId)

    RETURN ContentRecommendation {
        contentId: bestContentId,
        title: contentInfo.title,
        expectedReward: maxQValue,
        explorationBonus: 0.0,
        isExploration: FALSE,
        confidence: confidence
    }
END
```

**Time Complexity**: O(n) where n = availableContent.length
**Space Complexity**: O(1)

---

### 3. Exploration Strategy (UCB-Based)

```
ALGORITHM: explore
INPUT:
    userId: STRING
    stateHash: STRING
    availableContent: ARRAY<STRING>
OUTPUT:
    ContentRecommendation

BEGIN
    maxUCB ← -INFINITY
    bestContentId ← NULL
    bestQValue ← 0.0
    bestBonus ← 0.0

    // Get total visit count for this state
    totalVisits ← getTotalStateVisits(userId, stateHash)

    // If state never visited, return random content
    IF totalVisits = 0 THEN
        bestContentId ← RANDOM_CHOICE(availableContent)

        RETURN ContentRecommendation {
            contentId: bestContentId,
            title: getContentInfo(bestContentId).title,
            expectedReward: 0.0,
            explorationBonus: INFINITY,
            isExploration: TRUE,
            confidence: 0.0
        }
    END IF

    // Calculate UCB for each action
    FOR EACH contentId IN availableContent DO
        // Get Q-value and visit count
        entry ← getQTableEntry(userId, stateHash, contentId)

        IF entry EXISTS THEN
            qValue ← entry.qValue
            actionVisits ← entry.visitCount
        ELSE
            qValue ← 0.0
            actionVisits ← 0
        END IF

        // Calculate UCB bonus: c * sqrt(ln(N) / n)
        // If action never visited, UCB = infinity
        IF actionVisits = 0 THEN
            ucbBonus ← INFINITY
        ELSE
            ucbBonus ← UCB_CONSTANT * SQRT(LN(totalVisits) / actionVisits)
        END IF

        // UCB value = Q-value + exploration bonus
        ucbValue ← qValue + ucbBonus

        IF ucbValue > maxUCB THEN
            maxUCB ← ucbValue
            bestContentId ← contentId
            bestQValue ← qValue
            bestBonus ← ucbBonus
        END IF
    END FOR

    // Calculate confidence
    entry ← getQTableEntry(userId, stateHash, bestContentId)
    confidence ← entry EXISTS ? 1.0 - EXP(-entry.visitCount / 10.0) : 0.0

    RETURN ContentRecommendation {
        contentId: bestContentId,
        title: getContentInfo(bestContentId).title,
        expectedReward: bestQValue,
        explorationBonus: bestBonus,
        isExploration: TRUE,
        confidence: confidence
    }
END
```

**Time Complexity**: O(n) where n = availableContent.length
**Space Complexity**: O(1)

---

### 4. Q-Value Update (TD Learning)

```
ALGORITHM: updatePolicy
INPUT:
    experience: EmotionalExperience
OUTPUT:
    VOID

BEGIN
    // Step 1: Extract experience components
    userId ← experience.userId
    contentId ← experience.contentId
    stateBefore ← experience.stateBefore
    stateAfter ← experience.stateAfter
    reward ← experience.reward

    // Step 2: Hash states
    currentStateHash ← hashState(stateBefore)
    nextStateHash ← hashState(stateAfter)

    // Step 3: Get current Q-value Q(s, a)
    currentQ ← getQValue(userId, currentStateHash, contentId)

    // Step 4: Get maximum Q-value for next state max_a' Q(s', a')
    maxNextQ ← getMaxQValue(userId, nextStateHash)

    // Step 5: TD Learning Update
    // Q(s,a) ← Q(s,a) + α[r + γ·max(Q(s',a')) - Q(s,a)]
    tdTarget ← reward + DISCOUNT_FACTOR * maxNextQ
    tdError ← tdTarget - currentQ
    newQ ← currentQ + LEARNING_RATE * tdError

    // Step 6: Update Q-table
    entry ← getQTableEntry(userId, currentStateHash, contentId)

    IF entry EXISTS THEN
        entry.qValue ← newQ
        entry.visitCount ← entry.visitCount + 1
        entry.lastUpdated ← CURRENT_TIMESTAMP()
    ELSE
        entry ← QTableEntry {
            userId: userId,
            stateHash: currentStateHash,
            contentId: contentId,
            qValue: newQ,
            visitCount: 1,
            lastUpdated: CURRENT_TIMESTAMP(),
            createdAt: CURRENT_TIMESTAMP()
        }
    END IF

    setQTableEntry(entry)

    // Step 7: Add to replay buffer
    addToReplayBuffer(experience)

    // Step 8: Decay exploration rate
    decayExplorationRate(userId)

    // Step 9: Log update for monitoring
    logPolicyUpdate(userId, currentStateHash, contentId, {
        oldQ: currentQ,
        newQ: newQ,
        tdError: tdError,
        reward: reward,
        visitCount: entry.visitCount
    })
END
```

**Time Complexity**: O(1) for single update
**Space Complexity**: O(1)

---

### 5. State Discretization

```
ALGORITHM: hashState
INPUT:
    emotionalState: EmotionalState
OUTPUT:
    STRING  // Format: "v:a:s" (e.g., "2:3:1")

BEGIN
    // Discretize valence: [-1.0, 1.0] → [0, 4]
    // Each bucket covers range of 0.4
    valenceBucket ← FLOOR((emotionalState.valence + 1.0) / 0.4)
    valenceBucket ← CLAMP(valenceBucket, 0, VALENCE_BUCKETS - 1)

    // Discretize arousal: [-1.0, 1.0] → [0, 4]
    arousalBucket ← FLOOR((emotionalState.arousal + 1.0) / 0.4)
    arousalBucket ← CLAMP(arousalBucket, 0, AROUSAL_BUCKETS - 1)

    // Discretize stress: [0.0, 1.0] → [0, 2]
    // Each bucket covers range of ~0.33
    stressBucket ← FLOOR(emotionalState.stress / 0.34)
    stressBucket ← CLAMP(stressBucket, 0, STRESS_BUCKETS - 1)

    // Create hash string
    stateHash ← valenceBucket + ":" + arousalBucket + ":" + stressBucket

    RETURN stateHash
END

HELPER FUNCTION: CLAMP
INPUT: value: INTEGER, min: INTEGER, max: INTEGER
OUTPUT: INTEGER
BEGIN
    IF value < min THEN RETURN min
    IF value > max THEN RETURN max
    RETURN value
END
```

**State Space Size**: 5 × 5 × 3 = 75 discrete states
**Time Complexity**: O(1)
**Space Complexity**: O(1)

---

## Reward Calculation

### 6. Reward Function

```
ALGORITHM: calculateReward
INPUT:
    stateBefore: EmotionalState
    stateAfter: EmotionalState
    desiredState: {valence: FLOAT, arousal: FLOAT}
OUTPUT:
    FLOAT  // Reward in range [-1.0, 1.0]

BEGIN
    // Step 1: Calculate actual movement vector
    actualDelta ← {
        valence: stateAfter.valence - stateBefore.valence,
        arousal: stateAfter.arousal - stateBefore.arousal
    }

    // Step 2: Calculate desired movement vector
    desiredDelta ← {
        valence: desiredState.valence - stateBefore.valence,
        arousal: desiredState.arousal - stateBefore.arousal
    }

    // Step 3: Direction Alignment (Cosine Similarity) - 60% weight
    dotProduct ← actualDelta.valence * desiredDelta.valence +
                 actualDelta.arousal * desiredDelta.arousal

    actualMagnitude ← SQRT(actualDelta.valence² + actualDelta.arousal²)
    desiredMagnitude ← SQRT(desiredDelta.valence² + desiredDelta.arousal²)

    IF actualMagnitude = 0 OR desiredMagnitude = 0 THEN
        directionScore ← 0.0
    ELSE
        // Cosine similarity: cos(θ) = (a·b) / (|a||b|)
        cosineSimilarity ← dotProduct / (actualMagnitude * desiredMagnitude)
        // Normalize from [-1, 1] to [0, 1]
        directionScore ← (cosineSimilarity + 1.0) / 2.0
    END IF

    // Step 4: Magnitude of Improvement - 40% weight
    // How much did we move in the right direction?
    IF desiredMagnitude > 0 THEN
        magnitudeScore ← MIN(actualMagnitude / desiredMagnitude, 1.0)
    ELSE
        // Already at desired state
        magnitudeScore ← 1.0
    END IF

    // Step 5: Calculate base reward
    baseReward ← 0.6 * directionScore + 0.4 * magnitudeScore

    // Step 6: Proximity Bonus
    // If we reached within 0.15 of desired state, bonus +0.2
    distance ← SQRT((stateAfter.valence - desiredState.valence)² +
                    (stateAfter.arousal - desiredState.arousal)²)

    IF distance < 0.15 THEN
        proximityBonus ← 0.2
    ELSE
        proximityBonus ← 0.0
    END IF

    // Step 7: Stress Penalty
    // Penalize if stress increased significantly
    stressIncrease ← stateAfter.stress - stateBefore.stress

    IF stressIncrease > 0.2 THEN
        stressPenalty ← -0.15
    ELSE
        stressPenalty ← 0.0
    END IF

    // Step 8: Calculate final reward
    finalReward ← baseReward + proximityBonus + stressPenalty

    // Clamp to [-1.0, 1.0]
    finalReward ← CLAMP_FLOAT(finalReward, -1.0, 1.0)

    RETURN finalReward
END

HELPER FUNCTION: CLAMP_FLOAT
INPUT: value: FLOAT, min: FLOAT, max: FLOAT
OUTPUT: FLOAT
BEGIN
    IF value < min THEN RETURN min
    IF value > max THEN RETURN max
    RETURN value
END
```

**Reward Components**:
1. **Direction Alignment** (60%): Cosine similarity between actual and desired movement
2. **Magnitude Score** (40%): How far we moved toward goal
3. **Proximity Bonus** (+0.2): Reached desired state (distance < 0.15)
4. **Stress Penalty** (-0.15): Significant stress increase (>0.2)

**Time Complexity**: O(1)
**Space Complexity**: O(1)

---

## Exploration Strategies

### 7. Exploration Rate Management

```
ALGORITHM: getUserExplorationRate
INPUT:
    userId: STRING
OUTPUT:
    FLOAT  // Current epsilon value

BEGIN
    // Retrieve user's episode count and current epsilon
    userStats ← getUserRLStats(userId)

    IF userStats NOT EXISTS THEN
        // New user: start with initial exploration rate
        RETURN INITIAL_EXPLORATION_RATE
    END IF

    RETURN MAX(userStats.currentEpsilon, MINIMUM_EXPLORATION_RATE)
END

ALGORITHM: decayExplorationRate
INPUT:
    userId: STRING
OUTPUT:
    VOID

BEGIN
    userStats ← getUserRLStats(userId)

    IF userStats NOT EXISTS THEN
        // Initialize user stats
        userStats ← {
            userId: userId,
            episodeCount: 0,
            currentEpsilon: INITIAL_EXPLORATION_RATE,
            totalReward: 0.0,
            lastUpdated: CURRENT_TIMESTAMP()
        }
    END IF

    // Increment episode count
    userStats.episodeCount ← userStats.episodeCount + 1

    // Decay epsilon: ε = ε * decay_rate
    newEpsilon ← userStats.currentEpsilon * EXPLORATION_DECAY

    // Ensure epsilon doesn't go below minimum
    userStats.currentEpsilon ← MAX(newEpsilon, MINIMUM_EXPLORATION_RATE)

    userStats.lastUpdated ← CURRENT_TIMESTAMP()

    // Persist updated stats
    setUserRLStats(userStats)
END
```

**Exploration Schedule**:
- Episode 0: ε = 0.15
- Episode 10: ε ≈ 0.089
- Episode 20: ε ≈ 0.10 (minimum reached)
- Episode 50+: ε = 0.10 (stable)

---

### 8. UCB Calculation Details

```
ALGORITHM: getTotalStateVisits
INPUT:
    userId: STRING
    stateHash: STRING
OUTPUT:
    INTEGER  // Total visits to this state

BEGIN
    // Query AgentDB for all Q-table entries matching state
    query ← {
        metadata: {
            userId: userId,
            stateHash: stateHash
        }
    }

    entries ← agentDB.query(query, limit: 1000)

    totalVisits ← 0
    FOR EACH entry IN entries DO
        totalVisits ← totalVisits + entry.visitCount
    END FOR

    RETURN totalVisits
END
```

**UCB Formula**:
```
UCB(s, a) = Q(s, a) + c * sqrt(ln(N(s)) / N(s, a))

Where:
- Q(s, a) = current Q-value estimate
- c = exploration constant (2.0)
- N(s) = total visits to state s
- N(s, a) = visits to action a in state s
```

---

## Experience Replay

### 9. Replay Buffer Management

```
ALGORITHM: addToReplayBuffer
INPUT:
    experience: EmotionalExperience
OUTPUT:
    VOID

BEGIN
    buffer ← getReplayBuffer()

    // Circular buffer: overwrite oldest if full
    IF buffer.currentSize < buffer.maxSize THEN
        buffer.currentSize ← buffer.currentSize + 1
    END IF

    buffer.experiences[buffer.insertIndex] ← experience
    buffer.insertIndex ← (buffer.insertIndex + 1) MOD buffer.maxSize

    saveReplayBuffer(buffer)
END

ALGORITHM: batchUpdate
INPUT:
    batchSize: INTEGER = BATCH_SIZE
OUTPUT:
    VOID

BEGIN
    buffer ← getReplayBuffer()

    IF buffer.currentSize < batchSize THEN
        // Not enough experiences yet
        RETURN
    END IF

    // Sample random batch from buffer
    sampledExperiences ← RANDOM_SAMPLE(buffer.experiences, batchSize)

    // Update policy for each sampled experience
    FOR EACH experience IN sampledExperiences DO
        updatePolicy(experience)
    END FOR

    // Log batch update
    logBatchUpdate(batchSize, CURRENT_TIMESTAMP())
END
```

**Replay Buffer Benefits**:
1. **Sample Efficiency**: Learn from past experiences multiple times
2. **Correlation Breaking**: Randomized sampling reduces sequential correlation
3. **Stability**: Smooths out noisy updates

---

### 10. Batch Learning Strategy

```
ALGORITHM: periodicBatchLearning
INPUT:
    NONE (scheduled task)
OUTPUT:
    VOID

BEGIN
    // Run every hour or after N new experiences
    buffer ← getReplayBuffer()

    IF buffer.currentSize < BATCH_SIZE THEN
        RETURN  // Not enough data
    END IF

    // Perform 5 batch updates with random sampling
    FOR i ← 1 TO 5 DO
        batchUpdate(BATCH_SIZE)
    END FOR

    logPeriodicLearning(CURRENT_TIMESTAMP())
END
```

---

## AgentDB Integration

### 11. Q-Table Storage Patterns

```
ALGORITHM: getQValue
INPUT:
    userId: STRING
    stateHash: STRING
    contentId: STRING
OUTPUT:
    FLOAT  // Q-value, defaults to 0.0

BEGIN
    // AgentDB key pattern: "qtable:{userId}:{stateHash}:{contentId}"
    key ← "qtable:" + userId + ":" + stateHash + ":" + contentId

    entry ← agentDB.get(key)

    IF entry EXISTS THEN
        RETURN entry.qValue
    ELSE
        RETURN 0.0  // Default Q-value for new state-action pairs
    END IF
END

ALGORITHM: setQTableEntry
INPUT:
    entry: QTableEntry
OUTPUT:
    VOID

BEGIN
    // Construct key
    key ← "qtable:" + entry.userId + ":" +
           entry.stateHash + ":" + entry.contentId

    // Store in AgentDB with metadata for querying
    agentDB.set(key, entry, {
        metadata: {
            userId: entry.userId,
            stateHash: entry.stateHash,
            contentId: entry.contentId,
            qValue: entry.qValue,
            visitCount: entry.visitCount
        },
        ttl: 90 * 24 * 60 * 60  // 90 days retention
    })
END

ALGORITHM: getMaxQValue
INPUT:
    userId: STRING
    stateHash: STRING
OUTPUT:
    FLOAT  // Maximum Q-value for state

BEGIN
    // Query all Q-values for this state
    query ← {
        metadata: {
            userId: userId,
            stateHash: stateHash
        }
    }

    entries ← agentDB.query(query, limit: 1000)

    IF entries.length = 0 THEN
        RETURN 0.0  // No Q-values yet
    END IF

    maxQ ← -INFINITY
    FOR EACH entry IN entries DO
        IF entry.qValue > maxQ THEN
            maxQ ← entry.qValue
        END IF
    END FOR

    RETURN maxQ
END
```

**AgentDB Key Patterns**:
```
Q-Table Entry:     qtable:{userId}:{stateHash}:{contentId}
User RL Stats:     rlstats:{userId}
Replay Buffer:     replay:{userId}
Episode History:   episodes:{userId}:{episodeId}
```

---

## Convergence Detection

### 12. Policy Convergence Monitoring

```
ALGORITHM: checkConvergence
INPUT:
    userId: STRING
OUTPUT:
    BOOLEAN  // True if policy has converged

BEGIN
    // Get recent TD errors
    recentErrors ← getRecentTDErrors(userId, count: 100)

    IF recentErrors.length < 100 THEN
        RETURN FALSE  // Not enough data
    END IF

    // Calculate mean absolute TD error
    sumAbsError ← 0.0
    FOR EACH error IN recentErrors DO
        sumAbsError ← sumAbsError + ABS(error)
    END FOR

    meanAbsError ← sumAbsError / recentErrors.length

    // Calculate standard deviation of TD errors
    variance ← 0.0
    FOR EACH error IN recentErrors DO
        variance ← variance + (error - meanAbsError)²
    END FOR

    stdDev ← SQRT(variance / recentErrors.length)

    // Convergence criteria:
    // 1. Mean absolute error < 0.05
    // 2. Standard deviation < 0.1
    // 3. At least 200 total updates

    totalUpdates ← getTotalUpdateCount(userId)

    hasConverged ← (meanAbsError < 0.05) AND
                   (stdDev < 0.1) AND
                   (totalUpdates >= 200)

    RETURN hasConverged
END
```

**Convergence Indicators**:
1. Mean absolute TD error < 0.05
2. TD error standard deviation < 0.1
3. Minimum 200 policy updates
4. Stable Q-values (< 1% change over 50 updates)

---

## Complexity Analysis

### Overall System Complexity

#### Time Complexity

| Operation | Complexity | Notes |
|-----------|------------|-------|
| `selectAction` | O(n) | n = available content count |
| `exploit` | O(n) | Linear scan for max Q-value |
| `explore` | O(n) | UCB calculation for each action |
| `updatePolicy` | O(1) | Single Q-value update |
| `hashState` | O(1) | Simple arithmetic operations |
| `calculateReward` | O(1) | Vector operations |
| `batchUpdate` | O(k) | k = batch size (32) |
| `getMaxQValue` | O(m) | m = actions in state (~20-50) |

#### Space Complexity

| Component | Complexity | Notes |
|-----------|------------|-------|
| Q-Table | O(S × A) | S = 75 states, A = content count |
| Replay Buffer | O(B) | B = 1000 experiences |
| User Stats | O(U) | U = user count |
| Episode History | O(E) | E = episodes per user (~100) |

**State Space**: 5 × 5 × 3 = 75 discrete states
**Action Space**: Variable (content catalog size, ~1000-10000 items)
**Q-Table Size**: 75 × A entries per user

---

## Example Scenarios

### Scenario 1: New User First Recommendation

```
SCENARIO: First-time user seeks uplifting content
==========================================

INPUT:
    userId: "user_001"
    emotionalState: {
        valence: -0.6,    // Negative mood
        arousal: -0.4,    // Low energy
        stress: 0.7       // High stress
    }
    desiredState: {
        valence: 0.6,     // Positive mood
        arousal: 0.3      // Moderate energy
    }
    availableContent: ["content_1", "content_2", ..., "content_50"]

EXECUTION TRACE:
==========================================

1. selectAction() called
   └─ stateHash ← hashState(emotionalState)
      ├─ valenceBucket = floor((-0.6 + 1.0) / 0.4) = floor(1.0) = 1
      ├─ arousalBucket = floor((-0.4 + 1.0) / 0.4) = floor(1.5) = 1
      └─ stressBucket = floor(0.7 / 0.34) = floor(2.06) = 2
      └─ stateHash = "1:1:2"

2. getUserExplorationRate("user_001")
   └─ No stats found (new user)
   └─ RETURN 0.15 (initial epsilon)

3. randomValue = 0.08 < 0.15
   └─ EXPLORE path chosen

4. explore("user_001", "1:1:2", availableContent)
   └─ getTotalStateVisits("user_001", "1:1:2")
      └─ No entries found
      └─ RETURN 0

   └─ totalVisits = 0, so random selection
   └─ bestContentId ← RANDOM_CHOICE(availableContent)
      └─ Selected: "content_23"

   └─ RETURN {
         contentId: "content_23",
         expectedReward: 0.0,
         explorationBonus: INFINITY,
         isExploration: TRUE,
         confidence: 0.0
      }

RESULT: Random exploration due to new state
```

---

### Scenario 2: Q-Value Update After Feedback

```
SCENARIO: Update policy after content consumption
==========================================

INPUT (Experience):
    userId: "user_001"
    stateBefore: {
        valence: -0.6,
        arousal: -0.4,
        stress: 0.7
    }
    stateAfter: {
        valence: 0.2,    // Improved!
        arousal: 0.1,
        stress: 0.5      // Reduced stress
    }
    contentId: "content_23"
    desiredState: {
        valence: 0.6,
        arousal: 0.3
    }

EXECUTION TRACE:
==========================================

1. calculateReward() called

   a) Calculate movement vectors:
      actualDelta = {
          valence: 0.2 - (-0.6) = 0.8,
          arousal: 0.1 - (-0.4) = 0.5
      }

      desiredDelta = {
          valence: 0.6 - (-0.6) = 1.2,
          arousal: 0.3 - (-0.4) = 0.7
      }

   b) Direction Alignment (60%):
      dotProduct = (0.8 × 1.2) + (0.5 × 0.7) = 0.96 + 0.35 = 1.31
      actualMagnitude = sqrt(0.8² + 0.5²) = sqrt(0.64 + 0.25) = 0.943
      desiredMagnitude = sqrt(1.2² + 0.7²) = sqrt(1.44 + 0.49) = 1.389

      cosineSimilarity = 1.31 / (0.943 × 1.389) = 1.31 / 1.310 = 1.0
      directionScore = (1.0 + 1.0) / 2 = 1.0  ✓ Perfect alignment!

   c) Magnitude of Improvement (40%):
      magnitudeScore = min(0.943 / 1.389, 1.0) = 0.679

   d) Base reward:
      baseReward = 0.6 × 1.0 + 0.4 × 0.679 = 0.6 + 0.272 = 0.872

   e) Proximity bonus:
      distance = sqrt((0.2 - 0.6)² + (0.1 - 0.3)²)
               = sqrt(0.16 + 0.04) = sqrt(0.2) = 0.447
      distance > 0.15, so proximityBonus = 0.0

   f) Stress penalty:
      stressIncrease = 0.5 - 0.7 = -0.2 (decreased)
      stressPenalty = 0.0 (no penalty for decrease)

   g) Final reward:
      reward = 0.872 + 0.0 + 0.0 = 0.872

2. updatePolicy() called with reward = 0.872

   a) State hashing:
      currentStateHash = "1:1:2"  (as before)
      nextStateHash = hashState(stateAfter)
         ├─ valenceBucket = floor((0.2 + 1.0) / 0.4) = 3
         ├─ arousalBucket = floor((0.1 + 1.0) / 0.4) = 2
         └─ stressBucket = floor(0.5 / 0.34) = 1
         └─ nextStateHash = "3:2:1"

   b) Get Q-values:
      currentQ = getQValue("user_001", "1:1:2", "content_23")
               = 0.0 (new entry)

      maxNextQ = getMaxQValue("user_001", "3:2:1")
               = 0.0 (new state)

   c) TD Learning:
      tdTarget = 0.872 + 0.95 × 0.0 = 0.872
      tdError = 0.872 - 0.0 = 0.872
      newQ = 0.0 + 0.1 × 0.872 = 0.0872

   d) Update Q-table:
      setQTableEntry({
          userId: "user_001",
          stateHash: "1:1:2",
          contentId: "content_23",
          qValue: 0.0872,
          visitCount: 1,
          lastUpdated: <timestamp>
      })

   e) Decay exploration:
      newEpsilon = 0.15 × 0.95 = 0.1425

RESULT: Q-value updated to 0.0872 for this state-action pair
```

---

### Scenario 3: Exploitation After Learning

```
SCENARIO: Second visit to same emotional state
==========================================

INPUT:
    userId: "user_001"
    emotionalState: {
        valence: -0.55,   // Similar to before
        arousal: -0.38,
        stress: 0.68
    }
    desiredState: {
        valence: 0.6,
        arousal: 0.3
    }
    availableContent: ["content_1", ..., "content_23", ..., "content_50"]

EXECUTION TRACE:
==========================================

1. selectAction() called
   └─ stateHash = "1:1:2" (same bucket due to similar values)

2. getUserExplorationRate("user_001")
   └─ currentEpsilon = 0.1425

3. randomValue = 0.68 > 0.1425
   └─ EXPLOIT path chosen (use learned Q-values)

4. exploit("user_001", "1:1:2", availableContent)

   Iterate through content:
   ├─ content_1: Q = 0.0
   ├─ content_2: Q = 0.0
   ├─ ...
   ├─ content_23: Q = 0.0872  ← HIGHEST!
   ├─ ...
   └─ content_50: Q = 0.0

   └─ bestContentId = "content_23"
      maxQValue = 0.0872
      visitCount = 1
      confidence = 1 - exp(-1/10) = 1 - 0.905 = 0.095

   └─ RETURN {
         contentId: "content_23",
         expectedReward: 0.0872,
         explorationBonus: 0.0,
         isExploration: FALSE,
         confidence: 0.095
      }

RESULT: System exploits learned knowledge and recommends content_23
```

---

### Scenario 4: Multi-Episode Learning Convergence

```
SCENARIO: Policy convergence after 10 episodes
==========================================

EPISODE HISTORY:
Episode 1: state "1:1:2" → content_23 → reward 0.872 → Q = 0.0872
Episode 2: state "1:1:2" → content_23 → reward 0.791 → Q = 0.1578
Episode 3: state "1:1:2" → content_45 → reward 0.654 → Q = 0.0654
Episode 4: state "1:1:2" → content_23 → reward 0.823 → Q = 0.2244
Episode 5: state "1:1:2" → content_23 → reward 0.798 → Q = 0.2818
Episode 6: state "1:1:2" → content_12 → reward 0.412 → Q = 0.0412
Episode 7: state "1:1:2" → content_23 → reward 0.805 → Q = 0.3333
Episode 8: state "1:1:2" → content_23 → reward 0.791 → Q = 0.3791
Episode 9: state "1:1:2" → content_23 → reward 0.786 → Q = 0.4198
Episode 10: state "1:1:2" → content_23 → reward 0.793 → Q = 0.4571

Q-VALUE PROGRESSION FOR content_23:
Episode:  1     2     3     4     5     6     7     8     9     10
Q-value:  0.09  0.16  0.16  0.22  0.28  0.28  0.33  0.38  0.42  0.46
TD Error: 0.87  0.71  N/A   0.66  0.57  N/A   0.51  0.46  0.41  0.37

CONVERGENCE ANALYSIS:
├─ Mean absolute TD error (last 6 episodes): 0.511
├─ Standard deviation: 0.124
├─ Total updates: 10
└─ Status: NOT CONVERGED (needs ~200 updates)

EXPLORATION RATE DECAY:
Episode:  0     1     2     3     4     5     6     7     8     9     10
Epsilon:  0.15  0.14  0.14  0.13  0.12  0.11  0.11  0.10  0.10  0.10  0.10
                                                      ↑ Minimum reached

RESULT: Policy learning in progress, content_23 emerging as optimal
```

---

### Scenario 5: Batch Learning from Replay Buffer

```
SCENARIO: Periodic batch update from experience replay
==========================================

REPLAY BUFFER STATE:
Size: 47 experiences
Sampling: 32 random experiences

SAMPLED EXPERIENCES (abbreviated):
[
    {state: "1:1:2", action: "content_23", reward: 0.872},
    {state: "3:2:1", action: "content_45", reward: 0.654},
    {state: "2:3:0", action: "content_12", reward: 0.412},
    ... (29 more)
]

BATCH UPDATE EXECUTION:
==========================================

FOR EACH of 32 sampled experiences:
    updatePolicy(experience)

Example update #1:
├─ Experience: state "1:1:2" → content_23 → reward 0.872
├─ Current Q("1:1:2", content_23) = 0.4571
├─ Max Q("3:2:1") = 0.3211
├─ TD target = 0.872 + 0.95 × 0.3211 = 1.177
├─ TD error = 1.177 - 0.4571 = 0.720
├─ New Q = 0.4571 + 0.1 × 0.720 = 0.5291
└─ Update applied

Example update #2:
├─ Experience: state "2:3:0" → content_12 → reward 0.412
├─ Current Q("2:3:0", content_12) = 0.2145
├─ Max Q("3:3:0") = 0.4521
├─ TD target = 0.412 + 0.95 × 0.4521 = 0.841
├─ TD error = 0.841 - 0.2145 = 0.627
├─ New Q = 0.2145 + 0.1 × 0.627 = 0.2772
└─ Update applied

... (30 more updates)

RESULT: Q-values refined across multiple state-action pairs
```

---

## Implementation Checklist

### Core RL Components
- [ ] Q-table storage in AgentDB with metadata indexing
- [ ] State discretization with 5×5×3 buckets
- [ ] ε-greedy action selection with decay
- [ ] UCB exploration bonus calculation
- [ ] TD learning Q-value updates
- [ ] Reward function with direction alignment
- [ ] Experience replay buffer (circular, size 1000)
- [ ] Batch learning (size 32)

### Monitoring & Analytics
- [ ] Per-user exploration rate tracking
- [ ] TD error logging for convergence detection
- [ ] Q-value change monitoring
- [ ] Episode reward tracking
- [ ] Action selection metrics (explore vs exploit ratio)
- [ ] State visitation frequency

### Performance Optimizations
- [ ] AgentDB query optimization for Q-value lookups
- [ ] Batch Q-value updates for efficiency
- [ ] Caching for frequently accessed states
- [ ] Asynchronous policy updates
- [ ] Periodic replay buffer pruning

### Testing Requirements
- [ ] Unit tests for reward calculation
- [ ] State hashing correctness tests
- [ ] Q-value update verification
- [ ] Exploration vs exploitation balance tests
- [ ] Convergence detection tests
- [ ] Replay buffer management tests

---

## References

1. **Sutton & Barto** - Reinforcement Learning: An Introduction (2nd Edition)
2. **UCB Algorithm** - Auer, P., Cesa-Bianchi, N., & Fischer, P. (2002)
3. **Experience Replay** - Lin, L. J. (1992). Self-improving reactive agents based on reinforcement learning
4. **Temporal Difference Learning** - Sutton, R. S. (1988)
5. **Epsilon-Greedy Exploration** - Watkins, C. J., & Dayan, P. (1992)

---

**Document Status**: Complete
**Next Phase**: Architecture (SPARC Phase 3)
**Implementation Target**: MVP v1.0.0
