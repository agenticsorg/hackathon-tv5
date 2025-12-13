//
// InteractionServiceTests.swift
// VibeCheckTests
//
// TDD tests for InteractionService - WASM learning integration
// SPARC Phase 4: RED - Tests written before implementation
//

import XCTest
import SwiftData
@testable import VibeCheck

@available(iOS 17.0, *)
class InteractionServiceTests: XCTestCase {

    var modelContainer: ModelContainer!
    var modelContext: ModelContext!
    var service: InteractionService!

    override func setUp() async throws {
        try await super.setUp()

        // In-memory SwiftData container
        let schema = Schema([MediaInteraction.self, WatchHistory.self, WatchlistItem.self])
        let config = ModelConfiguration(isStoredInMemoryOnly: true)
        modelContainer = try ModelContainer(for: schema, configurations: [config])
        modelContext = ModelContext(modelContainer)

        // Create service (will use mock bridge in tests)
        service = InteractionService(modelContext: modelContext)
    }

    override func tearDown() async throws {
        service = nil
        modelContainer = nil
        modelContext = nil
        try await super.tearDown()
    }

    // MARK: - Rating Operations

    func testRateMediaThumbsUp() async throws {
        // Given: A media item and mood
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState(energy: .moderate, stress: .neutral)

        // When: Rating thumbs up
        let interaction = try await service.rate(
            mediaItem: mediaItem,
            rating: .thumbsUp,
            mood: mood
        )

        // Then: Interaction should be created with rating
        XCTAssertEqual(interaction.mediaId, mediaItem.id)
        XCTAssertEqual(interaction.rating, .thumbsUp)
    }

    func testRateMediaThumbsDown() async throws {
        // Given: A media item and mood
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState(energy: .low, stress: .stressed)

        // When: Rating thumbs down
        let interaction = try await service.rate(
            mediaItem: mediaItem,
            rating: .thumbsDown,
            mood: mood
        )

        // Then: Interaction should have thumbs down
        XCTAssertEqual(interaction.rating, .thumbsDown)
    }

    func testToggleRatingClearsIfSame() async throws {
        // Given: Existing thumbs up interaction
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState.chill
        _ = try await service.rate(mediaItem: mediaItem, rating: .thumbsUp, mood: mood)

        // When: Rating thumbs up again (toggle)
        let interaction = try await service.toggleRating(
            mediaItem: mediaItem,
            rating: .thumbsUp,
            mood: mood
        )

        // Then: Rating should be cleared
        XCTAssertNil(interaction.rating)
    }

    func testToggleRatingSwitchesRating() async throws {
        // Given: Existing thumbs up interaction
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState.chill
        _ = try await service.rate(mediaItem: mediaItem, rating: .thumbsUp, mood: mood)

        // When: Toggling to thumbs down
        let interaction = try await service.toggleRating(
            mediaItem: mediaItem,
            rating: .thumbsDown,
            mood: mood
        )

        // Then: Rating should switch
        XCTAssertEqual(interaction.rating, .thumbsDown)
    }

    // MARK: - Seen Status Operations

    func testMarkAsSeen() async throws {
        // Given: A media item
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState.chill

        // When: Marking as seen
        let interaction = try await service.markAsSeen(
            mediaItem: mediaItem,
            mood: mood
        )

        // Then: Should be marked as seen
        XCTAssertTrue(interaction.hasSeen)
        XCTAssertNotNil(interaction.seenAt)
    }

    func testMarkAsUnseen() async throws {
        // Given: Seen interaction
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState.chill
        _ = try await service.markAsSeen(mediaItem: mediaItem, mood: mood)

        // When: Marking as unseen
        let interaction = try await service.markAsUnseen(mediaItem: mediaItem)

        // Then: Should be unseen
        XCTAssertFalse(interaction.hasSeen)
    }

    func testToggleSeen() async throws {
        // Given: Unseen media
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState.chill

        // When: Toggling seen
        let interaction1 = try await service.toggleSeen(mediaItem: mediaItem, mood: mood)
        XCTAssertTrue(interaction1.hasSeen)

        // When: Toggling again
        let interaction2 = try await service.toggleSeen(mediaItem: mediaItem, mood: mood)
        XCTAssertFalse(interaction2.hasSeen)
    }

    // MARK: - WASM Learning Integration

    func testRatingTriggersLearning() async throws {
        // Given: Service with learning enabled
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState(energy: .high, stress: .relaxed)

        // When: Rating thumbs up
        _ = try await service.rate(
            mediaItem: mediaItem,
            rating: .thumbsUp,
            mood: mood
        )

        // Then: Learning should be triggered (verified via mock or stats)
        // Note: In real implementation, this verifies LearningMemoryService was called
        let stats = await service.getLearningStats()
        XCTAssertGreaterThanOrEqual(stats.totalInteractions, 1)
    }

    func testSeenTriggersLearning() async throws {
        // Given: Service with learning enabled
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState.chill

        // When: Marking as seen
        _ = try await service.markAsSeen(mediaItem: mediaItem, mood: mood)

        // Then: Learning should be triggered with .watched feedback
        let stats = await service.getLearningStats()
        XCTAssertGreaterThanOrEqual(stats.seenCount, 1)
    }

    // MARK: - Fetch Operations

    func testGetInteraction() async throws {
        // Given: Existing interaction
        let mediaItem = MediaItem.samples.first!
        _ = try await service.rate(
            mediaItem: mediaItem,
            rating: .thumbsUp,
            mood: MoodState.chill
        )

        // When: Getting interaction
        let interaction = try await service.getInteraction(for: mediaItem.id)

        // Then: Should return the interaction
        XCTAssertNotNil(interaction)
        XCTAssertEqual(interaction?.rating, .thumbsUp)
    }

