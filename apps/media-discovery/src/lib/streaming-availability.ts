/**
 * Streaming Availability Service
 *
 * Provides real-time streaming availability data using TMDB's watch providers API
 * (powered by JustWatch data). This transforms "find me a rom-com" into
 * "find me a rom-com streaming on Netflix/Hulu tonight".
 */

import { tmdb } from './tmdb';

// Streaming provider information
export interface StreamingProvider {
  id: number;
  name: string;
  logoPath: string | null;
  displayPriority: number;
}

// How content is available (rent, buy, flatrate/subscription, free, ads)
export type AvailabilityType = 'flatrate' | 'rent' | 'buy' | 'free' | 'ads';

// Content availability on a specific provider
export interface ProviderAvailability {
  provider: StreamingProvider;
  type: AvailabilityType;
  link?: string;
}

// Full streaming availability for a piece of content
export interface StreamingAvailability {
  contentId: number;
  mediaType: 'movie' | 'tv';
  region: string;
  providers: {
    flatrate: ProviderAvailability[];   // Subscription services (Netflix, Disney+, etc.)
    rent: ProviderAvailability[];        // Rental options
    buy: ProviderAvailability[];         // Purchase options
    free: ProviderAvailability[];        // Free with ads
    ads: ProviderAvailability[];         // Ad-supported streaming
  };
  link: string;  // Link to JustWatch or TMDB page
  lastUpdated: Date;
}

// Major streaming services by provider ID (for filtering)
export const MAJOR_STREAMING_SERVICES: Record<string, number[]> = {
  // Subscription services
  netflix: [8],
  amazon_prime: [9, 119],  // Prime Video and Prime Video with Ads
  disney_plus: [337],
  hulu: [15],
  hbo_max: [384, 1899],    // Max (formerly HBO Max)
  apple_tv_plus: [350],
  paramount_plus: [531],
  peacock: [386, 387],
  // Free services
  tubi: [73],
  pluto: [300],
  freevee: [613],
  crackle: [12],
};

// Provider name normalization
const PROVIDER_DISPLAY_NAMES: Record<number, string> = {
  8: 'Netflix',
  9: 'Amazon Prime Video',
  15: 'Hulu',
  337: 'Disney+',
  350: 'Apple TV+',
  384: 'Max',
  386: 'Peacock',
  531: 'Paramount+',
  73: 'Tubi',
  300: 'Pluto TV',
  613: 'Freevee',
  1899: 'Max',
};

// Cache for provider logos
const providerCache = new Map<number, StreamingProvider>();

/**
 * Get streaming availability for a movie
 */
export async function getMovieStreamingAvailability(
  movieId: number,
  region: string = 'US'
): Promise<StreamingAvailability | null> {
  if (!tmdb) return null;

  try {
    const response = await tmdb.movies.watchProviders(movieId);
    const regionData = response.results?.[region];

    if (!regionData) {
      return null;
    }

    return transformWatchProviders(movieId, 'movie', region, regionData);
  } catch (error) {
    console.error(`Failed to get streaming availability for movie ${movieId}:`, error);
    return null;
  }
}

/**
 * Get streaming availability for a TV show
 */
export async function getTVStreamingAvailability(
  tvId: number,
  region: string = 'US'
): Promise<StreamingAvailability | null> {
  if (!tmdb) return null;

  try {
    const response = await tmdb.tvShows.watchProviders(tvId);
    const regionData = response.results?.[region];

    if (!regionData) {
      return null;
    }

    return transformWatchProviders(tvId, 'tv', region, regionData);
  } catch (error) {
    console.error(`Failed to get streaming availability for TV show ${tvId}:`, error);
    return null;
  }
}

/**
 * Get streaming availability for any content type
 */
export async function getStreamingAvailability(
  contentId: number,
  mediaType: 'movie' | 'tv',
  region: string = 'US'
): Promise<StreamingAvailability | null> {
  return mediaType === 'movie'
    ? getMovieStreamingAvailability(contentId, region)
    : getTVStreamingAvailability(contentId, region);
}

/**
 * Get streaming availability for multiple content items in parallel
 */
