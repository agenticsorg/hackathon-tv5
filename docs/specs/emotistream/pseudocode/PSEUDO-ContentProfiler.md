# EmotiStream Nexus - Content Profiler Pseudocode

## Component Overview

The Content Profiler analyzes content metadata using Gemini API to generate emotional profiles and embeddings for semantic search in RuVector.

---

## Data Structures

### ContentMetadata
```
STRUCTURE ContentMetadata:
    contentId: String
    title: String
    description: String
    platform: String = "mock"
    genres: Array<String>
    category: Enum<'movie', 'series', 'documentary', 'music', 'meditation', 'short'>
    tags: Array<String>
    duration: Integer (in minutes)
END STRUCTURE
```

### EmotionalContentProfile
```
STRUCTURE EmotionalContentProfile:
    contentId: String
    primaryTone: String
    valenceDelta: Float (-1.0 to +1.0)
    arousalDelta: Float (-1.0 to +1.0)
    intensity: Float (0.0 to 1.0)
    complexity: Float (0.0 to 1.0)
    targetStates: Array<TargetState>
    embeddingId: String
    timestamp: Integer (Unix timestamp in ms)
END STRUCTURE

STRUCTURE TargetState:
    currentValence: Float (-1.0 to +1.0)
    currentArousal: Float (-1.0 to +1.0)
    description: String
END STRUCTURE
```

### RuVectorEntry
```
STRUCTURE RuVectorEntry:
    id: String (contentId)
    embedding: Float32Array[1536]
    metadata: Object {
        contentId: String,
        title: String,
        primaryTone: String,
        valenceDelta: Float,
        arousalDelta: Float,
        intensity: Float,
        complexity: Float,
        genres: Array<String>,
        category: String,
        duration: Integer
    }
END STRUCTURE
```

### BatchProcessingState
```
STRUCTURE BatchProcessingState:
    totalItems: Integer
    processedItems: Integer
    failedItems: Array<String>  // contentIds that failed
    currentBatch: Integer
    totalBatches: Integer
    startTime: Integer
    estimatedCompletion: Integer (timestamp)
END STRUCTURE
```

---

## Constants and Configuration

```
CONSTANTS:
    BATCH_SIZE = 10
    MAX_RETRIES = 3
    RETRY_DELAY_MS = 2000
    GEMINI_RATE_LIMIT_PER_MINUTE = 60
    GEMINI_TIMEOUT_MS = 30000

    EMBEDDING_DIMENSIONS = 1536
    HNSW_M = 16
    HNSW_EF_CONSTRUCTION = 200

    AGENTDB_TABLE = "emotional_content_profiles"
    RUVECTOR_COLLECTION = "content_embeddings"

    MEMORY_NAMESPACE = "emotistream/content-profiler"
END CONSTANTS
```

---

## Main Algorithms

### 1. Batch Content Profiling

```
ALGORITHM: BatchProfileContent
INPUT: contents (Array<ContentMetadata>), batchSize (Integer)
OUTPUT: ProfileResult {success: Integer, failed: Integer, errors: Array}

BEGIN
    // Initialize tracking
    state ← CreateBatchState(contents.length, batchSize)
    results ← {success: 0, failed: 0, errors: []}

    // Initialize storage
    InitializeAgentDBTable(AGENTDB_TABLE)
    InitializeRuVectorCollection(RUVECTOR_COLLECTION)

    // Split into batches
    batches ← SplitIntoBatches(contents, batchSize)
    state.totalBatches ← batches.length

    // Process each batch
    FOR EACH batch IN batches DO
        state.currentBatch ← state.currentBatch + 1

        LOG("Processing batch " + state.currentBatch + "/" + state.totalBatches)

        // Process batch items in parallel
        batchPromises ← []

        FOR EACH content IN batch DO
            promise ← ProcessSingleContent(content)
            batchPromises.append(promise)
        END FOR

        // Wait for batch completion
        batchResults ← AwaitAll(batchPromises)

        // Analyze batch results
        FOR EACH result IN batchResults DO
            IF result.success THEN
                results.success ← results.success + 1
                state.processedItems ← state.processedItems + 1
            ELSE
                results.failed ← results.failed + 1
                state.failedItems.append(result.contentId)
                results.errors.append({
                    contentId: result.contentId,
                    error: result.error,
                    timestamp: GetCurrentTime()
                })
            END IF
        END FOR

        // Rate limiting between batches
        IF state.currentBatch < state.totalBatches THEN
            // Calculate delay to respect rate limits
            itemsPerMinute ← GEMINI_RATE_LIMIT_PER_MINUTE
            delayMs ← CalculateRateLimitDelay(batchSize, itemsPerMinute)
            Sleep(delayMs)
        END IF

        // Update progress
        UpdateProgress(state)
    END FOR

    // Retry failed items
    IF state.failedItems.length > 0 THEN
        LOG("Retrying " + state.failedItems.length + " failed items")
        retryResults ← RetryFailedItems(state.failedItems, contents)

        results.success ← results.success + retryResults.success
        results.failed ← results.failed + retryResults.failed
        results.errors ← results.errors.concat(retryResults.errors)
    END IF

    // Store final state in memory
    StoreMemory(
        MEMORY_NAMESPACE + "/batch-results",
        results,
        ttl: 3600
    )

    RETURN results
END
```

