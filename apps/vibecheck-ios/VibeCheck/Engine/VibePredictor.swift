import Foundation

struct VibeContext {
    let keywords: [String]
    let explanation: String
    let mood: MoodState
    let mlConfidence: Double // How confident the ML model is (0-1)
    let trainingIterations: Int // How many times the model has learned

    // Default initializer for backwards compatibility
    init(keywords: [String], explanation: String, mood: MoodState, mlConfidence: Double = 0.0, trainingIterations: Int = 0) {
        self.keywords = keywords
        self.explanation = explanation
        self.mood = mood
        self.mlConfidence = mlConfidence
        self.trainingIterations = trainingIterations
    }
}

@available(iOS 15.0, *)
class VibePredictor {

    // Singleton for shared WASM bridge
    static let shared = VibePredictor()

    // WASM bridge for ML inference
    private var bridge: RuvectorBridge?
    private var isMLReady = false

    // Fallback thresholds (used when ML not available or untrained)
    private let lowSleepThreshold: Double = 6.0
    private let highActivityThreshold: Double = 8000.0
    private let stressHRVThreshold: Double = 40.0

    // Track training progress
    private(set) var trainingIterations: Int = 0

    init() {}

    // MARK: - ML Initialization

    /// Initialize the WASM ML learner
    /// Call this once at app startup
    func initializeML() async {
        do {
            bridge = RuvectorBridge()
            try await bridge?.loadFromBundle()
            try bridge?.initLearner()
            trainingIterations = (try? bridge?.getLearnerIterations()) ?? 0
            isMLReady = true
            print("✅ VibePredictor: ML initialized with \(trainingIterations) prior training iterations")
        } catch {
            print("⚠️ VibePredictor: ML init failed, using rule-based fallback: \(error)")
            isMLReady = false
        }
    }

    // MARK: - Learning

    /// Learn from user feedback about their actual energy level
    /// Call this when user corrects or confirms their mood
    func learn(hrv: Double?, sleepHours: Double?, steps: Double?, actualEnergy: MoodState.Energy) {
        guard isMLReady, let bridge = bridge else { return }

        let hrvFloat = Float(hrv ?? 50.0)
        let sleepFloat = Float(sleepHours ?? 7.0)
        let stepsFloat = Float(steps ?? 5000.0)
        let energyLabel = Float(actualEnergy.fillAmount) // 0.1 to 1.0

        do {
            try bridge.learnHealth(hrv: hrvFloat, sleepHours: sleepFloat, steps: stepsFloat, energyLabel: energyLabel)
            trainingIterations = (try? bridge.getLearnerIterations()) ?? trainingIterations + 1
            print("🧠 VibePredictor: Learned from feedback (iteration \(trainingIterations))")
        } catch {
            print("⚠️ VibePredictor: Learning failed: \(error)")
        }
    }

    // MARK: - Prediction

    func predictVibe(
        hrv: Double?,
        sleepHours: Double?,
        steps: Double?,
        timeOfDay: Date = Date()
    ) -> VibeContext {

        // Try ML prediction first
        if isMLReady, let bridge = bridge, trainingIterations >= 5 {
            return predictVibeML(
                bridge: bridge,
                hrv: hrv,
                sleepHours: sleepHours,
                steps: steps,
                timeOfDay: timeOfDay
            )
        }

        // Fallback to rule-based
        return predictVibeRuleBased(
            hrv: hrv,
            sleepHours: sleepHours,
            steps: steps,
            timeOfDay: timeOfDay
        )
    }

    // MARK: - ML Prediction

