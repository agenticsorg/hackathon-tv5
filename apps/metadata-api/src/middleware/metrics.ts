/**
 * Prometheus-compatible Metrics Middleware
 *
 * Provides custom metrics for Cloud Monitoring integration:
 * - Request duration histogram (P50, P95, P99)
 * - Request counter by endpoint and status
 * - Error counter by type
 * - In-flight request gauge
 *
 * Metrics are exposed at /metrics endpoint in Prometheus format
 * and automatically scraped by GCP Cloud Monitoring
 */

import { Request, Response, NextFunction } from 'express';
import { logger } from '../utils/logger';

/**
 * Custom metrics collector
 * Simple in-memory implementation compatible with Cloud Monitoring
 */
class MetricsCollector {
  private requestDurations: Map<string, number[]> = new Map();
  private requestCounts: Map<string, number> = new Map();
  private errorCounts: Map<string, number> = new Map();
  private inFlightRequests = 0;
  private startTime: number = Date.now();

  /**
   * Record request duration
   */
  recordRequestDuration(endpoint: string, method: string, statusCode: number, duration: number): void {
    const key = `${method}_${endpoint}_${statusCode}`;

    // Store duration for percentile calculation
    if (!this.requestDurations.has(key)) {
      this.requestDurations.set(key, []);
    }
    const durations = this.requestDurations.get(key)!;
    durations.push(duration);

    // Keep only last 1000 samples per key to prevent memory growth
    if (durations.length > 1000) {
      durations.shift();
    }

    // Increment request counter
    const countKey = `${method}_${endpoint}_${statusCode}`;
    this.requestCounts.set(countKey, (this.requestCounts.get(countKey) || 0) + 1);
  }

  /**
   * Record error
   */
  recordError(errorType: string): void {
    this.errorCounts.set(errorType, (this.errorCounts.get(errorType) || 0) + 1);
  }

  /**
   * Increment in-flight requests
   */
  incrementInFlight(): void {
    this.inFlightRequests++;
  }

  /**
   * Decrement in-flight requests
   */
  decrementInFlight(): void {
    this.inFlightRequests--;
  }

  /**
   * Calculate percentile from array of values
   */
  private calculatePercentile(values: number[], percentile: number): number {
    if (values.length === 0) return 0;

    const sorted = [...values].sort((a, b) => a - b);
    const index = Math.ceil((percentile / 100) * sorted.length) - 1;
    return sorted[Math.max(0, index)];
  }

  /**
   * Get metrics in Prometheus format
   */
  getPrometheusMetrics(): string {
    const lines: string[] = [];
    const timestamp = Date.now();

    // Request duration histogram
    lines.push('# HELP metadata_api_request_duration_milliseconds Request duration in milliseconds');
    lines.push('# TYPE metadata_api_request_duration_milliseconds histogram');

    for (const [key, durations] of this.requestDurations.entries()) {
      const [method, endpoint, statusCode] = key.split('_');
      const p50 = this.calculatePercentile(durations, 50);
      const p95 = this.calculatePercentile(durations, 95);
      const p99 = this.calculatePercentile(durations, 99);
      const count = durations.length;
      const sum = durations.reduce((a, b) => a + b, 0);

      // Histogram buckets
      const buckets = [10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000];
      for (const bucket of buckets) {
        const bucketCount = durations.filter(d => d <= bucket).length;
        lines.push(
          `metadata_api_request_duration_milliseconds_bucket{method="${method}",endpoint="${endpoint}",status_code="${statusCode}",le="${bucket}"} ${bucketCount} ${timestamp}`
        );
      }

      // Infinity bucket
      lines.push(
        `metadata_api_request_duration_milliseconds_bucket{method="${method}",endpoint="${endpoint}",status_code="${statusCode}",le="+Inf"} ${count} ${timestamp}`
      );

      // Sum and count
      lines.push(
        `metadata_api_request_duration_milliseconds_sum{method="${method}",endpoint="${endpoint}",status_code="${statusCode}"} ${sum} ${timestamp}`
      );
      lines.push(
        `metadata_api_request_duration_milliseconds_count{method="${method}",endpoint="${endpoint}",status_code="${statusCode}"} ${count} ${timestamp}`
      );

      // Percentiles as separate metrics for easier querying
      lines.push(
        `metadata_api_request_duration_p50{method="${method}",endpoint="${endpoint}",status_code="${statusCode}"} ${p50} ${timestamp}`
      );
      lines.push(
        `metadata_api_request_duration_p95{method="${method}",endpoint="${endpoint}",status_code="${statusCode}"} ${p95} ${timestamp}`
      );
      lines.push(
        `metadata_api_request_duration_p99{method="${method}",endpoint="${endpoint}",status_code="${statusCode}"} ${p99} ${timestamp}`
      );
    }

    // Request counter
    lines.push('# HELP metadata_api_requests_total Total number of requests');
    lines.push('# TYPE metadata_api_requests_total counter');

    for (const [key, count] of this.requestCounts.entries()) {
      const [method, endpoint, statusCode] = key.split('_');
      lines.push(
        `metadata_api_requests_total{method="${method}",endpoint="${endpoint}",status_code="${statusCode}"} ${count} ${timestamp}`
      );
    }

    // Error counter
    lines.push('# HELP metadata_api_errors_total Total number of errors');
    lines.push('# TYPE metadata_api_errors_total counter');

    for (const [errorType, count] of this.errorCounts.entries()) {
      lines.push(
        `metadata_api_errors_total{error_type="${errorType}"} ${count} ${timestamp}`
      );
    }

    // In-flight requests gauge
    lines.push('# HELP metadata_api_requests_in_flight Current number of requests being processed');
    lines.push('# TYPE metadata_api_requests_in_flight gauge');
    lines.push(`metadata_api_requests_in_flight ${this.inFlightRequests} ${timestamp}`);

    // Uptime
    lines.push('# HELP metadata_api_uptime_seconds Service uptime in seconds');
    lines.push('# TYPE metadata_api_uptime_seconds counter');
    const uptime = Math.floor((timestamp - this.startTime) / 1000);
    lines.push(`metadata_api_uptime_seconds ${uptime} ${timestamp}`);

    return lines.join('\n') + '\n';
  }