### 2. Process Single Content Item

```
ALGORITHM: ProcessSingleContent
INPUT: content (ContentMetadata)
OUTPUT: ProcessResult {success: Boolean, contentId: String, error: String}

BEGIN
    retryCount ← 0
    lastError ← null

    WHILE retryCount < MAX_RETRIES DO
        TRY
            // Step 1: Generate emotional profile using Gemini
            profile ← ProfileContentWithGemini(content)

            // Step 2: Generate embedding vector
            embedding ← GenerateEmotionEmbedding(profile, content)

            // Step 3: Store profile in AgentDB
            StoreProfileInAgentDB(profile)

            // Step 4: Store embedding in RuVector
            embeddingId ← StoreEmbeddingInRuVector(
                content.contentId,
                embedding,
                CreateEmbeddingMetadata(profile, content)
            )

            // Step 5: Update profile with embedding ID
            profile.embeddingId ← embeddingId
            UpdateProfileInAgentDB(profile)

            // Success
            RETURN {
                success: true,
                contentId: content.contentId,
                error: null
            }

        CATCH error
            lastError ← error
            retryCount ← retryCount + 1

            IF retryCount < MAX_RETRIES THEN
                LOG("Retry " + retryCount + " for content " + content.contentId)
                Sleep(RETRY_DELAY_MS * retryCount) // Exponential backoff
            END IF
        END TRY
    END WHILE

    // All retries failed
    RETURN {
        success: false,
        contentId: content.contentId,
        error: lastError.message
    }
END
```

### 3. Gemini-Based Content Profiling

