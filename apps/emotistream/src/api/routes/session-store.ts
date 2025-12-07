import { FileStore } from '../../persistence/file-store.js';

/**
 * Session data stored for each user-content pair to track emotional state
 * when recommendations are generated
 */
export interface SessionData {
  stateBefore: {
    valence: number;
    arousal: number;
    stress: number;
    confidence: number;
  };
  desiredState: {
    targetValence: number;
    targetArousal: number;
    targetStress: number;
    intensity: 'subtle' | 'moderate' | 'significant';
    reasoning: string;
  };
  contentId: string;
  timestamp: number;
}

/**
 * Shared session store using FileStore for persistence across restarts
 * Stores emotional state when recommendations are generated for later feedback processing
 */
export const sessionStore = new FileStore<SessionData>('sessions.json');

/**
 * Clean up old sessions (older than 24 hours)
 * Should be called periodically to prevent unbounded growth
 */
export function cleanupSessions(): void {
  const oneDayAgo = Date.now() - 24 * 60 * 60 * 1000;
  let cleanedCount = 0;

  for (const [key, session] of sessionStore.entries()) {
    if (session.timestamp < oneDayAgo) {
      sessionStore.delete(key);
      cleanedCount++;
    }
  }

  if (cleanedCount > 0) {
    console.log(`Cleaned up ${cleanedCount} old sessions`);
  }
}

/**
 * Schedule periodic cleanup of old sessions
 * Runs every hour
 */
export function scheduleCleanup(): NodeJS.Timeout {
  const ONE_HOUR = 60 * 60 * 1000;
  return setInterval(() => {
    cleanupSessions();
  }, ONE_HOUR);
}
