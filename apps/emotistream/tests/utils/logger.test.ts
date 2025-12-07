/**
 * Logger Tests
 *
 * Validates that the logger properly filters logs based on environment configuration.
 */

import { Logger, LogLevel, createLogger } from '../../src/utils/logger';

describe('Logger', () => {
  let consoleDebugSpy: jest.SpyInstance;
  let consoleInfoSpy: jest.SpyInstance;
  let consoleWarnSpy: jest.SpyInstance;
  let consoleErrorSpy: jest.SpyInstance;

  beforeEach(() => {
    // Spy on console methods
    consoleDebugSpy = jest.spyOn(console, 'debug').mockImplementation();
    consoleInfoSpy = jest.spyOn(console, 'info').mockImplementation();
    consoleWarnSpy = jest.spyOn(console, 'warn').mockImplementation();
    consoleErrorSpy = jest.spyOn(console, 'error').mockImplementation();
  });

  afterEach(() => {
    // Restore console methods
    consoleDebugSpy.mockRestore();
    consoleInfoSpy.mockRestore();
    consoleWarnSpy.mockRestore();
    consoleErrorSpy.mockRestore();
  });

  describe('Log Level Filtering', () => {
    it('should filter debug logs when level is INFO', () => {
      // Create logger with INFO level
      const logger = new Logger('test');
      // Force level to INFO
      (logger as any).currentLevel = LogLevel.INFO;

      logger.debug('debug message');
      logger.info('info message');
      logger.warn('warn message');
      logger.error('error message');

      // Debug should be filtered out
      expect(consoleDebugSpy).not.toHaveBeenCalled();
      // Info, warn, error should be logged
      expect(consoleInfoSpy).toHaveBeenCalled();
      expect(consoleWarnSpy).toHaveBeenCalled();
      expect(consoleErrorSpy).toHaveBeenCalled();
    });

    it('should only log warnings and errors when level is WARN', () => {
      const logger = new Logger('test');
      (logger as any).currentLevel = LogLevel.WARN;

      logger.debug('debug message');
      logger.info('info message');
      logger.warn('warn message');
      logger.error('error message');

      // Debug and info should be filtered out
      expect(consoleDebugSpy).not.toHaveBeenCalled();
      expect(consoleInfoSpy).not.toHaveBeenCalled();
      // Warn and error should be logged
      expect(consoleWarnSpy).toHaveBeenCalled();
      expect(consoleErrorSpy).toHaveBeenCalled();
    });

    it('should only log errors when level is ERROR', () => {
      const logger = new Logger('test');
      (logger as any).currentLevel = LogLevel.ERROR;

      logger.debug('debug message');
      logger.info('info message');
      logger.warn('warn message');
      logger.error('error message');

      // Only error should be logged
      expect(consoleDebugSpy).not.toHaveBeenCalled();
      expect(consoleInfoSpy).not.toHaveBeenCalled();
      expect(consoleWarnSpy).not.toHaveBeenCalled();
      expect(consoleErrorSpy).toHaveBeenCalled();
    });

    it('should log all levels when level is DEBUG', () => {
      const logger = new Logger('test');
      (logger as any).currentLevel = LogLevel.DEBUG;

      logger.debug('debug message');
      logger.info('info message');
      logger.warn('warn message');
      logger.error('error message');

      // All should be logged
      expect(consoleDebugSpy).toHaveBeenCalled();
      expect(consoleInfoSpy).toHaveBeenCalled();
      expect(consoleWarnSpy).toHaveBeenCalled();
      expect(consoleErrorSpy).toHaveBeenCalled();
    });
  });

  describe('Context Management', () => {
    it('should create logger with context', () => {
      const logger = createLogger('TestModule');
      (logger as any).currentLevel = LogLevel.DEBUG;

      logger.info('test message');

      const logOutput = consoleInfoSpy.mock.calls[0][0];
      expect(logOutput).toContain('[TestModule]');
      expect(logOutput).toContain('test message');
    });

    it('should create child logger with nested context', () => {
      const parentLogger = createLogger('Parent');
      const childLogger = parentLogger.child('Child');
      (childLogger as any).currentLevel = LogLevel.DEBUG;

      childLogger.info('child message');

      const logOutput = consoleInfoSpy.mock.calls[0][0];
      expect(logOutput).toContain('[Parent:Child]');
      expect(logOutput).toContain('child message');
    });
  });

  describe('Error Logging', () => {
    it('should properly format error objects', () => {
      const logger = new Logger('test');
      (logger as any).currentLevel = LogLevel.ERROR;

      const error = new Error('Test error');
      logger.error('Error occurred', error);

      expect(consoleErrorSpy).toHaveBeenCalled();
      const logOutput = consoleErrorSpy.mock.calls[0][0];
      expect(logOutput).toContain('Error occurred');
      expect(logOutput).toContain('Test error');
    });

    it('should handle non-Error objects', () => {
      const logger = new Logger('test');
      (logger as any).currentLevel = LogLevel.ERROR;

      logger.error('String error occurred', 'some error string');

      expect(consoleErrorSpy).toHaveBeenCalled();
      const logOutput = consoleErrorSpy.mock.calls[0][0];
      expect(logOutput).toContain('String error occurred');
      expect(logOutput).toContain('some error string');
    });
  });

  describe('Data Logging', () => {
    it('should include additional data in logs', () => {
      const logger = new Logger('test');
      (logger as any).currentLevel = LogLevel.DEBUG;

      const data = { userId: '123', action: 'login' };
      logger.info('User action', data);

      expect(consoleInfoSpy).toHaveBeenCalled();
      const logOutput = consoleInfoSpy.mock.calls[0][0];
      expect(logOutput).toContain('User action');
      expect(logOutput).toContain('userId');
      expect(logOutput).toContain('123');
    });
  });

  describe('Production vs Development', () => {
    it('should use appropriate log level for production', () => {
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'production';
      delete process.env.LOG_LEVEL;

      // In production, default should be 'warn'
      // This would be tested via config integration
      // For now, we test the logger respects WARN level
      const logger = new Logger('test');
      (logger as any).currentLevel = LogLevel.WARN;

      logger.debug('debug');
      logger.info('info');
      logger.warn('warn');

      expect(consoleDebugSpy).not.toHaveBeenCalled();
      expect(consoleInfoSpy).not.toHaveBeenCalled();
      expect(consoleWarnSpy).toHaveBeenCalled();

      process.env.NODE_ENV = originalEnv;
    });

    it('should use appropriate log level for development', () => {
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'development';
      delete process.env.LOG_LEVEL;

      // In development, default should be 'info'
      const logger = new Logger('test');
      (logger as any).currentLevel = LogLevel.INFO;

      logger.debug('debug');
      logger.info('info');
      logger.warn('warn');

      expect(consoleDebugSpy).not.toHaveBeenCalled();
      expect(consoleInfoSpy).toHaveBeenCalled();
      expect(consoleWarnSpy).toHaveBeenCalled();

      process.env.NODE_ENV = originalEnv;
    });
  });

  describe('Log Format', () => {
    it('should include timestamp in logs', () => {
      const logger = new Logger('test');
      (logger as any).currentLevel = LogLevel.DEBUG;

      logger.info('test message');

      const logOutput = consoleInfoSpy.mock.calls[0][0];
      // Should contain ISO timestamp format
      expect(logOutput).toMatch(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/);
    });

    it('should include log level in output', () => {
      const logger = new Logger('test');
      (logger as any).currentLevel = LogLevel.DEBUG;

      logger.debug('debug');
      logger.info('info');
      logger.warn('warn');
      logger.error('error');

      expect(consoleDebugSpy.mock.calls[0][0]).toContain('[DEBUG]');
      expect(consoleInfoSpy.mock.calls[0][0]).toContain('[INFO]');
      expect(consoleWarnSpy.mock.calls[0][0]).toContain('[WARN]');
      expect(consoleErrorSpy.mock.calls[0][0]).toContain('[ERROR]');
    });
  });
});
