//
// RuvectorBridgeTests.swift
// VibeCheckTests
//
// Tests for Ruvector WASM integration
//

import XCTest
@testable import VibeCheck

class RuvectorBridgeTests: XCTestCase {
    
    var bridge: RuvectorBridge!
    let wasmPath = "/tmp/ruvector/examples/wasm/ios/target/wasm32-wasi/release/ruvector_ios_wasm.was"
    
    override func setUp() async throws {
        try await super.setUp()
        bridge = RuvectorBridge()
    }
    
    override func tearDown() async throws {
        bridge = nil
        try await super.tearDown()
    }
    
    // MARK: - Lifecycle Tests
    
    func testWASMLoads() async throws {
        // Given: Fresh bridge instance
        XCTAssertFalse(bridge.isReady)
        
        // When: Loading WASM module
        try await bridge.load(wasmPath: wasmPath)
        
        // Then: Bridge should be ready
        XCTAssertTrue(bridge.isReady)
    }
    
    func testLoadingInvalidPathThrows() async throws {
        // When/Then: Loading invalid path should throw
        await assertThrowsError {
            try await bridge.load(wasmPath: "/invalid/path.wasm")
        }
    }
    
    // MARK: - Context Mapping Tests
    
    func testVibeContextMapsToRuvectorState() async throws {
        // Given: VibeCheck context
        let context = VibeContext(
            mood: MoodState(energy: .high, stress: .low),
            biometrics: Biometrics(
                hrv: .init(value: 50, date: Date()),
                sleep: .init(hours: 7.5, quality: 0.8),
                activity: .init(steps: 8000, activeMinutes: 45)
            ),
            keywords: ["action", "intense"]
        )
        
        // When: Mapping to Ruvector VibeState
        let vibeState = bridge.mapToVibeState(context)
        
        // Then: Should map correctly
        XCTAssertEqual(vibeState.energy, 0.8, accuracy: 0.1)  // high energy
        XCTAssertGreaterThan(vibeState.mood, 0.3)  // low stress = positive mood
        XCTAssertEqual(vibeState.focus, 0.7, accuracy: 0.2)  // moderate focus
    }
    
    // MARK: - Learning Tests
    
    func testRecordWatchEvent() async throws {
        // Given: Loaded bridge
        try await bridge.load(wasmPath: wasmPath)
        
        let item = MediaItem(
            id: "test-1",
            title: "Test Action Movie",
            overview: "An intense action thriller",
            genres: [Genre(name: "Action"), Genre(name: "Thriller")],
            tone: ["intense", "fast-paced"],
            intensity: 0.8,
            runtime: 120,
            year: 2024,
            platforms: []
        )
        
        let context = VibeContext(
            mood: MoodState(energy: .high, stress: .low),
            biometrics: mockBiometrics,
            keywords: ["action"]
        )
        
        // When: Recording watch event
        try await bridge.recordWatchEvent(
            item,
            context: context,
            durationSeconds: 3600  // Watched full movie
        )
        
        // Then: Should not throw (successful recording)
        // Internal state should be updated (verified in integration tests)
    }
    
    func testLearnFromInteraction() async throws {
        // Given: Loaded bridge with watch history
        try await bridge.load(wasmPath: wasmPath)
        try await bridge.recordWatchEvent(mockMediaItem, context: mockContext, durationSeconds: 1800)
        
        // When: User enjoyed the content (high satisfaction)
        try await bridge.learn(satisfaction: 0.9)
        
        // Then: Learning should reinforce this pattern
        // (Verified by checking recommendations improve)
    }
    
    // MARK: - Recommendation Tests
    
    func testGetRecommendations() async throws {
        // Given: Loaded bridge
        try await bridge.load(wasmPath: wasmPath)
        
        let context = VibeContext(
            mood: MoodState(energy: .balanced, stress: .low),
            biometrics: mockBiometrics,
            keywords: ["comedy"]
        )
        
        // When: Requesting recommendations
        let recommendations = try await bridge.getRecommendations(
            for: context,
            limit: 5
        )
        
        // Then: Should return items
        XCTAssertGreaterThan(recommendations.count, 0)
        XCTAssertLessThanOrEqual(recommendations.count, 5)
    }
    
