import SwiftUI
import SwiftData
import NaturalLanguage
import WasmKit

struct BenchmarkResult: Identifiable {
    let id = UUID()
    let name: String
    let target: String
    let actual: String
    let status: Status
    let isReal: Bool  // Indicates if this is a real measurement vs simulated

    enum Status {
        case pass, slow, fail

        var icon: String {
            switch self {
            case .pass: return "checkmark.circle.fill"
            case .slow: return "exclamationmark.triangle.fill"
            case .fail: return "xmark.circle.fill"
            }
        }

        var color: Color {
            switch self {
            case .pass: return .green
            case .slow: return .yellow
            case .fail: return .red
            }
        }
    }

    init(name: String, target: String, actual: String, status: Status, isReal: Bool = true) {
        self.name = name
        self.target = target
        self.actual = actual
        self.status = status
        self.isReal = isReal
    }
}

struct BenchmarkView: View {
    @Environment(\.modelContext) private var modelContext
    @Query private var moodLogs: [MoodLog]
    @Query private var watchHistory: [WatchHistory]
    @Query private var watchlistItems: [WatchlistItem]

    @State private var results: [BenchmarkResult] = []
    @State private var isRunning = false

    /// Build identifier - increment when making changes to verify deployment
    /// Format: v{version}.{build}-{revision}
    private let buildIdentifier = "v1.0.1-r16"  // LearningMemory + HNSW integration
    @State private var memoryUsage: String = "—"
    @State private var totalTime: String = "—"
    @State private var vectorCount: Int = 0

