import Foundation

struct MediaItem: Identifiable, Codable, Hashable {
    let id: String
    let title: String
    let overview: String
    let genres: [String]
    let tone: [String]
    let intensity: Double // 0.0 - 1.0
    let runtime: Int // minutes
    let year: Int
    let platforms: [String]
    let posterPath: String?
    let backdropPath: String?
    let rating: Double?
    let isRewatch: Bool
    
    // The Movie Sommelier's Note
    var sommelierRationale: String? = nil

    var posterURL: URL? {
        guard let path = posterPath else { return nil }
        return URL(string: "https://image.tmdb.org/t/p/w500\(path)")
    }

    var backdropURL: URL? {
        guard let path = backdropPath else { return nil }
        return URL(string: "https://image.tmdb.org/t/p/w780\(path)")
    }

    var formattedRuntime: String {
        if runtime < 60 {
            return "\(runtime)m"
        } else {
            let hours = runtime / 60
            let mins = runtime % 60
            return mins > 0 ? "\(hours)h \(mins)m" : "\(hours)h"
        }
    }

    init(
        id: String,
        title: String,
        overview: String = "",
        genres: [String] = [],
        tone: [String] = [],
        intensity: Double = 0.5,
        runtime: Int = 90,
        year: Int = 2024,
        platforms: [String] = [],
        posterPath: String? = nil,
        backdropPath: String? = nil,
        rating: Double? = nil,
        isRewatch: Bool = false
    ) {
        self.id = id
        self.title = title
        self.overview = overview
        self.genres = genres
        self.tone = tone
        self.intensity = intensity
        self.runtime = runtime
        self.year = year
        self.platforms = platforms
        self.posterPath = posterPath
        self.backdropPath = backdropPath
        self.rating = rating
        self.isRewatch = isRewatch
    }

    // MARK: - Semantic Search
    var semanticVector: [Double]? = nil

    /// Text used to generate the semantic embedding (Title + Overview + Tone + Genres)
    var embeddingText: String {
        let toneStr = tone.joined(separator: ", ")
        let genreStr = genres.joined(separator: ", ")
        return "\(title). \(overview) Mood: \(toneStr). Genres: \(genreStr)."
    }
}

// MARK: - Sample Data
extension MediaItem {
    static let samples: [MediaItem] = [
        MediaItem(
            id: "tt1",
            title: "The Bear",
            overview: "A young chef returns home to run his family's sandwich shop.",
            genres: ["drama", "comedy"],
            tone: ["intense", "feel-good"],
            intensity: 0.7,
            runtime: 30,
            year: 2024,
            platforms: ["hulu"],
            rating: 8.9
        ),
        MediaItem(
            id: "tt2",
            title: "Abbott Elementary",
            overview: "A group of dedicated teachers navigate a Philadelphia public school.",
            genres: ["comedy"],
            tone: ["feel-good", "light"],
            intensity: 0.3,
            runtime: 22,
            year: 2024,
            platforms: ["hulu", "max"],
            rating: 8.2
        ),
        MediaItem(
            id: "tt3",
            title: "Severance",
            overview: "Employees undergo a procedure to separate work and personal memories.",
            genres: ["thriller", "sci-fi", "drama"],
            tone: ["slow", "mysterious"],
            intensity: 0.6,
            runtime: 55,
            year: 2024,
            platforms: ["apple"],
            rating: 8.7
        ),
        MediaItem(
            id: "tt4",
            title: "Ted Lasso",
            overview: "An American football coach leads a British soccer team.",
            genres: ["comedy", "drama"],
            tone: ["feel-good", "heartwarming"],
            intensity: 0.3,
            runtime: 45,
            year: 2023,
            platforms: ["apple"],
            rating: 8.8
        ),
        MediaItem(
            id: "tt5",
            title: "Planet Earth III",
            overview: "Documentary series exploring Earth's natural wonders.",
            genres: ["documentary"],
            tone: ["calm", "slow"],
            intensity: 0.2,
            runtime: 50,
            year: 2023,
            platforms: ["max", "discovery+"],
            rating: 9.2
        ),
        MediaItem(
            id: "tt6",
            title: "John Wick: Chapter 4",
            overview: "Legendary hitman John Wick faces his most dangerous adversaries yet.",
            genres: ["action", "thriller"],
            tone: ["intense", "exciting"],
            intensity: 0.9,
            runtime: 169,
            year: 2023,
            platforms: ["prime"],
            rating: 7.7
        ),
        MediaItem(
            id: "tt7",
            title: "Slow Horses",
            overview: "British intelligence agents who have messed up end up in Slough House.",
            genres: ["thriller", "drama"],
            tone: ["slow", "witty"],
            intensity: 0.5,
            runtime: 45,
            year: 2024,
            platforms: ["apple"],
            rating: 8.1
        ),
        MediaItem(
            id: "tt8",
            title: "Only Murders in the Building",
            overview: "Three strangers investigate a murder in their apartment building.",
            genres: ["comedy", "mystery"],
            tone: ["light", "witty"],
            intensity: 0.4,
            runtime: 35,
            year: 2024,
            platforms: ["hulu"],
            rating: 8.1
        ),
        MediaItem(
            id: "tt9",
            title: "Shogun",
            overview: "An English sailor becomes embroiled in political intrigue in feudal Japan.",
            genres: ["drama", "history"],
            tone: ["epic", "slow"],
            intensity: 0.6,
            runtime: 60,
            year: 2024,
            platforms: ["hulu", "fx"],
            rating: 8.7
        ),
        MediaItem(
            id: "tt10",
            title: "Spirited Away",
            overview: "A young girl enters a world of spirits to save her parents.",
            genres: ["animation", "fantasy"],
            tone: ["magical", "gentle"],
            intensity: 0.4,
            runtime: 125,
            year: 2001,
            platforms: ["max", "netflix"],
            rating: 8.6,
            isRewatch: true
        )
    ]
}
