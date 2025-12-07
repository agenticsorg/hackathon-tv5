/**
 * Manual Voice Query Test Script
 * Quick verification that all query types return results
 */

import { semanticSearch } from '../src/lib/natural-language-search.ts';

const testQueries = [
  // Mood queries (previously failing)
  { query: 'show me something cool', category: 'Mood' },
  { query: 'bring me something interesting', category: 'Mood' },
  { query: 'I want something awesome', category: 'Mood' },
  { query: 'show me something good', category: 'Mood' },

  // Person queries (previously failing)
  { query: 'show me something Richard Gere played', category: 'Person' },
  { query: 'movies with Tom Hanks', category: 'Person' },
  { query: 'films directed by Spielberg', category: 'Person' },

  // Trending queries (previously failing)
  { query: "what's new on Netflix", category: 'Trending' },
  { query: "what's trending", category: 'Trending' },
  { query: 'latest movies', category: 'Trending' },

  // Generic queries (previously failing - fallback)
  { query: 'something to watch', category: 'Generic' },
  { query: 'entertain me', category: 'Generic' },
  { query: 'surprise me', category: 'Generic' },
];

console.log('🎬 Voice Query Comprehensive Test\n');
console.log('Testing queries that previously returned NO results...\n');

let passCount = 0;
let failCount = 0;

for (const { query, category } of testQueries) {
  try {
    console.log(`\n[${category}] Testing: "${query}"`);
    const results = await semanticSearch(query);

    if (results.length > 0) {
      passCount++;
      console.log(`✅ PASS - ${results.length} results`);
      console.log(`   Top result: ${results[0].content.title}`);
      console.log(`   Match reason: ${results[0].matchReasons.join(', ')}`);
    } else {
      failCount++;
      console.log(`❌ FAIL - No results returned`);
    }
  } catch (error) {
    failCount++;
    console.log(`❌ ERROR - ${error.message}`);
  }
}

console.log(`\n${'='.repeat(60)}`);
console.log(`\n📊 Test Summary:`);
console.log(`   ✅ Passed: ${passCount}/${testQueries.length}`);
console.log(`   ❌ Failed: ${failCount}/${testQueries.length}`);
console.log(`   Success Rate: ${((passCount / testQueries.length) * 100).toFixed(1)}%`);

if (failCount === 0) {
  console.log(`\n🎉 ALL TESTS PASSED! Voice queries are fully fixed.`);
} else {
  console.log(`\n⚠️  Some queries still failing. Review above for details.`);
}
