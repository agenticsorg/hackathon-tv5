/**
 * Streaming Availability Integration Tests
 *
 * Tests real API calls to TMDB watch providers (powered by JustWatch)
 * Requires NEXT_PUBLIC_TMDB_ACCESS_TOKEN environment variable
 */

import { describe, it, expect } from 'vitest';
import {
  getStreamingAvailability,
  getBatchStreamingAvailability,
  parseStreamingPreference,
  filterByStreamingService,
  formatStreamingProviders,
  getStreamingBadge,
  isStreamingNow,
  MAJOR_STREAMING_SERVICES,
} from '@/lib/streaming-availability';

// Skip all tests if no API key
const describeWithAPI = process.env.NEXT_PUBLIC_TMDB_ACCESS_TOKEN
  ? describe
  : describe.skip;

describe('Streaming Preference Parsing', () => {
  it('should detect Netflix in query', () => {
    const result = parseStreamingPreference('Find me a comedy on Netflix');

    // Netflix IDs from MAJOR_STREAMING_SERVICES.netflix = [8]
    expect(result.serviceIds).toContain(8);
    expect(result.serviceName).toBe('Netflix');
  });

  it('should detect Disney+ variations', () => {
    const queries = [
      'something on Disney+',
      'Disney Plus movies',
      'disney shows',
    ];

    for (const query of queries) {
      const result = parseStreamingPreference(query);
      // Disney+ ID from MAJOR_STREAMING_SERVICES.disney_plus = [337]
      expect(result.serviceIds).toContain(337);
    }
  });

  it('should detect HBO Max / Max', () => {
    const result = parseStreamingPreference('Best Max series');

    // Max/HBO IDs from MAJOR_STREAMING_SERVICES.hbo_max = [384, 1899]
    expect(result.serviceIds.some(id => [384, 1899].includes(id))).toBe(true);
    expect(result.serviceName).toBe('Max');
  });

  it('should detect Amazon Prime Video', () => {
    const queries = [
      'Prime Video originals',
      'Amazon Prime movies',
      'something on prime',
    ];

    for (const query of queries) {
      const result = parseStreamingPreference(query);
      // Prime IDs from MAJOR_STREAMING_SERVICES.amazon_prime = [9, 119]
      expect(result.serviceIds.some(id => [9, 119].includes(id))).toBe(true);
    }
  });

  it('should return first matched streaming service', () => {
    // The function returns on first match (Netflix comes before Hulu in the pattern list)
    const result = parseStreamingPreference(
      'Movies available on Netflix or Hulu'
    );

    // Should return Netflix (first match in the pattern list)
    expect(result.serviceIds).toContain(8);  // Netflix
    expect(result.serviceName).toBe('Netflix');
  });

  it('should detect all major services with generic streaming query', () => {
    const result = parseStreamingPreference(
      'Something good streaming tonight'
    );

    // Generic "streaming" query returns all major services
    expect(result.serviceIds.length).toBeGreaterThan(5);
    expect(result.serviceName).toBeNull();
  });

  it('should return empty for queries without streaming mentions', () => {
    const result = parseStreamingPreference('A good action movie from the 80s');

    expect(result.serviceIds).toEqual([]);
    expect(result.serviceName).toBeNull();
  });
});

