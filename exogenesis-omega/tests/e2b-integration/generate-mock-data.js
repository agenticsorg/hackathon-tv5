#!/usr/bin/env node
/**
 * Generate Mock TV Viewing Data for Exogenesis Omega Testing
 * Uses deterministic generation for reproducible tests
 */

import { writeFileSync } from 'fs';
import { randomUUID } from 'crypto';

// Deterministic random number generator
class SeededRandom {
  constructor(seed = 42) {
    this.seed = seed;
  }

  next() {
    this.seed = (this.seed * 1103515245 + 12345) & 0x7fffffff;
    return this.seed / 0x7fffffff;
  }

  nextInt(min, max) {
    return Math.floor(this.next() * (max - min + 1)) + min;
  }

  choice(arr) {
    return arr[this.nextInt(0, arr.length - 1)];
  }
}

const CONTENT_TYPES = ['movie', 'series', 'documentary', 'sports', 'news', 'live'];
const GENRES = ['action', 'comedy', 'drama', 'horror', 'sci-fi', 'romance', 'thriller', 'documentary', 'sports', 'news'];
const TIMES_OF_DAY = ['morning', 'afternoon', 'evening', 'night'];
const DAYS_OF_WEEK = ['monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday'];
const USER_ACTIONS = ['play', 'pause', 'resume', 'stop', 'skip', 'rewind', 'fast_forward', 'complete'];
const RECOMMENDATION_SOURCES = ['home', 'search', 'continue_watching', 'recommended', 'trending', 'category'];

function generateDeviceId(rng, index) {
  const hex = index.toString(16).padStart(8, '0');
  return `tv-${hex}`;
}

function generateContentId(rng) {
  return `content-${rng.nextInt(1, 999999).toString().padStart(6, '0')}`;
}

function generateViewingEvent(rng, deviceId, sessionId) {
  const contentType = rng.choice(CONTENT_TYPES);
  const watchPercentage = Math.round(rng.next() * 100) / 100;

  return {
    device_id: deviceId,
    session_id: sessionId,
    content_id: generateContentId(rng),
    content_type: contentType,
    genre: rng.choice(GENRES),
    watch_percentage: watchPercentage,
    watch_duration_seconds: rng.nextInt(60, 7200),
    time_of_day: rng.choice(TIMES_OF_DAY),
    day_of_week: rng.choice(DAYS_OF_WEEK),
    user_action: watchPercentage >= 0.9 ? 'complete' : rng.choice(USER_ACTIONS),
    interaction_count: rng.nextInt(0, 20),
    timestamp: new Date(Date.now() - rng.nextInt(0, 86400000 * 7)).toISOString(),
    context: {
      previous_content: rng.next() > 0.3 ? generateContentId(rng) : null,
      recommendation_source: rng.choice(RECOMMENDATION_SOURCES),
      household_size: rng.nextInt(1, 5)
    }
  };
}

function generateClientData(clientIndex, eventsPerClient, seed) {
  const rng = new SeededRandom(seed + clientIndex * 1000);
  const deviceId = generateDeviceId(rng, clientIndex);
  const sessionId = randomUUID();

  const events = [];
  for (let i = 0; i < eventsPerClient; i++) {
    events.push(generateViewingEvent(rng, deviceId, sessionId));
  }

  return {
    client_id: clientIndex,
    device_id: deviceId,
    session_id: sessionId,
    events
  };
}

function generateTestData(numClients = 10, eventsPerClient = 100, seed = 42) {
  console.log(`Generating test data for ${numClients} clients, ${eventsPerClient} events each...`);

  const clients = [];
  for (let i = 0; i < numClients; i++) {
    clients.push(generateClientData(i, eventsPerClient, seed));
  }

  const testData = {
    metadata: {
      generated_at: new Date().toISOString(),
      num_clients: numClients,
      events_per_client: eventsPerClient,
      total_events: numClients * eventsPerClient,
      seed
    },
    clients
  };

  return testData;
}

// Generate and save test data
const testData = generateTestData(10, 100, 42);

writeFileSync('test-data.json', JSON.stringify(testData, null, 2));
console.log(`Generated ${testData.metadata.total_events} events for ${testData.metadata.num_clients} clients`);
console.log('Saved to test-data.json');

// Also generate individual client files for parallel testing
for (const client of testData.clients) {
  writeFileSync(`client-${client.client_id}.json`, JSON.stringify(client, null, 2));
}
console.log(`Generated ${testData.metadata.num_clients} individual client files`);

export { generateTestData, generateViewingEvent, generateClientData };
