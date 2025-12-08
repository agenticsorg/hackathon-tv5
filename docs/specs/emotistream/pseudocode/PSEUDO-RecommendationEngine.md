# EmotiStream Nexus - Recommendation Engine Pseudocode

## Component Overview

The Recommendation Engine fuses Reinforcement Learning policy (Q-values) with semantic vector search to produce emotionally-aware content recommendations. It optimizes for emotional state transitions using hybrid ranking.

---

## Data Structures

### Input Types

```
STRUCTURE: EmotionalState
    id: STRING                    // Unique state identifier
    userId: STRING                // User identifier
    timestamp: TIMESTAMP          // When state was recorded
    valence: FLOAT               // -1.0 (negative) to 1.0 (positive)
    arousal: FLOAT               // -1.0 (calm) to 1.0 (excited)
    stressLevel: FLOAT           // 0.0 (relaxed) to 1.0 (stressed)
    dominance: FLOAT             // -1.0 (submissive) to 1.0 (dominant)
    rawMetrics: OBJECT           // Raw sensor data
END STRUCTURE

STRUCTURE: RecommendationRequest
    userId: STRING
    emotionalStateId: STRING
    limit: INTEGER               // Default: 20
    explicitDesiredState: OPTIONAL {
        valence: FLOAT
        arousal: FLOAT
    }
    includeExploration: BOOLEAN  // Default: false
    explorationRate: FLOAT       // Default: 0.1 (10% exploration)
END STRUCTURE

STRUCTURE: EmotionalContentProfile
    contentId: STRING
    title: STRING
    platform: STRING             // "Netflix", "YouTube", etc.
    valenceDelta: FLOAT         // Expected change in valence
    arousalDelta: FLOAT         // Expected change in arousal
    stressReduction: FLOAT      // Expected stress reduction
    duration: INTEGER           // Duration in minutes
    genres: ARRAY<STRING>
    embedding: FLOAT32ARRAY[1536] // Semantic vector
END STRUCTURE
```

### Output Types

```
STRUCTURE: EmotionalRecommendation
    contentId: STRING
    title: STRING
    platform: STRING
    emotionalProfile: EmotionalContentProfile
    predictedOutcome: {
        postViewingValence: FLOAT
        postViewingArousal: FLOAT
        postViewingStress: FLOAT
        confidence: FLOAT        // 0.0 to 1.0
    }
    qValue: FLOAT               // Q-value from RL policy
    similarityScore: FLOAT      // Vector similarity [0, 1]
    hybridScore: FLOAT          // Final ranking score
    isExploration: BOOLEAN      // Was this an exploration pick?
    rank: INTEGER               // Final ranking position
    reasoning: STRING           // Human-readable explanation
END STRUCTURE

STRUCTURE: SearchCandidate
    contentId: STRING
    profile: EmotionalContentProfile
    similarity: FLOAT           // Vector similarity score
    distance: FLOAT             // Vector distance (lower is better)
END STRUCTURE
```

---

## Main Algorithm: Content Recommendation

### Primary Entry Point

```
ALGORITHM: recommend
INPUT: request (RecommendationRequest)
OUTPUT: recommendations (ARRAY<EmotionalRecommendation>)

BEGIN
    // Step 1: Load current emotional state
    currentState ← LoadEmotionalState(request.emotionalStateId)
    IF currentState is NULL THEN
        THROW ERROR("Emotional state not found")
    END IF

    // Step 2: Determine desired emotional state
    IF request.explicitDesiredState is NOT NULL THEN
        desiredState ← request.explicitDesiredState
    ELSE
        desiredState ← PredictDesiredState(currentState)
    END IF

    // Step 3: Create transition vector for semantic search
    transitionVector ← CreateTransitionVector(currentState, desiredState)

    // Step 4: Search RuVector for semantically similar content
    searchTopK ← request.limit * 3  // Get 3x candidates for reranking
    candidates ← SearchByTransition(transitionVector, searchTopK)

    // Step 5: Filter already-watched content
    candidates ← FilterWatchedContent(request.userId, candidates)

    // Step 6: Re-rank using hybrid Q-value + similarity scoring
    stateHash ← HashEmotionalState(currentState)
    rankedCandidates ← RerankWithQValues(
        request.userId,
        candidates,
        stateHash,
        desiredState
    )

    // Step 7: Apply exploration strategy
    IF request.includeExploration THEN
        rankedCandidates ← ApplyExploration(
            rankedCandidates,
            request.explorationRate
        )
    END IF

    // Step 8: Select top N and generate reasoning
    finalRecommendations ← []
    FOR i ← 0 TO MIN(request.limit - 1, LENGTH(rankedCandidates) - 1) DO
        candidate ← rankedCandidates[i]

        // Predict viewing outcome
        outcome ← PredictOutcome(currentState, candidate.profile)

        // Generate human-readable reasoning
        reasoning ← GenerateReasoning(
            currentState,
            desiredState,
            candidate.profile,
            candidate.qValue,
            candidate.isExploration
        )

        // Create recommendation object
        recommendation ← EmotionalRecommendation {
            contentId: candidate.contentId,
            title: candidate.profile.title,
            platform: candidate.profile.platform,
            emotionalProfile: candidate.profile,
            predictedOutcome: outcome,
            qValue: candidate.qValue,
            similarityScore: candidate.similarity,
            hybridScore: candidate.hybridScore,
            isExploration: candidate.isExploration,
            rank: i + 1,
            reasoning: reasoning
        }

        finalRecommendations.APPEND(recommendation)
    END FOR

    // Step 9: Log recommendation event for learning
    LogRecommendationEvent(request.userId, currentState, finalRecommendations)

    RETURN finalRecommendations
END
```

