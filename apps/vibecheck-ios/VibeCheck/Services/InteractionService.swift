//
// InteractionService.swift
// VibeCheck
//
// Service for managing user interactions with media content.
// Integrates with SwiftData for persistence and LearningMemoryService
// for WASM-based personalization via RuvectorBridge.
//

import Foundation
import SwiftData

// MARK: - Interaction Statistics

/// Statistics about user interactions for analytics and debugging
struct InteractionStats {
    let thumbsUpCount: Int
    let thumbsDownCount: Int
    let seenCount: Int
    let totalInteractions: Int

    var likeRatio: Double {
        guard thumbsUpCount + thumbsDownCount > 0 else { return 0 }
        return Double(thumbsUpCount) / Double(thumbsUpCount + thumbsDownCount)
    }
}

// MARK: - InteractionService

/// Manages user interactions with media content, providing persistence
/// and integration with the WASM-based learning system.
@available(iOS 17.0, *)
@MainActor
class InteractionService: ObservableObject {

    // MARK: - Properties

    private let modelContext: ModelContext
    private var learningMemory: LearningMemoryService?

    /// Published state for UI binding
    @Published private(set) var isLearningEnabled: Bool = false

    // MARK: - Initialization

    init(modelContext: ModelContext) {
        self.modelContext = modelContext
    }

    /// Initialize with learning memory service for WASM integration
    func initializeLearning(with learningMemory: LearningMemoryService) async {
        self.learningMemory = learningMemory
        self.isLearningEnabled = await learningMemory.isReady
    }

    // MARK: - Rating Operations

    /// Rate a media item with thumbs up or down
    /// - Parameters:
    ///   - mediaItem: The media item to rate
    ///   - rating: The rating (thumbsUp or thumbsDown)
    ///   - mood: Current mood state for learning context
    /// - Returns: The updated interaction
    @discardableResult
    func rate(
        mediaItem: MediaItem,
        rating: Rating,
        mood: MoodState
    ) async throws -> MediaInteraction {
        let interaction = try MediaInteraction.findOrCreate(
            mediaId: mediaItem.id,
            mediaTitle: mediaItem.title,
            in: modelContext
        )

        interaction.rating = rating
        interaction.moodHint = mood.recommendationHint

        try modelContext.save()

        // Trigger learning in WASM
        await triggerLearning(
            mediaItem: mediaItem,
            feedback: rating.feedbackType,
            mood: mood
        )

        return interaction
    }

    /// Toggle rating - same rating clears it, different rating switches
    /// - Parameters:
    ///   - mediaItem: The media item
    ///   - rating: The rating to toggle
    ///   - mood: Current mood state
    /// - Returns: The updated interaction
    @discardableResult
    func toggleRating(
        mediaItem: MediaItem,
        rating: Rating,
        mood: MoodState
    ) async throws -> MediaInteraction {
        let interaction = try MediaInteraction.findOrCreate(
            mediaId: mediaItem.id,
            mediaTitle: mediaItem.title,
            in: modelContext
        )

        let previousRating = interaction.rating
        interaction.toggleRating(rating)
        interaction.moodHint = mood.recommendationHint

        try modelContext.save()

        // Trigger learning based on new state
        if let newRating = interaction.rating {
            await triggerLearning(
                mediaItem: mediaItem,
                feedback: newRating.feedbackType,
                mood: mood
            )
        } else if previousRating != nil {
            // Rating was cleared - could trigger neutral feedback
            // For now, we don't send feedback when clearing
        }

        return interaction
    }

    // MARK: - Seen Status Operations

    /// Mark media as seen/watched
    /// - Parameters:
    ///   - mediaItem: The media item
    ///   - mood: Current mood state
    /// - Returns: The updated interaction
    @discardableResult
    func markAsSeen(
        mediaItem: MediaItem,
        mood: MoodState
    ) async throws -> MediaInteraction {
        let interaction = try MediaInteraction.findOrCreate(
            mediaId: mediaItem.id,
            mediaTitle: mediaItem.title,
            in: modelContext
        )

        // Only mark as seen if not already seen
        if !interaction.hasSeen {
            interaction.markAsSeen()
            interaction.moodHint = mood.recommendationHint

            // Create WatchHistory entry
            try createWatchHistoryEntry(
                mediaItem: mediaItem,
                mood: mood
            )

            try modelContext.save()

            // Trigger learning with .watched feedback
            await triggerLearning(
                mediaItem: mediaItem,
                feedback: .watched,
                mood: mood
            )
        }

        return interaction
    }

    /// Mark media as unseen
    /// - Parameter mediaItem: The media item
    /// - Returns: The updated interaction
    @discardableResult
    func markAsUnseen(mediaItem: MediaItem) async throws -> MediaInteraction {
        let interaction = try MediaInteraction.findOrCreate(
            mediaId: mediaItem.id,
            mediaTitle: mediaItem.title,
            in: modelContext
        )

        interaction.markAsUnseen()

        // Remove WatchHistory entry
        try removeWatchHistoryEntry(mediaId: mediaItem.id)

        try modelContext.save()

        return interaction
    }

