-- RuVector PostgreSQL Optimized Benchmark Suite
-- Optimized for performance with SQL-native operations and parallel execution

-- ============================================================================
-- SETUP
-- ============================================================================

DROP SCHEMA IF EXISTS ruvector_bench CASCADE;
CREATE SCHEMA ruvector_bench;
SET search_path TO ruvector_bench, public;

-- Performance settings
SET max_parallel_workers_per_gather = 4;
SET parallel_tuple_cost = 0.001;
SET parallel_setup_cost = 100;
SET work_mem = '256MB';
SET maintenance_work_mem = '512MB';
SET jit = on;
SET jit_above_cost = 10000;

\timing on

-- ============================================================================
-- 1. OPTIMIZED HYPERBOLIC EMBEDDINGS (SQL-native, vectorized)
-- ============================================================================

-- Fast array dot product using SQL
CREATE OR REPLACE FUNCTION arr_dot(a float8[], b float8[])
RETURNS float8 AS $$
    SELECT COALESCE(sum(a[i] * b[i]), 0)
    FROM generate_subscripts(a, 1) i;
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- Fast array norm squared
CREATE OR REPLACE FUNCTION arr_norm_sq(a float8[])
RETURNS float8 AS $$
    SELECT COALESCE(sum(x * x), 0) FROM unnest(a) x;
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- Fast array difference norm squared
CREATE OR REPLACE FUNCTION arr_diff_norm_sq(a float8[], b float8[])
RETURNS float8 AS $$
    SELECT COALESCE(sum((a[i] - b[i])^2), 0)
    FROM generate_subscripts(a, 1) i;
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- Optimized Poincaré distance (SQL-native)
CREATE OR REPLACE FUNCTION poincare_distance(a float8[], b float8[], curvature float8 DEFAULT -1.0)
RETURNS float8 AS $$
    WITH norms AS (
        SELECT
            abs(curvature) AS c,
            arr_norm_sq(a) AS a_sq,
            arr_norm_sq(b) AS b_sq,
            arr_diff_norm_sq(a, b) AS diff_sq
    )
    SELECT CASE
        WHEN (1.0 - c * a_sq) * (1.0 - c * b_sq) <= 0 THEN 'infinity'::float8
        ELSE (1.0 / sqrt(c)) * asinh(sqrt(2.0 * c * diff_sq / ((1.0 - c * a_sq) * (1.0 - c * b_sq))))
    END
    FROM norms;
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- Optimized Lorentz distance
CREATE OR REPLACE FUNCTION lorentz_distance(a float8[], b float8[])
RETURNS float8 AS $$
    WITH minkowski AS (
        SELECT -a[1]*b[1] + COALESCE(sum(a[i]*b[i]), 0) AS prod
        FROM generate_subscripts(a, 1) i WHERE i > 1
    )
    SELECT CASE WHEN prod >= -1 THEN 0 ELSE acosh(-prod) END FROM minkowski;
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- Optimized Möbius addition
CREATE OR REPLACE FUNCTION mobius_add(a float8[], b float8[], curvature float8 DEFAULT -1.0)
RETURNS float8[] AS $$
    WITH params AS (
        SELECT
            abs(curvature) AS c,
            arr_norm_sq(a) AS a_sq,
            arr_norm_sq(b) AS b_sq,
            arr_dot(a, b) AS ab
    ),
    coeffs AS (
        SELECT
            c,
            1.0 + 2.0*c*ab + c*b_sq AS num_a,
            1.0 - c*a_sq AS num_b,
            1.0 + 2.0*c*ab + c*c*a_sq*b_sq AS denom
        FROM params
    )
    SELECT array_agg((num_a * a[i] + num_b * b[i]) / denom ORDER BY i)
    FROM coeffs, generate_subscripts(a, 1) i;
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- Optimized exponential map
CREATE OR REPLACE FUNCTION exp_map(x float8[], v float8[], curvature float8 DEFAULT -1.0)
RETURNS float8[] AS $$
    WITH params AS (
        SELECT
            abs(curvature) AS c,
            sqrt(arr_norm_sq(v)) AS v_norm,
            arr_norm_sq(x) AS x_sq
    ),
    scaled AS (
        SELECT
            CASE WHEN v_norm < 1e-10 THEN NULL
            ELSE tanh(sqrt(c) * (2.0 / (1.0 - c * x_sq)) * v_norm / 2.0) / (sqrt(c) * v_norm)
            END AS coef
        FROM params
    )
    SELECT CASE
        WHEN coef IS NULL THEN x
        ELSE mobius_add(x, (SELECT array_agg(v[i] * coef ORDER BY i) FROM generate_subscripts(v, 1) i), curvature)
    END
    FROM scaled;
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- ============================================================================
-- 2. OPTIMIZED GNN OPERATIONS (Set-based)
-- ============================================================================

