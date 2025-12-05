/**
 * AgentDB - Main database wrapper class
 * 
 * Provides a unified interface to all AgentDB controllers
 */
import Database from 'better-sqlite3';
import { ReflexionMemory } from '../controllers/ReflexionMemory.js';
import { SkillLibrary } from '../controllers/SkillLibrary.js';
import { CausalMemoryGraph } from '../controllers/CausalMemoryGraph.js';
import { EmbeddingService } from '../controllers/EmbeddingService.js';
import { createBackend } from '../backends/factory.js';
import type { VectorBackend } from '../backends/VectorBackend.js';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export interface AgentDBConfig {
  dbPath?: string;
  namespace?: string;
  enableAttention?: boolean;
  attentionConfig?: Record<string, any>;
}

export class AgentDB {
  private db: Database.Database;
  private reflexion!: ReflexionMemory;
  private skills!: SkillLibrary;
  private causalGraph!: CausalMemoryGraph;
  private embedder!: EmbeddingService;
  public vectorBackend!: VectorBackend;
  private initialized = false;
  private config: AgentDBConfig;

  constructor(config: AgentDBConfig = {}) {
    this.config = config;
    const dbPath = config.dbPath || ':memory:';
    this.db = new Database(dbPath);
    this.db.pragma('journal_mode = WAL');
  }

  async initialize(): Promise<void> {
    if (this.initialized) return;

    // Load schemas
    // When compiled, this file is at dist/src/core/AgentDB.js
    // Schemas are at dist/schemas/, so we need ../../schemas/
    const schemaPath = path.join(__dirname, '../../schemas/schema.sql');
    if (fs.existsSync(schemaPath)) {
      const schema = fs.readFileSync(schemaPath, 'utf-8');
      this.db.exec(schema);
    }

    const frontierSchemaPath = path.join(__dirname, '../../schemas/frontier-schema.sql');
    if (fs.existsSync(frontierSchemaPath)) {
      const frontierSchema = fs.readFileSync(frontierSchemaPath, 'utf-8');
      this.db.exec(frontierSchema);
    }

    // Initialize embedder with default Xenova model
    // Falls back to mock embeddings if @xenova/transformers is not available
    this.embedder = new EmbeddingService({
      model: 'Xenova/all-MiniLM-L6-v2',
      dimension: 384,
      provider: 'transformers'
    });
    await this.embedder.initialize();

    // Initialize vector backend
    this.vectorBackend = await createBackend('auto', {
      dimensions: 384,
      metric: 'cosine'
    });

    // Initialize controllers
    this.reflexion = new ReflexionMemory(this.db, this.embedder);
    this.skills = new SkillLibrary(this.db, this.embedder);
    this.causalGraph = new CausalMemoryGraph(this.db);

    this.initialized = true;
  }

  getController(name: string): any {
    if (!this.initialized) {
      throw new Error('AgentDB not initialized. Call initialize() first.');
    }

    switch (name) {
      case 'memory':
      case 'reflexion':
        return this.reflexion;
      case 'skills':
        return this.skills;
      case 'causal':
      case 'causalGraph':
        return this.causalGraph;
      default:
        throw new Error(`Unknown controller: ${name}`);
    }
  }

  async close(): Promise<void> {
    if (this.db) {
      this.db.close();
    }
  }

  // Expose database for advanced usage
  get database(): Database.Database {
    return this.db;
  }
}
