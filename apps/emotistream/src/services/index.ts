/**
 * ServiceContainer - Singleton for dependency injection
 * Manages lifecycle and dependencies of all core EmotiStream modules
 */

import { EmotionDetector } from '../emotion/detector.js';
import { RLPolicyEngine } from '../rl/policy-engine.js';
import { RecommendationEngine } from '../recommendations/engine.js';
import { FeedbackProcessor } from '../feedback/processor.js';
import { QTable } from '../rl/q-table.js';
import { RewardCalculator } from '../rl/reward-calculator.js';
import { EpsilonGreedyStrategy } from '../rl/exploration/epsilon-greedy.js';
import { ContentProfiler } from '../content/profiler.js';
import { JWTService } from '../auth/jwt-service.js';
import { PasswordService } from '../auth/password-service.js';
import { IUserStore, getUserStoreAdapter } from '../persistence/user-store-adapter.js';
import { initializeDatabase, checkConnection } from '../persistence/postgres-client.js';
import { createLogger } from '../utils/logger.js';

const logger = createLogger('ServiceContainer');

export class ServiceContainer {
  private static instance: ServiceContainer;

  // Core services
  public readonly emotionDetector: EmotionDetector;
  public readonly qTable: QTable;
  public readonly rewardCalculator: RewardCalculator;
  public readonly explorationStrategy: EpsilonGreedyStrategy;
  public readonly policyEngine: RLPolicyEngine;
  public readonly recommendationEngine: RecommendationEngine;
  public readonly feedbackProcessor: FeedbackProcessor;
  public readonly contentProfiler: ContentProfiler;

  // Auth services
  public readonly jwtService: JWTService;
  public readonly passwordService: PasswordService;
  public readonly userStore: IUserStore;

  private initialized: boolean = false;

  private constructor() {
    // Step 1: Initialize foundational services
    this.emotionDetector = new EmotionDetector();
    this.qTable = new QTable();
    this.rewardCalculator = new RewardCalculator();
    this.contentProfiler = new ContentProfiler();

    // Step 2: Initialize exploration strategy
    this.explorationStrategy = new EpsilonGreedyStrategy(
      0.15,  // Initial epsilon (15% exploration)
      0.01,  // Minimum epsilon (1% exploration)
      0.995  // Decay rate per experience
    );

    // Step 3: Initialize RL policy engine
    this.policyEngine = new RLPolicyEngine(
      this.qTable,
      this.rewardCalculator,
      this.explorationStrategy
    );

    // Step 4: Initialize recommendation engine
    this.recommendationEngine = new RecommendationEngine();

    // Step 5: Initialize feedback processor
    this.feedbackProcessor = new FeedbackProcessor();

    // Step 6: Initialize auth services
    this.jwtService = new JWTService();
    this.passwordService = new PasswordService();
    // User store is initialized later in initialize() to support PostgreSQL
    this.userStore = getUserStoreAdapter();
  }

  /**
   * Initialize async services (database, TMDB content loading)
   * Must be called after getInstance() before using recommendations
   */
  public async initialize(): Promise<void> {
    if (this.initialized) return;

    logger.info('Initializing EmotiStream services...');

    // Initialize PostgreSQL database if enabled
    const usePostgres = process.env.USE_POSTGRES === 'true';
    if (usePostgres) {
      logger.info('PostgreSQL mode enabled');
      try {
        const connected = await checkConnection();
        if (connected) {
          logger.info('Database connection established');
          await initializeDatabase();
          logger.info('Database schema initialized');
        } else {
          logger.warn('Database connection failed, falling back to in-memory storage');
        }
      } catch (error) {
        logger.error('Database initialization error', error);
        logger.warn('Falling back to in-memory storage');
      }
    } else {
      logger.info('Using in-memory storage (set USE_POSTGRES=true for persistence)');
    }

    // Initialize recommendation engine with TMDB content (or mock fallback)
    await this.recommendationEngine.initialize(100);

    const source = this.recommendationEngine.isUsingTMDB() ? 'TMDB (real movies/TV)' : 'Mock data';
    logger.info(`Content source: ${source}`);

    this.initialized = true;
    logger.info('EmotiStream services ready');
  }

  /**
   * Check if services are initialized
   */
  public isInitialized(): boolean {
    return this.initialized;
  }

  /**
   * Check if using real TMDB data
   */
  public isUsingTMDB(): boolean {
    return this.recommendationEngine.isUsingTMDB();
  }

  public static getInstance(): ServiceContainer {
    if (!ServiceContainer.instance) {
      ServiceContainer.instance = new ServiceContainer();
    }
    return ServiceContainer.instance;
  }

  public static resetInstance(): void {
    ServiceContainer.instance = null as any;
  }

  public getExplorationRate(): number {
    return this.explorationStrategy.getEpsilon();
  }
}

export const getServices = () => ServiceContainer.getInstance();
