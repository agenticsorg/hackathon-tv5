import Foundation

struct VibeContext {
    let keywords: [String]
    let explanation: String
    let mood: MoodState
}

class VibePredictor {
    
    // Thresholds
    private let lowSleepThreshold: Double = 6.0
    private let highActivityThreshold: Double = 8000.0
    private let stressHRVThreshold: Double = 40.0 // Simplified ms
    
    func predictVibe(
        hrv: Double?,
        sleepHours: Double?,
        steps: Double?,
        timeOfDay: Date = Date()
    ) -> VibeContext {
        
        var keywords: [String] = []
        var reasons: [String] = []
        
        // 1. Analyze Energy Level (Sleep & Steps)
        let sleep = sleepHours ?? 7.5 // Default to average if missing
        let stepCount = steps ?? 0
        
        var energy: MoodState.Energy = .moderate
        
        if sleep < lowSleepThreshold {
            energy = .low
            keywords.append(contentsOf: ["comfort", "gentle", "familiar", "slow-paced"])
            reasons.append("you didn't get much sleep")
        } else if stepCount > highActivityThreshold {
            energy = .high
            keywords.append(contentsOf: ["action", "adventure", "exciting", "fast-paced"])
            reasons.append("you've been very active today")
        } else {
            keywords.append(contentsOf: ["balanced", "popular", "engaging"])
        }
        
        // 2. Analyze Stress Level (HRV)
        // Note: Lower HRV generally correlates with higher stress
        var stress: MoodState.Stress = .neutral
        
        if let currentHRV = hrv {
            if currentHRV < stressHRVThreshold {
                stress = .stressed
                keywords.append(contentsOf: ["calming", "meditative", "nature", "hopeful"])
                reasons.append("your stress levels seem elevated")
            } else if currentHRV > 60 {
                stress = .relaxed
                keywords.append(contentsOf: ["creative", "complex", "thought-provoking"])
            }
        }
        
        // 3. Time of Day adjustments
        let hour = Calendar.current.component(.hour, from: timeOfDay)
        if hour < 6 || hour > 22 {
            keywords.append(contentsOf: ["dreamy", "surreal", "dark"])
        } else if hour > 6 && hour < 11 {
            keywords.append(contentsOf: ["inspiring", "motivational"])
        }
        
        // 4. Construct Explanation
        let explanation: String
        if reasons.isEmpty {
            explanation = "you're in a solid, balanced flow"
        } else {
            explanation = reasons.joined(separator: " and ")
        }
        
        // 5. Create MoodState proxy for UI compatibility
        let mood = MoodState(energy: energy, stress: stress)
        
        return VibeContext(
            keywords: Array(Set(keywords)), // De-duplicate
            explanation: explanation,
            mood: mood
        )
    }
}
