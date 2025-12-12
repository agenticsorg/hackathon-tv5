/**
 * User Preference Tracking System
 * Manages user preferences, watch history, and preference learning
 */

import type { UserPreferences, WatchHistoryEntry, MediaContent, Recommendation } from '@/types/media';

// In-memory store for development (would be Firestore in production)
const userPreferencesStore = new Map<string, UserPreferences>();

// Preference learning weights
const LEARNING_WEIGHTS = {
  watch_completed: 0.3,
  watch_partial: 0.1,
  explicit_like: 0.5,
  explicit_dislike: -0.4,
  high_rating: 0.4,
  low_rating: -0.2,
};

/**
 * Initialize or get user preferences
 */
export async function getUserPreferences(userId: string): Promise<UserPreferences> {
  let prefs = userPreferencesStore.get(userId);

  if (!prefs) {
    prefs = {
      userId,
      favoriteGenres: [],
      likedContent: [],
      dislikedContent: [],
      watchHistory: [],
      preferenceVector: undefined,
      updatedAt: new Date(),
    };
    userPreferencesStore.set(userId, prefs);
  }

  return prefs;
}

/**
 * Save user preferences
 */
export async function saveUserPreferences(prefs: UserPreferences): Promise<void> {
  prefs.updatedAt = new Date();
  userPreferencesStore.set(prefs.userId, prefs);
}

/**
 * Record content view/watch
 */
export async function recordWatch(
  userId: string,
  contentId: number,
  mediaType: 'movie' | 'tv',
  progress: number,
  rating?: number
): Promise<void> {
  const prefs = await getUserPreferences(userId);

  // Check if already in history
  const existingIndex = prefs.watchHistory.findIndex(
    h => h.contentId === contentId && h.mediaType === mediaType
  );

  const entry: WatchHistoryEntry = {
    contentId,
    mediaType,
    watchedAt: new Date(),
    progress,
    rating,
    completed: progress >= 0.9,
  };

  if (existingIndex >= 0) {
    // Update existing entry
    const existing = prefs.watchHistory[existingIndex];
    prefs.watchHistory[existingIndex] = {
      ...entry,
      progress: Math.max(existing.progress, progress),
      rating: rating ?? existing.rating,
      completed: existing.completed || entry.completed,
    };
  } else {
    // Add new entry
    prefs.watchHistory.push(entry);
  }

  // Keep only last 500 entries
  if (prefs.watchHistory.length > 500) {
    prefs.watchHistory = prefs.watchHistory.slice(-500);
  }

  await saveUserPreferences(prefs);
}

/**
 * Record explicit like/dislike
 */
export async function recordFeedback(
  userId: string,
  contentId: number,
  feedback: 'like' | 'dislike'
): Promise<void> {
  const prefs = await getUserPreferences(userId);

  if (feedback === 'like') {
    if (!prefs.likedContent.includes(contentId)) {
      prefs.likedContent.push(contentId);
    }
    // Remove from disliked if present
    prefs.dislikedContent = prefs.dislikedContent.filter(id => id !== contentId);
  } else {
    if (!prefs.dislikedContent.includes(contentId)) {
      prefs.dislikedContent.push(contentId);
    }
    // Remove from liked if present
    prefs.likedContent = prefs.likedContent.filter(id => id !== contentId);
  }

  await saveUserPreferences(prefs);
}

/**
 * Update favorite genres based on viewing behavior
 */