---

## Transition Vector Generation

```
ALGORITHM: CreateTransitionVector
INPUT:
    currentState (EmotionalState)
    desiredState (DesiredState with valence, arousal)
OUTPUT: transitionVector (FLOAT32ARRAY[1536])

BEGIN
    // Calculate transition deltas
    valenceDelta ← desiredState.valence - currentState.valence
    arousalDelta ← desiredState.arousal - currentState.arousal

    // Normalize deltas to [-1, 1]
    valenceDelta ← CLAMP(valenceDelta, -2.0, 2.0) / 2.0
    arousalDelta ← CLAMP(arousalDelta, -2.0, 2.0) / 2.0

    // Create feature vector for embedding
    features ← {
        // Current state (normalized)
        "current_valence": currentState.valence,
        "current_arousal": currentState.arousal,
        "current_stress": currentState.stressLevel,

        // Desired transition
        "valence_delta": valenceDelta,
        "arousal_delta": arousalDelta,
        "stress_reduction_needed": currentState.stressLevel,

        // Emotional quadrant encoding
        "quadrant_current": GetEmotionalQuadrant(
            currentState.valence,
            currentState.arousal
        ),
        "quadrant_desired": GetEmotionalQuadrant(
            desiredState.valence,
            desiredState.arousal
        ),

        // Time context
        "time_of_day": GetTimeOfDayCategory(),
        "day_of_week": GetDayOfWeekCategory()
    }

    // Generate text prompt for embedding model
    prompt ← GenerateTransitionPrompt(features)

    // Example prompt:
    // "Find content that transitions emotions from stressed anxious (valence: -0.4,
    //  arousal: 0.6) to calm relaxed (valence: 0.5, arousal: -0.4).
    //  Need stress reduction of 0.8. Suitable for evening viewing."

    // Get embedding from OpenAI/Voyage model
    transitionVector ← EmbeddingModel.embed(prompt)

    RETURN transitionVector
END

SUBROUTINE: GenerateTransitionPrompt
INPUT: features (OBJECT)
OUTPUT: prompt (STRING)

BEGIN
    currentEmotion ← DescribeEmotionalState(
        features.current_valence,
        features.current_arousal,
        features.current_stress
    )

    desiredEmotion ← DescribeEmotionalState(
        features.current_valence + features.valence_delta,
        features.current_arousal + features.arousal_delta,
        features.current_stress - features.stress_reduction_needed
    )

    prompt ← "Find content that transitions emotions from " +
             currentEmotion + " (valence: " +
             ROUND(features.current_valence, 2) + ", arousal: " +
             ROUND(features.current_arousal, 2) + ") to " +
             desiredEmotion + " (valence: " +
             ROUND(features.current_valence + features.valence_delta, 2) +
             ", arousal: " +
             ROUND(features.current_arousal + features.arousal_delta, 2) + ")."

    IF features.stress_reduction_needed > 0.5 THEN
        prompt ← prompt + " Need significant stress reduction of " +
                 ROUND(features.stress_reduction_needed, 2) + "."
    END IF

    prompt ← prompt + " Suitable for " + features.time_of_day + " viewing."

    RETURN prompt
END

SUBROUTINE: DescribeEmotionalState
INPUT: valence, arousal, stress (FLOAT)
OUTPUT: description (STRING)

BEGIN
    // Map to emotional labels
    IF valence > 0.3 AND arousal > 0.3 THEN
        emotion ← "excited happy"
    ELSE IF valence > 0.3 AND arousal < -0.3 THEN
        emotion ← "calm content"
    ELSE IF valence < -0.3 AND arousal > 0.3 THEN
        emotion ← "stressed anxious"
    ELSE IF valence < -0.3 AND arousal < -0.3 THEN
        emotion ← "sad lethargic"
    ELSE IF arousal > 0.5 THEN
        emotion ← "energized alert"
    ELSE IF arousal < -0.5 THEN
        emotion ← "relaxed calm"
    ELSE
        emotion ← "neutral balanced"
    END IF

    IF stress > 0.7 THEN
        emotion ← "highly stressed " + emotion
    ELSE IF stress > 0.4 THEN
        emotion ← "moderately stressed " + emotion
    END IF

    RETURN emotion
END

SUBROUTINE: GetEmotionalQuadrant
INPUT: valence, arousal (FLOAT)
OUTPUT: quadrant (STRING)

BEGIN
    // Russell's Circumplex Model quadrants
    IF valence >= 0 AND arousal >= 0 THEN
        RETURN "high_positive"  // Excited, Happy
    ELSE IF valence >= 0 AND arousal < 0 THEN
        RETURN "low_positive"   // Calm, Relaxed
    ELSE IF valence < 0 AND arousal >= 0 THEN
        RETURN "high_negative"  // Anxious, Stressed
    ELSE
        RETURN "low_negative"   // Sad, Depressed
    END IF
END
```

