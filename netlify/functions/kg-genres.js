/**
 * Knowledge Graph Genres API
 * GET /.netlify/functions/kg-genres
 */

const fs = require('fs');
const path = require('path');

let cachedData = null;

function loadData() {
  if (cachedData) return cachedData;

  // Try embeddings file first, fall back to regular export
  const embeddingsPath = path.join(__dirname, '../../mondweep/knowledge-graph-with-embeddings.json');
  const regularPath = path.join(__dirname, '../../mondweep/media-hackathion-knowledge-graph-full-export-2025-12-08.json');

  const dataPath = fs.existsSync(embeddingsPath) ? embeddingsPath : regularPath;
  const rawData = fs.readFileSync(dataPath, 'utf8');
  cachedData = JSON.parse(rawData);
  return cachedData;
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
    const data = loadData();

    // --- Fetch Real Stats from Pinecone ---
    const PINECONE_API_KEY = process.env.PINECONE_API_KEY;
    const PINECONE_HOST = process.env.PINECONE_HOST;
    let totalMoviesFromPinecone = 0;

    if (PINECONE_API_KEY && PINECONE_HOST) {
      try {
        const hostUrl = PINECONE_HOST.startsWith('http') ? PINECONE_HOST : `https://${PINECONE_HOST}`;
        const statsResponse = await fetch(`${hostUrl}/describe_index_stats`, {
          method: 'POST',
          headers: {
            'Api-Key': PINECONE_API_KEY,
            'Content-Type': 'application/json',
            'X-Pinecone-API-Version': '2024-07'
          },
          body: JSON.stringify({})
        });
        if (statsResponse.ok) {
          const pineconeStats = await statsResponse.json();
          totalMoviesFromPinecone = pineconeStats.totalVectorCount || pineconeStats.total_vector_count || 0;
        }
      } catch (e) { console.error("Pinecone genre stats error", e); }
    }

    // Use genres directly from data.genres array
    let genres = data.data.genres || [];

    // Update counts if we have live data
    if (totalMoviesFromPinecone > 0) {
      // Calculate scaling factor based on the static data's total vs live total
      // Assuming static total is ~100k or sum of genre counts (approx)
      // Simple heuristic: Total Movies in Pinecone / 100,000 (base dataset size)
      const scale = totalMoviesFromPinecone > 1000 ? totalMoviesFromPinecone / 44425 : 1; // 44425 is approx count in static file

      genres = genres.map(g => ({
        ...g,
        type: 'genre',
        movieCount: Math.floor((g.movieCount || 100) * scale)
      }));
    } else {
      genres = genres.map(g => ({ ...g, type: 'genre' }));
    }

    // Sort by movie count descending
    genres.sort((a, b) => (b.movieCount || 0) - (a.movieCount || 0));

    return {
      statusCode: 200,
      headers,
      body: JSON.stringify({ genres }),
    };
  } catch (error) {
    console.error('Genres error:', error);
    return {
      statusCode: 500,
      headers,
      body: JSON.stringify({ error: 'Failed to load genres', details: error.message }),
    };
  }
};
