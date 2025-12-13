//
// MediaInteractionTests.swift
// VibeCheckTests
//
// TDD tests for MediaInteraction model and service
// SPARC Phase 2: RED - Tests written before implementation
//

import XCTest
import SwiftData
@testable import VibeCheck

// MARK: - MediaInteraction Model Tests

@available(iOS 17.0, *)
class MediaInteractionModelTests: XCTestCase {

    var modelContainer: ModelContainer!
    var modelContext: ModelContext!

    override func setUp() async throws {
        try await super.setUp()

        // In-memory SwiftData container for testing
        let schema = Schema([MediaInteraction.self])
        let config = ModelConfiguration(isStoredInMemoryOnly: true)
        modelContainer = try ModelContainer(for: schema, configurations: [config])
        modelContext = ModelContext(modelContainer)
    }

    override func tearDown() async throws {
        modelContainer = nil
        modelContext = nil
        try await super.tearDown()
    }

    // MARK: - Rating Tests

    func testCreateInteractionWithThumbsUp() throws {
        // Given: A media item ID
        let mediaId = "movie-123"

        // When: Creating interaction with thumbs up
        let interaction = MediaInteraction(
            mediaId: mediaId,
            mediaTitle: "Test Movie",
            rating: .thumbsUp
        )

        // Then: Should have correct rating
        XCTAssertEqual(interaction.mediaId, mediaId)
        XCTAssertEqual(interaction.rating, .thumbsUp)
        XCTAssertFalse(interaction.hasSeen)
    }

    func testCreateInteractionWithThumbsDown() throws {
        // Given/When: Creating interaction with thumbs down
        let interaction = MediaInteraction(
            mediaId: "movie-456",
            mediaTitle: "Bad Movie",
            rating: .thumbsDown
        )

        // Then: Should have thumbs down rating
        XCTAssertEqual(interaction.rating, .thumbsDown)
    }

    func testInteractionDefaultsToNoRating() throws {
        // When: Creating interaction without explicit rating
        let interaction = MediaInteraction(
            mediaId: "movie-789",
            mediaTitle: "Neutral Movie"
        )

        // Then: Rating should be nil
        XCTAssertNil(interaction.rating)
    }

    func testToggleRating() throws {
        // Given: Interaction with no rating
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie"
        )

        // When: Setting thumbs up
        interaction.rating = .thumbsUp
        XCTAssertEqual(interaction.rating, .thumbsUp)

        // When: Toggling same rating (should clear)
        interaction.toggleRating(.thumbsUp)
        XCTAssertNil(interaction.rating)

        // When: Setting thumbs down
        interaction.toggleRating(.thumbsDown)
        XCTAssertEqual(interaction.rating, .thumbsDown)