```
ALGORITHM: ProfileContentWithGemini
INPUT: content (ContentMetadata)
OUTPUT: EmotionalContentProfile

BEGIN
    // Build prompt
    prompt ← ConstructGeminiPrompt(content)

    // Call Gemini API
    requestBody ← {
        contents: [{
            parts: [{text: prompt}]
        }],
        generationConfig: {
            temperature: 0.7,
            topK: 40,
            topP: 0.95,
            maxOutputTokens: 1024,
            responseMimeType: "application/json"
        }
    }

    // Make API call with timeout
    response ← CallGeminiAPI(
        model: "gemini-1.5-flash",
        body: requestBody,
        timeout: GEMINI_TIMEOUT_MS
    )

    // Parse response
    IF response.status != 200 THEN
        THROW Error("Gemini API error: " + response.status)
    END IF

    // Extract JSON from response
    geminiResult ← ParseGeminiResponse(response)

    // Validate response structure
    ValidateGeminiResult(geminiResult)

    // Create emotional profile
    profile ← EmotionalContentProfile{
        contentId: content.contentId,
        primaryTone: geminiResult.primaryTone,
        valenceDelta: Clamp(geminiResult.valenceDelta, -1.0, 1.0),
        arousalDelta: Clamp(geminiResult.arousalDelta, -1.0, 1.0),
        intensity: Clamp(geminiResult.intensity, 0.0, 1.0),
        complexity: Clamp(geminiResult.complexity, 0.0, 1.0),
        targetStates: geminiResult.targetStates,
        embeddingId: null,  // Will be set after RuVector storage
        timestamp: GetCurrentTime()
    }

    RETURN profile
END

SUBROUTINE: ConstructGeminiPrompt
INPUT: content (ContentMetadata)
OUTPUT: prompt (String)

BEGIN
    prompt ← "Analyze the emotional impact of this content:\n\n"
    prompt ← prompt + "Title: " + content.title + "\n"
    prompt ← prompt + "Description: " + content.description + "\n"
    prompt ← prompt + "Genres: " + Join(content.genres, ", ") + "\n"
    prompt ← prompt + "Category: " + content.category + "\n"
    prompt ← prompt + "Tags: " + Join(content.tags, ", ") + "\n"
    prompt ← prompt + "Duration: " + content.duration + " minutes\n\n"

    prompt ← prompt + "Provide:\n"
    prompt ← prompt + "1. Primary emotional tone (calm, uplifting, thrilling, melancholic, cathartic, etc.)\n"
    prompt ← prompt + "2. Valence delta: expected change in viewer's valence (-1 to +1)\n"
    prompt ← prompt + "3. Arousal delta: expected change in viewer's arousal (-1 to +1)\n"
    prompt ← prompt + "4. Emotional intensity: 0 (subtle) to 1 (intense)\n"
    prompt ← prompt + "5. Emotional complexity: 0 (simple) to 1 (nuanced, mixed emotions)\n"
    prompt ← prompt + "6. Target viewer states: which emotional states is this content good for?\n\n"

    prompt ← prompt + "Format as JSON:\n"
    prompt ← prompt + "{\n"
    prompt ← prompt + '  "primaryTone": "...",\n'
    prompt ← prompt + '  "valenceDelta": 0.0,\n'
    prompt ← prompt + '  "arousalDelta": 0.0,\n'
    prompt ← prompt + '  "intensity": 0.0,\n'
    prompt ← prompt + '  "complexity": 0.0,\n'
    prompt ← prompt + '  "targetStates": [\n'
    prompt ← prompt + '    {"currentValence": 0.0, "currentArousal": 0.0, "description": "..."}\n'
    prompt ← prompt + '  ]\n'
    prompt ← prompt + "}"

    RETURN prompt
END

SUBROUTINE: ParseGeminiResponse
INPUT: response (APIResponse)
OUTPUT: geminiResult (Object)

BEGIN
    // Extract text from Gemini response structure
    IF NOT response.candidates OR response.candidates.length = 0 THEN
        THROW Error("No candidates in Gemini response")
    END IF

    candidate ← response.candidates[0]

    IF NOT candidate.content OR NOT candidate.content.parts THEN
        THROW Error("Invalid response structure")
    END IF

    textContent ← candidate.content.parts[0].text

    // Parse JSON (Gemini returns JSON in code blocks sometimes)
    cleanedText ← RemoveCodeBlockMarkers(textContent)
    geminiResult ← ParseJSON(cleanedText)

    RETURN geminiResult
END

SUBROUTINE: ValidateGeminiResult
INPUT: result (Object)
OUTPUT: void (throws error if invalid)

BEGIN
    requiredFields ← [
        "primaryTone",
        "valenceDelta",
        "arousalDelta",
        "intensity",
        "complexity",
        "targetStates"
    ]

    FOR EACH field IN requiredFields DO
        IF NOT result.hasProperty(field) THEN
            THROW Error("Missing required field: " + field)
        END IF
    END FOR

    // Validate numeric ranges
    IF NOT IsInRange(result.valenceDelta, -1.0, 1.0) THEN
        THROW Error("valenceDelta out of range")
    END IF

    IF NOT IsInRange(result.arousalDelta, -1.0, 1.0) THEN
        THROW Error("arousalDelta out of range")
    END IF

    IF NOT IsInRange(result.intensity, 0.0, 1.0) THEN
        THROW Error("intensity out of range")
    END IF

    IF NOT IsInRange(result.complexity, 0.0, 1.0) THEN
        THROW Error("complexity out of range")
    END IF

    // Validate targetStates array
    IF NOT IsArray(result.targetStates) OR result.targetStates.length = 0 THEN
        THROW Error("targetStates must be non-empty array")
    END IF
END
```

### 4. Emotion Embedding Generation

