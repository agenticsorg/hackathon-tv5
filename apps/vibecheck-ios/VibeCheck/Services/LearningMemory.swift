//
// LearningMemory.swift
// VibeCheck
//
// Learning Memory system for storing (mood, media, feedback) tuples
// in a WASM HNSW vector index for on-device personalized recommendations.
//

import Foundation

// MARK: - Feedback Types

/// Types of user feedback on media content
enum FeedbackType: String, Codable, CaseIterable {
    case liked      // User explicitly liked (thumbs up, heart)
    case disliked   // User explicitly disliked (thumbs down)
    case watched    // User watched significant portion (>70%)
    case skipped    // User skipped within first 5 minutes
    case completed  // User finished the entire content
    case abandoned  // User stopped mid-way (30-70%)

    /// Numeric score for learning (-1.0 to 1.0)
    var learningScore: Float {
        switch self {
        case .liked:     return 1.0
        case .completed: return 0.8
        case .watched:   return 0.5
        case .abandoned: return -0.2
        case .skipped:   return -0.5
        case .disliked:  return -1.0
        }
    }

    /// Weight for embedding influence
    var embeddingWeight: Float {
        switch self {
        case .liked:     return 1.5
        case .completed: return 1.2
        case .watched:   return 1.0
        case .abandoned: return 0.5
        case .skipped:   return 0.3
        case .disliked:  return 0.2
        }
    }
}

// MARK: - Mood Context Embedding

/// Captures mood state as a normalized vector component
struct MoodEmbedding: Codable, Equatable {
    let energy: Float      // 0.0 (exhausted) to 1.0 (wired)
    let stress: Float      // 0.0 (relaxed) to 1.0 (stressed)
    let confidence: Float  // Model confidence 0.0 to 1.0
    let timeOfDay: Float   // Normalized: 0.0 (midnight) to 1.0 (23:59)
    let dayOfWeek: Float   // 0.0 (Sunday) to 1.0 (Saturday)

    /// Create from MoodState
    init(from mood: MoodState, timestamp: Date = Date()) {
        self.energy = Float(mood.energy.fillAmount)
        self.stress = Float(mood.stress.fillAmount)
        self.confidence = Float(mood.confidence)

        let calendar = Calendar.current
        let hour = calendar.component(.hour, from: timestamp)
        let minute = calendar.component(.minute, from: timestamp)
        self.timeOfDay = Float(hour * 60 + minute) / 1440.0

        let weekday = calendar.component(.weekday, from: timestamp)
        self.dayOfWeek = Float(weekday - 1) / 6.0
    }

    /// Convert to array for vector operations
    var asArray: [Float] {
        [energy, stress, confidence, timeOfDay, dayOfWeek]
    }

    static let dimension: Int = 5
}

// MARK: - Learning Memory Entry

/// A single learning memory entry: (mood, media, feedback) tuple
struct LearningMemoryEntry: Codable, Identifiable {
    let id: UUID
    let timestamp: Date

    let moodEmbedding: MoodEmbedding
    let mediaId: String
    let mediaTitle: String
    let feedbackType: FeedbackType

    var combinedEmbedding: [Float]?
    var hnswId: Int32?

    let mediaGenres: [String]
    let mediaTones: [String]
    let watchDurationSeconds: Int?
    let completionPercent: Double?

    init(
        mood: MoodState,
        mediaItem: MediaItem,
        feedback: FeedbackType,
        watchDurationSeconds: Int? = nil,
        completionPercent: Double? = nil
    ) {
        self.id = UUID()
        self.timestamp = Date()
        self.moodEmbedding = MoodEmbedding(from: mood)
        self.mediaId = mediaItem.id
        self.mediaTitle = mediaItem.title
        self.feedbackType = feedback
        self.mediaGenres = mediaItem.genres
        self.mediaTones = mediaItem.tone
        self.watchDurationSeconds = watchDurationSeconds
        self.completionPercent = completionPercent
        self.combinedEmbedding = nil
        self.hnswId = nil
    }

    /// Total embedding dimension: 512 (NLEmbedding) + 5 (mood) = 517
    static let embeddingDimension: Int = 512 + MoodEmbedding.dimension
}

// MARK: - Similar Experience Result

struct SimilarExperience {
    let entry: LearningMemoryEntry
    let similarity: Float
    let moodSimilarity: Float
    let mediaSimilarity: Float

    var effectiveScore: Float {
        similarity * entry.feedbackType.learningScore
    }
}

// MARK: - Learned Preferences

struct LearnedPreferences {
    let genreScores: [String: Float]
    let toneScores: [String: Float]
    let sampleSize: Int

    var topGenres: [String] {
        genreScores.sorted { $0.value > $1.value }.prefix(5).map(\.key)
    }

    var topTones: [String] {
        toneScores.sorted { $0.value > $1.value }.prefix(5).map(\.key)
    }
}

// MARK: - Statistics

