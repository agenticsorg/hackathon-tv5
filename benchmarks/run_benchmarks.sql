-- RuVector Benchmark Execution Script
-- Run after ruvector_benchmark.sql to execute performance tests

SET search_path TO ruvector_bench, public;

\echo ''
\echo '=============================================='
\echo 'BENCHMARK 1: Dense Vector Operations'
\echo '=============================================='

-- Cosine similarity search (pgvector baseline)
\echo ''
\echo '1.1 pgvector Cosine Search (10K vectors, top-10)'
EXPLAIN ANALYZE
SELECT id, embedding <=> (SELECT embedding FROM benchmark_vectors WHERE id = 1) AS distance
FROM benchmark_vectors
ORDER BY distance
LIMIT 10;

-- L2 distance search
\echo ''
\echo '1.2 pgvector L2 Search (10K vectors, top-10)'
EXPLAIN ANALYZE
SELECT id, embedding <-> (SELECT embedding FROM benchmark_vectors WHERE id = 1) AS distance
FROM benchmark_vectors
ORDER BY distance
LIMIT 10;

-- Inner product search
\echo ''
\echo '1.3 pgvector Inner Product Search (10K vectors, top-10)'
EXPLAIN ANALYZE
SELECT id, embedding <#> (SELECT embedding FROM benchmark_vectors WHERE id = 1) AS distance
FROM benchmark_vectors
ORDER BY distance
LIMIT 10;

\echo ''
\echo '=============================================='
\echo 'BENCHMARK 2: Hyperbolic Embeddings'
\echo '=============================================='

-- Poincaré distance calculation
\echo ''
\echo '2.1 Poincaré Distance (5K pairs)'
EXPLAIN ANALYZE
SELECT
    a.id AS id_a,
    b.id AS id_b,
    poincare_distance(a.poincare_embedding, b.poincare_embedding) AS distance
FROM
    benchmark_hyperbolic a,
    benchmark_hyperbolic b
WHERE a.id < b.id AND a.id <= 100 AND b.id <= 100;

-- Poincaré nearest neighbors
\echo ''
\echo '2.2 Poincaré Nearest Neighbors (top-10 from 5K)'
EXPLAIN ANALYZE
SELECT
    id,
    poincare_distance(
        poincare_embedding,
        (SELECT poincare_embedding FROM benchmark_hyperbolic WHERE id = 1)
    ) AS distance
FROM benchmark_hyperbolic
WHERE id != 1
ORDER BY distance
LIMIT 10;

-- Möbius addition
\echo ''
\echo '2.3 Möbius Addition (1K operations)'
EXPLAIN ANALYZE
SELECT
    a.id,
    mobius_add(a.poincare_embedding, b.poincare_embedding) AS result
FROM
    benchmark_hyperbolic a,
    benchmark_hyperbolic b
WHERE a.id <= 50 AND b.id <= 20;

-- Lorentz distance (convert first)
\echo ''
\echo '2.4 Exponential Map (1K operations)'
EXPLAIN ANALYZE
SELECT
    id,
    exp_map(
        poincare_embedding,
        random_unit_vector(32, 0.1)
    ) AS mapped
FROM benchmark_hyperbolic
WHERE id <= 1000;

\echo ''
\echo '=============================================='
\echo 'BENCHMARK 3: Graph Neural Network Operations'
\echo '=============================================='

-- Build adjacency matrix for a subgraph
\echo ''
\echo '3.1 Build Adjacency Matrix (100 nodes)'
EXPLAIN ANALYZE
WITH node_subset AS (
    SELECT id, features FROM benchmark_graph_nodes WHERE id <= 100
),
adj_matrix AS (
    SELECT
        e.source_id,
        e.target_id,
        e.weight
    FROM benchmark_graph_edges e
    WHERE e.source_id <= 100 AND e.target_id <= 100
)
SELECT count(*) FROM adj_matrix;