    func testRecommendationsImproveWithLearning() async throws {
        // Given: Fresh bridge
        try await bridge.load(wasmPath: wasmPath)
        
        let context = VibeContext(
            mood: MoodState(energy: .high, stress: .low),
            biometrics: mockBiometrics,
            keywords: []
        )
        
        // When: Initial recommendations
        let initialRecs = try await bridge.getRecommendations(for: context, limit: 10)
        
        // User watches several action movies with high satisfaction
        let actionMovies = mockActionMovies()
        for movie in actionMovies {
            try await bridge.recordWatchEvent(movie, context: context, durationSeconds: movie.runtime * 60)
            try await bridge.learn(satisfaction: 0.9)
        }
        
        // New recommendations
        let updatedRecs = try await bridge.getRecommendations(for: context, limit: 10)
        
        // Then: Should have more action movies
        let initialActionCount = initialRecs.filter { $0.hasGenre("Action") }.count
        let updatedActionCount = updatedRecs.filter { $0.hasGenre("Action") }.count
        
        XCTAssertGreaterThan(updatedActionCount, initialActionCount,
                           "Learning should increase action movie recommendations")
    }
    
    // MARK: - Persistence Tests
    
    func testStatePersistence() async throws {
        // Given: Bridge with learned state
        try await bridge.load(wasmPath: wasmPath)
        
        for movie in mockActionMovies() {
            try await bridge.recordWatchEvent(movie, context: mockContext, durationSeconds: 1800)
        }
        
        // When: Saving state
        let stateData = try await bridge.saveState()
        
        // Then: State data should exist
        XCTAssertGreaterThan(stateData.count, 0)
        
        // When: Creating new bridge and loading state
        let newBridge = RuvectorBridge()
        try await newBridge.load(wasmPath: wasmPath)
        try await newBridge.loadState(stateData)
        
        // Then: Recommendations should be similar
        let originalRecs = try await bridge.getRecommendations(for: mockContext, limit: 5)
        let restoredRecs = try await newBridge.getRecommendations(for: mockContext, limit: 5)
        
        XCTAssertEqual(originalRecs.map { $0.id }, restoredRecs.map { $0.id },
                       "Restored bridge should produce same recommendations")
    }
    
    // MARK: - Performance Tests
    
    func testRecommendationPerformance() async throws {
        // Given: Loaded bridge
        try await bridge.load(wasmPath: wasmPath)
        
        // When: Measuring recommendation time
        let start = Date()
        _ = try await bridge.getRecommendations(for: mockContext, limit: 20)
        let duration = Date().timeIntervalSince(start)
        
        // Then: Should be fast (< 50ms as per spec)
        XCTAssertLessThan(duration, 0.050, "Recommendations should return in < 50ms")
    }
    
    func testMemoryUsage() async throws {
        // Given: Loaded bridge
        try await bridge.load(wasmPath: wasmPath)
        
        // When: Recording many events
        for i in 0..<100 {
            let item = mockMediaItem(id: "item-\(i)")
            try await bridge.recordWatchEvent(item, context: mockContext, durationSeconds: 60)
        }
        
        // Then: Memory usage should be reasonable
        // (Verified by Instruments profiling - manual test)
        // Expected: < 15 MB per spec
    }
    
    // MARK: - Helper Methods
    
    private var mockBiometrics: Biometrics {
        Biometrics(
            hrv: .init(value: 45, date: Date()),
            sleep: .init(hours: 7.0, quality: 0.7),
            activity: .init(steps: 5000, activeMinutes: 30)
        )
    }
    
    private var mockContext: VibeContext {
        VibeContext(
            mood: MoodState(energy: .balanced, stress: .low),
            biometrics: mockBiometrics,
            keywords: []
        )
    }
    
    private var mockMediaItem: MediaItem {
        MediaItem(
            id: "mock-1",
            title: "Mock Movie",
            overview: "Test overview",
            genres: [Genre(name: "Drama")],
            tone: ["thoughtful"],
            intensity: 0.5,
            runtime: 120,
            year: 2024,
            platforms: []
        )
    }
    
    private func mockMediaItem(id: String) -> MediaItem {
        MediaItem(
            id: id,
            title: "Movie \(id)",
            overview: "Test",
            genres: [Genre(name: "Drama")],
            tone: [],
            intensity: 0.5,
            runtime: 90,
            year: 2024,
            platforms: []
        )
    }
    
    private func mockActionMovies() -> [MediaItem] {
        return (1...5).map { i in
            MediaItem(
                id: "action-\(i)",
                title: "Action Movie \(i)",
                overview: "Intense action",
                genres: [Genre(name: "Action")],
                tone: ["intense", "fast-paced"],
                intensity: 0.9,
                runtime: 120,
                year: 2024,
                platforms: []
            )
        }
    }
    
    private func assertThrowsError(_ expression: @autoclosure () async throws -> Void) async {
        do {
            _ = try await expression()
            XCTFail("Expected expression to throw an error")
        } catch {
            // Expected
        }
    }
}

// MARK: - Helper Extensions

extension MediaItem {
    func hasGenre(_ genreName: String) -> Bool {
        return genres.contains { $0.name == genreName }
    }
}
