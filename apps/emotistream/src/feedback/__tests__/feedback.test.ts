/**
 * Feedback Module Tests
 * EmotiStream MVP
 */

import { FeedbackProcessor } from '../processor';
import { RewardCalculator } from '../reward-calculator';
import { ExperienceStore } from '../experience-store';
import { UserProfileManager } from '../user-profile';
import type { EmotionalState, DesiredState } from '../../emotion/types';
import type { FeedbackRequest, EmotionalExperience } from '../types';

describe('FeedbackProcessor', () => {
  let processor: FeedbackProcessor;

  beforeEach(() => {
    processor = new FeedbackProcessor();
  });

  afterEach(() => {
    processor.clearAll();
  });

  test('should process feedback and calculate positive reward for aligned movement', () => {
    // User is sad and wants to feel better
    const stateBefore: EmotionalState = {
      valence: -0.6,
      arousal: 0.2,
      stressLevel: 0.7,
      primaryEmotion: 'sadness',
      emotionVector: new Float32Array([0.1, 0.1, 0.1, 0.1, 0.5, 0.1, 0.05, 0.05]),
      confidence: 0.8,
      timestamp: Date.now() - 1800000, // 30 min ago
    };

    const desiredState: DesiredState = {
      targetValence: 0.5,
      targetArousal: -0.2,
      targetStress: 0.3,
      intensity: 'moderate',
      reasoning: 'User wants to feel calm and positive',
    };

    // After watching content, user feels better (moved toward desired state)
    const actualPostState: EmotionalState = {
      valence: 0.3,
      arousal: -0.1,
      stressLevel: 0.4,
      primaryEmotion: 'joy',
      emotionVector: new Float32Array([0.6, 0.1, 0.05, 0.05, 0.1, 0.05, 0.05, 0.1]),
      confidence: 0.8,
      timestamp: Date.now(),
    };

    const request: FeedbackRequest = {
      userId: 'user-001',
      contentId: 'content-123',
      actualPostState,
      watchDuration: 30,
      completed: true,
      explicitRating: 5,
    };

    const response = processor.process(request, stateBefore, desiredState);

    // Verify positive reward
    expect(response.reward).toBeGreaterThan(0.3);
    expect(response.reward).toBeLessThanOrEqual(1.0);
    expect(response.policyUpdated).toBe(true);
    expect(response.learningProgress.totalExperiences).toBe(1);
  });

  test('should penalize reward for misaligned movement', () => {
    // User is stressed and wants to relax
    const stateBefore: EmotionalState = {
      valence: -0.4,
      arousal: 0.6,
      stressLevel: 0.8,
      primaryEmotion: 'fear',
      emotionVector: new Float32Array([0.05, 0.05, 0.5, 0.1, 0.1, 0.1, 0.1, 0.1]),
      confidence: 0.7,
      timestamp: Date.now() - 1800000,
    };

    const desiredState: DesiredState = {
      targetValence: 0.3,
      targetArousal: -0.5,
      targetStress: 0.2,
      intensity: 'significant',
      reasoning: 'User wants to relax and feel positive',
    };

    // After watching, user became more stressed (wrong direction)
    const actualPostState: EmotionalState = {
      valence: -0.5,
      arousal: 0.8,
      stressLevel: 0.9,
      primaryEmotion: 'anger',
      emotionVector: new Float32Array([0.05, 0.05, 0.2, 0.05, 0.1, 0.1, 0.5, 0.05]),
      confidence: 0.8,
      timestamp: Date.now(),
    };

    const request: FeedbackRequest = {
      userId: 'user-002',
      contentId: 'content-456',
      actualPostState,
      watchDuration: 15,
      completed: false, // Abandoned early
    };

    const response = processor.process(request, stateBefore, desiredState);

    // Verify negative or low reward
    expect(response.reward).toBeLessThan(0);
    expect(response.policyUpdated).toBe(true);
  });

  test('should track learning progress over multiple experiences', () => {
    const userId = 'user-003';
    const stateBefore: EmotionalState = {
      valence: 0,
      arousal: 0,
      stressLevel: 0.5,
      primaryEmotion: 'surprise',
      emotionVector: new Float32Array(8).fill(0.125),
      confidence: 0.7,
      timestamp: Date.now(),
    };

    const desiredState: DesiredState = {
      targetValence: 0.5,
      targetArousal: -0.3,
      targetStress: 0.2,
      intensity: 'moderate',
      reasoning: 'Relax',
    };

    // Submit multiple feedback entries
    for (let i = 0; i < 10; i++) {
      const actualPostState: EmotionalState = {
        valence: 0.4 + i * 0.01,
        arousal: -0.2,
        stressLevel: 0.3,
        primaryEmotion: 'joy',
        emotionVector: new Float32Array(8).fill(0.125),
        confidence: 0.8,
        timestamp: Date.now(),
      };

      const request: FeedbackRequest = {
        userId,
        contentId: `content-${i}`,
        actualPostState,
        watchDuration: 30,
        completed: true,
      };

      processor.process(request, stateBefore, desiredState);
    }

    const progress = processor.getLearningProgress(userId);
    expect(progress.totalExperiences).toBe(10);
    expect(progress.avgReward).toBeGreaterThan(0);
    expect(progress.explorationRate).toBeLessThan(0.3); // Should decay
    expect(progress.convergenceScore).toBeGreaterThan(0);
  });
});