-- GraphSAGE mean aggregation
\echo ''
\echo '3.2 GraphSAGE Mean Aggregation (100 nodes)'
EXPLAIN ANALYZE
WITH neighbors AS (
    SELECT
        n.id AS node_id,
        n.features AS node_features,
        array_agg(n2.features) AS neighbor_features
    FROM benchmark_graph_nodes n
    LEFT JOIN benchmark_graph_edges e ON n.id = e.source_id
    LEFT JOIN benchmark_graph_nodes n2 ON e.target_id = n2.id
    WHERE n.id <= 100
    GROUP BY n.id, n.features
)
SELECT
    node_id,
    graphsage_mean(node_features, neighbor_features) AS aggregated
FROM neighbors
LIMIT 10;

\echo ''
\echo '=============================================='
\echo 'BENCHMARK 4: Attention Mechanisms'
\echo '=============================================='

-- Scaled dot-product attention
\echo ''
\echo '4.1 Scaled Dot-Product Attention (100 queries x 10 keys)'
EXPLAIN ANALYZE
WITH queries AS (
    SELECT id, embedding_arr FROM benchmark_vectors WHERE id <= 100
),
keys AS (
    SELECT array_agg(embedding_arr) AS k FROM benchmark_vectors WHERE id <= 10
),
vals AS (
    SELECT array_agg(embedding_arr) AS v FROM benchmark_vectors WHERE id <= 10
)
SELECT
    q.id,
    scaled_dot_attention(q.embedding_arr, k.k, v.v) AS attention_output
FROM queries q, keys k, vals v
LIMIT 10;

\echo ''
\echo '=============================================='
\echo 'BENCHMARK 5: Sparse Vectors & BM25'
\echo '=============================================='

-- Sparse dot product
\echo ''
\echo '5.1 Sparse Dot Product (2K x 2K sample)'
EXPLAIN ANALYZE
SELECT
    a.id AS id_a,
    b.id AS id_b,
    sparse_dot(a.sparse_vec, b.sparse_vec) AS similarity
FROM benchmark_sparse a, benchmark_sparse b
WHERE a.id <= 50 AND b.id <= 50 AND a.id < b.id;

-- BM25 search
\echo ''
\echo '5.2 BM25 Search (1K documents)'
EXPLAIN ANALYZE
SELECT
    id,
    title,
    bm25_score(
        ARRAY['machine', 'learning', 'neural'],
        content,
        100.0,
        1000
    ) AS score
FROM benchmark_documents
ORDER BY score DESC
LIMIT 10;

-- Full-text search comparison
\echo ''
\echo '5.3 PostgreSQL Full-Text Search (1K documents)'
EXPLAIN ANALYZE
SELECT
    id,
    title,
    ts_rank(to_tsvector(content), plainto_tsquery('machine learning neural')) AS score
FROM benchmark_documents
WHERE to_tsvector(content) @@ plainto_tsquery('machine learning neural')
ORDER BY score DESC
LIMIT 10;

\echo ''
\echo '=============================================='
\echo 'BENCHMARK 6: Vector Quantization'
\echo '=============================================='

-- Binary quantization
\echo ''
\echo '6.1 Binary Quantization (10K vectors)'
EXPLAIN ANALYZE
SELECT
    id,
    binary_quantize(embedding_arr) AS binary_vec
FROM benchmark_vectors
WHERE id <= 1000;

-- Scalar quantization
\echo ''
\echo '6.2 Scalar Quantization (10K vectors)'
EXPLAIN ANALYZE
SELECT
    id,
    scalar_quantize(embedding_arr) AS quantized
FROM benchmark_vectors
WHERE id <= 1000;

-- Hamming distance on binary vectors
\echo ''
\echo '6.3 Hamming Distance (1K binary vectors)'
EXPLAIN ANALYZE
WITH binary_vecs AS (
    SELECT id, binary_quantize(embedding_arr) AS bvec
    FROM benchmark_vectors
    WHERE id <= 100
)
SELECT
    a.id AS id_a,
    b.id AS id_b,
    hamming_distance(a.bvec, b.bvec) AS distance
FROM binary_vecs a, binary_vecs b
WHERE a.id < b.id AND a.id <= 50;

