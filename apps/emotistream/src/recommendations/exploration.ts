/**
 * ExplorationStrategy - Inject diverse content using epsilon-greedy
 */

import { RankedContent } from './types';

export class ExplorationStrategy {
  private explorationRate: number;
  private readonly decayFactor: number = 0.95;
  private readonly minRate: number = 0.1;

  constructor(initialRate: number = 0.3) {
    this.explorationRate = initialRate;
  }

  /**
   * Inject exploration picks into ranked list
   */
  inject(ranked: RankedContent[], rate?: number): RankedContent[] {
    const effectiveRate = rate ?? this.explorationRate;
    const explorationCount = Math.floor(ranked.length * effectiveRate);
    const result = [...ranked];

    // Mark random items as exploration and boost their scores
    let injected = 0;
    const attempts = ranked.length * 2; // Prevent infinite loop

    for (let i = 0; i < attempts && injected < explorationCount; i++) {
      if (Math.random() < effectiveRate) {
        // Pick from lower-ranked items (bottom 50%)
        const midpoint = Math.floor(ranked.length / 2);
        const explorationIdx = this.randomInt(midpoint, ranked.length - 1);

        if (!result[explorationIdx].isExploration) {
          result[explorationIdx].isExploration = true;
          // Boost score to surface it
          result[explorationIdx].combinedScore += 0.2;
          injected++;
        }
      }
    }

    // Re-sort after exploration boosts
    result.sort((a, b) => b.combinedScore - a.combinedScore);

    return result;
  }

  /**
   * Decay exploration rate over time
   */
  decay(): void {
    this.explorationRate = Math.max(
      this.minRate,
      this.explorationRate * this.decayFactor
    );
  }

  /**
   * Get current exploration rate
   */
  getRate(): number {
    return this.explorationRate;
  }

  /**
   * Reset exploration rate
   */
  reset(rate: number = 0.3): void {
    this.explorationRate = rate;
  }

  private randomInt(min: number, max: number): number {
    return Math.floor(Math.random() * (max - min + 1)) + min;
  }
}