  /**
   * Get metrics summary as JSON
   */
  getMetricsSummary(): object {
    const summary: any = {
      uptime_seconds: Math.floor((Date.now() - this.startTime) / 1000),
      in_flight_requests: this.inFlightRequests,
      endpoints: {},
      errors: Object.fromEntries(this.errorCounts)
    };

    // Aggregate by endpoint
    for (const [key, durations] of this.requestDurations.entries()) {
      const [method, endpoint, statusCode] = key.split('_');
      const endpointKey = `${method} ${endpoint}`;

      if (!summary.endpoints[endpointKey]) {
        summary.endpoints[endpointKey] = {
          total_requests: 0,
          status_codes: {}
        };
      }

      summary.endpoints[endpointKey].status_codes[statusCode] = {
        count: durations.length,
        p50_ms: this.calculatePercentile(durations, 50),
        p95_ms: this.calculatePercentile(durations, 95),
        p99_ms: this.calculatePercentile(durations, 99),
        avg_ms: durations.reduce((a, b) => a + b, 0) / durations.length
      };

      summary.endpoints[endpointKey].total_requests += durations.length;
    }

    return summary;
  }

  /**
   * Reset all metrics (useful for testing)
   */
  reset(): void {
    this.requestDurations.clear();
    this.requestCounts.clear();
    this.errorCounts.clear();
    this.inFlightRequests = 0;
    this.startTime = Date.now();
  }
}

// Singleton instance
export const metricsCollector = new MetricsCollector();

/**
 * Metrics middleware
 * Automatically tracks request duration and counts
 */
export const metricsMiddleware = (req: Request, res: Response, next: NextFunction): void => {
  const startTime = Date.now();
  metricsCollector.incrementInFlight();

  // Normalize endpoint path (remove IDs and query params)
  const normalizedPath = req.path
    .replace(/\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/gi, '/:id')
    .replace(/\/\d+/g, '/:id')
    .replace(/\?.*$/, '');

  // Track response
  const originalSend = res.send;
  res.send = function (data): Response {
    const duration = Date.now() - startTime;
    const statusCode = res.statusCode;
    const method = req.method;

    // Record metrics
    metricsCollector.recordRequestDuration(normalizedPath, method, statusCode, duration);
    metricsCollector.decrementInFlight();

    // Record errors
    if (statusCode >= 500) {
      metricsCollector.recordError('5xx_server_error');
    } else if (statusCode >= 400) {
      metricsCollector.recordError('4xx_client_error');
    }

    // Log slow requests (P95 threshold)
    if (duration > 500) {
      logger.warn('Slow request detected', {
        method,
        path: normalizedPath,
        statusCode,
        duration_ms: duration,
        user_agent: req.get('user-agent')
      });
    }

    return originalSend.call(this, data);
  };

  next();
};

/**
 * Metrics endpoint handler
 * Exposes metrics in Prometheus format
 */
export const metricsHandler = (req: Request, res: Response): void => {
  const format = req.query.format || 'prometheus';

  if (format === 'json') {
    res.setHeader('Content-Type', 'application/json');
    res.send(JSON.stringify(metricsCollector.getMetricsSummary(), null, 2));
  } else {
    res.setHeader('Content-Type', 'text/plain; version=0.0.4');
    res.send(metricsCollector.getPrometheusMetrics());
  }
};

/**
 * Health check with metrics
 */
export const healthCheckWithMetrics = (req: Request, res: Response): void => {
  const summary = metricsCollector.getMetricsSummary() as any;
  const totalRequests = Object.values(summary.endpoints).reduce(
    (sum: number, endpoint: any) => sum + endpoint.total_requests,
    0
  );

  res.status(200).json({
    status: 'healthy',
    service: 'nexus-ummid-metadata-api',
    version: '1.0.0',
    timestamp: new Date().toISOString(),
    uptime_seconds: summary.uptime_seconds,
    metrics: {
      total_requests: totalRequests,
      in_flight_requests: summary.in_flight_requests,
      error_count: Object.values(summary.errors).reduce((a: number, b: any) => a + b, 0)
    }
  });
};

export default metricsMiddleware;
