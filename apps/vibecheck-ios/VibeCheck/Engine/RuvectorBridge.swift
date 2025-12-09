//
// RuvectorBridge.swift
// VibeCheck
//
// Swift wrapper around Ruvector WASM for on-device learning
// Uses WasmKit (https://github.com/swiftwasm/WasmKit) for pure-Swift WASM execution
//

import Foundation
import WasmKit
import WasmKitWASI

/// Bridge between VibeCheck and Ruvector WASM recommendation engine
///
/// Provides privacy-preserving on-device learning with high-performance WASM execution.
/// All learning happens locally - zero network requests.
@available(iOS 15.0, *)
class RuvectorBridge {

    // MARK: - Types

    enum RuvectorError: Error, LocalizedError {
        case wasmNotLoaded
        case invalidPath
        case loadFailed(String)
        case functionNotFound(String)
        case instantiationFailed(String)

        var errorDescription: String? {
            switch self {
            case .wasmNotLoaded: return "WASM module not loaded"
            case .invalidPath: return "Invalid WASM file path"
            case .loadFailed(let msg): return "Failed to load WASM: \(msg)"
            case .functionNotFound(let name): return "Function '\(name)' not found in WASM exports"
            case .instantiationFailed(let msg): return "WASM instantiation failed: \(msg)"
            }
        }
    }

    // MARK: - Properties

    private var engine: Engine?
    private var store: Store?
    private var wasmModule: Module?
    private var wasmInstance: Instance?

    /// Whether the WASM module is loaded and ready
    private(set) var isReady: Bool = false

    /// Time taken to load the WASM module (for benchmarking)
    private(set) var loadTimeMs: Double = 0

    /// List of exported function names (for debugging)
    private(set) var exportedFunctions: [String] = []

    // MARK: - Initialization

    init() {}

    // MARK: - Lifecycle

    /// Load the WASM module from a file path
    ///
    /// - Parameter wasmPath: Path to the .wasm file
    /// - Throws: RuvectorError if loading fails
    func load(wasmPath: String) async throws {
        guard FileManager.default.fileExists(atPath: wasmPath) else {
            throw RuvectorError.invalidPath
        }

        let startTime = CFAbsoluteTimeGetCurrent()

        do {
            // Load WASM bytes
            let wasmData = try Data(contentsOf: URL(fileURLWithPath: wasmPath))
            let wasmBytes = Array(wasmData)

            // Create WasmKit engine and store
            self.engine = Engine()
            self.store = Store(engine: engine!)

            // Parse WASM module using WasmKit's parseWasm function
            self.wasmModule = try parseWasm(bytes: wasmBytes)

            // Create WASI bridge to provide system imports (fd_write, random_get, etc.)
            // ruvector.wasm requires WASI for I/O and random number generation
            let wasi = try WASIBridgeToHost()
            var imports = Imports()
            wasi.link(to: &imports, store: store!)

            // Instantiate module with WASI imports
            self.wasmInstance = try wasmModule!.instantiate(store: store!, imports: imports)

            // Record exported functions for debugging
            self.exportedFunctions = wasmInstance!.exports.map { $0.0 }

            // VERIFICATION: Actually call a WASM function to confirm execution works
            // This catches issues where module loads but functions trap
            try verifyWASMExecution()

            // INITIALIZATION: Call rec_init and ios_learner_init to enable subsystems
            // This makes bench_dot_product and ios_get_energy work
            try initializeSubsystems()

            // Calculate load time
            self.loadTimeMs = (CFAbsoluteTimeGetCurrent() - startTime) * 1000

            self.isReady = true
            print("✅ RuvectorBridge: WASM loaded and verified in \(String(format: "%.1f", loadTimeMs))ms")
            print("   Exports: \(exportedFunctions.joined(separator: ", "))")

        } catch let error as RuvectorError {
            throw error
        } catch {
            throw RuvectorError.loadFailed(error.localizedDescription)
        }
    }

    /// Load the WASM module from bundle
    func loadFromBundle() async throws {
        guard let path = Bundle.main.path(forResource: "ruvector", ofType: "wasm") else {
            throw RuvectorError.invalidPath
        }
        try await load(wasmPath: path)
    }

    // MARK: - Verification & Initialization

    /// Initialize the recommendation engine subsystems
    /// Must be called after instantiation to enable benchmark functions
    private func initializeSubsystems() throws {
        // 1. Call rec_init to initialize recommendation engine (required for bench_dot_product, etc.)
        if let recInitFunc = getExportedFunction(name: "rec_init") {
            do {
                _ = try recInitFunc.invoke([])
                print("   ✅ rec_init() - recommendation engine initialized")
            } catch {
                print("   ⚠️ rec_init() failed: \(error) - benchmarks may not work")
            }
        }

        // 2. Call ios_learner_init to enable ML inference
        if let learnerInitFunc = getExportedFunction(name: "ios_learner_init") {
            do {
                _ = try learnerInitFunc.invoke([])
                print("   ✅ ios_learner_init() - ML learner initialized")
            } catch {
                print("   ⚠️ ios_learner_init() failed: \(error) - ML inference may not work")
            }
        }
    }