describe('RewardCalculator', () => {
  let calculator: RewardCalculator;

  beforeEach(() => {
    calculator = new RewardCalculator();
  });

  test('should calculate high reward for aligned movement toward desired state', () => {
    const stateBefore: EmotionalState = {
      valence: -0.6,
      arousal: 0.3,
      stressLevel: 0.7,
      primaryEmotion: 'sadness',
      emotionVector: new Float32Array(8),
      confidence: 0.8,
      timestamp: Date.now(),
    };

    const stateAfter: EmotionalState = {
      valence: 0.2,
      arousal: -0.1,
      stressLevel: 0.3,
      primaryEmotion: 'joy',
      emotionVector: new Float32Array(8),
      confidence: 0.8,
      timestamp: Date.now(),
    };

    const desiredState: DesiredState = {
      targetValence: 0.5,
      targetArousal: -0.2,
      targetStress: 0.2,
      intensity: 'moderate',
      reasoning: 'Move to calm and positive',
    };

    const reward = calculator.calculate(stateBefore, stateAfter, desiredState);

    expect(reward).toBeGreaterThan(0.5);
    expect(reward).toBeLessThanOrEqual(1.0);
  });

  test('should calculate proximity bonus when close to target', () => {
    const stateBefore: EmotionalState = {
      valence: 0.3,
      arousal: -0.1,
      stressLevel: 0.3,
      primaryEmotion: 'joy',
      emotionVector: new Float32Array(8),
      confidence: 0.8,
      timestamp: Date.now(),
    };

    const stateAfter: EmotionalState = {
      valence: 0.48,
      arousal: -0.18,
      stressLevel: 0.22,
      primaryEmotion: 'joy',
      emotionVector: new Float32Array(8),
      confidence: 0.9,
      timestamp: Date.now(),
    };

    const desiredState: DesiredState = {
      targetValence: 0.5,
      targetArousal: -0.2,
      targetStress: 0.2,
      intensity: 'subtle',
      reasoning: 'Fine-tune to perfect state',
    };

    const components = calculator.calculateComponents(
      stateBefore,
      stateAfter,
      desiredState
    );

    // Should get proximity bonus since very close to target
    expect(components.proximityBonus).toBeGreaterThan(0);
    expect(components.totalReward).toBeGreaterThan(0.6);
  });

  test('should calculate negative reward for opposite direction', () => {
    const stateBefore: EmotionalState = {
      valence: 0.5,
      arousal: -0.3,
      stressLevel: 0.2,
      primaryEmotion: 'joy',
      emotionVector: new Float32Array(8),
      confidence: 0.8,
      timestamp: Date.now(),
    };

    const stateAfter: EmotionalState = {
      valence: -0.4,
      arousal: 0.6,
      stressLevel: 0.8,
      primaryEmotion: 'anger',
      emotionVector: new Float32Array(8),
      confidence: 0.8,
      timestamp: Date.now(),
    };

    const desiredState: DesiredState = {
      targetValence: 0.7,
      targetArousal: -0.5,
      targetStress: 0.1,
      intensity: 'subtle',
      reasoning: 'Maintain calm state',
    };

    const reward = calculator.calculate(stateBefore, stateAfter, desiredState);

    expect(reward).toBeLessThan(0);
  });

  test('should apply completion penalty correctly', () => {
    // Early abandonment
    const earlyPenalty = calculator.calculateCompletionPenalty(false, 5, 30);
    expect(earlyPenalty).toBe(-0.2);

    // Mid abandonment
    const midPenalty = calculator.calculateCompletionPenalty(false, 12, 30);
    expect(midPenalty).toBe(-0.1);

    // Late abandonment
    const latePenalty = calculator.calculateCompletionPenalty(false, 25, 30);
    expect(latePenalty).toBe(-0.05);

    // Completed
    const noPenalty = calculator.calculateCompletionPenalty(true, 30, 30);
    expect(noPenalty).toBe(0);
  });
});

