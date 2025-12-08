/**
 * Watch Party Logic Tests
 *
 * Tests preference merging and group recommendation algorithms
 */

import { describe, it, expect } from 'vitest';
import {
  createWatchParty,
  mergePreferences,
  scoreResultsForParty,
  generatePartySearchQuery,
  recordPartyDecision,
  getPartyStats,
  type PartyMember,
  type WatchParty,
} from '@/lib/watch-party';
import type { SearchResult, MediaContent } from '@/types/media';

// Test fixtures
const createMockMember = (overrides: Partial<PartyMember> = {}): PartyMember => ({
  userId: `user_${Math.random().toString(36).slice(2)}`,
  name: 'Test User',
  preferences: {
    favoriteGenres: [28, 35], // Action, Comedy
    dislikedGenres: [],
    moodPreferences: [],
    recentlyWatched: [],
  },
  ...overrides,
});

const createMockSearchResult = (overrides: Partial<SearchResult> = {}): SearchResult => ({
  content: {
    id: Math.floor(Math.random() * 100000),
    title: 'Test Movie',
    overview: 'A test movie',
    posterPath: null,
    backdropPath: null,
    releaseDate: '2024-01-01',
    voteAverage: 7.5,
    voteCount: 1000,
    popularity: 100,
    genreIds: [28],
    mediaType: 'movie',
  },
  relevanceScore: 0.8,
  matchReasons: ['Test match'],
  ...overrides,
});

describe('Watch Party Creation', () => {
  it('should create a party with unique ID', () => {
    const members = [createMockMember(), createMockMember()];
    const party = createWatchParty(members, 'Movie Night');

    expect(party.partyId).toMatch(/^party_\d+_\w+$/);
    expect(party.name).toBe('Movie Night');
    expect(party.members).toHaveLength(2);
    expect(party.historyLog).toEqual([]);
    expect(party.fairnessScores.size).toBe(2);
  });

  it('should initialize fairness scores for all members', () => {
    const members = [
      createMockMember({ userId: 'alice' }),
      createMockMember({ userId: 'bob' }),
      createMockMember({ userId: 'carol' }),
    ];
    const party = createWatchParty(members);

    for (const member of members) {
      expect(party.fairnessScores.get(member.userId)).toBe(0);
    }
  });
});

describe('Preference Merging', () => {
  it('should merge shared genre preferences', () => {
    const members = [
      createMockMember({
        userId: 'a',
        preferences: { favoriteGenres: [28, 35], dislikedGenres: [] },
      }),
      createMockMember({
        userId: 'b',
        preferences: { favoriteGenres: [28, 18], dislikedGenres: [] },
      }),
    ];
    const party = createWatchParty(members);
    const merged = mergePreferences(party);

    // Action (28) should have high score (both like it)
    const actionGenre = merged.genres.find(g => g.genreId === 28);
    expect(actionGenre).toBeDefined();
    expect(actionGenre?.supportingMembers).toContain('a');
    expect(actionGenre?.supportingMembers).toContain('b');
    expect(actionGenre?.score).toBeGreaterThan(0.5);
  });

  it('should veto genres disliked by majority', () => {
    const members = [
      createMockMember({
        userId: 'a',
        preferences: { favoriteGenres: [28], dislikedGenres: [27] }, // Dislikes Horror
      }),
      createMockMember({
        userId: 'b',
        preferences: { favoriteGenres: [28], dislikedGenres: [27] }, // Dislikes Horror
      }),
      createMockMember({
        userId: 'c',
        preferences: { favoriteGenres: [27], dislikedGenres: [] }, // Likes Horror
      }),
    ];
    const party = createWatchParty(members);
    const merged = mergePreferences(party);

    // Horror (27) should be excluded (2/3 dislike)
    expect(merged.excludeGenres).toContain(27);

    // Should have a veto conflict recorded
    expect(merged.conflicts.some(c => c.type === 'veto')).toBe(true);
  });

  it('should track genre conflicts when opinions differ', () => {
    const members = [
      createMockMember({
        userId: 'a',
        preferences: { favoriteGenres: [10749], dislikedGenres: [] }, // Romance
      }),
      createMockMember({
        userId: 'b',
        preferences: { favoriteGenres: [], dislikedGenres: [10749] }, // Dislikes Romance
      }),
    ];
    const party = createWatchParty(members);
    const merged = mergePreferences(party);

    const romanceConflict = merged.conflicts.find(
      c => c.type === 'genre_conflict' && c.description.includes('10749')
    );
    expect(romanceConflict).toBeDefined();
  });

  it('should exclude recently watched content', () => {
    const members = [
      createMockMember({
        userId: 'a',
        preferences: {
          favoriteGenres: [28],
          dislikedGenres: [],
          recentlyWatched: [123, 456],
        },
      }),
      createMockMember({
        userId: 'b',
        preferences: {
          favoriteGenres: [28],
          dislikedGenres: [],
          recentlyWatched: [789],
        },
      }),
    ];
    const party = createWatchParty(members);
    const merged = mergePreferences(party);

    expect(merged.excludeContent).toContain(123);
    expect(merged.excludeContent).toContain(456);
    expect(merged.excludeContent).toContain(789);
  });
});

