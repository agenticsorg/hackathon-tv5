import Foundation
import SwiftData

@Model
class WatchHistory {
    var mediaId: String
    var mediaTitle: String
    var timestamp: Date
    var completionPercent: Double
    var moodHint: String?

    init(
        mediaId: String,
        mediaTitle: String,
        timestamp: Date = Date(),
        completionPercent: Double = 0,
        moodHint: String? = nil
    ) {
        self.mediaId = mediaId
        self.mediaTitle = mediaTitle
        self.timestamp = timestamp
        self.completionPercent = completionPercent
        self.moodHint = moodHint
    }
}

@Model
class UserPreferences {
    var favoriteGenres: [String]
    var avoidGenres: [String]
    var avoidTitles: [String]
    var preferredMinRuntime: Int?
    var preferredMaxRuntime: Int?
    var subscriptions: [String]

    init(
        favoriteGenres: [String] = [],
        avoidGenres: [String] = [],
        avoidTitles: [String] = [],
        preferredMinRuntime: Int? = nil,
        preferredMaxRuntime: Int? = nil,
        subscriptions: [String] = []
    ) {
        self.favoriteGenres = favoriteGenres
        self.avoidGenres = avoidGenres
        self.avoidTitles = avoidTitles
        self.preferredMinRuntime = preferredMinRuntime
        self.preferredMaxRuntime = preferredMaxRuntime
        self.subscriptions = subscriptions
    }

    static var `default`: UserPreferences {
        UserPreferences(
            favoriteGenres: ["comedy", "drama", "sci-fi"],
            avoidGenres: [],
            avoidTitles: [],
            subscriptions: ["netflix", "hulu", "apple", "max", "prime"]
        )
    }
}

@Model
class MoodLog {
    var timestamp: Date
    var energy: String
    var stress: String
    var hrv: Double?
    var sleepHours: Double?
    var steps: Double?
    var recommendationHint: String

    init(mood: MoodState, hrv: Double? = nil, sleepHours: Double? = nil, steps: Double? = nil) {
        self.timestamp = Date()
        self.energy = mood.energy.rawValue
        self.stress = mood.stress.rawValue
        self.hrv = hrv
        self.sleepHours = sleepHours
        self.steps = steps
        self.recommendationHint = mood.recommendationHint
    }
}

@Model
class WatchlistItem {
    var mediaId: String
    var mediaTitle: String
    var addedDate: Date
    var platform: String?
    var notes: String?

    init(mediaId: String, mediaTitle: String, platform: String? = nil, notes: String? = nil) {
        self.mediaId = mediaId
        self.mediaTitle = mediaTitle
        self.addedDate = Date()
        self.platform = platform
        self.notes = notes
    }
}


