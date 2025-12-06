/**
 * RecommendationEngine - Main orchestrator for content recommendations
 * Combines RL policy (Q-values) with semantic vector search
 */

import { ContentProfiler } from '../content/profiler.js';
import { QTable } from '../rl/q-table.js';
import { HybridRanker, SearchCandidate } from './ranker.js';
import { OutcomePredictor } from './outcome-predictor.js';
import { ReasoningGenerator } from './reasoning.js';
import { ExplorationStrategy } from './exploration.js';
import {
  RecommendationRequest,
  Recommendation,
  DesiredState
} from './types.js';
import { EmotionalContentProfile } from '../content/types.js';

export class RecommendationEngine {
  private profiler: ContentProfiler;
  private qTable: QTable;
  private ranker: HybridRanker;
  private outcomePredictor: OutcomePredictor;
  private reasoningGenerator: ReasoningGenerator;
  private explorationStrategy: ExplorationStrategy;
  private initialized: boolean = false;

  constructor() {
    this.profiler = new ContentProfiler();
    this.qTable = new QTable();
    this.ranker = new HybridRanker(this.qTable);
    this.outcomePredictor = new OutcomePredictor();
    this.reasoningGenerator = new ReasoningGenerator();
    this.explorationStrategy = new ExplorationStrategy();
  }

  /**
   * Initialize the recommendation engine with content
   */
  async initialize(contentCount: number = 100): Promise<void> {
    if (this.initialized) return;

    console.log('Initializing RecommendationEngine...');
    await this.profiler.initialize(contentCount);
    this.initialized = true;
    console.log('RecommendationEngine ready');
  }

  /**
   * Check if using real TMDB data
   */
  isUsingTMDB(): boolean {
    return this.profiler.isUsingTMDB();
  }

  /**
   * Get personalized recommendations
   */
  async recommend(
    userId: string,
    currentState: { valence: number; arousal: number; stress: number },
    limit: number = 20
  ): Promise<Recommendation[]> {
    // Build request
    const request: RecommendationRequest = {
      userId,
      currentState: {
        valence: currentState.valence,
        arousal: currentState.arousal,
        stress: currentState.stress,
        confidence: 0.8
      },
      limit,
      includeExploration: true,
      explorationRate: 0.15
    };

    return this.getRecommendations(request);
  }

  /**
   * Get recommendations from full request
   */
  async getRecommendations(
    request: RecommendationRequest
  ): Promise<Recommendation[]> {
    // Step 1: Determine desired state (homeostasis by default)
    const desiredState = request.desiredState ?? this.predictDesiredState(request.currentState);

    // Step 2: Build transition vector
    const transitionVector = this.buildTransitionVector(
      request.currentState,
      desiredState
    );

    // Step 3: Search for semantically similar content
    const searchLimit = (request.limit ?? 20) * 3; // Get 3x for re-ranking
    const searchResults = await this.profiler.search(transitionVector, searchLimit);

    // Step 4: Convert to candidates
    const candidates: SearchCandidate[] = searchResults.map(result => ({
      contentId: result.contentId,
      title: result.title,
      profile: result.profile,
      similarityScore: result.similarityScore
    }));

    if (candidates.length === 0) {
      return [];
    }

    // Step 5: Hybrid ranking (Q-values + similarity)
    const ranked = await this.ranker.rank(
      candidates,
      request.currentState,
      desiredState
    );

    // Step 6: Apply exploration
    const explored = request.includeExploration
      ? this.explorationStrategy.inject(ranked, request.explorationRate)
      : ranked;

    // Step 7: Generate final recommendations
    const finalLimit = request.limit ?? 20;
    const topRanked = explored.slice(0, finalLimit);

    const recommendations = topRanked.map((ranked, idx) => {
      const outcome = this.outcomePredictor.predict(
        request.currentState,
        ranked.profile
      );

      const reasoning = this.reasoningGenerator.generate(
        request.currentState,
        desiredState,
        ranked.profile,
        ranked.qValue,
        ranked.isExploration
      );

      return {
        contentId: ranked.contentId,
        title: ranked.title,
        qValue: ranked.qValue,
        similarityScore: ranked.similarityScore,
        combinedScore: ranked.combinedScore,
        predictedOutcome: outcome,
        reasoning,
        isExploration: ranked.isExploration,
        rank: idx + 1,
        profile: ranked.profile
      };
    });

    return recommendations;
  }

  /**
   * Predict desired emotional state (homeostasis rules)
   */
  private predictDesiredState(currentState: { valence: number; arousal: number; stress: number }): DesiredState {
    // Stress reduction rule
    if (currentState.stress > 0.6) {
      return {
        valence: Math.max(currentState.valence, 0.3),
        arousal: Math.min(currentState.arousal, -0.3),
        confidence: 0.9
      };
    }

    // Sadness lift rule
    if (currentState.valence < -0.4) {
      return {
        valence: Math.max(currentState.valence + 0.4, 0.2),
        arousal: Math.max(currentState.arousal, -0.2),
        confidence: 0.85
      };
    }

    // Anxiety reduction rule
    if (currentState.valence < 0 && currentState.arousal > 0.4) {
      return {
        valence: Math.max(currentState.valence + 0.3, 0.1),
        arousal: Math.max(currentState.arousal - 0.5, -0.3),
        confidence: 0.9
      };
    }

    // Boredom stimulation rule
    if (Math.abs(currentState.valence) < 0.2 && currentState.arousal < -0.3) {
      return {
        valence: Math.max(currentState.valence + 0.2, 0.3),
        arousal: Math.max(currentState.arousal + 0.4, 0.2),
        confidence: 0.7
      };
    }

    // Default: maintain current state (homeostasis)
    return {
      valence: currentState.valence,
      arousal: currentState.arousal,
      confidence: 0.6
    };
  }

  /**
   * Build transition vector from current to desired state
   * In real implementation, this would use embedding model
   */
  private buildTransitionVector(
    currentState: { valence: number; arousal: number; stress: number },
    desiredState: DesiredState
  ): Float32Array {
    // Create a simple transition vector encoding
    // In production, this would be an embedding of a text prompt
    const vector = new Float32Array(1536);

    // Encode current state
    vector[0] = currentState.valence;
    vector[1] = currentState.arousal;
    vector[2] = currentState.stress;

    // Encode desired state
    vector[3] = desiredState.valence;
    vector[4] = desiredState.arousal;

    // Encode deltas
    vector[5] = desiredState.valence - currentState.valence;
    vector[6] = desiredState.arousal - currentState.arousal;

    // Fill rest with random noise (simulating embedding)
    for (let i = 7; i < vector.length; i++) {
      vector[i] = (Math.random() - 0.5) * 0.1;
    }

    return vector;
  }

  /**
   * Get content profiler for external access
   */
  getProfiler(): ContentProfiler {
    return this.profiler;
  }

  /**
   * Get Q-table for external access
   */
  getQTable(): QTable {
    return this.qTable;
  }
}
