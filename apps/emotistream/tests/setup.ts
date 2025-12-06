/**
 * Jest Test Setup
 *
 * Global setup and teardown for all tests.
 */

import 'reflect-metadata';

// Set test environment variables
process.env.NODE_ENV = 'test';
process.env.LOG_LEVEL = 'error';
process.env.QTABLE_DB_PATH = ':memory:';
process.env.CONTENT_DB_PATH = ':memory:';

// Global test timeout
jest.setTimeout(10000);

// Mock Gemini API for tests
jest.mock('@google/generative-ai', () => ({
  GoogleGenerativeAI: jest.fn().mockImplementation(() => ({
    getGenerativeModel: jest.fn().mockReturnValue({
      generateContent: jest.fn(),
    }),
  })),
}));

// Suppress console output during tests (optional)
if (process.env.SILENT_TESTS === 'true') {
  global.console = {
    ...console,
    log: jest.fn(),
    debug: jest.fn(),
    info: jest.fn(),
    warn: jest.fn(),
    error: jest.fn(),
  };
}

// Global test utilities
global.beforeAll(() => {
  // Setup before all tests
});

global.afterAll(() => {
  // Cleanup after all tests
});

global.beforeEach(() => {
  // Setup before each test
  jest.clearAllMocks();
});

global.afterEach(() => {
  // Cleanup after each test
});
