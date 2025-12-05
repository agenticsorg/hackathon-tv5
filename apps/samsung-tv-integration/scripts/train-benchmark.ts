#!/usr/bin/env node
/**
 * Training & Benchmarking Suite
 *
 * Trains the Q-Learning system and measures performance
 */

import { PreferenceLearningSystem } from '../src/learning/preference-learning.js';
import {
  generateContentEmbedding,
  cosineSimilarity,
  batchSimilarity,
  ContentEmbeddingCache
} from '../src/learning/embeddings.js';
import { ContentMetadata, Genre, ContentType, ViewingSession } from '../src/learning/types.js';

// Sample content library
const SAMPLE_CONTENT: ContentMetadata[] = [
  { id: 'movie-1', title: 'The Dark Knight', type: 'movie', genres: ['action', 'crime', 'drama'], rating: 9.0, duration: 152, popularity: 95, releaseYear: 2008, actors: ['Christian Bale'], directors: ['Christopher Nolan'], keywords: ['batman', 'joker'] },
  { id: 'movie-2', title: 'Inception', type: 'movie', genres: ['action', 'science_fiction', 'thriller'], rating: 8.8, duration: 148, popularity: 90, releaseYear: 2010, actors: ['Leonardo DiCaprio'], directors: ['Christopher Nolan'], keywords: ['dreams', 'heist'] },
  { id: 'movie-3', title: 'Interstellar', type: 'movie', genres: ['science_fiction', 'drama', 'adventure'], rating: 8.6, duration: 169, popularity: 88, releaseYear: 2014, actors: ['Matthew McConaughey'], directors: ['Christopher Nolan'], keywords: ['space', 'time'] },
  { id: 'movie-4', title: 'The Shawshank Redemption', type: 'movie', genres: ['drama'], rating: 9.3, duration: 142, popularity: 92, releaseYear: 1994, actors: ['Tim Robbins'], directors: ['Frank Darabont'], keywords: ['prison', 'hope'] },
  { id: 'movie-5', title: 'Pulp Fiction', type: 'movie', genres: ['crime', 'drama'], rating: 8.9, duration: 154, popularity: 85, releaseYear: 1994, actors: ['John Travolta'], directors: ['Quentin Tarantino'], keywords: ['gangster', 'nonlinear'] },
  { id: 'movie-6', title: 'The Godfather', type: 'movie', genres: ['crime', 'drama'], rating: 9.2, duration: 175, popularity: 90, releaseYear: 1972, actors: ['Marlon Brando'], directors: ['Francis Ford Coppola'], keywords: ['mafia', 'family'] },
  { id: 'movie-7', title: 'Fight Club', type: 'movie', genres: ['drama', 'thriller'], rating: 8.8, duration: 139, popularity: 82, releaseYear: 1999, actors: ['Brad Pitt'], directors: ['David Fincher'], keywords: ['identity', 'anarchy'] },
  { id: 'movie-8', title: 'Forrest Gump', type: 'movie', genres: ['drama', 'romance'], rating: 8.8, duration: 142, popularity: 88, releaseYear: 1994, actors: ['Tom Hanks'], directors: ['Robert Zemeckis'], keywords: ['history', 'life'] },
  { id: 'movie-9', title: 'The Matrix', type: 'movie', genres: ['action', 'science_fiction'], rating: 8.7, duration: 136, popularity: 87, releaseYear: 1999, actors: ['Keanu Reeves'], directors: ['Lana Wachowski'], keywords: ['simulation', 'reality'] },
  { id: 'movie-10', title: 'Goodfellas', type: 'movie', genres: ['crime', 'drama'], rating: 8.7, duration: 146, popularity: 80, releaseYear: 1990, actors: ['Robert De Niro'], directors: ['Martin Scorsese'], keywords: ['mafia', 'true story'] },
  { id: 'tv-1', title: 'Breaking Bad', type: 'tv_show', genres: ['crime', 'drama', 'thriller'], rating: 9.5, duration: 49, popularity: 95, releaseYear: 2008, actors: ['Bryan Cranston'], directors: ['Vince Gilligan'], keywords: ['meth', 'chemistry'] },
  { id: 'tv-2', title: 'Game of Thrones', type: 'tv_show', genres: ['drama', 'fantasy', 'adventure'], rating: 9.2, duration: 57, popularity: 93, releaseYear: 2011, actors: ['Emilia Clarke'], directors: ['David Benioff'], keywords: ['dragons', 'throne'] },
  { id: 'tv-3', title: 'The Wire', type: 'tv_show', genres: ['crime', 'drama'], rating: 9.3, duration: 60, popularity: 78, releaseYear: 2002, actors: ['Dominic West'], directors: ['David Simon'], keywords: ['police', 'baltimore'] },
  { id: 'tv-4', title: 'Stranger Things', type: 'tv_show', genres: ['drama', 'fantasy', 'horror'], rating: 8.7, duration: 51, popularity: 90, releaseYear: 2016, actors: ['Millie Bobby Brown'], directors: ['Duffer Brothers'], keywords: ['80s', 'supernatural'] },
  { id: 'tv-5', title: 'The Office', type: 'tv_show', genres: ['comedy'], rating: 8.9, duration: 22, popularity: 88, releaseYear: 2005, actors: ['Steve Carell'], directors: ['Greg Daniels'], keywords: ['workplace', 'mockumentary'] },
  { id: 'tv-6', title: 'Friends', type: 'tv_show', genres: ['comedy', 'romance'], rating: 8.9, duration: 22, popularity: 92, releaseYear: 1994, actors: ['Jennifer Aniston'], directors: ['David Crane'], keywords: ['nyc', 'friendship'] },
  { id: 'tv-7', title: 'Chernobyl', type: 'tv_show', genres: ['drama', 'history', 'thriller'], rating: 9.4, duration: 65, popularity: 85, releaseYear: 2019, actors: ['Jared Harris'], directors: ['Craig Mazin'], keywords: ['nuclear', 'disaster'] },
  { id: 'tv-8', title: 'The Mandalorian', type: 'tv_show', genres: ['action', 'adventure', 'science_fiction'], rating: 8.7, duration: 40, popularity: 89, releaseYear: 2019, actors: ['Pedro Pascal'], directors: ['Jon Favreau'], keywords: ['star wars', 'bounty hunter'] },
  { id: 'doc-1', title: 'Planet Earth II', type: 'documentary', genres: ['documentary'], rating: 9.5, duration: 50, popularity: 85, releaseYear: 2016, actors: ['David Attenborough'], directors: ['BBC'], keywords: ['nature', 'animals'] },
  { id: 'doc-2', title: 'Our Planet', type: 'documentary', genres: ['documentary'], rating: 9.3, duration: 50, popularity: 82, releaseYear: 2019, actors: ['David Attenborough'], directors: ['Netflix'], keywords: ['nature', 'climate'] },
];