export async function updateGenrePreferences(
  userId: string,
  content: MediaContent,
  signal: 'watch' | 'complete' | 'like' | 'dislike' | 'rate',
  rating?: number
): Promise<void> {
  const prefs = await getUserPreferences(userId);

  // Calculate score impact
  let impact = 0;
  switch (signal) {
    case 'watch':
      impact = LEARNING_WEIGHTS.watch_partial;
      break;
    case 'complete':
      impact = LEARNING_WEIGHTS.watch_completed;
      break;
    case 'like':
      impact = LEARNING_WEIGHTS.explicit_like;
      break;
    case 'dislike':
      impact = LEARNING_WEIGHTS.explicit_dislike;
      break;
    case 'rate':
      impact = rating && rating >= 7 ? LEARNING_WEIGHTS.high_rating : LEARNING_WEIGHTS.low_rating;
      break;
  }

  // Update genre scores
  const genreScores = new Map<number, number>();

  // Initialize with existing preferences
  for (const genreId of prefs.favoriteGenres) {
    genreScores.set(genreId, 1);
  }

  // Apply impact to content genres
  for (const genreId of content.genreIds) {
    const current = genreScores.get(genreId) || 0;
    genreScores.set(genreId, current + impact);
  }

  // Sort by score and take top genres
  const sortedGenres = Array.from(genreScores.entries())
    .filter(([_, score]) => score > 0)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 10)
    .map(([id]) => id);

  prefs.favoriteGenres = sortedGenres;
  await saveUserPreferences(prefs);
}

/**
 * Generate preference vector from user behavior
 * This vector can be used for similarity matching with content embeddings
 */
export async function generatePreferenceVector(userId: string): Promise<number[] | undefined> {
  const prefs = await getUserPreferences(userId);

  if (prefs.watchHistory.length < 5) {
    // Not enough data to generate meaningful vector
    return undefined;
  }

  // In production, this would aggregate content embeddings
  // For now, return undefined to indicate not enough data
  return undefined;
}

/**
 * Get personalized recommendations based on preferences
 */
export async function getPersonalizedScore(
  userId: string,
  content: MediaContent
): Promise<number> {
  const prefs = await getUserPreferences(userId);
  let score = 0;

  // Boost for matching genres
  const matchingGenres = content.genreIds.filter(id => prefs.favoriteGenres.includes(id));
  score += matchingGenres.length * 0.1;

  // Penalty if disliked
  if (prefs.dislikedContent.includes(content.id)) {
    score -= 0.5;
  }

  // Small boost if already liked (for related content)
  if (prefs.likedContent.includes(content.id)) {
    score += 0.2;
  }

  // Penalty if already watched recently
  const recentWatch = prefs.watchHistory.find(
    h => h.contentId === content.id && h.mediaType === content.mediaType
  );
  if (recentWatch && recentWatch.completed) {
    const daysSinceWatch = (Date.now() - recentWatch.watchedAt.getTime()) / (1000 * 60 * 60 * 24);
    if (daysSinceWatch < 30) {
      score -= 0.3;
    }
  }

  return Math.max(0, Math.min(1, score));
}

/**
 * Calculate recommendation adjustments based on user feedback
 */
export async function adjustRecommendations(
  userId: string,
  recommendations: Recommendation[]
): Promise<Recommendation[]> {
  const prefs = await getUserPreferences(userId);

  return recommendations.map(rec => {
    const personalizedScore = getPersonalizedScoreSync(prefs, rec.content);
    const adjustedScore = rec.score + personalizedScore * 0.3;

    return {
      ...rec,
      score: Math.max(0, Math.min(1, adjustedScore)),
      reasons: personalizedScore > 0
        ? [...rec.reasons, 'Matches your preferences']
        : rec.reasons,
    };
  }).sort((a, b) => b.score - a.score);
}

// Sync version for use in map operations
function getPersonalizedScoreSync(prefs: UserPreferences, content: MediaContent): number {
  let score = 0;

  const matchingGenres = content.genreIds.filter(id => prefs.favoriteGenres.includes(id));
  score += matchingGenres.length * 0.1;

  if (prefs.dislikedContent.includes(content.id)) {
    score -= 0.5;
  }

  if (prefs.likedContent.includes(content.id)) {
    score += 0.2;
  }

  return score;
}

/**
 * Export user preferences (for data portability)
 */
export async function exportUserData(userId: string): Promise<UserPreferences | null> {
  return userPreferencesStore.get(userId) || null;
}

/**
 * Delete user preferences (for GDPR compliance)
 */
export async function deleteUserData(userId: string): Promise<void> {
  userPreferencesStore.delete(userId);
}
