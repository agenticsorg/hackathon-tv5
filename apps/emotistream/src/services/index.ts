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
import { UserStore } from '../persistence/user-store.js';

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
  public readonly userStore: UserStore;

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
    this.userStore = new UserStore();

    // Step 7: Load seed content for demo
    this.loadSeedContent();
  }

  private async loadSeedContent(): Promise<void> {
    const seedContent = [
      { contentId: 'calming-001', title: 'Ocean Waves Meditation', category: 'meditation' as const, genres: ['relaxation', 'nature'], description: 'Soothing ocean sounds for deep relaxation', platform: 'mock' as const, tags: ['calm', 'sleep'], duration: 30 },
      { contentId: 'uplifting-001', title: 'Feel Good Comedy Special', category: 'movie' as const, genres: ['comedy', 'standup'], description: 'A hilarious comedy special to lift your spirits', platform: 'mock' as const, tags: ['funny', 'happy'], duration: 60 },
      { contentId: 'inspiring-001', title: 'Nature Documentary: Earth', category: 'documentary' as const, genres: ['nature', 'science'], description: 'Beautiful nature documentary showcasing earths wonders', platform: 'mock' as const, tags: ['inspiring', 'beautiful'], duration: 90 },
      { contentId: 'exciting-001', title: 'Action Adventure Movie', category: 'movie' as const, genres: ['action', 'adventure'], description: 'Heart-pounding action adventure', platform: 'mock' as const, tags: ['thrilling', 'exciting'], duration: 120 },
      { contentId: 'peaceful-001', title: 'Yoga for Stress Relief', category: 'meditation' as const, genres: ['yoga', 'wellness'], description: 'Gentle yoga session for stress relief', platform: 'mock' as const, tags: ['peaceful', 'calm'], duration: 45 },
      { contentId: 'drama-001', title: 'Heartwarming Drama Series', category: 'series' as const, genres: ['drama', 'family'], description: 'Emotional drama about family bonds', platform: 'mock' as const, tags: ['emotional', 'touching'], duration: 50 },
      { contentId: 'music-001', title: 'Classical Piano Collection', category: 'music' as const, genres: ['classical', 'instrumental'], description: 'Beautiful piano pieces for relaxation', platform: 'mock' as const, tags: ['calming', 'focus'], duration: 60 },
      { contentId: 'thriller-001', title: 'Mystery Thriller Movie', category: 'movie' as const, genres: ['thriller', 'mystery'], description: 'Gripping mystery with unexpected twists', platform: 'mock' as const, tags: ['suspense', 'engaging'], duration: 110 },
      { contentId: 'short-001', title: 'Funny Animal Compilation', category: 'short' as const, genres: ['comedy', 'animals'], description: 'Hilarious animal clips to brighten your day', platform: 'mock' as const, tags: ['cute', 'funny'], duration: 15 },
      { contentId: 'documentary-002', title: 'Mind Science Documentary', category: 'documentary' as const, genres: ['science', 'psychology'], description: 'Fascinating exploration of the human mind', platform: 'mock' as const, tags: ['educational', 'thought-provoking'], duration: 75 },
    ];

    const profiler = this.recommendationEngine.getProfiler();
    for (const content of seedContent) {
      await profiler.profile(content);
    }
    console.log(`🎬 Loaded ${seedContent.length} seed content items`);
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