    /// Verify WASM execution actually works by calling a simple function
    /// This catches trap errors early instead of reporting false "load success"
    private func verifyWASMExecution() throws {
        // Try multiple functions in order of simplicity
        let testFunctions = ["has_simd", "get_bridge_info", "ios_learner_iterations"]

        for funcName in testFunctions {
            if let testFunc = getExportedFunction(name: funcName) {
                do {
                    _ = try testFunc.invoke([])
                    print("   ✅ Verified: \(funcName)() executed successfully")
                    return // Success!
                } catch {
                    print("   ⚠️ \(funcName)() trapped: \(error)")
                    // Try next function
                }
            }
        }

        // If all simple functions fail, the WASM is broken
        throw RuvectorError.loadFailed("WASM functions trap on execution - module may be corrupted or incompatible")
    }

    // MARK: - Export Access

    /// Check if an exported function exists
    private func hasExportedFunction(name: String) -> Bool {
        guard let instance = wasmInstance else { return false }
        if case .function(_) = instance.export(name) {
            return true
        }
        return false
    }

    /// Get an exported function by name
    private func getExportedFunction(name: String) -> Function? {
        guard let instance = wasmInstance else { return nil }
        if case .function(let function) = instance.export(name) {
            return function
        }
        return nil
    }

    // MARK: - Benchmarking

    /// Benchmark WASM module loading
    static func benchmarkLoad(wasmPath: String) async -> (success: Bool, timeMs: Double, error: String?) {
        let bridge = RuvectorBridge()
        do {
            try await bridge.load(wasmPath: wasmPath)
            return (true, bridge.loadTimeMs, nil)
        } catch {
            return (false, 0, error.localizedDescription)
        }
    }

    /// Benchmark dot product operation (real SIMD-optimized vector math)
    /// Uses bench_dot_product from ruvector.wasm - actual hyperbolic embedding math
    func benchmarkDotProduct(iterations: Int = 100) throws -> Double {
        guard isReady else {
            throw RuvectorError.wasmNotLoaded
        }

        // Use the actual bench_dot_product function from ruvector.wasm
        // This tests real vector math used in recommendations
        if let dotFunc = getExportedFunction(name: "bench_dot_product") {
            let start = CFAbsoluteTimeGetCurrent()

            for _ in 0..<iterations {
                // bench_dot_product takes dimension size, returns f32 result
                _ = try dotFunc.invoke([.i32(128)]) // 128-dim vectors typical for embeddings
            }

            let totalTime = CFAbsoluteTimeGetCurrent() - start
            return (totalTime * 1000) / Double(iterations)
        }

        return -1 // Function not available
    }

    /// Benchmark HNSW search operation (nearest neighbor lookup)
    func benchmarkHNSWSearch(iterations: Int = 10) throws -> Double {
        guard isReady else {
            throw RuvectorError.wasmNotLoaded
        }

        // First check if we can create an HNSW index
        guard let createFunc = getExportedFunction(name: "hnsw_create"),
              let insertFunc = getExportedFunction(name: "hnsw_insert"),
              let searchFunc = getExportedFunction(name: "hnsw_search") else {
            return -1
        }

        // Create index with 64 dimensions, M=16, ef_construction=200
        _ = try createFunc.invoke([.i32(64), .i32(16), .i32(200)])

        let start = CFAbsoluteTimeGetCurrent()

        for _ in 0..<iterations {
            // Search for k=5 nearest neighbors
            _ = try searchFunc.invoke([.i32(5)])
        }

        let totalTime = CFAbsoluteTimeGetCurrent() - start
        return (totalTime * 1000) / Double(iterations)
    }

    /// Check if SIMD is available in the WASM module
    func hasSIMD() -> Bool {
        if let simdFunc = getExportedFunction(name: "has_simd") {
            if let result = try? simdFunc.invoke([]) {
                if case .i32(let value) = result.first {
                    return value != 0
                }
            }
        }
        return false
    }

    /// Get bridge info from WASM (version, capabilities)
    func getBridgeInfo() -> String? {
        if let infoFunc = getExportedFunction(name: "get_bridge_info") {
            if let result = try? infoFunc.invoke([]) {
                if case .i32(let ptr) = result.first {
                    return "ptr:\(ptr)" // Would need memory access to read string
                }
            }
        }
        return nil
    }

