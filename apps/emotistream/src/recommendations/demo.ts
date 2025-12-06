/**
 * RecommendationEngine Demo
 * Showcases the complete recommendation flow
 */

import { RecommendationEngine } from './engine';
import { MockCatalogGenerator } from '../content/mock-catalog';

async function runDemo() {
  console.log('=== EmotiStream RecommendationEngine Demo ===\n');

  // Step 1: Initialize engine
  console.log('Step 1: Initializing RecommendationEngine...');
  const engine = new RecommendationEngine();

  // Step 2: Generate and profile mock content
  console.log('Step 2: Generating mock content catalog...');
  const catalogGenerator = new MockCatalogGenerator();
  const catalog = catalogGenerator.generate(100);
  console.log(`Generated ${catalog.length} content items\n`);

  console.log('Step 3: Profiling content with emotional characteristics...');
  const profiler = engine.getProfiler();
  await profiler.batchProfile(catalog, 20);
  console.log('Content profiling complete\n');

  // Demo scenarios
  const scenarios = [
    {
      name: 'Stressed User',
      userId: 'user_stressed_001',
      state: { valence: -0.4, arousal: 0.6, stress: 0.8 },
      description: 'User is stressed and anxious, needs calming content'
    },
    {
      name: 'Happy User',
      userId: 'user_happy_001',
      state: { valence: 0.7, arousal: 0.3, stress: 0.2 },
      description: 'User is happy and content, maintain positive mood'
    },
    {
      name: 'Bored User',
      userId: 'user_bored_001',
      state: { valence: 0.0, arousal: -0.5, stress: 0.3 },
      description: 'User is bored and low-energy, needs stimulation'
    },
    {
      name: 'Sad User',
      userId: 'user_sad_001',
      state: { valence: -0.6, arousal: -0.3, stress: 0.5 },
      description: 'User is sad and lethargic, needs mood lift'
    }
  ];

  // Run scenarios
  for (const scenario of scenarios) {
    console.log(`\n${'='.repeat(60)}`);
    console.log(`Scenario: ${scenario.name}`);
    console.log(`Description: ${scenario.description}`);
    console.log(`Current State: valence=${scenario.state.valence}, arousal=${scenario.state.arousal}, stress=${scenario.state.stress}`);
    console.log('='.repeat(60));

    const recommendations = await engine.recommend(
      scenario.userId,
      scenario.state,
      5
    );

    console.log(`\nTop 5 Recommendations:\n`);

    recommendations.forEach((rec, idx) => {
      console.log(`${idx + 1}. ${rec.title}`);
      console.log(`   Content ID: ${rec.contentId}`);
      console.log(`   Q-Value: ${rec.qValue.toFixed(3)}`);
      console.log(`   Similarity: ${rec.similarityScore.toFixed(3)}`);
      console.log(`   Combined Score: ${rec.combinedScore.toFixed(3)}`);
      console.log(`   Exploration: ${rec.isExploration ? 'Yes' : 'No'}`);
      console.log(`   Predicted Outcome:`);
      console.log(`     - Valence: ${rec.predictedOutcome.expectedValence.toFixed(2)}`);
      console.log(`     - Arousal: ${rec.predictedOutcome.expectedArousal.toFixed(2)}`);
      console.log(`     - Stress: ${rec.predictedOutcome.expectedStress.toFixed(2)}`);
      console.log(`     - Confidence: ${rec.predictedOutcome.confidence.toFixed(2)}`);
      console.log(`   Reasoning: ${rec.reasoning}`);
      console.log('');
    });
  }

  console.log('\n=== Demo Complete ===\n');
  console.log('Key Features Demonstrated:');
  console.log('✓ Hybrid ranking (70% Q-value + 30% similarity)');
  console.log('✓ Emotional outcome prediction');
  console.log('✓ Human-readable reasoning generation');
  console.log('✓ Exploration strategy (ε-greedy)');
  console.log('✓ State-based personalization');
  console.log('✓ Homeostasis-driven desired state prediction');
}

// Run demo
if (require.main === module) {
  runDemo().catch(console.error);
}

export { runDemo };
