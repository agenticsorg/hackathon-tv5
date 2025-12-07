import { check } from 'k6';
import { Rate } from 'k6/metrics';
import http from 'k6/http';

// Custom metrics
export const errorRate = new Rate('errors');

// Base configuration
export const config = {
  baseUrl: __ENV.BASE_URL || 'http://localhost:8080',
  authUrl: __ENV.AUTH_URL || 'http://localhost:8081',
  discoveryUrl: __ENV.DISCOVERY_URL || 'http://localhost:8082',
  sonaUrl: __ENV.SONA_URL || 'http://localhost:8083',
  syncUrl: __ENV.SYNC_URL || 'http://localhost:8084',
  playbackUrl: __ENV.PLAYBACK_URL || 'http://localhost:8085',
  ingestionUrl: __ENV.INGESTION_URL || 'http://localhost:8086',
};

// Authentication helper
export function authenticate(username = 'test@example.com', password = 'testpass123') {
  const loginPayload = JSON.stringify({
    email: username,
    password: password,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(`${config.authUrl}/api/auth/login`, loginPayload, params);

  check(res, {
    'login successful': (r) => r.status === 200,
    'token received': (r) => r.json('access_token') !== undefined,
  });

  return res.json('access_token');
}

// Get authenticated headers
export function getAuthHeaders(token) {
  return {
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
  };
}

// Common thresholds based on SPARC performance targets
export const thresholds = {
  // Search API: <500ms p95
  search: {
    'http_req_duration{endpoint:search}': ['p(95)<500'],
    'http_req_failed{endpoint:search}': ['rate<0.01'],
  },

  // SONA recommendations: <5ms p95
  sona: {
    'http_req_duration{endpoint:sona}': ['p(95)<5'],
    'http_req_failed{endpoint:sona}': ['rate<0.01'],
  },

  // Sync operations: <100ms
  sync: {
    'http_req_duration{endpoint:sync}': ['p(95)<100'],
    'http_req_failed{endpoint:sync}': ['rate<0.01'],
  },

  // Auth operations: <50ms p95
  auth: {
    'http_req_duration{endpoint:auth}': ['p(95)<50'],
    'http_req_failed{endpoint:auth}': ['rate<0.01'],
  },

  // Playback: <200ms
  playback: {
    'http_req_duration{endpoint:playback}': ['p(95)<200'],
    'http_req_failed{endpoint:playback}': ['rate<0.01'],
  },

  // Ingestion: <1s
  ingestion: {
    'http_req_duration{endpoint:ingestion}': ['p(95)<1000'],
    'http_req_failed{endpoint:ingestion}': ['rate<0.01'],
  },
};

// Combined thresholds for all endpoints
export const allThresholds = {
  'http_req_duration': ['p(95)<500', 'p(99)<1000'],
  'http_req_failed': ['rate<0.01'],
  'errors': ['rate<0.01'],
  'http_reqs': ['rate>1000'], // Minimum 1000 RPS
  ...thresholds.search,
  ...thresholds.sona,
  ...thresholds.sync,
  ...thresholds.auth,
  ...thresholds.playback,
  ...thresholds.ingestion,
};

// InfluxDB configuration
export const influxConfig = {
  output: 'influxdb=http://localhost:8086/k6',
  db: 'k6',
  tagsAsFields: ['endpoint', 'method', 'status'],
};

// Test data generators
export function generateUserId() {
  return `user-${Math.floor(Math.random() * 10000)}`;
}

export function generateMediaId() {
  return `media-${Math.floor(Math.random() * 50000)}`;
}

export function generateSearchQuery() {
  const queries = [
    'action movies',
    'comedy series',
    'thriller 2024',
    'family entertainment',
    'documentary nature',
    'sci-fi classics',
    'romantic comedy',
    'horror films',
  ];
  return queries[Math.floor(Math.random() * queries.length)];
}

// Common check functions
export function checkResponse(response, endpoint) {
  const result = check(response, {
    'status is 200': (r) => r.status === 200,
    'response time OK': (r) => r.timings.duration < 1000,
    'has valid JSON': (r) => {
      try {
        JSON.parse(r.body);
        return true;
      } catch (e) {
        return false;
      }
    },
  });

  errorRate.add(!result);
  return result;
}

// Scenario weights for realistic traffic distribution
export const scenarioWeights = {
  search: 0.40,      // 40% of traffic
  recommendations: 0.25, // 25% of traffic
  playback: 0.20,    // 20% of traffic
  auth: 0.10,        // 10% of traffic
  sync: 0.05,        // 5% of traffic
};

// VU stages helpers
export function getRampUpStages(targetVUs, duration) {
  return [
    { duration: '1m', target: Math.floor(targetVUs * 0.1) },
    { duration: '2m', target: Math.floor(targetVUs * 0.5) },
    { duration: '2m', target: targetVUs },
    { duration: duration, target: targetVUs },
  ];
}

export function getRampDownStages(targetVUs, sustainDuration) {
  return [
    ...getRampUpStages(targetVUs, sustainDuration),
    { duration: '2m', target: Math.floor(targetVUs * 0.5) },
    { duration: '1m', target: 0 },
  ];
}
