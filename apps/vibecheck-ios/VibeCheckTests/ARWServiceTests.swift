import Foundation

// MARK: - Mock Helpers

class MockURLProtocol: URLProtocol {
    static var requestHandler: ((URLRequest) throws -> (HTTPURLResponse, Data?))?
    
    override class func canInit(with request: URLRequest) -> Bool {
        return true
    }
    
    override class func canonicalRequest(for request: URLRequest) -> URLRequest {
        return request
    }
    
    override func startLoading() {
        guard let handler = MockURLProtocol.requestHandler else {
            fatalError("Received unexpected request with no handler set")
        }
        
        do {
            let (response, data) = try handler(request)
            client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
            if let data = data {
                client?.urlProtocol(self, didLoad: data)
            }
            client?.urlProtocolDidFinishLoading(self)
        } catch {
            client?.urlProtocol(self, didFailWithError: error)
        }
    }
    
    override func stopLoading() {}
}

// MARK: - Test Suite

func assert(_ condition: Bool, _ message: String, file: String = #file, line: Int = #line) {
    if !condition {
        print("❌ FAILED: \(message) at \(file):\(line)")
        exit(1)
    }
}

func assertEqual<T: Equatable>(_ actual: T, _ expected: T, _ message: String = "", file: String = #file, line: Int = #line) {
    if actual != expected {
        print("❌ FAILED: \(message) - Expected \(expected), got \(actual) at \(file):\(line)")
        exit(1)
    }
}

@main
class ARWTestRunner {
    static func main() async {
        print("🏃 Running ARWService Tests...")
        
        // Setup
        let config = URLSessionConfiguration.ephemeral
        config.protocolClasses = [MockURLProtocol.self]
        let session = URLSession(configuration: config)
        ARWService.shared.configure(session: session)
        
        do {
            try await testFetchManifestSuccess()
            try await testSearchSuccess()
            print("✅ All ARW Tests Passed!")
        } catch {
            print("❌ Unexpected Error: \(error)")
            exit(1)
        }
    }
    
    static func testSearch_MapsArtworkCorrectly() async throws {
        // 1. Prepare Mock Data with Artwork
        let mockJSON = """
        {
          "success": true,
          "results": [
            {
              "content": {
                "id": 550,
                "title": "Fight Club",
                "overview": "An insomniac office worker...",
                "media_type": "movie",
                "genre_ids": [18],
                "vote_average": 8.4,
                "poster_path": "/pB8BM7pdSp6B6Ih7Qf4n6a8MIxdf.jpg",
                "backdrop_path": "/hZkgoQYus5vegHoetLkCJzb17zJ.jpg"
              },
              "relevance_score": 0.9,
              "match_reasons": ["intense"],
              "explanation": "Gripping psychological drama."
            }
          ]
        }
        """.data(using: .utf8)!
        
        MockURLProtocol.requestHandler = { request in
             // Return manifest for the first call if needed, or just specific responses based on URL
             if request.url?.absoluteString.contains("search") == true {
                 return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, mockJSON)
             } else {
                 // Manifest fallback
                 let manifestJSON = """
                 {
                     "version": "1.0",
                     "profile": "media-discovery",
                     "site": { "name": "Media", "description": "Search" },
                     "actions": [
                         { "id": "semantic_search", "endpoint": "/api/search", "method": "POST" }
                     ]
                 }
                 """.data(using: .utf8)!
                 return (HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!, manifestJSON)
             }
        }
        
        // 2. Execute Search
        let results = try await ARWService.shared.search(query: "test")
        
        // 3. Assertions
        guard let item = results.first else {
            assert(false, "Should return 1 item")
            return
        }
        
        assert(item.posterPath == "/pB8BM7pdSp6B6Ih7Qf4n6a8MIxdf.jpg", "Poster path mismatch")
        
        // Check computed URLs
        let expectedPosterURL = URL(string: "https://image.tmdb.org/t/p/w500/pB8BM7pdSp6B6Ih7Qf4n6a8MIxdf.jpg")
        let expectedBackdropURL = URL(string: "https://image.tmdb.org/t/p/w780/hZkgoQYus5vegHoetLkCJzb17zJ.jpg")
        
        assert(item.posterURL == expectedPosterURL, "Computed Poster URL mismatch")
        assert(item.backdropURL == expectedBackdropURL, "Computed Backdrop URL mismatch")
        
        print("✅ testSearch_MapsArtworkCorrectly Passed")
    }
    
    static func testFetchManifestSuccess() async throws {
        print("   testFetchManifestSuccess...")
        let json = """
        {
            "version": "0.1",
            "profile": "ARW-1",
            "site": {
                "name": "Test Site",
                "description": "Test Description"
            },
            "actions": [
                {
                    "id": "semantic_search",
                    "endpoint": "/api/search",
                    "method": "POST"
                }
            ]
        }
        """
        let data = json.data(using: .utf8)!
        
        MockURLProtocol.requestHandler = { request in
            guard let url = request.url, url.absoluteString.hasSuffix("/.well-known/arw-manifest.json") else {
                throw URLError(.badURL)
            }
            let response = HTTPURLResponse(url: url, statusCode: 200, httpVersion: nil, headerFields: nil)!
            return (response, data)
        }
        
        let manifest = try await ARWService.shared.fetchManifest()
        assertEqual(manifest.site.name, "Test Site")
        assertEqual(manifest.actions.first?.id, "semantic_search")
    }
    
    static func testSearchSuccess() async throws {
        print("   testSearchSuccess...")
        
        // 1. Mock Manifest
        let manifestJson = """
        {
            "version": "0.1",
            "profile": "ARW-1",
            "site": { "name": "Test", "description": "Test" },
            "actions": [ { "id": "semantic_search", "endpoint": "/api/search", "method": "POST" } ]
        }
        """
        
        // 2. Mock Search Response
        let searchJson = """
        {
            "success": true,
            "results": [
                {
                    "content": {
                        "id": 123,
                        "title": "Test Movie",
                        "overview": "A test movie overview",
                        "voteAverage": 8.5,
                        "mediaType": "movie",
                        "genreIds": [28]
                    },
                    "relevanceScore": 0.9,
                    "matchReasons": ["intense"],
                    "explanation": "Because you like action."
                }
            ]
        }
        """
        
        MockURLProtocol.requestHandler = { request in
            if request.url?.absoluteString.hasSuffix("arw-manifest.json") == true {
                let response = HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!
                return (response, manifestJson.data(using: .utf8)!)
            } else if request.url?.absoluteString.hasSuffix("/api/search") == true {
                let response = HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!
                return (response, searchJson.data(using: .utf8)!)
            }
            throw URLError(.badURL)
        }
        
        let results = try await ARWService.shared.search(query: "Test Query")
        
        assertEqual(results.count, 1)
        assertEqual(results.first?.title, "Test Movie")
        assertEqual(results.first?.sommelierRationale, "Because you like action.")
    }
}

