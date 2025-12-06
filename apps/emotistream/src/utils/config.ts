/**
 * EmotiStream Configuration
 *
 * Central configuration for all hyperparameters, constants, and settings.
 */

/**
 * Main configuration object
 */
export const CONFIG = {
  /**
   * Reinforcement Learning Hyperparameters
   */
  rl: {
    alpha: 0.1,              // Learning rate (0 to 1)
    gamma: 0.95,             // Discount factor for future rewards (0 to 1)
    epsilon: 0.15,           // Initial exploration rate (0 to 1)
    epsilonDecay: 0.95,      // Epsilon decay factor per episode
    epsilonMin: 0.10,        // Minimum epsilon (always explore 10%)
    ucbConstant: 2.0,        // UCB exploration constant (higher = more exploration)
  },

  /**
   * State Discretization Buckets
   *
   * Number of buckets for discretizing continuous emotional state dimensions.
   * Total state space = valence * arousal * stress = 5 * 5 * 3 = 75 states
   */
  stateBuckets: {
    valence: 5,              // Valence buckets: very negative, negative, neutral, positive, very positive
    arousal: 5,              // Arousal buckets: very calm, calm, moderate, excited, very excited
    stress: 3,               // Stress buckets: low, moderate, high
  },

  /**
   * Recommendation Ranking Weights
   */
  ranking: {
    qValueWeight: 0.7,       // Weight for Q-learning score (0 to 1)
    similarityWeight: 0.3,   // Weight for emotional similarity (0 to 1)
  },

  /**
   * Reward Function Weights
   */
  reward: {
    directionWeight: 0.6,    // Weight for moving toward desired state
    magnitudeWeight: 0.4,    // Weight for distance traveled
    proximityBonus: 0.1,     // Bonus when reaching desired state
    completionPenalty: -0.3, // Penalty for not completing content
  },

  /**
   * Embedding Configuration
   */
  embedding: {
    dimensions: 1536,        // Gemini embedding dimensions (text-embedding-004)
  },

  /**
   * HNSW Index Parameters (for AgentDB)
   */
  hnsw: {
    m: 16,                   // Number of bi-directional links (higher = better recall, more memory)
    efConstruction: 200,     // Size of dynamic candidate list during construction
  },

  /**
   * API Server Configuration
   */
  api: {
    port: 3000,              // Server port
    rateLimit: 100,          // Requests per minute per IP
  },

  /**
   * Gemini API Configuration
   */
  gemini: {
    model: 'gemini-2.0-flash-exp',  // Model for emotion detection
    embeddingModel: 'text-embedding-004',  // Model for embeddings
    temperature: 0.7,        // Response temperature (0 to 1)
    maxRetries: 3,           // Max API retry attempts
    retryDelay: 1000,        // Delay between retries (ms)
  },

  /**
   * Content Profiler Configuration
   */
  contentProfiler: {
    minJourneyPoints: 5,     // Minimum emotional journey data points
    maxJourneyPoints: 20,    // Maximum emotional journey data points
  },

  /**
   * Feedback Processing Configuration
   */
  feedback: {
    minWatchTimeRatio: 0.1,  // Minimum watch time to consider valid feedback (10%)
    completionThreshold: 0.9, // Watch ratio to consider content "completed" (90%)
  },

  /**
   * Logging Configuration
   */
  logging: {
    level: process.env.LOG_LEVEL || 'info',  // Log level: debug, info, warn, error
    pretty: process.env.NODE_ENV !== 'production',  // Pretty print logs in dev
  },

  /**
   * Database Configuration
   */
  database: {
    qtablePath: process.env.QTABLE_DB_PATH || './data/qtable.db',
    contentPath: process.env.CONTENT_DB_PATH || './data/content.adb',
    backupInterval: 3600000,  // Backup interval in ms (1 hour)
  },
} as const;

/**
 * Type-safe configuration access
 */
export type AppConfig = typeof CONFIG;

/**
 * Environment-specific configuration overrides
 */
export const getConfig = () => {
  // Allow runtime overrides from environment variables
  const rl = {
    alpha: process.env.RL_ALPHA ? parseFloat(process.env.RL_ALPHA) : CONFIG.rl.alpha,
    gamma: process.env.RL_GAMMA ? parseFloat(process.env.RL_GAMMA) : CONFIG.rl.gamma,
    epsilon: process.env.RL_EPSILON ? parseFloat(process.env.RL_EPSILON) : CONFIG.rl.epsilon,
    epsilonDecay: CONFIG.rl.epsilonDecay,
    epsilonMin: CONFIG.rl.epsilonMin,
    ucbConstant: CONFIG.rl.ucbConstant,
  };

  const api = {
    port: process.env.API_PORT ? parseInt(process.env.API_PORT, 10) : CONFIG.api.port,
    rateLimit: CONFIG.api.rateLimit,
  };

  return {
    ...CONFIG,
    rl,
    api,
  };
};

/**
 * Validate configuration on startup
 */
export const validateConfig = (config: AppConfig): void => {
  // Validate RL parameters
  if (config.rl.alpha < 0 || config.rl.alpha > 1) {
    throw new Error('RL alpha must be between 0 and 1');
  }
  if (config.rl.gamma < 0 || config.rl.gamma > 1) {
    throw new Error('RL gamma must be between 0 and 1');
  }
  if (config.rl.epsilon < 0 || config.rl.epsilon > 1) {
    throw new Error('RL epsilon must be between 0 and 1');
  }

  // Validate ranking weights sum to 1
  const rankingSum = config.ranking.qValueWeight + config.ranking.similarityWeight;
  if (Math.abs(rankingSum - 1.0) > 0.001) {
    throw new Error('Ranking weights must sum to 1.0');
  }

  // Validate state buckets
  if (config.stateBuckets.valence < 2 || config.stateBuckets.arousal < 2 || config.stateBuckets.stress < 2) {
    throw new Error('State buckets must be at least 2');
  }

  // Validate API configuration
  if (config.api.port < 1024 || config.api.port > 65535) {
    throw new Error('API port must be between 1024 and 65535');
  }
};

/**
 * Export default validated configuration
 */
export default getConfig();
