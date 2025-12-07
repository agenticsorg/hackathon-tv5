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

// Spike test configuration: 100K users sudden load (10x normal), measure recovery
export const options = {
  scenarios: {
    search_spike: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        // Normal load
        { duration: '2m', target: Math.floor(10000 * scenarioWeights.search) },
        { duration: '3m', target: Math.floor(10000 * scenarioWeights.search) },
        // Sudden spike to 10x (100K total users)
        { duration: '30s', target: Math.floor(100000 * scenarioWeights.search) },
        { duration: '2m', target: Math.floor(100000 * scenarioWeights.search) },
        // Recovery to normal
        { duration: '1m', target: Math.floor(10000 * scenarioWeights.search) },
        { duration: '3m', target: Math.floor(10000 * scenarioWeights.search) },
        // Ramp down
        { duration: '1m', target: 0 },
      ],
      gracefulRampDown: '30s',
      exec: 'searchScenario',
    },
    recommendations_spike: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: Math.floor(10000 * scenarioWeights.recommendations) },
        { duration: '3m', target: Math.floor(10000 * scenarioWeights.recommendations) },
        { duration: '30s', target: Math.floor(100000 * scenarioWeights.recommendations) },
        { duration: '2m', target: Math.floor(100000 * scenarioWeights.recommendations) },
        { duration: '1m', target: Math.floor(10000 * scenarioWeights.recommendations) },
        { duration: '3m', target: Math.floor(10000 * scenarioWeights.recommendations) },
        { duration: '1m', target: 0 },
      ],
      gracefulRampDown: '30s',
      exec: 'recommendationsScenario',
    },
    playback_spike: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: Math.floor(10000 * scenarioWeights.playback) },
        { duration: '3m', target: Math.floor(10000 * scenarioWeights.playback) },
        { duration: '30s', target: Math.floor(100000 * scenarioWeights.playback) },
        { duration: '2m', target: Math.floor(100000 * scenarioWeights.playback) },
        { duration: '1m', target: Math.floor(10000 * scenarioWeights.playback) },
        { duration: '3m', target: Math.floor(10000 * scenarioWeights.playback) },
        { duration: '1m', target: 0 },
      ],
      gracefulRampDown: '30s',
      exec: 'playbackScenario',
    },
    auth_spike: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: Math.floor(10000 * scenarioWeights.auth) },
        { duration: '3m', target: Math.floor(10000 * scenarioWeights.auth) },
        { duration: '30s', target: Math.floor(100000 * scenarioWeights.auth) },
        { duration: '2m', target: Math.floor(100000 * scenarioWeights.auth) },
        { duration: '1m', target: Math.floor(10000 * scenarioWeights.auth) },
        { duration: '3m', target: Math.floor(10000 * scenarioWeights.auth) },
        { duration: '1m', target: 0 },
      ],
      gracefulRampDown: '30s',
      exec: 'authScenario',
    },
    sync_spike: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: Math.floor(10000 * scenarioWeights.sync) },
        { duration: '3m', target: Math.floor(10000 * scenarioWeights.sync) },
        { duration: '30s', target: Math.floor(100000 * scenarioWeights.sync) },
        { duration: '2m', target: Math.floor(100000 * scenarioWeights.sync) },
        { duration: '1m', target: Math.floor(10000 * scenarioWeights.sync) },
        { duration: '3m', target: Math.floor(10000 * scenarioWeights.sync) },
        { duration: '1m', target: 0 },
      ],
      gracefulRampDown: '30s',
      exec: 'syncScenario',
    },
  },
  thresholds: {
    // Very relaxed thresholds - focus on system survival and recovery
    'http_req_duration': ['p(95)<3000', 'p(99)<5000'],
    'http_req_failed': ['rate<0.1'], // Allow 10% error during spike
    'errors': ['rate<0.1'],
  },
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)', 'p(99.9)', 'p(99.99)'],
  noConnectionReuse: false,
};

// Setup function
export function setup() {
  const token = authenticate('spike-test@example.com', 'testpass123');
  return { token };
}

// Search scenario - simplified for spike load
export function searchScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'search', method: 'GET' };
  params.timeout = '10s'; // Increased timeout for spike

  const query = generateSearchQuery();
  const res = http.get(
    `${config.discoveryUrl}/api/search?q=${encodeURIComponent(query)}&limit=10`,
    params
  );

  checkResponse(res, 'search');
  sleep(0.5); // Reduced think time to maximize load
}

// Recommendations scenario - simplified
export function recommendationsScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'sona', method: 'GET' };
  params.timeout = '10s';

  const userId = generateUserId();
  const res = http.get(
    `${config.sonaUrl}/api/recommendations/${userId}?limit=5`,
    params
  );

  checkResponse(res, 'sona');
  sleep(0.5);
}

// Playback scenario - basic operations only
export function playbackScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'playback', method: 'POST' };
  params.timeout = '10s';

  const userId = generateUserId();
  const mediaId = generateMediaId();

  // Just start playback - no progress updates during spike
  const res = http.post(
    `${config.playbackUrl}/api/playback/start`,
    JSON.stringify({ user_id: userId, media_id: mediaId }),
    params
  );

  checkResponse(res, 'playback');
  sleep(1);
}

// Auth scenario - login only
export function authScenario() {
  const params = {
    headers: { 'Content-Type': 'application/json' },
    tags: { endpoint: 'auth', method: 'POST' },
    timeout: '10s',
  };

  const loginPayload = JSON.stringify({
    email: `spike-user-${Math.floor(Math.random() * 100000)}@example.com`,
    password: 'testpass123',
  });

  const res = http.post(`${config.authUrl}/api/auth/login`, loginPayload, params);
  checkResponse(res, 'auth');

  sleep(0.5);
}

// Sync scenario - minimal payload
export function syncScenario(data) {
  const params = getAuthHeaders(data.token);
  params.tags = { endpoint: 'sync', method: 'POST' };
  params.timeout = '10s';

  const userId = generateUserId();
  const syncPayload = JSON.stringify({
    user_id: userId,
    device_id: `spike-device-${Math.floor(Math.random() * 10000)}`,
    timestamp: new Date().toISOString(),
    data: {
      preferences: { theme: 'dark' },
    },
  });

  const res = http.post(`${config.syncUrl}/api/sync`, syncPayload, params);
  checkResponse(res, 'sync');

  sleep(0.5);
}

// Teardown function
export function teardown(data) {
  console.log('Spike test completed - analyze recovery time and error rates');
  console.log('Key metrics to review:');
  console.log('- Error rate during spike vs normal');
  console.log('- Response time degradation');
  console.log('- Recovery time to baseline performance');
  console.log('- System stability post-spike');
}