describe('ExperienceStore', () => {
  let store: ExperienceStore;

  beforeEach(() => {
    store = new ExperienceStore();
  });

  test('should store and retrieve experiences', () => {
    const experience: EmotionalExperience = {
      userId: 'user-001',
      timestamp: Date.now(),
      stateBefore: {
        valence: -0.5,
        arousal: 0.3,
        stressLevel: 0.6,
        primaryEmotion: 'sadness',
        emotionVector: new Float32Array(8),
        confidence: 0.8,
        timestamp: Date.now(),
      },
      action: 'content-123',
      stateAfter: {
        valence: 0.3,
        arousal: -0.2,
        stressLevel: 0.3,
        primaryEmotion: 'joy',
        emotionVector: new Float32Array(8),
        confidence: 0.8,
        timestamp: Date.now(),
      },
      reward: 0.75,
      desiredState: {
        targetValence: 0.5,
        targetArousal: -0.3,
        targetStress: 0.2,
        intensity: 'moderate',
        reasoning: 'Test',
      },
    };

    store.store(experience);

    const retrieved = store.getRecent('user-001', 1);
    expect(retrieved).toHaveLength(1);
    expect(retrieved[0].reward).toBe(0.75);
  });

  test('should calculate average reward correctly', () => {
    const userId = 'user-002';

    for (let i = 0; i < 5; i++) {
      const experience: EmotionalExperience = {
        userId,
        timestamp: Date.now(),
        stateBefore: {} as EmotionalState,
        action: `content-${i}`,
        stateAfter: {} as EmotionalState,
        reward: i * 0.2, // 0, 0.2, 0.4, 0.6, 0.8
        desiredState: {} as DesiredState,
      };
      store.store(experience);
    }

    const avgReward = store.getAverageReward(userId);
    expect(avgReward).toBeCloseTo(0.4, 2); // Average of 0, 0.2, 0.4, 0.6, 0.8
  });

  test('should enforce maximum experience limit', () => {
    const userId = 'user-003';

    // Add more than max experiences
    for (let i = 0; i < 1100; i++) {
      const experience: EmotionalExperience = {
        userId,
        timestamp: Date.now() + i,
        stateBefore: {} as EmotionalState,
        action: `content-${i}`,
        stateAfter: {} as EmotionalState,
        reward: 0.5,
        desiredState: {} as DesiredState,
      };
      store.store(experience);
    }

    const count = store.getCount(userId);
    expect(count).toBe(1000); // Should cap at 1000
  });
});

describe('UserProfileManager', () => {
  let manager: UserProfileManager;

  beforeEach(() => {
    manager = new UserProfileManager();
  });

  test('should initialize new user with default values', () => {
    const stats = manager.getStats('new-user');

    expect(stats.totalExperiences).toBe(0);
    expect(stats.avgReward).toBe(0);
    expect(stats.explorationRate).toBe(0.3); // Initial 30%
    expect(stats.convergenceScore).toBe(0);
  });

  test('should update stats after feedback', () => {
    const userId = 'user-001';

    manager.update(userId, 0.8);

    const stats = manager.getStats(userId);
    expect(stats.totalExperiences).toBe(1);
    expect(stats.avgReward).toBeGreaterThan(0);
    expect(stats.explorationRate).toBeLessThan(0.3); // Should decay
  });

  test('should decay exploration rate over time', () => {
    const userId = 'user-002';
    const initialRate = manager.getExplorationRate(userId);

    // Simulate 100 experiences
    for (let i = 0; i < 100; i++) {
      manager.update(userId, 0.5);
    }

    const finalRate = manager.getExplorationRate(userId);
    expect(finalRate).toBeLessThan(initialRate);
    expect(finalRate).toBeGreaterThanOrEqual(0.05); // Should not go below min
  });

  test('should calculate convergence score correctly', () => {
    const userId = 'user-003';

    // Add multiple positive experiences
    for (let i = 0; i < 50; i++) {
      manager.update(userId, 0.7);
    }

    const stats = manager.getStats(userId);
    expect(stats.convergenceScore).toBeGreaterThan(0.5);
    expect(stats.convergenceScore).toBeLessThanOrEqual(1.0);
  });
});
