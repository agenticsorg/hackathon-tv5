/**
 * Mock Recommendation Engine for CLI Demo
 *
 * Simulates RL-based recommendations without requiring full system.
 */

import { EmotionalState, DesiredState, Recommendation } from '../../types/index.js';

interface MockContent {
  id: string;
  title: string;
  genre: string[];
  emotionalProfile: {
    valence: number;
    arousal: number;
    stress: number;
  };
}

/**
 * Mock recommendation engine with Q-learning simulation
 */
export class MockRecommendationEngine {
  private mockCatalog: MockContent[];
  private qValues: Map<string, number>;
  private explorationRate: number;

  constructor() {
    this.mockCatalog = this.createMockCatalog();
    this.qValues = this.initializeQValues();
    this.explorationRate = 0.2; // 20% exploration
  }

  /**
   * Get personalized recommendations
   */
  async getRecommendations(
    currentState: EmotionalState,
    desiredState: DesiredState,
    userId: string,
    count: number
  ): Promise<Recommendation[]> {
    const recommendations: Recommendation[] = [];

    // Calculate similarity scores for all content
    const scoredContent = this.mockCatalog.map(content => ({
      content,
      similarityScore: this.calculateSimilarity(content.emotionalProfile, desiredState),
      qValue: this.getQValue(currentState, content.id)
    }));

    // Determine exploration vs exploitation for each recommendation
    for (let i = 0; i < count && i < scoredContent.length; i++) {
      const isExploration = Math.random() < this.explorationRate;

      let selected: typeof scoredContent[0];

      if (isExploration) {
        // Exploration: select based on UCB or random
        selected = this.selectExploration(scoredContent, i);
      } else {
        // Exploitation: select highest combined score
        selected = this.selectExploitation(scoredContent, i);
      }

      const combinedScore = 0.6 * selected.qValue + 0.4 * selected.similarityScore;

      recommendations.push({
        contentId: selected.content.id,
        title: selected.content.title,
        qValue: selected.qValue,
        similarityScore: selected.similarityScore,
        combinedScore,
        predictedOutcome: {
          expectedValence: selected.content.emotionalProfile.valence,
          expectedArousal: selected.content.emotionalProfile.arousal,
          expectedStress: selected.content.emotionalProfile.stress,
          confidence: 0.75
        },
        reasoning: this.generateReasoning(selected.content, isExploration),
        isExploration
      });

      // Remove selected from pool
      scoredContent.splice(scoredContent.indexOf(selected), 1);
    }

    return recommendations;
  }

  /**
   * Calculate emotional similarity
   */
  private calculateSimilarity(
    profile: { valence: number; arousal: number; stress: number },
    desired: DesiredState
  ): number {
    const valenceDist = Math.abs(profile.valence - desired.targetValence);
    const arousalDist = Math.abs(profile.arousal - desired.targetArousal);
    const stressDist = Math.abs(profile.stress - desired.targetStress);

    const totalDist = Math.sqrt(valenceDist ** 2 + arousalDist ** 2 + stressDist ** 2);
    const maxDist = Math.sqrt(2 ** 2 + 2 ** 2 + 1 ** 2); // Max possible distance

    return 1 - (totalDist / maxDist);
  }

  /**
   * Get Q-value for state-action pair
   */
  private getQValue(state: EmotionalState, contentId: string): number {
    const stateHash = this.hashState(state);
    const key = `${stateHash}:${contentId}`;

    return this.qValues.get(key) || 0.3; // Default Q-value
  }

  /**
   * Select content for exploration
   */
  private selectExploration(
    pool: Array<{ content: MockContent; similarityScore: number; qValue: number }>,
    index: number
  ): typeof pool[0] {
    // Random selection for exploration
    const randomIndex = Math.floor(Math.random() * pool.length);
    return pool[randomIndex];
  }

  /**
   * Select content for exploitation
   */
  private selectExploitation(
    pool: Array<{ content: MockContent; similarityScore: number; qValue: number }>,
    index: number
  ): typeof pool[0] {
    // Select highest combined score
    return pool.reduce((best, current) => {
      const currentScore = 0.6 * current.qValue + 0.4 * current.similarityScore;
      const bestScore = 0.6 * best.qValue + 0.4 * best.similarityScore;
      return currentScore > bestScore ? current : best;
    });
  }