-- GraphSAGE mean aggregation - optimized with single query
CREATE OR REPLACE FUNCTION graphsage_mean(node_features float8[], neighbor_features float8[][])
RETURNS float8[] AS $$
    WITH dims AS (
        SELECT generate_subscripts(node_features, 1) AS i
    ),
    agg AS (
        SELECT i, COALESCE(avg(neighbor_features[j][i]), 0) AS mean_val
        FROM dims, generate_subscripts(neighbor_features, 1) j
        GROUP BY i
    )
    SELECT node_features || array_agg(mean_val ORDER BY i)
    FROM agg;
$$ LANGUAGE SQL IMMUTABLE PARALLEL SAFE;

-- ============================================================================
-- 3. OPTIMIZED ATTENTION (Vectorized softmax)
-- ============================================================================

-- Fast softmax for arrays
CREATE OR REPLACE FUNCTION arr_softmax(scores float8[])
RETURNS float8[] AS $$
    WITH max_s AS (SELECT max(x) AS m FROM unnest(scores) x),
    exp_s AS (SELECT exp(x - m) AS e FROM unnest(scores) x, max_s),
    sum_e AS (SELECT sum(e) AS s FROM exp_s)
    SELECT array_agg(e / s) FROM exp_s, sum_e;
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- Optimized scaled dot attention
CREATE OR REPLACE FUNCTION scaled_dot_attention(query float8[], keys float8[][], vals float8[][])
RETURNS float8[] AS $$
    WITH params AS (
        SELECT 1.0 / sqrt(array_length(query, 1)) AS scale
    ),
    scores AS (
        SELECT i, sum(query[j] * keys[i][j]) * scale AS score
        FROM params, generate_subscripts(keys, 1) i, generate_subscripts(query, 1) j
        GROUP BY i, scale
    ),
    max_s AS (SELECT max(score) AS m FROM scores),
    softmax AS (
        SELECT i, exp(score - m) / sum(exp(score - m)) OVER () AS weight
        FROM scores, max_s
    ),
    output AS (
        SELECT j, sum(weight * vals[i][j]) AS val
        FROM softmax, generate_subscripts(vals, 2) j
        GROUP BY j
    )
    SELECT array_agg(val ORDER BY j) FROM output;
$$ LANGUAGE SQL IMMUTABLE PARALLEL SAFE;

-- ============================================================================
-- 4. OPTIMIZED SPARSE VECTORS (Indexed operations)
-- ============================================================================

DO $$ BEGIN
    CREATE TYPE sparse_vector AS (indices int[], values float8[], dim int);
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- Optimized sparse creation using array operations
CREATE OR REPLACE FUNCTION to_sparse(dense float8[], threshold float8 DEFAULT 0.0)
RETURNS sparse_vector AS $$
    SELECT ROW(
        array_agg(i ORDER BY i),
        array_agg(dense[i] ORDER BY i),
        array_length(dense, 1)
    )::sparse_vector
    FROM generate_subscripts(dense, 1) i
    WHERE abs(dense[i]) > threshold;
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- Optimized sparse dot product using hash join
CREATE OR REPLACE FUNCTION sparse_dot(a sparse_vector, b sparse_vector)
RETURNS float8 AS $$
    SELECT COALESCE(sum(av.v * bv.v), 0)
    FROM (SELECT unnest(a.indices) AS i, unnest(a.values) AS v) av
    JOIN (SELECT unnest(b.indices) AS i, unnest(b.values) AS v) bv ON av.i = bv.i;
