/**
 * EmotiStream Structured Logger
 *
 * Provides consistent logging across the application with different log levels.
 */

import { CONFIG } from './config';

/**
 * Log levels
 */
export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

/**
 * Log level mapping
 */
const LOG_LEVEL_MAP: Record<string, LogLevel> = {
  debug: LogLevel.DEBUG,
  info: LogLevel.INFO,
  warn: LogLevel.WARN,
  error: LogLevel.ERROR,
};

/**
 * Log entry structure
 */
interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  context?: string;
  data?: unknown;
  error?: {
    message: string;
    stack?: string;
    code?: string;
  };
}

/**
 * Logger class
 */
export class Logger {
  private currentLevel: LogLevel;
  private pretty: boolean;
  private context?: string;

  constructor(context?: string) {
    this.currentLevel = LOG_LEVEL_MAP[CONFIG.logging.level] || LogLevel.INFO;
    this.pretty = CONFIG.logging.pretty;
    this.context = context;
  }

  /**
   * Create a child logger with additional context
   */
  child(context: string): Logger {
    const childContext = this.context ? `${this.context}:${context}` : context;
    return new Logger(childContext);
  }

  /**
   * Debug log
   */
  debug(message: string, data?: unknown): void {
    this.log(LogLevel.DEBUG, message, data);
  }

  /**
   * Info log
   */
  info(message: string, data?: unknown): void {
    this.log(LogLevel.INFO, message, data);
  }

  /**
   * Warning log
   */
  warn(message: string, data?: unknown): void {
    this.log(LogLevel.WARN, message, data);
  }

  /**
   * Error log
   */
  error(message: string, error?: Error | unknown, data?: unknown): void {
    const errorData = error instanceof Error
      ? {
          message: error.message,
          stack: error.stack,
          code: (error as any).code,
        }
      : { message: String(error) };

    this.log(LogLevel.ERROR, message, data, errorData);
  }

  /**
   * Internal log method
   */
  private log(
    level: LogLevel,
    message: string,
    data?: unknown,
    error?: { message: string; stack?: string; code?: string }
  ): void {
    if (level < this.currentLevel) {
      return;
    }

    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level: LogLevel[level],
      message,
      context: this.context,
      data,
      error,
    };

    const output = this.pretty ? this.formatPretty(entry) : this.formatJSON(entry);
    const logFn = this.getLogFunction(level);
    logFn(output);
  }

  /**
   * Format log entry as JSON
   */
  private formatJSON(entry: LogEntry): string {
    return JSON.stringify(entry);
  }

  /**
   * Format log entry as pretty text
   */
  private formatPretty(entry: LogEntry): string {
    const parts: string[] = [
      `[${entry.timestamp}]`,
      `[${entry.level}]`,
    ];

    if (entry.context) {
      parts.push(`[${entry.context}]`);
    }

    parts.push(entry.message);

    if (entry.data) {
      parts.push('\n  Data:', JSON.stringify(entry.data, null, 2));
    }

    if (entry.error) {
      parts.push('\n  Error:', entry.error.message);
      if (entry.error.code) {
        parts.push(`(${entry.error.code})`);
      }
      if (entry.error.stack) {
        parts.push('\n', entry.error.stack);
      }
    }

    return parts.join(' ');
  }

  /**
   * Get appropriate console method for log level
   */
  private getLogFunction(level: LogLevel): (...args: any[]) => void {
    switch (level) {
      case LogLevel.DEBUG:
        return console.debug;
      case LogLevel.INFO:
        return console.info;
      case LogLevel.WARN:
        return console.warn;
      case LogLevel.ERROR:
        return console.error;
      default:
        return console.log;
    }
  }
}

/**
 * Create default logger instance
 */
export const logger = new Logger('EmotiStream');

/**
 * Create logger for specific module
 */
export const createLogger = (context: string): Logger => {
  return new Logger(context);
};

/**
 * Export default logger
 */
export default logger;
