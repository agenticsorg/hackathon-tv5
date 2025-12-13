import NaturalLanguage

if #available(macOS 10.15, iOS 13.0, *) {
    guard let embedding = NLEmbedding.sentenceEmbedding(for: .english) else {
        print("Error: NLEmbedding model not found.")
        exit(1)
    }
    
    let text = "Comforting vibes"
    if let vector = embedding.vector(for: text) {
        print("SUCCESS: Generated vector of size \(vector.count) for '\(text)'")
        
        let v2 = embedding.vector(for: "Relaxing mood")!
        
        // Manual Cosine Sim
        var dot = 0.0
        var mag1 = 0.0
        var mag2 = 0.0
        for i in 0..<vector.count {
            dot += vector[i] * v2[i]
            mag1 += vector[i] * vector[i]
            mag2 += v2[i] * v2[i]
        }
        let sim = dot / (sqrt(mag1) * sqrt(mag2))
        print("Similarity check: \(String(format: "%.3f", sim)) (Should be > 0.5)")
        
    } else {
        print("Error: Failed to generate vector.")
    }
} else {
    print("Error: OS too old.")
}
