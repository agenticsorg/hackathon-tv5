/**
 * FeedbackProcessor Usage Example
 * EmotiStream MVP
 *
 * This example demonstrates how to use the FeedbackProcessor module
 * in a complete recommendation feedback loop.
 */

import { FeedbackProcessor } from './processor.js';
import type { FeedbackRequest } from './types.js';
import type { EmotionalState, DesiredState } from '../emotion/types.js';

/**
 * Example 1: Positive Feedback - Content Improved User's Mood
 */
function example1_PositiveFeedback(): void {
  console.log('=== Example 1: Positive Feedback ===\n');

  const processor = new FeedbackProcessor();

  // User is feeling sad and stressed before watching
  const stateBefore: EmotionalState = {
    valence: -0.6, // Negative mood
    arousal: 0.2, // Slightly activated
    stressLevel: 0.7, // High stress
    primaryEmotion: 'sadness',
    emotionVector: new Float32Array([0.1, 0.1, 0.1, 0.1, 0.5, 0.1, 0.05, 0.05]),
    confidence: 0.8,
    timestamp: Date.now() - 1800000, // 30 min ago
  };

  // User wants to feel calm and positive
  const desiredState: DesiredState = {
    targetValence: 0.5, // Positive mood
    targetArousal: -0.2, // Calm
    targetStress: 0.3, // Low stress
    intensity: 'moderate',
    reasoning: 'User wants to relax and feel better',
  };

  // After watching uplifting content, user feels much better
  const actualPostState: EmotionalState = {
    valence: 0.4, // Improved to positive
    arousal: -0.1, // Calmer
    stressLevel: 0.4, // Lower stress
    primaryEmotion: 'joy',
    emotionVector: new Float32Array([0.6, 0.1, 0.05, 0.05, 0.1, 0.05, 0.05, 0.1]),
    confidence: 0.8,
    timestamp: Date.now(),
  };

  const request: FeedbackRequest = {
    userId: 'user-001',
    contentId: 'uplifting-movie-123',
    actualPostState,
    watchDuration: 30,
    completed: true,
    explicitRating: 5,
  };

  const response = processor.process(request, stateBefore, desiredState);

  console.log('Feedback Response:');
  console.log(`  Reward: ${response.reward.toFixed(3)} (expected: 0.6-0.8)`);
  console.log(`  Policy Updated: ${response.policyUpdated}`);
  console.log(`  New Q-Value: ${response.newQValue.toFixed(3)}`);
  console.log('\nLearning Progress:');
  console.log(`  Total Experiences: ${response.learningProgress.totalExperiences}`);
  console.log(`  Avg Reward: ${response.learningProgress.avgReward.toFixed(3)}`);
  console.log(`  Exploration Rate: ${response.learningProgress.explorationRate.toFixed(3)}`);
  console.log(`  Convergence Score: ${response.learningProgress.convergenceScore.toFixed(3)}\n`);
}

/**
 * Example 2: Negative Feedback - Content Made Things Worse
 */
function example2_NegativeFeedback(): void {
  console.log('=== Example 2: Negative Feedback ===\n');

  const processor = new FeedbackProcessor();

  // User is stressed and wants to relax
  const stateBefore: EmotionalState = {
    valence: -0.3,
    arousal: 0.6, // High arousal
    stressLevel: 0.8, // Very stressed
    primaryEmotion: 'fear',
    emotionVector: new Float32Array([0.05, 0.05, 0.5, 0.1, 0.1, 0.1, 0.1, 0.1]),
    confidence: 0.7,
    timestamp: Date.now() - 1800000,
  };

  const desiredState: DesiredState = {
    targetValence: 0.4,
    targetArousal: -0.5, // Very calm
    targetStress: 0.2, // Low stress
    intensity: 'significant',
    reasoning: 'User needs significant stress reduction',
  };

  // After watching intense thriller, user became MORE stressed
  const actualPostState: EmotionalState = {
    valence: -0.5, // Worse mood
    arousal: 0.8, // Even more activated
    stressLevel: 0.9, // Higher stress
    primaryEmotion: 'anger',
    emotionVector: new Float32Array([0.05, 0.05, 0.3, 0.05, 0.1, 0.1, 0.4, 0.05]),
    confidence: 0.8,
    timestamp: Date.now(),
  };

  const request: FeedbackRequest = {
    userId: 'user-002',
    contentId: 'intense-thriller-456',
    actualPostState,
    watchDuration: 20, // Didn't finish
    completed: false,
  };

  const response = processor.process(request, stateBefore, desiredState);

  console.log('Feedback Response:');
  console.log(`  Reward: ${response.reward.toFixed(3)} (expected: -0.5 to -0.3)`);
  console.log(`  Policy Updated: ${response.policyUpdated}`);
  console.log(`  New Q-Value: ${response.newQValue.toFixed(3)}`);
  console.log('\nLearning Progress:');
  console.log(`  Total Experiences: ${response.learningProgress.totalExperiences}`);
  console.log(`  Avg Reward: ${response.learningProgress.avgReward.toFixed(3)}`);
  console.log('  → System will learn to avoid similar content for this user\n');
}