struct LearningMemoryStats {
    let totalEntries: Int
    let indexedVectors: Int
    let maxCapacity: Int
    let feedbackDistribution: [FeedbackType: Int]
    let oldestEntry: Date?
    let newestEntry: Date?

    var utilizationPercent: Double {
        guard maxCapacity > 0 else { return 0 }
        return Double(indexedVectors) / Double(maxCapacity) * 100
    }
}

// MARK: - Errors

enum LearningMemoryError: Error, LocalizedError {
    case notInitialized
    case embeddingFailed
    case indexInsertFailed
    case indexSearchFailed
    case wasmMemoryError

    var errorDescription: String? {
        switch self {
        case .notInitialized: return "Learning memory not initialized"
        case .embeddingFailed: return "Failed to generate embedding"
        case .indexInsertFailed: return "Failed to insert into HNSW index"
        case .indexSearchFailed: return "Failed to search HNSW index"
        case .wasmMemoryError: return "WASM memory allocation failed"
        }
    }
}

// MARK: - Learning Memory Service

@available(iOS 15.0, *)
actor LearningMemoryService {

    static let shared = LearningMemoryService()

    private let embeddingService = VectorEmbeddingService.shared
    private var ruvectorBridge: RuvectorBridge?

    private var entries: [UUID: LearningMemoryEntry] = [:]
    private var isInitialized = false
    private var nextHnswId: Int32 = 0

    private init() {}

    // MARK: - Lifecycle

    func initialize(bridge: RuvectorBridge) async {
        self.ruvectorBridge = bridge
        self.isInitialized = bridge.isReady
        await loadPersistedEntries()

        // Index existing media items on initialization
        if isInitialized {
            await indexMediaCatalog()
        }
    }

    var isReady: Bool { isInitialized }

    // MARK: - Index Media Catalog

    /// Index all sample media items into HNSW for search
    private func indexMediaCatalog() async {
        guard let bridge = ruvectorBridge, bridge.isReady else { return }

        print("📚 LearningMemory: Indexing \(MediaItem.samples.count) media items...")

        for (index, item) in MediaItem.samples.enumerated() {
            // Create a neutral mood for catalog indexing
            let neutralMood = MoodState(energy: .moderate, stress: .neutral)

            do {
                // Generate embedding for this media item
                let embedding = try generateCombinedEmbedding(
                    mood: neutralMood,
                    mediaItem: item,
                    feedback: .watched // Neutral weight
                )

                // Insert into HNSW
                let success = bridge.insertVector(embedding, id: Int32(index))
                if success {
                    print("   ✅ Indexed: \(item.title) (id=\(index))")
                }
            } catch {
                print("   ❌ Failed to index \(item.title): \(error)")
            }
        }

        print("📚 LearningMemory: Catalog indexing complete. Vector count: \(bridge.getVectorCount())")
    }

    // MARK: - Record Feedback

    func recordFeedback(
        mood: MoodState,
        mediaItem: MediaItem,
        feedback: FeedbackType,
        watchDuration: Int? = nil,
        completion: Double? = nil
    ) async throws -> LearningMemoryEntry {

        guard isInitialized, let bridge = ruvectorBridge else {
            throw LearningMemoryError.notInitialized
        }

        var entry = LearningMemoryEntry(
            mood: mood,
            mediaItem: mediaItem,
            feedback: feedback,
            watchDurationSeconds: watchDuration,
            completionPercent: completion
        )

        // Generate combined embedding
        let embedding = try generateCombinedEmbedding(
            mood: mood,
            mediaItem: mediaItem,
            feedback: feedback
        )
        entry.combinedEmbedding = embedding

        // Insert into HNSW index
        let id = nextHnswId
        nextHnswId += 1

        // Offset by catalog size to avoid ID collisions
        let hnswId = id + Int32(MediaItem.samples.count)
        let success = bridge.insertVector(embedding, id: hnswId)

        if success {
            entry.hnswId = hnswId
            entries[entry.id] = entry
            await persistEntries()
            print("✅ LearningMemory: Recorded \(feedback.rawValue) for '\(mediaItem.title)'")
        } else {
            throw LearningMemoryError.indexInsertFailed
        }

        return entry
    }

    // MARK: - Find Similar

    func findSimilarExperiences(
        mood: MoodState,
        mediaHint: String? = nil,
        limit: Int = 10
    ) async throws -> [SimilarExperience] {

        guard isInitialized, let bridge = ruvectorBridge else {
            throw LearningMemoryError.notInitialized
        }

        // Generate query embedding
        let query = try generateQueryEmbedding(mood: mood, mediaHint: mediaHint)

        // Search HNSW index
        let results = bridge.searchHnsw(query: query, k: Int32(limit))

        // Map to experiences
        var experiences: [SimilarExperience] = []
        let queryMood = MoodEmbedding(from: mood)

        for (hnswId, similarity) in results {
            // Check if this is a catalog item
            if hnswId < Int32(MediaItem.samples.count) {
                let mediaItem = MediaItem.samples[Int(hnswId)]
                // Create synthetic entry for catalog items
                let entry = LearningMemoryEntry(
                    mood: mood,
                    mediaItem: mediaItem,
                    feedback: .watched
                )
                experiences.append(SimilarExperience(
                    entry: entry,
                    similarity: similarity,
                    moodSimilarity: 1.0,
                    mediaSimilarity: similarity
                ))
            } else if let entry = findEntry(byHnswId: hnswId) {
                let moodSim = calculateMoodSimilarity(query: queryMood, stored: entry.moodEmbedding)
                experiences.append(SimilarExperience(
                    entry: entry,
                    similarity: similarity,
                    moodSimilarity: moodSim,
                    mediaSimilarity: similarity - moodSim * 0.1
                ))
            }
        }

        return experiences.sorted { $0.effectiveScore > $1.effectiveScore }
    }

    // MARK: - Learned Preferences

    func getLearnedPreferences(for mood: MoodState) async throws -> LearnedPreferences {
        let experiences = try await findSimilarExperiences(mood: mood, limit: 50)

        var genreScores: [String: Float] = [:]
        var toneScores: [String: Float] = [:]

        for exp in experiences {
            let weight = exp.effectiveScore
            for genre in exp.entry.mediaGenres {
                genreScores[genre, default: 0] += weight
            }
            for tone in exp.entry.mediaTones {
                toneScores[tone, default: 0] += weight
            }
        }

        return LearnedPreferences(
            genreScores: genreScores,
            toneScores: toneScores,
            sampleSize: experiences.count
        )
    }

    // MARK: - Embedding Generation

    private func generateCombinedEmbedding(
        mood: MoodState,
        mediaItem: MediaItem,
        feedback: FeedbackType
    ) throws -> [Float] {

        guard let mediaEmbedding = embeddingService.embed(text: mediaItem.embeddingText) else {
            throw LearningMemoryError.embeddingFailed
        }

        let moodEmbed = MoodEmbedding(from: mood)
        let weight = feedback.embeddingWeight

        // Weight and combine
        var combined = mediaEmbedding.map { Float($0) * weight }
        combined.append(contentsOf: moodEmbed.asArray)

        // Normalize
        let norm = sqrt(combined.reduce(0) { $0 + $1 * $1 })
        if norm > 0 {
            combined = combined.map { $0 / norm }
        }

        return combined
    }

    private func generateQueryEmbedding(mood: MoodState, mediaHint: String?) throws -> [Float] {
        let mediaVector: [Float]
        if let hint = mediaHint, let embedding = embeddingService.embed(text: hint) {
            mediaVector = embedding.map { Float($0) }
        } else {
            mediaVector = [Float](repeating: 0, count: 512)
        }

        let moodEmbed = MoodEmbedding(from: mood)
        var combined = mediaVector
        combined.append(contentsOf: moodEmbed.asArray)

        let norm = sqrt(combined.reduce(0) { $0 + $1 * $1 })
        if norm > 0 {
            combined = combined.map { $0 / norm }
        }

        return combined
    }

    // MARK: - Helpers

    private func findEntry(byHnswId id: Int32) -> LearningMemoryEntry? {
        entries.values.first { $0.hnswId == id }
    }

    private func calculateMoodSimilarity(query: MoodEmbedding, stored: MoodEmbedding) -> Float {
        let q = query.asArray
        let s = stored.asArray

        var dot: Float = 0
        var normQ: Float = 0
        var normS: Float = 0

        for i in 0..<q.count {
            dot += q[i] * s[i]
            normQ += q[i] * q[i]
            normS += s[i] * s[i]
        }

        guard normQ > 0 && normS > 0 else { return 0 }
        return dot / (sqrt(normQ) * sqrt(normS))
    }

    // MARK: - Persistence

    private let persistenceKey = "learning_memory_entries_v1"

    private func persistEntries() async {
        let array = Array(entries.values)
        if let data = try? JSONEncoder().encode(array) {
            UserDefaults.standard.set(data, forKey: persistenceKey)
        }
    }

    private func loadPersistedEntries() async {
        guard let data = UserDefaults.standard.data(forKey: persistenceKey),
              let loaded = try? JSONDecoder().decode([LearningMemoryEntry].self, from: data) else {
            return
        }

        for entry in loaded {
            entries[entry.id] = entry
            if let id = entry.hnswId {
                nextHnswId = max(nextHnswId, id + 1)
            }
        }
    }

    // MARK: - Statistics

    var statistics: LearningMemoryStats {
        let feedbackCounts = Dictionary(
            grouping: entries.values,
            by: { $0.feedbackType }
        ).mapValues { $0.count }

        let vectorCount = ruvectorBridge?.getVectorCount() ?? 0

        return LearningMemoryStats(
            totalEntries: entries.count,
            indexedVectors: vectorCount,
            maxCapacity: 7000, // Approx max in 16MB WASM memory
            feedbackDistribution: feedbackCounts,
            oldestEntry: entries.values.map(\.timestamp).min(),
            newestEntry: entries.values.map(\.timestamp).max()
        )
    }
}
