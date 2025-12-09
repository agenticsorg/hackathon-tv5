/**
 * Knowledge Graph Module
 *
 * Exports all knowledge graph components for the Nexus-UMMID platform.
 */

// Schema types
export * from './schema';

// GCS Reader
export { GCSReader, getGCSReader, type GCSReaderConfig, type ProgressCallback } from './gcs-reader';

// Processor
export {
  KnowledgeGraphProcessor,
  getProcessor,
  type ProcessedMovie,
  type ProcessingStats,
} from './processor';

// Store
export { KnowledgeGraphStore, getStore, type StoreConfig } from './store';

// Ingestion Pipeline
export {
  IngestionPipeline,
  createIngestionPipeline,
  type IngestionConfig,
  type IngestionProgress,
  type IngestionResult,
} from './ingestion-pipeline';