    /// Legacy benchmark - kept for compatibility but prefers bench_dot_product
    func benchmarkSimpleOp(iterations: Int = 1000) throws -> Double {
        // Try the real dot product benchmark first
        let dotResult = try benchmarkDotProduct(iterations: iterations)
        if dotResult >= 0 {
            return dotResult
        }

        // Fallback to generic benchmark if available
        guard isReady else {
            throw RuvectorError.wasmNotLoaded
        }

        if let benchFunc = getExportedFunction(name: "benchmark") {
            let start = CFAbsoluteTimeGetCurrent()

            for _ in 0..<iterations {
                _ = try benchFunc.invoke([])
            }

            let totalTime = CFAbsoluteTimeGetCurrent() - start
            return (totalTime * 1000) / Double(iterations)
        }

        return -1 // No benchmark function available
    }

    /// Get list of exported functions (for debugging)
    func listExports() -> [String] {
        return exportedFunctions
    }

    // MARK: - On-Device ML Learning

    /// Initialize the iOS learner for on-device personalization
    /// Must be called before using learn/predict functions
    func initLearner() throws {
        guard isReady else {
            throw RuvectorError.wasmNotLoaded
        }

        if let initFunc = getExportedFunction(name: "ios_learner_init") {
            _ = try initFunc.invoke([])
            print("✅ RuvectorBridge: iOS learner initialized")
        } else {
            throw RuvectorError.functionNotFound("ios_learner_init")
        }
    }

    /// Learn from health data (HRV, sleep, steps)
    /// - Parameters:
    ///   - hrv: Heart rate variability in ms (typically 20-100ms)
    ///   - sleepHours: Hours of sleep (0-12+)
    ///   - steps: Step count (0-30000+)
    ///   - energyLabel: User-reported energy level (0.0=exhausted, 1.0=wired)
    func learnHealth(hrv: Float, sleepHours: Float, steps: Float, energyLabel: Float) throws {
        guard isReady else {
            throw RuvectorError.wasmNotLoaded
        }

        if let learnFunc = getExportedFunction(name: "ios_learn_health") {
            // Pass health metrics and user's energy label for supervised learning
            _ = try learnFunc.invoke([
                .f32(hrv.bitPattern),
                .f32(sleepHours.bitPattern),
                .f32(steps.bitPattern),
                .f32(energyLabel.bitPattern)
            ])
        } else {
            throw RuvectorError.functionNotFound("ios_learn_health")
        }
    }

    /// Predict energy level from current health data
    /// - Parameters:
    ///   - hrv: Heart rate variability in ms
    ///   - sleepHours: Hours of sleep
    ///   - steps: Step count
    /// - Returns: Predicted energy level (0.0=exhausted to 1.0=wired)
    func predictEnergy(hrv: Float, sleepHours: Float, steps: Float) throws -> Float {
        guard isReady else {
            throw RuvectorError.wasmNotLoaded
        }

        if let getEnergyFunc = getExportedFunction(name: "ios_get_energy") {
            let result = try getEnergyFunc.invoke([
                .f32(hrv.bitPattern),
                .f32(sleepHours.bitPattern),
                .f32(steps.bitPattern)
            ])

            if case .f32(let bits) = result.first {
                return Float(bitPattern: bits)
            }
        }

        throw RuvectorError.functionNotFound("ios_get_energy")
    }

    /// Get the number of training iterations completed
    func getLearnerIterations() throws -> Int {
        guard isReady else {
            throw RuvectorError.wasmNotLoaded
        }

        if let iterFunc = getExportedFunction(name: "ios_learner_iterations") {
            let result = try iterFunc.invoke([])
            if case .i32(let count) = result.first {
                return Int(count)
            }
        }

        return 0
    }

    /// Check if this is a good time for communication (notifications)
    /// Based on learned patterns from location and communication data
    func isGoodCommTime() throws -> Bool {
        guard isReady else {
            throw RuvectorError.wasmNotLoaded
        }

        if let commFunc = getExportedFunction(name: "ios_is_good_comm_time") {
            let result = try commFunc.invoke([])
            if case .i32(let value) = result.first {
                return value != 0
            }
        }

        return true // Default to allowing communication
    }

    /// Benchmark ML inference performance
    func benchmarkMLInference(iterations: Int = 100) throws -> Double {
        guard isReady else {
            throw RuvectorError.wasmNotLoaded
        }

        guard let getEnergyFunc = getExportedFunction(name: "ios_get_energy") else {
            return -1
        }

        // Typical health values for benchmarking
        let hrv: Float = 45.0
        let sleep: Float = 7.0
        let steps: Float = 5000.0

        let start = CFAbsoluteTimeGetCurrent()

        for _ in 0..<iterations {
            _ = try getEnergyFunc.invoke([
                .f32(hrv.bitPattern),
                .f32(sleep.bitPattern),
                .f32(steps.bitPattern)
            ])
        }

        let totalTime = CFAbsoluteTimeGetCurrent() - start
        return (totalTime * 1000) / Double(iterations)
    }
}
