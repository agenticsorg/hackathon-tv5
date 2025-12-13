//
// RecommendationEngine+Ruvector.swift
// VibeCheck
//
// Ruvector WASM integration for RecommendationEngine
// Add this file after completing Xcode integration steps
//

import Foundation

extension RecommendationEngine {
    
    // MARK: - Ruvector Integration
    
    /// Initialize Ruvector WASM module
    /// Call this in init() or on first use
    func initializeRuvector() async {
        guard let wasmPath = Bundle.main.path(forResource: "ruvector", ofType: "wasm") else {
            print("⚠️  ruvector.wasm not found in bundle")
            return
        }
        
        do {
            try await ruvectorBridge.load(wasmPath: wasmPath)
            print("✅ Ruvector WASM loaded successfully")
        } catch {
            print("❌ Failed to load Ruvector: \(error)")
        }
    }
    
    /// Get hybrid recommendations (Ruvector + ARW + Local)
    func getHybridRecommendations(
        for context: VibeContext,
        limit: Int = 20
    ) async -> [MediaItem] {
        var results: [MediaItem] = []
        
        // Strategy 1: Try Ruvector first (learned, personalized)
        if ruvectorBridge.isReady {
            do {
                let localRecs = try await ruvectorBridge.getRecommendations(
                    for: context,
                    limit: limit / 2  // Reserve half for remote
                )
                results.append(contentsOf: localRecs)
                print("✅ Ruvector: \(localRecs.count) recommendations")
            } catch {
                print("⚠️  Ruvector failed: \(error)")
            }
        }
        
        // Strategy 2: ARW backend (remote, semantic)
        do {
            let remoteRecs = try await arwService.search(
                query: context.keywords.joined(separator: " "),
                limit: limit / 2
            )
            results.append(contentsOf: remoteRecs)
            print("✅ ARW: \(remoteRecs.count) recommendations")
        } catch {
            print("⚠️  ARW failed: \(error)")
        }
        
        // Strategy 3: Local fallback
        if results.count < limit {
            let localRecs = localStore.recommend(
                for: context,
                limit: limit - results.count
            )
            results.append(contentsOf: localRecs)
            print("✅ Local: \(localRecs.count) recommendations")
        }
        
        // Deduplicate by ID
        var seen = Set<String>()
        results = results.filter { item in
            guard !seen.contains(item.id) else { return false }
            seen.insert(item.id)
            return true
        }
        
        return Array(results.prefix(limit))
    }
    
    /// Record watch event for learning
    func recordWatchEvent(
        _ item: MediaItem,
        context: VibeContext,
        durationSeconds: Int
    ) async {
        guard ruvectorBridge.isReady else { return }
        
        do {
            try await ruvectorBridge.recordWatchEvent(
                item,
                context: context,
                durationSeconds: durationSeconds
            )
            print("✅ Recorded watch event: \(item.title) (\(durationSeconds)s)")
        } catch {
            print("❌ Failed to record watch event: \(error)")
        }
    }
    
    /// Learn from user satisfaction
    func learnFromSatisfaction(_ satisfaction: Double) async {
        guard ruvectorBridge.isReady else { return }
        
        do {
            try await ruvectorBridge.learn(satisfaction: satisfaction)
            print("✅ Learning updated (satisfaction: \(satisfaction))")
        } catch {
            print("❌ Failed to update learning: \(error)")
        }
    }
    
    /// Save learned state to persistent storage
    func saveRuvectorState() async {
        guard ruvectorBridge.isReady else { return }
        
        do {
            let stateData = try await ruvectorBridge.saveState()
            
            // Save to UserDefaults or SwiftData
            UserDefaults.standard.set(stateData, forKey: "ruvector_state")
            
            print("✅ Ruvector state saved (\(stateData.count) bytes)")
        } catch {
            print("❌ Failed to save Ruvector state: \(error)")
        }
    }
    
    /// Load previously saved state
    func loadRuvectorState() async {
        guard ruvectorBridge.isReady else { return }
        guard let stateData = UserDefaults.standard.data(forKey: "ruvector_state") else {
            print("ℹ️  No saved Ruvector state found")
            return
        }
        
        do {
            try await ruvectorBridge.loadState(stateData)
            print("✅ Ruvector state loaded (\(stateData.count) bytes)")
        } catch {
            print("❌ Failed to load Ruvector state: \(error)")
        }
    }
}

// MARK: - Modified RecommendationEngine

/*
 Add these properties to RecommendationEngine class:
 
 class RecommendationEngine {
     // Existing properties...
     
     // NEW: Ruvector integration
     private let ruvectorBridge = RuvectorBridge()
     private let arwService = ARWService()
     private let localStore = LocalStore()
     
     init(catalog: [MediaItem] = MediaItem.samples) {
         self.catalog = catalog
         
         // Initialize Ruvector asynchronously
         Task {
             await initializeRuvector()
             await loadRuvectorState()
         }
     }
     
     // Update existing generateRecommendations to use hybrid approach:
     func generateRecommendations(
         mood: MoodState,
         preferences: UserPreferences,
         limit: Int = 5
     ) async -> [MediaItem] {
         let context = VibeContext(
             mood: mood,
             biometrics: getCurrentBiometrics(),
             keywords: mood.recommendationHint.keywords
         )
         
         return await getHybridRecommendations(for: context, limit: limit)
     }
 }
 */
