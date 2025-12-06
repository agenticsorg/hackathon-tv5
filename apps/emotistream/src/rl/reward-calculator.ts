import { EmotionalState, DesiredState } from './types';

export class RewardCalculator {
  private readonly directionWeight = 0.6;
  private readonly magnitudeWeight = 0.4;
  private readonly proximityThreshold = 0.15;
  private readonly proximityBonusValue = 0.2;

  calculate(before: EmotionalState, after: EmotionalState, desired: DesiredState): number {
    const directionScore = this.directionAlignment(before, after, desired);
    const magnitudeScore = this.magnitude(before, after, desired);
    const baseReward = this.directionWeight * directionScore +
                       this.magnitudeWeight * magnitudeScore;
    const bonus = this.calculateProximityBonus(after, desired);
    const reward = baseReward + bonus;
    return Math.max(-1.0, Math.min(1.0, reward));
  }

  private directionAlignment(before: EmotionalState, after: EmotionalState, desired: DesiredState): number {
    const actualDelta = {
      valence: after.valence - before.valence,
      arousal: after.arousal - before.arousal
    };

    const desiredDelta = {
      valence: desired.valence - before.valence,
      arousal: desired.arousal - before.arousal
    };

    const dotProduct = actualDelta.valence * desiredDelta.valence +
                      actualDelta.arousal * desiredDelta.arousal;

    const actualMagnitude = Math.sqrt(actualDelta.valence ** 2 + actualDelta.arousal ** 2);
    const desiredMagnitude = Math.sqrt(desiredDelta.valence ** 2 + desiredDelta.arousal ** 2);

    if (actualMagnitude === 0 || desiredMagnitude === 0) {
      return 0.0;
    }

    const cosineSimilarity = dotProduct / (actualMagnitude * desiredMagnitude);
    return (cosineSimilarity + 1.0) / 2.0;
  }

  private magnitude(before: EmotionalState, after: EmotionalState, desired: DesiredState): number {
    const actualDelta = {
      valence: after.valence - before.valence,
      arousal: after.arousal - before.arousal
    };

    const desiredDelta = {
      valence: desired.valence - before.valence,
      arousal: desired.arousal - before.arousal
    };

    const actualMagnitude = Math.sqrt(actualDelta.valence ** 2 + actualDelta.arousal ** 2);
    const desiredMagnitude = Math.sqrt(desiredDelta.valence ** 2 + desiredDelta.arousal ** 2);

    if (desiredMagnitude === 0) {
      return 1.0;
    }

    return Math.min(actualMagnitude / desiredMagnitude, 1.0);
  }

  private calculateProximityBonus(after: EmotionalState, desired: DesiredState): number {
    const distance = Math.sqrt(
      (after.valence - desired.valence) ** 2 +
      (after.arousal - desired.arousal) ** 2
    );

    return distance < this.proximityThreshold ? this.proximityBonusValue : 0.0;
  }
}
