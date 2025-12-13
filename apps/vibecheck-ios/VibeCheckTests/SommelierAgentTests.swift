import XCTest
@testable import VibeCheck

final class SommelierAgentTests: XCTestCase {
    
    var agent: SommelierAgent!
    
    override func setUp() {
        super.setUp()
        agent = SommelierAgent.shared
    }
    
    func testComfortRationale() {
        // Given: User is tired/stressed (Comfort vibe)
        let mood = MoodState(energy: .low, stress: .stressed) // "Comfort"
        let context = VibeContext(
            keywords: ["gentle", "familiar"],
            explanation: "you are tired",
            mood: mood
        )
        
        let item = MediaItem(
            id: "1", title: "The Office", genres: ["Comedy"], 
            tone: ["feel-good"], intensity: 0.2, isRewatch: true
        )
        
        // When
        let rationale = agent.rationale(for: item, context: context)
        
        // Then
        XCTAssertTrue(rationale.contains("The Office"))
        XCTAssertTrue(rationale.lowercased().contains("comfort") || rationale.lowercased().contains("favorite"))
    }
    
    func testExcitingRationale() {
        // Given: User is high energy/relaxed (Exciting vibe)
        let mood = MoodState(energy: .high, stress: .relaxed) // "Exciting"
        let context = VibeContext(
            keywords: ["action", "fast"],
            explanation: "you have energy",
            mood: mood
        )
        
        let item = MediaItem(
            id: "2", title: "Mad Max", genres: ["Action"], 
            tone: ["intense"], intensity: 0.9
        )
        
        // When
        let rationale = agent.rationale(for: item, context: context)
        
        // Then
        XCTAssertTrue(rationale.contains("Mad Max"))
        XCTAssertTrue(rationale.lowercased().contains("energy") || rationale.lowercased().contains("intensity"))
    }
}