        // When: Switching to thumbs up
        interaction.toggleRating(.thumbsUp)
        XCTAssertEqual(interaction.rating, .thumbsUp)
    }

    // MARK: - Seen Status Tests (alternate name: "Watched", "Viewed", "Consumed")

    func testMarkAsSeen() throws {
        // Given: Unseen interaction
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie"
        )
        XCTAssertFalse(interaction.hasSeen)

        // When: Marking as seen
        interaction.markAsSeen()

        // Then: Should be marked as seen with timestamp
        XCTAssertTrue(interaction.hasSeen)
        XCTAssertNotNil(interaction.seenAt)
    }

    func testMarkAsUnseen() throws {
        // Given: Seen interaction
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie"
        )
        interaction.markAsSeen()
        XCTAssertTrue(interaction.hasSeen)

        // When: Marking as unseen
        interaction.markAsUnseen()

        // Then: Should be unseen
        XCTAssertFalse(interaction.hasSeen)
        XCTAssertNil(interaction.seenAt)
    }

    func testToggleSeen() throws {
        // Given: Unseen interaction
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie"
        )

        // When: Toggling seen
        interaction.toggleSeen()
        XCTAssertTrue(interaction.hasSeen)

        // When: Toggling again
        interaction.toggleSeen()
        XCTAssertFalse(interaction.hasSeen)
    }

    // MARK: - Persistence Tests

    func testPersistInteraction() throws {
        // Given: An interaction
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie",
            rating: .thumbsUp
        )
        interaction.markAsSeen()

        // When: Saving to context
        modelContext.insert(interaction)
        try modelContext.save()

        // Then: Should be retrievable
        let descriptor = FetchDescriptor<MediaInteraction>(
            predicate: #Predicate { $0.mediaId == "movie-123" }
        )
        let fetched = try modelContext.fetch(descriptor)

        XCTAssertEqual(fetched.count, 1)
        XCTAssertEqual(fetched.first?.rating, .thumbsUp)
        XCTAssertTrue(fetched.first?.hasSeen ?? false)
    }

    func testUpdateExistingInteraction() throws {
        // Given: Persisted interaction
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie"
        )
        modelContext.insert(interaction)
        try modelContext.save()

        // When: Updating rating
        interaction.rating = .thumbsDown
        try modelContext.save()

        // Then: Change should persist
        let descriptor = FetchDescriptor<MediaInteraction>(
            predicate: #Predicate { $0.mediaId == "movie-123" }
        )
        let fetched = try modelContext.fetch(descriptor)

        XCTAssertEqual(fetched.first?.rating, .thumbsDown)
    }

    func testFindOrCreateInteraction() throws {
        // Given: No existing interaction
        let mediaId = "movie-new"

        // When: Finding or creating
        let interaction = try MediaInteraction.findOrCreate(
            mediaId: mediaId,
            mediaTitle: "New Movie",
            in: modelContext
        )

        // Then: Should create new
        XCTAssertEqual(interaction.mediaId, mediaId)

        // When: Finding again
        let found = try MediaInteraction.findOrCreate(
            mediaId: mediaId,
            mediaTitle: "New Movie",
            in: modelContext
        )

        // Then: Should return same instance
        XCTAssertEqual(interaction.id, found.id)
    }

    // MARK: - Learning Score Tests

    func testLearningScoreForThumbsUp() throws {
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie",
            rating: .thumbsUp
        )

        // Thumbs up should have positive learning score
        XCTAssertEqual(interaction.learningScore, 1.0, accuracy: 0.01)
    }

    func testLearningScoreForThumbsDown() throws {
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie",
            rating: .thumbsDown
        )

        // Thumbs down should have negative learning score
        XCTAssertEqual(interaction.learningScore, -1.0, accuracy: 0.01)
    }

    func testLearningScoreForSeenOnly() throws {
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie"
        )
        interaction.markAsSeen()

        // Seen without rating should have moderate positive score
        XCTAssertEqual(interaction.learningScore, 0.5, accuracy: 0.01)
    }

    func testLearningScoreForNoInteraction() throws {
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie"
        )

        // No interaction should have zero score
        XCTAssertEqual(interaction.learningScore, 0.0, accuracy: 0.01)
    }

    // MARK: - Mood Context Tests

    func testInteractionStoresMoodContext() throws {
        // Given: Interaction with mood context
        let interaction = MediaInteraction(
            mediaId: "movie-123",
            mediaTitle: "Test Movie",
            rating: .thumbsUp,
            moodHint: "comfort"
        )

        // Then: Mood hint should be stored
        XCTAssertEqual(interaction.moodHint, "comfort")
    }
}

// MARK: - Rating Enum Tests

@available(iOS 17.0, *)
class RatingTypeTests: XCTestCase {

    func testRatingRawValues() {
        XCTAssertEqual(Rating.thumbsUp.rawValue, "thumbsUp")
        XCTAssertEqual(Rating.thumbsDown.rawValue, "thumbsDown")
    }

    func testRatingFeedbackType() {
        // Thumbs up maps to .liked
        XCTAssertEqual(Rating.thumbsUp.feedbackType, FeedbackType.liked)

        // Thumbs down maps to .disliked
        XCTAssertEqual(Rating.thumbsDown.feedbackType, FeedbackType.disliked)
    }

    func testRatingIcon() {
        XCTAssertEqual(Rating.thumbsUp.iconName, "hand.thumbsup.fill")
        XCTAssertEqual(Rating.thumbsDown.iconName, "hand.thumbsdown.fill")
    }
}
