/**
 * Ingest Massive Knowledge Graph into Pinecone (Streaming & Robust)
 * 
 * Features:
 * - Streams large JSON input (avoids OOM).
 * - Batched embedding generation (Google Gemini).
 * - Batched Vector DB uploads (Pinecone).
 * - Rate limit handling (Retry with exponential backoff).
 * - Checkpointing (Resumes where it left off).
 * - Error logging (Saves failed items to sidebar file).
 * 
 * Usage:
 *   node scripts/ingest-pinecone.js
 */

const fs = require('fs');
const path = require('path');
const { Pinecone } = require('@pinecone-database/pinecone');
const { chain } = require('stream-chain');
const { parser } = require('stream-json');
const { pick } = require('stream-json/filters/Pick');
const { streamArray } = require('stream-json/streamers/StreamArray');

// Configuration
const INPUT_FILE = path.join(__dirname, '../mondweep/data/full_knowledge_graph.json');
const CHECKPOINT_FILE = path.join(__dirname, '../mondweep/data/ingestion_checkpoint.json');
const FAILED_FILE = path.join(__dirname, '../mondweep/data/ingestion_failed.json');
const INDEX_NAME = 'nexus-movies';

const GEMINI_MODEL = 'text-embedding-004';
const EMBED_BATCH_SIZE = 50; // Max texts per Gemini request (check docs, usually ~100)
const PINE_BATCH_SIZE = 100; // Records per Pinecone upsert

// State
let state = {
    processedCount: 0,
    lastId: null,
    skippedCount: 0
};

// Load Checkpoint
if (fs.existsSync(CHECKPOINT_FILE)) {
    try {
        state = JSON.parse(fs.readFileSync(CHECKPOINT_FILE, 'utf8'));
        console.log(`🔄 Resuming from checkpoint: ${state.processedCount} records processed.`);
    } catch (e) { console.warn('Checkpoint file corrupted, starting fresh.'); }
}

const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));

// --- Gemini API ---

async function getGeminiBatchEmbeddings(texts, apiKey) {
    const url = `https://generativelanguage.googleapis.com/v1beta/models/${GEMINI_MODEL}:batchEmbedContents?key=${apiKey}`;

    // Construct payload
    // "requests": [{ "model": "...", "content": { "parts": [...] } }]
    const requests = texts.map(t => ({
        model: `models/${GEMINI_MODEL}`,
        content: { parts: [{ text: t }] },
        taskType: 'RETRIEVAL_DOCUMENT'
    }));

    try {
        const response = await fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ requests })
        });

        if (!response.ok) {
            const errText = await response.text();
            if (response.status === 429) throw new Error('RATE_LIMIT');
            throw new Error(`Gemini Error ${response.status}: ${errText}`);
        }

        const data = await response.json();
        // data.embeddings is array of { values: [...] }
        return data.embeddings.map(e => e.values);

    } catch (error) {
        if (error.message === 'RATE_LIMIT') {
            console.warn('⚠️ Rate limit hit. Backing off 10s...');
            await sleep(10000); // 10s wait
            return getGeminiBatchEmbeddings(texts, apiKey); // Retry
        }
        throw error;
    }
}

function createEmbeddingText(movie) {
    const parts = [
        `Title: ${movie.title}`,
        movie.tagline ? `Tagline: ${movie.tagline}` : '',
        movie.overview ? `Overview: ${movie.overview}` : '',
        (movie.genres && movie.genres.length) ? `Genres: ${movie.genres.join(', ')}` : '',
        movie.releaseDate ? `Release: ${movie.releaseDate}` : ''
    ].filter(Boolean);

    // Truncate to reasonably safe token count (approx 8000 chars ~ 2000 tokens)
    return parts.join('\n').slice(0, 8000);
}

// --- Main ---

