import Foundation
import NaturalLanguage

class VectorEmbeddingService {
    static let shared = VectorEmbeddingService()
    
    private let embedding = NLEmbedding.sentenceEmbedding(for: .english)
    
    // Cache for item embeddings to avoid re-computing
    private var itemCache: [String: [Double]] = [:]
    
    private init() {}
    
    /// Generate a vector embedding for a given text string
    func embed(text: String) -> [Double]? {
        return embedding?.vector(for: text)
    }
    
    /// Calculate cosine similarity between two vectors
    func cosineSimilarity(_ v1: [Double], _ v2: [Double]) -> Double {
        guard v1.count == v2.count else { return 0.0 }
        
        var dotProduct = 0.0
        var norm1 = 0.0
        var norm2 = 0.0
        
        for i in 0..<v1.count {
            dotProduct += v1[i] * v2[i]
            norm1 += v1[i] * v1[i]
            norm2 += v2[i] * v2[i]
        }
        
        if norm1 == 0 || norm2 == 0 { return 0.0 }
        return dotProduct / (sqrt(norm1) * sqrt(norm2))
    }
    
    /// Search for items matching the query text semantically
    func search(query: String, in items: [MediaItem], limit: Int = 10) -> [MediaItem] {
        guard let queryVector = embed(text: query) else {
            print("VectorEmbeddingService: Failed to embed query")
            return []
        }
        
        // 1. Ensure all items have embeddings (lazy load)
        var enrichedItems = items
        for i in 0..<enrichedItems.count {
            if enrichedItems[i].semanticVector == nil {
                if let cached = itemCache[enrichedItems[i].id] {
                    enrichedItems[i].semanticVector = cached
                } else if let vector = embed(text: enrichedItems[i].embeddingText) {
                    enrichedItems[i].semanticVector = vector
                    itemCache[enrichedItems[i].id] = vector
                }
            }
        }
        
        // 2. Score items
        let scoredItems = enrichedItems.compactMap { item -> (MediaItem, Double)? in
            guard let itemVector = item.semanticVector else { return nil }
            let score = cosineSimilarity(queryVector, itemVector)
            return (item, score)
        }
        
        // 3. Sort by similarity
        let sortedItems = scoredItems.sorted { $0.1 > $1.1 }
        
        // Debug print
        print("--- Semantic Search Results for: '\(query)' ---")
        for (item, score) in sortedItems.prefix(5) {
            print("[\(String(format: "%.3f", score))] \(item.title)")
        }
        
        return sortedItems.prefix(limit).map { $0.0 }
    }
}