```
ALGORITHM: GenerateEmotionEmbedding
INPUT: profile (EmotionalContentProfile), content (ContentMetadata)
OUTPUT: embedding (Float32Array[1536])

BEGIN
    // Initialize embedding vector
    embedding ← Float32Array[EMBEDDING_DIMENSIONS]
    FillWithZeros(embedding)

    // Encoding strategy: Use different segments of the 1536D vector
    // Segment 1 (0-255): Primary tone encoding
    // Segment 2 (256-511): Valence/arousal encoding
    // Segment 3 (512-767): Intensity/complexity encoding
    // Segment 4 (768-1023): Target states encoding
    // Segment 5 (1024-1279): Genre/category encoding
    // Segment 6 (1280-1535): Reserved for future use

    // Segment 1: Encode primary tone (one-hot style)
    toneIndex ← GetToneIndex(profile.primaryTone)
    embedding[toneIndex] ← 1.0

    // Segment 2: Encode valence/arousal deltas
    // Map valence delta (-1 to +1) to dimensions 256-383
    EncodeRangeValue(
        embedding,
        startIdx: 256,
        endIdx: 383,
        value: profile.valenceDelta,
        minValue: -1.0,
        maxValue: 1.0
    )

    // Map arousal delta (-1 to +1) to dimensions 384-511
    EncodeRangeValue(
        embedding,
        startIdx: 384,
        endIdx: 511,
        value: profile.arousalDelta,
        minValue: -1.0,
        maxValue: 1.0
    )

    // Segment 3: Encode intensity and complexity
    EncodeRangeValue(
        embedding,
        startIdx: 512,
        endIdx: 639,
        value: profile.intensity,
        minValue: 0.0,
        maxValue: 1.0
    )

    EncodeRangeValue(
        embedding,
        startIdx: 640,
        endIdx: 767,
        value: profile.complexity,
        minValue: 0.0,
        maxValue: 1.0
    )

    // Segment 4: Encode target states
    // Use first 3 target states, encode each as valence+arousal pair
    targetStartIdx ← 768
    FOR i ← 0 TO MIN(profile.targetStates.length - 1, 2) DO
        state ← profile.targetStates[i]

        // Valence of target state
        EncodeRangeValue(
            embedding,
            startIdx: targetStartIdx + (i * 86),
            endIdx: targetStartIdx + (i * 86) + 42,
            value: state.currentValence,
            minValue: -1.0,
            maxValue: 1.0
        )

        // Arousal of target state
        EncodeRangeValue(
            embedding,
            startIdx: targetStartIdx + (i * 86) + 43,
            endIdx: targetStartIdx + (i * 86) + 85,
            value: state.currentArousal,
            minValue: -1.0,
            maxValue: 1.0
        )
    END FOR

    // Segment 5: Encode genres and category
    genreStartIdx ← 1024
    FOR EACH genre IN content.genres DO
        genreIdx ← GetGenreIndex(genre)
        IF genreIdx >= 0 AND genreIdx < 128 THEN
            embedding[genreStartIdx + genreIdx] ← 1.0
        END IF
    END FOR

    categoryIdx ← GetCategoryIndex(content.category)
    embedding[genreStartIdx + 128 + categoryIdx] ← 1.0

    // Normalize embedding to unit length
    embedding ← NormalizeVector(embedding)

    RETURN embedding
END

SUBROUTINE: EncodeRangeValue
INPUT: embedding (Float32Array), startIdx (Int), endIdx (Int),
       value (Float), minValue (Float), maxValue (Float)
OUTPUT: void (modifies embedding)

BEGIN
    // Normalize value to 0-1 range
    normalized ← (value - minValue) / (maxValue - minValue)

    // Use Gaussian-like encoding for smooth transitions
    rangeSize ← endIdx - startIdx + 1
    center ← normalized * rangeSize
    sigma ← rangeSize / 6.0  // Standard deviation

    FOR i ← 0 TO rangeSize - 1 DO
        distance ← i - center
        gaussianValue ← Exp(-(distance * distance) / (2 * sigma * sigma))
        embedding[startIdx + i] ← gaussianValue
    END FOR
END

SUBROUTINE: NormalizeVector
INPUT: vector (Float32Array)
OUTPUT: normalized (Float32Array)

BEGIN
    // Calculate magnitude
    magnitude ← 0.0
    FOR EACH value IN vector DO
        magnitude ← magnitude + (value * value)
    END FOR
    magnitude ← SquareRoot(magnitude)

    // Avoid division by zero
    IF magnitude = 0.0 THEN
        RETURN vector
    END IF

    // Normalize
    normalized ← Float32Array[vector.length]
    FOR i ← 0 TO vector.length - 1 DO
        normalized[i] ← vector[i] / magnitude
    END FOR

    RETURN normalized
END
```

