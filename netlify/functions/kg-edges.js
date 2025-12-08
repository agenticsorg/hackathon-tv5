/**
 * Knowledge Graph Edges API
 * GET /.netlify/functions/kg-edges?movieIds=123,456,789
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
    const allEdges = data.data.edges || [];

    // Parse query parameters
    const params = event.queryStringParameters || {};
    const movieIdsParam = params.movieIds || '';
    const movieIds = movieIdsParam ? movieIdsParam.split(',').map(id => id.trim()) : [];

    let edges = allEdges;

    // Filter edges by movieIds if provided
    if (movieIds.length > 0) {
      const movieIdSet = new Set(movieIds);
      edges = allEdges.filter(edge => movieIdSet.has(edge.movieId));
    }

    return {
      statusCode: 200,
      headers,
      body: JSON.stringify({
        edges,
        total: edges.length,
      }),
    };
  } catch (error) {
    console.error('Edges error:', error);
    return {
      statusCode: 500,
      headers,
      body: JSON.stringify({ error: 'Failed to load edges', details: error.message }),
    };
  }
};