/**
 * Example 3: Learning Progress Over Time
 */
function example3_LearningProgress(): void {
  console.log('=== Example 3: Learning Progress Over Time ===\n');

  const processor = new FeedbackProcessor();
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
    targetValence: 0.6,
    targetArousal: -0.3,
    targetStress: 0.2,
    intensity: 'moderate',
    reasoning: 'User wants calm positivity',
  };

  console.log('Simulating 20 content consumption cycles...\n');

  for (let i = 0; i < 20; i++) {
    // Simulate improving content selection over time
    const improvement = i * 0.02; // Gets better each time

    const actualPostState: EmotionalState = {
      valence: 0.5 + improvement,
      arousal: -0.2 + improvement * 0.5,
      stressLevel: 0.3 - improvement * 0.5,
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

    // Show progress every 5 experiences
    if ((i + 1) % 5 === 0) {
      const progress = processor.getLearningProgress(userId);
      console.log(`After ${i + 1} experiences:`);
      console.log(`  Avg Reward: ${progress.avgReward.toFixed(3)}`);
      console.log(`  Exploration Rate: ${progress.explorationRate.toFixed(3)}`);
      console.log(`  Convergence: ${(progress.convergenceScore * 100).toFixed(1)}%\n`);
    }
  }

  const avgReward = processor.getAverageReward(userId);
  console.log(`Final Average Reward: ${avgReward.toFixed(3)}`);
  console.log('→ System has learned user preferences!\n');
}

/**
 * Example 4: Analyzing Recent Experiences
 */
function example4_ExperienceAnalysis(): void {
  console.log('=== Example 4: Experience Analysis ===\n');

  const processor = new FeedbackProcessor();
  const userId = 'user-004';

  // Add some varied experiences
  const experiences = [
    { contentId: 'comedy-1', reward: 0.8, emotion: 'joy' },
    { contentId: 'drama-1', reward: 0.6, emotion: 'sadness' },
    { contentId: 'action-1', reward: -0.2, emotion: 'fear' },
    { contentId: 'comedy-2', reward: 0.7, emotion: 'joy' },
    { contentId: 'documentary-1', reward: 0.4, emotion: 'surprise' },
  ];

  experiences.forEach((exp, i) => {
    const stateBefore: EmotionalState = {
      valence: 0,
      arousal: 0,
      stressLevel: 0.5,
      primaryEmotion: 'surprise',
      emotionVector: new Float32Array(8).fill(0.125),
      confidence: 0.7,
      timestamp: Date.now() - (5 - i) * 60000,
    };

    const actualPostState: EmotionalState = {
      valence: exp.reward,
      arousal: 0,
      stressLevel: 0.3,
      primaryEmotion: exp.emotion as any,
      emotionVector: new Float32Array(8).fill(0.125),
      confidence: 0.8,
      timestamp: Date.now(),
    };

    const request: FeedbackRequest = {
      userId,
      contentId: exp.contentId,
      actualPostState,
      watchDuration: 30,
      completed: true,
    };

    processor.process(request, stateBefore, {
      targetValence: 0.5,
      targetArousal: -0.2,
      targetStress: 0.3,
      intensity: 'moderate',
      reasoning: 'Default',
    });
  });

  const recentExperiences = processor.getRecentExperiences(userId, 5);

  console.log('Recent Experiences:');
  recentExperiences.forEach((exp, i) => {
    console.log(`  ${i + 1}. Content: ${exp.action}`);
    console.log(`     Reward: ${exp.reward.toFixed(3)}`);
    console.log(`     Valence: ${exp.stateBefore.valence.toFixed(2)} → ${exp.stateAfter.valence.toFixed(2)}`);
  });

  const avgReward = processor.getAverageReward(userId);
  console.log(`\nAverage Reward: ${avgReward.toFixed(3)}`);
  console.log('→ Comedy performs best for this user!\n');
}

/**
 * Run all examples
 */
function runAllExamples(): void {
  console.log('\n');
  console.log('╔═══════════════════════════════════════════════════════════╗');
  console.log('║   EmotiStream FeedbackProcessor - Usage Examples         ║');
  console.log('╚═══════════════════════════════════════════════════════════╝');
  console.log('\n');

  example1_PositiveFeedback();
  example2_NegativeFeedback();
  example3_LearningProgress();
  example4_ExperienceAnalysis();

  console.log('═══════════════════════════════════════════════════════════');
  console.log('All examples completed successfully!');
  console.log('═══════════════════════════════════════════════════════════\n');
}

// Run examples if this file is executed directly
if (require.main === module) {
  runAllExamples();
}

export { runAllExamples };