### 5. RuVector Storage

```
ALGORITHM: StoreEmbeddingInRuVector
INPUT: contentId (String), embedding (Float32Array), metadata (Object)
OUTPUT: embeddingId (String)

BEGIN
    // Ensure RuVector collection exists
    collection ← GetOrCreateCollection(RUVECTOR_COLLECTION)

    // Upsert embedding with metadata
    embeddingId ← collection.upsert({
        id: contentId,
        embedding: embedding,
        metadata: metadata
    })

    // Verify storage
    IF NOT embeddingId THEN
        THROW Error("Failed to store embedding in RuVector")
    END IF

    RETURN embeddingId
END

SUBROUTINE: GetOrCreateCollection
INPUT: collectionName (String)
OUTPUT: collection (RuVectorCollection)

BEGIN
    TRY
        collection ← RuVector.getCollection(collectionName)
        RETURN collection
    CATCH error
        // Collection doesn't exist, create it
        collection ← RuVector.createCollection({
            name: collectionName,
            dimension: EMBEDDING_DIMENSIONS,
            indexType: "hnsw",
            indexConfig: {
                m: HNSW_M,
                efConstruction: HNSW_EF_CONSTRUCTION
            },
            metric: "cosine"
        })

        RETURN collection
    END TRY
END

SUBROUTINE: CreateEmbeddingMetadata
INPUT: profile (EmotionalContentProfile), content (ContentMetadata)
OUTPUT: metadata (Object)

BEGIN
    metadata ← {
        contentId: content.contentId,
        title: content.title,
        primaryTone: profile.primaryTone,
        valenceDelta: profile.valenceDelta,
        arousalDelta: profile.arousalDelta,
        intensity: profile.intensity,
        complexity: profile.complexity,
        genres: content.genres,
        category: content.category,
        duration: content.duration,
        tags: content.tags,
        platform: content.platform,
        timestamp: profile.timestamp
    }

    RETURN metadata
END
```

### 6. AgentDB Storage

```
ALGORITHM: StoreProfileInAgentDB
INPUT: profile (EmotionalContentProfile)
OUTPUT: void

BEGIN
    // Serialize profile to JSON
    profileData ← {
        contentId: profile.contentId,
        primaryTone: profile.primaryTone,
        valenceDelta: profile.valenceDelta,
        arousalDelta: profile.arousalDelta,
        intensity: profile.intensity,
        complexity: profile.complexity,
        targetStates: SerializeTargetStates(profile.targetStates),
        embeddingId: profile.embeddingId,
        timestamp: profile.timestamp
    }

    // Store in AgentDB
    AgentDB.insert(AGENTDB_TABLE, profileData)

    // Log success
    LOG("Stored profile for content: " + profile.contentId)
END

ALGORITHM: GetContentProfile
INPUT: contentId (String)
OUTPUT: profile (EmotionalContentProfile) or null

BEGIN
    // Query AgentDB
    results ← AgentDB.query(
        AGENTDB_TABLE,
        where: {contentId: contentId}
    )

    IF results.length = 0 THEN
        RETURN null
    END IF

    // Deserialize first result
    data ← results[0]
    profile ← EmotionalContentProfile{
        contentId: data.contentId,
        primaryTone: data.primaryTone,
        valenceDelta: data.valenceDelta,
        arousalDelta: data.arousalDelta,
        intensity: data.intensity,
        complexity: data.complexity,
        targetStates: DeserializeTargetStates(data.targetStates),
        embeddingId: data.embeddingId,
        timestamp: data.timestamp
    }

    RETURN profile
END
```

### 7. Semantic Search by Emotional Transition