    func testGetInteractionReturnsNilIfNotFound() async throws {
        // When: Getting non-existent interaction
        let interaction = try await service.getInteraction(for: "non-existent-id")

        // Then: Should return nil
        XCTAssertNil(interaction)
    }

    func testGetAllLikedMedia() async throws {
        // Given: Multiple interactions
        let items = Array(MediaItem.samples.prefix(3))
        let mood = MoodState.chill

        _ = try await service.rate(mediaItem: items[0], rating: .thumbsUp, mood: mood)
        _ = try await service.rate(mediaItem: items[1], rating: .thumbsDown, mood: mood)
        _ = try await service.rate(mediaItem: items[2], rating: .thumbsUp, mood: mood)

        // When: Getting liked media
        let liked = try await service.getAllLiked()

        // Then: Should return only thumbs up items
        XCTAssertEqual(liked.count, 2)
        XCTAssertTrue(liked.allSatisfy { $0.rating == .thumbsUp })
    }

    func testGetAllSeenMedia() async throws {
        // Given: Multiple interactions
        let items = Array(MediaItem.samples.prefix(3))
        let mood = MoodState.chill

        _ = try await service.markAsSeen(mediaItem: items[0], mood: mood)
        _ = try await service.markAsSeen(mediaItem: items[1], mood: mood)
        // items[2] not marked as seen

        // When: Getting seen media
        let seen = try await service.getAllSeen()

        // Then: Should return only seen items
        XCTAssertEqual(seen.count, 2)
        XCTAssertTrue(seen.allSatisfy { $0.hasSeen })
    }

    // MARK: - Batch Operations

    func testGetInteractionsForMultipleMedia() async throws {
        // Given: Multiple interactions
        let items = Array(MediaItem.samples.prefix(3))
        let mood = MoodState.chill

        for item in items {
            _ = try await service.rate(mediaItem: item, rating: .thumbsUp, mood: mood)
        }

        // When: Getting interactions for specific IDs
        let mediaIds = items.map { $0.id }
        let interactions = try await service.getInteractions(for: mediaIds)

        // Then: Should return all matching interactions
        XCTAssertEqual(interactions.count, 3)
    }

    // MARK: - Statistics

    func testLearningStats() async throws {
        // Given: Various interactions
        let items = Array(MediaItem.samples.prefix(4))
        let mood = MoodState.chill

        _ = try await service.rate(mediaItem: items[0], rating: .thumbsUp, mood: mood)
        _ = try await service.rate(mediaItem: items[1], rating: .thumbsDown, mood: mood)
        _ = try await service.markAsSeen(mediaItem: items[2], mood: mood)
        _ = try await service.markAsSeen(mediaItem: items[3], mood: mood)

        // When: Getting stats
        let stats = await service.getLearningStats()

        // Then: Stats should reflect interactions
        XCTAssertEqual(stats.thumbsUpCount, 1)
        XCTAssertEqual(stats.thumbsDownCount, 1)
        XCTAssertEqual(stats.seenCount, 2)
        XCTAssertEqual(stats.totalInteractions, 4)
    }
}

// MARK: - Integration with Existing Systems

@available(iOS 17.0, *)
class InteractionWatchlistIntegrationTests: XCTestCase {

    var modelContainer: ModelContainer!
    var modelContext: ModelContext!
    var service: InteractionService!

    override func setUp() async throws {
        try await super.setUp()
        let schema = Schema([MediaInteraction.self, WatchHistory.self, WatchlistItem.self])
        let config = ModelConfiguration(isStoredInMemoryOnly: true)
        modelContainer = try ModelContainer(for: schema, configurations: [config])
        modelContext = ModelContext(modelContainer)
        service = InteractionService(modelContext: modelContext)
    }

    override func tearDown() async throws {
        service = nil
        modelContainer = nil
        modelContext = nil
        try await super.tearDown()
    }

    func testSeenCreatesWatchHistory() async throws {
        // Given: A media item
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState.chill

        // When: Marking as seen
        _ = try await service.markAsSeen(mediaItem: mediaItem, mood: mood)

        // Then: WatchHistory entry should be created
        let descriptor = FetchDescriptor<WatchHistory>(
            predicate: #Predicate { $0.mediaId == mediaItem.id }
        )
        let history = try modelContext.fetch(descriptor)

        XCTAssertEqual(history.count, 1)
        XCTAssertEqual(history.first?.mediaTitle, mediaItem.title)
    }

    func testUnseenRemovesFromWatchHistory() async throws {
        // Given: Seen media with history
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState.chill
        _ = try await service.markAsSeen(mediaItem: mediaItem, mood: mood)

        // When: Marking as unseen
        _ = try await service.markAsUnseen(mediaItem: mediaItem)

        // Then: WatchHistory entry should be removed
        let descriptor = FetchDescriptor<WatchHistory>(
            predicate: #Predicate { $0.mediaId == mediaItem.id }
        )
        let history = try modelContext.fetch(descriptor)

        XCTAssertEqual(history.count, 0)
    }

    func testInteractionDoesNotDuplicateWatchHistory() async throws {
        // Given: Already seen media
        let mediaItem = MediaItem.samples.first!
        let mood = MoodState.chill
        _ = try await service.markAsSeen(mediaItem: mediaItem, mood: mood)

        // When: Marking as seen again
        _ = try await service.markAsSeen(mediaItem: mediaItem, mood: mood)

        // Then: Should not duplicate history
        let descriptor = FetchDescriptor<WatchHistory>(
            predicate: #Predicate { $0.mediaId == mediaItem.id }
        )
        let history = try modelContext.fetch(descriptor)

        XCTAssertEqual(history.count, 1)
    }
}
