/**
 * Verification Script for EmotionDetector Module
 * Runs comprehensive checks to ensure all components are working
 */

import { EmotionDetector, hashState, predictDesiredState } from '../src/emotion';

async function verifyModule() {
  console.log('🔍 Verifying EmotionDetector Module...\n');

  let passed = 0;
  let failed = 0;

  // Test 1: Module imports
  console.log('✓ Test 1: Module imports successful');
  passed++;

  // Test 2: Create detector instance
  const detector = new EmotionDetector();
  console.log('✓ Test 2: EmotionDetector instantiation successful');
  passed++;

  // Test 3: Analyze happy emotion
  try {
    const result1 = await detector.analyzeText('I am so happy and excited!');
    if (result1.currentState.primaryEmotion === 'joy' && result1.currentState.valence > 0.5) {
      console.log('✓ Test 3: Happy emotion detection correct');
      passed++;
    } else {
      console.log('✗ Test 3: Happy emotion detection incorrect');
      failed++;
    }
  } catch (error) {
    console.log('✗ Test 3: Failed -', (error as Error).message);
    failed++;
  }

  // Test 4: Analyze sad emotion
  try {
    const result2 = await detector.analyzeText('I feel so sad and depressed');
    if (result2.currentState.primaryEmotion === 'sadness' && result2.currentState.valence < 0) {
      console.log('✓ Test 4: Sad emotion detection correct');
      passed++;
    } else {
      console.log('✗ Test 4: Sad emotion detection incorrect');
      failed++;
    }
  } catch (error) {
    console.log('✗ Test 4: Failed -', (error as Error).message);
    failed++;
  }

  // Test 5: Analyze stressed emotion
  try {
    const result3 = await detector.analyzeText('I am extremely stressed and anxious');
    if (result3.currentState.stressLevel > 0.6 && result3.desiredState.targetArousal < 0) {
      console.log('✓ Test 5: Stress detection and desired state prediction correct');
      passed++;
    } else {
      console.log('✗ Test 5: Stress detection or prediction incorrect');
      failed++;
    }
  } catch (error) {
    console.log('✗ Test 5: Failed -', (error as Error).message);
    failed++;
  }

  // Test 6: Emotion vector validation
  try {
    const result4 = await detector.analyzeText('I am happy');
    const sum = Array.from(result4.currentState.emotionVector).reduce((a, b) => a + b, 0);
    if (Math.abs(sum - 1.0) < 0.01) {
      console.log('✓ Test 6: Emotion vector normalization correct');
      passed++;
    } else {
      console.log('✗ Test 6: Emotion vector not normalized (sum=' + sum + ')');
      failed++;
    }
  } catch (error) {
    console.log('✗ Test 6: Failed -', (error as Error).message);
    failed++;
  }

  // Test 7: State hash generation
  try {
    const result5 = await detector.analyzeText('Test text');
    if (/^\d:\d:\d$/.test(result5.stateHash)) {
      console.log('✓ Test 7: State hash format correct');
      passed++;
    } else {
      console.log('✗ Test 7: State hash format incorrect');
      failed++;
    }
  } catch (error) {
    console.log('✗ Test 7: Failed -', (error as Error).message);
    failed++;
  }

  // Test 8: Input validation (empty)
  try {
    await detector.analyzeText('');
    console.log('✗ Test 8: Should reject empty input');
    failed++;
  } catch (error) {
    console.log('✓ Test 8: Empty input rejected correctly');
    passed++;
  }

  // Test 9: Input validation (too short)
  try {
    await detector.analyzeText('ab');
    console.log('✗ Test 9: Should reject short input');
    failed++;
  } catch (error) {
    console.log('✓ Test 9: Short input rejected correctly');
    passed++;
  }

  // Test 10: All emotion types
  const testEmotions = [
    { text: 'I am joyful', expected: 'joy' },
    { text: 'I feel sad', expected: 'sadness' },
    { text: 'I am angry', expected: 'anger' },
    { text: 'I am anxious', expected: 'fear' },
    { text: 'I trust you', expected: 'trust' },
    { text: 'What a surprise', expected: 'surprise' },
  ];

  let emotionTestsPassed = 0;
  for (const test of testEmotions) {
    try {
      const result = await detector.analyzeText(test.text);
      if (result.currentState.primaryEmotion === test.expected) {
        emotionTestsPassed++;
      }
    } catch (error) {
      // Skip
    }
  }

  if (emotionTestsPassed >= 5) {
    console.log(`✓ Test 10: Multiple emotion types detected (${emotionTestsPassed}/${testEmotions.length})`);
    passed++;
  } else {
    console.log(`✗ Test 10: Multiple emotion detection failed (${emotionTestsPassed}/${testEmotions.length})`);
    failed++;
  }

  // Summary
  console.log('\n' + '='.repeat(60));
  console.log(`📊 Results: ${passed} passed, ${failed} failed`);
  console.log('='.repeat(60));

  if (failed === 0) {
    console.log('\n✅ All tests passed! EmotionDetector module is working correctly.');
    return 0;
  } else {
    console.log('\n⚠️  Some tests failed. Please review the implementation.');
    return 1;
  }
}

// Run verification
verifyModule()
  .then((exitCode) => process.exit(exitCode))
  .catch((error) => {
    console.error('❌ Verification failed:', error);
    process.exit(1);
  });
