
import dbData from '@/data/db.json';
import type { Movie, TVShow, MediaContent } from '@/types/media';

// Types for our local DB structure
interface LocalDB {
    movies: any[];
    tv: any[];
    anime: any[];
    genres: Record<string, string>;
}

const db = dbData as LocalDB;

// Helper to calculate relevance score (simple RAG-like scoring)
function calculateScore(item: any, preferences: any, favorite?: any): number {
    let score = 0;

    // 1. Genre Match
    // If we have a favorite movie, use its genres for boosting too
    const targetGenres = preferences.genres || (favorite?.genreIds) || [];
    if (targetGenres.length && item.genreIds) {
        const matchCount = item.genreIds.filter((id: number) => targetGenres.includes(id)).length;
        score += matchCount * 3; // Increased weight: 3 points per genre match
    }

    // 2. Language Match (Strict for Anime, Exclusion for Western Animation)
    if (preferences.withOriginalLanguage) {
        if (item.originalLanguage === preferences.withOriginalLanguage) {
            score += 5; // High boost for language match
        } else if (preferences.strictLanguage) {
            return -1; // Filter out if strict
        }
    }

    // Exclude languages (e.g., for Western Animation)
    if (preferences.excludeOriginalLanguages) {
        if (preferences.excludeOriginalLanguages.includes(item.originalLanguage)) {
            return -1;
        }
    }

    // 3. Era/Year Match
    if (preferences.yearMin && item.releaseYear) {
        if (parseInt(item.releaseYear) >= preferences.yearMin) score += 3;
        else return -1; // Strict filter
    }
    if (preferences.yearMax && item.releaseYear) {
        if (parseInt(item.releaseYear) <= preferences.yearMax) score += 3;
        else return -1; // Strict filter
    }

    // 4. Intent Match
    if (preferences.intent === 'learn') {
        // Boost Documentary (99), History (36), War (10752) - often biopics/historical
        if (item.genreIds?.some((id: number) => [99, 36, 10752].includes(id))) {
            score += 10; // Massive boost for learning content
        } else {
            score -= 5; // Penalize non-learning content
        }
    } else if (preferences.intent === 'kill_time') {
        // Boost Entertainment (Comedy, Action, Adventure, Sci-Fi, Fantasy, Horror, Romance, Thriller)
        // Basically everything NOT Documentary/History
        if (!item.genreIds?.some((id: number) => [99, 36].includes(id))) {
            score += 2;
        }
    }

    // 5. Favorite Movie Boost (Rating & Reliability)
    if (favorite) {
        // Boost if rating is same or higher
        if (item.voteAverage >= (favorite.voteAverage || 7)) {
            score += 5;
        }

        // Reliability Boost (Logarithmic scale of vote count)
        // More votes = more reliable rating = higher confidence
        if (item.voteCount > 0) {
            score += Math.log10(item.voteCount);
        }
    }

    // 6. Popularity Boost (Base score)
    score += (item.popularity || 0) / 100;

    return score;
}

export const localDB = {
    /**
     * Search/Recommend from local data
     */
    recommend: (options: {
        type: 'movie' | 'tv' | 'anime' | 'all';
        genres?: number[];
        yearMin?: number;
        yearMax?: number;
        withOriginalLanguage?: string;
        excludeOriginalLanguages?: string[];
        excludeGenres?: number[]; // New param for strict exclusion
        intent?: 'learn' | 'kill_time';
        favorite?: any; // Pass full favorite object
        limit?: number;
    }) => {
        let candidates: any[] = [];

        // Select source pool
        if (options.type === 'movie') candidates = db.movies;
        else if (options.type === 'tv') candidates = db.tv;
        else if (options.type === 'anime') candidates = db.anime;
        else candidates = [...db.movies, ...db.tv];

        // Score and Filter
        let ranked = candidates
            .filter(item => {
                // Hard Exclusion: Genres
                if (options.excludeGenres && item.genreIds) {
                    if (item.genreIds.some((id: number) => options.excludeGenres!.includes(id))) {
                        return false;
                    }
                }
                return true;
            })
            .map(item => ({
                item,
                score: calculateScore(item, {
                    genres: options.genres,
                    yearMin: options.yearMin,
                    yearMax: options.yearMax,
                    withOriginalLanguage: options.withOriginalLanguage,
                    excludeOriginalLanguages: options.excludeOriginalLanguages,
                    strictLanguage: options.type === 'anime', // Strict language for anime
                    intent: options.intent
                }, options.favorite)
            }))
            .filter(entry => entry.score > 0) // Remove filtered/irrelevant items
            .sort((a, b) => (b.item.voteCount || 0) - (a.item.voteCount || 0)); // Sort by voteCount (desc)

        // Diversity Logic: Distinct Release Years for top N
        const limit = options.limit || 10;
        const finalResults: any[] = [];
        const seenYears = new Set<string>();

        // First pass: Try to find unique years
        for (const entry of ranked) {
            if (finalResults.length >= limit) break;

            const year = entry.item.releaseYear;
            if (year && !seenYears.has(year)) {
                finalResults.push(entry.item);
                seenYears.add(year);
            }
        }

        // Second pass: Fill if not enough unique years
        if (finalResults.length < limit) {
            for (const entry of ranked) {
                if (finalResults.length >= limit) break;
                if (!finalResults.includes(entry.item)) {
                    finalResults.push(entry.item);
                }
            }
        }

        return finalResults;
    },

    /**
     * Get item by ID
     */
    getById: (id: number, type: 'movie' | 'tv') => {
        const pool = type === 'movie' ? db.movies : [...db.tv, ...db.anime];
        return pool.find(item => item.id === id);
    }
};
