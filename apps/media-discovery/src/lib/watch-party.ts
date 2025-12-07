/**
 * Watch Party - Multi-User Preference Merging
 *
 * Implements intelligent preference merging for group viewing decisions.
 * Features:
 * - Democratic voting on genres
 * - Conflict resolution (veto system)
 * - Mood consensus finding
 * - Fairness tracking across sessions
 */

import type { UserPreferences, SearchResult, MediaContent } from '@/types/media';

// Watch party member with preferences
export interface PartyMember {
  userId: string;
  name: string;
  preferences: {
    favoriteGenres: number[];
    dislikedGenres: number[];
    moodPreferences?: string[];
    recentlyWatched?: number[]; // Content IDs to avoid
    weight?: number; // Optional influence weight (1.0 = normal)
  };
}

// Watch party session
export interface WatchParty {
  partyId: string;
  name?: string;
  members: PartyMember[];
  createdAt: Date;
  fairnessScores: Map<string, number>; // Track whose preferences were used most
  historyLog: PartyHistoryEntry[];
}

// History of party decisions
interface PartyHistoryEntry {
  contentId: number;
  selectedAt: Date;
  satisfaction: Map<string, number>; // Member ID -> satisfaction score 0-1
}

// Merged preferences result
export interface MergedPreferences {
  genres: GenreScore[];
  excludeGenres: number[];
  moods: string[];
  excludeContent: number[];
  weights: Map<string, number>;
  conflicts: ConflictInfo[];
}

interface GenreScore {
  genreId: number;
  score: number;
  supportingMembers: string[];
  opposingMembers: string[];
}

interface ConflictInfo {
  type: 'genre_conflict' | 'mood_conflict' | 'veto';
  description: string;
  members: string[];
  resolution: string;
}

/**
 * Create a new watch party
 */
