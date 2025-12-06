/**
 * Vertex AI Embeddings Service
 *
 * Generates semantic embeddings using Google's text-embedding-004 model
 * for content discovery and similarity matching.
 */

import { PredictionServiceClient } from '@google-cloud/aiplatform';
import { google } from '@google-cloud/aiplatform/build/protos/protos';
import winston from 'winston';

// Logger configuration
const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.json(),
  transports: [
    new winston.transports.Console({
      format: winston.format.combine(
        winston.format.colorize(),
        winston.format.simple()
      )
    })
  ]
});

/**
 * Configuration for Vertex AI Embeddings
 */
export interface EmbeddingsConfig {
  projectId: string;
  location: string;
  model: string;
  maxRetries: number;
  retryDelayMs: number;
  batchSize: number;
}

/**
 * Default configuration
 */
const DEFAULT_CONFIG: EmbeddingsConfig = {
  projectId: process.env.GOOGLE_CLOUD_PROJECT || 'agentics-foundation25lon-1899',
  location: 'us-central1',
  model: 'text-embedding-004',
  maxRetries: 3,
  retryDelayMs: 1000,
  batchSize: 5 // Vertex AI recommends batch size of 5 for embeddings
};

/**
 * Embedding generation result
 */
export interface EmbeddingResult {
  embedding: number[];
  text: string;
  dimensions: number;
  modelVersion: string;
}

/**
 * Batch embedding result
 */
export interface BatchEmbeddingResult {
  embeddings: number[][];
  texts: string[];
  dimensions: number;
  modelVersion: string;
  totalTokens?: number;
}

/**
 * VertexAIEmbeddings Class
 *
 * Handles embedding generation with retry logic, batching,
 * and comprehensive error handling.
 */
export class VertexAIEmbeddings {
  private client: PredictionServiceClient;
  private config: EmbeddingsConfig;
  private endpoint: string;

  constructor(config?: Partial<EmbeddingsConfig>) {
    this.config = { ...DEFAULT_CONFIG, ...config };

    // Initialize Vertex AI client
    this.client = new PredictionServiceClient({
      apiEndpoint: `${this.config.location}-aiplatform.googleapis.com`
    });

    // Construct model endpoint
    this.endpoint = `projects/${this.config.projectId}/locations/${this.config.location}/publishers/google/models/${this.config.model}`;

    logger.info('VertexAI Embeddings initialized', {
      projectId: this.config.projectId,
      location: this.config.location,
      model: this.config.model
    });
  }

  /**
   * Generate embedding for a single text
   *
   * @param text - Input text to embed
   * @returns Promise resolving to embedding vector
   */
  async generateEmbedding(text: string): Promise<number[]> {
    if (!text || text.trim().length === 0) {
      throw new Error('Input text cannot be empty');
    }

    const startTime = Date.now();

    try {
      const result = await this.generateEmbeddingWithRetry(text);

      const latency = Date.now() - startTime;
      logger.info('Embedding generated', {
        textLength: text.length,
        dimensions: result.length,
        latencyMs: latency
      });

      return result;
    } catch (error) {
      logger.error('Failed to generate embedding', {
        error: error instanceof Error ? error.message : 'Unknown error',
        textLength: text.length
      });
      throw error;
    }
  }

  /**
   * Generate embeddings for multiple texts in batches
   *
   * @param texts - Array of input texts
   * @returns Promise resolving to array of embedding vectors
   */
  async generateBatchEmbeddings(texts: string[]): Promise<number[][]> {
    if (!texts || texts.length === 0) {
      throw new Error('Input texts array cannot be empty');
    }

    const startTime = Date.now();
    const allEmbeddings: number[][] = [];

    try {
      // Process in batches to respect API limits
      for (let i = 0; i < texts.length; i += this.config.batchSize) {
        const batch = texts.slice(i, i + this.config.batchSize);
        const batchEmbeddings = await this.processBatch(batch);
        allEmbeddings.push(...batchEmbeddings);

        logger.debug('Batch processed', {
          batchNumber: Math.floor(i / this.config.batchSize) + 1,
          batchSize: batch.length,
          totalProcessed: allEmbeddings.length
        });
      }

      const latency = Date.now() - startTime;
      logger.info('Batch embeddings generated', {
        totalTexts: texts.length,
        batches: Math.ceil(texts.length / this.config.batchSize),
        latencyMs: latency,
        avgLatencyPerText: Math.round(latency / texts.length)
      });

      return allEmbeddings;
    } catch (error) {
      logger.error('Failed to generate batch embeddings', {
        error: error instanceof Error ? error.message : 'Unknown error',
        totalTexts: texts.length
      });
      throw error;
    }
  }

