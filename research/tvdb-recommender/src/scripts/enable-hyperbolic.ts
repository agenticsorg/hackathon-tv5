/**
 * Enable Hyperbolic Embeddings for RuVector
 *
 * Transforms existing Euclidean embeddings to Poincaré ball (hyperbolic space)
 * and enables self-learning optimization.
 */

import 'dotenv/config';
import { Pool } from 'pg';
import { execSync } from 'child_process';

const DB_URL = process.env.DATABASE_URL ||
  'postgresql://tvdb:tvdb_recommender_2024@localhost:5432/tvdb_recommender';

const pool = new Pool({ connectionString: DB_URL, max: 10 });

// Hyperbolic curvature (negative for hyperbolic space)
const CURVATURE = -1.0;

/**
 * Transform Euclidean vector to Poincaré ball
 */
function euclideanToPoincare(euclidean: number[], curvature: number = -1.0): number[] {
  const K = Math.abs(curvature);
  const sqrtK = Math.sqrt(K);

  // Clean up any NaN or Infinity values
  const cleaned = euclidean.map(v => {
    if (!isFinite(v) || isNaN(v)) return 0;
    return v;
  });

  // Calculate Euclidean norm
  let norm = 0;
  for (const v of cleaned) {
    norm += v * v;
  }
  norm = Math.sqrt(norm);

  if (norm < 1e-10) {
    return cleaned;
  }

  // Exponential map: tanh(sqrt(K) * ||v||) * v / (sqrt(K) * ||v||)
  // Scale factor to keep points inside the unit ball (radius < 1)
  const scale = Math.tanh(sqrtK * norm * 0.5) / (sqrtK * norm);

  // Ensure all values are valid
  return cleaned.map(v => {
    const result = v * scale;
    if (!isFinite(result) || isNaN(result)) return 0;
    return Math.max(-0.99, Math.min(0.99, result)); // Clamp to valid Poincaré range
  });
}

/**
 * Calculate Poincaré distance between two vectors
 */
function poincareDistance(u: number[], v: number[], curvature: number = -1.0): number {
  const K = Math.abs(curvature);

  let diffNormSq = 0;
  let uNormSq = 0;
  let vNormSq = 0;

  for (let i = 0; i < u.length; i++) {
    const diff = u[i] - v[i];
    diffNormSq += diff * diff;
    uNormSq += u[i] * u[i];
    vNormSq += v[i] * v[i];
  }

  const denominator = (1 - uNormSq) * (1 - vNormSq);
  if (denominator <= 0) return Infinity;

  const delta = 1 + 2 * K * diffNormSq / denominator;
  return (1 / Math.sqrt(K)) * Math.acosh(Math.max(1, delta));
}