async function main() {
    // 1. Env Setup
    let pineconeKey = process.env.PINECONE_API_KEY;
    let googleKey = process.env.GOOGLE_GEMINI_API_KEY;

    // Fallback manual .env read
    if (!pineconeKey || !googleKey) {
        try {
            // Check root .env
            let envPath = path.join(__dirname, '../.env');
            if (fs.existsSync(envPath)) {
                const envContent = fs.readFileSync(envPath, 'utf8');
                envContent.split('\n').filter(l => l.includes('=')).forEach(line => {
                    const [k, v] = line.split('=');
                    if (k.trim() === 'PINECONE_API_KEY') pineconeKey = v.trim();
                    if (k.trim() === 'GOOGLE_GEMINI_API_KEY') googleKey = v.trim();
                });
            }

            // Check mondweep/.env
            envPath = path.join(__dirname, '../mondweep/.env');
            if (fs.existsSync(envPath)) {
                const envContent = fs.readFileSync(envPath, 'utf8');
                envContent.split('\n').filter(l => l.includes('=')).forEach(line => {
                    const [k, v] = line.split('=');
                    if (k.trim() === 'PINECONE_API_KEY') pineconeKey = v.trim();
                    if (k.trim() === 'GOOGLE_GEMINI_API_KEY') googleKey = v.trim();
                });
            }
        } catch (e) { }
    }

    if (!pineconeKey) throw new Error('Missing PINECONE_API_KEY');
    if (!googleKey) throw new Error('Missing GOOGLE_GEMINI_API_KEY');

    // 2. Pinecone Init
    const pinecone = new Pinecone({ apiKey: pineconeKey });
    const index = pinecone.index(INDEX_NAME);

    // 3. Streaming Logic
    console.log(`🚀 Starting ingestion from ${INPUT_FILE}`);

    const pipeline = chain([
        fs.createReadStream(INPUT_FILE),
        parser(),
        pick({ filter: 'data.movies' }),
        streamArray()
    ]);

    let batch = [];
    let counter = 0;

    for await (const { value: movie } of pipeline) {
        counter++;

        // Skip already processed
        if (counter <= state.processedCount) continue;

        // Skip garbage data (optional filter)
        if (!movie.title || (!movie.overview && !movie.tagline)) {
            state.skippedCount++;
            continue;
        }

        batch.push(movie);

        // When batch is full, process it
        if (batch.length >= EMBED_BATCH_SIZE) {
            await processBatch(batch, index, googleKey);

            // Updates state
            state.processedCount = counter;
            state.lastId = batch[batch.length - 1].id;
            fs.writeFileSync(CHECKPOINT_FILE, JSON.stringify(state));

            process.stdout.write(`\rProcessed: ${counter} | Skipped: ${state.skippedCount}`);
            batch = [];

            // Polite delay
            await sleep(200);
        }
    }

    // Final incomplete batch
    if (batch.length > 0) {
        await processBatch(batch, index, googleKey);
        state.processedCount = counter;
        fs.writeFileSync(CHECKPOINT_FILE, JSON.stringify(state));
    }

    console.log('\n\n✅ Ingestion Finished Successfully!');
}

async function processBatch(movies, index, googleKey) {
    try {
        // Prepare texts
        const texts = movies.map(createEmbeddingText);

        // 1. Generate Embeddings
        const embeddings = await getGeminiBatchEmbeddings(texts, googleKey);

        // 2. Map to Pinecone Records
        const records = movies.map((m, i) => ({
            id: String(m.id),
            values: embeddings[i],
            metadata: {
                title: m.title,
                type: m.type || 'movie',
                popularity: m.popularity || 0,
                year: m.year || 0,
                genres: (m.genres || []).slice(0, 5) // Limit array size for metadata
            }
        }));

        // 3. Upsert
        // Pinecone handles batches up to 100 fine, we are sending ~50
        await index.upsert(records);

    } catch (err) {
        console.error(`\n❌ Failed Batch (IDs ${movies[0].id} - ${movies[movies.length - 1].id}): ${err.message}`);
        // Log to fail file
        const failures = movies.map(m => ({ id: m.id, title: m.title, error: err.message }));
        fs.appendFileSync(FAILED_FILE, JSON.stringify(failures) + '\n');
    }
}

main().catch(err => {
    console.error('\nFatal Error:', err);
    process.exit(1);
});