---

## Semantic Search Integration

```
ALGORITHM: SearchByTransition
INPUT:
    transitionVector (FLOAT32ARRAY[1536])
    topK (INTEGER)
OUTPUT: candidates (ARRAY<SearchCandidate>)

BEGIN
    // Query RuVector for similar content embeddings
    searchResults ← RuVectorClient.search({
        collectionName: "emotistream_content",
        vector: transitionVector,
        limit: topK,
        filter: {
            // Optional: Filter by platform, duration, etc.
            isActive: true
        }
    })

    candidates ← []
    FOR EACH result IN searchResults DO
        // Load full content profile
        profile ← LoadContentProfile(result.id)

        // Convert distance to similarity score [0, 1]
        // Assuming cosine distance in [0, 2], convert to similarity
        similarity ← 1.0 - (result.distance / 2.0)
        similarity ← MAX(0.0, MIN(1.0, similarity))

        candidate ← SearchCandidate {
            contentId: result.id,
            profile: profile,
            similarity: similarity,
            distance: result.distance
        }

        candidates.APPEND(candidate)
    END FOR

    RETURN candidates
END
```

---

## Hybrid Re-Ranking with Q-Values

```
ALGORITHM: RerankWithQValues
INPUT:
    userId (STRING)
    candidates (ARRAY<SearchCandidate>)
    stateHash (STRING)
    desiredState (DesiredState)
OUTPUT: rankedCandidates (ARRAY<RankedCandidate>)

CONSTANTS:
    Q_WEIGHT = 0.7              // 70% weight to Q-value
    SIMILARITY_WEIGHT = 0.3     // 30% weight to similarity
    DEFAULT_Q_VALUE = 0.5       // For unexplored state-action pairs

BEGIN
    rankedCandidates ← []

    FOR EACH candidate IN candidates DO
        // Construct state-action key for Q-table lookup
        actionKey ← ConstructActionKey(candidate.contentId, candidate.profile)

        // Retrieve Q-value from AgentDB (RL policy)
        qValue ← AgentDB.getQValue(userId, stateHash, actionKey)

        IF qValue is NULL THEN
            // Unexplored state-action pair
            qValue ← DEFAULT_Q_VALUE

            // Bonus for exploration (slight preference for new content)
            explorationBonus ← 0.1
            qValue ← qValue + explorationBonus
        END IF

        // Normalize Q-value to [0, 1] if needed
        qValueNormalized ← NormalizeQValue(qValue)

        // Calculate hybrid score
        hybridScore ← (qValueNormalized * Q_WEIGHT) +
                      (candidate.similarity * SIMILARITY_WEIGHT)

        // Adjust score based on desired outcome alignment
        outcomeAlignment ← CalculateOutcomeAlignment(
            candidate.profile,
            desiredState
        )
        hybridScore ← hybridScore * outcomeAlignment

        rankedCandidate ← {
            contentId: candidate.contentId,
            profile: candidate.profile,
            similarity: candidate.similarity,
            qValue: qValue,
            qValueNormalized: qValueNormalized,
            hybridScore: hybridScore,
            isExploration: (qValue = DEFAULT_Q_VALUE)
        }

        rankedCandidates.APPEND(rankedCandidate)
    END FOR

    // Sort by hybrid score descending
    rankedCandidates.SORT_BY(hybridScore, DESCENDING)

    RETURN rankedCandidates
END

SUBROUTINE: ConstructActionKey
INPUT: contentId (STRING), profile (EmotionalContentProfile)
OUTPUT: actionKey (STRING)

BEGIN
    // Create deterministic key for Q-table
    // Format: "content:{id}:valence:{delta}:arousal:{delta}"

    actionKey ← "content:" + contentId +
                ":v:" + ROUND(profile.valenceDelta, 2) +
                ":a:" + ROUND(profile.arousalDelta, 2)

    RETURN actionKey
END

SUBROUTINE: NormalizeQValue
INPUT: qValue (FLOAT)
OUTPUT: normalized (FLOAT)

BEGIN
    // Assuming Q-values are in range [-1, 1] from RL training
    // Normalize to [0, 1] for scoring

    normalized ← (qValue + 1.0) / 2.0
    normalized ← MAX(0.0, MIN(1.0, normalized))

    RETURN normalized
END

SUBROUTINE: CalculateOutcomeAlignment
INPUT:
    profile (EmotionalContentProfile)
    desiredState (DesiredState)
OUTPUT: alignmentScore (FLOAT)

BEGIN
    // Calculate how well content's emotional delta aligns with desired transition

    // Desired deltas (implicit from current state tracking)
    desiredValenceDelta ← desiredState.valence  // Simplified assumption
    desiredArousalDelta ← desiredState.arousal

    // Calculate alignment using cosine similarity of delta vectors
    dotProduct ← (profile.valenceDelta * desiredValenceDelta) +
                 (profile.arousalDelta * desiredArousalDelta)

    magnitudeProfile ← SQRT(
        (profile.valenceDelta ^ 2) + (profile.arousalDelta ^ 2)
    )

    magnitudeDesired ← SQRT(
        (desiredValenceDelta ^ 2) + (desiredArousalDelta ^ 2)
    )

    IF magnitudeProfile = 0 OR magnitudeDesired = 0 THEN
        RETURN 0.5  // Neutral alignment
    END IF

    // Cosine similarity in [-1, 1]
    cosineSimilarity ← dotProduct / (magnitudeProfile * magnitudeDesired)

    // Convert to [0, 1] with 0.5 as neutral
    alignmentScore ← (cosineSimilarity + 1.0) / 2.0

    // Boost if alignment is strong
    IF alignmentScore > 0.8 THEN
        alignmentScore ← 1.0 + ((alignmentScore - 0.8) * 0.5)  // Up to 1.1x boost
    END IF

    RETURN alignmentScore
END
```

