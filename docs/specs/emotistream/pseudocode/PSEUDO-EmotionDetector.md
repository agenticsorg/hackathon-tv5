# EmotionDetector Pseudocode Specification

**Component**: Emotion Detection System
**Version**: 1.0.0
**SPARC Phase**: Pseudocode
**Last Updated**: 2025-12-05

---

## Table of Contents
1. [Class Structure](#class-structure)
2. [Data Structures](#data-structures)
3. [Core Algorithms](#core-algorithms)
4. [Error Handling](#error-handling)
5. [Complexity Analysis](#complexity-analysis)
6. [Integration Notes](#integration-notes)

---

## Class Structure

```
CLASS: EmotionDetector

CONSTANTS:
    GEMINI_API_TIMEOUT = 30000  // 30 seconds in milliseconds
    MAX_RETRY_ATTEMPTS = 3
    RETRY_DELAY_MS = 1000
    PLUTCHIK_EMOTIONS = ["joy", "sadness", "anger", "fear", "trust", "disgust", "surprise", "anticipation"]
    DEFAULT_CONFIDENCE = 0.5
    NEUTRAL_VALENCE = 0.0
    NEUTRAL_AROUSAL = 0.0

DEPENDENCIES:
    geminiClient: GeminiAPI
    agentDBClient: AgentDB
    logger: Logger

METHODS:
    // Public interface
    analyzeText(text: string, userId: string): Promise<EmotionalState>

    // API communication
    callGeminiEmotionAPI(text: string, attemptNumber: integer): Promise<GeminiResponse>

    // Emotional mapping
    mapToValenceArousal(geminiResponse: GeminiResponse): {valence: float, arousal: float}
    generateEmotionVector(primaryEmotion: string, intensity: float): Float32Array
    calculateStressLevel(valence: float, arousal: float): float
    calculateConfidence(geminiResponse: GeminiResponse): float

    // Fallback and validation
    createFallbackState(userId: string): EmotionalState
    validateText(text: string): boolean
    validateGeminiResponse(response: GeminiResponse): boolean

    // Persistence
    saveToAgentDB(emotionalState: EmotionalState): Promise<void>
```

---

## Data Structures

### Primary Types

```
TYPE: EmotionalState
STRUCTURE:
    emotionalStateId: string        // UUID v4 format
    userId: string                  // User identifier
    valence: float                  // Range: -1.0 to +1.0
    arousal: float                  // Range: -1.0 to +1.0
    primaryEmotion: string          // One of PLUTCHIK_EMOTIONS
    emotionVector: Float32Array     // 8D vector, values 0.0 to 1.0
    stressLevel: float              // Range: 0.0 to 1.0
    confidence: float               // Range: 0.0 to 1.0
    timestamp: integer              // Unix timestamp in milliseconds
    rawText: string                 // Original input text

INVARIANTS:
    - valence MUST be in range [-1.0, 1.0]
    - arousal MUST be in range [-1.0, 1.0]
    - stressLevel MUST be in range [0.0, 1.0]
    - confidence MUST be in range [0.0, 1.0]
    - emotionVector length MUST equal 8
    - emotionVector elements MUST sum to approximately 1.0
    - primaryEmotion MUST be in PLUTCHIK_EMOTIONS
```

```
TYPE: GeminiResponse
STRUCTURE:
    primaryEmotion: string
    valence: float
    arousal: float
    stressLevel: float
    confidence: float
    reasoning: string               // Gemini's explanation
    rawResponse: object             // Full API response
```

```
TYPE: EmotionVectorWeights
STRUCTURE:
    // Mapping of Plutchik emotions to base vector positions
    joy: 0          // Index 0
    sadness: 1      // Index 1
    anger: 2        // Index 2
    fear: 3         // Index 3
    trust: 4        // Index 4
    disgust: 5      // Index 5
    surprise: 6     // Index 6
    anticipation: 7 // Index 7
```

---

## Core Algorithms

### Algorithm 1: Main Entry Point

```
ALGORITHM: analyzeText
INPUT: text (string), userId (string)
OUTPUT: emotionalState (EmotionalState)

BEGIN
    // Step 1: Input validation
    IF NOT validateText(text) THEN
        logger.warn("Invalid text input", {userId, textLength: text.length})
        RETURN createFallbackState(userId)
    END IF

    // Step 2: Call Gemini API with retry logic
    geminiResponse ← NULL
    lastError ← NULL

    FOR attemptNumber FROM 1 TO MAX_RETRY_ATTEMPTS DO
        TRY
            geminiResponse ← callGeminiEmotionAPI(text, attemptNumber)

            // Validate response
            IF validateGeminiResponse(geminiResponse) THEN
                BREAK  // Success, exit retry loop
            ELSE
                logger.warn("Invalid Gemini response", {attempt: attemptNumber})
                lastError ← Error("Invalid response structure")
            END IF

        CATCH timeoutError
            logger.warn("Gemini API timeout", {attempt: attemptNumber})
            lastError ← timeoutError

            IF attemptNumber < MAX_RETRY_ATTEMPTS THEN
                SLEEP(RETRY_DELAY_MS * attemptNumber)  // Exponential backoff
            END IF

        CATCH rateLimitError
            logger.warn("Rate limit exceeded", {attempt: attemptNumber})
            lastError ← rateLimitError

            IF attemptNumber < MAX_RETRY_ATTEMPTS THEN
                SLEEP(RETRY_DELAY_MS * attemptNumber * 2)  // Longer backoff
            END IF

        CATCH apiError
            logger.error("Gemini API error", {error: apiError, attempt: attemptNumber})
            lastError ← apiError
            BREAK  // Fatal error, don't retry
        END TRY
    END FOR

    // Step 3: Handle API failure
    IF geminiResponse IS NULL THEN
        logger.error("All Gemini API attempts failed", {userId, error: lastError})
        RETURN createFallbackState(userId)
    END IF

    // Step 4: Map Gemini response to emotional dimensions
    {valence, arousal} ← mapToValenceArousal(geminiResponse)

    // Step 5: Generate 8D emotion vector
    emotionVector ← generateEmotionVector(
        geminiResponse.primaryEmotion,
        1.0  // Full intensity
    )

    // Step 6: Calculate derived metrics
    stressLevel ← calculateStressLevel(valence, arousal)
    confidence ← calculateConfidence(geminiResponse)

    // Step 7: Construct emotional state
    emotionalState ← EmotionalState {
        emotionalStateId: generateUUID(),
        userId: userId,
        valence: CLAMP(valence, -1.0, 1.0),
        arousal: CLAMP(arousal, -1.0, 1.0),
        primaryEmotion: geminiResponse.primaryEmotion,
        emotionVector: emotionVector,
        stressLevel: CLAMP(stressLevel, 0.0, 1.0),
        confidence: CLAMP(confidence, 0.0, 1.0),
        timestamp: getCurrentTimestamp(),
        rawText: text
    }

    // Step 8: Persist to AgentDB (async, non-blocking)
    saveToAgentDB(emotionalState)
        .CATCH(error => logger.error("Failed to save to AgentDB", {error}))

    // Step 9: Return result
    RETURN emotionalState
END
```

**Complexity Analysis:**
- Time: O(1) excluding API call (API call is I/O bound, not CPU bound)
- Space: O(1) - Fixed size emotional state object
- Network: O(n) retries where n = MAX_RETRY_ATTEMPTS

---

### Algorithm 2: Gemini API Communication

```
ALGORITHM: callGeminiEmotionAPI
INPUT: text (string), attemptNumber (integer)
OUTPUT: geminiResponse (GeminiResponse)

SUBROUTINES:
    buildPrompt(text: string): string
    parseGeminiJSON(rawResponse: string): GeminiResponse

BEGIN
    // Step 1: Construct structured prompt
    prompt ← buildPrompt(text)

    // Step 2: Prepare API request
    apiRequest ← {
        model: "gemini-2.0-flash-exp",
        contents: [{
            parts: [{text: prompt}]
        }],
        generationConfig: {
            temperature: 0.3,        // Lower temperature for consistency
            topP: 0.8,
            topK: 40,
            maxOutputTokens: 256,    // Keep response concise
            responseMimeType: "application/json"
        },
        safetySettings: [
            {category: "HARM_CATEGORY_HARASSMENT", threshold: "BLOCK_NONE"},
            {category: "HARM_CATEGORY_HATE_SPEECH", threshold: "BLOCK_NONE"},
            {category: "HARM_CATEGORY_SEXUALLY_EXPLICIT", threshold: "BLOCK_NONE"},
            {category: "HARM_CATEGORY_DANGEROUS_CONTENT", threshold: "BLOCK_NONE"}
        ]
    }

    // Step 3: Set timeout wrapper
    timeoutPromise ← createTimeout(GEMINI_API_TIMEOUT)

    // Step 4: Execute API call with timeout race
    TRY
        rawResponse ← AWAIT Promise.race([
            geminiClient.generateContent(apiRequest),
            timeoutPromise
        ])

    CATCH timeoutError
        logger.error("Gemini API timeout exceeded", {
            attempt: attemptNumber,
            timeout: GEMINI_API_TIMEOUT
        })
        THROW TimeoutError("Gemini API call exceeded 30s timeout")
    END TRY

    // Step 5: Extract JSON from response
    IF rawResponse.candidates IS EMPTY THEN
        THROW APIError("No candidates in Gemini response")
    END IF

    responseText ← rawResponse.candidates[0].content.parts[0].text

    // Step 6: Parse JSON response
    TRY
        geminiResponse ← parseGeminiJSON(responseText)
        geminiResponse.rawResponse ← rawResponse

    CATCH parseError
        logger.error("Failed to parse Gemini JSON", {
            error: parseError,
            responseText: responseText
        })
        THROW ParseError("Invalid JSON in Gemini response")
    END TRY

    // Step 7: Return parsed response
    RETURN geminiResponse
END

SUBROUTINE: buildPrompt
INPUT: text (string)
OUTPUT: prompt (string)

BEGIN
    // Structured prompt with clear instructions
    prompt ← """
Analyze the emotional state from this text: "{text}"

You are an expert emotion analyst. Extract the following emotional dimensions:

1. **Primary Emotion**: Choose ONE from [joy, sadness, anger, fear, trust, disgust, surprise, anticipation]

2. **Valence**: Emotional pleasantness
   - Range: -1.0 (very negative) to +1.0 (very positive)
   - Examples: "I love this!" → +0.8, "I hate everything" → -0.9

3. **Arousal**: Emotional activation/energy level
   - Range: -1.0 (very calm/sleepy) to +1.0 (very excited/agitated)
   - Examples: "I'm thrilled!" → +0.9, "I feel peaceful" → -0.6

4. **Stress Level**: Psychological stress
   - Range: 0.0 (completely relaxed) to 1.0 (extremely stressed)
   - Consider: urgency, pressure, anxiety, overwhelm

5. **Confidence**: How certain are you about this analysis?
   - Range: 0.0 (very uncertain) to 1.0 (very certain)

Respond ONLY with valid JSON:
{{
  "primaryEmotion": "...",
  "valence": 0.0,
  "arousal": 0.0,
  "stressLevel": 0.0,
  "confidence": 0.0,
  "reasoning": "Brief explanation (max 50 words)"
}}
"""

    // Replace placeholder with actual text
    prompt ← REPLACE(prompt, "{text}", escapeJSON(text))

    RETURN prompt
END
```

**Complexity Analysis:**
- Time: O(1) for prompt construction, O(network) for API call
- Space: O(n) where n = text.length
- Network: 1 API call per invocation

---

### Algorithm 3: Valence-Arousal Mapping

```
ALGORITHM: mapToValenceArousal
INPUT: geminiResponse (GeminiResponse)
OUTPUT: {valence: float, arousal: float}

BEGIN
    // Step 1: Extract raw values from Gemini
    rawValence ← geminiResponse.valence
    rawArousal ← geminiResponse.arousal

    // Step 2: Validate ranges
    IF rawValence IS NULL OR rawValence < -1.0 OR rawValence > 1.0 THEN
        logger.warn("Invalid valence from Gemini", {rawValence})
        rawValence ← NEUTRAL_VALENCE
    END IF

    IF rawArousal IS NULL OR rawArousal < -1.0 OR rawArousal > 1.0 THEN
        logger.warn("Invalid arousal from Gemini", {rawArousal})
        rawArousal ← NEUTRAL_AROUSAL
    END IF

    // Step 3: Apply Russell's Circumplex constraints
    // Ensure values fall within valid circumplex space
    magnitude ← SQRT(rawValence² + rawArousal²)

    IF magnitude > 1.414 THEN  // √2, max distance in unit circle
        // Normalize to unit circle
        scaleFactor ← 1.414 / magnitude
        rawValence ← rawValence * scaleFactor
        rawArousal ← rawArousal * scaleFactor

        logger.debug("Normalized valence-arousal to circumplex", {
            original: {valence: geminiResponse.valence, arousal: geminiResponse.arousal},
            normalized: {valence: rawValence, arousal: rawArousal}
        })
    END IF

    // Step 4: Round to 2 decimal places for consistency
    valence ← ROUND(rawValence, 2)
    arousal ← ROUND(rawArousal, 2)

    // Step 5: Return mapped values
    RETURN {valence: valence, arousal: arousal}
END
```

**Russell's Circumplex Quadrants:**
```
         High Arousal (+1.0)
              |
    Tense    |    Excited
    Nervous  |    Elated
         Q2  |  Q1
             |
-1.0 --------+-------- +1.0
   Negative  |  Positive
    Valence  |  Valence
         Q3  |  Q4
             |
    Sad      |    Calm
    Bored    |    Relaxed
             |
         Low Arousal (-1.0)
```

**Complexity Analysis:**
- Time: O(1) - Fixed operations
- Space: O(1) - Two float values

---

### Algorithm 4: Plutchik Emotion Vector Generation

```
ALGORITHM: generateEmotionVector
INPUT: primaryEmotion (string), intensity (float)
OUTPUT: emotionVector (Float32Array of length 8)

CONSTANTS:
    // Opposite emotion pairs in Plutchik's wheel
    OPPOSITE_PAIRS = {
        "joy": "sadness",
        "sadness": "joy",
        "anger": "fear",
        "fear": "anger",
        "trust": "disgust",
        "disgust": "trust",
        "surprise": "anticipation",
        "anticipation": "surprise"
    }

    // Adjacent emotions (neighbors on wheel)
    ADJACENT_EMOTIONS = {
        "joy": ["trust", "anticipation"],
        "sadness": ["disgust", "fear"],
        "anger": ["disgust", "anticipation"],
        "fear": ["surprise", "sadness"],
        "trust": ["joy", "fear"],
        "disgust": ["sadness", "anger"],
        "surprise": ["fear", "joy"],
        "anticipation": ["joy", "anger"]
    }

BEGIN
    // Step 1: Initialize 8D vector with zeros
    emotionVector ← Float32Array[8] FILLED WITH 0.0

    // Step 2: Validate primary emotion
    IF primaryEmotion NOT IN PLUTCHIK_EMOTIONS THEN
        logger.warn("Invalid primary emotion", {primaryEmotion})
        primaryEmotion ← "trust"  // Default to neutral-positive
    END IF

    // Step 3: Clamp intensity to valid range
    intensity ← CLAMP(intensity, 0.0, 1.0)

    // Step 4: Get emotion index
    primaryIndex ← PLUTCHIK_EMOTIONS.indexOf(primaryEmotion)

    // Step 5: Set primary emotion intensity
    emotionVector[primaryIndex] ← intensity

    // Step 6: Add complementary emotions (adjacent emotions at lower intensity)
    adjacentEmotions ← ADJACENT_EMOTIONS[primaryEmotion]
    adjacentIntensity ← intensity * 0.3  // 30% of primary intensity

    FOR EACH adjacentEmotion IN adjacentEmotions DO
        adjacentIndex ← PLUTCHIK_EMOTIONS.indexOf(adjacentEmotion)
        emotionVector[adjacentIndex] ← adjacentIntensity
    END FOR

    // Step 7: Suppress opposite emotion
    oppositeEmotion ← OPPOSITE_PAIRS[primaryEmotion]
    oppositeIndex ← PLUTCHIK_EMOTIONS.indexOf(oppositeEmotion)
    emotionVector[oppositeIndex] ← 0.0

    // Step 8: Normalize vector to sum to 1.0 (probability distribution)
    vectorSum ← SUM(emotionVector)

    IF vectorSum > 0 THEN
        FOR i FROM 0 TO 7 DO
            emotionVector[i] ← emotionVector[i] / vectorSum
        END FOR
    ELSE
        // Fallback: uniform distribution if somehow sum is zero
        FOR i FROM 0 TO 7 DO
            emotionVector[i] ← 1.0 / 8.0
        END FOR
    END IF

    // Step 9: Round to 4 decimal places
    FOR i FROM 0 TO 7 DO
        emotionVector[i] ← ROUND(emotionVector[i], 4)
    END FOR

    // Step 10: Return normalized vector
    RETURN emotionVector
END
```

**Example Outputs:**

Input: primaryEmotion = "joy", intensity = 0.8
```
[0] joy:          0.5714  (primary: 0.8 / sum)
[1] sadness:      0.0000  (opposite: suppressed)
[2] anger:        0.0000
[3] fear:         0.0000
[4] trust:        0.1714  (adjacent: 0.24 / sum)
[5] disgust:      0.0000
[6] surprise:     0.0000
[7] anticipation: 0.1714  (adjacent: 0.24 / sum)
Sum = 1.0000
```

Input: primaryEmotion = "fear", intensity = 0.6
```
[0] joy:          0.0000
[1] sadness:      0.1304  (adjacent)
[2] anger:        0.0000  (opposite: suppressed)
[3] fear:         0.6522  (primary)
[4] trust:        0.1304  (adjacent)
[5] disgust:      0.0000
[6] surprise:     0.0870
[7] anticipation: 0.0000
Sum = 1.0000
```

**Complexity Analysis:**
- Time: O(1) - Fixed 8-element array operations
- Space: O(1) - 8 float values

---

### Algorithm 5: Stress Level Calculation

```
ALGORITHM: calculateStressLevel
INPUT: valence (float), arousal (float)
OUTPUT: stressLevel (float)

CONSTANTS:
    // Stress weights for quadrants (Russell's Circumplex)
    Q1_WEIGHT = 0.3   // High arousal + Positive valence (excited, less stressed)
    Q2_WEIGHT = 0.9   // High arousal + Negative valence (anxious, high stress)
    Q3_WEIGHT = 0.6   // Low arousal + Negative valence (depressed, moderate stress)
    Q4_WEIGHT = 0.1   // Low arousal + Positive valence (calm, low stress)

BEGIN
    // Step 1: Determine quadrant in Russell's Circumplex
    isHighArousal ← (arousal > 0)
    isPositiveValence ← (valence > 0)

    // Step 2: Select base stress weight by quadrant
    IF isHighArousal AND isPositiveValence THEN
        // Q1: Excited, energized (low-moderate stress)
        baseStress ← Q1_WEIGHT

    ELSE IF isHighArousal AND NOT isPositiveValence THEN
        // Q2: Tense, anxious (high stress)
        baseStress ← Q2_WEIGHT

    ELSE IF NOT isHighArousal AND NOT isPositiveValence THEN
        // Q3: Sad, bored (moderate stress)
        baseStress ← Q3_WEIGHT

    ELSE  // Low arousal + Positive valence
        // Q4: Calm, relaxed (low stress)
        baseStress ← Q4_WEIGHT
    END IF

    // Step 3: Calculate distance from origin (emotional intensity)
    emotionalIntensity ← SQRT(valence² + arousal²)

    // Step 4: Adjust stress by emotional intensity
    // Higher intensity = higher stress (up to √2 max distance)
    intensityFactor ← emotionalIntensity / 1.414  // Normalize to 0-1

    // Step 5: Compute final stress level
    // Combine base stress with intensity
    stressLevel ← baseStress * (0.7 + 0.3 * intensityFactor)

    // Step 6: Special case: Extreme negative valence boosts stress
    IF valence < -0.7 THEN
        negativeBoost ← (ABS(valence) - 0.7) * 0.5  // Up to +0.15 boost
        stressLevel ← stressLevel + negativeBoost
    END IF

    // Step 7: Clamp to valid range
    stressLevel ← CLAMP(stressLevel, 0.0, 1.0)

    // Step 8: Round to 2 decimal places
    stressLevel ← ROUND(stressLevel, 2)

    RETURN stressLevel
END
```

**Stress Calculation Examples:**

| Valence | Arousal | Quadrant | Base | Intensity | Final Stress | Emotion State      |
|---------|---------|----------|------|-----------|--------------|-------------------|
| +0.8    | +0.6    | Q1       | 0.3  | 0.71      | 0.36         | Joyful, excited   |
| -0.9    | +0.8    | Q2       | 0.9  | 0.85      | 1.00         | Angry, panicked   |
| -0.6    | -0.4    | Q3       | 0.6  | 0.51      | 0.54         | Sad, tired        |
| +0.7    | -0.3    | Q4       | 0.1  | 0.54      | 0.09         | Calm, content     |

**Complexity Analysis:**
- Time: O(1) - Fixed arithmetic operations
- Space: O(1) - Single float value

---

### Algorithm 6: Confidence Calculation

```
ALGORITHM: calculateConfidence
INPUT: geminiResponse (GeminiResponse)
OUTPUT: confidence (float)

BEGIN
    // Step 1: Extract Gemini's self-reported confidence
    geminiConfidence ← geminiResponse.confidence

    // Step 2: Validate Gemini confidence
    IF geminiConfidence IS NULL OR geminiConfidence < 0.0 OR geminiConfidence > 1.0 THEN
        logger.warn("Invalid confidence from Gemini", {geminiConfidence})
        geminiConfidence ← DEFAULT_CONFIDENCE
    END IF

    // Step 3: Calculate consistency score
    // Check if valence and arousal align with primary emotion
    consistencyScore ← calculateEmotionConsistency(
        geminiResponse.primaryEmotion,
        geminiResponse.valence,
        geminiResponse.arousal
    )

    // Step 4: Check reasoning quality
    reasoningLength ← LENGTH(geminiResponse.reasoning)
    reasoningScore ← 0.0

    IF reasoningLength > 10 THEN  // Has meaningful reasoning
        reasoningScore ← 1.0
    ELSE IF reasoningLength > 0 THEN  // Has some reasoning
        reasoningScore ← 0.5
    ELSE
        reasoningScore ← 0.0  // No reasoning provided
    END IF

    // Step 5: Combine factors (weighted average)
    confidence ← (
        geminiConfidence * 0.6 +      // 60% weight on Gemini's confidence
        consistencyScore * 0.3 +       // 30% weight on consistency
        reasoningScore * 0.1           // 10% weight on reasoning
    )

    // Step 6: Clamp to valid range
    confidence ← CLAMP(confidence, 0.0, 1.0)

    // Step 7: Round to 2 decimal places
    confidence ← ROUND(confidence, 2)

    RETURN confidence
END

SUBROUTINE: calculateEmotionConsistency
INPUT: primaryEmotion (string), valence (float), arousal (float)
OUTPUT: consistencyScore (float)

CONSTANTS:
    // Expected valence-arousal ranges for each emotion
    EMOTION_RANGES = {
        "joy":          {valence: [0.5, 1.0],   arousal: [0.3, 1.0]},
        "sadness":      {valence: [-1.0, -0.3], arousal: [-0.7, 0.0]},
        "anger":        {valence: [-1.0, -0.4], arousal: [0.4, 1.0]},
        "fear":         {valence: [-1.0, -0.2], arousal: [0.2, 1.0]},
        "trust":        {valence: [0.3, 1.0],   arousal: [-0.4, 0.4]},
        "disgust":      {valence: [-1.0, -0.3], arousal: [-0.2, 0.5]},
        "surprise":     {valence: [-0.3, 0.7],  arousal: [0.5, 1.0]},
        "anticipation": {valence: [0.0, 0.8],   arousal: [0.2, 0.8]}
    }

BEGIN
    // Get expected ranges for primary emotion
    expectedRange ← EMOTION_RANGES[primaryEmotion]

    // Check if valence is in expected range
    valenceMatch ← (
        valence >= expectedRange.valence[0] AND
        valence <= expectedRange.valence[1]
    )

    // Check if arousal is in expected range
    arousalMatch ← (
        arousal >= expectedRange.arousal[0] AND
        arousal <= expectedRange.arousal[1]
    )

    // Calculate consistency score
    IF valenceMatch AND arousalMatch THEN
        consistencyScore ← 1.0  // Perfect consistency
    ELSE IF valenceMatch OR arousalMatch THEN
        consistencyScore ← 0.6  // Partial consistency
    ELSE
        consistencyScore ← 0.3  // Inconsistent
    END IF

    RETURN consistencyScore
END
```

**Complexity Analysis:**
- Time: O(1) - Fixed comparisons
- Space: O(1) - Single float value

---

### Algorithm 7: Fallback State Generation

```
ALGORITHM: createFallbackState
INPUT: userId (string)
OUTPUT: emotionalState (EmotionalState)

BEGIN
    // Step 1: Log fallback creation
    logger.warn("Creating fallback emotional state", {
        userId: userId,
        reason: "API failure or invalid input"
    })

    // Step 2: Generate neutral emotion vector
    // Equal distribution across all emotions
    neutralVector ← Float32Array[8]
    FOR i FROM 0 TO 7 DO
        neutralVector[i] ← 1.0 / 8.0  // 0.125 for each emotion
    END FOR

    // Step 3: Construct fallback state
    fallbackState ← EmotionalState {
        emotionalStateId: generateUUID(),
        userId: userId,
        valence: NEUTRAL_VALENCE,      // 0.0
        arousal: NEUTRAL_AROUSAL,      // 0.0
        primaryEmotion: "trust",       // Neutral-positive default
        emotionVector: neutralVector,
        stressLevel: 0.5,              // Medium stress (unknown state)
        confidence: 0.0,               // Zero confidence (fallback)
        timestamp: getCurrentTimestamp(),
        rawText: ""
    }

    // Step 4: Return fallback state
    RETURN fallbackState
END
```

**Complexity Analysis:**
- Time: O(1) - Fixed operations
- Space: O(1) - Single EmotionalState object

---

## Error Handling

### Input Validation

```
ALGORITHM: validateText
INPUT: text (string)
OUTPUT: isValid (boolean)

BEGIN
    // Check 1: Not null or undefined
    IF text IS NULL OR text IS UNDEFINED THEN
        RETURN false
    END IF

    // Check 2: Not empty string
    IF TRIM(text).length = 0 THEN
        RETURN false
    END IF

    // Check 3: Reasonable length (not too short or too long)
    textLength ← LENGTH(text)
    IF textLength < 3 THEN
        logger.warn("Text too short for analysis", {length: textLength})
        RETURN false
    END IF

    IF textLength > 5000 THEN
        logger.warn("Text exceeds maximum length", {length: textLength})
        RETURN false  // Or truncate: text ← text.substring(0, 5000)
    END IF

    // Check 4: Contains some alphanumeric characters
    hasAlphanumeric ← REGEX_TEST(text, /[a-zA-Z0-9]/)
    IF NOT hasAlphanumeric THEN
        logger.warn("Text contains no alphanumeric characters")
        RETURN false
    END IF

    RETURN true
END
```

### Response Validation

```
ALGORITHM: validateGeminiResponse
INPUT: response (GeminiResponse)
OUTPUT: isValid (boolean)

BEGIN
    // Check 1: Response object exists
    IF response IS NULL OR response IS UNDEFINED THEN
        RETURN false
    END IF

    // Check 2: Required fields present
    requiredFields ← ["primaryEmotion", "valence", "arousal", "stressLevel", "confidence"]

    FOR EACH field IN requiredFields DO
        IF response[field] IS NULL OR response[field] IS UNDEFINED THEN
            logger.warn("Missing required field in Gemini response", {field})
            RETURN false
        END IF
    END FOR

    // Check 3: Primary emotion is valid
    IF response.primaryEmotion NOT IN PLUTCHIK_EMOTIONS THEN
        logger.warn("Invalid primary emotion", {emotion: response.primaryEmotion})
        RETURN false
    END IF

    // Check 4: Numeric values in valid ranges
    IF response.valence < -1.0 OR response.valence > 1.0 THEN
        logger.warn("Valence out of range", {valence: response.valence})
        RETURN false
    END IF

    IF response.arousal < -1.0 OR response.arousal > 1.0 THEN
        logger.warn("Arousal out of range", {arousal: response.arousal})
        RETURN false
    END IF

    IF response.stressLevel < 0.0 OR response.stressLevel > 1.0 THEN
        logger.warn("Stress level out of range", {stressLevel: response.stressLevel})
        RETURN false
    END IF

    IF response.confidence < 0.0 OR response.confidence > 1.0 THEN
        logger.warn("Confidence out of range", {confidence: response.confidence})
        RETURN false
    END IF

    RETURN true
END
```

### Error Recovery Patterns

```
PATTERN: Retry with Exponential Backoff

FOR attemptNumber FROM 1 TO MAX_RETRY_ATTEMPTS DO
    TRY
        result ← performAPICall()
        RETURN result  // Success

    CATCH error
        IF attemptNumber = MAX_RETRY_ATTEMPTS THEN
            logger.error("Max retries exceeded", {error})
            THROW error
        END IF

        // Calculate backoff delay
        baseDelay ← RETRY_DELAY_MS
        backoffDelay ← baseDelay * (2 ^ (attemptNumber - 1))  // Exponential
        jitter ← RANDOM(0, backoffDelay * 0.2)  // Add jitter
        totalDelay ← backoffDelay + jitter

        logger.info("Retrying after delay", {
            attempt: attemptNumber,
            delay: totalDelay
        })

        SLEEP(totalDelay)
    END TRY
END FOR
```

---

## Complexity Analysis

### Overall System Complexity

#### Time Complexity

**analyzeText()**: O(1) + O(network)
- Input validation: O(n) where n = text.length (linear scan)
- API call: O(network) - I/O bound, not CPU bound
- Valence-arousal mapping: O(1)
- Emotion vector generation: O(1) - Fixed 8 elements
- Stress calculation: O(1)
- Confidence calculation: O(1)
- AgentDB save: O(1) async - Non-blocking

**Total**: O(n) for text validation, dominated by O(network) for API call

#### Space Complexity

**analyzeText()**: O(n) + O(1)
- Input text storage: O(n) where n = text.length
- EmotionalState object: O(1) - Fixed size
- Emotion vector: O(1) - Fixed 8 elements
- Temporary variables: O(1)

**Total**: O(n) dominated by input text storage

#### Network Complexity

- API calls: 1 call + up to 3 retries = max 4 API calls
- Timeout: 30 seconds per call
- Worst case: 30s * 3 retries = 90 seconds total

### Performance Characteristics

| Operation              | Time     | Space | Network |
|------------------------|----------|-------|---------|
| Text validation        | O(n)     | O(1)  | -       |
| Gemini API call        | O(1)*    | O(1)  | 1 call  |
| Response parsing       | O(m)     | O(m)  | -       |
| Valence-arousal map    | O(1)     | O(1)  | -       |
| Emotion vector gen     | O(1)     | O(1)  | -       |
| Stress calculation     | O(1)     | O(1)  | -       |
| Confidence calc        | O(1)     | O(1)  | -       |
| AgentDB save (async)   | O(1)     | O(1)  | 1 query |

*API call is I/O bound (network latency), not CPU bound

### Scalability Considerations

**Bottlenecks:**
1. **Gemini API rate limits**: ~60 requests/minute
2. **API latency**: Average 2-5 seconds per request
3. **Network timeouts**: 30 second limit

**Optimizations:**
1. **Caching**: Cache results for identical text inputs (TTL: 5 minutes)
2. **Batching**: Process multiple texts in single API call (future enhancement)
3. **Prefetching**: Predict next analysis needs based on user patterns
4. **Circuit breaker**: Stop retries if API is consistently failing

---

## Integration Notes

### AgentDB Integration

```
ALGORITHM: saveToAgentDB
INPUT: emotionalState (EmotionalState)
OUTPUT: Promise<void>

BEGIN
    // Step 1: Prepare AgentDB document
    document ← {
        collection: "emotional_states",
        id: emotionalState.emotionalStateId,
        data: {
            userId: emotionalState.userId,
            valence: emotionalState.valence,
            arousal: emotionalState.arousal,
            primaryEmotion: emotionalState.primaryEmotion,
            emotionVector: Array.from(emotionalState.emotionVector),  // Convert Float32Array
            stressLevel: emotionalState.stressLevel,
            confidence: emotionalState.confidence,
            timestamp: emotionalState.timestamp,
            rawText: emotionalState.rawText
        },
        metadata: {
            source: "EmotionDetector",
            version: "1.0.0"
        },
        embeddings: emotionalState.emotionVector  // Use emotion vector as embedding
    }

    // Step 2: Create AgentDB indexes (if not exists)
    AWAIT agentDBClient.createIndex({
        collection: "emotional_states",
        fields: ["userId", "timestamp"],
        unique: false
    })

    // Step 3: Insert document
    TRY
        AWAIT agentDBClient.insert(document)
        logger.info("Saved emotional state to AgentDB", {
            stateId: emotionalState.emotionalStateId,
            userId: emotionalState.userId
        })

    CATCH error
        logger.error("Failed to save to AgentDB", {
            error: error,
            stateId: emotionalState.emotionalStateId
        })
        THROW error
    END TRY
END
```

### Querying Emotional History

```
ALGORITHM: getEmotionalHistory
INPUT: userId (string), limit (integer), fromTimestamp (integer)
OUTPUT: Promise<Array<EmotionalState>>

BEGIN
    // Step 1: Query AgentDB with vector similarity
    query ← {
        collection: "emotional_states",
        filter: {
            userId: userId,
            timestamp: {$gte: fromTimestamp}
        },
        sort: {timestamp: -1},  // Most recent first
        limit: limit
    }

    // Step 2: Execute query
    results ← AWAIT agentDBClient.query(query)

    // Step 3: Convert to EmotionalState objects
    emotionalHistory ← []
    FOR EACH doc IN results DO
        state ← EmotionalState {
            emotionalStateId: doc.id,
            userId: doc.data.userId,
            valence: doc.data.valence,
            arousal: doc.data.arousal,
            primaryEmotion: doc.data.primaryEmotion,
            emotionVector: Float32Array.from(doc.data.emotionVector),
            stressLevel: doc.data.stressLevel,
            confidence: doc.data.confidence,
            timestamp: doc.data.timestamp,
            rawText: doc.data.rawText
        }
        emotionalHistory.append(state)
    END FOR

    RETURN emotionalHistory
END
```

### Finding Similar Emotional States

```
ALGORITHM: findSimilarStates
INPUT: targetState (EmotionalState), topK (integer)
OUTPUT: Promise<Array<EmotionalState>>

BEGIN
    // Step 1: Use AgentDB vector similarity search
    // Emotional states with similar emotion vectors
    query ← {
        collection: "emotional_states",
        vector: targetState.emotionVector,
        topK: topK,
        includeDistance: true
    }

    // Step 2: Execute similarity search
    results ← AWAIT agentDBClient.vectorSearch(query)

    // Step 3: Filter out self (if querying existing state)
    similarStates ← []
    FOR EACH result IN results DO
        IF result.id != targetState.emotionalStateId THEN
            similarStates.append({
                state: convertToEmotionalState(result.data),
                similarity: 1 - result.distance  // Convert distance to similarity
            })
        END IF
    END FOR

    RETURN similarStates
END
```

---

## Example Input/Output Scenarios

### Scenario 1: Happy User

**Input:**
```
text: "I just got promoted at work! I'm so excited and grateful for this opportunity!"
userId: "user_12345"
```

**Expected Output:**
```
EmotionalState {
  emotionalStateId: "550e8400-e29b-41d4-a716-446655440000",
  userId: "user_12345",
  valence: 0.92,           // Very positive
  arousal: 0.78,           // High energy
  primaryEmotion: "joy",
  emotionVector: [
    0.5714,  // joy (primary)
    0.0000,  // sadness (suppressed)
    0.0000,  // anger
    0.0000,  // fear
    0.2857,  // trust (adjacent)
    0.0000,  // disgust
    0.0000,  // surprise
    0.1429   // anticipation (adjacent)
  ],
  stressLevel: 0.28,       // Low stress (excited, not anxious)
  confidence: 0.95,        // High confidence
  timestamp: 1735916400000,
  rawText: "I just got promoted at work! I'm so excited and grateful..."
}
```

### Scenario 2: Stressed User

**Input:**
```
text: "I have three deadlines tomorrow and I haven't slept in 30 hours. I don't know if I can do this."
userId: "user_67890"
```

**Expected Output:**
```
EmotionalState {
  emotionalStateId: "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  userId: "user_67890",
  valence: -0.68,          // Negative
  arousal: 0.85,           // Very high arousal
  primaryEmotion: "fear",
  emotionVector: [
    0.0000,  // joy
    0.1304,  // sadness (adjacent)
    0.0000,  // anger (suppressed)
    0.6522,  // fear (primary)
    0.1304,  // trust (adjacent)
    0.0000,  // disgust
    0.0870,  // surprise
    0.0000   // anticipation
  ],
  stressLevel: 0.96,       // Very high stress
  confidence: 0.91,
  timestamp: 1735916460000,
  rawText: "I have three deadlines tomorrow and I haven't slept..."
}
```

### Scenario 3: Calm User

**Input:**
```
text: "Just finished my morning meditation. Feeling peaceful and ready for the day."
userId: "user_11111"
```

**Expected Output:**
```
EmotionalState {
  emotionalStateId: "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  userId: "user_11111",
  valence: 0.65,           // Positive
  arousal: -0.42,          // Low arousal (calm)
  primaryEmotion: "trust",
  emotionVector: [
    0.2500,  // joy (adjacent)
    0.0000,  // sadness
    0.0000,  // anger
    0.2500,  // fear (adjacent)
    0.5000,  // trust (primary)
    0.0000,  // disgust (suppressed)
    0.0000,  // surprise
    0.0000   // anticipation
  ],
  stressLevel: 0.08,       // Very low stress
  confidence: 0.88,
  timestamp: 1735916520000,
  rawText: "Just finished my morning meditation. Feeling peaceful..."
}
```

### Scenario 4: API Timeout Fallback

**Input:**
```
text: "This is a test text"
userId: "user_99999"
// Gemini API times out after 30 seconds
```

**Expected Output:**
```
EmotionalState {
  emotionalStateId: "fb5c8a1d-9e23-4b67-8901-234567890abc",
  userId: "user_99999",
  valence: 0.0,            // Neutral
  arousal: 0.0,            // Neutral
  primaryEmotion: "trust",
  emotionVector: [
    0.125,   // joy
    0.125,   // sadness
    0.125,   // anger
    0.125,   // fear
    0.125,   // trust
    0.125,   // disgust
    0.125,   // surprise
    0.125    // anticipation (uniform distribution)
  ],
  stressLevel: 0.5,        // Unknown/medium
  confidence: 0.0,         // Zero confidence (fallback)
  timestamp: 1735916580000,
  rawText: ""
}

// Logged Warning:
"Creating fallback emotional state due to API failure (timeout after 3 retries)"
```

---

## Implementation Checklist

- [ ] Set up Gemini API client with authentication
- [ ] Implement timeout mechanism (30s limit)
- [ ] Create retry logic with exponential backoff
- [ ] Implement valence-arousal mapping to Russell's Circumplex
- [ ] Build Plutchik 8D emotion vector generator
- [ ] Develop stress level calculation algorithm
- [ ] Create confidence scoring system
- [ ] Implement fallback state generation
- [ ] Add comprehensive input validation
- [ ] Build AgentDB integration for emotional history
- [ ] Create vector similarity search for similar states
- [ ] Add structured logging for debugging
- [ ] Write unit tests for each algorithm (95% coverage target)
- [ ] Create integration tests with Gemini API
- [ ] Add performance benchmarks (target: <5s per analysis)
- [ ] Document API rate limit handling
- [ ] Implement caching layer (optional optimization)

---

## Performance Targets

| Metric                    | Target        | Measurement                          |
|---------------------------|---------------|--------------------------------------|
| Average response time     | < 3 seconds   | End-to-end (including API call)      |
| P95 response time         | < 5 seconds   | 95th percentile latency              |
| API success rate          | > 98%         | Successful responses / total calls   |
| Fallback rate             | < 2%          | Fallback states / total analyses     |
| Confidence (average)      | > 0.8         | Mean confidence across all analyses  |
| AgentDB save success      | > 99.5%       | Successful saves / total attempts    |
| Memory footprint          | < 50 MB       | Peak memory usage per instance       |
| Throughput                | 20 req/min    | Limited by Gemini API rate limits    |

---

## Future Enhancements

1. **Batch Processing**: Analyze multiple texts in single API call
2. **Real-time Streaming**: WebSocket support for live emotion tracking
3. **Multi-language Support**: Detect and analyze non-English text
4. **Context Awareness**: Use conversation history for better accuracy
5. **Custom Emotion Models**: Fine-tune Gemini for specific domains
6. **Edge Case Detection**: Identify sarcasm, irony, mixed emotions
7. **Temporal Analysis**: Track emotional trends over time
8. **Personalization**: Learn user-specific emotional patterns

---

**Document Status**: Complete
**Review Status**: Pending architecture review
**Next Phase**: SPARC Architecture (system design)
