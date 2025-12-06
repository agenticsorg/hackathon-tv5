#!/usr/bin/env npx tsx
/**
 * Quick test script for TMDB integration
 * Run: npx tsx scripts/test-tmdb.ts
 */

import dotenv from 'dotenv';
dotenv.config();

import { isTMDBConfigured, getTrending, getPopularMovies } from '../src/content/tmdb-client.js';
import { tmdbCatalog } from '../src/content/tmdb-catalog.js';

async function testTMDB() {
  console.log('\n🎬 TMDB Integration Test\n');
  console.log('═'.repeat(50));

  // Check if configured
  const configured = isTMDBConfigured();
  console.log(`\n✅ TMDB configured: ${configured}`);

  if (!configured) {
    console.log('\n❌ TMDB_ACCESS_TOKEN not found in environment.');
    console.log('   Add it to apps/emotistream/.env:');
    console.log('   TMDB_ACCESS_TOKEN=your_token_here\n');
    process.exit(1);
  }

  try {
    // Test direct API calls
    console.log('\n📡 Testing direct TMDB API...');

    const trending = await getTrending('movie', 'week');
    console.log(`   Trending movies: ${trending.length} items`);
    console.log(`   First: "${(trending[0] as any).title}" (${(trending[0] as any).vote_average}⭐)`);

    const popular = await getPopularMovies(1);
    console.log(`   Popular movies: ${popular.results.length} items`);

    // Test catalog
    console.log('\n📦 Testing TMDBCatalog...');

    const catalog = await tmdbCatalog.fetchCatalog(20);
    console.log(`   Catalog size: ${catalog.length} items`);

    // Show sample content
    console.log('\n📋 Sample content:');
    catalog.slice(0, 5).forEach((item, i) => {
      console.log(`   ${i + 1}. [${item.category}] ${item.title}`);
      console.log(`      Genres: ${item.genres.join(', ')}`);
      console.log(`      Tags: ${item.tags.slice(0, 4).join(', ')}`);
      console.log(`      Rating: ${item.rating}⭐ | Poster: ${item.posterUrl ? '✅' : '❌'}`);
    });

    console.log('\n═'.repeat(50));
    console.log('✅ TMDB integration working!\n');

  } catch (error) {
    console.error('\n❌ Error:', error);
    process.exit(1);
  }
}

testTMDB();