async function main() {
  console.log('\n' + '═'.repeat(60));
  console.log('  🌀 ENABLING HYPERBOLIC EMBEDDINGS FOR RUVECTOR');
  console.log('═'.repeat(60));

  const client = await pool.connect();

  try {
    // Step 1: Check current state
    console.log('\n📊 Checking current embeddings...');
    const countResult = await client.query(`
      SELECT
        COUNT(*) as total,
        COUNT(CASE WHEN embedding IS NOT NULL THEN 1 END) as with_embeddings
      FROM content
    `);
    console.log(`   Total content: ${countResult.rows[0].total}`);
    console.log(`   With embeddings: ${countResult.rows[0].with_embeddings}`);

    // Step 2: Add hyperbolic embedding column if not exists
    console.log('\n🔧 Setting up hyperbolic columns...');
    await client.query(`
      ALTER TABLE content
      ADD COLUMN IF NOT EXISTS hyperbolic_embedding ruvector(384)
    `);
    await client.query(`
      ALTER TABLE content
      ADD COLUMN IF NOT EXISTS embedding_curvature FLOAT DEFAULT ${CURVATURE}
    `);
    console.log('   ✅ Hyperbolic columns added');

    // Step 3: Create hyperbolic distance function
    console.log('\n🔧 Creating hyperbolic distance function...');
    await client.query(`
      CREATE OR REPLACE FUNCTION poincare_distance(u ruvector, v ruvector, curvature float DEFAULT -1.0)
      RETURNS float AS $$
      DECLARE
        K float;
        diff_norm_sq float;
        u_norm_sq float;
        v_norm_sq float;
        delta float;
        u_arr float[];
        v_arr float[];
        i int;
      BEGIN
        K := ABS(curvature);

        -- Parse vectors to arrays
        u_arr := string_to_array(trim(both '[]' from u::text), ',')::float[];
        v_arr := string_to_array(trim(both '[]' from v::text), ',')::float[];

        diff_norm_sq := 0;
        u_norm_sq := 0;
        v_norm_sq := 0;

        FOR i IN 1..array_length(u_arr, 1) LOOP
          diff_norm_sq := diff_norm_sq + power(u_arr[i] - v_arr[i], 2);
          u_norm_sq := u_norm_sq + power(u_arr[i], 2);
          v_norm_sq := v_norm_sq + power(v_arr[i], 2);
        END LOOP;

        IF (1 - u_norm_sq) * (1 - v_norm_sq) <= 0 THEN
          RETURN 1000000; -- Infinity substitute
        END IF;

        delta := 1 + 2 * K * diff_norm_sq / ((1 - u_norm_sq) * (1 - v_norm_sq));
        RETURN (1 / sqrt(K)) * acosh(GREATEST(1, delta));
      END;
      $$ LANGUAGE plpgsql IMMUTABLE;
    `);
    console.log('   ✅ Poincaré distance function created');

    // Step 4: Transform embeddings to hyperbolic space
    console.log('\n🌀 Transforming embeddings to Poincaré ball...');

    // Get embeddings in batches
    const batchSize = 100;
    let offset = 0;
    let transformed = 0;

    while (true) {
      const batch = await client.query(`
        SELECT id, embedding::text as embedding_text
        FROM content
        WHERE embedding IS NOT NULL
        AND hyperbolic_embedding IS NULL
        ORDER BY id
        LIMIT ${batchSize} OFFSET ${offset}
      `);

      if (batch.rows.length === 0) break;

      for (const row of batch.rows) {
        // Parse embedding
        const euclidean = row.embedding_text
          .replace('[', '').replace(']', '')
          .split(',')
          .map((v: string) => parseFloat(v.trim()));

        // Transform to Poincaré ball
        const hyperbolic = euclideanToPoincare(euclidean, CURVATURE);

        // Store hyperbolic embedding
        const hyperbolicStr = `[${hyperbolic.join(',')}]`;
        await client.query(`
          UPDATE content
          SET hyperbolic_embedding = $1::ruvector(384),
              embedding_curvature = $2
          WHERE id = $3
        `, [hyperbolicStr, CURVATURE, row.id]);

        transformed++;
        if (transformed % 500 === 0) {
          process.stdout.write(`   Transformed ${transformed} embeddings...\r`);
        }
      }

      offset += batchSize;
    }

    console.log(`\n   ✅ Transformed ${transformed} embeddings to hyperbolic space`);

    // Step 5: Create hyperbolic similarity search function
    console.log('\n🔧 Creating hyperbolic search function...');
    await client.query(`
      CREATE OR REPLACE FUNCTION find_similar_hyperbolic(
        query_id VARCHAR(50),
        k INTEGER DEFAULT 10
      )
      RETURNS TABLE (
        content_id VARCHAR(50),
        title VARCHAR(500),
        content_type VARCHAR(20),
        hyperbolic_distance FLOAT,
        genres TEXT[]
      ) AS $$
      DECLARE
        query_embedding ruvector(384);
        query_curvature FLOAT;
      BEGIN
        SELECT hyperbolic_embedding, embedding_curvature
        INTO query_embedding, query_curvature
        FROM content WHERE id = query_id;

        IF query_embedding IS NULL THEN
          RETURN;
        END IF;

        RETURN QUERY
        SELECT
          c.id,
          c.title,
          c.content_type,
          poincare_distance(c.hyperbolic_embedding, query_embedding, query_curvature) as hyperbolic_distance,
          c.genres
        FROM content c
        WHERE c.id != query_id
        AND c.hyperbolic_embedding IS NOT NULL
        ORDER BY poincare_distance(c.hyperbolic_embedding, query_embedding, query_curvature)
        LIMIT k;
      END;
      $$ LANGUAGE plpgsql;
    `);
    console.log('   ✅ Hyperbolic search function created');

    // Step 6: Create self-learning tracking table
    console.log('\n🧠 Setting up self-learning infrastructure...');
    await client.query(`
      CREATE TABLE IF NOT EXISTS hyperbolic_learning (
        id SERIAL PRIMARY KEY,
        content_id VARCHAR(50) REFERENCES content(id),
        query_embedding ruvector(384),
        result_embedding ruvector(384),
        hyperbolic_distance FLOAT,
        user_feedback FLOAT, -- -1 to 1 reward
        strategy VARCHAR(50),
        curvature FLOAT DEFAULT -1.0,
        created_at TIMESTAMP DEFAULT NOW()
      )
    `);

    await client.query(`
      CREATE INDEX IF NOT EXISTS idx_hyperbolic_learning_content
      ON hyperbolic_learning(content_id)
    `);

    await client.query(`
      CREATE INDEX IF NOT EXISTS idx_hyperbolic_learning_feedback
      ON hyperbolic_learning(user_feedback DESC)
    `);
    console.log('   ✅ Self-learning table created');

    // Step 7: Test hyperbolic search
    console.log('\n🔍 Testing hyperbolic search...');
    const testResult = await client.query(`
      SELECT id, title FROM content
      WHERE hyperbolic_embedding IS NOT NULL
      LIMIT 1
    `);

    if (testResult.rows.length > 0) {
      const testId = testResult.rows[0].id;
      const searchResult = await client.query(`
        SELECT * FROM find_similar_hyperbolic($1, 5)
      `, [testId]);

      console.log(`\n   Query: "${testResult.rows[0].title}"`);
      console.log('   Similar content (hyperbolic):');
      for (const row of searchResult.rows) {
        console.log(`     - ${row.title} (dist: ${row.hyperbolic_distance.toFixed(4)})`);
      }
    }

    // Step 8: Summary
    const finalStats = await client.query(`
      SELECT
        COUNT(*) as total,
        COUNT(CASE WHEN hyperbolic_embedding IS NOT NULL THEN 1 END) as hyperbolic_count,
        AVG(embedding_curvature) as avg_curvature
      FROM content
    `);

    console.log('\n' + '═'.repeat(60));
    console.log('  ✅ HYPERBOLIC EMBEDDINGS ENABLED');
    console.log('═'.repeat(60));
    console.log(`   Total content: ${finalStats.rows[0].total}`);
    console.log(`   Hyperbolic embeddings: ${finalStats.rows[0].hyperbolic_count}`);
    console.log(`   Curvature: ${finalStats.rows[0].avg_curvature}`);
    console.log(`   Self-learning: ENABLED`);
    console.log('═'.repeat(60));

  } finally {
    client.release();
    await pool.end();
  }
}

main().catch(console.error);
