/**
 * Generate Embeddings for Knowledge Graph Movies
 *
 * This script reads the exported JSON, generates embeddings via OpenAI,
 * and saves the result with embeddings included.
 *
 * Usage:
 *   OPENAI_API_KEY=sk-xxx node scripts/generate-embeddings.js
 *
 * Or set the key in your environment first.
 */

const fs = require('fs');
const path = require('path');

// Configuration
const INPUT_FILE = path.join(__dirname, '../mondweep/media-hackathion-knowledge-graph-full-export-2025-12-08.json');
const OUTPUT_FILE = path.join(__dirname, '../mondweep/knowledge-graph-with-embeddings.json');
const OPENAI_MODEL = 'text-embedding-3-small'; // 1536 dimensions, cheapest
const BATCH_SIZE = 20; // Process 20 movies at a time
const DELAY_MS = 500; // Delay between batches to avoid rate limits

async function generateEmbedding(text, apiKey) {
  const response = await fetch('https://api.openai.com/v1/embeddings', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${apiKey}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      model: OPENAI_MODEL,
      input: text,
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`OpenAI API error: ${response.status} - ${error}`);
  }

  const data = await response.json();
  return data.data[0].embedding;
}

async function generateBatchEmbeddings(texts, apiKey) {
  const response = await fetch('https://api.openai.com/v1/embeddings', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${apiKey}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      model: OPENAI_MODEL,
      input: texts,
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`OpenAI API error: ${response.status} - ${error}`);
  }

  const data = await response.json();
  // Sort by index to maintain order
  return data.data.sort((a, b) => a.index - b.index).map(d => d.embedding);
}

function createEmbeddingText(movie) {
  // Combine title, tagline, and overview for richer embedding
  const parts = [movie.title];
  if (movie.tagline) parts.push(movie.tagline);
  if (movie.overview) parts.push(movie.overview);
  return parts.join('. ').slice(0, 8000); // Limit to avoid token limits
}

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
  const apiKey = process.env.OPENAI_API_KEY;

  if (!apiKey) {
    console.error('Error: OPENAI_API_KEY environment variable is required');
    console.error('Usage: OPENAI_API_KEY=sk-xxx node scripts/generate-embeddings.js');
    process.exit(1);
  }

  console.log('Loading data from:', INPUT_FILE);
  const rawData = fs.readFileSync(INPUT_FILE, 'utf8');
  const data = JSON.parse(rawData);

  const movies = data.data.movies;
  console.log(`Found ${movies.length} movies to process`);

  let processed = 0;
  let failed = 0;
  const startTime = Date.now();

  // Process in batches
  for (let i = 0; i < movies.length; i += BATCH_SIZE) {
    const batch = movies.slice(i, i + BATCH_SIZE);
    const texts = batch.map(m => createEmbeddingText(m));

    try {
      console.log(`Processing batch ${Math.floor(i / BATCH_SIZE) + 1}/${Math.ceil(movies.length / BATCH_SIZE)} (movies ${i + 1}-${Math.min(i + BATCH_SIZE, movies.length)})`);

      const embeddings = await generateBatchEmbeddings(texts, apiKey);

      // Assign embeddings to movies
      batch.forEach((movie, idx) => {
        movie.embedding = embeddings[idx];
        movie.embeddingModel = OPENAI_MODEL;
      });

      processed += batch.length;

      // Progress update
      const elapsed = (Date.now() - startTime) / 1000;
      const rate = processed / elapsed;
      const remaining = (movies.length - processed) / rate;
      console.log(`  ✓ ${processed}/${movies.length} done (${rate.toFixed(1)}/sec, ~${remaining.toFixed(0)}s remaining)`);

      // Rate limit delay
      if (i + BATCH_SIZE < movies.length) {
        await sleep(DELAY_MS);
      }
    } catch (error) {
      console.error(`  ✗ Batch failed:`, error.message);
      failed += batch.length;

      // Try individual processing for failed batch
      for (const movie of batch) {
        try {
          const text = createEmbeddingText(movie);
          movie.embedding = await generateEmbedding(text, apiKey);
          movie.embeddingModel = OPENAI_MODEL;
          processed++;
          failed--;
          await sleep(100);
        } catch (e) {
          console.error(`    Failed to process movie ${movie.id}: ${e.message}`);
        }
      }
    }
  }

  // Update metadata
  data.includesEmbeddings = true;
  data.embeddingModel = OPENAI_MODEL;
  data.embeddingDimensions = 1536;
  data.embeddingsGeneratedAt = new Date().toISOString();

  // Save output
  console.log('\nSaving to:', OUTPUT_FILE);
  fs.writeFileSync(OUTPUT_FILE, JSON.stringify(data, null, 2));

  const totalTime = ((Date.now() - startTime) / 1000).toFixed(1);
  console.log(`\n✅ Complete!`);
  console.log(`   Processed: ${processed} movies`);
  console.log(`   Failed: ${failed} movies`);
  console.log(`   Time: ${totalTime}s`);
  console.log(`   Output: ${OUTPUT_FILE}`);
}

main().catch(console.error);
