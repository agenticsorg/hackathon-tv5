/**
 * Generate Embeddings for Knowledge Graph Movies using Google Gemini
 *
 * This script reads the exported JSON, generates embeddings via Google's
 * Gemini embedding model, and saves the result with embeddings included.
 *
 * Usage:
 *   GOOGLE_GEMINI_API_KEY=xxx node scripts/generate-embeddings.js
 *
 * Or set the key in your environment first.
 */

const fs = require('fs');
const path = require('path');

// Configuration
const INPUT_FILE = path.join(__dirname, '../mondweep/media-hackathion-knowledge-graph-full-export-2025-12-08.json');
const OUTPUT_FILE = path.join(__dirname, '../mondweep/knowledge-graph-with-embeddings.json');
const GEMINI_MODEL = 'text-embedding-004'; // Google's latest embedding model
const BATCH_SIZE = 10; // Process 10 movies at a time (Gemini has different rate limits)
const DELAY_MS = 1000; // Delay between batches to avoid rate limits

async function generateEmbedding(text, apiKey) {
  const url = `https://generativelanguage.googleapis.com/v1beta/models/${GEMINI_MODEL}:embedContent?key=${apiKey}`;

  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      model: `models/${GEMINI_MODEL}`,
      content: {
        parts: [{ text }]
      },
      taskType: 'RETRIEVAL_DOCUMENT',
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Gemini API error: ${response.status} - ${error}`);
  }

  const data = await response.json();
  return data.embedding.values;
}

async function generateBatchEmbeddings(texts, apiKey) {
  const url = `https://generativelanguage.googleapis.com/v1beta/models/${GEMINI_MODEL}:batchEmbedContents?key=${apiKey}`;

  const requests = texts.map(text => ({
    model: `models/${GEMINI_MODEL}`,
    content: {
      parts: [{ text }]
    },
    taskType: 'RETRIEVAL_DOCUMENT',
  }));

  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ requests }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Gemini API error: ${response.status} - ${error}`);
  }

  const data = await response.json();
  return data.embeddings.map(e => e.values);
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
  // Try environment variable first, then fall back to .env file
  let apiKey = process.env.GOOGLE_GEMINI_API_KEY;

  if (!apiKey) {
    // Try reading from .env file
    try {
      const envPath = path.join(__dirname, '../mondweep/.env');
      const envContent = fs.readFileSync(envPath, 'utf8');
      const match = envContent.match(/GOOGLE_GEMINI_API_KEY=(.+)/);
      if (match) {
        apiKey = match[1].trim();
      }
    } catch (e) {
      // Ignore file read errors
    }
  }

  if (!apiKey) {
    console.error('Error: GOOGLE_GEMINI_API_KEY is required');
    console.error('Usage: GOOGLE_GEMINI_API_KEY=xxx node scripts/generate-embeddings.js');
    console.error('Or set it in mondweep/.env file');
    process.exit(1);
  }

  console.log('Using Gemini embedding model:', GEMINI_MODEL);
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
        movie.embeddingModel = GEMINI_MODEL;
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

      // Try individual processing for failed batch
      for (const movie of batch) {
        try {
          const text = createEmbeddingText(movie);
          movie.embedding = await generateEmbedding(text, apiKey);
          movie.embeddingModel = GEMINI_MODEL;
          processed++;
          console.log(`    ✓ Recovered: ${movie.title}`);
          await sleep(200);
        } catch (e) {
          console.error(`    ✗ Failed: ${movie.title} - ${e.message}`);
          failed++;
        }
      }
    }
  }

  // Update metadata
  data.includesEmbeddings = true;
  data.embeddingModel = GEMINI_MODEL;
  data.embeddingDimensions = movies[0]?.embedding?.length || 768;
  data.embeddingsGeneratedAt = new Date().toISOString();

  // Save output
  console.log('\nSaving to:', OUTPUT_FILE);
  fs.writeFileSync(OUTPUT_FILE, JSON.stringify(data));

  const fileSize = (fs.statSync(OUTPUT_FILE).size / 1024 / 1024).toFixed(2);
  const totalTime = ((Date.now() - startTime) / 1000).toFixed(1);

  console.log(`\n✅ Complete!`);
  console.log(`   Processed: ${processed} movies`);
  console.log(`   Failed: ${failed} movies`);
  console.log(`   Time: ${totalTime}s`);
  console.log(`   Output: ${OUTPUT_FILE} (${fileSize} MB)`);
  console.log(`   Embedding dimensions: ${data.embeddingDimensions}`);
}

main().catch(console.error);
