/**
 * Knowledge Graph Semantic Search API
 * POST /.netlify/functions/kg-search
 * Body: { "query": "movies about redemption", "limit": 10 }
 *
 * Supports both:
 * - Semantic search (if embeddings are available and OPENAI_API_KEY is set)
 * - Text-based fallback search
 */

const fs = require('fs');
const path = require('path');

let cachedData = null;
let hasEmbeddings = false;

function loadData() {
  if (cachedData) return cachedData;

  // Try to load embeddings file first, fall back to regular export
  const embeddingsPath = path.join(__dirname, '../../mondweep/knowledge-graph-with-embeddings.json');
  const regularPath = path.join(__dirname, '../../mondweep/media-hackathion-knowledge-graph-full-export-2025-12-08.json');

  let dataPath = regularPath;
  if (fs.existsSync(embeddingsPath)) {
    dataPath = embeddingsPath;
    hasEmbeddings = true;
    console.log('Using embeddings file for semantic search');
  } else {
    console.log('Embeddings file not found, using text search');
  }

  const rawData = fs.readFileSync(dataPath, 'utf8');
  cachedData = JSON.parse(rawData);

  // Check if data actually has embeddings
  if (cachedData.data.movies[0]?.embedding) {
    hasEmbeddings = true;
  }

  return cachedData;
}

// Cosine similarity between two vectors
function cosineSimilarity(a, b) {
  if (!a || !b || a.length !== b.length) return 0;

  let dotProduct = 0;
  let normA = 0;
  let normB = 0;

  for (let i = 0; i < a.length; i++) {
    dotProduct += a[i] * b[i];
    normA += a[i] * a[i];
    normB += b[i] * b[i];
  }

  normA = Math.sqrt(normA);
  normB = Math.sqrt(normB);

  if (normA === 0 || normB === 0) return 0;
  return dotProduct / (normA * normB);
}

// Get embedding for query from OpenAI
async function getQueryEmbedding(query, apiKey) {
  const response = await fetch('https://api.openai.com/v1/embeddings', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${apiKey}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      model: 'text-embedding-3-small',
      input: query,
    }),
  });

  if (!response.ok) {
    throw new Error(`OpenAI API error: ${response.status}`);
  }

  const data = await response.json();
  return data.data[0].embedding;
}

// Semantic search using embeddings
async function semanticSearch(movies, query, limit, apiKey) {
  // Get query embedding
  const queryEmbedding = await getQueryEmbedding(query, apiKey);

  // Calculate similarity for all movies with embeddings
  const scored = movies
    .filter(m => m.embedding)
    .map(movie => ({
      ...movie,
      similarity: cosineSimilarity(queryEmbedding, movie.embedding),
    }))
    .sort((a, b) => b.similarity - a.similarity)
    .slice(0, limit);

  return scored.map(movie => ({
    movie: {
      id: movie.id,
      title: movie.title,
      overview: movie.overview,
      posterPath: movie.posterPath,
      releaseDate: movie.releaseDate,
      voteAverage: movie.voteAverage,
      popularity: movie.popularity,
      runtime: movie.runtime,
      platformReadiness: movie.platformReadiness,
    },
    similarity: movie.similarity,
  }));
}

// Text-based fallback search
function textSearch(movies, query, limit) {
  const queryTerms = query.toLowerCase().split(/\s+/).filter(t => t.length > 1);

  const scored = movies.map(movie => {
    let score = 0;
    const title = (movie.title || '').toLowerCase();
    const overview = (movie.overview || '').toLowerCase();
    const tagline = (movie.tagline || '').toLowerCase();

    queryTerms.forEach(term => {
      if (title.includes(term)) {
        score += 10;
        if (title.startsWith(term)) score += 5;
      }
      if (overview.includes(term)) {
        score += 3;
        const matches = overview.split(term).length - 1;
        score += Math.min(matches, 3);
      }
      if (tagline.includes(term)) {
        score += 2;
      }
    });

    if (movie.popularity) {
      score += Math.log10(movie.popularity + 1) * 0.5;
    }

    return { ...movie, searchScore: score };
  })
    .filter(m => m.searchScore > 0)
    .sort((a, b) => b.searchScore - a.searchScore)
    .slice(0, limit);

  return scored.map(movie => ({
    movie: {
      id: movie.id,
      title: movie.title,
      overview: movie.overview,
      posterPath: movie.posterPath,
      releaseDate: movie.releaseDate,
      voteAverage: movie.voteAverage,
      popularity: movie.popularity,
      runtime: movie.runtime,
      platformReadiness: movie.platformReadiness,
    },
    similarity: Math.min(movie.searchScore / 20, 1),
  }));
}

exports.handler = async (event, context) => {
  const headers = {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Headers': 'Content-Type',
    'Content-Type': 'application/json',
  };

  if (event.httpMethod === 'OPTIONS') {
    return { statusCode: 200, headers, body: '' };
  }

  try {
    // Parse request body
    let body = {};
    if (event.body) {
      try {
        body = JSON.parse(event.body);
      } catch (e) {
        body.query = event.queryStringParameters?.query;
      }
    }

    const query = body.query || event.queryStringParameters?.query;
    const limit = parseInt(body.limit || event.queryStringParameters?.limit) || 10;

    if (!query) {
      return {
        statusCode: 400,
        headers,
        body: JSON.stringify({ error: 'Search query is required' }),
      };
    }

    const data = loadData();
    const movies = data.data.movies || [];

    // Check if we can do semantic search
    const apiKey = process.env.OPENAI_API_KEY;
    const canDoSemantic = hasEmbeddings && apiKey;

    let results;
    let searchType;

    if (canDoSemantic) {
      try {
        results = await semanticSearch(movies, query, limit, apiKey);
        searchType = 'semantic';
      } catch (error) {
        console.error('Semantic search failed, falling back to text:', error.message);
        results = textSearch(movies, query, limit);
        searchType = 'text (fallback)';
      }
    } else {
      results = textSearch(movies, query, limit);
      searchType = 'text';
    }

    return {
      statusCode: 200,
      headers,
      body: JSON.stringify({
        query,
        results,
        total: results.length,
        searchType,
        embeddingsAvailable: hasEmbeddings,
      }),
    };
  } catch (error) {
    console.error('Search error:', error);
    return {
      statusCode: 500,
      headers,
      body: JSON.stringify({ error: 'Search failed', details: error.message }),
    };
  }
};