// User viewing patterns for simulation
interface UserProfile {
  name: string;
  favoriteGenres: Genre[];
  favoriteTypes: ContentType[];
  avgCompletionRate: number;
  avgRating: number;
}

const USER_PROFILES: UserProfile[] = [
  { name: 'Action Lover', favoriteGenres: ['action', 'thriller', 'science_fiction'], favoriteTypes: ['movie'], avgCompletionRate: 0.85, avgRating: 4.2 },
  { name: 'Drama Fan', favoriteGenres: ['drama', 'crime'], favoriteTypes: ['movie', 'tv_show'], avgCompletionRate: 0.92, avgRating: 4.5 },
  { name: 'TV Binger', favoriteGenres: ['drama', 'comedy'], favoriteTypes: ['tv_show'], avgCompletionRate: 0.78, avgRating: 4.0 },
  { name: 'Documentary Enthusiast', favoriteGenres: ['documentary', 'history'], favoriteTypes: ['documentary', 'tv_show'], avgCompletionRate: 0.95, avgRating: 4.8 },
  { name: 'Casual Viewer', favoriteGenres: ['comedy', 'family', 'animation'], favoriteTypes: ['movie', 'tv_show'], avgCompletionRate: 0.65, avgRating: 3.8 },
];

function generateSession(content: ContentMetadata, profile: UserProfile): ViewingSession {
  const isPreferred = content.genres.some(g => profile.favoriteGenres.includes(g)) &&
                      profile.favoriteTypes.includes(content.type);

  const completionRate = isPreferred
    ? Math.min(1, profile.avgCompletionRate + Math.random() * 0.15)
    : Math.max(0.1, profile.avgCompletionRate - 0.2 + Math.random() * 0.3);

  const rating = isPreferred
    ? Math.min(5, Math.round(profile.avgRating + Math.random()))
    : Math.max(1, Math.round(profile.avgRating - 1 + Math.random() * 2));

  return {
    id: `session-${Date.now()}-${Math.random().toString(36).slice(2)}`,
    contentId: content.id,
    contentMetadata: content,
    startTime: new Date().toISOString(),
    watchDuration: Math.round((content.duration || 90) * completionRate),
    completionRate,
    userRating: rating,
    implicit: {
      paused: Math.floor(Math.random() * 5),
      rewound: isPreferred ? Math.floor(Math.random() * 3) : 0,
      fastForwarded: isPreferred ? 0 : Math.floor(Math.random() * 5),
      volumeChanges: Math.floor(Math.random() * 4),
    },
  };
}