    /// Toggle seen status
    /// - Parameters:
    ///   - mediaItem: The media item
    ///   - mood: Current mood state (used when marking as seen)
    /// - Returns: The updated interaction
    @discardableResult
    func toggleSeen(
        mediaItem: MediaItem,
        mood: MoodState
    ) async throws -> MediaInteraction {
        let interaction = try MediaInteraction.find(
            mediaId: mediaItem.id,
            in: modelContext
        )

        if let existing = interaction, existing.hasSeen {
            return try await markAsUnseen(mediaItem: mediaItem)
        } else {
            return try await markAsSeen(mediaItem: mediaItem, mood: mood)
        }
    }

    // MARK: - Fetch Operations

    /// Get interaction for a specific media item
    func getInteraction(for mediaId: String) async throws -> MediaInteraction? {
        return try MediaInteraction.find(mediaId: mediaId, in: modelContext)
    }

    /// Get interactions for multiple media items
    func getInteractions(for mediaIds: [String]) async throws -> [MediaInteraction] {
        return try MediaInteraction.fetchMultiple(mediaIds: mediaIds, in: modelContext)
    }

    /// Get all liked (thumbs up) interactions
    func getAllLiked() async throws -> [MediaInteraction] {
        return try MediaInteraction.fetchByRating(.thumbsUp, in: modelContext)
    }

    /// Get all disliked (thumbs down) interactions
    func getAllDisliked() async throws -> [MediaInteraction] {
        return try MediaInteraction.fetchByRating(.thumbsDown, in: modelContext)
    }

    /// Get all seen interactions
    func getAllSeen() async throws -> [MediaInteraction] {
        return try MediaInteraction.fetchSeen(in: modelContext)
    }

    // MARK: - Statistics

    /// Get learning statistics
    func getLearningStats() async -> InteractionStats {
        do {
            let liked = try await getAllLiked()
            let disliked = try await getAllDisliked()
            let seen = try await getAllSeen()

            // Calculate total unique interactions
            let allDescriptor = FetchDescriptor<MediaInteraction>()
            let total = try modelContext.fetchCount(allDescriptor)

            return InteractionStats(
                thumbsUpCount: liked.count,
                thumbsDownCount: disliked.count,
                seenCount: seen.count,
                totalInteractions: total
            )
        } catch {
            print("❌ InteractionService: Failed to get stats: \(error)")
            return InteractionStats(
                thumbsUpCount: 0,
                thumbsDownCount: 0,
                seenCount: 0,
                totalInteractions: 0
            )
        }
    }

    // MARK: - WASM Learning Integration

    /// Trigger learning in the WASM-based LearningMemoryService
    private func triggerLearning(
        mediaItem: MediaItem,
        feedback: FeedbackType,
        mood: MoodState
    ) async {
        guard let learningMemory = learningMemory, await learningMemory.isReady else {
            print("⚠️ InteractionService: Learning not available")
            return
        }

        do {
            _ = try await learningMemory.recordFeedback(
                mood: mood,
                mediaItem: mediaItem,
                feedback: feedback,
                watchDuration: nil,
                completion: feedback == .watched ? 0.7 : nil
            )
            print("✅ InteractionService: Recorded \(feedback.rawValue) for '\(mediaItem.title)'")
        } catch {
            print("❌ InteractionService: Learning failed: \(error)")
        }
    }

    // MARK: - WatchHistory Integration

    /// Create WatchHistory entry when marking as seen
    private func createWatchHistoryEntry(
        mediaItem: MediaItem,
        mood: MoodState
    ) throws {
        // Check if entry already exists
        let descriptor = FetchDescriptor<WatchHistory>(
            predicate: #Predicate { $0.mediaId == mediaItem.id }
        )

        if try modelContext.fetch(descriptor).isEmpty {
            let historyEntry = WatchHistory(
                mediaId: mediaItem.id,
                mediaTitle: mediaItem.title,
                completionPercent: 1.0, // Marked as seen = completed
                moodHint: mood.recommendationHint
            )
            modelContext.insert(historyEntry)
        }
    }

    /// Remove WatchHistory entry when marking as unseen
    private func removeWatchHistoryEntry(mediaId: String) throws {
        let descriptor = FetchDescriptor<WatchHistory>(
            predicate: #Predicate { $0.mediaId == mediaId }
        )

        let entries = try modelContext.fetch(descriptor)
        for entry in entries {
            modelContext.delete(entry)
        }
    }
}

// MARK: - Environment Key

@available(iOS 17.0, *)
struct InteractionServiceKey: EnvironmentKey {
    static let defaultValue: InteractionService? = nil
}

@available(iOS 17.0, *)
extension EnvironmentValues {
    var interactionService: InteractionService? {
        get { self[InteractionServiceKey.self] }
        set { self[InteractionServiceKey.self] = newValue }
    }
}
