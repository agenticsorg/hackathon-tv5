/**
 * State Management Unit Tests (TDD - Red Phase)
 */

import { StateHasher } from '../../../src/emotion/state-hasher';
import { DesiredStatePredictor } from '../../../src/emotion/desired-state';
import { EmotionalState, PlutchikEmotion } from '../../../src/emotion/types';

describe('StateHasher', () => {
  let hasher: StateHasher;

  beforeEach(() => {
    hasher = new StateHasher();
  });

  it('should hash emotional state to discrete string', () => {
    // Arrange
    const state: EmotionalState = {
      emotionalStateId: 'test',
      userId: 'user_123',
      valence: 0.5,
      arousal: 0.3,
      primaryEmotion: 'joy',
      emotionVector: new Float32Array(8),
      stressLevel: 0.6,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'test',
    };

    // Act
    const hash = hasher.hash(state);

    // Assert
    expect(hash).toMatch(/^\d+:\d+:\d+$/); // Format: "v:a:s"
  });

  it('should use 5×5×3 buckets (valence×arousal×stress)', () => {
    // Arrange
    const state: EmotionalState = {
      emotionalStateId: 'test',
      userId: 'user_123',
      valence: 0.0,
      arousal: 0.0,
      primaryEmotion: 'joy',
      emotionVector: new Float32Array(8),
      stressLevel: 0.5,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'test',
    };

    // Act
    const hash = hasher.hash(state);
    const parts = hash.split(':').map(Number);

    // Assert
    expect(parts[0]).toBeGreaterThanOrEqual(0);
    expect(parts[0]).toBeLessThan(5); // Valence buckets
    expect(parts[1]).toBeGreaterThanOrEqual(0);
    expect(parts[1]).toBeLessThan(5); // Arousal buckets
    expect(parts[2]).toBeGreaterThanOrEqual(0);
    expect(parts[2]).toBeLessThan(3); // Stress buckets
  });

  it('should produce same hash for similar states', () => {
    // Arrange
    const state1: EmotionalState = {
      emotionalStateId: 'test1',
      userId: 'user_123',
      valence: 0.5,
      arousal: 0.3,
      primaryEmotion: 'joy',
      emotionVector: new Float32Array(8),
      stressLevel: 0.6,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'test',
    };

    const state2: EmotionalState = {
      ...state1,
      emotionalStateId: 'test2',
      valence: 0.52, // Slightly different, same bucket
      arousal: 0.31,
    };

    // Act
    const hash1 = hasher.hash(state1);
    const hash2 = hasher.hash(state2);

    // Assert
    expect(hash1).toBe(hash2);
  });

  it('should produce different hash for different buckets', () => {
    // Arrange
    const state1: EmotionalState = {
      emotionalStateId: 'test1',
      userId: 'user_123',
      valence: -0.5,
      arousal: 0.3,
      primaryEmotion: 'joy',
      emotionVector: new Float32Array(8),
      stressLevel: 0.6,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'test',
    };

    const state2: EmotionalState = {
      ...state1,
      emotionalStateId: 'test2',
      valence: 0.5, // Different bucket
    };

    // Act
    const hash1 = hasher.hash(state1);
    const hash2 = hasher.hash(state2);

    // Assert
    expect(hash1).not.toBe(hash2);
  });
});

describe('DesiredStatePredictor', () => {
  let predictor: DesiredStatePredictor;

  beforeEach(() => {
    predictor = new DesiredStatePredictor();
  });

  it('should predict calming state for high stress', () => {
    // Arrange
    const currentState: EmotionalState = {
      emotionalStateId: 'test',
      userId: 'user_123',
      valence: -0.6,
      arousal: 0.7,
      primaryEmotion: 'fear',
      emotionVector: new Float32Array(8),
      stressLevel: 0.85,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'test',
    };

    // Act
    const desired = predictor.predict(currentState);

    // Assert
    expect(desired.arousal).toBeLessThan(0.0); // Want calm
    expect(desired.valence).toBeGreaterThan(0.0); // Want positive
    expect(desired.reasoning).toContain('stress');
  });

  it('should predict uplifting state for low mood', () => {
    // Arrange
    const currentState: EmotionalState = {
      emotionalStateId: 'test',
      userId: 'user_123',
      valence: -0.7,
      arousal: -0.3,
      primaryEmotion: 'sadness',
      emotionVector: new Float32Array(8),
      stressLevel: 0.5,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'test',
    };

    // Act
    const desired = predictor.predict(currentState);

    // Assert
    expect(desired.valence).toBeGreaterThan(0.5); // Want positive
    expect(desired.arousal).toBeGreaterThan(0.0); // Want energizing
  });

  it('should predict calming state for anxious (high arousal + negative)', () => {
    // Arrange
    const currentState: EmotionalState = {
      emotionalStateId: 'test',
      userId: 'user_123',
      valence: -0.4,
      arousal: 0.8,
      primaryEmotion: 'fear',
      emotionVector: new Float32Array(8),
      stressLevel: 0.7,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'test',
    };

    // Act
    const desired = predictor.predict(currentState);

    // Assert
    expect(desired.arousal).toBeLessThan(currentState.arousal); // Want lower arousal
  });

  it('should predict energizing state for low energy (low arousal)', () => {
    // Arrange
    const currentState: EmotionalState = {
      emotionalStateId: 'test',
      userId: 'user_123',
      valence: 0.2,
      arousal: -0.7,
      primaryEmotion: 'trust',
      emotionVector: new Float32Array(8),
      stressLevel: 0.3,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'test',
    };

    // Act
    const desired = predictor.predict(currentState);

    // Assert
    expect(desired.arousal).toBeGreaterThan(currentState.arousal); // Want higher arousal
  });

  it('should return default desired state for neutral mood', () => {
    // Arrange
    const currentState: EmotionalState = {
      emotionalStateId: 'test',
      userId: 'user_123',
      valence: 0.1,
      arousal: 0.1,
      primaryEmotion: 'trust',
      emotionVector: new Float32Array(8),
      stressLevel: 0.3,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'test',
    };

    // Act
    const desired = predictor.predict(currentState);

    // Assert
    expect(desired.valence).toBeGreaterThan(0.5);
    expect(desired.arousal).toBeGreaterThan(-0.2);
    expect(desired.confidence).toBeGreaterThan(0.0);
  });

  it('should include reasoning for prediction', () => {
    // Arrange
    const currentState: EmotionalState = {
      emotionalStateId: 'test',
      userId: 'user_123',
      valence: -0.6,
      arousal: 0.7,
      primaryEmotion: 'fear',
      emotionVector: new Float32Array(8),
      stressLevel: 0.85,
      confidence: 0.9,
      timestamp: Date.now(),
      rawText: 'test',
    };

    // Act
    const desired = predictor.predict(currentState);

    // Assert
    expect(desired.reasoning).toBeDefined();
    expect(desired.reasoning.length).toBeGreaterThan(0);
  });
});
