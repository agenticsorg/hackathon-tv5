/**
 * Knowledge Graph Semantic Search API (Pinecone Version)
 * POST /.netlify/functions/kg-search
 * Body: { "query": "movies about redemption", "limit": 10 }
 */

// const { Pinecone } = require('@pinecone-database/pinecone'); // Removed to avoid dependency error, using fetch instead

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
    // Parse request
    let body = {};
    if (event.body) {
      try {
        body = JSON.parse(event.body);
      } catch (e) {
        body.query = event.queryStringParameters?.query;
      }
    }
    const query = body.query || event.queryStringParameters?.query;
    const limit = parseInt(body.limit || event.queryStringParameters?.limit) || 20;

    if (!query) {
      return {
        statusCode: 400,
        headers,
        body: JSON.stringify({ error: 'Search query is required' }),
      };
    }

    // Config
    const PINECONE_API_KEY = process.env.PINECONE_API_KEY;
    const PINECONE_HOST = process.env.PINECONE_HOST; // E.g. https://index-host.svc.pinecone.io
    const GEMINI_API_KEY = process.env.GOOGLE_GEMINI_API_KEY || process.env.GEMINI_API_KEY;

    // Check configuration
    if (!PINECONE_API_KEY || !PINECONE_HOST || !GEMINI_API_KEY) {
      console.error("Missing Configuration: ", {
        hasPineconeKey: !!PINECONE_API_KEY,
        hasPineconeHost: !!PINECONE_HOST,
        hasGeminiKey: !!GEMINI_API_KEY
      });
      return {
        statusCode: 500,
        headers,
        body: JSON.stringify({
          error: 'Server configuration error. Missing API Keys.',
          details: 'Please ensure PINECONE_API_KEY, PINECONE_HOST, and GOOGLE_GEMINI_API_KEY are set.'
        }),
      };
    }

    // 1. Generate Embedding
    const model = 'text-embedding-004';
    const embedUrl = `https://generativelanguage.googleapis.com/v1beta/models/${model}:embedContent?key=${GEMINI_API_KEY}`;

    const embedResponse = await fetch(embedUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: `models/${model}`,
        content: { parts: [{ text: query }] },
        taskType: 'RETRIEVAL_QUERY',
      }),
    });

    if (!embedResponse.ok) {
      throw new Error(`Gemini Embedding Failed: ${embedResponse.status} ${await embedResponse.text()}`);
    }

    const embedData = await embedResponse.json();
    const vector = embedData.embedding.values;

    // 2. Search Pinecone (REST API)
    // Host should be the full URL, e.g., https://media-knowledge-graph-....pinecone.io
    // Endpoint: /vectors/query

    // Ensure host starts with https://
    const hostUrl = PINECONE_HOST.startsWith('http') ? PINECONE_HOST : `https://${PINECONE_HOST}`;
    const searchUrl = `${hostUrl}/query`;

    const searchResponse = await fetch(searchUrl, {
      method: 'POST',
      headers: {
        'Api-Key': PINECONE_API_KEY,
        'Content-Type': 'application/json',
        'X-Pinecone-API-Version': '2024-07'
      },
      body: JSON.stringify({
        vector: vector,
        topK: limit,
        includeMetadata: true,
        includeValues: false
      })
    });

    if (!searchResponse.ok) {
      throw new Error(`Pinecone Search Failed: ${searchResponse.status} ${await searchResponse.text()}`);
    }

    const searchData = await searchResponse.json();
    const matches = searchData.matches || [];

    // 3. Map to Result Format
    const results = matches.map(match => {
      const md = match.metadata || {};
      return {
        id: match.id,
        title: md.title || 'Unknown',
        overview: md.overview || '',
        posterPath: md.poster_path || null, // Map from snake_case to camelCase
        releaseDate: md.year ? `${md.year}-01-01` : null, // Approx
        year: md.year ? parseInt(md.year) : null,
        voteAverage: md.vote_average || 0,
        popularity: md.popularity || 0,
        genres: md.genres ? md.genres.split(',').map(g => g.trim()) : [],
        similarity: match.score,
        platformReadiness: { netflix: true, amazon: true, fast: true } // Mock status
      };
    });

    return {
      statusCode: 200,
      headers,
      body: JSON.stringify({
        query,
        results,
        total: results.length,
        searchType: 'semantic-pinecone',
        embeddingsAvailable: true
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
