import { sleep } from 'k6';
import http from 'k6/http';
import { Counter, Trend } from 'k6/metrics';
import {
  config,
  allThresholds,
  authenticate,
  getAuthHeaders,
  generateUserId,
  generateMediaId,
  generateSearchQuery,
  checkResponse,
  scenarioWeights,
} from './config.js';

// Custom metrics for memory leak detection
const memoryTrend = new Trend('memory_usage');
const connectionCount = new Counter('active_connections');
const gcCount = new Counter('gc_events');

// Soak test configuration: 24-hour sustained load, memory leak detection
export const options = {
  scenarios: {
    search_soak: {
      executor: 'constant-vus',
      vus: Math.floor(10000 * scenarioWeights.search), // 4000 VUs
      duration: '24h',
      gracefulStop: '2m',
      exec: 'searchScenario',
    },
    recommendations_soak: {
      executor: 'constant-vus',
      vus: Math.floor(10000 * scenarioWeights.recommendations), // 2500 VUs
      duration: '24h',
      gracefulStop: '2m',
      exec: 'recommendationsScenario',
    },
    playback_soak: {
      executor: 'constant-vus',
      vus: Math.floor(10000 * scenarioWeights.playback), // 2000 VUs
      duration: '24h',
      gracefulStop: '2m',
      exec: 'playbackScenario',
    },
    auth_soak: {
      executor: 'constant-vus',
      vus: Math.floor(10000 * scenarioWeights.auth), // 1000 VUs
      duration: '24h',
      gracefulStop: '2m',
      exec: 'authScenario',
    },
    sync_soak: {
      executor: 'constant-vus',
      vus: Math.floor(10000 * scenarioWeights.sync), // 500 VUs
      duration: '24h',
      gracefulStop: '2m',
      exec: 'syncScenario',
    },
    // Health check scenario - monitors system metrics
    health_monitor: {
      executor: 'constant-vus',
      vus: 1,
      duration: '24h',
      gracefulStop: '30s',
      exec: 'healthCheckScenario',
    },
  },
  thresholds: {
    // Strict thresholds - performance should not degrade over time
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],
    'http_req_failed': ['rate<0.01'],
    'errors': ['rate<0.01'],
    'http_reqs': ['rate>1000'],
    // Memory leak indicators
    'memory_usage': ['p(95)<1073741824'], // 1GB p95
    'active_connections': ['count<50000'],
  },
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
  noConnectionReuse: false,
  userAgent: 'k6-soak-test/1.0',
};

// Setup function
export function setup() {
  const token = authenticate('soak-test@example.com', 'testpass123');
  console.log('Starting 24-hour soak test...');
  console.log('Monitor for:');
  console.log('- Memory leaks (gradually increasing response times)');
  console.log('- Connection leaks (increasing active connections)');
  console.log('- Resource exhaustion (increasing error rates)');
  console.log('- Performance degradation over time');
  return {
    token,
    startTime: Date.now(),
  };
}

// Search scenario with variable patterns
export function searchScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'search', method: 'GET' };

  const query = generateSearchQuery();
  const limit = [10, 20, 50][Math.floor(Math.random() * 3)];
  const offset = Math.floor(Math.random() * 200);

  const res = http.get(
    `${config.discoveryUrl}/api/search?q=${encodeURIComponent(query)}&limit=${limit}&offset=${offset}`,
    params
  );

  checkResponse(res, 'search');
  connectionCount.add(1);

  // Variable think time to simulate realistic usage
  sleep(1 + Math.random() * 2);
}

// Recommendations scenario with caching patterns
export function recommendationsScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'sona', method: 'GET' };

  // Use same user IDs periodically to test caching
  const userId = Math.random() > 0.3
    ? generateUserId()
    : `cached-user-${Math.floor(Math.random() * 100)}`;

  const res = http.get(
    `${config.sonaUrl}/api/recommendations/${userId}?limit=10`,
    params
  );

  checkResponse(res, 'sona');
  connectionCount.add(1);
  sleep(2 + Math.random() * 3);
}

// Playback scenario with long sessions
export function playbackScenario(data) {
  const params = getAuthHeaders(data.token);

  const userId = generateUserId();
  const mediaId = generateMediaId();

  // Start playback
  params.tags = { endpoint: 'playback', method: 'POST' };
  let res = http.post(
    `${config.playbackUrl}/api/playback/start`,
    JSON.stringify({ user_id: userId, media_id: mediaId }),
    params
  );
  checkResponse(res, 'playback');
  connectionCount.add(1);

  // Simulate watching - periodic progress updates
  const watchDuration = 5 + Math.floor(Math.random() * 10);
  for (let i = 0; i < watchDuration; i++) {
    sleep(30); // 30 seconds between updates

    params.tags = { endpoint: 'playback', method: 'PUT' };
    res = http.put(
      `${config.playbackUrl}/api/playback/progress`,
      JSON.stringify({
        user_id: userId,
        media_id: mediaId,
        position: (i + 1) * 30
      }),
      params
    );
    checkResponse(res, 'playback');
  }

  // End playback
  params.tags = { endpoint: 'playback', method: 'POST' };
  res = http.post(
    `${config.playbackUrl}/api/playback/end`,
    JSON.stringify({ user_id: userId, media_id: mediaId }),
    params
  );
  checkResponse(res, 'playback');

  sleep(5);
}

