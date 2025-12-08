import Foundation
import Observation
import NaturalLanguage

// Mock Types to avoid dragging in everything if possible
// But we need the real Engine logic.

// We will rely on the real files being passed to swiftc.

@main
class IntegrationVerifier {
    static func main() async {
        print("--- Verifying Sommelier Engine Integration ---")
        
        let engine = RecommendationEngine()
        
        // Mock Context: "Stressed and Low Energy" -> Comfort
        let mood = MoodState(energy: .low, stress: .stressed)
        let context = VibeContext(
            keywords: ["gentle", "familiar", "warm"],
            explanation: "you had a rough day",
            mood: mood
        )
        
        // Preferences
        let prefs = UserPreferences() 
        // We assume UserPreferences defaults are empty/permissive. 
        // We need to make sure UserPreferences is available or mocked.
        // It's in `Models/UserPreferences.swift` (I need to check if I need to include it)
        
        print("Refreshing recommendations for mood: \(mood.recommendationHint)...")
        
        // We need to call refresh. It uses Task/MainActor.
        // We are in async static main.
        
        // Note: Engine.refresh is async internally (Task { ... }) but changes state on MainActor.
        // We can't await it easily unless we modify Engine to return values or wait.
        // For this test, we might sleep.
        
        engine.refresh(context: context, preferences: prefs)
        
        // Wait for async task (ARW timeout is 5s, so we wait 6s to be safe)
        try? await Task.sleep(nanoseconds: 6 * 1_000_000_000)
        
        let recs = engine.recommendations
        print("Got \(recs.count) recommendations.")
        
        for item in recs {
            print("\n🎬 \(item.title)")
            if let rationale = item.sommelierRationale {
                print("🍷 Sommelier: \"\(rationale)\"")
            } else {
                print("❌ NO RATIONAL GENERATED")
                exit(1)
            }
        }
        
        if recs.isEmpty {
           print("❌ No recs found.")
           exit(1)
        }
        
        print("\n✅ Integration Verification Passed.")
    }
}