describeWithAPI('Streaming Availability API', () => {
  // Well-known content IDs for testing
  const INCEPTION_ID = 27205; // Movie: Inception (2010)
  const STRANGER_THINGS_ID = 66732; // TV: Stranger Things

  it('should get streaming availability for a popular movie (Inception)', async () => {
    const availability = await getStreamingAvailability(INCEPTION_ID, 'movie', 'US');

    // May or may not be available, but should return valid structure
    if (availability) {
      expect(availability.contentId).toBe(INCEPTION_ID);
      expect(availability.mediaType).toBe('movie');
      expect(availability.region).toBe('US');
      expect(availability.providers).toHaveProperty('flatrate');
      expect(availability.providers).toHaveProperty('rent');
      expect(availability.providers).toHaveProperty('buy');
      expect(availability.providers).toHaveProperty('free');

      console.log('Inception streaming providers:', {
        flatrate: availability.providers.flatrate.map(p => p.provider.name),
        rent: availability.providers.rent.map(p => p.provider.name),
        buy: availability.providers.buy.map(p => p.provider.name),
        free: availability.providers.free.map(p => p.provider.name),
      });
    }
  });

  it('should get streaming availability for a Netflix original (Stranger Things)', async () => {
    const availability = await getStreamingAvailability(STRANGER_THINGS_ID, 'tv', 'US');

    if (availability) {
      expect(availability.contentId).toBe(STRANGER_THINGS_ID);
      expect(availability.mediaType).toBe('tv');

      // Stranger Things should be on Netflix
      const hasNetflix = availability.providers.flatrate.some(
        p => p.provider.name.toLowerCase().includes('netflix')
      );
      expect(hasNetflix).toBe(true);

      console.log('Stranger Things streaming providers:', {
        flatrate: availability.providers.flatrate.map(p => p.provider.name),
      });
    }
  });

  it('should handle batch streaming lookups efficiently', async () => {
    const content = [
      { id: INCEPTION_ID, mediaType: 'movie' as const },
      { id: STRANGER_THINGS_ID, mediaType: 'tv' as const },
      { id: 278, mediaType: 'movie' as const }, // Shawshank Redemption
    ];

    const start = Date.now();
    const results = await getBatchStreamingAvailability(content, 'US');
    const duration = Date.now() - start;

    expect(results.size).toBeGreaterThan(0);
    console.log(`Batch lookup for ${content.length} items: ${duration}ms`);

    // Should complete in reasonable time (parallel execution)
    expect(duration).toBeLessThan(10000);
  });

  it('should work with different regions', async () => {
    const regions = ['US', 'GB', 'DE', 'CA'];

    for (const region of regions) {
      const availability = await getStreamingAvailability(
        INCEPTION_ID,
        'movie',
        region
      );

      // Availability varies by region
      if (availability) {
        expect(availability.region).toBe(region);
        console.log(
          `Inception in ${region}:`,
          availability.providers.flatrate.length,
          'streaming services'
        );
      }
    }
  });

  it('should return null for non-existent content', async () => {
    const availability = await getStreamingAvailability(
      999999999, // Non-existent ID
      'movie',
      'US'
    );

    expect(availability).toBeNull();
  });
});

describe('Streaming Badge & Formatting', () => {
  const mockAvailability = {
    contentId: 1,
    mediaType: 'movie' as const,
    region: 'US',
    providers: {
      flatrate: [
        { provider: { id: 8, name: 'Netflix', logoPath: '/path' } },
        { provider: { id: 9, name: 'Amazon Prime Video', logoPath: '/path' } },
      ],
      rent: [{ provider: { id: 3, name: 'Apple TV', logoPath: '/path' } }],
      buy: [{ provider: { id: 3, name: 'Apple TV', logoPath: '/path' } }],
      free: [],
      ads: [],
    },
  };

  it('should format streaming providers correctly', () => {
    const formatted = formatStreamingProviders(mockAvailability);

    expect(formatted).toContain('Netflix');
    expect(formatted).toContain('Amazon Prime Video');
  });

  it('should return correct badge for subscription content', () => {
    const badge = getStreamingBadge(mockAvailability);

    expect(badge.type).toBe('subscription');
    // Badge text includes provider name like "Netflix"
    expect(badge.text.length).toBeGreaterThan(0);
    expect(badge.color).toBeTruthy();
  });

  it('should detect content that is streaming now', () => {
    expect(isStreamingNow(mockAvailability)).toBe(true);
  });

  it('should detect content not streaming (rental only)', () => {
    const rentalOnly = {
      ...mockAvailability,
      providers: {
        ...mockAvailability.providers,
        flatrate: [],
        free: [],
      },
    };

    expect(isStreamingNow(rentalOnly)).toBe(false);
    const badge = getStreamingBadge(rentalOnly);
    expect(badge.type).toBe('rental');
  });

  it('should filter by streaming service', () => {
    // Netflix ID is 8
    expect(filterByStreamingService(mockAvailability, [8])).toBe(true);

    // HBO Max ID is 384 (not in mock)
    expect(filterByStreamingService(mockAvailability, [384])).toBe(false);
  });
});