// Auth scenario with session lifecycle
export function authScenario() {
  let params = {
    headers: { 'Content-Type': 'application/json' },
    tags: { endpoint: 'auth', method: 'POST' },
  };

  // Login
  const email = `soak-user-${Math.floor(Math.random() * 10000)}@example.com`;
  const loginPayload = JSON.stringify({
    email: email,
    password: 'testpass123',
  });

  let res = http.post(`${config.authUrl}/api/auth/login`, loginPayload, params);
  checkResponse(res, 'auth');
  connectionCount.add(1);

  const token = res.json('access_token');

  // Simulate active session
  const sessionDuration = 5 + Math.floor(Math.random() * 10);
  for (let i = 0; i < sessionDuration; i++) {
    sleep(60); // Check every minute

    // Verify token
    params = getAuthHeaders(token);
    params.tags = { endpoint: 'auth', method: 'GET' };
    res = http.get(`${config.authUrl}/api/auth/verify`, params);
    checkResponse(res, 'auth');
  }

  // Logout
  params.tags = { endpoint: 'auth', method: 'POST' };
  res = http.post(`${config.authUrl}/api/auth/logout`, null, params);
  checkResponse(res, 'auth');

  sleep(5);
}

// Sync scenario with incremental data
export function syncScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'sync', method: 'POST' };

  const userId = generateUserId();
  const deviceId = `device-${Math.floor(Math.random() * 1000)}`;

  // Sync with varying payload sizes
  const watchlistSize = 5 + Math.floor(Math.random() * 20);
  const historySize = 10 + Math.floor(Math.random() * 40);

  const syncPayload = JSON.stringify({
    user_id: userId,
    device_id: deviceId,
    timestamp: new Date().toISOString(),
    data: {
      watchlist: Array.from({ length: watchlistSize }, () => generateMediaId()),
      history: Array.from({ length: historySize }, () => ({
        media_id: generateMediaId(),
        position: Math.floor(Math.random() * 3600),
        timestamp: new Date(Date.now() - Math.random() * 86400000 * 7).toISOString(),
      })),
      preferences: {
        theme: Math.random() > 0.5 ? 'dark' : 'light',
        language: ['en', 'es', 'fr', 'de'][Math.floor(Math.random() * 4)],
        quality: ['sd', 'hd', '4k'][Math.floor(Math.random() * 3)],
        autoplay: Math.random() > 0.5,
        subtitles: Math.random() > 0.3,
      },
    },
  });

  const res = http.post(`${config.syncUrl}/api/sync`, syncPayload, params);
  checkResponse(res, 'sync');
  connectionCount.add(1);

  sleep(5 + Math.random() * 10);
}

// Health check scenario - monitors for degradation
export function healthCheckScenario(data) {
  const params = getAuthHeaders(data.token);

  // Check each service health endpoint
  const services = [
    { name: 'auth', url: config.authUrl },
    { name: 'discovery', url: config.discoveryUrl },
    { name: 'sona', url: config.sonaUrl },
    { name: 'sync', url: config.syncUrl },
    { name: 'playback', url: config.playbackUrl },
  ];

  for (const service of services) {
    params.tags = { endpoint: 'health', service: service.name };
    const res = http.get(`${service.url}/health`, params);

    const healthy = res.status === 200;
    if (!healthy) {
      console.error(`${service.name} health check failed: ${res.status}`);
    }

    // Track memory usage if available in response
    try {
      const health = res.json();
      if (health.memory_usage) {
        memoryTrend.add(health.memory_usage);
      }
      if (health.gc_count) {
        gcCount.add(health.gc_count);
      }
    } catch (e) {
      // Health endpoint might not return JSON
    }
  }

  sleep(60); // Check every minute
}

// Teardown function
export function teardown(data) {
  const endTime = Date.now();
  const duration = (endTime - data.startTime) / 1000 / 60 / 60; // hours

  console.log(`Soak test completed after ${duration.toFixed(2)} hours`);
  console.log('Review metrics for:');
  console.log('- Memory usage trends (should be stable)');
  console.log('- Response time trends (should not increase)');
  console.log('- Error rate trends (should remain low)');
  console.log('- Connection count (should not leak)');
  console.log('- GC frequency (should be consistent)');
}
