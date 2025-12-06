/**
 * EmotionDetector Demo
 * Example usage of the EmotionDetector module
 */

import { EmotionDetector } from '../src/emotion';

async function demo() {
  const detector = new EmotionDetector();

  console.log('=== EmotionDetector Demo ===\n');

  const testTexts = [
    "I'm so happy and excited about my new job!",
    'I feel sad and lonely today',
    'This deadline is making me so stressed and anxious',
    "I'm furious about what happened",
    'Feeling calm and peaceful this morning',
    'So tired and exhausted from work',
    'Wow, what a surprise!',
    'The weather is normal today',
  ];

  for (const text of testTexts) {
    console.log(`\n📝 Text: "${text}"\n`);

    try {
      const result = await detector.analyzeText(text);

      console.log('🎯 Current State:');
      console.log(`   Emotion: ${result.currentState.primaryEmotion}`);
      console.log(`   Valence: ${result.currentState.valence.toFixed(2)} (${result.currentState.valence > 0 ? 'positive' : 'negative'})`);
      console.log(`   Arousal: ${result.currentState.arousal.toFixed(2)} (${result.currentState.arousal > 0 ? 'high' : 'low'} energy)`);
      console.log(`   Stress: ${result.currentState.stressLevel.toFixed(2)}`);
      console.log(`   Confidence: ${result.currentState.confidence.toFixed(2)}`);

      console.log('\n🎯 Desired State:');
      console.log(`   Target Valence: ${result.desiredState.targetValence.toFixed(2)}`);
      console.log(`   Target Arousal: ${result.desiredState.targetArousal.toFixed(2)}`);
      console.log(`   Target Stress: ${result.desiredState.targetStress.toFixed(2)}`);
      console.log(`   Intensity: ${result.desiredState.intensity}`);
      console.log(`   Reasoning: ${result.desiredState.reasoning}`);

      console.log('\n🔢 State Hash:', result.stateHash);

      console.log('\n🎨 Emotion Vector (8D):');
      const emotions = ['joy', 'trust', 'fear', 'surprise', 'sadness', 'disgust', 'anger', 'anticipation'];
      const vector = Array.from(result.currentState.emotionVector);
      emotions.forEach((emotion, i) => {
        const bar = '█'.repeat(Math.round(vector[i] * 20));
        console.log(`   ${emotion.padEnd(12)} ${vector[i].toFixed(3)} ${bar}`);
      });

      console.log('\n' + '─'.repeat(80));
    } catch (error) {
      console.error('❌ Error:', (error as Error).message);
    }
  }
}

// Run demo
demo().catch(console.error);
