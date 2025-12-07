/**
 * E2B Test Runner
 *
 * Spawns an E2B sandbox with full internet access to run
 * real TMDB API integration tests
 */

import Sandbox from '@e2b/code-interpreter';

const TMDB_TOKEN = process.env.TMDB_API_TOKEN || process.env.NEXT_PUBLIC_TMDB_ACCESS_TOKEN;
const E2B_API_KEY = process.env.E2B_API_KEY;

if (!E2B_API_KEY) {
  console.error('❌ E2B_API_KEY not set');
  process.exit(1);
}

if (!TMDB_TOKEN) {
  console.error('❌ TMDB token not set');
  process.exit(1);
}

async function runTests() {
  console.log('🚀 Spawning E2B sandbox for TMDB API tests...\n');

  const sandbox = await Sandbox.create({
    apiKey: E2B_API_KEY,
  });

  try {
    console.log(`📦 Sandbox ID: ${sandbox.sandboxId}`);
    console.log('⏳ Installing dependencies and running tests...\n');

    // Install node and run a simple TMDB API test
    const result = await sandbox.runCode(`
import fetch from 'node-fetch';

const TMDB_TOKEN = "${TMDB_TOKEN}";

async function testTMDB() {
  const results = {
    passed: [],
    failed: [],
  };

  // Test 1: Search for "Inception"
  try {
    const searchRes = await fetch(
      'https://api.themoviedb.org/3/search/movie?query=Inception',
      {
        headers: {
          Authorization: \`Bearer \${TMDB_TOKEN}\`,
          'Content-Type': 'application/json',
        },
      }
    );
    const searchData = await searchRes.json();

    if (searchData.results && searchData.results.length > 0) {
      const inception = searchData.results.find(m => m.title === 'Inception');
      if (inception && inception.vote_average > 7) {
        results.passed.push('✅ Search: Found Inception with rating ' + inception.vote_average);
      } else {
        results.failed.push('❌ Search: Inception not found or low rating');
      }
    } else {
      results.failed.push('❌ Search: No results returned');
    }
  } catch (e) {
    results.failed.push('❌ Search: ' + e.message);
  }

  // Test 2: Get trending movies
  try {
    const trendingRes = await fetch(
      'https://api.themoviedb.org/3/trending/movie/week',
      {
        headers: {
          Authorization: \`Bearer \${TMDB_TOKEN}\`,
          'Content-Type': 'application/json',
        },
      }
    );
    const trendingData = await trendingRes.json();

    if (trendingData.results && trendingData.results.length > 0) {
      results.passed.push('✅ Trending: Got ' + trendingData.results.length + ' trending movies');
      results.passed.push('   Top 3: ' + trendingData.results.slice(0, 3).map(m => m.title).join(', '));
    } else {
      results.failed.push('❌ Trending: No results');
    }
  } catch (e) {
    results.failed.push('❌ Trending: ' + e.message);
  }

  // Test 3: Get movie details (Shawshank Redemption ID: 278)
  try {
    const detailsRes = await fetch(
      'https://api.themoviedb.org/3/movie/278',
      {
        headers: {
          Authorization: \`Bearer \${TMDB_TOKEN}\`,
          'Content-Type': 'application/json',
        },
      }
    );
    const movie = await detailsRes.json();

    if (movie.title === 'The Shawshank Redemption') {
      results.passed.push('✅ Details: Shawshank Redemption - Rating: ' + movie.vote_average + ', Runtime: ' + movie.runtime + 'min');
    } else {
      results.failed.push('❌ Details: Wrong movie returned: ' + movie.title);
    }
  } catch (e) {
    results.failed.push('❌ Details: ' + e.message);
  }

  // Test 4: Get streaming providers (Stranger Things ID: 66732)
  try {
    const streamRes = await fetch(
      'https://api.themoviedb.org/3/tv/66732/watch/providers',
      {
        headers: {
          Authorization: \`Bearer \${TMDB_TOKEN}\`,
          'Content-Type': 'application/json',
        },
      }
    );
    const streamData = await streamRes.json();

    if (streamData.results && streamData.results.US) {
      const usProviders = streamData.results.US;
      const flatrate = usProviders.flatrate || [];
      const netflix = flatrate.find(p => p.provider_name === 'Netflix');

      if (netflix) {
        results.passed.push('✅ Streaming: Stranger Things available on Netflix (provider_id: ' + netflix.provider_id + ')');
      } else {
        results.passed.push('✅ Streaming: Got providers but Netflix not in flatrate. Available: ' + flatrate.map(p => p.provider_name).join(', '));
      }
    } else {
      results.failed.push('❌ Streaming: No US providers found');
    }
  } catch (e) {
    results.failed.push('❌ Streaming: ' + e.message);
  }

  // Test 5: Discover action movies
  try {
    const discoverRes = await fetch(
      'https://api.themoviedb.org/3/discover/movie?with_genres=28&sort_by=popularity.desc',
      {
        headers: {
          Authorization: \`Bearer \${TMDB_TOKEN}\`,
          'Content-Type': 'application/json',
        },
      }
    );
    const discoverData = await discoverRes.json();

    if (discoverData.results && discoverData.results.length > 0) {
      results.passed.push('✅ Discover: Found ' + discoverData.results.length + ' action movies');
      results.passed.push('   Top 3: ' + discoverData.results.slice(0, 3).map(m => m.title).join(', '));
    } else {
      results.failed.push('❌ Discover: No action movies found');
    }
  } catch (e) {
    results.failed.push('❌ Discover: ' + e.message);
  }

  // Test 6: Get genres
  try {
    const genresRes = await fetch(
      'https://api.themoviedb.org/3/genre/movie/list',
      {
        headers: {
          Authorization: \`Bearer \${TMDB_TOKEN}\`,
          'Content-Type': 'application/json',
        },
      }
    );
    const genresData = await genresRes.json();

    if (genresData.genres && genresData.genres.length > 0) {
      results.passed.push('✅ Genres: Got ' + genresData.genres.length + ' genres');
      const actionGenre = genresData.genres.find(g => g.name === 'Action');
      if (actionGenre) {
        results.passed.push('   Action genre ID: ' + actionGenre.id);
      }
    } else {
      results.failed.push('❌ Genres: No genres returned');
    }
  } catch (e) {
    results.failed.push('❌ Genres: ' + e.message);
  }

  return results;
}

testTMDB().then(results => {
  console.log('\\n=== TMDB API Integration Test Results ===\\n');

  console.log('PASSED (' + results.passed.length + '):');
  results.passed.forEach(msg => console.log(msg));

  if (results.failed.length > 0) {
    console.log('\\nFAILED (' + results.failed.length + '):');
    results.failed.forEach(msg => console.log(msg));
  }

  console.log('\\n==========================================');
  console.log('Total: ' + results.passed.length + ' passed, ' + results.failed.length + ' failed');
  console.log('==========================================');
});
    `);

    // Print output
    if (result.logs.stdout.length > 0) {
      console.log(result.logs.stdout.join('\n'));
    }
    if (result.logs.stderr.length > 0) {
      console.error('Errors:', result.logs.stderr.join('\n'));
    }

  } finally {
    await sandbox.kill();
    console.log('\n🧹 Sandbox terminated');
  }
}

runTests().catch(console.error);
