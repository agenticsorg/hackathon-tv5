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

    // Use genres directly from data.genres array
    let genres = data.data.genres || [];

    // Sort by movie count descending
    genres = genres.map(g => ({
      ...g,
      type: 'genre',
    })).sort((a, b) => (b.movieCount || 0) - (a.movieCount || 0));

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
