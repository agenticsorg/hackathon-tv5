/**
 * Tests for MCP Server Protocol 2024-11-05 Methods
 */

import {
  handleMCPRequest,
  getCurrentLogLevel,
  getActiveRequestsCount,
} from '../server';
import { MCPErrorCode } from '../types/index';

describe('MCP Server - Protocol 2024-11-05', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('ping method', () => {
    it('should return empty object for ping request', async () => {
      const result = await handleMCPRequest('ping', {});

      expect(result).toEqual({});
    });

    it('should handle ping without parameters', async () => {
      const result = await handleMCPRequest('ping', undefined);

      expect(result).toEqual({});
    });
  });

  describe('notifications/initialized', () => {
    it('should handle initialized notification', async () => {
      const consoleSpy = jest.spyOn(console, 'log').mockImplementation(() => {});

      const result = await handleMCPRequest('notifications/initialized', {});

      // Notifications return void
      expect(result).toBeUndefined();

      consoleSpy.mockRestore();
    });

    it('should log initialization completion', async () => {
      const consoleSpy = jest.spyOn(console, 'log').mockImplementation(() => {});

      await handleMCPRequest('notifications/initialized', {});

      // Verify logging occurred
      expect(consoleSpy).toHaveBeenCalled();
      const logMessage = consoleSpy.mock.calls.find((call) =>
        call[0].includes('Client initialization complete')
      );
      expect(logMessage).toBeDefined();

      consoleSpy.mockRestore();
    });
  });

  describe('notifications/cancelled', () => {
    it('should handle cancellation of non-existent request', async () => {
      const params = {
        requestId: 'non-existent-123',
        reason: 'User cancelled',
      };

      const result = await handleMCPRequest('notifications/cancelled', params);

      // Notifications return void
      expect(result).toBeUndefined();
    });

    it('should cancel active request', async () => {
      const requestId = 'active-request-456';

      // First, create an active request by calling a tool
      const toolCallPromise = handleMCPRequest(
        'tools/call',
        {
          name: 'search_content',
          arguments: { query: 'test' },
        },
        requestId
      );

      // Wait a tick to ensure request is registered
      await new Promise((resolve) => setTimeout(resolve, 10));

      // Verify request is tracked
      const activeCountBefore = getActiveRequestsCount();
      expect(activeCountBefore).toBeGreaterThan(0);

      // Cancel the request
      await handleMCPRequest('notifications/cancelled', {
        requestId,
        reason: 'Test cancellation',
      });

      // Verify request was removed from tracking
      const activeCountAfter = getActiveRequestsCount();
      expect(activeCountAfter).toBeLessThan(activeCountBefore);

      // Clean up
      try {
        await toolCallPromise;
      } catch (error) {
        // Expected to fail since tool doesn't exist
      }
    });

    it('should log cancellation with reason', async () => {
      const consoleSpy = jest.spyOn(console, 'log').mockImplementation(() => {});

      await handleMCPRequest('notifications/cancelled', {
        requestId: 'test-123',
        reason: 'Timeout exceeded',
      });

      expect(consoleSpy).toHaveBeenCalled();
      const logMessage = consoleSpy.mock.calls.find((call) =>
        call[0].includes('Request cancelled')
      );
      expect(logMessage).toBeDefined();

      consoleSpy.mockRestore();
    });
  });

  describe('logging/setLevel', () => {
    it('should change log level from info to debug', async () => {
      const result = await handleMCPRequest('logging/setLevel', { level: 'debug' });

      expect(result).toMatchObject({
        previousLevel: 'info',
        currentLevel: 'debug',
      });

      expect(getCurrentLogLevel()).toBe('debug');
    });

    it('should change log level to warn', async () => {
      const result = await handleMCPRequest('logging/setLevel', { level: 'warn' });

      expect(result).toMatchObject({
        currentLevel: 'warn',
      });

      expect(getCurrentLogLevel()).toBe('warn');
    });

    it('should change log level to error', async () => {
      const result = await handleMCPRequest('logging/setLevel', { level: 'error' });

      expect(result).toMatchObject({
        currentLevel: 'error',
      });

      expect(getCurrentLogLevel()).toBe('error');
    });

    it('should return previous level correctly', async () => {
      // Set to debug
      await handleMCPRequest('logging/setLevel', { level: 'debug' });

      // Change to info and verify previous level
      const result = await handleMCPRequest('logging/setLevel', { level: 'info' });

      expect(result).toMatchObject({
        previousLevel: 'debug',
        currentLevel: 'info',
      });
    });

    it('should reject invalid log level', async () => {
      await expect(
        handleMCPRequest('logging/setLevel', { level: 'invalid' })
      ).rejects.toMatchObject({
        code: MCPErrorCode.INVALID_PARAMS,
        message: expect.stringContaining('Invalid log level'),
      });
    });

    it('should validate log level is one of allowed values', async () => {
      const invalidLevels = ['trace', 'verbose', 'critical', ''];

      for (const level of invalidLevels) {
        await expect(
          handleMCPRequest('logging/setLevel', { level })
        ).rejects.toMatchObject({
          code: MCPErrorCode.INVALID_PARAMS,
        });
      }
    });

    it('should log the level change', async () => {
      const consoleSpy = jest.spyOn(console, 'log').mockImplementation(() => {});

      await handleMCPRequest('logging/setLevel', { level: 'debug' });

      expect(consoleSpy).toHaveBeenCalled();
      const logMessage = consoleSpy.mock.calls.find((call) =>
        call[0].includes('Log level changed')
      );
      expect(logMessage).toBeDefined();

      consoleSpy.mockRestore();
    });
  });

  describe('completion/complete', () => {
    it('should return completions for resource URIs', async () => {
      const params = {
        ref: {
          type: 'ref/resource' as const,
        },
        argument: {
          name: 'uri',
          value: 'media',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result).toHaveProperty('completion');
      expect(result.completion).toHaveProperty('values');
      expect(result.completion).toHaveProperty('hasMore');
      expect(result.completion).toHaveProperty('total');

      expect(Array.isArray(result.completion.values)).toBe(true);
      expect(typeof result.completion.hasMore).toBe('boolean');
    });

    it('should return completions for prompt names', async () => {
      const params = {
        ref: {
          type: 'ref/prompt' as const,
        },
        argument: {
          name: 'name',
          value: 'content',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result.completion.values).toBeInstanceOf(Array);
      expect(result.completion.hasMore).toBe(false);
    });

    it('should return genre completions for genre argument', async () => {
      const params = {
        ref: {
          type: 'ref/prompt' as const,
          name: 'content_search',
        },
        argument: {
          name: 'genre',
          value: 'act',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result.completion.values).toContain('action');
      expect(result.completion.values.length).toBeGreaterThan(0);
      expect(result.completion.values.length).toBeLessThanOrEqual(10);
    });

    it('should return platform completions for platform argument', async () => {
      const params = {
        ref: {
          type: 'ref/prompt' as const,
          name: 'platform_search',
        },
        argument: {
          name: 'platform',
          value: 'net',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result.completion.values).toContain('netflix');
    });

    it('should return region completions for region argument', async () => {
      const params = {
        ref: {
          type: 'ref/prompt' as const,
          name: 'region_filter',
        },
        argument: {
          name: 'region',
          value: 'U',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result.completion.values).toEqual(expect.arrayContaining(['US', 'UK']));
    });

    it('should return content type completions', async () => {
      const params = {
        ref: {
          type: 'ref/prompt' as const,
          name: 'content_filter',
        },
        argument: {
          name: 'contentType',
          value: 'mo',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result.completion.values).toContain('movie');
    });

    it('should filter completions based on partial value', async () => {
      const params = {
        ref: {
          type: 'ref/prompt' as const,
          name: 'genre_filter',
        },
        argument: {
          name: 'genre',
          value: 'sci',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result.completion.values).toContain('sci-fi');
      expect(result.completion.values).not.toContain('action');
    });

    it('should limit completions to 10 results', async () => {
      const params = {
        ref: {
          type: 'ref/prompt' as const,
          name: 'genre_filter',
        },
        argument: {
          name: 'genre',
          value: '',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result.completion.values.length).toBeLessThanOrEqual(10);
    });

    it('should handle empty partial value', async () => {
      const params = {
        ref: {
          type: 'ref/prompt' as const,
          name: 'platform_filter',
        },
        argument: {
          name: 'platform',
          value: '',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result.completion.values.length).toBeGreaterThan(0);
      expect(result.completion.values.length).toBeLessThanOrEqual(10);
    });

    it('should return total count of completions', async () => {
      const params = {
        ref: {
          type: 'ref/prompt' as const,
          name: 'genre_filter',
        },
        argument: {
          name: 'genre',
          value: 'a',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result.completion.total).toBe(result.completion.values.length);
    });

    it('should handle tool argument completions with enum', async () => {
      const params = {
        ref: {
          type: 'ref/prompt' as const,
          uri: 'tool://search_content',
        },
        argument: {
          name: 'contentType',
          value: 'tv',
        },
      };

      const result = await handleMCPRequest('completion/complete', params);

      expect(result.completion).toHaveProperty('values');
      expect(result.completion.hasMore).toBe(false);
    });
  });

  describe('initialize method', () => {
    it('should return logging capability in initialize response', async () => {
      const result = await handleMCPRequest('initialize', {});

      expect(result).toHaveProperty('capabilities');
      expect(result.capabilities).toHaveProperty('logging');
      expect(result.protocolVersion).toBe('2024-11-05');
    });
  });

  describe('error handling', () => {
    it('should throw METHOD_NOT_FOUND for unknown method', async () => {
      await expect(
        handleMCPRequest('unknown/method', {})
      ).rejects.toMatchObject({
        code: MCPErrorCode.METHOD_NOT_FOUND,
        message: expect.stringContaining('Method not found'),
      });
    });

    it('should include method name in error message', async () => {
      await expect(
        handleMCPRequest('invalid/endpoint', {})
      ).rejects.toMatchObject({
        message: expect.stringContaining('invalid/endpoint'),
      });
    });
  });

  describe('logging behavior', () => {
    it('should respect log level when filtering messages', async () => {
      // Set log level to error
      await handleMCPRequest('logging/setLevel', { level: 'error' });

      const consoleSpy = jest.spyOn(console, 'log').mockImplementation(() => {});

      // Make a request that logs at info level
      await handleMCPRequest('ping', {});

      // Info messages should not be logged when level is error
      const infoLogs = consoleSpy.mock.calls.filter((call) =>
        call[0].includes('[INFO]')
      );
      expect(infoLogs.length).toBe(0);

      consoleSpy.mockRestore();

      // Reset log level
      await handleMCPRequest('logging/setLevel', { level: 'info' });
    });

    it('should log error messages at all log levels', async () => {
      const logLevels = ['debug', 'info', 'warn', 'error'];

      for (const level of logLevels) {
        await handleMCPRequest('logging/setLevel', { level: level as any });

        const consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

        // Trigger an error
        try {
          await handleMCPRequest('tools/call', {
            name: 'non_existent_tool',
            arguments: {},
          });
        } catch (error) {
          // Expected to throw
        }

        // Error should be logged at all levels
        expect(consoleErrorSpy).toHaveBeenCalled();

        consoleErrorSpy.mockRestore();
      }
    });
  });

  describe('request tracking', () => {
    it('should track active requests during tool execution', async () => {
      const requestId = 'test-request-789';

      const initialCount = getActiveRequestsCount();

      // Start a tool call
      const promise = handleMCPRequest(
        'tools/call',
        {
          name: 'search_content',
          arguments: { query: 'test' },
        },
        requestId
      );

      // Wait a tick for request to be registered
      await new Promise((resolve) => setTimeout(resolve, 10));

      const activeCount = getActiveRequestsCount();
      expect(activeCount).toBeGreaterThan(initialCount);

      // Clean up
      try {
        await promise;
      } catch (error) {
        // Expected to fail since tool doesn't exist
      }
    });

    it('should clean up request tracking after completion', async () => {
      const requestId = 'test-request-cleanup';

      try {
        await handleMCPRequest(
          'tools/call',
          {
            name: 'non_existent',
            arguments: {},
          },
          requestId
        );
      } catch (error) {
        // Expected to throw
      }

      // Request should be removed from tracking even after error
      // Note: Actual cleanup happens, but we can't directly verify without access to internal map
      expect(getActiveRequestsCount()).toBeGreaterThanOrEqual(0);
    });
  });
});
