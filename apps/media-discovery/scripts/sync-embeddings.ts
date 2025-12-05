#!/usr/bin/env npx tsx
/**
 * Content Embedding Sync Pipeline
 *
 * This script fetches content from TMDB and generates embeddings for vector search.
 * Uses the ruvector npm package for embedded vector database storage.
 * Run periodically to keep embeddings up-to-date.
 *
 * Usage:
 *   npx tsx scripts/sync-embeddings.ts [options]
 *
 * Options:
 *   --full      Sync all content (movies + TV shows)
 *   --movies    Sync only movies
 *   --tv        Sync only TV shows
 *   --trending  Sync only trending content
 *   --limit N   Limit to N items per category
 */

// Load environment variables from .env.local
import { config } from 'dotenv';
config({ path: '.env.local' });

import { TMDB } from 'tmdb-ts';
import { VectorDB } from 'ruvector';

// Configuration
const TMDB_TOKEN = process.env.NEXT_PUBLIC_TMDB_ACCESS_TOKEN;
const OPENAI_API_KEY = process.env.OPENAI_API_KEY;
const STORAGE_PATH = process.env.RUVECTOR_STORAGE_PATH || './data/media-vectors.db';
const EMBEDDING_DIMENSIONS = 768;

if (!TMDB_TOKEN) {
  console.error('Error: NEXT_PUBLIC_TMDB_ACCESS_TOKEN is required');
  process.exit(1);
}

const tmdb = new TMDB(TMDB_TOKEN);

interface ContentItem {
  id: number;
  title: string;
  overview: string;
  mediaType: 'movie' | 'tv';
  genreIds: number[];
  voteAverage?: number;
  releaseDate?: string;
  posterPath?: string | null;
}

interface EmbeddingResult {
  item: ContentItem;
  embedding: Float32Array;
  text: string;
}

// Parse command line arguments
function parseArgs(): { mode: string; limit: number } {
  const args = process.argv.slice(2);
  let mode = 'trending';
  let limit = 100;

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--full':
        mode = 'full';
        break;
      case '--movies':
        mode = 'movies';
        break;
      case '--tv':
        mode = 'tv';
        break;
      case '--trending':
        mode = 'trending';
        break;
      case '--limit':
        limit = parseInt(args[++i], 10) || 100;
        break;
    }
  }

  return { mode, limit };
}

// Fetch content from TMDB
async function fetchContent(mode: string, limit: number): Promise<ContentItem[]> {
  const items: ContentItem[] = [];
  const pagesNeeded = Math.ceil(limit / 20);

  console.log(`Fetching ${mode} content (limit: ${limit})...`);

  if (mode === 'trending' || mode === 'full') {
    for (let page = 1; page <= pagesNeeded && items.length < limit; page++) {
      try {
        const trending = await tmdb.trending.trending('all', 'week');
        for (const item of trending.results) {
          if (items.length >= limit) break;
          if (item.media_type === 'movie' || item.media_type === 'tv') {
            items.push({
              id: item.id,
              title: item.media_type === 'tv' ? (item as any).name : (item as any).title,
              overview: item.overview || '',
              mediaType: item.media_type,
              genreIds: item.genre_ids || [],
              voteAverage: item.vote_average,
              releaseDate: item.media_type === 'tv' ? (item as any).first_air_date : (item as any).release_date,
              posterPath: item.poster_path,
            });
          }
        }
      } catch (error) {
        console.error(`Error fetching trending page ${page}:`, error);
      }
    }
  }

  if (mode === 'movies' || mode === 'full') {
    for (let page = 1; page <= pagesNeeded && items.length < limit; page++) {
      try {
        const movies = await tmdb.movies.popular({ page });
        for (const movie of movies.results) {
          if (items.length >= limit) break;
          if (!items.some(i => i.id === movie.id && i.mediaType === 'movie')) {
            items.push({
              id: movie.id,
              title: movie.title,
              overview: movie.overview || '',
              mediaType: 'movie',
              genreIds: movie.genre_ids || [],
              voteAverage: movie.vote_average,
              releaseDate: movie.release_date,
              posterPath: movie.poster_path,
            });
          }
        }
      } catch (error) {
        console.error(`Error fetching movies page ${page}:`, error);
      }
    }
  }

  if (mode === 'tv' || mode === 'full') {
    for (let page = 1; page <= pagesNeeded && items.length < limit; page++) {
      try {
        const shows = await tmdb.tvShows.popular({ page });
        for (const show of shows.results) {
          if (items.length >= limit) break;
          if (!items.some(i => i.id === show.id && i.mediaType === 'tv')) {
            items.push({
              id: show.id,
              title: show.name,
              overview: show.overview || '',
              mediaType: 'tv',
              genreIds: show.genre_ids || [],
              voteAverage: show.vote_average,
              releaseDate: show.first_air_date,
              posterPath: show.poster_path,
            });
          }
        }
      } catch (error) {
        console.error(`Error fetching TV shows page ${page}:`, error);
      }
    }
  }

  console.log(`Fetched ${items.length} content items`);
  return items.slice(0, limit);
}

