/**
 * Knowledge Graph Movies API
 * GET /.netlify/functions/kg-movies?limit=20&offset=0&sortBy=popularity&sortOrder=desc
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
    const data = loadData();
    let movies = [...(data.data.movies || [])];

    // Parse query parameters
    const params = event.queryStringParameters || {};
    const limit = parseInt(params.limit) || 20;
    const offset = parseInt(params.offset) || 0;
    const sortBy = params.sortBy || 'popularity';
    const sortOrder = params.sortOrder || 'desc';
    const genre = params.genre;

    // Filter by genre if provided
    if (genre) {
      movies = movies.filter(m => {
        if (m.genres) {
          return m.genres.some(g => g.id === genre || g.name === genre);
        }
        return false;
      });
    }

    // Sort movies
    movies.sort((a, b) => {
      let aVal = a[sortBy] || 0;
      let bVal = b[sortBy] || 0;

      if (sortBy === 'title') {
        aVal = (a.title || '').toLowerCase();
        bVal = (b.title || '').toLowerCase();
        return sortOrder === 'asc' ? aVal.localeCompare(bVal) : bVal.localeCompare(aVal);
      }

      if (sortOrder === 'asc') {
        return aVal - bVal;
      }
      return bVal - aVal;
    });

    // Paginate
    const total = movies.length;
    const paginatedMovies = movies.slice(offset, offset + limit);
    const hasMore = offset + limit < total;

    // Map to expected format
    const movieResults = paginatedMovies.map(movie => ({
      ...movie,
      type: 'movie',
    }));

    return {
      statusCode: 200,
      headers,
      body: JSON.stringify({
        movies: movieResults,
        total,
        limit,
        offset,
        hasMore,
        pagination: {
          total,
          limit,
          offset,
          hasMore,
        },
      }),
    };
  } catch (error) {
    console.error('Movies error:', error);
    return {
      statusCode: 500,
      headers,
      body: JSON.stringify({ error: 'Failed to load movies', details: error.message }),
    };
  }
};
