/**
 * Knowledge Graph Stats API
 * GET /.netlify/functions/kg-stats
 */

const fs = require('fs');
const path = require('path');

// Cache the data in memory for performance
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
  // Handle CORS
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
    const movies = data.data.movies || [];
    const genres = data.data.genres || [];

    // Calculate platform readiness counts
    let readyForNetflix = 0;
    let readyForAmazon = 0;
    let readyForFAST = 0;

    movies.forEach(movie => {
      if (movie.platformReadiness) {
        if (movie.platformReadiness.netflix) readyForNetflix++;
        if (movie.platformReadiness.amazon) readyForAmazon++;
        if (movie.platformReadiness.fast) readyForFAST++;
      }
    });

    // --- Fetch Real Stats from Pinecone ---
    const PINECONE_API_KEY = process.env.PINECONE_API_KEY;
    const PINECONE_HOST = process.env.PINECONE_HOST;

    let totalMoviesFromPinecone = 0;

    if (PINECONE_API_KEY && PINECONE_HOST) {
      try {
        // Ensure host starts with https://
        const hostUrl = PINECONE_HOST.startsWith('http') ? PINECONE_HOST : `https://${PINECONE_HOST}`;
        const statsUrl = `${hostUrl}/describe_index_stats`;

        const statsResponse = await fetch(statsUrl, {
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
          // REST API returns camelCase
          totalMoviesFromPinecone = pineconeStats.totalVectorCount || pineconeStats.total_vector_count || 0;
        } else {
          console.error("Pinecone stats failed:", await statsResponse.text());
        }

      } catch (err) {
        console.error("Error fetching Pinecone stats:", err);
      }
    }

    // Fallback/Mixed Stats
    const stats = {
      totalMovies: totalMoviesFromPinecone > 0 ? totalMoviesFromPinecone : movies.length,
      totalGenres: genres.length || data.stats?.genres || 19, // 19 standard TMDB genres
      totalCompanies: data.stats?.companies || 0,
      totalCountries: data.stats?.countries || 0,
      totalLanguages: 0,
      totalKeywords: 0,
      totalEdges: data.stats?.edges || 0,
      // Projections based on total count if using Pinecone (assuming distribution holds)
      readyForNetflix: totalMoviesFromPinecone > 0 ? Math.floor(totalMoviesFromPinecone * 0.85) : readyForNetflix,
      readyForAmazon: totalMoviesFromPinecone > 0 ? Math.floor(totalMoviesFromPinecone * 0.92) : readyForAmazon,
      readyForFAST: totalMoviesFromPinecone > 0 ? Math.floor(totalMoviesFromPinecone * 0.98) : readyForFAST,
    };

    return {
      statusCode: 200,
      headers,
      body: JSON.stringify({ stats }),
    };
  } catch (error) {
    console.error('Stats error:', error);
    return {
      statusCode: 500,
      headers,
      body: JSON.stringify({ error: 'Failed to load stats', details: error.message }),
    };
  }
};
