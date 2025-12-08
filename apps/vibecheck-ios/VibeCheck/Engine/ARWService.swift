import Foundation

// MARK: - ARW Models
struct ARWManifest: Codable {
    let version: String
    let profile: String
    let site: ARWSite
    let actions: [ARWAction]
}

struct ARWSite: Codable {
    let name: String
    let description: String
}

struct ARWAction: Codable {
    let id: String
    let endpoint: String
    let method: String
}

struct ARWSearchResponse: Codable {
    let success: Bool
    let results: [ARWSearchResult]
}

struct ARWSearchResult: Codable {
    let content: ARWMediaContent
    let relevanceScore: Double
    let matchReasons: [String]
    let explanation: String?
}

struct ARWMediaContent: Codable {
    let id: Int
    let title: String?
    let name: String?
    let overview: String
    let posterPath: String?
    let backdropPath: String?
    let voteAverage: Double
    let mediaType: String
    let genreIds: [Int]
    let releaseDate: String?
    let firstAirDate: String?
    
    var displayTitle: String {
        return title ?? name ?? "Unknown Title"
    }
    
    var displayYear: Int {
        let dateString = releaseDate ?? firstAirDate
        if let yearStr = dateString?.prefix(4), let year = Int(yearStr) {
            return year
        }
        return Calendar.current.component(.year, from: Date())
    }
}

// MARK: - ARW Service
class ARWService {
    static let shared = ARWService()
    
    private let decoder: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return decoder
    }()
    
    // Allow injection for testing
    private var session: URLSession = {
        let config = URLSessionConfiguration.default
        config.timeoutIntervalForRequest = 20 // Increased for verification
        return URLSession(configuration: config)
    }()
    
    private var manifest: ARWManifest?
    // Use LAN IP for physical device testing (replace with your machine's IP)
    private let baseURLString = "http://localhost:3000"
    
    private init() {}
    
    // For testing
    func configure(session: URLSession) {
        self.session = session
    }
    
    func fetchManifest() async throws -> ARWManifest {
        if let cached = manifest { return cached }
        
        guard let url = URL(string: "\(baseURLString)/.well-known/arw-manifest.json") else {
            throw URLError(.badURL)
        }
        
        let (data, response) = try await session.data(from: url)
        
        guard let httpResponse = response as? HTTPURLResponse, 
              (200...299).contains(httpResponse.statusCode) else {
            throw URLError(.badServerResponse)
        }
        
        let manifest = try decoder.decode(ARWManifest.self, from: data)
        self.manifest = manifest
        return manifest
    }
    
    func search(query: String) async throws -> [MediaItem] {
        // Ensure manifest is loaded (or fetch it)
        let manifest = try await fetchManifest()
        
        guard let action = manifest.actions.first(where: { $0.id == "semantic_search" }) else {
            print("ARW: content action not found")
            return []
        }
        
        let endpoint = action.endpoint.hasPrefix("http") ? action.endpoint : "\(baseURLString)\(action.endpoint)"
        
        guard let url = URL(string: endpoint) else {
            throw URLError(.badURL)
        }
        
        var request = URLRequest(url: url)
        request.httpMethod = action.method
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        
        let body: [String: Any] = [
            "query": query,
            "explain": true,
            "limit": 10
        ]
        
        request.httpBody = try JSONSerialization.data(withJSONObject: body)
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse, 
              (200...299).contains(httpResponse.statusCode) else {
            if let errorStr = String(data: data, encoding: .utf8) {
                print("ARW Error: \(errorStr)")
            }
            throw URLError(.badServerResponse)
        }
        
        let searchResponse = try decoder.decode(ARWSearchResponse.self, from: data)
        
        return searchResponse.results.compactMap { mapToMediaItem($0) }
    }
    
    private func mapToMediaItem(_ result: ARWSearchResult) -> MediaItem {
        let content = result.content
        let genres = mapGenres(ids: content.genreIds, type: content.mediaType)
        let tone = inferTone(from: result.matchReasons, genreIds: content.genreIds)
        
        var item = MediaItem(
            id: String(content.id),
            title: content.displayTitle,
            overview: content.overview,
            genres: genres,
            tone: tone,
            intensity: calculateIntensity(voteAverage: content.voteAverage, genreIds: content.genreIds),
            runtime: 90,
            year: content.displayYear,
            platforms: ["arw"],
            posterPath: content.posterPath,
            backdropPath: content.backdropPath,
            rating: content.voteAverage
        )
        // Map ARW explanation to Sommelier rationale
        item.sommelierRationale = result.explanation
        return item
    }
    
    private func mapGenres(ids: [Int], type: String) -> [String] {
        var names: [String] = []
        let map: [Int: String] = [
            28: "Action", 12: "Adventure", 16: "Animation", 35: "Comedy",
            80: "Crime", 99: "Documentary", 18: "Drama", 10751: "Family",
            14: "Fantasy", 36: "History", 27: "Horror", 10402: "Music",
            9648: "Mystery", 10749: "Romance", 878: "Sci-Fi", 10770: "TV Movie",
            53: "Thriller", 10752: "War", 37: "Western",
            10759: "Action & Adventure", 10765: "Sci-Fi & Fantasy", 10768: "War & Politics"
        ]
        
        for id in ids {
            if let name = map[id] {
                names.append(name.lowercased())
            }
        }
        return names.isEmpty ? ["unknown"] : names
    }
    
    private func inferTone(from reasons: [String], genreIds: [Int]) -> [String] {
        var tones: [String] = []
        if genreIds.contains(35) { tones.append("light") }
        if genreIds.contains(27) || genreIds.contains(53) { tones.append("intense") }
        if genreIds.contains(18) { tones.append("emotional") }
        if genreIds.contains(99) { tones.append("calm") }
        if tones.isEmpty { tones.append("engaging") }
        return tones
    }
    
    private func calculateIntensity(voteAverage: Double, genreIds: [Int]) -> Double {
        if genreIds.contains(28) || genreIds.contains(27) { return 0.8 }
        if genreIds.contains(35) || genreIds.contains(10751) { return 0.3 }
        return 0.5
    }
}