  /**
   * Generate reasoning for recommendation
   */
  private generateReasoning(content: MockContent, isExploration: boolean): string {
    if (isExploration) {
      return `Exploring new content to discover preferences`;
    }

    const reasons = [
      `High Q-value based on past experiences`,
      `Strong emotional profile match`,
      `Genre preferences align with your mood`,
      `Optimal for your target emotional state`
    ];

    return reasons[Math.floor(Math.random() * reasons.length)];
  }

  /**
   * Hash emotional state for Q-table lookup
   */
  private hashState(state: EmotionalState): string {
    const v = Math.round(state.valence * 10) / 10;
    const a = Math.round(state.arousal * 10) / 10;
    const s = Math.round(state.stressLevel * 10) / 10;

    return `v${v.toFixed(1)}:a${a.toFixed(1)}:s${s.toFixed(1)}`;
  }

  /**
   * Initialize Q-values with some pre-trained values
   */
  private initializeQValues(): Map<string, number> {
    const qValues = new Map<string, number>();

    // Add some realistic Q-values
    this.mockCatalog.forEach(content => {
      // Higher Q-values for calming content when stressed
      if (content.emotionalProfile.stress < 0.3) {
        qValues.set(`v-0.5:a0.2:s0.8:${content.id}`, 0.75);
      }

      // Higher Q-values for uplifting content when sad
      if (content.emotionalProfile.valence > 0.5) {
        qValues.set(`v-0.6:a-0.2:s0.5:${content.id}`, 0.70);
      }

      // Default moderate Q-values
      qValues.set(`v0.0:a0.0:s0.3:${content.id}`, 0.50);
    });

    return qValues;
  }

  /**
   * Create mock content catalog
   */
  private createMockCatalog(): MockContent[] {
    return [
      {
        id: 'calm-nature-1',
        title: 'Peaceful Mountain Meditation',
        genre: ['documentary', 'nature'],
        emotionalProfile: { valence: 0.6, arousal: -0.7, stress: 0.1 }
      },
      {
        id: 'comedy-uplift-1',
        title: 'Laughter Therapy: Stand-Up Special',
        genre: ['comedy'],
        emotionalProfile: { valence: 0.8, arousal: 0.2, stress: 0.1 }
      },
      {
        id: 'drama-emotional-1',
        title: 'The Art of Resilience',
        genre: ['drama', 'inspirational'],
        emotionalProfile: { valence: 0.5, arousal: -0.3, stress: 0.3 }
      },
      {
        id: 'action-exciting-1',
        title: 'Adrenaline Rush: Extreme Sports',
        genre: ['action', 'documentary'],
        emotionalProfile: { valence: 0.7, arousal: 0.8, stress: 0.4 }
      },
      {
        id: 'relaxation-1',
        title: 'Ocean Waves & Sunset',
        genre: ['relaxation', 'nature'],
        emotionalProfile: { valence: 0.6, arousal: -0.8, stress: 0.05 }
      },
      {
        id: 'music-therapy-1',
        title: 'Classical Music for Stress Relief',
        genre: ['music', 'therapy'],
        emotionalProfile: { valence: 0.5, arousal: -0.6, stress: 0.15 }
      },
      {
        id: 'inspiring-stories-1',
        title: 'Stories of Hope and Triumph',
        genre: ['documentary', 'inspirational'],
        emotionalProfile: { valence: 0.7, arousal: -0.2, stress: 0.2 }
      },
      {
        id: 'gentle-comedy-1',
        title: 'Heartwarming Family Sitcom',
        genre: ['comedy', 'family'],
        emotionalProfile: { valence: 0.6, arousal: -0.1, stress: 0.15 }
      },
      {
        id: 'mindfulness-1',
        title: 'Guided Mindfulness Journey',
        genre: ['wellness', 'meditation'],
        emotionalProfile: { valence: 0.5, arousal: -0.7, stress: 0.1 }
      },
      {
        id: 'adventure-light-1',
        title: 'Beautiful Earth: Travel Documentary',
        genre: ['travel', 'documentary'],
        emotionalProfile: { valence: 0.6, arousal: 0.1, stress: 0.2 }
      }
    ];
  }
}