---

## Desired State Prediction

```
ALGORITHM: PredictDesiredState
INPUT: currentState (EmotionalState)
OUTPUT: desiredState (DesiredState with valence, arousal)

BEGIN
    // Rule-based heuristics for emotional regulation goals

    // Rule 1: High stress → calm down
    IF currentState.stressLevel > 0.6 THEN
        RETURN {
            valence: 0.5,      // Mildly positive
            arousal: -0.4,     // Calm
            reasoning: "stress_reduction"
        }
    END IF

    // Rule 2: Sad (low valence, low arousal) → uplifting
    IF currentState.valence < -0.3 AND currentState.arousal < -0.2 THEN
        RETURN {
            valence: 0.6,      // Positive
            arousal: 0.4,      // Energized
            reasoning: "mood_lift"
        }
    END IF

    // Rule 3: Anxious (high arousal, negative valence) → grounding
    IF currentState.arousal > 0.5 AND currentState.valence < 0 THEN
        RETURN {
            valence: 0.3,      // Slightly positive
            arousal: -0.3,     // Calm
            reasoning: "anxiety_reduction"
        }
    END IF

    // Rule 4: Bored (neutral, low arousal) → stimulation
    IF ABS(currentState.valence) < 0.2 AND currentState.arousal < -0.4 THEN
        RETURN {
            valence: 0.5,      // Positive
            arousal: 0.5,      // Energized
            reasoning: "stimulation"
        }
    END IF

    // Rule 5: Overstimulated (high arousal, positive) → maintain but calm slightly
    IF currentState.arousal > 0.6 AND currentState.valence > 0.3 THEN
        RETURN {
            valence: currentState.valence,
            arousal: currentState.arousal - 0.3,  // Reduce arousal slightly
            reasoning: "arousal_regulation"
        }
    END IF

    // Default: Maintain homeostasis with slight positive bias
    RETURN {
        valence: MAX(currentState.valence, 0.2),  // Slight positive bias
        arousal: currentState.arousal * 0.8,       // Slight calming
        reasoning: "homeostasis"
    }
END
```

---

## Outcome Prediction

