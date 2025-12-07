/**
 * Search Quality Analytics
 *
 * Tracks and analyzes search performance metrics:
 * - Query patterns and intent distribution
 * - Result quality signals (clicks, engagement)
 * - Latency tracking
 * - A/B test support
 */

// Analytics event types
export type AnalyticsEventType =
  | 'search_initiated'
  | 'search_completed'
  | 'result_clicked'
  | 'result_viewed'
  | 'result_selected'
  | 'session_started'
  | 'session_ended'
  | 'voice_search_used'
  | 'filter_applied'
  | 'streaming_filter_used'
  | 'watch_party_created'
  | 'recommendation_feedback';

// Base analytics event
interface BaseAnalyticsEvent {
  eventId: string;
  eventType: AnalyticsEventType;
  timestamp: Date;
  sessionId: string;
  userId?: string;
  metadata?: Record<string, unknown>;
}

// Search-specific event
export interface SearchEvent extends BaseAnalyticsEvent {
  eventType: 'search_initiated' | 'search_completed';
  query: string;
  queryLength: number;
  intentDetected?: string[];
  streamingServiceMentioned?: string;
  voiceSearch: boolean;
}

// Search completion event
export interface SearchCompletedEvent extends SearchEvent {
  eventType: 'search_completed';
  resultCount: number;
  latencyMs: number;
  cacheHit: boolean;
  topResultIds: number[];
}

// Result interaction event
export interface ResultInteractionEvent extends BaseAnalyticsEvent {
  eventType: 'result_clicked' | 'result_viewed' | 'result_selected';
  contentId: number;
  contentType: 'movie' | 'tv';
  position: number;
  relevanceScore: number;
  query: string;
}

// Feedback event
export interface FeedbackEvent extends BaseAnalyticsEvent {
  eventType: 'recommendation_feedback';
  contentId: number;
  feedbackType: 'like' | 'dislike' | 'not_interested' | 'wrong_genre';
  query?: string;
}

// Aggregated metrics
export interface SearchMetrics {
  period: 'hour' | 'day' | 'week';
  startTime: Date;
  endTime: Date;
  totalSearches: number;
  uniqueUsers: number;
  avgLatencyMs: number;
  p95LatencyMs: number;
  cacheHitRate: number;
  avgResultCount: number;
  clickThroughRate: number;
  voiceSearchRate: number;
  streamingFilterRate: number;
  topQueries: Array<{ query: string; count: number }>;
  topIntents: Array<{ intent: string; count: number }>;
  resultQualityScore: number;
}

// In-memory analytics store (would use a proper analytics DB in production)
class AnalyticsStore {
  private events: BaseAnalyticsEvent[] = [];
  private readonly maxEvents = 10000;
  private readonly retentionMs = 24 * 60 * 60 * 1000; // 24 hours

  add(event: BaseAnalyticsEvent): void {
    this.events.push(event);
    this.cleanup();
  }

  query<T extends BaseAnalyticsEvent>(
    filter: Partial<{ eventType: AnalyticsEventType; sessionId: string; startTime: Date; endTime: Date }>
  ): T[] {
    return this.events.filter(event => {
      if (filter.eventType && event.eventType !== filter.eventType) return false;
      if (filter.sessionId && event.sessionId !== filter.sessionId) return false;
      if (filter.startTime && event.timestamp < filter.startTime) return false;
      if (filter.endTime && event.timestamp > filter.endTime) return false;
      return true;
    }) as T[];
  }

  private cleanup(): void {
    const cutoff = Date.now() - this.retentionMs;

    // Remove old events
    this.events = this.events.filter(e => e.timestamp.getTime() > cutoff);

    // Trim to max size
    if (this.events.length > this.maxEvents) {
      this.events = this.events.slice(-this.maxEvents);
    }
  }

  clear(): void {
    this.events = [];
  }

  size(): number {
    return this.events.length;
  }
}

// Global analytics store instance
const analyticsStore = new AnalyticsStore();

/**
 * Generate unique event ID
 */