// Generate text for embedding
function generateEmbeddingText(item: ContentItem): string {
  const parts = [
    item.title,
    item.overview,
    `Type: ${item.mediaType}`,
  ];

  return parts.filter(Boolean).join('. ');
}

// Generate mock embedding for testing (when no OpenAI key)
function generateMockEmbedding(text: string): Float32Array {
  const embedding = new Float32Array(EMBEDDING_DIMENSIONS);
  for (let i = 0; i < text.length; i++) {
    embedding[(text.charCodeAt(i) + i) % EMBEDDING_DIMENSIONS] += 0.01;
  }
  // Normalize
  let mag = 0;
  for (let i = 0; i < embedding.length; i++) {
    mag += embedding[i] * embedding[i];
  }
  mag = Math.sqrt(mag);
  if (mag > 0) {
    for (let i = 0; i < embedding.length; i++) {
      embedding[i] /= mag;
    }
  }
  return embedding;
}

// Generate embedding using OpenAI API
async function generateEmbedding(text: string): Promise<Float32Array | null> {
  if (!OPENAI_API_KEY) {
    // Return mock embedding for testing
    return generateMockEmbedding(text);
  }

  try {
    const response = await fetch('https://api.openai.com/v1/embeddings', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${OPENAI_API_KEY}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model: 'text-embedding-3-small',
        input: text,
        dimensions: EMBEDDING_DIMENSIONS,
      }),
    });

    if (!response.ok) {
      throw new Error(`OpenAI API error: ${response.status}`);
    }

    const data = await response.json();
    return new Float32Array(data.data[0].embedding);
  } catch (error) {
    console.error('Embedding generation error:', error);
    return null;
  }
}

// Store embeddings in RuVector
async function storeEmbeddings(db: InstanceType<typeof VectorDB>, embeddings: EmbeddingResult[]): Promise<number> {
  let stored = 0;

  for (const { item, embedding } of embeddings) {
    const id = `${item.mediaType}-${item.id}`;

    try {
      await db.insert({
        id,
        vector: embedding,
        metadata: {
          contentId: item.id,
          mediaType: item.mediaType,
          title: item.title,
          overview: item.overview.slice(0, 500),
          genreIds: item.genreIds,
          voteAverage: item.voteAverage || 0,
          releaseDate: item.releaseDate || '',
          posterPath: item.posterPath || null,
        },
      });
      stored++;
    } catch (error) {
      console.error(`Error storing ${id}:`, error);
    }
  }

  return stored;
}

// Main sync function
async function main() {
  const { mode, limit } = parseArgs();
  console.log(`\n=== Content Embedding Sync ===`);
  console.log(`Mode: ${mode}`);
  console.log(`Limit: ${limit}`);
  console.log(`Storage: ${STORAGE_PATH}`);
  console.log(`OpenAI: ${OPENAI_API_KEY ? 'Configured' : 'Not configured (using mock embeddings)'}\n`);

  // Initialize vector database
  console.log('Initializing RuVector database...');
  const db = new VectorDB({
    dimensions: EMBEDDING_DIMENSIONS,
    maxElements: 100000,
    storagePath: STORAGE_PATH,
  });

  const existingCount = await db.len();
  console.log(`Existing vectors: ${existingCount}\n`);

  // Step 1: Fetch content
  const content = await fetchContent(mode, limit);
  if (content.length === 0) {
    console.log('No content to process');
    return;
  }

  // Step 2: Generate embeddings
  console.log('\nGenerating embeddings...');
  const embeddings: EmbeddingResult[] = [];
  let processed = 0;

  for (const item of content) {
    const text = generateEmbeddingText(item);
    const embedding = await generateEmbedding(text);

    if (embedding) {
      embeddings.push({
        item,
        embedding,
        text,
      });
    }

    processed++;
    if (processed % 10 === 0) {
      console.log(`  Processed ${processed}/${content.length} items`);
    }

    // Rate limiting for OpenAI API
    if (OPENAI_API_KEY) {
      await new Promise(resolve => setTimeout(resolve, 50));
    }
  }

  console.log(`Generated ${embeddings.length} embeddings`);

  // Step 3: Store in RuVector
  console.log('\nStoring embeddings in RuVector...');
  const stored = await storeEmbeddings(db, embeddings);
  console.log(`Stored ${stored} embeddings`);

  // Get final count
  const finalCount = await db.len();

  // Summary
  console.log('\n=== Sync Complete ===');
  console.log(`Content fetched: ${content.length}`);
  console.log(`Embeddings generated: ${embeddings.length}`);
  console.log(`Embeddings stored: ${stored}`);
  console.log(`Total vectors in database: ${finalCount}`);
}

main().catch(console.error);
