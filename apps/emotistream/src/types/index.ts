/**
 * EmotiStream Core Type Definitions
 *
 * This file contains all shared TypeScript interfaces and types
 * used across the EmotiStream emotion-aware recommendation system.
 */

/**
 * EmotionalState - Russell's Circumplex Model + Plutchik's Wheel
 *
 * Represents a user's emotional state in a continuous 2D space with
 * additional stress measurement and emotion classification.
 */
export interface EmotionalState {
  valence: number;              // -1 (negative) to +1 (positive)
  arousal: number;              // -1 (calm) to +1 (excited)
  stressLevel: number;          // 0 (relaxed) to 1 (stressed)
  primaryEmotion: string;       // joy, sadness, anger, fear, etc.
  emotionVector: Float32Array;  // Plutchik 8D: joy, trust, fear, surprise, sadness, disgust, anger, anticipation
  confidence: number;           // 0 to 1, confidence in detection
  timestamp: number;            // Unix timestamp
}

/**
 * DesiredState - Target emotional state for recommendations
 *
 * Describes where the user wants to move emotionally through content consumption.
 */
export interface DesiredState {
  targetValence: number;        // Desired valence (-1 to +1)
  targetArousal: number;        // Desired arousal (-1 to +1)
  targetStress: number;         // Desired stress (0 to 1)
  intensity: 'subtle' | 'moderate' | 'significant';  // Transition intensity
  reasoning: string;            // Why this target state was chosen
}

/**
 * QTableEntry - Q-Learning state-action value
 *
 * Stores learned Q-values for state-action pairs in the RL policy.
 */
export interface QTableEntry {
  userId: string;               // User identifier
  stateHash: string;            // Discretized state key (format: "v:a:s")
  contentId: string;            // Action (content) identifier
  qValue: number;               // Learned Q-value
  visitCount: number;           // Number of times this state-action was visited
  lastUpdated: number;          // Unix timestamp of last update
}

/**
 * ContentMetadata - Basic content information
 */
export interface ContentMetadata {
  id: string;                   // Unique content identifier
  title: string;                // Content title
  type: 'movie' | 'series' | 'documentary';  // Content type
  genre: string[];              // Genres (e.g., ['action', 'thriller'])
  duration: number;             // Duration in minutes
  releaseYear: number;          // Year of release
  description: string;          // Content description/synopsis
}

/**
 * EmotionalContentProfile - Content with emotional characteristics
 *
 * Extends ContentMetadata with emotional journey and vectors for similarity matching.
 */
export interface EmotionalContentProfile extends ContentMetadata {
  emotionalJourney: EmotionalState[];  // Sequence of emotional states during content
  dominantEmotion: string;             // Primary emotion evoked
  emotionalVector: Float32Array;       // 8D emotional embedding (Plutchik)
  transitionVector: Float32Array;      // Emotional transition pattern vector
}

/**
 * Recommendation - Content recommendation with RL-based ranking
 */
export interface Recommendation {
  contentId: string;                   // Content identifier
  title: string;                       // Content title
  qValue: number;                      // Q-Learning value
  similarityScore: number;             // Emotional profile similarity
  combinedScore: number;               // Final ranking score
  predictedOutcome: PredictedOutcome;  // Expected emotional outcome
  reasoning: string;                   // Explanation for recommendation
  isExploration: boolean;              // Whether this is an exploration action
}

/**
 * PredictedOutcome - Expected emotional state after content consumption
 */
export interface PredictedOutcome {
  expectedValence: number;      // Predicted valence
  expectedArousal: number;      // Predicted arousal
  expectedStress: number;       // Predicted stress
  confidence: number;           // Prediction confidence (0 to 1)
}

/**
 * EmotionalExperience - Replay buffer entry for batch learning
 *
 * Stores state transition experiences for offline learning and analysis.
 */
