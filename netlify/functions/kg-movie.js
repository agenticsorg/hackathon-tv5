/**
 * Knowledge Graph Single Movie API
 * GET /.netlify/functions/kg-movie?id=12345
 */

const fs = require('fs');
const path = require('path');

let cachedData = null;

function loadData() {
  if (cachedData) return cachedData;

  const dataPath = path.join(__dirname, '../../mondweep/media-hackathion-knowledge-graph-full-export-2025-12-08.json');
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
    const params = event.queryStringParameters || {};
    const movieId = params.id;

    if (!movieId) {
      return {
        statusCode: 400,
        headers,
        body: JSON.stringify({ error: 'Movie ID is required' }),
      };
    }

    const data = loadData();
    const movies = data.data.movies || [];

    const movie = movies.find(m => m.id === movieId || m.id === String(movieId));

    if (!movie) {
      return {
        statusCode: 404,
        headers,
        body: JSON.stringify({ error: 'Movie not found' }),
      };
    }

    return {
      statusCode: 200,
      headers,
      body: JSON.stringify({ movie }),
    };
  } catch (error) {
    console.error('Movie detail error:', error);
    return {
      statusCode: 500,
      headers,
      body: JSON.stringify({ error: 'Failed to load movie', details: error.message }),
    };
  }
};
