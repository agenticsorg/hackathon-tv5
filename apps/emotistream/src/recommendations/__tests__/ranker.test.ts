/**
 * HybridRanker Unit Tests
 */

import { HybridRanker, SearchCandidate } from '../ranker';
import { QTable } from '../../rl/q-table';
import { EmotionalContentProfile } from '../../content/types';

describe('HybridRanker', () => {
  let ranker: HybridRanker;
  let qTable: QTable;

  beforeEach(() => {
    qTable = new QTable();
    ranker = new HybridRanker(qTable);
  });

  const createMockProfile = (valenceDelta: number, arousalDelta: number): EmotionalContentProfile => ({
    contentId: 'test',
    primaryTone: 'neutral',
    valenceDelta,
    arousalDelta,
    intensity: 0.5,
    complexity: 0.5,
    targetStates: [],
    embeddingId: 'emb_test',
    timestamp: Date.now()
  });

  const createCandidate = (
    id: string,
    similarityScore: number,
    valenceDelta: number,
    arousalDelta: number
  ): SearchCandidate => ({
    contentId: id,
    title: `Content ${id}`,
    profile: { ...createMockProfile(valenceDelta, arousalDelta), contentId: id },
    similarityScore
  });

  describe('rank()', () => {
    it('should rank by hybrid score (70% Q + 30% similarity)', async () => {
      const candidates: SearchCandidate[] = [
        createCandidate('A', 0.9, 0.3, -0.4), // High similarity, low Q
        createCandidate('B', 0.6, 0.5, -0.3), // Medium similarity
        createCandidate('C', 0.7, 0.4, -0.5)  // Good balance
      ];

      // Set Q-values
      await qTable.updateQValue('v:5:a:5:s:2', 'A', 0.2);
      await qTable.updateQValue('v:5:a:5:s:2', 'B', 0.8);
      await qTable.updateQValue('v:5:a:5:s:2', 'C', 0.6);

      const ranked = await ranker.rank(
        candidates,
        { valence: 0.0, arousal: 0.0, stress: 0.4, confidence: 0.8 },
        { valence: 0.5, arousal: -0.3, confidence: 0.9 }
      );

      expect(ranked).toHaveLength(3);
      // B should rank highest due to high Q-value (0.8)
      expect(ranked[0].contentId).toBe('B');
    });

    it('should use default Q-value for unexplored content', async () => {
      const candidates: SearchCandidate[] = [
        createCandidate('unexplored', 0.8, 0.4, -0.3)
      ];

      const ranked = await ranker.rank(
        candidates,
        { valence: 0.0, arousal: 0.0, stress: 0.5, confidence: 0.8 },
        { valence: 0.3, arousal: -0.2, confidence: 0.8 }
      );

      expect(ranked[0].qValue).toBe(0.5); // Default Q-value
    });

    it('should apply outcome alignment boost', async () => {
      const candidates: SearchCandidate[] = [
        createCandidate('aligned', 0.7, 0.6, -0.5),   // Well aligned
        createCandidate('misaligned', 0.7, -0.5, 0.6) // Opposite direction
      ];

      const ranked = await ranker.rank(
        candidates,
        { valence: -0.3, arousal: 0.4, stress: 0.7, confidence: 0.8 },
        { valence: 0.5, arousal: -0.3, confidence: 0.9 }
      );

      // Aligned content should have higher outcome alignment
      expect(ranked[0].outcomeAlignment).toBeGreaterThan(ranked[1].outcomeAlignment);
    });
  });
});
