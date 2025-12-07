/**
 * Voice Query Integration Tests
 * Tests all voice query scenarios to ensure NO empty results
 */

import { describe, it, expect, beforeAll } from '@jest/globals';
import { semanticSearch } from '@/lib/natural-language-search';

describe('Voice Query Tests - Complete Coverage', () => {
  // Ensure TMDB API key is available
  beforeAll(() => {
    if (!process.env.NEXT_PUBLIC_TMDB_ACCESS_TOKEN) {
      throw new Error('TMDB_ACCESS_TOKEN must be set for integration tests');
    }
  });

  describe('1. Mood/Feeling Queries (Previously Failing)', () => {
    it('should return results for "show me something cool"', async () => {
      const results = await semanticSearch('show me something cool');

      expect(results.length).toBeGreaterThan(0);
      expect(results[0].content).toBeDefined();
      expect(results[0].relevanceScore).toBeGreaterThan(0);
      console.log(`✅ "cool" query returned ${results.length} results`);
    }, 15000);

    it('should return results for "bring me something interesting"', async () => {
      const results = await semanticSearch('bring me something interesting');

      expect(results.length).toBeGreaterThan(0);
      expect(results[0].matchReasons.length).toBeGreaterThan(0);
      console.log(`✅ "interesting" query returned ${results.length} results`);
    }, 15000);

    it('should return results for "I want something awesome"', async () => {
      const results = await semanticSearch('I want something awesome');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "awesome" query returned ${results.length} results`);
    }, 15000);

    it('should return results for "show me something good"', async () => {
      const results = await semanticSearch('show me something good');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "good" query returned ${results.length} results`);
    }, 15000);

    it('should return results for "find me something amazing"', async () => {
      const results = await semanticSearch('find me something amazing');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "amazing" query returned ${results.length} results`);
    }, 15000);

    it('should return results for "something entertaining"', async () => {
      const results = await semanticSearch('something entertaining');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "entertaining" query returned ${results.length} results`);
    }, 15000);
  });

  describe('2. Actor/Person Queries (Previously Failing)', () => {
    it('should return results for "show me something Richard Gere played"', async () => {
      const results = await semanticSearch('show me something Richard Gere played');

      expect(results.length).toBeGreaterThan(0);
      expect(results.some(r => r.matchReasons.some(reason => reason.includes('Richard Gere')))).toBe(true);
      console.log(`✅ "Richard Gere" query returned ${results.length} results`);
    }, 15000);

    it('should return results for "movies with Tom Hanks"', async () => {
      const results = await semanticSearch('movies with Tom Hanks');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "Tom Hanks" query returned ${results.length} results`);
      console.log(`   First result: ${results[0].content.title}`);
    }, 15000);

    it('should return results for "films directed by Spielberg"', async () => {
      const results = await semanticSearch('films directed by Spielberg');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "Spielberg" query returned ${results.length} results`);
    }, 15000);

    it('should return results for "starring Morgan Freeman"', async () => {
      const results = await semanticSearch('starring Morgan Freeman');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "Morgan Freeman" query returned ${results.length} results`);
    }, 15000);
  });

  describe('3. Platform/Trending Queries (Previously Failing)', () => {
    it('should return results for "what\'s new on Netflix"', async () => {
      const results = await semanticSearch("what's new on Netflix");

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "what's new on Netflix" returned ${results.length} results`);
    }, 15000);

    it('should return results for "what\'s trending"', async () => {
      const results = await semanticSearch("what's trending");

      expect(results.length).toBeGreaterThan(0);
      expect(results.some(r => r.matchReasons.includes('Currently trending'))).toBe(true);
      console.log(`✅ "trending" query returned ${results.length} results`);
    }, 15000);

    it('should return results for "what\'s popular"', async () => {
      const results = await semanticSearch("what's popular");

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "popular" query returned ${results.length} results`);
    }, 15000);

    it('should return results for "latest movies"', async () => {
      const results = await semanticSearch('latest movies');

      expect(results.length).toBeGreaterThan(0);
      expect(results.some(r => r.matchReasons.includes('Recent release'))).toBe(true);
      console.log(`✅ "latest movies" returned ${results.length} results`);
    }, 15000);

    it('should return results for "new releases"', async () => {
      const results = await semanticSearch('new releases');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "new releases" returned ${results.length} results`);
    }, 15000);
  });

  describe('4. Generic/Vague Queries (Previously Failing - Fallback Strategy)', () => {
    it('should return results for "something to watch"', async () => {
      const results = await semanticSearch('something to watch');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "something to watch" returned ${results.length} results (fallback)`);
    }, 15000);

    it('should return results for "entertain me"', async () => {
      const results = await semanticSearch('entertain me');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "entertain me" returned ${results.length} results`);
    }, 15000);

    it('should return results for "surprise me"', async () => {
      const results = await semanticSearch('surprise me');

      expect(results.length).toBeGreaterThan(0);
      expect(results[0].matchReasons.length).toBeGreaterThan(0);
      console.log(`✅ "surprise me" returned ${results.length} results (fallback)`);
    }, 15000);

    it('should return results for "I\'m bored"', async () => {
      const results = await semanticSearch("I'm bored");

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "I'm bored" returned ${results.length} results`);
    }, 15000);

    it('should return results for "find me a movie"', async () => {
      const results = await semanticSearch('find me a movie');

      expect(results.length).toBeGreaterThan(0);
      expect(results[0].content.mediaType).toBe('movie');
      console.log(`✅ "find me a movie" returned ${results.length} results`);
    }, 15000);
  });

  describe('5. Combined Mood + Genre Queries', () => {
    it('should return results for "cool action movie"', async () => {
      const results = await semanticSearch('cool action movie');

      expect(results.length).toBeGreaterThan(0);
      expect(results.some(r => r.content.genreIds.includes(28))).toBe(true); // Action genre ID
      console.log(`✅ "cool action movie" returned ${results.length} results`);
    }, 15000);

    it('should return results for "interesting sci-fi"', async () => {
      const results = await semanticSearch('interesting sci-fi');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ "interesting sci-fi" returned ${results.length} results`);
    }, 15000);

    it('should return results for "awesome comedy"', async () => {
      const results = await semanticSearch('awesome comedy');

      expect(results.length).toBeGreaterThan(0);
      expect(results.some(r => r.content.genreIds.includes(35))).toBe(true); // Comedy genre ID
      console.log(`✅ "awesome comedy" returned ${results.length} results`);
    }, 15000);
  });

  describe('6. Edge Cases & Special Queries', () => {
    it('should handle empty string gracefully', async () => {
      const results = await semanticSearch('');

      // Should still return fallback results
      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ Empty query returned ${results.length} fallback results`);
    }, 15000);

    it('should handle single word queries', async () => {
      const results = await semanticSearch('cool');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ Single word "cool" returned ${results.length} results`);
    }, 15000);

    it('should handle very vague queries', async () => {
      const results = await semanticSearch('show me stuff');

      expect(results.length).toBeGreaterThan(0);
      console.log(`✅ Vague "show me stuff" returned ${results.length} results`);
    }, 15000);
  });

  describe('7. Quality Checks', () => {
    it('should never return zero results for any query', async () => {
      const testQueries = [
        'cool',
        'interesting',
        'awesome',
        'good',
        'amazing',
        'trending',
        'new',
        'something to watch',
        'entertain me',
        'Tom Hanks',
        'action',
        'comedy',
      ];

      for (const query of testQueries) {
        const results = await semanticSearch(query);
        expect(results.length).toBeGreaterThan(0);
        console.log(`✅ "${query}" → ${results.length} results`);
      }
    }, 60000);

    it('should return relevance scores between 0 and 1', async () => {
      const results = await semanticSearch('cool action movie');

      results.forEach(result => {
        expect(result.relevanceScore).toBeGreaterThanOrEqual(0);
        expect(result.relevanceScore).toBeLessThanOrEqual(1);
      });
    }, 15000);

    it('should include match reasons for all results', async () => {
      const results = await semanticSearch('trending movies');

      results.forEach(result => {
        expect(result.matchReasons.length).toBeGreaterThan(0);
      });
    }, 15000);
  });
});
