/**
 * Manual verification script for ContentProfiler implementation
 */

import { ContentProfiler } from '../src/content/profiler.js';
import { EmbeddingGenerator } from '../src/content/embedding-generator.js';
import { VectorStore } from '../src/content/vector-store.js';
import { MockCatalogGenerator } from '../src/content/mock-catalog.js';
import { BatchProcessor } from '../src/content/batch-processor.js';
import { ContentMetadata } from '../src/content/types.js';

async function verify() {
  console.log('🧪 Verifying ContentProfiler Implementation...\n');

  let passed = 0;
  let failed = 0;

  // Test 1: EmbeddingGenerator creates 1536D vectors
  try {
    const generator = new EmbeddingGenerator();
    const mockProfile = {
      contentId: 'test_001',
      primaryTone: 'uplifting',
      valenceDelta: 0.6,
      arousalDelta: 0.2,
      intensity: 0.7,
      complexity: 0.5,
      targetStates: [{ currentValence: 0.3, currentArousal: 0.1, description: 'test' }],
      embeddingId: '',
      timestamp: Date.now()
    };
    const mockContent = {
      contentId: 'test_001',
      title: 'Test',
      description: 'Test',
      platform: 'mock' as const,
      genres: ['drama'],
      category: 'movie' as const,
      tags: ['test'],
      duration: 120
    };

    const embedding = generator.generate(mockProfile, mockContent);

    if (embedding.length === 1536) {
      console.log('✅ Test 1: Embedding dimension is 1536');
      passed++;
    } else {
      console.log(`❌ Test 1: Expected 1536, got ${embedding.length}`);
      failed++;
    }

    // Verify normalization
    let magnitude = 0;
    for (let i = 0; i < embedding.length; i++) {
      magnitude += embedding[i] * embedding[i];
    }
    magnitude = Math.sqrt(magnitude);

    if (Math.abs(magnitude - 1.0) < 0.001) {
      console.log('✅ Test 2: Embedding is normalized to unit length');
      passed++;
    } else {
      console.log(`❌ Test 2: Magnitude is ${magnitude}, expected 1.0`);
      failed++;
    }
  } catch (error) {
    console.log(`❌ Test 1-2 failed: ${error}`);
    failed += 2;
  }

  // Test 3: MockCatalogGenerator creates 200 items
  try {
    const catalogGenerator = new MockCatalogGenerator();
    const catalog = catalogGenerator.generate(200);

    if (catalog.length === 200) {
      console.log('✅ Test 3: Mock catalog has 200 items');
      passed++;
    } else {
      console.log(`❌ Test 3: Expected 200 items, got ${catalog.length}`);
      failed++;
    }

    const categories = new Set(catalog.map(c => c.category));
    if (categories.size === 6) {
      console.log('✅ Test 4: All 6 categories present');
      passed++;
    } else {
      console.log(`❌ Test 4: Expected 6 categories, got ${categories.size}`);
      failed++;
    }
  } catch (error) {
    console.log(`❌ Test 3-4 failed: ${error}`);
    failed += 2;
  }

  // Test 5: VectorStore search
  try {
    const store = new VectorStore();
    const vector1 = new Float32Array(1536);
    vector1.fill(0.5);
    const vector2 = new Float32Array(1536);
    vector2.fill(0.3);

    await store.upsert('test1', vector1, { title: 'Test 1' });
    await store.upsert('test2', vector2, { title: 'Test 2' });

    const results = await store.search(vector1, 2);

    if (results.length === 2) {
      console.log('✅ Test 5: VectorStore returns results');
      passed++;
    } else {
      console.log(`❌ Test 5: Expected 2 results, got ${results.length}`);
      failed++;
    }

    if (results[0].score >= results[1].score) {
      console.log('✅ Test 6: Results are sorted by similarity');
      passed++;
    } else {
      console.log(`❌ Test 6: Results not sorted correctly`);
      failed++;
    }
  } catch (error) {
    console.log(`❌ Test 5-6 failed: ${error}`);
    failed += 2;
  }

  // Test 7: ContentProfiler integration
  try {
    const profiler = new ContentProfiler();
    const mockContent: ContentMetadata = {
      contentId: 'test_profile',
      title: 'Test Content',
      description: 'Test description',
      platform: 'mock',
      genres: ['drama', 'comedy'],
      category: 'movie',
      tags: ['emotional'],
      duration: 120
    };

    const profile = await profiler.profile(mockContent);

    if (profile.contentId === mockContent.contentId) {
      console.log('✅ Test 7: ContentProfiler creates profile');
      passed++;
    } else {
      console.log(`❌ Test 7: Profile contentId mismatch`);
      failed++;
    }

    if (profile.valenceDelta >= -1 && profile.valenceDelta <= 1) {
      console.log('✅ Test 8: Valence delta in valid range');
      passed++;
    } else {
      console.log(`❌ Test 8: Valence delta out of range: ${profile.valenceDelta}`);
      failed++;
    }

    if (profile.embeddingId) {
      console.log('✅ Test 9: Embedding ID generated');
      passed++;
    } else {
      console.log(`❌ Test 9: No embedding ID`);
      failed++;
    }
  } catch (error) {
    console.log(`❌ Test 7-9 failed: ${error}`);
    failed += 3;
  }

  // Test 10: BatchProcessor
  try {
    const processor = new BatchProcessor();
    const mockContents: ContentMetadata[] = Array.from({ length: 5 }, (_, i) => ({
      contentId: `batch_${i}`,
      title: `Batch ${i}`,
      description: 'Test',
      platform: 'mock' as const,
      genres: ['drama'],
      category: 'movie' as const,
      tags: ['test'],
      duration: 120
    }));

    let count = 0;
    for await (const profile of processor.profile(mockContents, 2)) {
      count++;
    }

    if (count === 5) {
      console.log('✅ Test 10: BatchProcessor processes all items');
      passed++;
    } else {
      console.log(`❌ Test 10: Expected 5 items, processed ${count}`);
      failed++;
    }
  } catch (error) {
    console.log(`❌ Test 10 failed: ${error}`);
    failed++;
  }

  // Summary
  console.log(`\n📊 Test Summary:`);
  console.log(`   Passed: ${passed}/10`);
  console.log(`   Failed: ${failed}/10`);
  console.log(`   Coverage: ${(passed / 10 * 100).toFixed(0)}%`);

  if (failed === 0) {
    console.log(`\n✨ All tests passed! Implementation complete.`);
  } else {
    console.log(`\n⚠️  Some tests failed. Review implementation.`);
    process.exit(1);
  }
}

verify().catch(console.error);
