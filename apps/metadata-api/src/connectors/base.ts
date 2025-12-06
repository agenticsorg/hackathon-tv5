/**
 * Base Platform Connector Interface
 *
 * Defines the contract for platform-specific metadata connectors
 * that transform Nexus-UMMID metadata into platform-specific formats.
 */

import { MediaMetadata, ValidationResult } from '../types/index.js';

/**
 * Platform Connector Interface
 *
 * All platform connectors (Amazon MEC, Netflix IMF, Apple UMC, etc.)
 * must implement this interface to ensure consistent behavior.
 */
export interface PlatformConnector {
  /**
   * Platform identifier
   */
  readonly platform: string;

  /**
   * Generate platform-specific feed/package
   *
   * @param metadata - Source media metadata from Nexus-UMMID
   * @returns Platform-specific feed structure
   */
  generateFeed(metadata: MediaMetadata): Promise<unknown>;

  /**
   * Validate metadata against platform requirements
   *
   * @param metadata - Media metadata to validate
   * @returns Validation result with errors and warnings
   */
  validate(metadata: MediaMetadata): Promise<ValidationResult>;

  /**
   * Export feed to string format (XML, JSON, etc.)
   *
   * @param feed - Platform-specific feed structure
   * @returns Serialized feed content
   */
  exportFeed(feed: unknown): string;
}

/**
 * Base Platform Connector
 *
 * Abstract base class providing common functionality for platform connectors
 */
export abstract class BasePlatformConnector implements PlatformConnector {
  abstract readonly platform: string;

  abstract generateFeed(metadata: MediaMetadata): Promise<unknown>;
  abstract validate(metadata: MediaMetadata): Promise<ValidationResult>;
  abstract exportFeed(feed: unknown): string;

  /**
   * Check if required field is present
   *
   * @param value - Value to check
   * @param fieldName - Name of the field for error reporting
   * @returns True if valid, false otherwise
   */
  protected hasRequiredField(value: unknown, fieldName: string): boolean {
    return value !== undefined && value !== null && value !== '';
  }

  /**
   * Validate date format
   *
   * @param date - Date to validate
   * @returns True if valid date, false otherwise
   */
  protected isValidDate(date: unknown): boolean {
    if (!date) return false;
    const d = date instanceof Date ? date : new Date(date as string);
    return d instanceof Date && !isNaN(d.getTime());
  }

  /**
   * Format date to ISO 8601
   *
   * @param date - Date to format
   * @returns ISO 8601 formatted date string
   */
  protected formatDate(date: Date | string): string {
    const d = date instanceof Date ? date : new Date(date);
    return d.toISOString();
  }
}
