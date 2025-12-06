/**
 * ReasoningGenerator - Create human-readable recommendation explanations
 */

import { EmotionalState, DesiredState } from './types';
import { EmotionalContentProfile } from '../content/types';

export class ReasoningGenerator {
  /**
   * Generate human-readable reasoning for recommendation
   */
  generate(
    currentState: EmotionalState,
    desiredState: DesiredState,
    contentProfile: EmotionalContentProfile,
    qValue: number,
    isExploration: boolean
  ): string {
    let reasoning = '';

    // Part 1: Current emotional context
    const currentDesc = this.describeEmotionalState(
      currentState.valence,
      currentState.arousal,
      currentState.stress
    );
    reasoning += `You're currently feeling ${currentDesc}. `;

    // Part 2: Desired transition
    const desiredDesc = this.describeEmotionalState(
      desiredState.valence,
      desiredState.arousal,
      0
    );
    reasoning += `This content will help you transition toward feeling ${desiredDesc}. `;

    // Part 3: Expected emotional changes
    if (contentProfile.valenceDelta > 0.2) {
      reasoning += 'It should improve your mood significantly. ';
    } else if (contentProfile.valenceDelta < -0.2) {
      reasoning += 'It may be emotionally intense. ';
    }

    if (contentProfile.arousalDelta > 0.3) {
      reasoning += 'Expect to feel more energized and alert. ';
    } else if (contentProfile.arousalDelta < -0.3) {
      reasoning += 'It will help you relax and unwind. ';
    }

    // Part 4: Recommendation confidence
    const normalizedQ = (qValue + 1.0) / 2.0;
    if (normalizedQ > 0.7) {
      reasoning += 'This content has worked well for similar emotional states. ';
    } else if (normalizedQ < 0.4) {
      reasoning += 'This is a personalized experimental pick. ';
    } else {
      reasoning += 'This matches your emotional needs well. ';
    }

    // Part 5: Exploration flag
    if (isExploration) {
      reasoning += '(New discovery for you!)';
    }

    return reasoning.trim();
  }

  /**
   * Describe emotional state in human terms
   */
  private describeEmotionalState(
    valence: number,
    arousal: number,
    stress: number
  ): string {
    let emotion = '';

    // Map to emotional quadrants
    if (valence > 0.3 && arousal > 0.3) {
      emotion = 'excited and happy';
    } else if (valence > 0.3 && arousal < -0.3) {
      emotion = 'calm and content';
    } else if (valence < -0.3 && arousal > 0.3) {
      emotion = 'stressed and anxious';
    } else if (valence < -0.3 && arousal < -0.3) {
      emotion = 'sad and lethargic';
    } else if (arousal > 0.5) {
      emotion = 'energized and alert';
    } else if (arousal < -0.5) {
      emotion = 'relaxed and calm';
    } else if (valence > 0.3) {
      emotion = 'positive and balanced';
    } else if (valence < -0.3) {
      emotion = 'down and subdued';
    } else {
      emotion = 'neutral and balanced';
    }

    // Stress modifier
    if (stress > 0.7) {
      emotion = `highly stressed, ${emotion}`;
    } else if (stress > 0.4) {
      emotion = `moderately stressed, ${emotion}`;
    }

    return emotion;
  }
}