```
ALGORITHM: PredictOutcome
INPUT:
    currentState (EmotionalState)
    contentProfile (EmotionalContentProfile)
OUTPUT: outcome (PredictedOutcome)

BEGIN
    // Predict post-viewing emotional state
    postValence ← currentState.valence + contentProfile.valenceDelta
    postArousal ← currentState.arousal + contentProfile.arousalDelta
    postStress ← MAX(0.0, currentState.stressLevel - contentProfile.stressReduction)

    // Clamp to valid ranges
    postValence ← CLAMP(postValence, -1.0, 1.0)
    postArousal ← CLAMP(postArousal, -1.0, 1.0)
    postStress ← CLAMP(postStress, 0.0, 1.0)

    // Calculate confidence based on historical data
    // High confidence if content has been watched many times with consistent outcomes
    watchCount ← contentProfile.totalWatches || 0
    outcomeVariance ← contentProfile.outcomeVariance || 1.0

    // Confidence increases with watch count, decreases with variance
    confidence ← (1.0 - EXP(-watchCount / 20.0)) * (1.0 - outcomeVariance)
    confidence ← MAX(0.1, MIN(0.95, confidence))  // [0.1, 0.95]

    outcome ← {
        postViewingValence: postValence,
        postViewingArousal: postArousal,
        postViewingStress: postStress,
        confidence: confidence
    }

    RETURN outcome
END
```

---

## Reasoning Generation

```
ALGORITHM: GenerateReasoning
INPUT:
    currentState (EmotionalState)
    desiredState (DesiredState)
    contentProfile (EmotionalContentProfile)
    qValue (FLOAT)
    isExploration (BOOLEAN)
OUTPUT: reasoning (STRING)

BEGIN
    reasoning ← ""

    // Part 1: Current emotional context
    currentDesc ← DescribeEmotionalState(
        currentState.valence,
        currentState.arousal,
        currentState.stressLevel
    )
    reasoning ← "You're currently feeling " + currentDesc + ". "

    // Part 2: Desired transition
    desiredDesc ← DescribeEmotionalState(
        desiredState.valence,
        desiredState.arousal,
        0  // Stress not part of desired state
    )
    reasoning ← reasoning + "This content will help you transition to feeling " +
                desiredDesc + ". "

    // Part 3: Expected emotional changes
    IF contentProfile.valenceDelta > 0.2 THEN
        reasoning ← reasoning + "It should improve your mood significantly. "
    ELSE IF contentProfile.valenceDelta < -0.2 THEN
        reasoning ← reasoning + "It may be emotionally intense. "
    END IF

    IF contentProfile.arousalDelta > 0.3 THEN
        reasoning ← reasoning + "Expect to feel more energized and alert. "
    ELSE IF contentProfile.arousalDelta < -0.3 THEN
        reasoning ← reasoning + "It will help you relax and unwind. "
    END IF

    IF contentProfile.stressReduction > 0.5 THEN
        reasoning ← reasoning + "Great for stress relief. "
    END IF

    // Part 4: Recommendation confidence
    IF qValue > 0.7 THEN
        reasoning ← reasoning + "Users in similar emotional states loved this content. "
    ELSE IF qValue < 0.3 THEN
        reasoning ← reasoning + "This is a personalized experimental pick. "
    ELSE
        reasoning ← reasoning + "This matches your emotional needs well. "
    END IF

    // Part 5: Exploration flag
    IF isExploration THEN
        reasoning ← reasoning + "(New discovery for you!)"
    END IF

    RETURN reasoning
END
```

---

## Filtering & Exploration

```
ALGORITHM: FilterWatchedContent
INPUT:
    userId (STRING)
    candidates (ARRAY<SearchCandidate>)
OUTPUT: filtered (ARRAY<SearchCandidate>)

BEGIN
    // Load user's watch history from AgentDB
    watchHistory ← AgentDB.query({
        namespace: "emotistream/watch_history",
        userId: userId,
        limit: 1000  // Recent watches
    })

    watchedContentIds ← SET()
    FOR EACH record IN watchHistory DO
        watchedContentIds.ADD(record.contentId)
    END FOR

    filtered ← []
    FOR EACH candidate IN candidates DO
        // Allow re-recommendations if watched >30 days ago
        lastWatchTime ← GetLastWatchTime(candidate.contentId, watchHistory)

        IF candidate.contentId NOT IN watchedContentIds THEN
            filtered.APPEND(candidate)
        ELSE IF lastWatchTime < (NOW() - 30_DAYS) THEN
            // Re-recommend if enough time has passed
            filtered.APPEND(candidate)
        END IF
    END FOR

    RETURN filtered
END

ALGORITHM: ApplyExploration
INPUT:
    rankedCandidates (ARRAY<RankedCandidate>)
    explorationRate (FLOAT)  // e.g., 0.1 for 10%
OUTPUT: exploredCandidates (ARRAY<RankedCandidate>)

BEGIN
    // Epsilon-greedy exploration strategy
    totalCount ← LENGTH(rankedCandidates)
    explorationCount ← FLOOR(totalCount * explorationRate)

    exploredCandidates ← []
    explorationInserted ← 0

    FOR i ← 0 TO totalCount - 1 DO
        // Insert exploration candidates at random positions
        IF explorationInserted < explorationCount THEN
            shouldExplore ← RANDOM() < explorationRate

            IF shouldExplore THEN
                // Pick random candidate from lower-ranked items
                explorationIndex ← RANDOM_INT(
                    totalCount * 0.5,  // Start from middle
                    totalCount - 1
                )

                explorationCandidate ← rankedCandidates[explorationIndex]
                explorationCandidate.isExploration ← true
                explorationCandidate.hybridScore ←
                    explorationCandidate.hybridScore + 0.2  // Boost score

                exploredCandidates.APPEND(explorationCandidate)
                explorationInserted ← explorationInserted + 1
                CONTINUE
            END IF
        END IF

        exploredCandidates.APPEND(rankedCandidates[i])
    END FOR

    // Re-sort after exploration injection
    exploredCandidates.SORT_BY(hybridScore, DESCENDING)

    RETURN exploredCandidates
END
```