export async function getBatchStreamingAvailability(
  items: Array<{ id: number; mediaType: 'movie' | 'tv' }>,
  region: string = 'US'
): Promise<Map<string, StreamingAvailability>> {
  const results = new Map<string, StreamingAvailability>();

  // Process in batches of 10 to avoid rate limiting
  const batchSize = 10;
  for (let i = 0; i < items.length; i += batchSize) {
    const batch = items.slice(i, i + batchSize);
    const promises = batch.map(item =>
      getStreamingAvailability(item.id, item.mediaType, region)
    );

    const batchResults = await Promise.all(promises);

    batch.forEach((item, index) => {
      const availability = batchResults[index];
      if (availability) {
        results.set(`${item.mediaType}-${item.id}`, availability);
      }
    });
  }

  return results;
}

/**
 * Filter content by streaming service
 */
export function filterByStreamingService(
  availability: StreamingAvailability,
  serviceIds: number[]
): boolean {
  const flatrateProviderIds = availability.providers.flatrate.map(p => p.provider.id);
  return serviceIds.some(id => flatrateProviderIds.includes(id));
}

/**
 * Check if content is available on any subscription service
 */
export function isStreamingNow(availability: StreamingAvailability): boolean {
  return availability.providers.flatrate.length > 0;
}

/**
 * Check if content is available for free (with ads)
 */
export function isFreeToWatch(availability: StreamingAvailability): boolean {
  return availability.providers.free.length > 0 || availability.providers.ads.length > 0;
}

/**
 * Get a formatted string of where content is streaming
 */
export function formatStreamingProviders(availability: StreamingAvailability): string {
  const providers = availability.providers.flatrate;

  if (providers.length === 0) {
    // Check free options
    const freeProviders = [...availability.providers.free, ...availability.providers.ads];
    if (freeProviders.length > 0) {
      return `Free on ${freeProviders.map(p => p.provider.name).join(', ')}`;
    }

    // Check rental/purchase options
    if (availability.providers.rent.length > 0) {
      return `Available to rent`;
    }
    if (availability.providers.buy.length > 0) {
      return `Available to buy`;
    }

    return 'Not currently streaming';
  }

  if (providers.length === 1) {
    return `Streaming on ${providers[0].provider.name}`;
  }

  if (providers.length === 2) {
    return `Streaming on ${providers[0].provider.name} and ${providers[1].provider.name}`;
  }

  const first = providers.slice(0, 2).map(p => p.provider.name).join(', ');
  return `Streaming on ${first} and ${providers.length - 2} more`;
}

/**
 * Get streaming badge for UI display
 */
export function getStreamingBadge(availability: StreamingAvailability): {
  text: string;
  type: 'subscription' | 'free' | 'rental' | 'purchase' | 'unavailable';
  color: string;
} {
  if (availability.providers.flatrate.length > 0) {
    return {
      text: availability.providers.flatrate[0].provider.name,
      type: 'subscription',
      color: '#e50914', // Netflix-like red
    };
  }

  if (availability.providers.free.length > 0 || availability.providers.ads.length > 0) {
    const freeProviders = [...availability.providers.free, ...availability.providers.ads];
    return {
      text: `Free on ${freeProviders[0].provider.name}`,
      type: 'free',
      color: '#00a651', // Green for free
    };
  }

  if (availability.providers.rent.length > 0) {
    return {
      text: 'Rent',
      type: 'rental',
      color: '#ff9800', // Orange for rental
    };
  }

  if (availability.providers.buy.length > 0) {
    return {
      text: 'Buy',
      type: 'purchase',
      color: '#2196f3', // Blue for purchase
    };
  }

  return {
    text: 'Not Available',
    type: 'unavailable',
    color: '#666',
  };
}

/**
 * Transform TMDB watch providers response to our format
 */