function generateEventId(): string {
  return `evt_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

/**
 * Track a search initiation
 */
export function trackSearchInitiated(
  sessionId: string,
  query: string,
  options?: {
    userId?: string;
    voiceSearch?: boolean;
    streamingService?: string;
    intents?: string[];
  }
): string {
  const eventId = generateEventId();

  const event: SearchEvent = {
    eventId,
    eventType: 'search_initiated',
    timestamp: new Date(),
    sessionId,
    userId: options?.userId,
    query,
    queryLength: query.length,
    voiceSearch: options?.voiceSearch ?? false,
    streamingServiceMentioned: options?.streamingService,
    intentDetected: options?.intents,
  };

  analyticsStore.add(event);
  return eventId;
}

/**
 * Track search completion
 */
export function trackSearchCompleted(
  sessionId: string,
  query: string,
  results: {
    count: number;
    latencyMs: number;
    cacheHit: boolean;
    topIds: number[];
  },
  options?: {
    userId?: string;
    voiceSearch?: boolean;
    intents?: string[];
  }
): void {
  const event: SearchCompletedEvent = {
    eventId: generateEventId(),
    eventType: 'search_completed',
    timestamp: new Date(),
    sessionId,
    userId: options?.userId,
    query,
    queryLength: query.length,
    voiceSearch: options?.voiceSearch ?? false,
    resultCount: results.count,
    latencyMs: results.latencyMs,
    cacheHit: results.cacheHit,
    topResultIds: results.topIds,
    intentDetected: options?.intents,
  };

  analyticsStore.add(event);
}

/**
 * Track result interaction
 */
export function trackResultInteraction(
  sessionId: string,
  interactionType: 'clicked' | 'viewed' | 'selected',
  result: {
    contentId: number;
    contentType: 'movie' | 'tv';
    position: number;
    relevanceScore: number;
    query: string;
  },
  userId?: string
): void {
  const eventType: ResultInteractionEvent['eventType'] =
    interactionType === 'clicked' ? 'result_clicked' :
    interactionType === 'viewed' ? 'result_viewed' :
    'result_selected';

  const event: ResultInteractionEvent = {
    eventId: generateEventId(),
    eventType,
    timestamp: new Date(),
    sessionId,
    userId,
    contentId: result.contentId,
    contentType: result.contentType,
    position: result.position,
    relevanceScore: result.relevanceScore,
    query: result.query,
  };

  analyticsStore.add(event);
}

/**
 * Track recommendation feedback
 */
export function trackFeedback(
  sessionId: string,
  contentId: number,
  feedbackType: FeedbackEvent['feedbackType'],
  options?: {
    userId?: string;
    query?: string;
  }
): void {
  const event: FeedbackEvent = {
    eventId: generateEventId(),
    eventType: 'recommendation_feedback',
    timestamp: new Date(),
    sessionId,
    userId: options?.userId,
    contentId,
    feedbackType,
    query: options?.query,
  };

  analyticsStore.add(event);
}

/**
 * Track generic event
 */
export function trackEvent(
  sessionId: string,
  eventType: AnalyticsEventType,
  metadata?: Record<string, unknown>,
  userId?: string
): void {
  const event: BaseAnalyticsEvent = {
    eventId: generateEventId(),
    eventType,
    timestamp: new Date(),
    sessionId,
    userId,
    metadata,
  };

  analyticsStore.add(event);
}

/**
 * Calculate search metrics for a time period
 */
export function getSearchMetrics(
  startTime: Date,
  endTime: Date,
  period: 'hour' | 'day' | 'week' = 'day'
): SearchMetrics {
  const searchCompleted = analyticsStore.query<SearchCompletedEvent>({
    eventType: 'search_completed',
    startTime,
    endTime,
  });

  const resultClicks = analyticsStore.query<ResultInteractionEvent>({
    eventType: 'result_clicked',
    startTime,
    endTime,
  });

  if (searchCompleted.length === 0) {
    return {
      period,
      startTime,
      endTime,
      totalSearches: 0,
      uniqueUsers: 0,
      avgLatencyMs: 0,
      p95LatencyMs: 0,
      cacheHitRate: 0,
      avgResultCount: 0,
      clickThroughRate: 0,
      voiceSearchRate: 0,
      streamingFilterRate: 0,
      topQueries: [],
      topIntents: [],
      resultQualityScore: 0,
    };
  }

  // Calculate metrics
  const totalSearches = searchCompleted.length;
  const uniqueUsers = new Set(searchCompleted.map(e => e.userId || e.sessionId)).size;

  // Latency
  const latencies = searchCompleted.map(e => e.latencyMs).sort((a, b) => a - b);
  const avgLatencyMs = latencies.reduce((a, b) => a + b, 0) / latencies.length;
  const p95Index = Math.floor(latencies.length * 0.95);
  const p95LatencyMs = latencies[p95Index] || latencies[latencies.length - 1];

  // Cache hit rate
  const cacheHits = searchCompleted.filter(e => e.cacheHit).length;
  const cacheHitRate = cacheHits / totalSearches;

  // Result count
  const avgResultCount = searchCompleted.reduce((a, e) => a + e.resultCount, 0) / totalSearches;

  // Click-through rate
  const searchesWithClicks = new Set(resultClicks.map(e => e.query)).size;
  const clickThroughRate = searchesWithClicks / totalSearches;

  // Voice search rate
  const voiceSearches = searchCompleted.filter(e => e.voiceSearch).length;
  const voiceSearchRate = voiceSearches / totalSearches;

  // Streaming filter rate
  const streamingFilters = searchCompleted.filter(e => e.streamingServiceMentioned).length;
  const streamingFilterRate = streamingFilters / totalSearches;

  // Top queries
  const queryCounts = new Map<string, number>();
  for (const event of searchCompleted) {
    const normalized = event.query.toLowerCase().trim();
    queryCounts.set(normalized, (queryCounts.get(normalized) || 0) + 1);
  }
  const topQueries = Array.from(queryCounts.entries())
    .map(([query, count]) => ({ query, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 10);

  // Top intents
  const intentCounts = new Map<string, number>();
  for (const event of searchCompleted) {
    for (const intent of event.intentDetected || []) {
      intentCounts.set(intent, (intentCounts.get(intent) || 0) + 1);
    }
  }
  const topIntents = Array.from(intentCounts.entries())
    .map(([intent, count]) => ({ intent, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 10);

  // Result quality score (based on CTR and click positions)
  const avgClickPosition = resultClicks.length > 0
    ? resultClicks.reduce((a, e) => a + e.position, 0) / resultClicks.length
    : 10;
  // Lower position = better, normalize to 0-1 score
  const positionScore = Math.max(0, 1 - (avgClickPosition - 1) / 9);
  const resultQualityScore = (clickThroughRate * 0.5 + positionScore * 0.5);

  return {
    period,
    startTime,
    endTime,
    totalSearches,
    uniqueUsers,
    avgLatencyMs: Math.round(avgLatencyMs),
    p95LatencyMs: Math.round(p95LatencyMs),
    cacheHitRate: Math.round(cacheHitRate * 100) / 100,
    avgResultCount: Math.round(avgResultCount * 10) / 10,
    clickThroughRate: Math.round(clickThroughRate * 100) / 100,
    voiceSearchRate: Math.round(voiceSearchRate * 100) / 100,
    streamingFilterRate: Math.round(streamingFilterRate * 100) / 100,
    topQueries,
    topIntents,
    resultQualityScore: Math.round(resultQualityScore * 100) / 100,
  };
}

/**
 * Get session analytics
 */
export function getSessionAnalytics(sessionId: string): {
  searches: number;
  clicks: number;
  avgLatency: number;
  topQueries: string[];
} {
  const searches = analyticsStore.query<SearchCompletedEvent>({
    eventType: 'search_completed',
    sessionId,
  });

  const clicks = analyticsStore.query<ResultInteractionEvent>({
    eventType: 'result_clicked',
    sessionId,
  });

  return {
    searches: searches.length,
    clicks: clicks.length,
    avgLatency: searches.length > 0
      ? Math.round(searches.reduce((a, e) => a + e.latencyMs, 0) / searches.length)
      : 0,
    topQueries: [...new Set(searches.map(e => e.query))].slice(0, 5),
  };
}

/**
 * Export analytics data (for external processing)
 */
export function exportAnalytics(
  startTime: Date,
  endTime: Date
): BaseAnalyticsEvent[] {
  return analyticsStore.query({ startTime, endTime });
}

/**
 * Get real-time metrics (last hour)
 */
export function getRealTimeMetrics(): SearchMetrics {
  const endTime = new Date();
  const startTime = new Date(endTime.getTime() - 60 * 60 * 1000);
  return getSearchMetrics(startTime, endTime, 'hour');
}

/**
 * Clear analytics (for testing)
 */
export function clearAnalytics(): void {
  analyticsStore.clear();
}