$$ LANGUAGE SQL IMMUTABLE PARALLEL SAFE;

-- Optimized BM25 using tsvector
CREATE OR REPLACE FUNCTION bm25_score(
    query_terms text[],
    document text,
    avg_doc_length float8 DEFAULT 100.0,
    total_docs int DEFAULT 1000,
    k1 float8 DEFAULT 1.2,
    b float8 DEFAULT 0.75
)
RETURNS float8 AS $$
    WITH doc_tokens AS (
        SELECT lower(word) AS term, count(*) AS tf
        FROM regexp_split_to_table(lower(document), '\s+') word
        GROUP BY lower(word)
    ),
    doc_len AS (
        SELECT count(*) AS len FROM regexp_split_to_table(document, '\s+')
    ),
    query AS (
        SELECT lower(unnest(query_terms)) AS term
    ),
    matches AS (
        SELECT
            dt.tf,
            ln((total_docs - total_docs/10 + 0.5) / (total_docs/10 + 0.5) + 1) AS idf,
            dl.len AS doc_length
        FROM query q
        JOIN doc_tokens dt ON q.term = dt.term
        CROSS JOIN doc_len dl
    )
    SELECT COALESCE(sum(
        idf * (tf * (k1 + 1)) / (tf + k1 * (1 - b + b * doc_length / avg_doc_length))
    ), 0)
    FROM matches;
$$ LANGUAGE SQL IMMUTABLE PARALLEL SAFE;

-- ============================================================================
-- 5. OPTIMIZED QUANTIZATION (Batch operations)
-- ============================================================================

-- Binary quantization using bit aggregation
CREATE OR REPLACE FUNCTION binary_quantize(vec float8[])
RETURNS bit varying AS $$
    SELECT string_agg(CASE WHEN x >= 0 THEN '1' ELSE '0' END, '' ORDER BY i)::bit varying
    FROM unnest(vec) WITH ORDINALITY AS t(x, i);
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- Scalar quantization (vectorized)
CREATE OR REPLACE FUNCTION scalar_quantize(vec float8[], min_val float8 DEFAULT -1.0, max_val float8 DEFAULT 1.0)
RETURNS int[] AS $$
    SELECT array_agg(
        floor(greatest(0, least(1, (x - min_val) / (max_val - min_val))) * 255)::int
        ORDER BY i
    )
    FROM unnest(vec) WITH ORDINALITY AS t(x, i);
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- Optimized Hamming distance using XOR
CREATE OR REPLACE FUNCTION hamming_distance(a bit varying, b bit varying)
RETURNS int AS $$
    SELECT bit_count(a # b);
$$ LANGUAGE SQL IMMUTABLE STRICT PARALLEL SAFE;

-- ============================================================================
-- 6. FAST DATA GENERATION
-- ============================================================================

-- Optimized random vector using generate_series
CREATE OR REPLACE FUNCTION random_vector(dim int)
RETURNS float8[] AS $$
    SELECT array_agg(random() * 2 - 1) FROM generate_series(1, dim);
$$ LANGUAGE SQL VOLATILE PARALLEL SAFE;

-- Optimized random unit vector
CREATE OR REPLACE FUNCTION random_unit_vector(dim int, max_norm float8 DEFAULT 0.9)
RETURNS float8[] AS $$
    WITH raw AS (SELECT array_agg(random() * 2 - 1) AS v FROM generate_series(1, dim)),
    norm AS (SELECT sqrt(arr_norm_sq(v)) AS n, v FROM raw),
    scaled AS (SELECT (random() * max_norm) / n AS scale, v FROM norm)
    SELECT array_agg(v[i] * scale ORDER BY i) FROM scaled, generate_subscripts(v, 1) i;
$$ LANGUAGE SQL VOLATILE PARALLEL SAFE;

-- ============================================================================
-- 7. BENCHMARK TABLES WITH OPTIMIZED STORAGE
-- ============================================================================

CREATE TABLE benchmark_vectors (
    id serial PRIMARY KEY,
    embedding vector(384),
    embedding_arr float8[]
) WITH (fillfactor = 90);

CREATE TABLE benchmark_hyperbolic (
    id serial PRIMARY KEY,
    poincare_embedding float8[],
    hierarchy_level smallint
) WITH (fillfactor = 90);

CREATE TABLE benchmark_graph_nodes (
    id serial PRIMARY KEY,
    features float8[],
    node_type text
) WITH (fillfactor = 90);

CREATE TABLE benchmark_graph_edges (
    source_id int NOT NULL,
    target_id int NOT NULL,
    weight float4 DEFAULT 1.0,
    PRIMARY KEY (source_id, target_id)
) WITH (fillfactor = 90);

CREATE TABLE benchmark_documents (
    id serial PRIMARY KEY,
    content text,
    title text,
    tsv tsvector GENERATED ALWAYS AS (to_tsvector('english', content)) STORED
) WITH (fillfactor = 90);

CREATE TABLE benchmark_sparse (
    id serial PRIMARY KEY,
    sparse_vec sparse_vector
) WITH (fillfactor = 90);

-- ============================================================================
-- 8. OPTIMIZED DATA POPULATION (Batch inserts)
-- ============================================================================

\echo 'Populating benchmark data...'

-- Batch insert vectors (faster than row-by-row)
INSERT INTO benchmark_vectors (embedding_arr)
SELECT random_vector(384) FROM generate_series(1, 10000);

-- Parallel vector conversion
UPDATE benchmark_vectors SET embedding = embedding_arr::vector;

-- Create HNSW index early for faster subsequent operations
CREATE INDEX CONCURRENTLY benchmark_vectors_hnsw_idx
ON benchmark_vectors USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 100);