function transformWatchProviders(
  contentId: number,
  mediaType: 'movie' | 'tv',
  region: string,
  data: {
    link?: string;
    flatrate?: TMDBProvider[];
    rent?: TMDBProvider[];
    buy?: TMDBProvider[];
    free?: TMDBProvider[];
    ads?: TMDBProvider[];
  }
): StreamingAvailability {
  const transform = (providers: TMDBProvider[] | undefined): ProviderAvailability[] => {
    if (!providers) return [];
    return providers.map(p => ({
      provider: {
        id: p.provider_id,
        name: PROVIDER_DISPLAY_NAMES[p.provider_id] || p.provider_name,
        logoPath: p.logo_path ? `https://image.tmdb.org/t/p/original${p.logo_path}` : null,
        displayPriority: p.display_priority,
      },
      type: 'flatrate' as AvailabilityType,
    }));
  };

  return {
    contentId,
    mediaType,
    region,
    providers: {
      flatrate: transform(data.flatrate),
      rent: transform(data.rent),
      buy: transform(data.buy),
      free: transform(data.free),
      ads: transform(data.ads),
    },
    link: data.link || `https://www.themoviedb.org/${mediaType}/${contentId}/watch`,
    lastUpdated: new Date(),
  };
}

// TMDB provider response type
interface TMDBProvider {
  provider_id: number;
  provider_name: string;
  logo_path: string | null;
  display_priority: number;
}

/**
 * Parse user query for streaming service preferences
 */
export function parseStreamingPreference(query: string): {
  serviceIds: number[];
  serviceName: string | null;
} {
  const queryLower = query.toLowerCase();

  // Check for explicit streaming service mentions
  const servicePatterns: Array<{ pattern: RegExp; ids: number[]; name: string }> = [
    { pattern: /\bnetflix\b/i, ids: MAJOR_STREAMING_SERVICES.netflix, name: 'Netflix' },
    { pattern: /\b(amazon|prime|prime video)\b/i, ids: MAJOR_STREAMING_SERVICES.amazon_prime, name: 'Prime Video' },
    { pattern: /\bdisney\+?\b/i, ids: MAJOR_STREAMING_SERVICES.disney_plus, name: 'Disney+' },
    { pattern: /\bhulu\b/i, ids: MAJOR_STREAMING_SERVICES.hulu, name: 'Hulu' },
    { pattern: /\b(hbo|max)\b/i, ids: MAJOR_STREAMING_SERVICES.hbo_max, name: 'Max' },
    { pattern: /\bapple\s*tv\+?\b/i, ids: MAJOR_STREAMING_SERVICES.apple_tv_plus, name: 'Apple TV+' },
    { pattern: /\bparamount\+?\b/i, ids: MAJOR_STREAMING_SERVICES.paramount_plus, name: 'Paramount+' },
    { pattern: /\bpeacock\b/i, ids: MAJOR_STREAMING_SERVICES.peacock, name: 'Peacock' },
    { pattern: /\btubi\b/i, ids: MAJOR_STREAMING_SERVICES.tubi, name: 'Tubi' },
    { pattern: /\bpluto\b/i, ids: MAJOR_STREAMING_SERVICES.pluto, name: 'Pluto TV' },
  ];

  for (const { pattern, ids, name } of servicePatterns) {
    if (pattern.test(query)) {
      return { serviceIds: ids, serviceName: name };
    }
  }

  // Check for generic "streaming" mentions
  if (/\bstreaming\b/i.test(query)) {
    // Return all major subscription services
    return {
      serviceIds: [
        ...MAJOR_STREAMING_SERVICES.netflix,
        ...MAJOR_STREAMING_SERVICES.amazon_prime,
        ...MAJOR_STREAMING_SERVICES.disney_plus,
        ...MAJOR_STREAMING_SERVICES.hulu,
        ...MAJOR_STREAMING_SERVICES.hbo_max,
        ...MAJOR_STREAMING_SERVICES.apple_tv_plus,
        ...MAJOR_STREAMING_SERVICES.paramount_plus,
        ...MAJOR_STREAMING_SERVICES.peacock,
      ],
      serviceName: null,
    };
  }

  // Check for "free" mentions
  if (/\bfree\b/i.test(query)) {
    return {
      serviceIds: [
        ...MAJOR_STREAMING_SERVICES.tubi,
        ...MAJOR_STREAMING_SERVICES.pluto,
        ...MAJOR_STREAMING_SERVICES.freevee,
        ...MAJOR_STREAMING_SERVICES.crackle,
      ],
      serviceName: 'Free services',
    };
  }

  return { serviceIds: [], serviceName: null };
}
