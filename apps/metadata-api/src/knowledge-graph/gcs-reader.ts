/**
 * GCS Reader for TMDB Dataset
 *
 * Streams the TMDB movie dataset from Google Cloud Storage bucket
 * for processing into the knowledge graph.
 */

import { Storage } from '@google-cloud/storage';
import { Transform } from 'stream';
import { parse } from 'csv-parse';
import { logger } from '../utils/logger';
import { TMDBMovieRow } from './schema';

/**
 * GCS Reader Configuration
 */
export interface GCSReaderConfig {
  projectId: string;
  bucketName: string;
  fileName: string;
}

const DEFAULT_CONFIG: GCSReaderConfig = {
  projectId: process.env.GCP_PROJECT_ID || 'agentics-foundation25lon-1899',
  bucketName: 'nexus-ummid-datasets',
  fileName: 'TMDB_movie_dataset_v11.csv',
};

/**
 * Progress callback for streaming
 */
export type ProgressCallback = (stats: {
  processed: number;
  bytesRead: number;
  currentTitle?: string;
}) => void;

/**
 * GCS Reader for streaming TMDB dataset
 */
export class GCSReader {
  private storage: Storage;
  private config: GCSReaderConfig;

  constructor(config?: Partial<GCSReaderConfig>) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.storage = new Storage({ projectId: this.config.projectId });

    logger.info('GCS Reader initialized', {
      bucket: this.config.bucketName,
      file: this.config.fileName,
    });
  }

  /**
   * Get file metadata
   */
  async getFileMetadata(): Promise<{
    size: number;
    updated: string;
    contentType: string;
  }> {
    const bucket = this.storage.bucket(this.config.bucketName);
    const file = bucket.file(this.config.fileName);
    const [metadata] = await file.getMetadata();

    return {
      size: parseInt(metadata.size as string, 10),
      updated: metadata.updated as string,
      contentType: metadata.contentType as string,
    };
  }

  /**
   * Stream TMDB rows from GCS with CSV parsing
   *
   * @param onProgress - Progress callback
   * @param limit - Optional limit on number of rows
   * @yields TMDBMovieRow objects
   */
  async *streamRows(
    onProgress?: ProgressCallback,
    limit?: number
  ): AsyncGenerator<TMDBMovieRow, void, unknown> {
    const bucket = this.storage.bucket(this.config.bucketName);
    const file = bucket.file(this.config.fileName);

    logger.info('Starting GCS stream', {
      bucket: this.config.bucketName,
      file: this.config.fileName,
      limit,
    });

    let processed = 0;
    let bytesRead = 0;

    // Create read stream from GCS
    const readStream = file.createReadStream();

    // Track bytes read
    const byteCounter = new Transform({
      transform(chunk, _encoding, callback) {
        bytesRead += chunk.length;
        callback(null, chunk);
      },
    });

    // CSV parser configuration
    const csvParser = parse({
      columns: true,          // Use first row as headers
      skip_empty_lines: true,
      trim: true,
      relax_column_count: true,
      relax_quotes: true,
      cast: false,            // Keep everything as strings for manual parsing
      on_record: (record) => {
        // Clean up any problematic fields
        return record;
      },
    });

    // Error handling
    readStream.on('error', (error) => {
      logger.error('GCS read stream error', { error: error.message });
      throw error;
    });

    csvParser.on('error', (error) => {
      logger.error('CSV parsing error', { error: error.message });
      // Continue processing - some rows may have issues
    });

    // Pipe GCS stream through byte counter and CSV parser
    const pipeline = readStream.pipe(byteCounter).pipe(csvParser);

    // Yield each parsed row
    for await (const row of pipeline) {
      processed++;

      // Report progress every 1000 rows
      if (onProgress && processed % 1000 === 0) {
        onProgress({
          processed,
          bytesRead,
          currentTitle: row.title,
        });
      }

      // Check limit
      if (limit && processed > limit) {
        logger.info('Reached row limit', { limit, processed });
        break;
      }

      yield row as TMDBMovieRow;
    }

    logger.info('Completed GCS stream', {
      totalProcessed: processed,
      totalBytesRead: bytesRead,
    });
  }

  /**
   * Read rows in batches for parallel processing
   *
   * @param batchSize - Number of rows per batch
   * @param onProgress - Progress callback
   * @param limit - Optional limit on total rows
   * @yields Batches of TMDBMovieRow objects
   */
  async *streamBatches(
    batchSize: number = 100,
    onProgress?: ProgressCallback,
    limit?: number
  ): AsyncGenerator<TMDBMovieRow[], void, unknown> {
    let batch: TMDBMovieRow[] = [];
    let totalProcessed = 0;

    for await (const row of this.streamRows(undefined, limit)) {
      batch.push(row);
      totalProcessed++;

      if (batch.length >= batchSize) {
        if (onProgress) {
          onProgress({
            processed: totalProcessed,
            bytesRead: 0, // Not tracked in batch mode
            currentTitle: batch[batch.length - 1]?.title,
          });
        }

        yield batch;
        batch = [];
      }
    }

    // Yield remaining rows
    if (batch.length > 0) {
      yield batch;
    }
  }

  /**
   * Count total rows in dataset (by streaming headers)
   * Note: This is slow for large files - use metadata instead
   */
  async estimateRowCount(): Promise<number> {
    const metadata = await this.getFileMetadata();
    // Estimate: ~450 bytes per row average in TMDB dataset
    return Math.floor(metadata.size / 450);
  }
}

/**
 * Singleton instance
 */
let gcsReaderInstance: GCSReader | null = null;

export function getGCSReader(config?: Partial<GCSReaderConfig>): GCSReader {
  if (!gcsReaderInstance) {
    gcsReaderInstance = new GCSReader(config);
  }
  return gcsReaderInstance;
}
