/**
 * HybridRanker - Combine Q-values and similarity scores
 */

import { QTable } from '../rl/q-table';
import { StateHasher } from './state-hasher';
import {
  EmotionalState,
  DesiredState,
  RankedContent,
  HybridRankingConfig
} from './types';
import { EmotionalContentProfile } from '../content/types';

export interface SearchCandidate {
  contentId: string;
  title: string;
  profile: EmotionalContentProfile;
  similarityScore: number;
}

export class HybridRanker {
  private stateHasher: StateHasher;
  private config: HybridRankingConfig;

  constructor(
    private qTable: QTable,
    config?: Partial<HybridRankingConfig>
  ) {
    this.stateHasher = new StateHasher();
    this.config = {
      qWeight: config?.qWeight ?? 0.7,
      similarityWeight: config?.similarityWeight ?? 0.3,
      defaultQValue: config?.defaultQValue ?? 0.5,
      explorationBonus: config?.explorationBonus ?? 0.1
    };
  }

  /**
   * Rank candidates using hybrid Q-value + similarity scoring
   */
  async rank(
    candidates: SearchCandidate[],
    currentState: EmotionalState,
    desiredState: DesiredState
  ): Promise<RankedContent[]> {
    const stateHash = this.stateHasher.hash(currentState);
    const ranked: RankedContent[] = [];

    for (const candidate of candidates) {
      // Get Q-value from table
      const qEntry = await this.qTable.get(stateHash.hash, candidate.contentId);
      const qValue = qEntry?.qValue ?? this.config.defaultQValue;

      // Normalize Q-value to [0, 1]
      const normalizedQ = (qValue + 1.0) / 2.0;

      // Calculate outcome alignment
      const alignment = this.calculateOutcomeAlignment(
        candidate.profile,
        currentState,
        desiredState
      );

      // Hybrid score: 70% Q-value + 30% similarity, multiplied by alignment
      const combinedScore =
        (normalizedQ * this.config.qWeight +
          candidate.similarityScore * this.config.similarityWeight) *
        alignment;

      ranked.push({
        contentId: candidate.contentId,
        title: candidate.title,
        profile: candidate.profile,
        qValue,
        similarityScore: candidate.similarityScore,
        combinedScore,
        outcomeAlignment: alignment,
        isExploration: false // Will be set by exploration strategy
      });
    }

    // Sort by combined score descending
    ranked.sort((a, b) => b.combinedScore - a.combinedScore);

    return ranked;
  }

  /**
   * Calculate how well content's delta aligns with desired transition
   */
  private calculateOutcomeAlignment(
    profile: EmotionalContentProfile,
    currentState: EmotionalState,
    desiredState: DesiredState
  ): number {
    // Desired deltas
    const desiredValenceDelta = desiredState.valence - currentState.valence;
    const desiredArousalDelta = desiredState.arousal - currentState.arousal;

    // Content's deltas
    const contentValenceDelta = profile.valenceDelta;
    const contentArousalDelta = profile.arousalDelta;

    // Cosine similarity of 2D delta vectors
    const dotProduct =
      contentValenceDelta * desiredValenceDelta +
      contentArousalDelta * desiredArousalDelta;

    const magnitudeContent = Math.sqrt(
      contentValenceDelta ** 2 + contentArousalDelta ** 2
    );

    const magnitudeDesired = Math.sqrt(
      desiredValenceDelta ** 2 + desiredArousalDelta ** 2
    );

    if (magnitudeContent === 0 || magnitudeDesired === 0) {
      return 0.5; // Neutral alignment
    }

    // Cosine similarity in [-1, 1]
    const cosineSim = dotProduct / (magnitudeContent * magnitudeDesired);

    // Convert to [0, 1] with 0.5 as neutral
    let alignmentScore = (cosineSim + 1.0) / 2.0;

    // Boost for strong alignment
    if (alignmentScore > 0.8) {
      alignmentScore = 1.0 + (alignmentScore - 0.8) * 0.5; // Up to 1.1x boost
    }

    return Math.min(1.1, alignmentScore);
  }
}