    var body: some View {
        List {
            // Data Context Section
            Section {
                StatRow(label: "Sample Media Items", value: "\(MediaItem.samples.count)", icon: "film")
                StatRow(label: "Mood Logs", value: "\(moodLogs.count)", icon: "heart.text.square")
                StatRow(label: "Watch History", value: "\(watchHistory.count)", icon: "clock.arrow.circlepath")
                StatRow(label: "Watchlist Items", value: "\(watchlistItems.count)", icon: "bookmark")
                StatRow(label: "Mood States", value: "\(MoodState.Energy.allCases.count * MoodState.Stress.allCases.count)", icon: "brain.head.profile")
                StatRow(label: "Recommendation Hints", value: "7", icon: "sparkles")
                StatRow(label: "Vector Embeddings (HNSW)", value: vectorCount > 0 ? "\(vectorCount)" : "—", icon: "arrow.triangle.branch")
            } header: {
                Text("Data Context")
            } footer: {
                Text("Records in the system that benchmarks operate on")
            }

            Section {
                HStack {
                    Text("Total Time")
                    Spacer()
                    Text(totalTime)
                        .monospacedDigit()
                        .foregroundStyle(.secondary)
                }

                HStack {
                    Text("Memory Usage")
                    Spacer()
                    Text(memoryUsage)
                        .monospacedDigit()
                        .foregroundStyle(.secondary)
                }
            } header: {
                Text("Summary")
            }

            Section {
                if results.isEmpty && !isRunning {
                    Text("Tap 'Run Benchmarks' to start")
                        .foregroundStyle(.secondary)
                } else {
                    ForEach(results) { result in
                        HStack {
                            Image(systemName: result.status.icon)
                                .foregroundStyle(result.status.color)

                            VStack(alignment: .leading, spacing: 2) {
                                HStack(spacing: 4) {
                                    Text(result.name)
                                        .font(.subheadline)
                                    if !result.isReal {
                                        Text("(sim)")
                                            .font(.caption2)
                                            .foregroundStyle(.orange)
                                    }
                                }
                                Text("Target: \(result.target)")
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }

                            Spacer()

                            Text(result.actual)
                                .font(.system(.caption, design: .monospaced))
                                .foregroundStyle(result.status.color)
                        }
                    }
                }
            } header: {
                Text("Results")
            } footer: {
                if !results.isEmpty {
                    Text("All benchmarks are real measurements except those marked (sim)")
                        .font(.caption2)
                }
            }

            Section {
                Button {
                    runBenchmarks()
                } label: {
                    HStack {
                        Spacer()
                        if isRunning {
                            ProgressView()
                                .padding(.trailing, 8)
                            Text("Running...")
                        } else {
                            Image(systemName: "play.fill")
                                .padding(.trailing, 4)
                            Text("Run Benchmarks")
                        }
                        Spacer()
                    }
                }
                .disabled(isRunning)
            } footer: {
                Text("Build: \(buildIdentifier)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(.top, 8)
            }
        }
        .navigationTitle("Performance")
        .navigationBarTitleDisplayMode(.inline)
    }

    private func runBenchmarks() {
        isRunning = true
        results = []

        Task {
            let startTime = CFAbsoluteTimeGetCurrent()
            var benchmarkResults: [BenchmarkResult] = []

            // 1. NLEmbedding Load Time (REAL - Apple's on-device NLP)
            let nlStart = CFAbsoluteTimeGetCurrent()
            let embedding = NLEmbedding.sentenceEmbedding(for: .english)
            let nlLoaded = embedding != nil
            let nlTime = (CFAbsoluteTimeGetCurrent() - nlStart) * 1000
            benchmarkResults.append(BenchmarkResult(
                name: "NLEmbedding Load",
                target: "<100ms",
                actual: nlLoaded ? String(format: "%.1fms", nlTime) : "FAILED",
                status: nlLoaded ? (nlTime < 100 ? .pass : (nlTime < 200 ? .slow : .fail)) : .fail,
                isReal: true
            ))

            // 2. Semantic Vector Generation (REAL - generates actual embedding)
            let vectorService = VectorEmbeddingService.shared
            let vectorStart = CFAbsoluteTimeGetCurrent()
            var vectorSuccess = false
            for _ in 0..<10 {
                if let _ = vectorService.embed(text: "I want a relaxing comedy movie") {
                    vectorSuccess = true
                }
            }
            let vectorTime = (CFAbsoluteTimeGetCurrent() - vectorStart) * 1000 / 10
            benchmarkResults.append(BenchmarkResult(
                name: "Vector Embedding",
                target: "<10ms/op",
                actual: vectorSuccess ? String(format: "%.2fms", vectorTime) : "FAILED",
                status: vectorSuccess ? (vectorTime < 10 ? .pass : (vectorTime < 50 ? .slow : .fail)) : .fail,
                isReal: true
            ))

            // 3. Semantic Search (REAL - searches MediaItem.samples)
            let searchStart = CFAbsoluteTimeGetCurrent()
            let searchResults = vectorService.search(
                query: "feel-good comedy for tired evening",
                in: MediaItem.samples,
                limit: 5
            )
            let searchTime = (CFAbsoluteTimeGetCurrent() - searchStart) * 1000
            benchmarkResults.append(BenchmarkResult(
                name: "Semantic Search (\(MediaItem.samples.count) items)",
                target: "<100ms",
                actual: String(format: "%.1fms (%d results)", searchTime, searchResults.count),
                status: searchTime < 100 ? .pass : (searchTime < 500 ? .slow : .fail),
                isReal: true
            ))

            // 4. Mood Classification (Rule-based fallback)
            let vibePredictor = VibePredictor()
            let moodStart = CFAbsoluteTimeGetCurrent()
            for _ in 0..<100 {
                _ = vibePredictor.predictVibe(
                    hrv: 35.0,
                    sleepHours: 5.5,
                    steps: 3000.0
                )
            }
            let moodTime = (CFAbsoluteTimeGetCurrent() - moodStart) * 1000 / 100
            benchmarkResults.append(BenchmarkResult(
                name: "Mood (rule-based)",
                target: "<1ms/op",
                actual: String(format: "%.3fms", moodTime),
                status: moodTime < 1 ? .pass : (moodTime < 5 ? .slow : .fail),
                isReal: true
            ))

            // 5. Recommendation Engine (REAL - filters and scores)
            let recEngine = RecommendationEngine()
            let recStart = CFAbsoluteTimeGetCurrent()
            for _ in 0..<10 {
                _ = recEngine.generateRecommendations(
                    mood: MoodState(energy: .low, stress: .stressed),
                    preferences: UserPreferences.default,
                    limit: 10
                )
            }
            let recTime = (CFAbsoluteTimeGetCurrent() - recStart) * 1000 / 10
            benchmarkResults.append(BenchmarkResult(
                name: "Rule-Based Recommendations",
                target: "<5ms",
                actual: String(format: "%.2fms", recTime),
                status: recTime < 5 ? .pass : (recTime < 20 ? .slow : .fail),
                isReal: true
            ))

            // 6. JSON Serialization (REAL - MoodState encode/decode)
            let encoder = JSONEncoder()
            let decoder = JSONDecoder()
            let jsonStart = CFAbsoluteTimeGetCurrent()
            for _ in 0..<100 {
                if let data = try? encoder.encode(MoodState.default) {
                    _ = try? decoder.decode(MoodState.self, from: data)
                }
            }
            let jsonTime = (CFAbsoluteTimeGetCurrent() - jsonStart) * 1000 / 100
            benchmarkResults.append(BenchmarkResult(
                name: "JSON Serialize/Deserialize",
                target: "<1ms/op",
                actual: String(format: "%.3fms", jsonTime),
                status: jsonTime < 1 ? .pass : (jsonTime < 5 ? .slow : .fail),
                isReal: true
            ))

            // 7. WASM Module Load (REAL - WasmKit runtime)
            let wasmResult = await benchmarkWASMLoad()
            benchmarkResults.append(wasmResult)

            // 8. WASM Dot Product (REAL - bench_dot_product from ruvector.wasm)
            let wasmDotResult = await benchmarkWASMDotProduct()
            benchmarkResults.append(wasmDotResult)

            // 9. WASM HNSW Search (REAL - nearest neighbor lookup)
            let wasmHNSWResult = await benchmarkWASMHNSW()
            benchmarkResults.append(wasmHNSWResult)

            // 10. WASM ML Inference (REAL - ios_get_energy from ruvector.wasm)
            let wasmMLResult = await benchmarkWASMMLInference()
            benchmarkResults.append(wasmMLResult)

            // 11. Memory Usage (REAL - actual process memory)
            let memoryBytes = getMemoryUsage()
            let memoryMB = Double(memoryBytes) / 1_000_000
            benchmarkResults.append(BenchmarkResult(
                name: "Memory Usage",
                target: "<100MB",
                actual: String(format: "%.1fMB", memoryMB),
                status: memoryMB < 100 ? .pass : (memoryMB < 200 ? .slow : .fail),
                isReal: true
            ))

            let totalTimeMs = (CFAbsoluteTimeGetCurrent() - startTime) * 1000

            await MainActor.run {
                results = benchmarkResults
                totalTime = String(format: "%.0fms", totalTimeMs)
                memoryUsage = String(format: "%.1fMB", memoryMB)
                isRunning = false
            }
        }
    }

    private func getMemoryUsage() -> UInt64 {
        var info = mach_task_basic_info()
        var count = mach_msg_type_number_t(MemoryLayout<mach_task_basic_info>.size) / 4

        let result = withUnsafeMutablePointer(to: &info) {
            $0.withMemoryRebound(to: integer_t.self, capacity: 1) {
                task_info(mach_task_self_, task_flavor_t(MACH_TASK_BASIC_INFO), $0, &count)
            }
        }

        return result == KERN_SUCCESS ? info.resident_size : 0
    }

    // MARK: - WASM Benchmarks (REAL - WasmKit runtime)

    private func benchmarkWASMLoad() async -> BenchmarkResult {
        guard let wasmPath = Bundle.main.path(forResource: "ruvector", ofType: "wasm") else {
            return BenchmarkResult(
                name: "WASM Module Load",
                target: "<100ms",
                actual: "NOT FOUND",
                status: .fail,
                isReal: false
            )
        }

        let bridge = RuvectorBridge()
        let start = CFAbsoluteTimeGetCurrent()

        do {
            try await bridge.load(wasmPath: wasmPath)
            let loadTime = (CFAbsoluteTimeGetCurrent() - start) * 1000

            let exports = bridge.listExports()
            let simdStatus = bridge.hasSIMD() ? "SIMD" : "scalar"
            let exportInfo = " (\(exports.count) fn, \(simdStatus))"

            // Index media items into HNSW if empty
            let initialCount = bridge.getVectorCount()
            if initialCount == 0 {
                print("📚 Indexing \(MediaItem.samples.count) media items into HNSW...")
                let vectorService = VectorEmbeddingService.shared

                for (index, item) in MediaItem.samples.enumerated() {
                    // Generate embedding for this media item
                    if let embedding = vectorService.embed(text: item.embeddingText) {
                        let floatVector = embedding.map { Float($0) }
                        let success = bridge.insertVector(floatVector, id: Int32(index))
                        if !success {
                            print("   ⚠️ Failed to index: \(item.title)")
                        }
                    }
                }
                print("📚 Indexing complete")
            }

            // Capture vector count from HNSW index
            let count = bridge.getVectorCount()
            await MainActor.run {
                vectorCount = count
            }

            return BenchmarkResult(
                name: "WASM Module Load",
                target: "<100ms",
                actual: String(format: "%.1fms%@", loadTime, exportInfo),
                status: loadTime < 100 ? .pass : (loadTime < 500 ? .slow : .fail),
                isReal: true
            )
        } catch {
            return BenchmarkResult(
                name: "WASM Module Load",
                target: "<100ms",
                actual: extractTrapReason(from: error),
                status: .fail,
                isReal: true
            )
        }
    }

    private func benchmarkWASMDotProduct() async -> BenchmarkResult {
        guard let wasmPath = Bundle.main.path(forResource: "ruvector", ofType: "wasm") else {
            return BenchmarkResult(
                name: "WASM Dot Product",
                target: "<0.1ms/op",
                actual: "NO WASM",
                status: .fail,
                isReal: false
            )
        }

        let bridge = RuvectorBridge()

        do {
            try await bridge.load(wasmPath: wasmPath)

            // Benchmark the real bench_dot_product function (128-dim vectors)
            let opTime = try bridge.benchmarkDotProduct(iterations: 100)

            if opTime < 0 {
                return BenchmarkResult(
                    name: "WASM Dot Product",
                    target: "<0.1ms/op",
                    actual: "fn not found",
                    status: .fail,
                    isReal: true
                )
            }

            return BenchmarkResult(
                name: "WASM Dot Product (128-dim)",
                target: "<0.1ms/op",
                actual: String(format: "%.4fms", opTime),
                status: opTime < 0.1 ? .pass : (opTime < 1 ? .slow : .fail),
                isReal: true
            )
        } catch {
            return BenchmarkResult(
                name: "WASM Dot Product",
                target: "<0.1ms/op",
                actual: extractTrapReason(from: error),
                status: .fail,
                isReal: true
            )
        }
    }

    private func benchmarkWASMHNSW() async -> BenchmarkResult {
        guard let wasmPath = Bundle.main.path(forResource: "ruvector", ofType: "wasm") else {
            return BenchmarkResult(
                name: "WASM HNSW Search",
                target: "<5ms/op",
                actual: "NO WASM",
                status: .fail,
                isReal: false
            )
        }

        let bridge = RuvectorBridge()

        do {
            try await bridge.load(wasmPath: wasmPath)

            // Benchmark HNSW nearest neighbor search
            let opTime = try bridge.benchmarkHNSWSearch(iterations: 10)

            if opTime < 0 {
                return BenchmarkResult(
                    name: "WASM HNSW Search",
                    target: "<5ms/op",
                    actual: "fn not found",
                    status: .slow, // Not a failure, just not available
                    isReal: true
                )
            }

            return BenchmarkResult(
                name: "WASM HNSW Search (k=5)",
                target: "<5ms/op",
                actual: String(format: "%.2fms", opTime),
                status: opTime < 5 ? .pass : (opTime < 20 ? .slow : .fail),
                isReal: true
            )
        } catch {
            return BenchmarkResult(
                name: "WASM HNSW Search",
                target: "<5ms/op",
                actual: extractTrapReason(from: error),
                status: .fail,
                isReal: true
            )
        }
    }

    private func benchmarkWASMMLInference() async -> BenchmarkResult {
        guard let wasmPath = Bundle.main.path(forResource: "ruvector", ofType: "wasm") else {
            return BenchmarkResult(
                name: "WASM ML Inference",
                target: "<1ms/op",
                actual: "NO WASM",
                status: .fail,
                isReal: false
            )
        }

        let bridge = RuvectorBridge()

        do {
            try await bridge.load(wasmPath: wasmPath)

            // Initialize the learner first
            try bridge.initLearner()

            // Benchmark ML inference (ios_get_energy)
            let opTime = try bridge.benchmarkMLInference(iterations: 100)

            if opTime < 0 {
                return BenchmarkResult(
                    name: "WASM ML Inference",
                    target: "<1ms/op",
                    actual: "fn not found",
                    status: .slow,
                    isReal: true
                )
            }

            return BenchmarkResult(
                name: "WASM ML Inference (energy)",
                target: "<1ms/op",
                actual: String(format: "%.4fms", opTime),
                status: opTime < 1 ? .pass : (opTime < 5 ? .slow : .fail),
                isReal: true
            )
        } catch {
            return BenchmarkResult(
                name: "WASM ML Inference",
                target: "<1ms/op",
                actual: extractTrapReason(from: error),
                status: .fail,
                isReal: true
            )
        }
    }
}

// MARK: - Helper to extract WASM trap reason

private func extractTrapReason(from error: Error) -> String {
    // WasmKit.Trap has a CustomStringConvertible that shows "Trap: <reason>"
    let description = String(describing: error)

    // Check for known trap patterns
    if description.contains("unreachable") {
        return "TRAP: unreachable"
    } else if description.contains("call stack exhausted") {
        return "TRAP: stack overflow"
    } else if description.contains("out of bounds memory") {
        return "TRAP: memory OOB"
    } else if description.contains("integer divide by zero") {
        return "TRAP: div by zero"
    } else if description.contains("integer overflow") {
        return "TRAP: int overflow"
    } else if description.contains("indirect call") {
        return "TRAP: null call"
    } else if description.contains("Trap:") {
        // Extract the reason from "Trap: <reason>"
        if let range = description.range(of: "Trap: ") {
            let reason = String(description[range.upperBound...]).prefix(20)
            return "TRAP: \(reason)"
        }
    }

    // Fallback to short description
    let shortDesc = error.localizedDescription.prefix(25)
    return "ERR: \(shortDesc)"
}

// MARK: - Stat Row

struct StatRow: View {
    let label: String
    let value: String
    let icon: String

    var body: some View {
        HStack {
            Image(systemName: icon)
                .foregroundStyle(.secondary)
                .frame(width: 24)
            Text(label)
            Spacer()
            Text(value)
                .monospacedDigit()
                .fontWeight(.medium)
                .foregroundStyle(.primary)
        }
    }
}

#Preview {
    NavigationStack {
        BenchmarkView()
    }
    .modelContainer(for: [MoodLog.self, WatchHistory.self, WatchlistItem.self], inMemory: true)
}