-- Insert hyperbolic embeddings
INSERT INTO benchmark_hyperbolic (poincare_embedding, hierarchy_level)
SELECT random_unit_vector(32, 0.95), (random() * 5)::smallint
FROM generate_series(1, 5000);

-- Insert graph data
INSERT INTO benchmark_graph_nodes (features, node_type)
SELECT random_vector(64),
    CASE (i % 3) WHEN 0 THEN 'user' WHEN 1 THEN 'item' ELSE 'tag' END
FROM generate_series(1, 1000) i;

-- Faster edge insertion with ON CONFLICT
INSERT INTO benchmark_graph_edges (source_id, target_id, weight)
SELECT DISTINCT ON (s, t)
    (random() * 999 + 1)::int AS s,
    (random() * 999 + 1)::int AS t,
    random()::float4
FROM generate_series(1, 6000)
WHERE (random() * 999 + 1)::int != (random() * 999 + 1)::int
ON CONFLICT DO NOTHING;

-- Index for graph traversal
CREATE INDEX benchmark_graph_edges_source_idx ON benchmark_graph_edges(source_id);
CREATE INDEX benchmark_graph_edges_target_idx ON benchmark_graph_edges(target_id);

-- Insert documents with full-text search support
INSERT INTO benchmark_documents (content, title)
SELECT
    'Document ' || i || ' covers machine learning vectors embeddings neural networks ' ||
    CASE (i % 5) WHEN 0 THEN 'attention transformers' WHEN 1 THEN 'graph networks'
    WHEN 2 THEN 'hyperbolic geometry' WHEN 3 THEN 'sparse retrieval' ELSE 'quantization' END,
    'Document ' || i
FROM generate_series(1, 1000) i;

-- GIN index for full-text search
CREATE INDEX benchmark_documents_tsv_idx ON benchmark_documents USING gin(tsv);

-- Insert sparse vectors
INSERT INTO benchmark_sparse (sparse_vec)
SELECT to_sparse(random_vector(1000), 0.8)
FROM generate_series(1, 2000);

-- Analyze tables for query optimization
ANALYZE;

\echo ''
\echo '=============================================='
\echo 'RuVector Optimized Benchmark Suite Ready'
\echo '=============================================='
\echo ''
