import Foundation

// Rudimentary Test Runner
@main
class SommelierRunner {
    static func main() {
        print("Running SommelierAgent Tests...")
        
        let agent = SommelierAgent.shared
        var failed = false
        
        // Test 1: Comfort Rationale
        let comfortMood = MoodState(energy: .low, stress: .stressed)
        let comfortContext = VibeContext(
            keywords: ["gentle"],
            explanation: "you are tired",
            mood: comfortMood
        )
        let comfortItem = MediaItem(id: "1", title: "The Office", tone: ["feel-good"], intensity: 0.2, isRewatch: true)
        
        let comfortRat = agent.rationale(for: comfortItem, context: comfortContext)
        if comfortRat.contains("The Office") && (comfortRat.lowercased().contains("comfort") || comfortRat.lowercased().contains("favorite")) {
            print("✅ testComfortRationale: PASSED")
        } else {
            print("❌ testComfortRationale: FAILED - Got: \(comfortRat)")
            failed = true
        }

        // Test 2: Exciting Rationale
        let excitingMood = MoodState(energy: .high, stress: .relaxed)
        let existingContext = VibeContext(
            keywords: ["action"],
            explanation: "you have energy",
            mood: excitingMood
        )
        let actionItem = MediaItem(id: "2", title: "Mad Max", tone: ["intense"], intensity: 0.9)
        
        let actionRat = agent.rationale(for: actionItem, context: existingContext)
        if actionRat.contains("Mad Max") && (actionRat.lowercased().contains("energy") || actionRat.lowercased().contains("intensity") || actionRat.lowercased().contains("revs")) {
             print("✅ testExcitingRationale: PASSED")
        } else {
             print("❌ testExcitingRationale: FAILED - Got: \(actionRat)")
             failed = true
        }
        
        if failed {
            exit(1)
        } else {
            print("All Tests Passed.")
            exit(0)
        }
    }
}

// Entry point

