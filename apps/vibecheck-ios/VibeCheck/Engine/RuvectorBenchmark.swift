//
// RuvectorBenchmark.swift
// VibeCheck
//
// Performance benchmark for Ruvector WASM integration
// Run this from the app to measure real-world performance
//

import Foundation
import SwiftUI

@MainActor
class RuvectorBenchmark: ObservableObject {
    @Published var results: [BenchmarkResult] = []
    @Published var isRunning = false
    
    struct BenchmarkResult: Identifiable {
        let id = UUID()
        let name: String
        let duration: TimeInterval
        let passed: Bool
        let target: TimeInterval?
        
        var status: String {
            guard let target = target else { return passed ? "✅ PASS" : "❌ FAIL" }
            return duration < target ? "✅ PASS" : "⚠️ SLOW"
        }
        
        var details: String {
            guard let target = target else { return "\(Int(duration * 1000))ms" }
            return "\(Int(duration * 1000))ms (target: \(Int(target * 1000))ms)"
        }
    }
    
    private let bridge = RuvectorBridge()
    
    func runBenchmarks() async {
        isRunning = true
        results = []
        
        print("🔬 Starting Ruvector Performance Benchmarks...")
        
        // Benchmark 1: WASM Load Time
        await benchmarkWASMLoad()
        
        // Benchmark 2: Context Mapping
        await benchmarkContextMapping()
        
        // Benchmark 3: Watch Event Recording
        await benchmarkWatchEvent()
        
        // Benchmark 4: Recommendation Query
        await benchmarkRecommendations()
        
        // Benchmark 5: State Persistence
        await benchmarkPersistence()
        
        // Benchmark 6: Memory Usage
        await benchmarkMemory()
        
        isRunning = false
        
        print("✅ Benchmarks Complete!")
        printSummary()
    }
    
    // MARK: - Individual Benchmarks
    
    private func benchmarkWASMLoad() async {
        print("\n📦 Benchmark 1: WASM Load Time")
        
        guard let wasmPath = Bundle.main.path(forResource: "ruvector", ofType: "wasm") else {
            results.append(BenchmarkResult(
                name: "WASM Load",
                duration: 0,
                passed: false,
                target: 0.1
            ))
            print("❌ WASM file not found")
            return
        }
        
        let start = Date()
        
        do {
            try await bridge.load(wasmPath: wasmPath)
            let duration = Date().timeIntervalSince(start)
            
            results.append(BenchmarkResult(
                name: "WASM Load",
                duration: duration,
                passed: bridge.isReady,
                target: 0.1 // 100ms target
            ))
            
            print("✅ Loaded in \(Int(duration * 1000))ms (target: 100ms)")
        } catch {
            results.append(BenchmarkResult(
                name: "WASM Load",
                duration: Date().timeIntervalSince(start),
                passed: false,
                target: 0.1
            ))
            print("❌ Failed: \(error)")
        }
    }
    
    private func benchmarkContextMapping() async {
        print("\n🗺️  Benchmark 2: Context Mapping")
        
        let context = VibeContext(
            mood: MoodState(energy: .high, stress: .low),
            biometrics: Biometrics(
                hrv: .init(value: 50, date: Date()),
                sleep: .init(hours: 7.5, quality: 0.8),
                activity: .init(steps: 8000, activeMinutes: 45)
            ),
            keywords: ["action", "intense"]
        )
        
        let iterations = 1000
        let start = Date()
        
        for _ in 0..<iterations {
            _ = bridge.mapToVibeState(context)
        }
        
        let duration = Date().timeIntervalSince(start)
        let avgDuration = duration / Double(iterations)
        
        results.append(BenchmarkResult(
            name: "Context Mapping (1K ops)",
            duration: avgDuration,
            passed: avgDuration < 0.001,
            target: 0.001 // 1ms per operation
        ))
        
        print("✅ \(iterations) mappings in \(Int(duration * 1000))ms")
        print("   Average: \(Int(avgDuration * 1_000_000))μs per operation")
    }
    
    private func benchmarkWatchEvent() async {
        print("\n📝 Benchmark 3: Watch Event Recording")
        
        guard bridge.isReady else {
            print("⚠️  Skipped (WASM not loaded)")
            return
        }
        
        let item = MediaItem(
            id: "bench-1",
            title: "Benchmark Movie",
            overview: "Test",
            genres: [Genre(name: "Action")],
            tone: ["intense"],
            intensity: 0.8,
            runtime: 120,
            year: 2024,
            platforms: []
        )
        
        let context = VibeContext(
            mood: MoodState(energy: .high, stress: .low),
            biometrics: mockBiometrics(),
            keywords: ["action"]
        )
        
        let start = Date()
        
        do {
            try await bridge.recordWatchEvent(item, context: context, durationSeconds: 3600)
            let duration = Date().timeIntervalSince(start)
            
            results.append(BenchmarkResult(
                name: "Watch Event Recording",
                duration: duration,
                passed: duration < 0.005,
                target: 0.005 // 5ms target
            ))
            
            print("✅ Recorded in \(Int(duration * 1000))ms (target: 5ms)")
        } catch {
            print("❌ Failed: \(error)")
        }
    }
    
