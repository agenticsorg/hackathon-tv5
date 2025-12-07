import { sleep } from 'k6';
import http from 'k6/http';
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

// Stress test configuration: 20K VUs, 3500 RPS peak, gradual ramp-up
export const options = {
  scenarios: {
    search_stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        // Gradual ramp to normal load
        { duration: '5m', target: Math.floor(10000 * scenarioWeights.search) },
        { duration: '5m', target: Math.floor(10000 * scenarioWeights.search) },
        // Increase to 1.5x normal
        { duration: '5m', target: Math.floor(15000 * scenarioWeights.search) },
        { duration: '5m', target: Math.floor(15000 * scenarioWeights.search) },
        // Increase to 2x normal (stress level)
        { duration: '5m', target: Math.floor(20000 * scenarioWeights.search) },
        { duration: '10m', target: Math.floor(20000 * scenarioWeights.search) },
        // Ramp down
        { duration: '5m', target: 0 },
      ],
      gracefulRampDown: '1m',
      exec: 'searchScenario',
    },
    recommendations_stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '5m', target: Math.floor(10000 * scenarioWeights.recommendations) },
        { duration: '5m', target: Math.floor(10000 * scenarioWeights.recommendations) },
        { duration: '5m', target: Math.floor(15000 * scenarioWeights.recommendations) },
        { duration: '5m', target: Math.floor(15000 * scenarioWeights.recommendations) },
        { duration: '5m', target: Math.floor(20000 * scenarioWeights.recommendations) },
        { duration: '10m', target: Math.floor(20000 * scenarioWeights.recommendations) },
        { duration: '5m', target: 0 },
      ],
      gracefulRampDown: '1m',
      exec: 'recommendationsScenario',
    },
    playback_stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '5m', target: Math.floor(10000 * scenarioWeights.playback) },
        { duration: '5m', target: Math.floor(10000 * scenarioWeights.playback) },
        { duration: '5m', target: Math.floor(15000 * scenarioWeights.playback) },
        { duration: '5m', target: Math.floor(15000 * scenarioWeights.playback) },
        { duration: '5m', target: Math.floor(20000 * scenarioWeights.playback) },
        { duration: '10m', target: Math.floor(20000 * scenarioWeights.playback) },
        { duration: '5m', target: 0 },
      ],
      gracefulRampDown: '1m',
      exec: 'playbackScenario',
    },
    auth_stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '5m', target: Math.floor(10000 * scenarioWeights.auth) },
        { duration: '5m', target: Math.floor(10000 * scenarioWeights.auth) },
        { duration: '5m', target: Math.floor(15000 * scenarioWeights.auth) },
        { duration: '5m', target: Math.floor(15000 * scenarioWeights.auth) },
        { duration: '5m', target: Math.floor(20000 * scenarioWeights.auth) },
        { duration: '10m', target: Math.floor(20000 * scenarioWeights.auth) },
        { duration: '5m', target: 0 },
      ],
      gracefulRampDown: '1m',
      exec: 'authScenario',
    },
    sync_stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '5m', target: Math.floor(10000 * scenarioWeights.sync) },
        { duration: '5m', target: Math.floor(10000 * scenarioWeights.sync) },
        { duration: '5m', target: Math.floor(15000 * scenarioWeights.sync) },
        { duration: '5m', target: Math.floor(15000 * scenarioWeights.sync) },
        { duration: '5m', target: Math.floor(20000 * scenarioWeights.sync) },
        { duration: '10m', target: Math.floor(20000 * scenarioWeights.sync) },
        { duration: '5m', target: 0 },
      ],
      gracefulRampDown: '1m',
      exec: 'syncScenario',
    },
  },
  thresholds: {
    // Relaxed thresholds for stress testing
    'http_req_duration': ['p(95)<1000', 'p(99)<2000'],
    'http_req_failed': ['rate<0.05'], // Allow 5% error rate under stress
    'errors': ['rate<0.05'],
    'http_reqs': ['rate>1000'],
  },
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)', 'p(99.9)'],
  noConnectionReuse: false,
};

