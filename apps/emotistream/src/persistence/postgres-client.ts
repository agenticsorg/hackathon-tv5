/**
 * PostgreSQL Client for EmotiStream
 *
 * Provides database connectivity and query utilities.
 */

import pg from 'pg';
import fs from 'fs';
import path from 'path';

const { Pool } = pg;

// Database configuration
// Supports both DATABASE_URL (cloud providers) and individual vars (docker-compose)
const getDatabaseConfig = () => {
  // If DATABASE_URL is provided (Neon, Supabase, Railway, etc.)
  if (process.env.DATABASE_URL) {
    return {
      connectionString: process.env.DATABASE_URL,
      ssl: process.env.DB_SSL === 'false' ? false : { rejectUnauthorized: false },
      max: 10, // Lower pool size for serverless
      idleTimeoutMillis: 10000,
      connectionTimeoutMillis: 10000,
    };
  }

  // Individual environment variables (docker-compose, Cloud SQL)
  return {
    host: process.env.DB_HOST || 'localhost',
    port: parseInt(process.env.DB_PORT || '5432'),
    database: process.env.DB_NAME || 'emotistream',
    user: process.env.DB_USER || 'emotistream',
    password: process.env.DB_PASSWORD || 'emotistream_pass',
    ssl: process.env.DB_SSL === 'true' ? { rejectUnauthorized: false } : false,
    max: 20, // Maximum pool size
    idleTimeoutMillis: 30000,
    connectionTimeoutMillis: 5000,
  };
};

const dbConfig = getDatabaseConfig();

let pool: pg.Pool | null = null;

/**
 * Get the database pool (singleton)
 */
export function getPool(): pg.Pool {
  if (!pool) {
    pool = new Pool(dbConfig);

    pool.on('error', (err) => {
      console.error('Unexpected database pool error:', err);
    });

    pool.on('connect', () => {
      console.log('New database connection established');
    });
  }
  return pool;
}

/**
 * Execute a query with parameters
 */
export async function query<T = any>(
  text: string,
  params?: any[]
): Promise<pg.QueryResult<T>> {
  const pool = getPool();
  const start = Date.now();

  try {
    const result = await pool.query<T>(text, params);
    const duration = Date.now() - start;

    if (duration > 100) {
      console.log('Slow query detected:', { text: text.substring(0, 100), duration, rows: result.rowCount });
    }

    return result;
  } catch (error) {
    console.error('Database query error:', { text: text.substring(0, 100), error });
    throw error;
  }
}

/**
 * Execute a query and return the first row
 */
export async function queryOne<T = any>(
  text: string,
  params?: any[]
): Promise<T | null> {
  const result = await query<T>(text, params);
  return result.rows[0] || null;
}

/**
 * Execute a query and return all rows
 */
export async function queryAll<T = any>(
  text: string,
  params?: any[]
): Promise<T[]> {
  const result = await query<T>(text, params);
  return result.rows;
}

/**
 * Initialize database schema
 * Supports both ruvector (local/docker) and pgvector (cloud providers)
 */
export async function initializeDatabase(): Promise<void> {
  console.log('Initializing database schema...');

  // Check if we should use pgvector schema (for cloud providers like Neon, Supabase)
  const usePgVector = process.env.USE_PGVECTOR === 'true' || process.env.DATABASE_URL;

  // Determine schema file location (works in both dev and production)
  const schemaFileName = usePgVector ? 'schema-pgvector.sql' : 'schema.sql';
  const possiblePaths = [
    path.join(process.cwd(), 'dist', 'persistence', schemaFileName),
    path.join(process.cwd(), 'src', 'persistence', schemaFileName),
    // Fallback to standard schema if pgvector schema not found
    path.join(process.cwd(), 'dist', 'persistence', 'schema.sql'),
    path.join(process.cwd(), 'src', 'persistence', 'schema.sql'),
  ];

  let schemaPath = '';
  for (const p of possiblePaths) {
    if (fs.existsSync(p)) {
      schemaPath = p;
      console.log(`Using schema: ${p}`);
      break;
    }
  }

  if (!schemaPath) {
    console.error('Schema file not found in any of:', possiblePaths);
    throw new Error('Database schema file not found');
  }

  const schema = fs.readFileSync(schemaPath, 'utf-8');

  try {
    await query(schema);
    console.log('Database schema initialized successfully');
  } catch (error) {
    console.error('Failed to initialize database schema:', error);
    throw error;
  }
}

/**
 * Check database connectivity
 */
export async function checkConnection(): Promise<boolean> {
  try {
    const result = await query('SELECT NOW() as time');
    console.log('Database connected at:', result.rows[0].time);
    return true;
  } catch (error) {
    console.error('Database connection check failed:', error);
    return false;
  }
}

/**
 * Close database pool
 */
export async function closePool(): Promise<void> {
  if (pool) {
    await pool.end();
    pool = null;
    console.log('Database pool closed');
  }
}

/**
 * Transaction helper
 */
export async function withTransaction<T>(
  callback: (client: pg.PoolClient) => Promise<T>
): Promise<T> {
  const pool = getPool();
  const client = await pool.connect();

  try {
    await client.query('BEGIN');
    const result = await callback(client);
    await client.query('COMMIT');
    return result;
  } catch (error) {
    await client.query('ROLLBACK');
    throw error;
  } finally {
    client.release();
  }
}

export default {
  getPool,
  query,
  queryOne,
  queryAll,
  initializeDatabase,
  checkConnection,
  closePool,
  withTransaction,
};