export function createWatchParty(
  members: PartyMember[],
  name?: string
): WatchParty {
  const partyId = `party_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;

  return {
    partyId,
    name,
    members,
    createdAt: new Date(),
    fairnessScores: new Map(members.map(m => [m.userId, 0])),
    historyLog: [],
  };
}

/**
 * Merge preferences from multiple party members
 * Uses democratic voting with veto for strong dislikes
 */
export function mergePreferences(party: WatchParty): MergedPreferences {
  const members = party.members;
  const genreVotes = new Map<number, { support: string[]; oppose: string[] }>();
  const moodVotes = new Map<string, number>();
  const excludeContent = new Set<number>();
  const conflicts: ConflictInfo[] = [];

  // Collect all genres mentioned
  const allGenres = new Set<number>();
  for (const member of members) {
    member.preferences.favoriteGenres.forEach(g => allGenres.add(g));
    member.preferences.dislikedGenres.forEach(g => allGenres.add(g));
  }

  // Vote on each genre
  for (const genreId of allGenres) {
    const votes = { support: [] as string[], oppose: [] as string[] };

    for (const member of members) {
      const weight = member.preferences.weight ?? 1.0;

      if (member.preferences.favoriteGenres.includes(genreId)) {
        votes.support.push(member.userId);
      } else if (member.preferences.dislikedGenres.includes(genreId)) {
        votes.oppose.push(member.userId);
      }
    }

    genreVotes.set(genreId, votes);
  }

  // Calculate genre scores
  const genres: GenreScore[] = [];
  const excludeGenres: number[] = [];

  for (const [genreId, votes] of genreVotes) {
    const supportCount = votes.support.length;
    const opposeCount = votes.oppose.length;
    const totalMembers = members.length;

    // Veto rule: if >50% actively dislike, exclude it
    if (opposeCount > totalMembers / 2) {
      excludeGenres.push(genreId);
      conflicts.push({
        type: 'veto',
        description: `Genre ${genreId} vetoed by majority`,
        members: votes.oppose,
        resolution: 'Excluded from search',
      });
      continue;
    }

    // Calculate score based on net support
    // Score = (supporters - opposers) / total, normalized to 0-1
    const netSupport = supportCount - opposeCount;
    const score = Math.max(0, (netSupport / totalMembers + 1) / 2);

    if (score > 0.25) { // Only include genres with some support
      genres.push({
        genreId,
        score,
        supportingMembers: votes.support,
        opposingMembers: votes.oppose,
      });
    }

    // Track conflicts for genres with split opinions
    if (supportCount > 0 && opposeCount > 0) {
      conflicts.push({
        type: 'genre_conflict',
        description: `Genre ${genreId}: ${supportCount} like, ${opposeCount} dislike`,
        members: [...votes.support, ...votes.oppose],
        resolution: score > 0.5 ? 'Included with reduced weight' : 'Excluded',
      });
    }
  }

  // Sort genres by score
  genres.sort((a, b) => b.score - a.score);

  // Collect mood preferences
  for (const member of members) {
    if (member.preferences.moodPreferences) {
      for (const mood of member.preferences.moodPreferences) {
        moodVotes.set(mood, (moodVotes.get(mood) || 0) + 1);
      }
    }
  }

  // Get moods with majority support
  const moods = Array.from(moodVotes.entries())
    .filter(([, count]) => count > members.length / 2)
    .map(([mood]) => mood);

  // Collect content to exclude (recently watched by any member)
  for (const member of members) {
    if (member.preferences.recentlyWatched) {
      member.preferences.recentlyWatched.forEach(id => excludeContent.add(id));
    }
  }

  // Calculate fairness weights (boost members who've been underrepresented)
  const weights = calculateFairnessWeights(party);

  return {
    genres,
    excludeGenres,
    moods,
    excludeContent: Array.from(excludeContent),
    weights,
    conflicts,
  };
}

/**
 * Calculate fairness weights to balance representation
 */
function calculateFairnessWeights(party: WatchParty): Map<string, number> {
  const weights = new Map<string, number>();

  if (party.historyLog.length === 0) {
    // No history yet, equal weights
    for (const member of party.members) {
      weights.set(member.userId, 1.0);
    }
    return weights;
  }

  // Calculate average satisfaction per member
  const satisfactionSums = new Map<string, number>();
  const satisfactionCounts = new Map<string, number>();

  for (const entry of party.historyLog) {
    for (const [memberId, satisfaction] of entry.satisfaction) {
      satisfactionSums.set(
        memberId,
        (satisfactionSums.get(memberId) || 0) + satisfaction
      );
      satisfactionCounts.set(
        memberId,
        (satisfactionCounts.get(memberId) || 0) + 1
      );
    }
  }

  // Calculate boost for underrepresented members
  const avgSatisfactions: [string, number][] = [];
  for (const member of party.members) {
    const sum = satisfactionSums.get(member.userId) || 0;
    const count = satisfactionCounts.get(member.userId) || 1;
    avgSatisfactions.push([member.userId, sum / count]);
  }

  // Sort by satisfaction (lowest first)
  avgSatisfactions.sort((a, b) => a[1] - b[1]);

  // Assign weights: lowest satisfaction gets highest boost
  const minSat = avgSatisfactions[0]?.[1] || 0.5;
  const maxSat = avgSatisfactions[avgSatisfactions.length - 1]?.[1] || 0.5;
  const range = Math.max(0.01, maxSat - minSat);

  for (const [memberId, avgSat] of avgSatisfactions) {
    // Weight inversely proportional to satisfaction
    // Range: 0.8 (highest sat) to 1.2 (lowest sat)
    const normalizedSat = (avgSat - minSat) / range;
    const weight = 1.2 - normalizedSat * 0.4;
    weights.set(memberId, weight);
  }

  return weights;
}

/**
 * Score and rank search results for a watch party
 */
export function scoreResultsForParty(
  results: SearchResult[],
  merged: MergedPreferences,
  party: WatchParty
): SearchResult[] {
  // Filter out excluded content
  let filtered = results.filter(
    r => !merged.excludeContent.includes(r.content.id)
  );

  // Filter out excluded genres
  if (merged.excludeGenres.length > 0) {
    filtered = filtered.filter(r => {
      const hasExcludedGenre = r.content.genreIds.some(
        g => merged.excludeGenres.includes(g)
      );
      return !hasExcludedGenre;
    });
  }

  // Score remaining results
  const scored = filtered.map(result => {
    let partyScore = 0;
    const partyReasons: string[] = [];

    // Genre matching
    for (const genre of merged.genres) {
      if (result.content.genreIds.includes(genre.genreId)) {
        partyScore += genre.score * 0.3;

        if (genre.supportingMembers.length > party.members.length / 2) {
          partyReasons.push('Crowd favorite genre');
        } else if (genre.supportingMembers.length > 1) {
          partyReasons.push('Shared interest');
        }
      }
    }

    // Boost for high ratings (group-safe choice)
    if (result.content.voteAverage >= 7.5) {
      partyScore += 0.1;
      partyReasons.push('Highly rated');
    }

    // Boost for popular content (more likely to satisfy everyone)
    if (result.content.popularity > 100) {
      partyScore += 0.05;
    }

    return {
      ...result,
      relevanceScore: Math.min(1, result.relevanceScore + partyScore),
      matchReasons: [...new Set([...result.matchReasons, ...partyReasons])],
    };
  });

  // Sort by party-adjusted score
  return scored.sort((a, b) => b.relevanceScore - a.relevanceScore);
}

/**
 * Record a party decision for fairness tracking
 */
export function recordPartyDecision(
  party: WatchParty,
  content: MediaContent,
  memberSatisfaction: Map<string, number>
): void {
  party.historyLog.push({
    contentId: content.id,
    selectedAt: new Date(),
    satisfaction: memberSatisfaction,
  });

  // Update fairness scores
  for (const [memberId, satisfaction] of memberSatisfaction) {
    const current = party.fairnessScores.get(memberId) || 0;
    party.fairnessScores.set(memberId, current + satisfaction);
  }
}

/**
 * Get party statistics
 */
export function getPartyStats(party: WatchParty): {
  totalDecisions: number;
  averageSatisfactionByMember: Map<string, number>;
  mostAccommodatedMember: string | null;
  leastAccommodatedMember: string | null;
} {
  const satisfactionTotals = new Map<string, number>();
  const satisfactionCounts = new Map<string, number>();

  for (const entry of party.historyLog) {
    for (const [memberId, satisfaction] of entry.satisfaction) {
      satisfactionTotals.set(
        memberId,
        (satisfactionTotals.get(memberId) || 0) + satisfaction
      );
      satisfactionCounts.set(
        memberId,
        (satisfactionCounts.get(memberId) || 0) + 1
      );
    }
  }

  const averages = new Map<string, number>();
  for (const [memberId, total] of satisfactionTotals) {
    const count = satisfactionCounts.get(memberId) || 1;
    averages.set(memberId, total / count);
  }

  // Find most/least accommodated
  let most: [string, number] | null = null;
  let least: [string, number] | null = null;

  for (const [memberId, avg] of averages) {
    if (!most || avg > most[1]) most = [memberId, avg];
    if (!least || avg < least[1]) least = [memberId, avg];
  }

  return {
    totalDecisions: party.historyLog.length,
    averageSatisfactionByMember: averages,
    mostAccommodatedMember: most?.[0] || null,
    leastAccommodatedMember: least?.[0] || null,
  };
}

/**
 * Generate a compromise search query for the party
 */
export function generatePartySearchQuery(merged: MergedPreferences): string {
  const parts: string[] = [];

  // Add top genres
  const topGenres = merged.genres.slice(0, 3);
  if (topGenres.length > 0) {
    // Note: In real implementation, map genre IDs to names
    parts.push('genres everyone enjoys');
  }

  // Add moods
  if (merged.moods.length > 0) {
    parts.push(merged.moods.join(' and '));
  }

  // Add fairness note if applicable
  if (merged.conflicts.length > 0) {
    parts.push('something for everyone');
  }

  return parts.join(', ') || 'something everyone can enjoy';
}
