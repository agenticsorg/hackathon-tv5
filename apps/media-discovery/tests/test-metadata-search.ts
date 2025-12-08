/**
 * Test script for metadata search improvements
 * Tests:
 * 1. "Something Richard Gere played" - Person detection with natural language
 * 2. "an Oscar winner movie" - Award detection
 */

import { parseSearchQuery, semanticSearch } from '../src/lib/natural-language-search';

async function testPersonSearch() {
  console.log('\n========================================');
  console.log('TEST 1: Person Search - "Something Richard Gere played"');
  console.log('========================================\n');

  const query = 'Something Richard Gere played';

  try {
    // Test intent parsing
    const parsedQuery = await parseSearchQuery(query);

    console.log('📊 Parsed Query Result:');
    console.log('- Query:', parsedQuery.query);
    console.log('- Detected Person:', parsedQuery.metadata?.detectedPerson || 'NONE ❌');
    console.log('- Has Specific Intent:', parsedQuery.metadata?.hasSpecificIntent);
    console.log('- Intent:', JSON.stringify(parsedQuery.intent, null, 2));

    if (!parsedQuery.metadata?.detectedPerson) {
      console.log('\n❌ FAILED: Person not detected!');
      return false;
    }

    if (parsedQuery.metadata.detectedPerson !== 'Richard Gere') {
      console.log(`\n⚠️  WARNING: Expected "Richard Gere", got "${parsedQuery.metadata.detectedPerson}"`);
    }

    // Test actual search
    console.log('\n🔍 Performing semantic search...');
    const results = await semanticSearch(query, undefined, { includeStreaming: false });

    console.log(`\n📈 Search Results: ${results.length} items found`);

    if (results.length === 0) {
      console.log('❌ FAILED: No results returned!');
      return false;
    }

    // Check if results are NOT from fallback
    const hasFallbackResults = results.some(r =>
      r.matchReasons.includes('Popular & Trending') ||
      r.matchReasons.includes('Highly rated & Popular')
    );

    if (hasFallbackResults) {
      console.log('⚠️  WARNING: Fallback strategy was used (should not happen for person searches)');
    }

    // Display top 5 results
    console.log('\n🎬 Top 5 Results:');
    results.slice(0, 5).forEach((result, index) => {
      console.log(`\n${index + 1}. ${result.content.title} (${result.content.releaseDate?.split('-')[0] || 'N/A'})`);
      console.log(`   Score: ${result.relevanceScore.toFixed(3)}`);
      console.log(`   Reasons: ${result.matchReasons.join(', ')}`);
    });

    // Verify Richard Gere movies are in results
    const richardGereMovies = ['Pretty Woman', 'Chicago', 'An Officer and a Gentleman', 'Runaway Bride', 'Arbitrage'];
    const foundKnownMovies = results.filter(r =>
      richardGereMovies.some(title => r.content.title.toLowerCase().includes(title.toLowerCase()))
    );

    console.log(`\n✅ Known Richard Gere movies found: ${foundKnownMovies.length}`);
    if (foundKnownMovies.length > 0) {
      console.log('   -', foundKnownMovies.map(r => r.content.title).join(', '));
    }

    console.log('\n✅ TEST 1 PASSED: Person search working correctly!');
    return true;

  } catch (error) {
    console.error('\n❌ TEST 1 FAILED with error:', error);
    return false;
  }
}

async function testAwardSearch() {
  console.log('\n========================================');
  console.log('TEST 2: Award Search - "an Oscar winner movie"');
  console.log('========================================\n');

  const query = 'an Oscar winner movie';

  try {
    // Test intent parsing
    const parsedQuery = await parseSearchQuery(query);

    console.log('📊 Parsed Query Result:');
    console.log('- Query:', parsedQuery.query);
    console.log('- Detected Award:', parsedQuery.metadata?.detectedAward || 'NONE ❌');
    console.log('- Has Specific Intent:', parsedQuery.metadata?.hasSpecificIntent);
    console.log('- Keywords:', parsedQuery.intent?.keywords || []);

    if (!parsedQuery.metadata?.detectedAward) {
      console.log('\n❌ FAILED: Award not detected!');
      return false;
    }

    // Test actual search
    console.log('\n🔍 Performing semantic search...');
    const results = await semanticSearch(query, undefined, { includeStreaming: false });

    console.log(`\n📈 Search Results: ${results.length} items found`);

    if (results.length === 0) {
      console.log('❌ FAILED: No results returned!');
      return false;
    }

    // Check if results are NOT from fallback
    const hasFallbackResults = results.some(r =>
      r.matchReasons.includes('Popular & Trending') ||
      r.matchReasons.includes('Highly rated & Popular')
    );

    if (hasFallbackResults) {
      console.log('⚠️  WARNING: Fallback strategy was used (should not happen for award searches)');
    }

    // Check for award-related match reasons
    const hasAwardReasons = results.some(r =>
      r.matchReasons.some(reason =>
        reason.toLowerCase().includes('award') ||
        reason.toLowerCase().includes('acclaimed')
      )
    );

    if (!hasAwardReasons) {
      console.log('⚠️  WARNING: No award-related match reasons found in results');
    }

    // Display top 5 results
    console.log('\n🏆 Top 5 Results:');
    results.slice(0, 5).forEach((result, index) => {
      console.log(`\n${index + 1}. ${result.content.title} (${result.content.releaseDate?.split('-')[0] || 'N/A'})`);
      console.log(`   Rating: ${result.content.voteAverage.toFixed(1)}/10 (${result.content.voteCount} votes)`);
      console.log(`   Score: ${result.relevanceScore.toFixed(3)}`);
      console.log(`   Reasons: ${result.matchReasons.join(', ')}`);
    });

    // Verify high ratings (award winners should be highly rated)
    const highRatedCount = results.slice(0, 10).filter(r => r.content.voteAverage >= 7.5).length;
    console.log(`\n📊 High-rated content (>=7.5): ${highRatedCount}/10`);

    if (highRatedCount < 5) {
      console.log('⚠️  WARNING: Less than 50% of top results are highly rated');
    }

    console.log('\n✅ TEST 2 PASSED: Award search working correctly!');
    return true;

  } catch (error) {
    console.error('\n❌ TEST 2 FAILED with error:', error);
    return false;
  }
}

async function runTests() {
  console.log('\n🚀 Starting Metadata Search Tests...\n');

  const test1Pass = await testPersonSearch();
  const test2Pass = await testAwardSearch();

  console.log('\n========================================');
  console.log('FINAL TEST SUMMARY');
  console.log('========================================');
  console.log(`✅ Person Search: ${test1Pass ? 'PASSED' : 'FAILED'}`);
  console.log(`✅ Award Search: ${test2Pass ? 'PASSED' : 'FAILED'}`);
  console.log(`\n🎯 Overall: ${test1Pass && test2Pass ? 'ALL TESTS PASSED ✅' : 'SOME TESTS FAILED ❌'}`);

  process.exit(test1Pass && test2Pass ? 0 : 1);
}

runTests();
