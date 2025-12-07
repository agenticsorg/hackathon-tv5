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

// Baseline test configuration: 10K VUs, 1000 RPS, 30 minutes
export const options = {
  scenarios: {
    search_baseline: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: Math.floor(10000 * scenarioWeights.search) }, // 4000 VUs
        { duration: '30m', target: Math.floor(10000 * scenarioWeights.search) },
        { duration: '2m', target: 0 },
      ],
      gracefulRampDown: '30s',
      exec: 'searchScenario',
    },
    recommendations_baseline: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: Math.floor(10000 * scenarioWeights.recommendations) }, // 2500 VUs
        { duration: '30m', target: Math.floor(10000 * scenarioWeights.recommendations) },
        { duration: '2m', target: 0 },
      ],
      gracefulRampDown: '30s',
      exec: 'recommendationsScenario',
    },
    playback_baseline: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: Math.floor(10000 * scenarioWeights.playback) }, // 2000 VUs
        { duration: '30m', target: Math.floor(10000 * scenarioWeights.playback) },
        { duration: '2m', target: 0 },
      ],
      gracefulRampDown: '30s',
      exec: 'playbackScenario',
    },
    auth_baseline: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: Math.floor(10000 * scenarioWeights.auth) }, // 1000 VUs
        { duration: '30m', target: Math.floor(10000 * scenarioWeights.auth) },
        { duration: '2m', target: 0 },
      ],
      gracefulRampDown: '30s',
      exec: 'authScenario',
    },
    sync_baseline: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: Math.floor(10000 * scenarioWeights.sync) }, // 500 VUs
        { duration: '30m', target: Math.floor(10000 * scenarioWeights.sync) },
        { duration: '2m', target: 0 },
      ],
      gracefulRampDown: '30s',
      exec: 'syncScenario',
    },
  },
  thresholds: allThresholds,
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
  noConnectionReuse: false,
};

// Setup function - runs once per VU
export function setup() {
  const token = authenticate('baseline-test@example.com', 'testpass123');
  return { token };
}

// Search scenario
export function searchScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'search', method: 'GET' };

  const query = generateSearchQuery();
  const res = http.get(
    `${config.discoveryUrl}/api/search?q=${encodeURIComponent(query)}&limit=20`,
    params
  );

  checkResponse(res, 'search');
  sleep(1);
}

// Recommendations scenario
export function recommendationsScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'sona', method: 'GET' };

  const userId = generateUserId();
  const res = http.get(
    `${config.sonaUrl}/api/recommendations/${userId}?limit=10`,
    params
  );

  checkResponse(res, 'sona');
  sleep(1);
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

  sleep(5);

  // Update progress
  params.tags = { endpoint: 'playback', method: 'PUT' };
  res = http.put(
    `${config.playbackUrl}/api/playback/progress`,
    JSON.stringify({ user_id: userId, media_id: mediaId, position: 300 }),
    params
  );
  checkResponse(res, 'playback');

  sleep(1);
}

// Auth scenario
export function authScenario() {
  let params = {
    headers: { 'Content-Type': 'application/json' },
    tags: { endpoint: 'auth', method: 'POST' },
  };

  // Login
  const loginPayload = JSON.stringify({
    email: `user-${Math.floor(Math.random() * 10000)}@example.com`,
    password: 'testpass123',
  });

  let res = http.post(`${config.authUrl}/api/auth/login`, loginPayload, params);
  checkResponse(res, 'auth');

  const token = res.json('access_token');

  sleep(2);

  // Verify token
  params = getAuthHeaders(token);
  params.tags = { endpoint: 'auth', method: 'GET' };
  res = http.get(`${config.authUrl}/api/auth/verify`, params);
  checkResponse(res, 'auth');

  sleep(1);
}

// Sync scenario
export function syncScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'sync', method: 'POST' };

  const userId = generateUserId();
  const syncPayload = JSON.stringify({
    user_id: userId,
    device_id: `device-${Math.floor(Math.random() * 1000)}`,
    timestamp: new Date().toISOString(),
    data: {
      watchlist: [generateMediaId(), generateMediaId()],
      preferences: { theme: 'dark', language: 'en' },
    },
  });

  const res = http.post(
    `${config.syncUrl}/api/sync`,
    syncPayload,
    params
  );

  checkResponse(res, 'sync');
  sleep(1);
}

// Teardown function
export function teardown(data) {
  console.log('Baseline test completed');
}
