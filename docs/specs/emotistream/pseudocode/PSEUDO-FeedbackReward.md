# Feedback & Reward System - Pseudocode

**SPARC Phase**: Pseudocode
**Component**: Feedback & Reward Processing
**Version**: 1.0.0
**Date**: 2025-12-05

## Table of Contents
1. [Overview](#overview)
2. [Data Structures](#data-structures)
3. [Core Algorithms](#core-algorithms)
4. [Subroutines](#subroutines)
5. [Complexity Analysis](#complexity-analysis)
6. [Integration Points](#integration-points)
7. [Example Calculations](#example-calculations)

---

## Overview

The Feedback & Reward system closes the reinforcement learning loop by:
1. Processing user feedback after content viewing
2. Calculating multi-factor rewards based on emotional state changes
3. Updating Q-values to improve future recommendations
4. Storing experiences for replay learning
5. Tracking user learning progress

**Key Design Principles**:
- Reward based on emotional improvement direction and magnitude
- Support multiple feedback modalities (text, ratings, emojis)
- Maintain experience replay buffer for batch learning
- Decay exploration over time as user preferences stabilize

---

## Data Structures

### Input Types

```
STRUCTURE: FeedbackRequest
    userId: String                          // User identifier
    contentId: String                       // Content identifier
    emotionalStateId: String                // Pre-viewing emotional state ID
    postViewingState: PostViewingState      // User feedback
    viewingDetails: ViewingDetails          // Optional viewing metadata
END STRUCTURE

STRUCTURE: PostViewingState
    text: String (optional)                 // Free-form text feedback
    explicitRating: Integer (optional)      // 1-5 star rating
    explicitEmoji: String (optional)        // Emoji feedback
END STRUCTURE

STRUCTURE: ViewingDetails
    completionRate: Float                   // 0.0-1.0 (percentage watched)
    durationSeconds: Integer                // Total viewing time
    pauseCount: Integer (optional)          // Number of pauses
    skipCount: Integer (optional)           // Number of skips
END STRUCTURE
```

### Output Types

```
STRUCTURE: FeedbackResponse
    experienceId: String                    // Unique experience identifier
    reward: Float                           // Calculated reward (-1.0 to 1.0)
    emotionalImprovement: Float             // Emotional state delta
    qValueBefore: Float                     // Q-value before update
    qValueAfter: Float                      // Q-value after update
    policyUpdated: Boolean                  // Whether RL policy was updated
    message: String                         // User-friendly feedback message
    insights: FeedbackInsights              // Additional analytics
END STRUCTURE

STRUCTURE: FeedbackInsights
    directionAlignment: Float               // Alignment with desired direction
    magnitudeScore: Float                   // Improvement magnitude
    proximityBonus: Float                   // Bonus for reaching target
    completionBonus: Float                  // Bonus for full viewing
END STRUCTURE
```

### Internal Types

```
STRUCTURE: EmotionalState
    valence: Float                          // -1.0 to 1.0 (negative to positive)
    arousal: Float                          // -1.0 to 1.0 (calm to excited)
    dominance: Float                        // -1.0 to 1.0 (submissive to dominant)
    confidence: Float                       // 0.0 to 1.0
    timestamp: DateTime
END STRUCTURE

STRUCTURE: EmotionalExperience
    experienceId: String
    userId: String
    contentId: String
    stateBeforeId: String
    stateAfter: EmotionalState
    desiredState: EmotionalState
    reward: Float
    qValueBefore: Float
    qValueAfter: Float
    timestamp: DateTime
    metadata: Object
END STRUCTURE

STRUCTURE: UserProfile
    userId: String
    totalExperiences: Integer
    avgReward: Float
    explorationRate: Float
    preferredGenres: Array<String>
    learningProgress: Float
END STRUCTURE
```

---

## Core Algorithms

### Algorithm 1: Process Feedback

```
ALGORITHM: ProcessFeedback
INPUT: request (FeedbackRequest)
OUTPUT: response (FeedbackResponse)

CONSTANTS:
    MIN_REWARD = -1.0
    MAX_REWARD = 1.0
    LEARNING_RATE = 0.1
    EXPLORATION_DECAY = 0.99

BEGIN
    // Step 1: Validate input
    IF NOT ValidateFeedbackRequest(request) THEN
        THROW ValidationError("Invalid feedback request")
    END IF

    // Step 2: Retrieve pre-viewing emotional state
    stateBeforeId ← request.emotionalStateId
    stateBefore ← EmotionalStateStore.get(stateBeforeId)

    IF stateBefore IS NULL THEN
        THROW NotFoundError("Pre-viewing state not found")
    END IF

    // Step 3: Get desired emotional state from recommendation
    recommendation ← RecommendationStore.get(request.userId, request.contentId)
    desiredState ← recommendation.targetEmotionalState

    // Step 4: Analyze post-viewing emotional state
    stateAfter ← NULL

    IF request.postViewingState.text IS NOT NULL THEN
        // Text-based feedback (most accurate)
        stateAfter ← AnalyzePostViewingState(request.postViewingState.text)
    ELSE IF request.postViewingState.explicitRating IS NOT NULL THEN
        // Explicit rating (less granular)
        stateAfter ← ConvertExplicitRating(request.postViewingState.explicitRating)
    ELSE IF request.postViewingState.explicitEmoji IS NOT NULL THEN
        // Emoji feedback (least granular)
        stateAfter ← ConvertEmojiToState(request.postViewingState.explicitEmoji)
    ELSE
        THROW ValidationError("No post-viewing feedback provided")
    END IF

    // Step 5: Calculate multi-factor reward
    baseReward ← CalculateReward(stateBefore, stateAfter, desiredState)

    // Step 6: Apply viewing behavior modifiers
    completionBonus ← 0.0
    IF request.viewingDetails IS NOT NULL THEN
        completionBonus ← CalculateCompletionBonus(request.viewingDetails)
    END IF

    finalReward ← CLAMP(baseReward + completionBonus, MIN_REWARD, MAX_REWARD)

    // Step 7: Get current Q-value
    rlEngine ← RLPolicyEngine.getInstance()
    qValueBefore ← rlEngine.getQValue(stateBefore, request.contentId)

    // Step 8: Update Q-value using Q-learning update rule
    // Q(s,a) ← Q(s,a) + α[r + γ max Q(s',a') - Q(s,a)]
    // For terminal state (post-viewing), γ max Q(s',a') = 0
    qValueAfter ← qValueBefore + LEARNING_RATE * (finalReward - qValueBefore)

    success ← rlEngine.updateQValue(stateBefore, request.contentId, qValueAfter)

    // Step 9: Store experience for replay learning
    experienceId ← GenerateUUID()
    experience ← EmotionalExperience {
        experienceId: experienceId,
        userId: request.userId,
        contentId: request.contentId,
        stateBeforeId: stateBeforeId,
        stateAfter: stateAfter,
        desiredState: desiredState,
        reward: finalReward,
        qValueBefore: qValueBefore,
        qValueAfter: qValueAfter,
        timestamp: CurrentDateTime(),
        metadata: {
            viewingDetails: request.viewingDetails,
            feedbackType: DetermineFeedbackType(request.postViewingState)
        }
    }

    StoreExperience(experience)

    // Step 10: Update user profile and learning progress
    UpdateUserProfile(request.userId, finalReward)

    // Step 11: Calculate emotional improvement metric
    emotionalImprovement ← CalculateEmotionalImprovement(
        stateBefore,
        stateAfter,
        desiredState
    )

    // Step 12: Generate user-friendly feedback message
    message ← GenerateFeedbackMessage(finalReward, emotionalImprovement)

    // Step 13: Compile detailed insights
    insights ← CalculateFeedbackInsights(
        stateBefore,
        stateAfter,
        desiredState,
        completionBonus
    )

    // Step 14: Return comprehensive response
    RETURN FeedbackResponse {
        experienceId: experienceId,
        reward: finalReward,
        emotionalImprovement: emotionalImprovement,
        qValueBefore: qValueBefore,
        qValueAfter: qValueAfter,
        policyUpdated: success,
        message: message,
        insights: insights
    }
END
```

**Complexity Analysis**:
- Time: O(1) - Constant time operations (DB lookups, arithmetic)
- Space: O(1) - Fixed-size data structures
- Database Operations: 5 reads, 3 writes

---

### Algorithm 2: Calculate Reward

```
ALGORITHM: CalculateReward
INPUT:
    stateBefore (EmotionalState)      // Pre-viewing state
    stateAfter (EmotionalState)       // Post-viewing state
    desiredState (EmotionalState)     // Target state
OUTPUT: reward (Float)                // Range: -1.0 to 1.0

CONSTANTS:
    DIRECTION_WEIGHT = 0.6            // 60% weight for alignment
    MAGNITUDE_WEIGHT = 0.4            // 40% weight for magnitude
    MAX_PROXIMITY_BONUS = 0.2         // Maximum bonus for reaching target
    NORMALIZATION_FACTOR = 2.0        // For magnitude scaling

BEGIN
    // Component 1: Direction Alignment (60% weight)
    // Measures if emotion moved in the desired direction
    // Uses cosine similarity between actual and desired vectors

    // Calculate actual emotional change vector
    actualDelta ← Vector {
        valence: stateAfter.valence - stateBefore.valence,
        arousal: stateAfter.arousal - stateBefore.arousal
    }

    // Calculate desired emotional change vector
    desiredDelta ← Vector {
        valence: desiredState.valence - stateBefore.valence,
        arousal: desiredState.arousal - stateBefore.arousal
    }

    // Cosine similarity: cos(θ) = (A·B) / (|A||B|)
    dotProduct ← (actualDelta.valence * desiredDelta.valence) +
                 (actualDelta.arousal * desiredDelta.arousal)

    actualMagnitude ← SQRT(actualDelta.valence² + actualDelta.arousal²)
    desiredMagnitude ← SQRT(desiredDelta.valence² + desiredDelta.arousal²)

    IF actualMagnitude = 0 OR desiredMagnitude = 0 THEN
        // No change or no desired change
        directionAlignment ← 0.0
    ELSE
        directionAlignment ← dotProduct / (actualMagnitude * desiredMagnitude)
        directionAlignment ← CLAMP(directionAlignment, -1.0, 1.0)
    END IF

    // Component 2: Improvement Magnitude (40% weight)
    // Measures the size of emotional change

    magnitude ← SQRT(actualDelta.valence² + actualDelta.arousal²)
    normalizedMagnitude ← MIN(1.0, magnitude / NORMALIZATION_FACTOR)

    // Component 3: Proximity Bonus (up to +0.2)
    // Rewards getting close to the desired state

    distance ← SQRT(
        (stateAfter.valence - desiredState.valence)² +
        (stateAfter.arousal - desiredState.arousal)²
    )

    proximityBonus ← MAX(0.0, MAX_PROXIMITY_BONUS * (1.0 - distance / 2.0))

    // Final Reward Calculation
    baseReward ← (directionAlignment * DIRECTION_WEIGHT) +
                 (normalizedMagnitude * MAGNITUDE_WEIGHT)

    finalReward ← baseReward + proximityBonus

    // Clamp to valid range
    finalReward ← CLAMP(finalReward, -1.0, 1.0)

    RETURN finalReward
END
```

**Complexity Analysis**:
- Time: O(1) - Fixed arithmetic operations
- Space: O(1) - Small fixed variables
- Numerical Stability: Uses SQRT and division with zero checks

---

### Algorithm 3: Calculate Completion Bonus

```
ALGORITHM: CalculateCompletionBonus
INPUT: viewingDetails (ViewingDetails)
OUTPUT: bonus (Float)                      // Range: -0.2 to 0.2

CONSTANTS:
    MAX_COMPLETION_BONUS = 0.2
    MIN_ACCEPTABLE_COMPLETION = 0.8
    PAUSE_PENALTY_FACTOR = 0.01
    SKIP_PENALTY_FACTOR = 0.02

BEGIN
    bonus ← 0.0

    // Completion rate bonus/penalty
    IF viewingDetails.completionRate >= MIN_ACCEPTABLE_COMPLETION THEN
        // Full viewing is a positive signal
        completionBonus ← MAX_COMPLETION_BONUS * viewingDetails.completionRate
        bonus ← bonus + completionBonus
    ELSE IF viewingDetails.completionRate < 0.3 THEN
        // Very low completion is a strong negative signal
        penalty ← -MAX_COMPLETION_BONUS * (1.0 - viewingDetails.completionRate)
        bonus ← bonus + penalty
    ELSE
        // Moderate completion: neutral to slightly negative
        penalty ← -MAX_COMPLETION_BONUS * 0.5 * (1.0 - viewingDetails.completionRate)
        bonus ← bonus + penalty
    END IF

    // Pause count penalty (frequent pausing suggests disengagement)
    IF viewingDetails.pauseCount IS NOT NULL THEN
        pausePenalty ← MIN(0.1, viewingDetails.pauseCount * PAUSE_PENALTY_FACTOR)
        bonus ← bonus - pausePenalty
    END IF

    // Skip count penalty (skipping suggests poor content match)
    IF viewingDetails.skipCount IS NOT NULL THEN
        skipPenalty ← MIN(0.15, viewingDetails.skipCount * SKIP_PENALTY_FACTOR)
        bonus ← bonus - skipPenalty
    END IF

    // Clamp to maximum bonus range
    bonus ← CLAMP(bonus, -MAX_COMPLETION_BONUS, MAX_COMPLETION_BONUS)

    RETURN bonus
END
```

**Complexity Analysis**:
- Time: O(1)
- Space: O(1)

---

## Subroutines

### Subroutine 1: Analyze Post-Viewing State

```
SUBROUTINE: AnalyzePostViewingState
INPUT: feedbackText (String)
OUTPUT: emotionalState (EmotionalState)

BEGIN
    // Use EmotionDetector service (Gemini-powered)
    emotionDetector ← EmotionDetector.getInstance()

    TRY
        // Call async emotion analysis
        analysisResult ← AWAIT emotionDetector.analyzeText(feedbackText)

        // Extract emotional dimensions
        emotionalState ← EmotionalState {
            valence: analysisResult.valence,
            arousal: analysisResult.arousal,
            dominance: analysisResult.dominance,
            confidence: analysisResult.confidence,
            timestamp: CurrentDateTime()
        }

        RETURN emotionalState

    CATCH error
        // Fallback to neutral state if analysis fails
        LOG_ERROR("Emotion analysis failed", error)

        RETURN EmotionalState {
            valence: 0.0,
            arousal: 0.0,
            dominance: 0.0,
            confidence: 0.0,
            timestamp: CurrentDateTime()
        }
    END TRY
END
```

---

### Subroutine 2: Convert Explicit Rating

```
SUBROUTINE: ConvertExplicitRating
INPUT: rating (Integer)               // 1-5 stars
OUTPUT: emotionalState (EmotionalState)

BEGIN
    // Rating-to-emotion mapping based on research
    // Higher ratings = higher valence, lower arousal (content satisfaction)

    SWITCH rating
        CASE 1:
            // Very negative, somewhat aroused (frustration)
            valence ← -0.8
            arousal ← 0.3
            dominance ← -0.3

        CASE 2:
            // Somewhat negative, slightly aroused (disappointment)
            valence ← -0.4
            arousal ← 0.1
            dominance ← -0.1

        CASE 3:
            // Neutral state
            valence ← 0.0
            arousal ← 0.0
            dominance ← 0.0

        CASE 4:
            // Somewhat positive, calm (satisfied)
            valence ← 0.4
            arousal ← -0.1
            dominance ← 0.1

        CASE 5:
            // Very positive, calm (very satisfied)
            valence ← 0.8
            arousal ← -0.2
            dominance ← 0.2

        DEFAULT:
            // Invalid rating: default to neutral
            valence ← 0.0
            arousal ← 0.0
            dominance ← 0.0
    END SWITCH

    RETURN EmotionalState {
        valence: valence,
        arousal: arousal,
        dominance: dominance,
        confidence: 0.6,              // Lower confidence than text analysis
        timestamp: CurrentDateTime()
    }
END
```

---

### Subroutine 3: Convert Emoji to State

```
SUBROUTINE: ConvertEmojiToState
INPUT: emoji (String)
OUTPUT: emotionalState (EmotionalState)

CONSTANTS:
    // Emoji mappings based on common interpretations
    EMOJI_MAPPINGS = {
        "😊": {valence: 0.7, arousal: -0.2, dominance: 0.2},   // Happy, calm
        "😄": {valence: 0.8, arousal: 0.3, dominance: 0.3},    // Very happy, excited
        "😢": {valence: -0.6, arousal: -0.3, dominance: -0.4}, // Sad, low energy
        "😭": {valence: -0.8, arousal: 0.2, dominance: -0.5},  // Very sad, crying
        "😡": {valence: -0.7, arousal: 0.8, dominance: 0.4},   // Angry, high arousal
        "😌": {valence: 0.5, arousal: -0.6, dominance: 0.1},   // Peaceful, relaxed
        "😴": {valence: 0.2, arousal: -0.8, dominance: -0.3},  // Sleepy, very calm
        "😐": {valence: 0.0, arousal: 0.0, dominance: 0.0},    // Neutral
        "👍": {valence: 0.6, arousal: 0.1, dominance: 0.2},    // Approval
        "👎": {valence: -0.6, arousal: 0.1, dominance: -0.2},  // Disapproval
        "❤️": {valence: 0.9, arousal: 0.2, dominance: 0.3},    // Love, positive
        "💔": {valence: -0.8, arousal: 0.3, dominance: -0.4}   // Heartbroken
    }

BEGIN
    IF EMOJI_MAPPINGS.hasKey(emoji) THEN
        mapping ← EMOJI_MAPPINGS[emoji]

        RETURN EmotionalState {
            valence: mapping.valence,
            arousal: mapping.arousal,
            dominance: mapping.dominance,
            confidence: 0.5,              // Lowest confidence
            timestamp: CurrentDateTime()
        }
    ELSE
        // Unknown emoji: default to neutral
        RETURN EmotionalState {
            valence: 0.0,
            arousal: 0.0,
            dominance: 0.0,
            confidence: 0.3,
            timestamp: CurrentDateTime()
        }
    END IF
END
```

---

### Subroutine 4: Store Experience

```
SUBROUTINE: StoreExperience
INPUT: experience (EmotionalExperience)
OUTPUT: success (Boolean)

CONSTANTS:
    MAX_EXPERIENCES_PER_USER = 1000
    EXPERIENCE_TTL_DAYS = 90

BEGIN
    agentDB ← AgentDB.getInstance()

    // Store individual experience
    experienceKey ← "exp:" + experience.experienceId

    success ← agentDB.set(
        key: experienceKey,
        value: experience,
        ttl: EXPERIENCE_TTL_DAYS * 24 * 3600
    )

    IF NOT success THEN
        LOG_ERROR("Failed to store experience", experience.experienceId)
        RETURN false
    END IF

    // Add to user's experience list
    userExperiencesKey ← "user:" + experience.userId + ":experiences"

    // Get current experience list
    experienceList ← agentDB.get(userExperiencesKey)

    IF experienceList IS NULL THEN
        experienceList ← []
    END IF

    // Add new experience to front
    experienceList.prepend(experience.experienceId)

    // Limit list size (keep most recent)
    IF experienceList.length > MAX_EXPERIENCES_PER_USER THEN
        // Remove oldest experiences
        removed ← experienceList.slice(MAX_EXPERIENCES_PER_USER)
        experienceList ← experienceList.slice(0, MAX_EXPERIENCES_PER_USER)

        // Delete removed experiences from database
        FOR EACH oldExpId IN removed DO
            agentDB.delete("exp:" + oldExpId)
        END FOR
    END IF

    // Update user's experience list
    success ← agentDB.set(userExperiencesKey, experienceList)

    // Also add to global experience replay buffer
    replayBufferKey ← "global:experience_replay"

    agentDB.listPush(replayBufferKey, experience.experienceId, MAX_EXPERIENCES_PER_USER)

    RETURN success
END
```

---

### Subroutine 5: Update User Profile

```
SUBROUTINE: UpdateUserProfile
INPUT:
    userId (String)
    reward (Float)
OUTPUT: success (Boolean)

CONSTANTS:
    EXPLORATION_DECAY = 0.99
    MIN_EXPLORATION_RATE = 0.05
    REWARD_SMOOTHING = 0.1              // For exponential moving average

BEGIN
    agentDB ← AgentDB.getInstance()
    profileKey ← "user:" + userId + ":profile"

    // Get current profile
    profile ← agentDB.get(profileKey)

    IF profile IS NULL THEN
        // Initialize new profile
        profile ← UserProfile {
            userId: userId,
            totalExperiences: 0,
            avgReward: 0.0,
            explorationRate: 0.3,         // Start with 30% exploration
            preferredGenres: [],
            learningProgress: 0.0
        }
    END IF

    // Update experience count
    profile.totalExperiences ← profile.totalExperiences + 1

    // Update average reward using exponential moving average
    // EMA: new_avg = α * new_value + (1 - α) * old_avg
    profile.avgReward ← REWARD_SMOOTHING * reward +
                       (1 - REWARD_SMOOTHING) * profile.avgReward

    // Decay exploration rate (exploit more as we learn)
    profile.explorationRate ← MAX(
        MIN_EXPLORATION_RATE,
        profile.explorationRate * EXPLORATION_DECAY
    )

    // Calculate learning progress (0-100)
    // Based on experience count and average reward
    experienceScore ← MIN(1.0, profile.totalExperiences / 100.0)
    rewardScore ← (profile.avgReward + 1.0) / 2.0  // Normalize -1..1 to 0..1

    profile.learningProgress ← (experienceScore * 0.6 + rewardScore * 0.4) * 100

    // Save updated profile
    success ← agentDB.set(profileKey, profile)

    IF success THEN
        LOG_INFO("Updated user profile", {
            userId: userId,
            totalExperiences: profile.totalExperiences,
            avgReward: profile.avgReward,
            explorationRate: profile.explorationRate,
            learningProgress: profile.learningProgress
        })
    ELSE
        LOG_ERROR("Failed to update user profile", userId)
    END IF

    RETURN success
END
```

---

### Subroutine 6: Generate Feedback Message

```
SUBROUTINE: GenerateFeedbackMessage
INPUT:
    reward (Float)                    // Range: -1.0 to 1.0
    emotionalImprovement (Float)      // Distance moved toward target
OUTPUT: message (String)

BEGIN
    // Determine message based on reward thresholds

    IF reward > 0.7 THEN
        messages ← [
            "Excellent choice! This content really helped improve your mood. 🎯",
            "Perfect match! You're moving in exactly the right direction. ✨",
            "Great feedback! We're learning what works best for you. 🌟"
        ]
        RETURN RandomChoice(messages)

    ELSE IF reward > 0.4 THEN
        messages ← [
            "Good choice! Your recommendations are getting better. 👍",
            "Nice improvement! We're fine-tuning your preferences. 📈",
            "Solid match! Your content selection is improving. ✓"
        ]
        RETURN RandomChoice(messages)

    ELSE IF reward > 0.1 THEN
        messages ← [
            "Thanks for the feedback. We're learning your preferences. 📊",
            "Noted! This helps us understand what you enjoy. 💡",
            "Feedback received. We'll adjust future recommendations. 🔄"
        ]
        RETURN RandomChoice(messages)

    ELSE IF reward > -0.3 THEN
        messages ← [
            "We're still learning. Next time will be better! 🎯",
            "Thanks for letting us know. We'll improve! 📈",
            "Feedback noted. We're adjusting our approach. 🔧"
        ]
        RETURN RandomChoice(messages)

    ELSE
        messages ← [
            "Sorry this wasn't a great match. We'll do better next time! 🎯",
            "We're learning from this. Future recommendations will improve! 💪",
            "Thanks for the honest feedback. We'll adjust significantly! 🔄"
        ]
        RETURN RandomChoice(messages)
    END IF
END
```

---

### Subroutine 7: Calculate Emotional Improvement

```
SUBROUTINE: CalculateEmotionalImprovement
INPUT:
    stateBefore (EmotionalState)
    stateAfter (EmotionalState)
    desiredState (EmotionalState)
OUTPUT: improvement (Float)           // 0.0 to 1.0

BEGIN
    // Calculate distance before viewing
    distanceBefore ← SQRT(
        (stateBefore.valence - desiredState.valence)² +
        (stateBefore.arousal - desiredState.arousal)²
    )

    // Calculate distance after viewing
    distanceAfter ← SQRT(
        (stateAfter.valence - desiredState.valence)² +
        (stateAfter.arousal - desiredState.arousal)²
    )

    // Calculate improvement (reduction in distance)
    IF distanceBefore = 0.0 THEN
        // Already at target state
        RETURN 1.0
    END IF

    improvement ← (distanceBefore - distanceAfter) / distanceBefore

    // Normalize to 0-1 range
    improvement ← MAX(0.0, MIN(1.0, improvement))

    RETURN improvement
END
```

---

### Subroutine 8: Calculate Feedback Insights

```
SUBROUTINE: CalculateFeedbackInsights
INPUT:
    stateBefore (EmotionalState)
    stateAfter (EmotionalState)
    desiredState (EmotionalState)
    completionBonus (Float)
OUTPUT: insights (FeedbackInsights)

BEGIN
    // Calculate direction alignment component
    actualDelta ← Vector {
        valence: stateAfter.valence - stateBefore.valence,
        arousal: stateAfter.arousal - stateBefore.arousal
    }

    desiredDelta ← Vector {
        valence: desiredState.valence - stateBefore.valence,
        arousal: desiredState.arousal - stateBefore.arousal
    }

    dotProduct ← (actualDelta.valence * desiredDelta.valence) +
                 (actualDelta.arousal * desiredDelta.arousal)

    actualMagnitude ← SQRT(actualDelta.valence² + actualDelta.arousal²)
    desiredMagnitude ← SQRT(desiredDelta.valence² + desiredDelta.arousal²)

    IF actualMagnitude > 0 AND desiredMagnitude > 0 THEN
        directionAlignment ← dotProduct / (actualMagnitude * desiredMagnitude)
    ELSE
        directionAlignment ← 0.0
    END IF

    // Calculate magnitude score
    magnitude ← actualMagnitude
    magnitudeScore ← MIN(1.0, magnitude / 2.0)

    // Calculate proximity bonus
    distance ← SQRT(
        (stateAfter.valence - desiredState.valence)² +
        (stateAfter.arousal - desiredState.arousal)²
    )

    proximityBonus ← MAX(0.0, 0.2 * (1.0 - distance / 2.0))

    // Compile insights
    RETURN FeedbackInsights {
        directionAlignment: directionAlignment,
        magnitudeScore: magnitudeScore,
        proximityBonus: proximityBonus,
        completionBonus: completionBonus
    }
END
```

---

### Subroutine 9: Validate Feedback Request

```
SUBROUTINE: ValidateFeedbackRequest
INPUT: request (FeedbackRequest)
OUTPUT: valid (Boolean)

BEGIN
    // Check required fields
    IF request.userId IS NULL OR request.userId = "" THEN
        RETURN false
    END IF

    IF request.contentId IS NULL OR request.contentId = "" THEN
        RETURN false
    END IF

    IF request.emotionalStateId IS NULL OR request.emotionalStateId = "" THEN
        RETURN false
    END IF

    // Check that at least one feedback type is provided
    hasText ← request.postViewingState.text IS NOT NULL
    hasRating ← request.postViewingState.explicitRating IS NOT NULL
    hasEmoji ← request.postViewingState.explicitEmoji IS NOT NULL

    IF NOT (hasText OR hasRating OR hasEmoji) THEN
        RETURN false
    END IF

    // Validate rating range if provided
    IF hasRating THEN
        rating ← request.postViewingState.explicitRating
        IF rating < 1 OR rating > 5 THEN
            RETURN false
        END IF
    END IF

    // Validate completion rate if provided
    IF request.viewingDetails IS NOT NULL THEN
        rate ← request.viewingDetails.completionRate
        IF rate < 0.0 OR rate > 1.0 THEN
            RETURN false
        END IF
    END IF

    RETURN true
END
```

---

## Complexity Analysis

### Time Complexity

**ProcessFeedback Algorithm**:
```
Operation                          | Complexity | Notes
-----------------------------------|------------|---------------------------
Input validation                   | O(1)       | Fixed field checks
Retrieve pre-viewing state         | O(1)       | Database lookup with key
Get recommendation                 | O(1)       | Database lookup
Analyze post-viewing state         | O(n)       | n = text length (Gemini API)
Calculate reward                   | O(1)       | Fixed arithmetic operations
Calculate completion bonus         | O(1)       | Fixed conditionals
Get Q-value                        | O(1)       | Hash table lookup
Update Q-value                     | O(1)       | Hash table update
Store experience                   | O(1)       | Database writes
Update user profile                | O(1)       | Database read/write
Calculate insights                 | O(1)       | Fixed arithmetic
Total                              | O(n)       | Dominated by text analysis
```

**CalculateReward Algorithm**:
```
All operations are fixed arithmetic: O(1)
```

### Space Complexity

```
Data Structure                     | Size       | Notes
-----------------------------------|------------|---------------------------
FeedbackRequest                    | O(1)       | Fixed structure
EmotionalState (3x)                | O(1)       | Fixed dimensions
EmotionalExperience                | O(1)       | Fixed fields
FeedbackResponse                   | O(1)       | Fixed structure
Temporary calculations             | O(1)       | Small variables
Total                              | O(1)       | Constant space
```

### Database Operations

```
Operation                          | Type       | Count per Request
-----------------------------------|------------|------------------
Get emotional state                | Read       | 1
Get recommendation                 | Read       | 1
Get Q-value                        | Read       | 1
Get user profile                   | Read       | 1
Update Q-value                     | Write      | 1
Store experience                   | Write      | 1
Update user experience list        | Write      | 1
Update user profile                | Write      | 1
Total                              |            | 4 reads, 4 writes
```

---

## Integration Points

### 1. EmotionDetector Service
```
// External service for text-based emotion analysis
interface EmotionDetector {
    analyzeText(text: string): Promise<{
        valence: number;
        arousal: number;
        dominance: number;
        confidence: number;
    }>;
}
```

### 2. RLPolicyEngine
```
// RL policy engine for Q-value management
interface RLPolicyEngine {
    getQValue(state: EmotionalState, contentId: string): number;
    updateQValue(state: EmotionalState, contentId: string, newValue: number): boolean;
}
```

### 3. AgentDB
```
// Database for persistent storage
interface AgentDB {
    get(key: string): any;
    set(key: string, value: any, ttl?: number): boolean;
    delete(key: string): boolean;
    listPush(key: string, value: any, maxLength?: number): boolean;
}
```

### 4. RecommendationStore
```
// Store for recommendation metadata
interface RecommendationStore {
    get(userId: string, contentId: string): {
        targetEmotionalState: EmotionalState;
        recommendedAt: DateTime;
        qValue: number;
    };
}
```

---

## Example Calculations

### Example 1: Positive Feedback - Text Analysis

**Scenario**: User was stressed, wanted to relax, watched comedy, felt better

```
INPUT:
    stateBefore: {valence: -0.4, arousal: 0.6}    // Stressed (negative, high arousal)
    stateAfter: {valence: 0.5, arousal: -0.2}     // Relaxed (positive, low arousal)
    desiredState: {valence: 0.6, arousal: -0.3}   // Target: calm and happy

CALCULATIONS:

1. Direction Alignment:
    actualDelta = {valence: 0.9, arousal: -0.8}
    desiredDelta = {valence: 1.0, arousal: -0.9}

    dotProduct = (0.9 × 1.0) + (-0.8 × -0.9) = 0.9 + 0.72 = 1.62
    actualMagnitude = √(0.9² + 0.8²) = √(0.81 + 0.64) = √1.45 = 1.204
    desiredMagnitude = √(1.0² + 0.9²) = √(1.0 + 0.81) = √1.81 = 1.345

    directionAlignment = 1.62 / (1.204 × 1.345) = 1.62 / 1.619 = 1.0
    (Perfect alignment!)

2. Improvement Magnitude:
    magnitude = 1.204
    normalizedMagnitude = min(1.0, 1.204 / 2.0) = min(1.0, 0.602) = 0.602

3. Proximity Bonus:
    distance = √((0.5 - 0.6)² + (-0.2 - (-0.3))²)
             = √(0.01 + 0.01) = √0.02 = 0.141

    proximityBonus = max(0, 0.2 × (1 - 0.141/2)) = 0.2 × 0.929 = 0.186

4. Base Reward:
    baseReward = (1.0 × 0.6) + (0.602 × 0.4) = 0.6 + 0.241 = 0.841

5. Final Reward:
    finalReward = 0.841 + 0.186 = 1.027
    Clamped to: 1.0

RESULT: reward = 1.0 (Maximum positive reward!)
MESSAGE: "Excellent choice! This content really helped improve your mood. 🎯"
```

---

### Example 2: Moderate Feedback - Star Rating

**Scenario**: User was sad, wanted uplift, watched drama, felt somewhat better

```
INPUT:
    stateBefore: {valence: -0.6, arousal: -0.4}   // Sad, low energy
    stateAfter: {valence: 0.4, arousal: -0.1}     // From 4-star rating
    desiredState: {valence: 0.7, arousal: 0.2}    // Target: happy, energized

CALCULATIONS:

1. Direction Alignment:
    actualDelta = {valence: 1.0, arousal: 0.3}
    desiredDelta = {valence: 1.3, arousal: 0.6}

    dotProduct = (1.0 × 1.3) + (0.3 × 0.6) = 1.3 + 0.18 = 1.48
    actualMagnitude = √(1.0² + 0.3²) = √1.09 = 1.044
    desiredMagnitude = √(1.3² + 0.6²) = √2.05 = 1.432

    directionAlignment = 1.48 / (1.044 × 1.432) = 1.48 / 1.495 = 0.990
    (Excellent alignment)

2. Improvement Magnitude:
    magnitude = 1.044
    normalizedMagnitude = min(1.0, 1.044 / 2.0) = 0.522

3. Proximity Bonus:
    distance = √((0.4 - 0.7)² + (-0.1 - 0.2)²)
             = √(0.09 + 0.09) = √0.18 = 0.424

    proximityBonus = max(0, 0.2 × (1 - 0.424/2)) = 0.2 × 0.788 = 0.158

4. Base Reward:
    baseReward = (0.990 × 0.6) + (0.522 × 0.4) = 0.594 + 0.209 = 0.803

5. Completion Bonus:
    Assume 95% completion, no pauses/skips
    completionBonus = 0.2 × 0.95 = 0.19

6. Final Reward:
    finalReward = 0.803 + 0.158 + 0.19 = 1.151
    Clamped to: 1.0

RESULT: reward = 1.0
MESSAGE: "Perfect match! You're moving in exactly the right direction. ✨"
```

---

### Example 3: Negative Feedback - Low Completion

**Scenario**: User wanted energy boost, started action movie, stopped after 25%

```
INPUT:
    stateBefore: {valence: 0.0, arousal: -0.5}    // Neutral but tired
    stateAfter: {valence: -0.6, arousal: 0.1}     // From 2-star rating
    desiredState: {valence: 0.3, arousal: 0.6}    // Target: energized
    completionRate: 0.25

CALCULATIONS:

1. Direction Alignment:
    actualDelta = {valence: -0.6, arousal: 0.6}
    desiredDelta = {valence: 0.3, arousal: 1.1}

    dotProduct = (-0.6 × 0.3) + (0.6 × 1.1) = -0.18 + 0.66 = 0.48
    actualMagnitude = √(0.36 + 0.36) = √0.72 = 0.849
    desiredMagnitude = √(0.09 + 1.21) = √1.3 = 1.140

    directionAlignment = 0.48 / (0.849 × 1.140) = 0.48 / 0.968 = 0.496
    (Weak alignment - wrong direction on valence)

2. Improvement Magnitude:
    magnitude = 0.849
    normalizedMagnitude = min(1.0, 0.849 / 2.0) = 0.425

3. Proximity Bonus:
    distance = √((-0.6 - 0.3)² + (0.1 - 0.6)²)
             = √(0.81 + 0.25) = √1.06 = 1.030

    proximityBonus = max(0, 0.2 × (1 - 1.030/2)) = max(0, -0.003) = 0
    (No bonus - moved away from target)

4. Base Reward:
    baseReward = (0.496 × 0.6) + (0.425 × 0.4) = 0.298 + 0.170 = 0.468

5. Completion Penalty:
    completionRate = 0.25 (very low)
    completionBonus = -0.2 × (1 - 0.25) = -0.2 × 0.75 = -0.15

6. Final Reward:
    finalReward = 0.468 + 0 + (-0.15) = 0.318

RESULT: reward = 0.318
MESSAGE: "Thanks for the feedback. We're learning your preferences. 📊"

Q-VALUE UPDATE:
    Assume qValueBefore = 0.5
    qValueAfter = 0.5 + 0.1 × (0.318 - 0.5) = 0.5 + 0.1 × (-0.182) = 0.482

    (Q-value decreased - content will be less likely recommended in this state)
```

---

### Example 4: Poor Match - Early Exit

**Scenario**: Wanted calm content, got thriller, exited after 15%

```
INPUT:
    stateBefore: {valence: -0.2, arousal: 0.4}    // Anxious
    stateAfter: {valence: -0.8, arousal: 0.8}     // From 1-star rating
    desiredState: {valence: 0.5, arousal: -0.6}   // Target: calm and positive
    completionRate: 0.15
    pauseCount: 3
    skipCount: 2

CALCULATIONS:

1. Direction Alignment:
    actualDelta = {valence: -0.6, arousal: 0.4}
    desiredDelta = {valence: 0.7, arousal: -1.0}

    dotProduct = (-0.6 × 0.7) + (0.4 × -1.0) = -0.42 + (-0.4) = -0.82
    actualMagnitude = √(0.36 + 0.16) = √0.52 = 0.721
    desiredMagnitude = √(0.49 + 1.0) = √1.49 = 1.221

    directionAlignment = -0.82 / (0.721 × 1.221) = -0.82 / 0.880 = -0.932
    (Strong negative - opposite direction!)

2. Improvement Magnitude:
    magnitude = 0.721
    normalizedMagnitude = min(1.0, 0.721 / 2.0) = 0.361

3. Proximity Bonus:
    distance = √((-0.8 - 0.5)² + (0.8 - (-0.6))²)
             = √(1.69 + 1.96) = √3.65 = 1.911

    proximityBonus = max(0, 0.2 × (1 - 1.911/2)) = max(0, -0.192) = 0

4. Base Reward:
    baseReward = (-0.932 × 0.6) + (0.361 × 0.4) = -0.559 + 0.144 = -0.415

5. Completion Penalty:
    completionBonus = -0.2 × (1 - 0.15) = -0.17
    pausePenalty = min(0.1, 3 × 0.01) = 0.03
    skipPenalty = min(0.15, 2 × 0.02) = 0.04

    totalPenalty = -0.17 - 0.03 - 0.04 = -0.24

6. Final Reward:
    finalReward = -0.415 + 0 + (-0.24) = -0.655

RESULT: reward = -0.655 (Strongly negative)
MESSAGE: "Sorry this wasn't a great match. We'll do better next time! 🎯"

Q-VALUE UPDATE:
    Assume qValueBefore = 0.6
    qValueAfter = 0.6 + 0.1 × (-0.655 - 0.6) = 0.6 + 0.1 × (-1.255) = 0.474

    (Significant Q-value drop - avoid this content in similar states)
```

---

## Design Patterns

### 1. Strategy Pattern (Feedback Type Handling)
```
Different strategies for processing feedback types:
- TextFeedbackStrategy (most accurate)
- RatingFeedbackStrategy (moderate accuracy)
- EmojiFeedbackStrategy (least accurate)

Selected at runtime based on available data
```

### 2. Template Method (Feedback Processing)
```
ProcessFeedback defines the skeleton:
1. Validate input
2. Retrieve state
3. Analyze feedback (strategy varies)
4. Calculate reward
5. Update Q-value
6. Store experience
7. Update profile
```

### 3. Repository Pattern (Data Access)
```
Abstracted data access through:
- EmotionalStateStore
- RecommendationStore
- ExperienceStore
- UserProfileStore

Allows swapping storage backends
```

---

## Error Handling

```
ERROR CASES:

1. Invalid Feedback Request:
    - Missing required fields → ValidationError
    - Invalid rating range → ValidationError
    - Invalid completion rate → ValidationError

2. State Not Found:
    - Pre-viewing state missing → NotFoundError
    - Recommendation not found → NotFoundError

3. Analysis Failure:
    - Gemini API error → Fallback to neutral state
    - Timeout → Retry with exponential backoff

4. Database Errors:
    - Write failure → Log error, return false
    - Read failure → Retry 3 times, then fail

5. Q-Value Update Failure:
    - Lock conflict → Retry with optimistic locking
    - Invalid state → Log error, skip update

RECOVERY STRATEGIES:
- Use fallback neutral states for analysis failures
- Retry database operations with exponential backoff
- Log all errors for monitoring
- Return partial success when possible
```

---

## Performance Considerations

1. **Batching**: Process multiple feedback requests in parallel
2. **Caching**: Cache user profiles and recent emotional states
3. **Async Processing**: Non-blocking emotion analysis
4. **Database Indexing**: Index on userId, contentId, timestamp
5. **Experience Pruning**: Limit experience history to prevent unbounded growth

---

## Testing Considerations

1. **Unit Tests**:
    - CalculateReward with various state combinations
    - ConvertExplicitRating for all 5 ratings
    - ConvertEmojiToState for all supported emojis
    - Completion bonus calculations

2. **Integration Tests**:
    - End-to-end feedback processing
    - Database persistence and retrieval
    - Q-value updates
    - Profile updates

3. **Edge Cases**:
    - Zero magnitude changes
    - Perfect alignment vs. perfect misalignment
    - Maximum/minimum rewards
    - Very low completion rates
    - Missing optional fields

---

**END OF PSEUDOCODE SPECIFICATION**
