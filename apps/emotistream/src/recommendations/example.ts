/**
 * RecommendationEngine Simple Example
 * Demonstrates basic usage without dependencies on other modules
 */

import { RecommendationEngine } from './engine';
import { OutcomePredictor } from './outcome-predictor';
import { ReasoningGenerator } from './reasoning';
import { StateHasher } from './state-hasher';
import { HybridRanker } from './ranker';
import { ExplorationStrategy } from './exploration';

// Example 1: Basic recommendation flow
export function basicExample() {
  console.log('=== Example 1: Basic Recommendation ===\n');

  const engine = new RecommendationEngine();

  // Stressed user needs calming content
  const userId = 'user_001';
  const currentState = {
    valence: -0.4,  // Negative mood
    arousal: 0.6,   // High arousal
    stress: 0.8     // Very stressed
  };

  console.log('Current Emotional State:');
  console.log(`  Valence: ${currentState.valence} (negative mood)`);
  console.log(`  Arousal: ${currentState.arousal} (high energy)`);
  console.log(`  Stress: ${currentState.stress} (very stressed)`);
  console.log('\nSystem will recommend calming, mood-lifting content...\n');
}

// Example 2: State hashing
export function stateHashingExample() {
  console.log('=== Example 2: State Hashing ===\n');

  const hasher = new StateHasher();

  const emotionalState = {
    valence: 0.35,
    arousal: -0.62,
    stress: 0.73,
    confidence: 0.8
  };

  const hash = hasher.hash(emotionalState);

  console.log('Emotional State:');
  console.log(`  Valence: ${emotionalState.valence}`);
  console.log(`  Arousal: ${emotionalState.arousal}`);
  console.log(`  Stress: ${emotionalState.stress}`);
  console.log('\nDiscretized Buckets:');
  console.log(`  Valence Bucket: ${hash.valenceBucket}`);
  console.log(`  Arousal Bucket: ${hash.arousalBucket}`);
  console.log(`  Stress Bucket: ${hash.stressBucket}`);
  console.log(`  Hash: ${hash.hash}`);
  console.log(`\nTotal State Space: ${hasher.getStateSpaceSize()} states\n`);
}

// Example 3: Outcome prediction
export function outcomePredictionExample() {
  console.log('=== Example 3: Outcome Prediction ===\n');

  const predictor = new OutcomePredictor();

  const currentState = {
    valence: -0.5,
    arousal: 0.7,
    stress: 0.9,
    confidence: 0.8
  };

  const contentProfile = {
    contentId: 'calm_nature_001',
    primaryTone: 'calming',
    valenceDelta: 0.6,   // Improves mood
    arousalDelta: -0.7,  // Reduces arousal
    intensity: 0.8,      // Strong effect
    complexity: 0.3,     // Simple content
    targetStates: [],
    embeddingId: 'emb_001',
    timestamp: Date.now()
  };

  const outcome = predictor.predict(currentState, contentProfile);

  console.log('Current State:');
  console.log(`  Valence: ${currentState.valence}`);
  console.log(`  Arousal: ${currentState.arousal}`);
  console.log(`  Stress: ${currentState.stress}`);
  console.log('\nContent Profile:');
  console.log(`  Valence Delta: ${contentProfile.valenceDelta}`);
  console.log(`  Arousal Delta: ${contentProfile.arousalDelta}`);
  console.log(`  Intensity: ${contentProfile.intensity}`);
  console.log('\nPredicted Outcome:');
  console.log(`  Expected Valence: ${outcome.expectedValence.toFixed(2)}`);
  console.log(`  Expected Arousal: ${outcome.expectedArousal.toFixed(2)}`);
  console.log(`  Expected Stress: ${outcome.expectedStress.toFixed(2)}`);
  console.log(`  Confidence: ${outcome.confidence.toFixed(2)}\n`);
}

// Example 4: Reasoning generation
export function reasoningExample() {
  console.log('=== Example 4: Reasoning Generation ===\n');

  const generator = new ReasoningGenerator();

  const currentState = {
    valence: -0.3,
    arousal: 0.5,
    stress: 0.7,
    confidence: 0.8
  };

  const desiredState = {
    valence: 0.5,
    arousal: -0.3,
    confidence: 0.9
  };

  const contentProfile = {
    contentId: 'meditation_001',
    primaryTone: 'calming',
    valenceDelta: 0.6,
    arousalDelta: -0.6,
    intensity: 0.7,
    complexity: 0.2,
    targetStates: [],
    embeddingId: 'emb_002',
    timestamp: Date.now()
  };

  const qValue = 0.75;
  const isExploration = false;

  const reasoning = generator.generate(
    currentState,
    desiredState,
    contentProfile,
    qValue,
    isExploration
  );

  console.log('Recommendation Reasoning:');
  console.log(`"${reasoning}"\n`);
}

// Example 5: Exploration strategy
export function explorationExample() {
  console.log('=== Example 5: Exploration Strategy ===\n');

  const strategy = new ExplorationStrategy(0.3);

  // Mock ranked content
  const rankedContent = [
    {
      contentId: 'top_1',
      title: 'Top Recommendation',
      profile: {} as any,
      qValue: 0.9,
      similarityScore: 0.85,
      combinedScore: 0.88,
      outcomeAlignment: 0.95,
      isExploration: false
    },
    {
      contentId: 'mid_1',
      title: 'Mid Recommendation',
      profile: {} as any,
      qValue: 0.6,
      similarityScore: 0.7,
      combinedScore: 0.64,
      outcomeAlignment: 0.8,
      isExploration: false
    },
    {
      contentId: 'low_1',
      title: 'Lower Recommendation',
      profile: {} as any,
      qValue: 0.3,
      similarityScore: 0.5,
      combinedScore: 0.36,
      outcomeAlignment: 0.6,
      isExploration: false
    }
  ];

  console.log('Original Rankings:');
  rankedContent.forEach((item, idx) => {
    console.log(`${idx + 1}. ${item.title} (score: ${item.combinedScore.toFixed(2)})`);
  });

  const withExploration = strategy.inject([...rankedContent], 0.3);

  console.log('\nAfter Exploration Injection:');
  withExploration.forEach((item, idx) => {
    const flag = item.isExploration ? ' [EXPLORATION]' : '';
    console.log(`${idx + 1}. ${item.title} (score: ${item.combinedScore.toFixed(2)})${flag}`);
  });

  console.log(`\nExploration Rate: ${(strategy.getRate() * 100).toFixed(0)}%\n`);
}

// Run all examples
if (require.main === module) {
  console.log('\n'.repeat(2));
  console.log('╔═══════════════════════════════════════════════════════════╗');
  console.log('║   EmotiStream RecommendationEngine Examples               ║');
  console.log('╚═══════════════════════════════════════════════════════════╝');
  console.log('\n');

  basicExample();
  stateHashingExample();
  outcomePredictionExample();
  reasoningExample();
  explorationExample();

  console.log('═'.repeat(60));
  console.log('All examples complete!\n');
}