    private func predictVibeML(
        bridge: RuvectorBridge,
        hrv: Double?,
        sleepHours: Double?,
        steps: Double?,
        timeOfDay: Date
    ) -> VibeContext {

        let hrvFloat = Float(hrv ?? 50.0)
        let sleepFloat = Float(sleepHours ?? 7.0)
        let stepsFloat = Float(steps ?? 5000.0)

        var keywords: [String] = []
        var reasons: [String] = []

        // 1. ML-based energy prediction
        var energy: MoodState.Energy = .moderate
        var mlConfidence: Double = 0.5

        do {
            let predictedEnergy = try bridge.predictEnergy(hrv: hrvFloat, sleepHours: sleepFloat, steps: stepsFloat)
            mlConfidence = min(1.0, Double(trainingIterations) / 20.0) // Confidence grows with training

            // Map 0-1 float to Energy enum
            energy = energyFromFloat(predictedEnergy)

            // Generate keywords based on ML prediction
            switch energy {
            case .exhausted:
                keywords.append(contentsOf: ["comfort", "gentle", "familiar", "cozy"])
                reasons.append("your body signals suggest you need rest")
            case .low:
                keywords.append(contentsOf: ["comfort", "gentle", "slow-paced"])
                reasons.append("you seem a bit tired")
            case .moderate:
                keywords.append(contentsOf: ["balanced", "popular", "engaging"])
            case .high:
                keywords.append(contentsOf: ["action", "adventure", "exciting"])
                reasons.append("you're feeling energized")
            case .wired:
                keywords.append(contentsOf: ["action", "fast-paced", "intense", "thriller"])
                reasons.append("you've got lots of energy to burn")
            }
        } catch {
            // Fall back to rule-based for energy
            energy = energyFromRules(sleepHours: sleepHours, steps: steps)
            keywords.append(contentsOf: ["balanced", "popular"])
        }

        // 2. Stress from HRV (still rule-based, could add ios_learn_stress in future)
        var stress: MoodState.Stress = .neutral
        if let currentHRV = hrv {
            if currentHRV < stressHRVThreshold {
                stress = .stressed
                keywords.append(contentsOf: ["calming", "meditative", "nature"])
                reasons.append("your HRV suggests some stress")
            } else if currentHRV > 60 {
                stress = .relaxed
                keywords.append(contentsOf: ["creative", "complex", "thought-provoking"])
            }
        }

        // 3. Time of day adjustments
        let hour = Calendar.current.component(.hour, from: timeOfDay)
        if hour < 6 || hour > 22 {
            keywords.append(contentsOf: ["dreamy", "surreal"])
        } else if hour > 6 && hour < 11 {
            keywords.append(contentsOf: ["inspiring", "motivational"])
        }

        // 4. Build explanation
        let explanation: String
        if reasons.isEmpty {
            explanation = "you're in a solid, balanced flow"
        } else {
            explanation = reasons.joined(separator: " and ")
        }

        let mood = MoodState(energy: energy, stress: stress, confidence: mlConfidence)

        return VibeContext(
            keywords: Array(Set(keywords)),
            explanation: explanation,
            mood: mood,
            mlConfidence: mlConfidence,
            trainingIterations: trainingIterations
        )
    }

    // MARK: - Rule-Based Fallback

    private func predictVibeRuleBased(
        hrv: Double?,
        sleepHours: Double?,
        steps: Double?,
        timeOfDay: Date
    ) -> VibeContext {

        var keywords: [String] = []
        var reasons: [String] = []

        let sleep = sleepHours ?? 7.5
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

        let hour = Calendar.current.component(.hour, from: timeOfDay)
        if hour < 6 || hour > 22 {
            keywords.append(contentsOf: ["dreamy", "surreal", "dark"])
        } else if hour > 6 && hour < 11 {
            keywords.append(contentsOf: ["inspiring", "motivational"])
        }

        let explanation: String
        if reasons.isEmpty {
            explanation = "you're in a solid, balanced flow"
        } else {
            explanation = reasons.joined(separator: " and ")
        }

        let mood = MoodState(energy: energy, stress: stress, confidence: 0.3) // Low confidence for rules

        return VibeContext(
            keywords: Array(Set(keywords)),
            explanation: explanation,
            mood: mood,
            mlConfidence: 0.0, // No ML used
            trainingIterations: 0
        )
    }

    // MARK: - Helpers

    private func energyFromFloat(_ value: Float) -> MoodState.Energy {
        switch value {
        case ..<0.15: return .exhausted
        case 0.15..<0.35: return .low
        case 0.35..<0.65: return .moderate
        case 0.65..<0.85: return .high
        default: return .wired
        }
    }

    private func energyFromRules(sleepHours: Double?, steps: Double?) -> MoodState.Energy {
        let sleep = sleepHours ?? 7.5
        let stepCount = steps ?? 0

        if sleep < lowSleepThreshold {
            return .low
        } else if stepCount > highActivityThreshold {
            return .high
        }
        return .moderate
    }

    // MARK: - Benchmarking

    /// Benchmark ML inference time
    func benchmarkMLInference(iterations: Int = 100) async -> Double {
        guard isMLReady, let bridge = bridge else {
            return -1
        }

        do {
            return try bridge.benchmarkMLInference(iterations: iterations)
        } catch {
            return -1
        }
    }
}