// Benchmark functions
function benchmarkEmbeddings(iterations: number = 1000): { opsPerSec: number; avgTimeMs: number } {
  const content = SAMPLE_CONTENT[0];
  const start = performance.now();

  for (let i = 0; i < iterations; i++) {
    generateContentEmbedding(content);
  }

  const elapsed = performance.now() - start;
  return {
    opsPerSec: Math.round((iterations / elapsed) * 1000),
    avgTimeMs: elapsed / iterations,
  };
}

function benchmarkSimilarity(iterations: number = 10000): { opsPerSec: number; avgTimeMs: number } {
  const embeddings = SAMPLE_CONTENT.slice(0, 10).map(c => generateContentEmbedding(c));
  const start = performance.now();

  for (let i = 0; i < iterations; i++) {
    const a = embeddings[i % embeddings.length];
    const b = embeddings[(i + 1) % embeddings.length];
    cosineSimilarity(a, b);
  }

  const elapsed = performance.now() - start;
  return {
    opsPerSec: Math.round((iterations / elapsed) * 1000),
    avgTimeMs: elapsed / iterations,
  };
}

function benchmarkBatchSimilarity(iterations: number = 100): { opsPerSec: number; avgTimeMs: number } {
  const embeddings = SAMPLE_CONTENT.map(c => generateContentEmbedding(c));
  const query = embeddings[0];
  const start = performance.now();

  for (let i = 0; i < iterations; i++) {
    batchSimilarity(query, embeddings, 10);
  }

  const elapsed = performance.now() - start;
  return {
    opsPerSec: Math.round((iterations / elapsed) * 1000),
    avgTimeMs: elapsed / iterations,
  };
}

function benchmarkCache(iterations: number = 5000): { hitRate: number; missTimeMs: number; hitTimeMs: number } {
  const cache = new ContentEmbeddingCache(100);
  let hits = 0;
  let misses = 0;
  let hitTime = 0;
  let missTime = 0;

  for (let i = 0; i < iterations; i++) {
    const content = SAMPLE_CONTENT[i % SAMPLE_CONTENT.length];
    const start = performance.now();
    const cached = cache.get(content.id);

    if (cached) {
      hits++;
      hitTime += performance.now() - start;
    } else {
      misses++;
      cache.getOrCompute(content);
      missTime += performance.now() - start;
    }
  }

  return {
    hitRate: hits / iterations,
    missTimeMs: missTime / misses,
    hitTimeMs: hitTime / (hits || 1),
  };
}

