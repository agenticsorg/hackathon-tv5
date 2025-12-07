/**
 * Platform Validator for Distribution Readiness
 *
 * Validates movies against Netflix IMF, Amazon MEC, and FAST MRSS requirements
 * and determines distribution readiness for each platform.
 */

import { MovieNode, PlatformValidation } from './schema';
import { NetflixIMFConnector } from '../connectors/netflix-imf';
import { AmazonMECConnector } from '../connectors/amazon-mec';
import { FASTMRSSConnector } from '../connectors/fast-mrss';
import { Platform } from '../connectors/types';
import { MediaMetadata } from '../types';
import { logger } from '../utils/logger';

/**
 * Platform readiness result for a movie
 */
export interface PlatformReadiness {
  netflix: boolean;
  amazon: boolean;
  fast: boolean;
  validationResults: PlatformValidation[];
  overallScore: number; // 0-100
}

/**
 * Batch validation result
 */
export interface BatchValidationResult {
  validated: number;
  netflixReady: number;
  amazonReady: number;
  fastReady: number;
  failed: number;
  durationMs: number;
}

// Connector instances (singleton pattern)
let netflixConnector: NetflixIMFConnector | null = null;
let amazonConnector: AmazonMECConnector | null = null;
let fastConnector: FASTMRSSConnector | null = null;

function getNetflixConnector(): NetflixIMFConnector {
  if (!netflixConnector) {
    netflixConnector = new NetflixIMFConnector();
  }
  return netflixConnector;
}

function getAmazonConnector(): AmazonMECConnector {
  if (!amazonConnector) {
    amazonConnector = new AmazonMECConnector();
  }
  return amazonConnector;
}

function getFastConnector(): FASTMRSSConnector {
  if (!fastConnector) {
    fastConnector = new FASTMRSSConnector(Platform.FAST_PLUTO);
  }
  return fastConnector;
}

/**
 * Convert MovieNode to MediaMetadata for connector validation
 */
function movieToMediaMetadata(movie: MovieNode): MediaMetadata {
  return {
    id: movie.id,
    title: movie.title,
    type: 'movie',
    synopsis: movie.overview || '',
    genres: [], // Genres are stored in edges, not directly on movie
    keywords: [],
    language: 'en', // Default, would need to query edges for actual language
    rating: movie.adult ? 'R' : 'PG-13',
    duration: movie.runtime,
    releaseDate: movie.releaseDate ? new Date(movie.releaseDate) : undefined,
    resolution: '4K', // Assume 4K for validation purposes
    createdAt: new Date(movie.createdAt),
    updatedAt: new Date(movie.updatedAt),
  };
}

/**
 * Validate a movie against Netflix requirements
 */
function validateForNetflix(movie: MovieNode): PlatformValidation {
  const connector = getNetflixConnector();
  const metadata = movieToMediaMetadata(movie);

  try {
    const result = connector.validate(metadata);

    // Calculate compliance score based on errors and warnings
    const errorWeight = 10;
    const warningWeight = 2;
    const errorPenalty = (result.errors?.length || 0) * errorWeight;
    const warningPenalty = (result.warnings?.length || 0) * warningWeight;
    const score = Math.max(0, 100 - errorPenalty - warningPenalty);

    return {
      platform: 'netflix',
      valid: result.valid,
      score,
      errors: (result.errors || []).map(e => ({
        field: e.field,
        message: e.message,
        severity: e.severity as 'critical' | 'error',
        code: e.platformRequirement || 'NETFLIX_ERR',
      })),
      warnings: (result.warnings || []).map(w => ({
        field: w.field,
        message: w.message,
        recommendation: w.recommendation || '',
      })),
      validatedAt: new Date().toISOString(),
    };
  } catch (error) {
    logger.error('Netflix validation error', { movieId: movie.id, error });
    return {
      platform: 'netflix',
      valid: false,
      score: 0,
      errors: [{
        field: 'system',
        message: error instanceof Error ? error.message : 'Validation failed',
        severity: 'critical',
        code: 'NETFLIX_SYSTEM_ERR',
      }],
      warnings: [],
      validatedAt: new Date().toISOString(),
    };
  }
}

/**
 * Validate a movie against Amazon MEC requirements
 */