```
ALGORITHM: SearchByEmotionalTransition
INPUT: currentState (EmotionalState), desiredState (EmotionalState), topK (Integer)
OUTPUT: recommendations (Array<ContentWithScore>)

BEGIN
    // Create transition query vector
    transitionVector ← CreateTransitionVector(currentState, desiredState)

    // Search RuVector
    collection ← RuVector.getCollection(RUVECTOR_COLLECTION)
    results ← collection.search({
        embedding: transitionVector,
        topK: topK,
        includeMetadata: true
    })

    // Enrich results with full profiles
    recommendations ← []
    FOR EACH result IN results DO
        profile ← GetContentProfile(result.id)

        IF profile THEN
            recommendations.append({
                contentId: result.id,
                metadata: result.metadata,
                profile: profile,
                similarityScore: result.score,
                relevanceReason: ExplainRelevance(
                    currentState,
                    desiredState,
                    profile
                )
            })
        END IF
    END FOR

    RETURN recommendations
END

SUBROUTINE: CreateTransitionVector
INPUT: currentState (EmotionalState), desiredState (EmotionalState)
OUTPUT: transitionVector (Float32Array[1536])

BEGIN
    // Calculate desired deltas
    valenceDelta ← desiredState.valence - currentState.valence
    arousalDelta ← desiredState.arousal - currentState.arousal

    // Create pseudo-profile for the desired transition
    pseudoProfile ← EmotionalContentProfile{
        contentId: "query",
        primaryTone: InferToneFromTransition(valenceDelta, arousalDelta),
        valenceDelta: valenceDelta,
        arousalDelta: arousalDelta,
        intensity: CalculateIntensity(valenceDelta, arousalDelta),
        complexity: 0.5,  // Neutral complexity preference
        targetStates: [{
            currentValence: currentState.valence,
            currentArousal: currentState.arousal,
            description: "current state"
        }],
        embeddingId: null,
        timestamp: GetCurrentTime()
    }

    // Create dummy content for encoding
    dummyContent ← ContentMetadata{
        contentId: "query",
        title: "",
        description: "",
        platform: "mock",
        genres: [],
        category: "movie",
        tags: [],
        duration: 0
    }

    // Generate embedding using same algorithm
    transitionVector ← GenerateEmotionEmbedding(pseudoProfile, dummyContent)

    RETURN transitionVector
END

SUBROUTINE: InferToneFromTransition
INPUT: valenceDelta (Float), arousalDelta (Float)
OUTPUT: tone (String)

BEGIN
    // Classify transition into quadrants
    IF valenceDelta > 0 AND arousalDelta > 0 THEN
        RETURN "uplifting"  // Increasing valence + arousal
    ELSE IF valenceDelta > 0 AND arousalDelta < 0 THEN
        RETURN "calming"    // Increasing valence, decreasing arousal
    ELSE IF valenceDelta < 0 AND arousalDelta > 0 THEN
        RETURN "thrilling"  // Decreasing valence, increasing arousal
    ELSE IF valenceDelta < 0 AND arousalDelta < 0 THEN
        RETURN "melancholic" // Decreasing both
    ELSE
        RETURN "neutral"
    END IF
END

SUBROUTINE: CalculateIntensity
INPUT: valenceDelta (Float), arousalDelta (Float)
OUTPUT: intensity (Float)

BEGIN
    // Magnitude of emotional change
    magnitude ← SquareRoot(valenceDelta^2 + arousalDelta^2)

    // Normalize to 0-1 range (max magnitude is sqrt(2))
    intensity ← magnitude / 1.414

    RETURN Clamp(intensity, 0.0, 1.0)
END
```

---

## Mock Content Catalog Structure

