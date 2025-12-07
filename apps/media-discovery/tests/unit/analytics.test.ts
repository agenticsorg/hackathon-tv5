/**
 * Analytics Tests
 *
 * Tests search quality metrics tracking
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  trackSearchInitiated,
  trackSearchCompleted,
  trackResultInteraction,
  trackFeedback,
  trackEvent,
  getSearchMetrics,
  getRealTimeMetrics,
  getSessionAnalytics,
  clearAnalytics,
} from '@/lib/analytics';

describe('Search Analytics', () => {
  const testSessionId = `session_${Date.now()}`;

  beforeEach(() => {
    clearAnalytics();
  });

  describe('Event Tracking', () => {
    it('should track search initiation', () => {
      const eventId = trackSearchInitiated(testSessionId, 'action movies');

      expect(eventId).toMatch(/^evt_\d+_\w+$/);
    });

    it('should track search completion with metrics', () => {
      trackSearchCompleted(testSessionId, 'comedy films', {
        count: 25,
        latencyMs: 150,
        cacheHit: false,
        topIds: [1, 2, 3, 4, 5],
      });

      const metrics = getRealTimeMetrics();

      expect(metrics.totalSearches).toBeGreaterThan(0);
    });

    it('should track voice search flag', () => {
      trackSearchInitiated(testSessionId, 'find me a thriller', {
        voiceSearch: true,
      });

      trackSearchCompleted(testSessionId, 'find me a thriller', {
        count: 10,
        latencyMs: 200,
        cacheHit: false,
        topIds: [1, 2, 3],
      }, {
        voiceSearch: true,
      });

      const metrics = getRealTimeMetrics();

      expect(metrics.voiceSearchRate).toBeGreaterThan(0);
    });

    it('should track result interactions', () => {
      trackResultInteraction(testSessionId, 'clicked', {
        contentId: 12345,
        contentType: 'movie',
        position: 1,
        relevanceScore: 0.95,
        query: 'action movies',
      });

      const sessionStats = getSessionAnalytics(testSessionId);

      expect(sessionStats.clicks).toBe(1);
    });

    it('should track feedback', () => {
      trackFeedback(testSessionId, 12345, 'like', {
        query: 'comedy films',
      });

      // Feedback should be recorded (no direct getter, but should not throw)
    });

    it('should track generic events', () => {
      trackEvent(testSessionId, 'watch_party_created', {
        memberCount: 4,
      });

      // Generic events should be recorded
    });
  });

  describe('Metrics Calculation', () => {
    it('should calculate average latency', () => {
      // Simulate 3 searches with different latencies
      for (let i = 0; i < 3; i++) {
        trackSearchCompleted(testSessionId, `query_${i}`, {
          count: 10,
          latencyMs: 100 + i * 50, // 100, 150, 200
          cacheHit: false,
          topIds: [1],
        });
      }

      const metrics = getRealTimeMetrics();

      expect(metrics.avgLatencyMs).toBe(150); // (100 + 150 + 200) / 3
    });

    it('should calculate cache hit rate', () => {
      // 1 cache hit, 2 misses
      trackSearchCompleted(testSessionId, 'query1', {
        count: 10,
        latencyMs: 10,
        cacheHit: true,
        topIds: [1],
      });

      trackSearchCompleted(testSessionId, 'query2', {
        count: 10,
        latencyMs: 100,
        cacheHit: false,
        topIds: [1],
      });

      trackSearchCompleted(testSessionId, 'query3', {
        count: 10,
        latencyMs: 100,
        cacheHit: false,
        topIds: [1],
      });

      const metrics = getRealTimeMetrics();

      expect(metrics.cacheHitRate).toBeCloseTo(0.33, 1);
    });

    it('should calculate click-through rate', () => {
      // 2 searches
      trackSearchCompleted(testSessionId, 'query1', {
        count: 10,
        latencyMs: 100,
        cacheHit: false,
        topIds: [1],
      });

      trackSearchCompleted(testSessionId, 'query2', {
        count: 10,
        latencyMs: 100,
        cacheHit: false,
        topIds: [1],
      });

      // 1 click
      trackResultInteraction(testSessionId, 'clicked', {
        contentId: 123,
        contentType: 'movie',
        position: 1,
        relevanceScore: 0.9,
        query: 'query1',
      });

      const metrics = getRealTimeMetrics();

      expect(metrics.clickThroughRate).toBe(0.5); // 1 search with click / 2 total
    });

    it('should track top queries', () => {
      // Same query multiple times
      for (let i = 0; i < 5; i++) {
        trackSearchCompleted(testSessionId, 'popular query', {
          count: 10,
          latencyMs: 100,
          cacheHit: false,
          topIds: [1],
        });
      }

      trackSearchCompleted(testSessionId, 'another query', {
        count: 10,
        latencyMs: 100,
        cacheHit: false,
        topIds: [1],
      });

      const metrics = getRealTimeMetrics();

      expect(metrics.topQueries.length).toBeGreaterThan(0);
      expect(metrics.topQueries[0].query).toBe('popular query');
      expect(metrics.topQueries[0].count).toBe(5);
    });
  });

  describe('Session Analytics', () => {
    it('should return session-specific stats', () => {
      const session1 = 'session_1';
      const session2 = 'session_2';

      // Session 1: 3 searches, 2 clicks
      for (let i = 0; i < 3; i++) {
        trackSearchCompleted(session1, `query_${i}`, {
          count: 10,
          latencyMs: 100,
          cacheHit: false,
          topIds: [1],
        });
      }

      trackResultInteraction(session1, 'clicked', {
        contentId: 1,
        contentType: 'movie',
        position: 1,
        relevanceScore: 0.9,
        query: 'query_0',
      });

      trackResultInteraction(session1, 'clicked', {
        contentId: 2,
        contentType: 'movie',
        position: 2,
        relevanceScore: 0.8,
        query: 'query_1',
      });

      // Session 2: 1 search
      trackSearchCompleted(session2, 'different query', {
        count: 5,
        latencyMs: 150,
        cacheHit: true,
        topIds: [1],
      });

      const stats1 = getSessionAnalytics(session1);
      const stats2 = getSessionAnalytics(session2);

      expect(stats1.searches).toBe(3);
      expect(stats1.clicks).toBe(2);
      expect(stats2.searches).toBe(1);
      expect(stats2.clicks).toBe(0);
    });
  });

  describe('Time-Based Metrics', () => {
    it('should filter by time range', () => {
      const now = new Date();
      const oneHourAgo = new Date(now.getTime() - 60 * 60 * 1000);
      const twoHoursAgo = new Date(now.getTime() - 2 * 60 * 60 * 1000);

      // Track some events
      trackSearchCompleted(testSessionId, 'recent query', {
        count: 10,
        latencyMs: 100,
        cacheHit: false,
        topIds: [1],
      });

      const hourMetrics = getSearchMetrics(oneHourAgo, now, 'hour');

      expect(hourMetrics.totalSearches).toBeGreaterThan(0);
      expect(hourMetrics.period).toBe('hour');
    });
  });
});