---

## Helper Functions

```
ALGORITHM: HashEmotionalState
INPUT: state (EmotionalState)
OUTPUT: stateHash (STRING)

BEGIN
    // Discretize continuous values for Q-table lookup
    valenceBucket ← FLOOR((state.valence + 1.0) / 0.2)  // 10 buckets
    arousalBucket ← FLOOR((state.arousal + 1.0) / 0.2)  // 10 buckets
    stressBucket ← FLOOR(state.stressLevel / 0.2)       // 5 buckets

    stateHash ← "v:" + valenceBucket +
                ":a:" + arousalBucket +
                ":s:" + stressBucket

    RETURN stateHash
END

ALGORITHM: LoadEmotionalState
INPUT: emotionalStateId (STRING)
OUTPUT: state (EmotionalState) or NULL

BEGIN
    state ← AgentDB.get({
        namespace: "emotistream/emotional_states",
        key: emotionalStateId
    })

    IF state is NULL THEN
        RETURN NULL
    END IF

    RETURN state
END

ALGORITHM: LoadContentProfile
INPUT: contentId (STRING)
OUTPUT: profile (EmotionalContentProfile)

BEGIN
    profile ← AgentDB.get({
        namespace: "emotistream/content_profiles",
        key: contentId
    })

    IF profile is NULL THEN
        // Fallback to default profile
        profile ← CreateDefaultProfile(contentId)
    END IF

    RETURN profile
END

ALGORITHM: LogRecommendationEvent
INPUT:
    userId (STRING)
    currentState (EmotionalState)
    recommendations (ARRAY<EmotionalRecommendation>)
OUTPUT: void

BEGIN
    event ← {
        userId: userId,
        timestamp: NOW(),
        emotionalStateId: currentState.id,
        currentValence: currentState.valence,
        currentArousal: currentState.arousal,
        currentStress: currentState.stressLevel,
        recommendedContentIds: MAP(recommendations, r → r.contentId),
        topRecommendation: recommendations[0].contentId
    }

    AgentDB.store({
        namespace: "emotistream/recommendation_events",
        key: "rec:" + userId + ":" + NOW(),
        value: event,
        ttl: 90_DAYS
    })
END
```

---

## Integration Points

### RLPolicyEngine Integration

```
INTEGRATION: RecommendationEngine ↔ RLPolicyEngine

1. Q-Value Retrieval:
   - RecommendationEngine.rerankWithQValues()
     → RLPolicyEngine.getQValue(userId, stateHash, actionKey)
   - Used during hybrid ranking (70% weight)

2. Learning Feedback Loop:
   - User watches content → EmotionalStateEngine tracks outcome
   - Outcome → RLPolicyEngine.updateQValue() via reward signal
   - Updated Q-values → Future RecommendationEngine queries

3. Exploration Strategy:
   - RecommendationEngine.applyExploration() uses epsilon-greedy
   - Aligns with RLPolicyEngine's exploration rate parameter
```

### RuVector Integration

```
INTEGRATION: RecommendationEngine ↔ RuVector

1. Semantic Search:
   - RecommendationEngine.createTransitionVector()
     → Generate 1536D embedding
   - RecommendationEngine.searchByTransition()
     → RuVector.search(vector, topK=50)
   - Returns semantically similar content

2. Content Ingestion:
   - New content → Emotional profiling
   - Profile → Generate embedding via OpenAI/Voyage
   - RuVector.upsert(contentId, embedding, metadata)

3. Metadata Filtering:
   - RuVector search can filter by:
     - platform (Netflix, YouTube)
     - duration (for time constraints)
     - isActive (content availability)
```