```
ALGORITHM: GenerateMockContentCatalog
INPUT: count (Integer)
OUTPUT: catalog (Array<ContentMetadata>)

BEGIN
    catalog ← []

    // Define content templates by category
    templates ← GetContentTemplates()

    FOR i ← 1 TO count DO
        // Select random category
        category ← RandomChoice([
            "movie", "series", "documentary",
            "music", "meditation", "short"
        ])

        // Get template for category
        template ← templates[category]

        // Generate content
        content ← ContentMetadata{
            contentId: "mock_" + category + "_" + i,
            title: GenerateTitle(category, i),
            description: GenerateDescription(category, template),
            platform: "mock",
            genres: RandomSample(template.genres, 2, 4),
            category: category,
            tags: RandomSample(template.tags, 3, 6),
            duration: RandomInt(template.minDuration, template.maxDuration)
        }

        catalog.append(content)
    END FOR

    RETURN catalog
END

SUBROUTINE: GetContentTemplates
OUTPUT: templates (Map<String, Template>)

BEGIN
    templates ← {
        "movie": {
            genres: ["drama", "comedy", "thriller", "romance", "action", "sci-fi"],
            tags: ["emotional", "thought-provoking", "feel-good", "intense", "inspiring"],
            minDuration: 90,
            maxDuration: 180
        },
        "series": {
            genres: ["drama", "comedy", "crime", "fantasy", "mystery"],
            tags: ["binge-worthy", "character-driven", "plot-twist", "episodic"],
            minDuration: 30,
            maxDuration: 60
        },
        "documentary": {
            genres: ["nature", "history", "science", "biographical", "social"],
            tags: ["educational", "eye-opening", "inspiring", "thought-provoking"],
            minDuration: 45,
            maxDuration: 120
        },
        "music": {
            genres: ["classical", "jazz", "ambient", "world", "electronic"],
            tags: ["relaxing", "energizing", "meditative", "uplifting", "atmospheric"],
            minDuration: 3,
            maxDuration: 60
        },
        "meditation": {
            genres: ["guided", "ambient", "nature-sounds", "mindfulness"],
            tags: ["calming", "stress-relief", "sleep", "focus", "breathing"],
            minDuration: 5,
            maxDuration: 45
        },
        "short": {
            genres: ["animation", "comedy", "experimental", "musical"],
            tags: ["quick-watch", "creative", "fun", "bite-sized"],
            minDuration: 1,
            maxDuration: 15
        }
    }

    RETURN templates
END
```

---

## Example Emotional Profiles

### Profile 1: Calming Nature Documentary

```
EmotionalContentProfile {
    contentId: "mock_documentary_001",
    primaryTone: "serene",
    valenceDelta: +0.3,        // Slight positive shift
    arousalDelta: -0.5,        // Significant calming effect
    intensity: 0.3,            // Gentle, not overwhelming
    complexity: 0.4,           // Simple, peaceful emotions
    targetStates: [
        {
            currentValence: -0.2,
            currentArousal: 0.6,
            description: "Stressed, anxious - good for unwinding"
        },
        {
            currentValence: 0.0,
            currentArousal: 0.3,
            description: "Neutral but restless - helps find peace"
        }
    ],
    embeddingId: "emb_001",
    timestamp: 1733395200000
}
```

### Profile 2: Uplifting Comedy

```
EmotionalContentProfile {
    contentId: "mock_movie_045",
    primaryTone: "uplifting",
    valenceDelta: +0.6,        // Strong positive shift
    arousalDelta: +0.2,        // Slight energy boost
    intensity: 0.6,            // Moderately intense joy
    complexity: 0.5,           // Mix of humor and heart
    targetStates: [
        {
            currentValence: -0.5,
            currentArousal: -0.3,
            description: "Sad, low energy - needs mood boost"
        },
        {
            currentValence: 0.0,
            currentArousal: 0.0,
            description: "Neutral - wants entertainment"
        }
    ],
    embeddingId: "emb_045",
    timestamp: 1733395200000
}
```

### Profile 3: Intense Thriller

```
EmotionalContentProfile {
    contentId: "mock_movie_089",
    primaryTone: "thrilling",
    valenceDelta: -0.1,        // Slight negative (tension)
    arousalDelta: +0.7,        // High arousal increase
    intensity: 0.9,            // Very intense experience
    complexity: 0.7,           // Complex emotional journey
    targetStates: [
        {
            currentValence: 0.2,
            currentArousal: -0.4,
            description: "Bored, needs excitement"
        },
        {
            currentValence: 0.0,
            currentArousal: -0.2,
            description: "Low energy, wants stimulation"
        }
    ],
    embeddingId: "emb_089",
    timestamp: 1733395200000
}
```

### Profile 4: Meditation Session

```
EmotionalContentProfile {
    contentId: "mock_meditation_012",
    primaryTone: "calm",
    valenceDelta: +0.2,        // Gentle positive shift
    arousalDelta: -0.8,        // Strong calming effect
    intensity: 0.2,            // Very subtle, gentle
    complexity: 0.1,           // Simple, focused calm
    targetStates: [
        {
            currentValence: -0.4,
            currentArousal: 0.7,
            description: "Anxious, stressed - needs deep calm"
        },
        {
            currentValence: 0.0,
            currentArousal: 0.5,
            description: "Can't sleep, mind racing"
        },
        {
            currentValence: 0.2,
            currentArousal: 0.4,
            description: "Wants to relax and center"
        }
    ],
    embeddingId: "emb_012",
    timestamp: 1733395200000
}
```