export interface EmotionalExperience {
  userId: string;               // User identifier
  timestamp: number;            // Unix timestamp
  stateBefore: EmotionalState;  // Emotional state before content
  action: string;               // Content ID consumed
  stateAfter: EmotionalState;   // Emotional state after content
  reward: number;               // Calculated reward
  desiredState: DesiredState;   // User's target state
}

/**
 * FeedbackRequest - User feedback after content consumption
 */
export interface FeedbackRequest {
  userId: string;               // User identifier
  contentId: string;            // Content consumed
  actualPostState: EmotionalState;  // Measured emotional state after
  watchDuration: number;        // Actual watch time in minutes
  completed: boolean;           // Whether content was fully consumed
  explicitRating?: number;      // Optional 1-5 star rating
}

/**
 * FeedbackResponse - System response to feedback
 */
export interface FeedbackResponse {
  reward: number;               // Calculated reward value
  policyUpdated: boolean;       // Whether Q-table was updated
  newQValue: number;            // Updated Q-value
  learningProgress: LearningProgress;  // Current learning metrics
}

/**
 * LearningProgress - RL policy learning metrics
 */
export interface LearningProgress {
  totalExperiences: number;     // Total experiences collected
  avgReward: number;            // Average reward across experiences
  explorationRate: number;      // Current epsilon value
  convergenceScore: number;     // Policy convergence metric (0 to 1)
}

/**
 * ActionSelection - Selected action from policy
 */
export interface ActionSelection {
  contentId: string;            // Selected content ID
  qValue: number;               // Q-value of selected action
  isExploration: boolean;       // Whether this is exploration
  explorationReason?: 'epsilon' | 'ucb' | 'novelty';  // Exploration strategy used
}

/**
 * PolicyUpdate - Q-value update event
 */
export interface PolicyUpdate {
  stateHash: string;            // State key
  contentId: string;            // Action (content)
  oldQValue: number;            // Previous Q-value
  newQValue: number;            // Updated Q-value
  tdError: number;              // Temporal difference error
  reward: number;               // Reward received
}

/**
 * SearchResult - Content search result with similarity score
 */
export interface SearchResult {
  contentId: string;            // Content identifier
  score: number;                // Similarity score
  profile: EmotionalContentProfile;  // Full content profile
}

/**
 * UserProfile - User preferences and learning history
 */
export interface UserProfile {
  userId: string;               // User identifier
  baselineState: EmotionalState;  // Typical emotional baseline
  preferredGenres: string[];    // Preferred content genres
  totalWatchTime: number;       // Total minutes watched
  completionRate: number;       // Average completion rate (0 to 1)
  lastActive: number;           // Last activity timestamp
}

/**
 * RewardComponents - Breakdown of reward calculation
 */
export interface RewardComponents {
  directionScore: number;       // Score for moving toward desired state
  magnitudeScore: number;       // Score for distance traveled
  proximityBonus: number;       // Bonus for reaching desired state
  completionPenalty: number;    // Penalty for not completing content
  totalReward: number;          // Final reward value
}

/**
 * EmbeddingRequest - Request for content embedding generation
 */
export interface EmbeddingRequest {
  contentId: string;            // Content to embed
  metadata: ContentMetadata;    // Content metadata
  emotionalJourney: EmotionalState[];  // Emotional journey data
}

/**
 * EmbeddingResponse - Generated content embedding
 */
export interface EmbeddingResponse {
  contentId: string;            // Content identifier
  embedding: Float32Array;      // Generated embedding vector
  dimensions: number;           // Embedding dimensions
  model: string;                // Model used for embedding
}

/**
 * APIError - Standardized API error response
 */
export interface APIError {
  error: string;                // Error type
  message: string;              // Human-readable error message
  details?: unknown;            // Additional error details
  timestamp: number;            // Error timestamp
}

/**
 * HealthStatus - System health check response
 */
export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  uptime: number;               // Uptime in seconds
  components: {
    database: boolean;          // Database connectivity
    gemini: boolean;            // Gemini API availability
    memory: number;             // Memory usage percentage
  };
  timestamp: number;            // Status check timestamp
}