\echo ''
\echo '=============================================='
\echo 'BENCHMARK 7: Combined Operations'
\echo '=============================================='

-- Hybrid search: Vector + BM25
\echo ''
\echo '7.1 Hybrid Search (Vector + BM25 fusion)'
EXPLAIN ANALYZE
WITH vector_scores AS (
    SELECT
        id,
        1.0 / (1.0 + (embedding <=> (SELECT embedding FROM benchmark_vectors WHERE id = 1))) AS v_score
    FROM benchmark_vectors
    WHERE id <= 100
),
bm25_scores AS (
    SELECT
        id,
        bm25_score(ARRAY['machine', 'learning'], 'machine learning neural network', 100.0, 100) AS b_score
    FROM benchmark_documents
    WHERE id <= 100
)
SELECT
    COALESCE(v.id, b.id) AS id,
    COALESCE(v.v_score, 0) * 0.7 + COALESCE(b.b_score, 0) * 0.3 AS hybrid_score
FROM vector_scores v
FULL OUTER JOIN bm25_scores b ON v.id = b.id
ORDER BY hybrid_score DESC
LIMIT 10;

\echo ''
\echo '=============================================='
\echo 'BENCHMARK 8: Index Performance (HNSW)'
\echo '=============================================='

-- Create HNSW index
\echo ''
\echo '8.1 Creating HNSW Index...'
DROP INDEX IF EXISTS benchmark_vectors_hnsw_idx;
CREATE INDEX benchmark_vectors_hnsw_idx ON benchmark_vectors
USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);

-- Search with index
\echo ''
\echo '8.2 HNSW Index Search (10K vectors, top-10)'
SET hnsw.ef_search = 100;
EXPLAIN ANALYZE
SELECT id, embedding <=> (SELECT embedding FROM benchmark_vectors WHERE id = 500) AS distance
FROM benchmark_vectors
ORDER BY distance
LIMIT 10;

-- IVFFlat index
\echo ''
\echo '8.3 Creating IVFFlat Index...'
DROP INDEX IF EXISTS benchmark_vectors_ivf_idx;
CREATE INDEX benchmark_vectors_ivf_idx ON benchmark_vectors
USING ivfflat (embedding vector_cosine_ops)
WITH (lists = 100);

-- Search with IVFFlat
\echo ''
\echo '8.4 IVFFlat Index Search (10K vectors, top-10)'
SET ivfflat.probes = 10;
EXPLAIN ANALYZE
SELECT id, embedding <=> (SELECT embedding FROM benchmark_vectors WHERE id = 500) AS distance
FROM benchmark_vectors
ORDER BY distance
LIMIT 10;

\echo ''
\echo '=============================================='
\echo 'BENCHMARK SUMMARY'
\echo '=============================================='
\echo ''

SELECT
    'benchmark_vectors' AS table_name,
    count(*) AS row_count,
    pg_size_pretty(pg_total_relation_size('benchmark_vectors')) AS total_size
FROM benchmark_vectors
UNION ALL
SELECT
    'benchmark_hyperbolic',
    count(*),
    pg_size_pretty(pg_total_relation_size('benchmark_hyperbolic'))
FROM benchmark_hyperbolic
UNION ALL
SELECT
    'benchmark_graph_nodes',
    count(*),
    pg_size_pretty(pg_total_relation_size('benchmark_graph_nodes'))
FROM benchmark_graph_nodes
UNION ALL
SELECT
    'benchmark_graph_edges',
    count(*),
    pg_size_pretty(pg_total_relation_size('benchmark_graph_edges'))
FROM benchmark_graph_edges
UNION ALL
SELECT
    'benchmark_documents',
    count(*),
    pg_size_pretty(pg_total_relation_size('benchmark_documents'))
FROM benchmark_documents
UNION ALL
SELECT
    'benchmark_sparse',
    count(*),
    pg_size_pretty(pg_total_relation_size('benchmark_sparse'))
FROM benchmark_sparse;

\echo ''
\echo 'Benchmark completed!'
\echo '=============================================='