async function trainAndBenchmark() {
  console.log('═══════════════════════════════════════════════════════════════');
  console.log('  Samsung TV Learning System - Training & Benchmark Suite');
  console.log('═══════════════════════════════════════════════════════════════\n');

  // Initialize learning system
  const learner = new PreferenceLearningSystem({
    learningRate: 0.1,
    discountFactor: 0.95,
    explorationRate: 0.3,
    minExploration: 0.05,
    explorationDecay: 0.995,
    memorySize: 1000,
    batchSize: 32,
  });

  // Add content library
  console.log('📚 Loading content library...');
  learner.addContents(SAMPLE_CONTENT);
  console.log(`   Loaded ${SAMPLE_CONTENT.length} content items\n`);

  // Run embedding benchmarks
  console.log('⚡ Running Embedding Benchmarks...\n');

  const embBench = benchmarkEmbeddings(1000);
  console.log(`   Content Embedding Generation:`);
  console.log(`   ├─ ${embBench.opsPerSec.toLocaleString()} ops/sec`);
  console.log(`   └─ ${embBench.avgTimeMs.toFixed(4)}ms avg\n`);

  const simBench = benchmarkSimilarity(10000);
  console.log(`   Cosine Similarity (WASM-optimized):`);
  console.log(`   ├─ ${simBench.opsPerSec.toLocaleString()} ops/sec`);
  console.log(`   └─ ${simBench.avgTimeMs.toFixed(6)}ms avg\n`);

  const batchBench = benchmarkBatchSimilarity(100);
  console.log(`   Batch Similarity (Top-10 from ${SAMPLE_CONTENT.length}):`);
  console.log(`   ├─ ${batchBench.opsPerSec.toLocaleString()} ops/sec`);
  console.log(`   └─ ${batchBench.avgTimeMs.toFixed(4)}ms avg\n`);

  const cacheBench = benchmarkCache(5000);
  console.log(`   Embedding Cache Performance:`);
  console.log(`   ├─ ${(cacheBench.hitRate * 100).toFixed(1)}% hit rate`);
  console.log(`   ├─ ${cacheBench.hitTimeMs.toFixed(6)}ms hit time`);
  console.log(`   └─ ${cacheBench.missTimeMs.toFixed(4)}ms miss time\n`);

  // Training simulation
  console.log('🧠 Training Q-Learning System...\n');

  const numEpisodes = 500;
  const sessionsPerEpisode = 10;
  const rewardHistory: number[] = [];
  const explorationHistory: number[] = [];

  const trainingStart = performance.now();

  for (let episode = 0; episode < numEpisodes; episode++) {
    const profile = USER_PROFILES[episode % USER_PROFILES.length];
    let episodeReward = 0;

    for (let s = 0; s < sessionsPerEpisode; s++) {
      // Get recommendation
      const state = learner.getCurrentState();
      const action = learner.selectAction(state);

      // Simulate user selecting from recommendations
      const recs = learner.getRecommendations(5);
      const selectedContent = recs.length > 0
        ? SAMPLE_CONTENT.find(c => c.id === recs[0].contentId) || SAMPLE_CONTENT[Math.floor(Math.random() * SAMPLE_CONTENT.length)]
        : SAMPLE_CONTENT[Math.floor(Math.random() * SAMPLE_CONTENT.length)];

      // Generate viewing session
      const session = generateSession(selectedContent, profile);

      // Record session (this updates Q-values)
      learner.recordSession(session, action);
      episodeReward += learner['calculateReward'](session);
    }

    rewardHistory.push(episodeReward / sessionsPerEpisode);
    explorationHistory.push(learner.getStats().explorationRate);

    // Experience replay periodically
    if (episode % 10 === 0) {
      learner.experienceReplay(32);
    }

    // Progress update
    if ((episode + 1) % 100 === 0) {
      const avgReward = rewardHistory.slice(-100).reduce((a, b) => a + b, 0) / 100;
      console.log(`   Episode ${episode + 1}/${numEpisodes} | Avg Reward: ${avgReward.toFixed(3)} | ε: ${learner.getStats().explorationRate.toFixed(3)}`);
    }
  }

  const trainingTime = performance.now() - trainingStart;

  // Final statistics
  const stats = learner.getStats();
  const prefs = learner.getPreferences();

  console.log('\n📊 Training Results:\n');
  console.log(`   Total Training Time: ${(trainingTime / 1000).toFixed(2)}s`);
  console.log(`   Sessions Processed: ${stats.totalSessions}`);
  console.log(`   Patterns Learned: ${stats.totalPatterns}`);
  console.log(`   Final Exploration Rate: ${(stats.explorationRate * 100).toFixed(1)}%`);
  console.log(`   Average Reward: ${stats.avgReward.toFixed(3)}`);

  // Reward improvement
  const earlyReward = rewardHistory.slice(0, 50).reduce((a, b) => a + b, 0) / 50;
  const lateReward = rewardHistory.slice(-50).reduce((a, b) => a + b, 0) / 50;
  const improvement = ((lateReward - earlyReward) / earlyReward * 100);

  console.log(`\n   Reward Improvement: ${improvement > 0 ? '+' : ''}${improvement.toFixed(1)}%`);
  console.log(`   ├─ Early (first 50): ${earlyReward.toFixed(3)}`);
  console.log(`   └─ Late (last 50): ${lateReward.toFixed(3)}`);

  // Top actions
  console.log('\n   Top Actions by Reward:');
  stats.topActions.slice(0, 5).forEach((a, i) => {
    console.log(`   ${i + 1}. ${a.action}: ${a.avgReward.toFixed(3)} avg (${a.count} uses)`);
  });

  // Learned preferences
  console.log('\n   Learned Preferences:');
  console.log(`   ├─ Favorite Genres: ${prefs.favoriteGenres.slice(0, 5).join(', ')}`);
  console.log(`   └─ Favorite Types: ${prefs.favoriteTypes.join(', ')}`);

  // Test recommendations
  console.log('\n🎯 Testing Recommendations:\n');
  const testRecs = learner.getRecommendations(5);
  testRecs.forEach((rec, i) => {
    console.log(`   ${i + 1}. ${rec.title} (${rec.type})`);
    console.log(`      Score: ${rec.score.toFixed(3)} | ${rec.reason}`);
  });

  // Export model size
  const model = learner.exportModel();
  const modelJson = JSON.stringify(model);
  console.log(`\n💾 Model Size: ${(modelJson.length / 1024).toFixed(1)} KB`);

  // Summary
  console.log('\n═══════════════════════════════════════════════════════════════');
  console.log('  Benchmark Summary');
  console.log('═══════════════════════════════════════════════════════════════');
  console.log(`
  Embedding Generation:  ${embBench.opsPerSec.toLocaleString()} ops/sec
  Cosine Similarity:     ${simBench.opsPerSec.toLocaleString()} ops/sec
  Batch Search (Top-10): ${batchBench.opsPerSec.toLocaleString()} ops/sec
  Cache Hit Rate:        ${(cacheBench.hitRate * 100).toFixed(1)}%

  Training Episodes:     ${numEpisodes}
  Training Time:         ${(trainingTime / 1000).toFixed(2)}s
  Reward Improvement:    ${improvement > 0 ? '+' : ''}${improvement.toFixed(1)}%
  Q-Table Entries:       ${model.qTable.length}
  Patterns Stored:       ${model.patterns.length}
  `);
  console.log('═══════════════════════════════════════════════════════════════\n');
}

// Run
trainAndBenchmark().catch(console.error);