async function validateForAmazon(movie: MovieNode): Promise<PlatformValidation> {
  const connector = getAmazonConnector();
  const metadata = movieToMediaMetadata(movie);

  try {
    const result = await connector.validate(metadata);

    const errorWeight = 10;
    const warningWeight = 2;
    const errorPenalty = (result.errors?.length || 0) * errorWeight;
    const warningPenalty = (result.warnings?.length || 0) * warningWeight;
    const score = Math.max(0, 100 - errorPenalty - warningPenalty);

    return {
      platform: 'amazon',
      valid: result.valid,
      score,
      errors: (result.errors || []).map(e => ({
        field: e.field,
        message: e.message,
        severity: e.severity as 'critical' | 'error',
        code: e.platformRequirement || 'AMAZON_ERR',
      })),
      warnings: (result.warnings || []).map(w => ({
        field: w.field,
        message: w.message,
        recommendation: w.recommendation || '',
      })),
      validatedAt: new Date().toISOString(),
    };
  } catch (error) {
    logger.error('Amazon validation error', { movieId: movie.id, error });
    return {
      platform: 'amazon',
      valid: false,
      score: 0,
      errors: [{
        field: 'system',
        message: error instanceof Error ? error.message : 'Validation failed',
        severity: 'critical',
        code: 'AMAZON_SYSTEM_ERR',
      }],
      warnings: [],
      validatedAt: new Date().toISOString(),
    };
  }
}

/**
 * Validate a movie against FAST MRSS requirements
 */
async function validateForFAST(movie: MovieNode): Promise<PlatformValidation> {
  const connector = getFastConnector();
  const metadata = movieToMediaMetadata(movie);

  try {
    const result = await connector.validate(metadata);

    const errorWeight = 10;
    const warningWeight = 2;
    const errorPenalty = (result.errors?.length || 0) * errorWeight;
    const warningPenalty = (result.warnings?.length || 0) * warningWeight;
    const score = Math.max(0, 100 - errorPenalty - warningPenalty);

    return {
      platform: 'fast',
      valid: result.valid,
      score,
      errors: (result.errors || []).map(e => ({
        field: e.field,
        message: e.message,
        severity: e.severity as 'critical' | 'error',
        code: e.platformRequirement || 'FAST_ERR',
      })),
      warnings: (result.warnings || []).map(w => ({
        field: w.field,
        message: w.message,
        recommendation: w.recommendation || '',
      })),
      validatedAt: new Date().toISOString(),
    };
  } catch (error) {
    logger.error('FAST validation error', { movieId: movie.id, error });
    return {
      platform: 'fast',
      valid: false,
      score: 0,
      errors: [{
        field: 'system',
        message: error instanceof Error ? error.message : 'Validation failed',
        severity: 'critical',
        code: 'FAST_SYSTEM_ERR',
      }],
      warnings: [],
      validatedAt: new Date().toISOString(),
    };
  }
}

/**
 * Validate a movie against all platforms
 */
export async function validateMovieForAllPlatforms(movie: MovieNode): Promise<PlatformReadiness> {
  // Run validations
  const netflixResult = validateForNetflix(movie);
  const [amazonResult, fastResult] = await Promise.all([
    validateForAmazon(movie),
    validateForFAST(movie),
  ]);

  const validationResults = [netflixResult, amazonResult, fastResult];

  // Calculate overall score (average of all platform scores)
  const overallScore = Math.round(
    validationResults.reduce((sum, r) => sum + r.score, 0) / validationResults.length
  );

  return {
    netflix: netflixResult.valid,
    amazon: amazonResult.valid,
    fast: fastResult.valid,
    validationResults,
    overallScore,
  };
}

/**
 * Quick validation check - returns just boolean readiness without full validation details
 * This is faster for use during ingestion
 */
export function quickValidateMovie(movie: MovieNode): {
  netflix: boolean;
  amazon: boolean;
  fast: boolean;
} {
  // Quick checks based on required fields
  const hasRequiredBase = Boolean(
    movie.title &&
    movie.title.length >= 1 &&
    movie.overview &&
    movie.overview.length >= 10
  );

  // Netflix requires: title, synopsis (50+ chars), runtime, release date
  const netflixReady = hasRequiredBase &&
    movie.overview.length >= 50 &&
    movie.runtime !== undefined &&
    movie.runtime > 0 &&
    movie.releaseDate !== undefined;

  // Amazon requires: title, synopsis, runtime, genre (via edges)
  const amazonReady = hasRequiredBase &&
    movie.runtime !== undefined &&
    movie.runtime > 0;

  // FAST requires: title, description, duration, thumbnail
  const fastReady = hasRequiredBase &&
    movie.runtime !== undefined &&
    movie.runtime > 0 &&
    (movie.posterPath !== undefined || movie.backdropPath !== undefined);

  return {
    netflix: Boolean(netflixReady),
    amazon: Boolean(amazonReady),
    fast: Boolean(fastReady),
  };
}

/**
 * Get distribution status based on platform readiness
 */
export function getDistributionStatus(readiness: {
  netflix: boolean;
  amazon: boolean;
  fast: boolean;
}): 'pending' | 'validated' | 'ready' | 'failed' {
  const platforms = [readiness.netflix, readiness.amazon, readiness.fast];
  const readyCount = platforms.filter(Boolean).length;

  if (readyCount === 0) return 'failed';
  if (readyCount === 3) return 'ready';
  if (readyCount >= 1) return 'validated';
  return 'pending';
}
