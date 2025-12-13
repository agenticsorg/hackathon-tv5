import Foundation

struct MoodState: Equatable, Codable {
    enum Energy: String, CaseIterable, Codable {
        case exhausted, low, moderate, high, wired

        var displayName: String {
            rawValue.capitalized
        }

        var fillAmount: Double {
            switch self {
            case .exhausted: return 0.1
            case .low: return 0.3
            case .moderate: return 0.5
            case .high: return 0.8
            case .wired: return 1.0
            }
        }
    }

    enum Stress: String, CaseIterable, Codable {
        case relaxed, calm, neutral, tense, stressed

        var displayName: String {
            rawValue.capitalized
        }

        var fillAmount: Double {
            switch self {
            case .relaxed: return 0.2
            case .calm: return 0.4
            case .neutral: return 0.5
            case .tense: return 0.7
            case .stressed: return 0.9
            }
        }
    }

    let energy: Energy
    let stress: Stress
    let confidence: Double
    let timestamp: Date

    init(energy: Energy, stress: Stress, confidence: Double = 0.5, timestamp: Date = Date()) {
        self.energy = energy
        self.stress = stress
        self.confidence = confidence
        self.timestamp = timestamp
    }

    var recommendationHint: String {
        switch (energy, stress) {
        case (.exhausted, _), (.low, .stressed):
            return "comfort"
        case (.low, .relaxed), (.low, .calm):
            return "gentle"
        case (.moderate, .relaxed), (.moderate, .calm):
            return "engaging"
        case (.high, .relaxed), (.high, .calm):
            return "exciting"
        case (_, .stressed), (_, .tense):
            return "light"
        case (.wired, _):
            return "calming"
        default:
            return "balanced"
        }
    }

    var moodDescription: String {
        switch (energy, stress) {
        case (.exhausted, _): return "Wiped Out"
        case (.low, .stressed): return "Running on Empty"
        case (.low, _): return "Mellow"
        case (.moderate, .relaxed): return "Chill"
        case (.moderate, .stressed): return "A Bit Wound Up"
        case (.moderate, _): return "Balanced"
        case (.high, .relaxed): return "Feeling Great"
        case (.high, .stressed): return "Wired"
        case (.high, _): return "Energetic"
        case (.wired, _): return "Buzzing"
        }
    }

    var moodIcon: String {
        switch recommendationHint {
        case "comfort": return "heart.circle.fill"
        case "gentle": return "leaf.circle.fill"
        case "light": return "sun.max.circle.fill"
        case "engaging": return "sparkles"
        case "exciting": return "bolt.circle.fill"
        case "calming": return "moon.circle.fill"
        default: return "circle.grid.cross.fill"
        }
    }

    // Preset moods for quick override
    static let tired = MoodState(energy: .low, stress: .neutral, confidence: 1.0)
    static let stressed = MoodState(energy: .moderate, stress: .stressed, confidence: 1.0)
    static let energetic = MoodState(energy: .high, stress: .relaxed, confidence: 1.0)
    static let chill = MoodState(energy: .moderate, stress: .calm, confidence: 1.0)
    static let `default` = MoodState(energy: .moderate, stress: .neutral, confidence: 0.3)
}
