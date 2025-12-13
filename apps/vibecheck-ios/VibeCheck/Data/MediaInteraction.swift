//
// MediaInteraction.swift
// VibeCheck
//
// SwiftData model for tracking user interactions with media content.
// Supports thumbs up/down ratings and "seen" status tracking.
// Integrates with LearningMemoryService for WASM-based personalization.
//

import Foundation
import SwiftData

// MARK: - Rating Type

/// User rating for media content
enum Rating: String, Codable, CaseIterable {
    case thumbsUp = "thumbsUp"
    case thumbsDown = "thumbsDown"

    /// Map to existing FeedbackType for learning system integration
    var feedbackType: FeedbackType {
        switch self {
        case .thumbsUp: return .liked
        case .thumbsDown: return .disliked
        }
    }

    /// SF Symbol icon name
    var iconName: String {
        switch self {
        case .thumbsUp: return "hand.thumbsup.fill"
        case .thumbsDown: return "hand.thumbsdown.fill"
        }
    }

    /// Unselected SF Symbol icon name
    var iconNameOutline: String {
        switch self {
        case .thumbsUp: return "hand.thumbsup"
        case .thumbsDown: return "hand.thumbsdown"
        }
    }

    /// Learning score (-1.0 to 1.0)
    var learningScore: Float {
        return feedbackType.learningScore
    }

    /// Accessibility label
    var accessibilityLabel: String {
        switch self {
        case .thumbsUp: return "Like"
        case .thumbsDown: return "Dislike"
        }
    }
}

// MARK: - MediaInteraction Model

/// Tracks user interactions with media content for personalization.
/// Stores ratings (thumbs up/down) and seen status.
@available(iOS 17.0, *)
@Model
class MediaInteraction {
    // MARK: - Properties

    /// Unique identifier for the media item
    @Attribute(.unique) var mediaId: String

    /// Title of the media (for display without re-fetching)
    var mediaTitle: String

    /// User's rating (nil = no rating)
    var ratingRawValue: String?

    /// Whether the user has seen/watched this content
    var hasSeen: Bool

    /// Timestamp when marked as seen (nil if not seen)
    var seenAt: Date?

    /// Mood context when interaction occurred
    var moodHint: String?

    /// Timestamp when the interaction was created
    var createdAt: Date

    /// Timestamp of the last update
    var updatedAt: Date

    // MARK: - Computed Properties

    /// Typed rating accessor
    var rating: Rating? {
        get {
            guard let rawValue = ratingRawValue else { return nil }
            return Rating(rawValue: rawValue)
        }
        set {
            ratingRawValue = newValue?.rawValue
            updatedAt = Date()
        }
    }

    /// Learning score for recommendation engine
    /// - thumbsUp: 1.0
    /// - thumbsDown: -1.0
    /// - seen (no rating): 0.5
    /// - no interaction: 0.0
    var learningScore: Float {
        if let rating = rating {
            return rating.learningScore
        } else if hasSeen {
            return FeedbackType.watched.learningScore
        }
        return 0.0
    }

    /// Feedback type for LearningMemoryService
    var feedbackType: FeedbackType? {
        if let rating = rating {
            return rating.feedbackType
        } else if hasSeen {
            return .watched
        }
        return nil
    }

    // MARK: - Initialization

    init(
        mediaId: String,
        mediaTitle: String,
        rating: Rating? = nil,
        hasSeen: Bool = false,
        moodHint: String? = nil
    ) {
        self.mediaId = mediaId
        self.mediaTitle = mediaTitle
        self.ratingRawValue = rating?.rawValue
        self.hasSeen = hasSeen
        self.moodHint = moodHint
        self.createdAt = Date()
        self.updatedAt = Date()
        self.seenAt = hasSeen ? Date() : nil
    }

    // MARK: - Actions

    /// Toggle rating - if same rating, clear it; otherwise set new rating
    func toggleRating(_ newRating: Rating) {
        if rating == newRating {
            rating = nil
        } else {
            rating = newRating
        }
    }

    /// Mark content as seen
    func markAsSeen() {
        hasSeen = true
        seenAt = Date()
        updatedAt = Date()
    }

    /// Mark content as unseen
    func markAsUnseen() {
        hasSeen = false
        seenAt = nil
        updatedAt = Date()
    }

    /// Toggle seen status
    func toggleSeen() {
        if hasSeen {
            markAsUnseen()
        } else {
            markAsSeen()
        }
    }

    // MARK: - Static Helpers

    /// Find existing interaction or create new one
    @MainActor
    static func findOrCreate(
        mediaId: String,
        mediaTitle: String,
        in context: ModelContext
    ) throws -> MediaInteraction {
        let descriptor = FetchDescriptor<MediaInteraction>(
            predicate: #Predicate { $0.mediaId == mediaId }
        )

        if let existing = try context.fetch(descriptor).first {
            return existing
        }

        let new = MediaInteraction(mediaId: mediaId, mediaTitle: mediaTitle)
        context.insert(new)
        return new
    }

    /// Fetch interaction by media ID
    @MainActor
    static func find(
        mediaId: String,
        in context: ModelContext
    ) throws -> MediaInteraction? {
        let descriptor = FetchDescriptor<MediaInteraction>(
            predicate: #Predicate { $0.mediaId == mediaId }
        )
        return try context.fetch(descriptor).first
    }

    /// Fetch all interactions with a specific rating
    @MainActor
    static func fetchByRating(
        _ rating: Rating,
        in context: ModelContext
    ) throws -> [MediaInteraction] {
        let ratingRaw = rating.rawValue
        let descriptor = FetchDescriptor<MediaInteraction>(
            predicate: #Predicate { $0.ratingRawValue == ratingRaw },
            sortBy: [SortDescriptor(\.updatedAt, order: .reverse)]
        )
        return try context.fetch(descriptor)
    }

    /// Fetch all seen interactions
    @MainActor
    static func fetchSeen(
        in context: ModelContext
    ) throws -> [MediaInteraction] {
        let descriptor = FetchDescriptor<MediaInteraction>(
            predicate: #Predicate { $0.hasSeen == true },
            sortBy: [SortDescriptor(\.seenAt, order: .reverse)]
        )
        return try context.fetch(descriptor)
    }

    /// Fetch interactions for multiple media IDs
    @MainActor
    static func fetchMultiple(
        mediaIds: [String],
        in context: ModelContext
    ) throws -> [MediaInteraction] {
        let descriptor = FetchDescriptor<MediaInteraction>(
            predicate: #Predicate { mediaIds.contains($0.mediaId) }
        )
        return try context.fetch(descriptor)
    }
}

// MARK: - Extensions for MediaItem

extension MediaItem {
    /// Create a MediaInteraction for this item
    @available(iOS 17.0, *)
    func createInteraction(
        rating: Rating? = nil,
        hasSeen: Bool = false,
        moodHint: String? = nil
    ) -> MediaInteraction {
        MediaInteraction(
            mediaId: id,
            mediaTitle: title,
            rating: rating,
            hasSeen: hasSeen,
            moodHint: moodHint
        )
    }
}
