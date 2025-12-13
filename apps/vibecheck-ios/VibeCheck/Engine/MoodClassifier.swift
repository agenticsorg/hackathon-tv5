import Foundation

class MoodClassifier {

    /// Classify the user's current mood based on biometric data
    func classify(
        hrv: Double?,
        sleepHours: Double?,
        restingHR: Double?,
        activity: HealthKitManager.ActivityLevel,
        timeOfDay: Date = Date()
    ) -> MoodState {

        let energy = classifyEnergy(
            sleepHours: sleepHours,
            activity: activity,
            timeOfDay: timeOfDay
        )

        let stress = classifyStress(
            hrv: hrv,
            restingHR: restingHR
        )

        let confidence = calculateConfidence(
            hasHRV: hrv != nil,
            hasSleep: sleepHours != nil,
            hasActivity: activity != .unknown
        )

        return MoodState(energy: energy, stress: stress, confidence: confidence)
    }

    // MARK: - Energy Classification

    private func classifyEnergy(
        sleepHours: Double?,
        activity: HealthKitManager.ActivityLevel,
        timeOfDay: Date
    ) -> MoodState.Energy {

        let hour = Calendar.current.component(.hour, from: timeOfDay)
        let isLateNight = hour >= 23 || hour < 5
        let isEarlyMorning = hour >= 5 && hour < 8

        var score = 0.0

        // Sleep contribution (-2 to +1)
        if let sleep = sleepHours {
            switch sleep {
            case 0..<4:
                score -= 2.0
            case 4..<5:
                score -= 1.5
            case 5..<6:
                score -= 0.5
            case 6..<7:
                score += 0.0
            case 7..<8:
                score += 0.5
            case 8..<9:
                score += 1.0
            default:
                // Oversleep can mean fatigue
                score += 0.5
            }
        }

        // Activity contribution (-0.5 to +1)
        switch activity {
        case .sedentary:
            score -= 0.5
        case .light:
            score += 0.0
        case .moderate:
            score += 0.5
        case .active:
            score += 1.0
        case .unknown:
            break
        }

        // Time of day modifiers
        if isLateNight {
            score -= 1.0
        } else if isEarlyMorning {
            score -= 0.3
        }

        // Map score to energy level
        switch score {
        case ..<(-1.5):
            return .exhausted
        case -1.5..<(-0.5):
            return .low
        case -0.5..<0.5:
            return .moderate
        case 0.5..<1.5:
            return .high
        default:
            return .wired
        }
    }

    // MARK: - Stress Classification

    private func classifyStress(hrv: Double?, restingHR: Double?) -> MoodState.Stress {
        // HRV: Higher values generally indicate better recovery and lower stress
        // Typical ranges vary widely by individual (20-80ms common)
        // This is a simplified model - ideally would be personalized to user's baseline

        guard let hrv = hrv else {
            // If no HRV, try to use resting HR as a secondary indicator
            if let rhr = restingHR {
                // Lower resting HR generally = better cardiovascular health / less acute stress
                switch rhr {
                case 0..<55:
                    return .relaxed
                case 55..<65:
                    return .calm
                case 65..<75:
                    return .neutral
                case 75..<85:
                    return .tense
                default:
                    return .stressed
                }
            }
            return .neutral
        }

        // HRV-based classification
        switch hrv {
        case 70...:
            return .relaxed
        case 50..<70:
            return .calm
        case 35..<50:
            return .neutral
        case 20..<35:
            return .tense
        default:
            return .stressed
        }
    }

    // MARK: - Confidence

    private func calculateConfidence(
        hasHRV: Bool,
        hasSleep: Bool,
        hasActivity: Bool
    ) -> Double {
        var score = 0.3 // Base confidence

        if hasHRV {
            score += 0.3
        }
        if hasSleep {
            score += 0.2
        }
        if hasActivity {
            score += 0.2
        }

        return min(score, 1.0)
    }
}