### AgentDB Integration

```
INTEGRATION: RecommendationEngine ↔ AgentDB

1. Watch History Tracking:
   - Namespace: "emotistream/watch_history"
   - Used by filterWatchedContent()
   - Prevents redundant recommendations

2. Content Profiles:
   - Namespace: "emotistream/content_profiles"
   - Stores EmotionalContentProfile for each piece of content
   - Updated as more viewing data accumulates

3. Recommendation Events:
   - Namespace: "emotistream/recommendation_events"
   - Logs all recommendation sessions
   - Used for analytics and debugging
```

---

## Example Recommendation Flow

### Scenario: Stressed User Seeking Relaxation

```
EXAMPLE FLOW:

INPUT:
    userId: "user123"
    emotionalStateId: "state_2024_001"
    Current State:
        valence: -0.3    (slightly negative)
        arousal: 0.6     (high energy/anxiety)
        stressLevel: 0.8 (very stressed)

STEP 1: Predict Desired State
    → PredictDesiredState(currentState)
    → stressLevel > 0.6 triggers "stress_reduction" rule
    → desiredState = { valence: 0.5, arousal: -0.4 }

STEP 2: Create Transition Vector
    → valenceDelta = 0.5 - (-0.3) = 0.8
    → arousalDelta = -0.4 - 0.6 = -1.0
    → Prompt: "Find content that transitions from stressed anxious
               (valence: -0.3, arousal: 0.6) to calm content
               (valence: 0.5, arousal: -0.4).
               Need stress reduction of 0.8. Suitable for evening viewing."
    → Embedding: [0.023, -0.156, 0.089, ..., 0.234] (1536D)

STEP 3: Search RuVector
    → RuVector.search(vector, topK=50)
    → Results:
        1. "Planet Earth II" (nature documentary)
           - similarity: 0.89
           - valenceDelta: +0.7, arousalDelta: -0.6

        2. "The Great British Baking Show"
           - similarity: 0.85
           - valenceDelta: +0.5, arousalDelta: -0.4

        3. "Meditation for Beginners"
           - similarity: 0.83
           - valenceDelta: +0.4, arousalDelta: -0.8

        ... (47 more)

STEP 4: Filter Watched Content
    → Check watch_history for user123
    → "Planet Earth II" watched 45 days ago → KEEP
    → "Baking Show S01E01" watched 2 days ago → REMOVE
    → 42 candidates remain

STEP 5: Re-rank with Q-Values
    → For "Planet Earth II":
        - stateHash: "v:3:a:8:s:4" (discretized current state)
        - actionKey: "content:planet_earth_ii:v:0.7:a:-0.6"
        - qValue: 0.82 (high past success)
        - qValueNormalized: (0.82 + 1.0) / 2.0 = 0.91
        - hybridScore: (0.91 * 0.7) + (0.89 * 0.3) = 0.637 + 0.267 = 0.904

    → For "Meditation for Beginners":
        - qValue: 0.5 (unexplored, default)
        - qValueNormalized: 0.75
        - hybridScore: (0.75 * 0.7) + (0.83 * 0.3) = 0.525 + 0.249 = 0.774

    → Ranked order:
        1. "Planet Earth II" (score: 0.904)
        2. "Headspace: Guide to Meditation" (score: 0.856)
        3. "Chef's Table" (score: 0.798)
        ...

STEP 6: Predict Outcomes
    → "Planet Earth II":
        - postValence: -0.3 + 0.7 = 0.4
        - postArousal: 0.6 + (-0.6) = 0.0
        - postStress: 0.8 - 0.7 = 0.1
        - confidence: 0.85 (watched 120 times, low variance)

STEP 7: Generate Reasoning
    → "You're currently feeling stressed anxious.
       This content will help you transition to feeling calm content.
       It will help you relax and unwind.
       Great for stress relief.
       Users in similar emotional states loved this content."

FINAL OUTPUT:
[
  {
    contentId: "planet_earth_ii",
    title: "Planet Earth II",
    platform: "Netflix",
    emotionalProfile: {
      valenceDelta: 0.7,
      arousalDelta: -0.6,
      stressReduction: 0.7,
      duration: 50
    },
    predictedOutcome: {
      postViewingValence: 0.4,
      postViewingArousal: 0.0,
      postViewingStress: 0.1,
      confidence: 0.85
    },
    qValue: 0.82,
    similarityScore: 0.89,
    hybridScore: 0.904,
    isExploration: false,
    rank: 1,
    reasoning: "You're currently feeling stressed anxious. This content..."
  },
  ... (19 more recommendations)
]
```

---

## Complexity Analysis

### Time Complexity

