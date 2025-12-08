import Foundation

// Mock UserPreferences matching the API used by RecommendationEngine
class UserPreferences {
    var favoriteGenres: [String] = []
    var avoidGenres: [String] = []
    var avoidTitles: [String] = []
    var preferredMinRuntime: Int? = nil
    var preferredMaxRuntime: Int? = nil
    var subscriptions: [String] = ["netflix", "hulu", "apple", "max", "prime"] // Default all

    init() {}
}
