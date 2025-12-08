/**
 * Watch Tracking Service
 *
 * Tracks user watch sessions for content consumption.
 */

import { v4 as uuidv4 } from 'uuid';
import { WatchSession } from '../types/feedback.js';
import { NotFoundError } from '../utils/errors.js';

export class WatchTracker {
  private sessions: Map<string, WatchSession>;

  constructor() {
    this.sessions = new Map();
  }

  /**
   * Start a new watch session
   */
  startSession(
    userId: string,
    contentId: string,
    contentTitle: string
  ): WatchSession {
    const sessionId = `watch_${uuidv4()}`;

    const session: WatchSession = {
      sessionId,
      userId,
      contentId,
      contentTitle,
      startTime: new Date(),
      duration: 0,
      completed: false,
      paused: false,
      pauseCount: 0,
    };

    this.sessions.set(sessionId, session);
    return session;
  }

  /**
   * Get active session
   */
  getSession(sessionId: string): WatchSession {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new NotFoundError('Watch session', sessionId);
    }
    return session;
  }

  /**
   * Pause session
   */
  pauseSession(sessionId: string): WatchSession {
    const session = this.getSession(sessionId);

    if (!session.paused && !session.endTime) {
      session.paused = true;
      session.pauseCount += 1;
      session.duration = Date.now() - session.startTime.getTime();
    }

    return session;
  }

  /**
   * Resume session
   */
  resumeSession(sessionId: string): WatchSession {
    const session = this.getSession(sessionId);

    if (session.paused) {
      session.paused = false;
      // Adjust start time to account for pause
      const now = Date.now();
      session.startTime = new Date(now - session.duration);
    }

    return session;
  }

  /**
   * End session
   */
  endSession(sessionId: string, completed: boolean): WatchSession {
    const session = this.getSession(sessionId);

    if (!session.endTime) {
      const now = new Date();
      session.endTime = now;
      session.duration = now.getTime() - session.startTime.getTime();
      session.completed = completed;
    }

    return session;
  }

  /**
   * Get elapsed time for active session
   */
  getElapsedTime(sessionId: string): number {
    const session = this.getSession(sessionId);

    if (session.endTime) {
      return session.duration;
    }

    if (session.paused) {
      return session.duration;
    }

    return Date.now() - session.startTime.getTime();
  }

  /**
   * Clean up old sessions (older than 24 hours)
   */
  cleanup(): void {
    const cutoff = Date.now() - 24 * 60 * 60 * 1000; // 24 hours ago

    for (const [sessionId, session] of this.sessions.entries()) {
      if (session.startTime.getTime() < cutoff) {
        this.sessions.delete(sessionId);
      }
    }
  }

  /**
   * Get all active sessions for a user
   */
  getUserSessions(userId: string): WatchSession[] {
    return Array.from(this.sessions.values()).filter(
      session => session.userId === userId && !session.endTime
    );
  }
}