**recommend() Overall:**
- Load emotional state: **O(1)** (AgentDB key lookup)
- Predict desired state: **O(1)** (rule evaluation)
- Create transition vector: **O(1)** (embedding API call, async)
- RuVector search: **O(log n)** where n = total content count (HNSW index)
- Filter watched content: **O(k)** where k = candidate count (~50)
- Re-rank with Q-values: **O(k log k)** (k lookups + sort)
- Generate recommendations: **O(m)** where m = limit (20)
- **Total: O(k log k)** dominated by re-ranking sort

**Space Complexity:**
- Transition vector: **O(1)** (fixed 1536D)
- Search candidates: **O(k)** (~50 items)
- Ranked results: **O(k)**
- Final recommendations: **O(m)** (20 items)
- **Total: O(k)** where k is constant (50)

### Optimization Opportunities

1. **Batch Q-Value Lookups**: Retrieve all Q-values in single AgentDB query
2. **Cache Content Profiles**: LRU cache for frequently recommended content
3. **Approximate Search**: Use RuVector's quantization for faster search
4. **Precompute Embeddings**: Cache transition vectors for common state patterns
5. **Parallel Ranking**: Score candidates concurrently using Promise.all()

---

## Edge Cases & Error Handling

### Edge Case 1: No Similar Content Found
```
IF LENGTH(searchResults) = 0 THEN
    // Fallback to popular content in desired emotional quadrant
    fallbackResults ← GetPopularContentByQuadrant(desiredState)
    RETURN fallbackResults
END IF
```

### Edge Case 2: All Content Already Watched
```
IF LENGTH(filteredCandidates) = 0 THEN
    // Allow re-recommendations with lower threshold
    filteredCandidates ← FilterWatchedContent(userId, candidates,
                                               minDaysSinceWatch: 7)
END IF
```

### Edge Case 3: Extreme Emotional States
```
IF ABS(currentState.valence) > 0.9 OR ABS(currentState.arousal) > 0.9 THEN
    // More conservative recommendations
    // Avoid content with extreme deltas
    candidates ← FilterByDeltaMagnitude(candidates, maxDelta: 0.4)
END IF
```

### Edge Case 4: New User (Cold Start)
```
IF watchHistory is EMPTY THEN
    // Use content-based filtering only (no Q-values available)
    // Rely 100% on semantic similarity
    hybridScore ← similarity  // Override hybrid formula
END IF
```

---

## Testing Strategy

### Unit Tests

1. **CreateTransitionVector**:
   - Test prompt generation for all emotional quadrants
   - Verify vector dimensions (1536)
   - Test edge cases (extreme valence/arousal)

2. **PredictDesiredState**:
   - Test all heuristic rules trigger correctly
   - Verify default homeostasis case
   - Test boundary conditions

3. **RerankWithQValues**:
   - Test hybrid scoring formula
   - Verify Q-value normalization
   - Test exploration bonus application

4. **GenerateReasoning**:
   - Test reasoning generation for various state combinations
   - Verify exploration flag inclusion
   - Test edge case descriptions

### Integration Tests

1. **End-to-End Recommendation Flow**:
   - Mock RuVector search results
   - Mock AgentDB Q-value responses
   - Verify final recommendations match expected ranking

2. **RuVector Integration**:
   - Test actual vector search with sample embeddings
   - Verify similarity score conversion
   - Test metadata filtering

3. **AgentDB Integration**:
   - Test watch history retrieval
   - Test Q-value lookups
   - Test recommendation event logging

### Performance Tests

1. **Recommendation Latency**:
   - Target: <500ms for 20 recommendations
   - Test with 1000+ content items in RuVector

2. **Concurrent Requests**:
   - Test 100 concurrent recommendation requests
   - Verify no race conditions in AgentDB/RuVector

3. **Memory Usage**:
   - Monitor memory for large candidate sets
   - Test garbage collection under load

---

## Future Enhancements

1. **Multi-Objective Optimization**:
   - Balance emotional goals with diversity, novelty, serendipity
   - Pareto-optimal recommendations

2. **Temporal Context**:
   - Time-of-day preferences (morning vs. evening)
   - Day-of-week patterns (weekday vs. weekend)

3. **Social Recommendations**:
   - Incorporate social graph for co-watching suggestions
   - Emotional contagion modeling

4. **Hybrid Embeddings**:
   - Combine semantic embeddings with emotional embeddings
   - Multi-vector search in RuVector

5. **Explainable AI**:
   - SHAP values for recommendation explanations
   - Counterfactual explanations ("Why not X?")

---

**Document Version:** 1.0
**Last Updated:** 2025-12-05
**Author:** SPARC Pseudocode Agent
**Status:** Ready for Architecture Phase
