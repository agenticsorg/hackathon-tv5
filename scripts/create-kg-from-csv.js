/**
 * Convert TMDB CSV to Knowledge Graph JSON Format
 * 
 * Usage:
 *   node scripts/create-kg-from-csv.js
 */

const fs = require('fs');
const path = require('path');
const csv = require('csv-parser');

// Config
const INPUT_CSV = path.join(__dirname, '../mondweep/data/tmdb_movies.csv');
const OUTPUT_JSON = path.join(__dirname, '../mondweep/data/full_knowledge_graph.json');

// Helper to format date
const toISO = (dateStr) => {
    if (!dateStr) return null;
    try {
        return new Date(dateStr).toISOString();
    } catch { return null; }
};

// Main processing function
async function processCsv() {
    console.log(`Reading CSV from: ${INPUT_CSV}`);
    console.log(`Writing JSON to: ${OUTPUT_JSON}`);

    const writeStream = fs.createWriteStream(OUTPUT_JSON);

    // Write Header
    const header = {
        exportedAt: new Date().toISOString(),
        version: "2.0",
        source: "TMDB_v11.csv",
        stats: { movies: 0 }, // Will update later if possible, or just leave as placeholder
        data: { movies: [] }
    };

    // We manually write the JSON structure to support streaming
    // { "exportedAt": "...", "data": { "movies": [
    writeStream.write(`{
  "exportedAt": "${header.exportedAt}",
  "version": "${header.version}",
  "source": "${header.source}",
  "data": {
    "movies": [\n`);

    let isFirst = true;
    let count = 0;

    const stream = fs.createReadStream(INPUT_CSV).pipe(csv());

    for await (const row of stream) {
        // Map CSV columns to Schema
        // CSV Headers: id,title,vote_average,vote_count,status,release_date,revenue,runtime,adult,backdrop_path,budget,homepage,imdb_id,original_language,original_title,overview,popularity,poster_path,tagline,genres,...

        try {
            const movie = {
                id: row.id,
                type: 'movie',
                title: row.title,
                originalTitle: row.original_title,
                overview: row.overview,
                tagline: row.tagline,
                adult: row.adult === 'True',
                status: row.status,
                releaseDate: row.release_date,
                year: row.release_date ? parseInt(row.release_date.split('-')[0]) : null,
                runtime: parseInt(row.runtime) || 0,
                budget: parseInt(row.budget) || 0,
                revenue: parseInt(row.revenue) || 0,
                voteAverage: parseFloat(row.vote_average) || 0,
                voteCount: parseInt(row.vote_count) || 0,
                popularity: parseFloat(row.popularity) || 0,
                posterPath: row.poster_path,
                backdropPath: row.backdrop_path,
                imdbId: row.imdb_id,
                homepage: row.homepage,
                genres: row.genres ? row.genres.split(',').map(g => g.trim()) : [],
                // Add some default fields from the schema
                distributionStatus: "ready",
                createdAt: new Date().toISOString(),
                platformReadiness: {
                    netflix: true,
                    amazon: true,
                    fast: true,
                    validatedAt: new Date().toISOString()
                }
            };

            // Basic validation - skip empty titles
            if (!movie.title) continue;

            const jsonStr = JSON.stringify(movie);

            if (!isFirst) {
                writeStream.write(',\n');
            }
            writeStream.write(jsonStr);
            isFirst = false;
            count++;

            if (count % 10000 === 0) {
                process.stdout.write(`\rProcessed ${count} records...`);
            }
        } catch (err) {
            console.error(`Error parsing row: ${err.message}`);
        }
    }

    // Write Footer
    writeStream.write('\n    ]\n  },\n');
    writeStream.write(`  "stats": { "movies": ${count} }\n`);
    writeStream.write('}');
    writeStream.end();

    console.log(`\n\n✅ Conversion complete!`);
    console.log(`   Total Records: ${count}`);
    console.log(`   Output File: ${OUTPUT_JSON}`);
}

processCsv().catch(err => console.error(err));
