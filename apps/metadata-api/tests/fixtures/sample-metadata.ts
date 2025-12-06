/**
 * Test Fixtures for Platform Connector Tests
 * Sample metadata for testing Netflix IMF, Amazon MEC, and FAST MRSS connectors
 */

import { MediaMetadata } from '../../src/types';

/**
 * Complete metadata with all fields for comprehensive testing
 */
export const completeMetadata: MediaMetadata = {
  id: 'asset-001',
  eidr: '10.5240/AAAA-BBBB-CCCC-DDDD-EEEE-1',
  title: 'The Quantum Paradox',
  type: 'movie',
  releaseDate: new Date('2024-03-15'),
  duration: 142,
  synopsis: 'A mind-bending thriller about quantum physics and parallel universes.',
  description: 'When a brilliant physicist discovers a way to communicate across parallel dimensions, she must race against time to prevent a catastrophic collision of realities.',
  genres: ['sci-fi', 'thriller', 'mystery'],
  keywords: ['quantum physics', 'parallel universes', 'time travel', 'conspiracy'],
  moodTags: ['suspenseful', 'mind-bending', 'thought-provoking'],
  themes: ['science', 'reality', 'choice', 'consequence'],
  director: ['Sarah Chen', 'Michael Torres'],
  cast: [
    { name: 'Emma Stone', role: 'lead', characterName: 'Dr. Sarah Mitchell', order: 1 },
    { name: 'Oscar Isaac', role: 'supporting', characterName: 'Professor James Reed', order: 2 },
    { name: 'Tilda Swinton', role: 'supporting', characterName: 'The Observer', order: 3 }
  ],
  producers: ['Christopher Nolan', 'Emma Thomas'],
  writers: ['Alex Garland', 'Charlie Kaufman'],
  language: 'en',
  originalLanguage: 'en',
  subtitles: ['en', 'es', 'fr', 'de', 'ja'],
  audioTracks: ['en-5.1', 'en-atmos', 'es-5.1', 'fr-5.1'],
  resolution: '4K',
  aspectRatio: '2.39:1',
  rating: 'PG-13',
  contentWarnings: ['intense sequences', 'brief strong language'],
  platforms: [
    {
      platform: 'netflix',
      region: 'US',
      availableFrom: new Date('2024-06-01'),
      availableUntil: new Date('2025-06-01'),
      url: 'https://netflix.com/watch/quantum-paradox',
      subscriptionRequired: true,
      validated: true,
      validatedAt: new Date('2024-05-15')
    },
    {
      platform: 'amazon',
      region: 'US',
      availableFrom: new Date('2024-06-15'),
      availableUntil: new Date('2025-06-15'),
      url: 'https://amazon.com/dp/quantum-paradox',
      subscriptionRequired: false,
      validated: true,
      validatedAt: new Date('2024-05-20')
    }
  ],
  territories: ['US', 'CA', 'GB', 'DE', 'FR', 'ES', 'IT', 'JP'],
  rightsExpiry: new Date('2027-12-31'),
  embedding: [...Array(1536)].map(() => Math.random()),
  similarContent: ['asset-002', 'asset-003', 'asset-004'],
  popularity: 8.7,
  userRating: 8.5,
  createdAt: new Date('2024-01-10'),
  updatedAt: new Date('2024-05-20'),
  enrichedAt: new Date('2024-05-15')
};

/**
 * Metadata with Dolby Vision technical specs for Netflix IMF testing
 */
export const dolbyVisionMetadata: MediaMetadata = {
  ...completeMetadata,
  id: 'asset-dolby-001',
  title: 'Neon Nights: A Visual Journey',
  resolution: '4K',
  aspectRatio: '2.39:1',
  audioTracks: ['en-atmos', 'en-5.1', 'es-atmos'],
  // Dolby Vision metadata would be in extended technical specs
};

/**
 * Metadata missing required Netflix fields for validation testing
 */
export const incompleteNetflixMetadata: Partial<MediaMetadata> = {
  id: 'asset-incomplete-001',
  title: 'Missing Fields Test',
  type: 'movie',
  genres: ['drama'],
  language: 'en',
  createdAt: new Date(),
  updatedAt: new Date()
  // Missing: EIDR, rating, resolution, audioTracks, etc.
};

/**
 * Multi-territory rights metadata for Amazon MEC testing
 */
