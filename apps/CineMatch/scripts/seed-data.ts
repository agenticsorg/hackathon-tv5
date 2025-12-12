
import { TMDB } from 'tmdb-ts';
import fs from 'fs/promises';
import path from 'path';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

const TMDB_ACCESS_TOKEN = process.env.NEXT_PUBLIC_TMDB_ACCESS_TOKEN;

if (!TMDB_ACCESS_TOKEN) {
    console.error('Error: NEXT_PUBLIC_TMDB_ACCESS_TOKEN is not defined in .env');
    process.exit(1);
}

const tmdb = new TMDB(TMDB_ACCESS_TOKEN);

const OUTPUT_FILE = path.join(process.cwd(), 'src/data/db.json');

// Configuration
// 200 pages * 20 results = 4000 items per category * 3 = 12,000 items total
const PAGES_TO_FETCH = 200;
const DELAY_MS = 250; // Increased delay to 250ms (4 req/s) to be safe with higher volume

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// Helper for raw fetch (since tmdb-ts might miss discover.tv)
async function rawDiscoverTV(params: Record<string, string | number>) {
    const queryParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
        queryParams.append(key, String(value));
    });

    const response = await fetch(
        `https://api.themoviedb.org/3/discover/tv?${queryParams.toString()}`,
        {
            headers: {
                Authorization: `Bearer ${TMDB_ACCESS_TOKEN}`,
                'Content-Type': 'application/json',
            },
        }
    );

    if (!response.ok) {
        throw new Error(`TMDB API Error: ${response.status} ${response.statusText}`);
    }

    return response.json();
}

async function fetchAllPages(fetcher: (page: number) => Promise<any>, totalPages: number, label: string) {
    let allResults: any[] = [];
    console.log(`Starting fetch for ${label}...`);

    for (let page = 1; page <= totalPages; page++) {
        try {
            const response = await fetcher(page);
            allResults = [...allResults, ...response.results];
            process.stdout.write(`\rFetched page ${page}/${totalPages} for ${label} (${allResults.length} items)`);
            await sleep(DELAY_MS);
        } catch (error) {
            console.error(`\nError fetching page ${page} for ${label}:`, error);
        }
    }
    console.log(`\nCompleted ${label}. Total: ${allResults.length}`);
    return allResults;
}

async function main() {
    console.log('🚀 Starting Database Seed (Target: ~12,000 items)...');

    // 1. Fetch Genres for mapping
    console.log('Fetching Genres...');
    const movieGenres = await tmdb.genres.movies();
    const tvGenres = await tmdb.genres.tvShows();

    const genreMap: Record<number, string> = {};
    [...movieGenres.genres, ...tvGenres.genres].forEach(g => {
        genreMap[g.id] = g.name;
    });

    // 2. Fetch Movies (Popularity Descending)
    const movies = await fetchAllPages(
        (page) => tmdb.discover.movie({
            page,
            sort_by: 'popularity.desc',
            'vote_count.gte': 10 // Filter out very obscure items
        }),
        PAGES_TO_FETCH,
        'Movies'
    );

    // 3. Fetch TV Shows (Popularity Descending)
    const tvShows = await fetchAllPages(
        (page) => rawDiscoverTV({
            page,
            sort_by: 'popularity.desc',
            'vote_count.gte': 10
        }),
        PAGES_TO_FETCH,
        'TV Shows'
    );

    // 4. Fetch Anime (Animation + Japanese, Popularity Descending)
    const anime = await fetchAllPages(
        (page) => rawDiscoverTV({
            page,
            with_genres: '16',
            with_original_language: 'ja',
            sort_by: 'popularity.desc',
            'vote_count.gte': 5 // Slightly lower threshold for Anime
        }),
        PAGES_TO_FETCH,
        'Anime'
    );

    // 5. Transform and Combine
    console.log('Transforming data...');

    const transform = (item: any, type: 'movie' | 'tv') => ({
        id: item.id,
        title: item.title || item.name,
        originalTitle: item.original_title || item.original_name,
        overview: item.overview,
        posterPath: item.poster_path,
        backdropPath: item.backdrop_path,
        mediaType: type,
        genreIds: item.genre_ids,
        genres: item.genre_ids?.map((id: number) => genreMap[id] || 'Unknown') || [],
        releaseDate: item.release_date || item.first_air_date,
        releaseYear: (item.release_date || item.first_air_date || '').split('-')[0],
        originalLanguage: item.original_language,
        voteAverage: item.vote_average,
        voteCount: item.vote_count,
        popularity: item.popularity
    });

    const db = {
        movies: movies.map(m => transform(m, 'movie')),
        tv: tvShows.map(t => transform(t, 'tv')),
        anime: anime.map(a => transform(a, 'tv')), // Anime are usually TV shows
        genres: genreMap,
        metadata: {
            generatedAt: new Date().toISOString(),
            counts: {
                movies: movies.length,
                tv: tvShows.length,
                anime: anime.length,
                total: movies.length + tvShows.length + anime.length
            }
        }
    };

    // 6. Save to file
    await fs.writeFile(OUTPUT_FILE, JSON.stringify(db, null, 2));
    console.log(`\n✅ Database saved to ${OUTPUT_FILE}`);
    console.log(`Stats: ${db.metadata.counts.movies} Movies, ${db.metadata.counts.tv} TV Shows, ${db.metadata.counts.anime} Anime`);
    console.log(`Total Items: ${db.metadata.counts.total}`);
}

main().catch(console.error);