// Setup function
export function setup() {
  const token = authenticate('stress-test@example.com', 'testpass123');
  return { token };
}

// Search scenario
export function searchScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'search', method: 'GET' };

  const query = generateSearchQuery();
  const res = http.get(
    `${config.discoveryUrl}/api/search?q=${encodeURIComponent(query)}&limit=20&offset=${Math.floor(Math.random() * 100)}`,
    params
  );

  checkResponse(res, 'search');
  sleep(Math.random() * 2); // Variable think time
}

// Recommendations scenario
export function recommendationsScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'sona', method: 'GET' };

  const userId = generateUserId();
  const res = http.get(
    `${config.sonaUrl}/api/recommendations/${userId}?limit=${5 + Math.floor(Math.random() * 15)}`,
    params
  );

  checkResponse(res, 'sona');
  sleep(Math.random() * 2);
}

// Playback scenario
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

  sleep(3 + Math.random() * 4);

  // Update progress multiple times
  for (let i = 0; i < 3; i++) {
    params.tags = { endpoint: 'playback', method: 'PUT' };
    res = http.put(
      `${config.playbackUrl}/api/playback/progress`,
      JSON.stringify({
        user_id: userId,
        media_id: mediaId,
        position: 300 + (i * 100)
      }),
      params
    );
    checkResponse(res, 'playback');
    sleep(1 + Math.random());
  }
}

// Auth scenario with profile operations
export function authScenario() {
  let params = {
    headers: { 'Content-Type': 'application/json' },
    tags: { endpoint: 'auth', method: 'POST' },
  };

  // Login
  const email = `stress-user-${Math.floor(Math.random() * 20000)}@example.com`;
  const loginPayload = JSON.stringify({
    email: email,
    password: 'testpass123',
  });

  let res = http.post(`${config.authUrl}/api/auth/login`, loginPayload, params);
  checkResponse(res, 'auth');

  const token = res.json('access_token');

  sleep(1);

  // Profile operations
  params = getAuthHeaders(token);
  params.tags = { endpoint: 'auth', method: 'GET' };
  res = http.get(`${config.authUrl}/api/auth/profile`, params);
  checkResponse(res, 'auth');

  sleep(Math.random() * 2);

  // Update profile
  params.tags = { endpoint: 'auth', method: 'PUT' };
  res = http.put(
    `${config.authUrl}/api/auth/profile`,
    JSON.stringify({ display_name: `StressUser${Math.floor(Math.random() * 1000)}` }),
    params
  );
  checkResponse(res, 'auth');

  sleep(1);
}

// Sync scenario with concurrent operations
export function syncScenario(data) {
  const params = getAuthHeaders(data.token);

  const userId = generateUserId();
  const deviceId = `device-${Math.floor(Math.random() * 5000)}`;

  // Sync multiple data types
  const syncPayload = JSON.stringify({
    user_id: userId,
    device_id: deviceId,
    timestamp: new Date().toISOString(),
    data: {
      watchlist: Array.from({ length: 10 }, () => generateMediaId()),
      history: Array.from({ length: 20 }, () => ({
        media_id: generateMediaId(),
        timestamp: new Date(Date.now() - Math.random() * 86400000).toISOString(),
      })),
      preferences: {
        theme: Math.random() > 0.5 ? 'dark' : 'light',
        language: Math.random() > 0.5 ? 'en' : 'es',
        quality: Math.random() > 0.5 ? 'hd' : '4k',
      },
    },
  });

  params.tags = { endpoint: 'sync', method: 'POST' };
  const res = http.post(`${config.syncUrl}/api/sync`, syncPayload, params);
  checkResponse(res, 'sync');

  sleep(Math.random() * 3);
}

// Teardown function
export function teardown(data) {
  console.log('Stress test completed - check for degradation points');
}