export const multiTerritoryMetadata: MediaMetadata = {
  ...completeMetadata,
  id: 'asset-multi-territory-001',
  title: 'Global Premiere',
  territories: ['US', 'CA', 'GB', 'DE', 'FR', 'ES', 'IT', 'JP', 'AU', 'NZ', 'IN', 'BR', 'MX'],
  platforms: [
    {
      platform: 'amazon',
      region: 'US',
      availableFrom: new Date('2024-07-01'),
      availableUntil: new Date('2025-07-01'),
      subscriptionRequired: false,
      validated: true,
      validatedAt: new Date()
    },
    {
      platform: 'amazon',
      region: 'GB',
      availableFrom: new Date('2024-07-01'),
      availableUntil: new Date('2025-07-01'),
      subscriptionRequired: true, // Different subscription model
      validated: true,
      validatedAt: new Date()
    },
    {
      platform: 'amazon',
      region: 'JP',
      availableFrom: new Date('2024-08-01'), // Delayed release
      availableUntil: new Date('2025-08-01'),
      subscriptionRequired: false,
      validated: true,
      validatedAt: new Date()
    }
  ]
};

/**
 * FAST channel metadata with scheduling information
 */
export const fastChannelMetadata: MediaMetadata = {
  id: 'asset-fast-001',
  title: 'Action Movie Marathon',
  type: 'movie',
  releaseDate: new Date('2023-05-10'),
  duration: 105,
  synopsis: 'High-octane action thriller with explosive sequences.',
  description: 'A former special ops agent must stop a terrorist plot in 24 hours.',
  genres: ['action', 'thriller'],
  keywords: ['action', 'explosions', 'special ops', 'terrorism'],
  moodTags: ['intense', 'fast-paced', 'adrenaline'],
  themes: ['heroism', 'justice', 'sacrifice'],
  director: ['John Woo'],
  cast: [
    { name: 'Tom Cruise', role: 'lead', characterName: 'Jack Reacher', order: 1 },
    { name: 'Charlize Theron', role: 'supporting', characterName: 'Agent Maria Santos', order: 2 }
  ],
  language: 'en',
  subtitles: ['en', 'es'],
  audioTracks: ['en-stereo', 'en-5.1'],
  resolution: '1080p',
  aspectRatio: '16:9',
  rating: 'TV-14',
  contentWarnings: ['violence', 'intense action sequences'],
  platforms: [
    {
      platform: 'custom',
      region: 'US',
      availableFrom: new Date('2024-06-01'),
      subscriptionRequired: false,
      validated: true,
      validatedAt: new Date()
    }
  ],
  territories: ['US'],
  popularity: 7.2,
  userRating: 7.5,
  createdAt: new Date('2024-01-01'),
  updatedAt: new Date('2024-05-01')
};

/**
 * Series episode metadata for testing episodic content
 */
export const seriesEpisodeMetadata: MediaMetadata = {
  id: 'asset-episode-s01e01',
  title: 'Pilot',
  type: 'episode',
  releaseDate: new Date('2024-01-15'),
  duration: 47,
  synopsis: 'A mysterious signal from deep space leads to an unexpected discovery.',
  description: 'The crew of the Odyssey investigates a strange transmission that could change humanity forever.',
  genres: ['sci-fi', 'drama'],
  keywords: ['space', 'discovery', 'mystery', 'alien'],
  moodTags: ['mysterious', 'epic', 'wonder'],
  themes: ['exploration', 'first contact', 'unknown'],
  director: ['Denis Villeneuve'],
  cast: [
    { name: 'Rebecca Ferguson', role: 'lead', characterName: 'Captain Elena Vasquez', order: 1 },
    { name: 'John Boyega', role: 'supporting', characterName: 'Engineer Marcus Wright', order: 2 }
  ],
  producers: ['J.J. Abrams'],
  writers: ['Lisa Joy', 'Jonathan Nolan'],
  language: 'en',
  subtitles: ['en', 'es', 'fr'],
  audioTracks: ['en-5.1', 'en-stereo'],
  resolution: '4K',
  aspectRatio: '16:9',
  rating: 'TV-MA',
  contentWarnings: ['violence', 'adult themes'],
  platforms: [
    {
      platform: 'netflix',
      region: 'US',
      availableFrom: new Date('2024-01-15'),
      subscriptionRequired: true,
      validated: true,
      validatedAt: new Date()
    }
  ],
  territories: ['US', 'CA', 'GB'],
  popularity: 9.1,
  userRating: 9.3,
  createdAt: new Date('2024-01-01'),
  updatedAt: new Date('2024-01-15')
};

/**
 * Minimal valid metadata for basic connector testing
 */
export const minimalMetadata: MediaMetadata = {
  id: 'asset-minimal-001',
  title: 'Minimal Test Asset',
  type: 'movie',
  genres: ['drama'],
  keywords: [],
  language: 'en',
  createdAt: new Date('2024-01-01'),
  updatedAt: new Date('2024-01-01')
};