---

## Complexity Analysis

### Time Complexity

**BatchProfileContent**:
- Input: n content items, batch size b
- Batches: ⌈n/b⌉
- Per item: O(G + E + S) where:
  - G = Gemini API call (network + processing)
  - E = Embedding generation (1536 dimensions)
  - S = Storage operations (AgentDB + RuVector)
- Total: O(n * (G + E + S))
- With parallelization within batches: O((n/b) * (G + E + S))

**GenerateEmotionEmbedding**:
- Encoding operations: O(d) where d = 1536 dimensions
- Normalization: O(d)
- Total: O(d) = O(1536) = O(1) (constant dimension)

**SearchByEmotionalTransition**:
- Create query vector: O(d)
- HNSW search: O(log n) where n = number of embeddings
- Retrieve profiles: O(k) where k = topK results
- Total: O(log n + k)

### Space Complexity

**Per Content Item**:
- Profile object: O(1) fixed fields + O(t) target states
- Embedding: O(1536) = O(1)
- Metadata: O(1)
- Total: O(1) per item

**Batch Processing**:
- Batch array: O(b) where b = batch size
- State tracking: O(1)
- Results: O(n) where n = total items
- Total: O(n) overall

**RuVector Collection**:
- n embeddings × 1536 dimensions
- HNSW index overhead: ~O(n * M) where M = 16
- Total: O(n) with constant factor

---

## Error Handling

```
ERROR SCENARIOS:

1. Gemini API Failures:
   - Timeout: Retry with exponential backoff
   - Rate limit: Add delay between batches
   - Invalid response: Log and skip item
   - JSON parse error: Retry with cleaner prompt

2. Storage Failures:
   - AgentDB connection lost: Retry with backoff
   - RuVector unavailable: Queue for later storage
   - Disk full: Stop processing, alert

3. Validation Failures:
   - Invalid profile values: Clamp to valid ranges
   - Missing required fields: Use defaults
   - Empty targetStates: Generate from deltas

4. Resource Exhaustion:
   - Memory limit: Reduce batch size
   - CPU throttling: Add delays between batches
   - Network congestion: Increase timeouts
```

---

## Performance Optimization Notes

1. **Batch Processing**: 10 items per batch balances throughput and error isolation
2. **Rate Limiting**: Respect Gemini's 60 requests/minute limit
3. **Parallel Encoding**: Embedding generation can be parallelized within batches
4. **HNSW Index**: M=16, efConstruction=200 balances build time and search quality
5. **Caching**: Store frequently accessed profiles in memory
6. **Lazy Loading**: Don't load all embeddings at once, use streaming queries

---

## Integration Points

1. **EmotionalState Tracker**: Provides currentState for search queries
2. **RecommendationEngine**: Consumes search results for content suggestions
3. **AgentDB**: Persistent storage for profiles
4. **RuVector**: Semantic search over emotion embeddings
5. **Gemini API**: External LLM for content analysis
6. **Mock Content Service**: Provides content catalog for profiling

---

## Implementation Checklist

- [ ] Implement ContentMetadata type
- [ ] Implement EmotionalContentProfile type
- [ ] Build Gemini API client with retry logic
- [ ] Implement batch processing with rate limiting
- [ ] Create embedding generation algorithm
- [ ] Integrate RuVector with HNSW configuration
- [ ] Set up AgentDB table for profiles
- [ ] Build semantic search by transition
- [ ] Generate mock content catalog (200 items)
- [ ] Create unit tests for embedding encoding
- [ ] Test batch processing with mock API
- [ ] Validate search results quality
- [ ] Document all public APIs
- [ ] Add error logging and monitoring
- [ ] Performance test with full catalog

---

**Document Version**: 1.0
**Created**: 2025-12-05
**SPARC Phase**: Pseudocode (Phase 2)
**Component**: Content Profiler
**Dependencies**: Gemini API, AgentDB, RuVector