describe('Result Scoring for Party', () => {
  it('should filter out excluded content', () => {
    const members = [
      createMockMember({
        preferences: {
          favoriteGenres: [28],
          dislikedGenres: [],
          recentlyWatched: [999],
        },
      }),
      createMockMember({
        preferences: { favoriteGenres: [28], dislikedGenres: [] },
      }),
    ];
    const party = createWatchParty(members);
    const merged = mergePreferences(party);

    const results = [
      createMockSearchResult({ content: { ...createMockSearchResult().content, id: 999 } }),
      createMockSearchResult({ content: { ...createMockSearchResult().content, id: 888 } }),
    ];

    const scored = scoreResultsForParty(results, merged, party);

    expect(scored.find(r => r.content.id === 999)).toBeUndefined();
    expect(scored.find(r => r.content.id === 888)).toBeDefined();
  });

  it('should boost highly rated content', () => {
    const members = [
      createMockMember({ preferences: { favoriteGenres: [28], dislikedGenres: [] } }),
      createMockMember({ preferences: { favoriteGenres: [28], dislikedGenres: [] } }),
    ];
    const party = createWatchParty(members);
    const merged = mergePreferences(party);

    const highRated = createMockSearchResult({
      content: { ...createMockSearchResult().content, id: 1, voteAverage: 8.5, voteCount: 5000, genreIds: [28] },
      relevanceScore: 0.5,
    });
    const lowRated = createMockSearchResult({
      content: { ...createMockSearchResult().content, id: 2, voteAverage: 5.0, voteCount: 100, genreIds: [28] },
      relevanceScore: 0.5,
    });

    const scored = scoreResultsForParty([highRated, lowRated], merged, party);

    // High rated should score higher (with same base score, high rating should boost)
    const highRatedResult = scored.find(r => r.content.id === 1);
    const lowRatedResult = scored.find(r => r.content.id === 2);

    expect(highRatedResult).toBeDefined();
    expect(lowRatedResult).toBeDefined();
    expect(highRatedResult!.relevanceScore).toBeGreaterThanOrEqual(lowRatedResult!.relevanceScore);
  });
});

describe('Party History and Fairness', () => {
  it('should record party decisions', () => {
    const members = [
      createMockMember({ userId: 'alice' }),
      createMockMember({ userId: 'bob' }),
    ];
    const party = createWatchParty(members);

    const content: MediaContent = {
      id: 123,
      title: 'Test Movie',
      overview: '',
      posterPath: null,
      backdropPath: null,
      releaseDate: '2024-01-01',
      voteAverage: 7.5,
      voteCount: 100,
      popularity: 50,
      genreIds: [28],
      mediaType: 'movie',
    };

    const satisfaction = new Map([
      ['alice', 0.9],
      ['bob', 0.6],
    ]);

    recordPartyDecision(party, content, satisfaction);

    expect(party.historyLog).toHaveLength(1);
    expect(party.historyLog[0].contentId).toBe(123);
    expect(party.fairnessScores.get('alice')).toBe(0.9);
    expect(party.fairnessScores.get('bob')).toBe(0.6);
  });

  it('should calculate party stats correctly', () => {
    const members = [
      createMockMember({ userId: 'alice', name: 'Alice' }),
      createMockMember({ userId: 'bob', name: 'Bob' }),
    ];
    const party = createWatchParty(members);

    // Simulate 3 decisions
    const content: MediaContent = {
      id: 1,
      title: 'Movie',
      overview: '',
      posterPath: null,
      backdropPath: null,
      releaseDate: '2024-01-01',
      voteAverage: 7,
      voteCount: 100,
      popularity: 50,
      genreIds: [],
      mediaType: 'movie',
    };

    recordPartyDecision(party, { ...content, id: 1 }, new Map([['alice', 0.9], ['bob', 0.5]]));
    recordPartyDecision(party, { ...content, id: 2 }, new Map([['alice', 0.8], ['bob', 0.6]]));
    recordPartyDecision(party, { ...content, id: 3 }, new Map([['alice', 0.7], ['bob', 0.7]]));

    const stats = getPartyStats(party);

    expect(stats.totalDecisions).toBe(3);
    expect(stats.mostAccommodatedMember).toBe('alice');
    expect(stats.leastAccommodatedMember).toBe('bob');

    const aliceAvg = stats.averageSatisfactionByMember.get('alice');
    const bobAvg = stats.averageSatisfactionByMember.get('bob');

    expect(aliceAvg).toBeCloseTo(0.8);
    expect(bobAvg).toBeCloseTo(0.6);
  });
});

describe('Party Search Query Generation', () => {
  it('should generate meaningful search query from merged preferences', () => {
    const members = [
      createMockMember({
        preferences: {
          favoriteGenres: [28],
          dislikedGenres: [],
          moodPreferences: ['exciting', 'fun'],
        },
      }),
      createMockMember({
        preferences: {
          favoriteGenres: [28],
          dislikedGenres: [],
          moodPreferences: ['exciting'],
        },
      }),
    ];
    const party = createWatchParty(members);
    const merged = mergePreferences(party);

    const query = generatePartySearchQuery(merged);

    expect(query.length).toBeGreaterThan(0);
    expect(query).toContain('everyone');
  });
});
