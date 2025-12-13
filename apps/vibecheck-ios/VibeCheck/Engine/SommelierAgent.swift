import Foundation

/// The "Movie Sommelier" Agent
/// Responsible for generating context-aware rationales for media recommendations
/// based on the user's current VibeContext.
class SommelierAgent {
    static let shared = SommelierAgent()
    
    private init() {}
    
    /// Generate a "Sommelier Note" explaining why this media fits the user's vibe
    func rationale(for item: MediaItem, context: VibeContext) -> String {
        // In the future, this is where the local LLM would be called.
        // For now, we use a sophisticated template system to simulate the agent.
        
        let mood = context.mood
        let reason = context.explanation // e.g., "you haven't slept much"
        
        // 1. Direct Match Rationale
        if item.isRewatch && mood.energy == .low {
            return "Since \(reason), rewatching a favorite like \(item.title) is perfect comfort food for your brain."
        }
        
        // 2. Vibe-Based Rationale
        switch mood.recommendationHint {
        case "comfort":
            return "You seem a bit drained. \(item.title) offers the warm, \(item.tone.first ?? "gentle") vibe you need right now."
        case "exciting":
            return "You've got energy to burn! \(item.title) matches your high revs with its intensity."
        case "calming":
            return "To help balance your stress, I picked \(item.title)—it's wonderfully \(item.tone.first ?? "chill")."
        case "gentle":
            return "Because \(reason), I've selected something soft and easy-going."
        case "engaging":
            return "You're in a balanced spot, so dive into \(item.title) for something truly gripping."
        default:
             return "Based on your balanced stats, \(item.title) is a top pick for you."
        }
    }
}