  /**
   * Process a batch of texts (internal method)
   *
   * @param texts - Batch of texts to process
   * @returns Array of embeddings
   */
  private async processBatch(texts: string[]): Promise<number[][]> {
    const embeddings: number[][] = [];

    // Process each text in the batch
    for (const text of texts) {
      if (!text || text.trim().length === 0) {
        logger.warn('Skipping empty text in batch');
        embeddings.push([]); // Placeholder for empty text
        continue;
      }

      try {
        const embedding = await this.generateEmbeddingWithRetry(text);
        embeddings.push(embedding);
      } catch (error) {
        logger.error('Failed to process text in batch', {
          error: error instanceof Error ? error.message : 'Unknown error',
          textPreview: text.substring(0, 50)
        });
        throw error;
      }
    }

    return embeddings;
  }

  /**
   * Generate embedding with exponential backoff retry logic
   *
   * @param text - Text to embed
   * @param attempt - Current retry attempt
   * @returns Embedding vector
   */
  private async generateEmbeddingWithRetry(
    text: string,
    attempt: number = 0
  ): Promise<number[]> {
    try {
      // Prepare the prediction request
      const instance = {
        content: text,
        task_type: 'SEMANTIC_SIMILARITY'
      };

      const instanceValue = this.client.helpers.toValue(instance);

      const request = {
        endpoint: this.endpoint,
        instances: [instanceValue]
      };

      // Call Vertex AI Prediction API
      const [response] = await this.client.predict(request);

      // Extract embedding from response
      if (!response.predictions || response.predictions.length === 0) {
        throw new Error('No predictions returned from Vertex AI');
      }

      const prediction = response.predictions[0];
      const embedding = this.extractEmbedding(prediction);

      if (!embedding || embedding.length === 0) {
        throw new Error('Invalid embedding vector received');
      }

      return embedding;

    } catch (error) {
      const isRetryable = this.isRetryableError(error);

      if (isRetryable && attempt < this.config.maxRetries) {
        const delay = this.config.retryDelayMs * Math.pow(2, attempt);

        logger.warn('Retrying embedding generation', {
          attempt: attempt + 1,
          maxRetries: this.config.maxRetries,
          delayMs: delay,
          error: error instanceof Error ? error.message : 'Unknown error'
        });

        await this.sleep(delay);
        return this.generateEmbeddingWithRetry(text, attempt + 1);
      }

      throw error;
    }
  }

  /**
   * Extract embedding vector from prediction response
   *
   * @param prediction - Vertex AI prediction object
   * @returns Embedding vector
   */
  private extractEmbedding(prediction: any): number[] {
    // Handle different response formats
    if (prediction.structValue?.fields?.embeddings?.structValue?.fields?.values?.listValue?.values) {
      return prediction.structValue.fields.embeddings.structValue.fields.values.listValue.values
        .map((v: any) => v.numberValue);
    }

    if (prediction.embeddings?.values) {
      return prediction.embeddings.values;
    }

    if (Array.isArray(prediction)) {
      return prediction;
    }

    throw new Error('Unable to extract embedding from prediction response');
  }

  /**
   * Check if error is retryable
   *
   * @param error - Error object
   * @returns True if error is retryable
   */
  private isRetryableError(error: any): boolean {
    if (!error) return false;

    // Network errors
    if (error.code === 'ECONNRESET' || error.code === 'ETIMEDOUT') {
      return true;
    }

    // Rate limiting and service unavailable
    if (error.code === 429 || error.code === 503 || error.code === 504) {
      return true;
    }

    // gRPC errors
    if (error.code === 14 || error.code === 4) { // UNAVAILABLE or DEADLINE_EXCEEDED
      return true;
    }

    return false;
  }

  /**
   * Sleep utility for retry delays
   *
   * @param ms - Milliseconds to sleep
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Get embedding dimensions for the current model
   *
   * @returns Number of dimensions
   */
  getDimensions(): number {
    // text-embedding-004 produces 768-dimensional embeddings
    return 768;
  }

  /**
   * Get model information
   *
   * @returns Model configuration
   */
  getModelInfo() {
    return {
      model: this.config.model,
      dimensions: this.getDimensions(),
      projectId: this.config.projectId,
      location: this.config.location
    };
  }

  /**
   * Close the client connection
   */
  async close(): Promise<void> {
    await this.client.close();
    logger.info('VertexAI Embeddings client closed');
  }
}

/**
 * Singleton instance for shared usage
 */
let embeddingsInstance: VertexAIEmbeddings | null = null;

/**
 * Get or create singleton embeddings instance
 *
 * @param config - Optional configuration
 * @returns VertexAIEmbeddings instance
 */
export function getEmbeddingsInstance(config?: Partial<EmbeddingsConfig>): VertexAIEmbeddings {
  if (!embeddingsInstance) {
    embeddingsInstance = new VertexAIEmbeddings(config);
  }
  return embeddingsInstance;
}