    private func benchmarkRecommendations() async {
        print("\n🎯 Benchmark 4: Recommendation Query")
        
        guard bridge.isReady else {
            print("⚠️  Skipped (WASM not loaded)")
            return
        }
        
        let context = VibeContext(
            mood: MoodState(energy: .balanced, stress: .low),
            biometrics: mockBiometrics(),
            keywords: ["comedy"]
        )
        
        let start = Date()
        
        do {
            let recommendations = try await bridge.getRecommendations(for: context, limit: 10)
            let duration = Date().timeIntervalSince(start)
            
            results.append(BenchmarkResult(
                name: "Recommendation Query (10 items)",
                duration: duration,
                passed: duration < 0.05,
                target: 0.05 // 50ms target
            ))
            
            print("✅ \(recommendations.count) recommendations in \(Int(duration * 1000))ms (target: 50ms)")
        } catch {
            print("❌ Failed: \(error)")
        }
    }
    
    private func benchmarkPersistence() async {
        print("\n💾 Benchmark 5: State Persistence")
        
        guard bridge.isReady else {
            print("⚠️  Skipped (WASM not loaded)")
            return
        }
        
        // Save
        let saveStart = Date()
        do {
            let stateData = try await bridge.saveState()
            let saveDuration = Date().timeIntervalSince(saveStart)
            
            print("✅ Saved \(stateData.count) bytes in \(Int(saveDuration * 1000))ms")
            
            // Load
            let loadStart = Date()
            try await bridge.loadState(stateData)
            let loadDuration = Date().timeIntervalSince(loadStart)
            
            results.append(BenchmarkResult(
                name: "State Save",
                duration: saveDuration,
                passed: saveDuration < 0.01,
                target: 0.01
            ))
            
            results.append(BenchmarkResult(
                name: "State Load",
                duration: loadDuration,
                passed: loadDuration < 0.01,
                target: 0.01
            ))
            
            print("✅ Loaded in \(Int(loadDuration * 1000))ms")
        } catch {
            print("❌ Failed: \(error)")
        }
    }
    
    private func benchmarkMemory() async {
        print("\n🧠 Benchmark 6: Memory Usage")
        
        let memoryBefore = getMemoryUsage()
        
        // Perform some operations
        for i in 0..<100 {
            let item = MediaItem(
                id: "mem-\(i)",
                title: "Movie \(i)",
                overview: "Test",
                genres: [Genre(name: "Action")],
                tone: [],
                intensity: 0.5,
                runtime: 90,
                year: 2024,
                platforms: []
            )
            
            try? await bridge.recordWatchEvent(
                item,
                context: VibeContext(
                    mood: MoodState(energy: .balanced, stress: .low),
                    biometrics: mockBiometrics(),
                    keywords: []
                ),
                durationSeconds: 60
            )
        }
        
        let memoryAfter = getMemoryUsage()
        let memoryDelta = memoryAfter - memoryBefore
        
        results.append(BenchmarkResult(
            name: "Memory Usage (100 events)",
            duration: Double(memoryDelta) / 1_000_000, // Convert to MB for display
            passed: memoryDelta < 15_000_000, // 15 MB
            target: 15.0
        ))
        
        print("📊 Memory: \(memoryBefore / 1_000_000)MB → \(memoryAfter / 1_000_000)MB")
        print("   Delta: \(memoryDelta / 1_000_000)MB (target: < 15MB)")
    }
    
    // MARK: - Helpers
    
    private func mockBiometrics() -> Biometrics {
        Biometrics(
            hrv: .init(value: 45, date: Date()),
            sleep: .init(hours: 7.0, quality: 0.7),
            activity: .init(steps: 5000, activeMinutes: 30)
        )
    }
    
    private func getMemoryUsage() -> Int64 {
        var info = mach_task_basic_info()
        var count = mach_msg_type_number_t(MemoryLayout<mach_task_basic_info>.size)/4
        
        let kerr: kern_return_t = withUnsafeMutablePointer(to: &info) {
            $0.withMemoryRebound(to: integer_t.self, capacity: 1) {
                task_info(mach_task_self_,
                         task_flavor_t(MACH_TASK_BASIC_INFO),
                         $0,
                         &count)
            }
        }
        
        if kerr == KERN_SUCCESS {
            return Int64(info.resident_size)
        }
        return 0
    }
    
    private func printSummary() {
        print("\n" + String(repeating: "=", count: 60))
        print("📊 BENCHMARK SUMMARY")
        print(String(repeating: "=", count: 60))
        
        for result in results {
            print("\(result.status) \(result.name): \(result.details)")
        }
        
        let passed = results.filter { $0.passed }.count
        let total = results.count
        
        print(String(repeating: "=", count: 60))
        print("Result: \(passed)/\(total) benchmarks passed")
        print(String(repeating: "=", count: 60))
    }
}

// MARK: - SwiftUI View

struct BenchmarkView: View {
    @StateObject private var benchmark = RuvectorBenchmark()
    
    var body: some View {
        NavigationView {
            List {
                Section {
                    Button(action: {
                        Task {
                            await benchmark.runBenchmarks()
                        }
                    }) {
                        HStack {
                            Image(systemName: "speedometer")
                            Text("Run Benchmarks")
                            Spacer()
                            if benchmark.isRunning {
                                ProgressView()
                            }
                        }
                    }
                    .disabled(benchmark.isRunning)
                }
                
                if !benchmark.results.isEmpty {
                    Section("Results") {
                        ForEach(benchmark.results) { result in
                            HStack {
                                Text(result.status)
                                    .font(.caption)
                                VStack(alignment: .leading) {
                                    Text(result.name)
                                        .font(.headline)
                                    Text(result.details)
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
                                Spacer()
                            }
                        }
                    }
                }
            }
            .navigationTitle("Ruvector Benchmark")
        }
    }
}
