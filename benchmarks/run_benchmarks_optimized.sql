-- RuVector Optimized Benchmark Execution
-- Uses parallel queries and optimized indexes

SET search_path TO ruvector_bench, public;
SET max_parallel_workers_per_gather = 4;
SET hnsw.ef_search = 100;
SET work_mem = '256MB';
SET jit = on;

\echo ''
\echo '=============================================='
\echo 'OPTIMIZED BENCHMARK 1: Dense Vector Operations'
\echo '=============================================='

\echo ''
\echo '1.1 HNSW Cosine Search (10K vectors, top-10) - INDEXED'
EXPLAIN ANALYZE
SELECT id, embedding <=> (SELECT embedding FROM benchmark_vectors WHERE id = 1) AS distance
FROM benchmark_vectors
ORDER BY distance
LIMIT 10;

\echo ''
\echo '1.2 Parallel L2 Search (10K vectors, top-10)'
EXPLAIN ANALYZE
SELECT id, embedding <-> (SELECT embedding FROM benchmark_vectors WHERE id = 500) AS distance
FROM benchmark_vectors
ORDER BY distance
LIMIT 10;

\echo ''
\echo '=============================================='
\echo 'OPTIMIZED BENCHMARK 2: Hyperbolic Embeddings'
\echo '=============================================='

\echo ''
\echo '2.1 Poincaré Distance Batch (optimized SQL functions)'
EXPLAIN ANALYZE
SELECT a.id, b.id, poincare_distance(a.poincare_embedding, b.poincare_embedding) AS dist
FROM benchmark_hyperbolic a, benchmark_hyperbolic b
WHERE a.id <= 50 AND b.id <= 50 AND a.id < b.id;

\echo ''
\echo '2.2 Poincaré KNN Search (top-10 from 5K)'
EXPLAIN ANALYZE
SELECT id, poincare_distance(poincare_embedding,
    (SELECT poincare_embedding FROM benchmark_hyperbolic WHERE id = 1)) AS dist
FROM benchmark_hyperbolic
WHERE id != 1
ORDER BY dist
LIMIT 10;

\echo ''
\echo '2.3 Möbius Addition Batch (1K ops)'
EXPLAIN ANALYZE
SELECT a.id, mobius_add(a.poincare_embedding, b.poincare_embedding)
FROM benchmark_hyperbolic a, benchmark_hyperbolic b
WHERE a.id <= 32 AND b.id <= 32;

\echo ''
\echo '=============================================='
\echo 'OPTIMIZED BENCHMARK 3: Graph Neural Networks'
\echo '=============================================='

\echo ''
\echo '3.1 Indexed Edge Lookup'
EXPLAIN ANALYZE
SELECT n.id, n.features, array_agg(n2.features) AS neighbor_features
FROM benchmark_graph_nodes n
JOIN benchmark_graph_edges e ON n.id = e.source_id
JOIN benchmark_graph_nodes n2 ON e.target_id = n2.id
WHERE n.id <= 100
GROUP BY n.id, n.features;

\echo ''
\echo '3.2 GraphSAGE Aggregation (100 nodes)'
EXPLAIN ANALYZE
WITH neighbors AS (
    SELECT n.id, n.features, array_agg(n2.features) AS nf
    FROM benchmark_graph_nodes n
    LEFT JOIN benchmark_graph_edges e ON n.id = e.source_id
    LEFT JOIN benchmark_graph_nodes n2 ON e.target_id = n2.id
    WHERE n.id <= 100
    GROUP BY n.id, n.features
)
SELECT id, graphsage_mean(features, nf) FROM neighbors LIMIT 20;

\echo ''
\echo '=============================================='
\echo 'OPTIMIZED BENCHMARK 4: Sparse Vectors & BM25'
\echo '=============================================='

\echo ''
\echo '4.1 Sparse Dot Product (hash join optimization)'
EXPLAIN ANALYZE
SELECT a.id, b.id, sparse_dot(a.sparse_vec, b.sparse_vec)
FROM benchmark_sparse a, benchmark_sparse b
WHERE a.id <= 30 AND b.id <= 30 AND a.id < b.id;

\echo ''
\echo '4.2 BM25 Search (optimized tokenization)'
EXPLAIN ANALYZE
SELECT id, title, bm25_score(ARRAY['machine', 'learning', 'neural'], content)
FROM benchmark_documents
ORDER BY bm25_score(ARRAY['machine', 'learning', 'neural'], content) DESC
LIMIT 10;

\echo ''
\echo '4.3 Native Full-Text Search (GIN indexed)'
EXPLAIN ANALYZE
SELECT id, title, ts_rank_cd(tsv, q) AS rank
FROM benchmark_documents, plainto_tsquery('english', 'machine learning neural') q
WHERE tsv @@ q
ORDER BY rank DESC
LIMIT 10;

\echo ''
\echo '=============================================='
\echo 'OPTIMIZED BENCHMARK 5: Vector Quantization'
\echo '=============================================='

\echo ''
\echo '5.1 Binary Quantization (batch, 1K vectors)'
EXPLAIN ANALYZE
SELECT id, binary_quantize(embedding_arr) FROM benchmark_vectors WHERE id <= 1000;

\echo ''
\echo '5.2 Scalar Quantization (batch, 1K vectors)'
EXPLAIN ANALYZE
SELECT id, scalar_quantize(embedding_arr) FROM benchmark_vectors WHERE id <= 1000;

\echo ''
\echo '5.3 Hamming Distance (bit_count optimization)'
EXPLAIN ANALYZE
WITH bvecs AS (
    SELECT id, binary_quantize(embedding_arr) AS bv FROM benchmark_vectors WHERE id <= 100
)
SELECT a.id, b.id, hamming_distance(a.bv, b.bv)
FROM bvecs a, bvecs b
WHERE a.id < b.id AND a.id <= 50;

\echo ''
\echo '=============================================='
\echo 'OPTIMIZED BENCHMARK 6: Hybrid Operations'
\echo '=============================================='

\echo ''
\echo '6.1 Hybrid Vector + BM25 Search'
EXPLAIN ANALYZE
WITH vec_results AS (
    SELECT id, 1.0 / (1.0 + (embedding <=> (SELECT embedding FROM benchmark_vectors WHERE id = 1))) AS vscore
    FROM benchmark_vectors
    ORDER BY embedding <=> (SELECT embedding FROM benchmark_vectors WHERE id = 1)
    LIMIT 100
),
text_results AS (
    SELECT id, ts_rank_cd(tsv, q) AS tscore
    FROM benchmark_documents, plainto_tsquery('machine learning') q
    WHERE tsv @@ q
)
SELECT COALESCE(v.id, t.id) AS id,
       COALESCE(v.vscore, 0) * 0.7 + COALESCE(t.tscore, 0) * 0.3 AS hybrid_score
FROM vec_results v
FULL OUTER JOIN text_results t ON v.id = t.id
ORDER BY hybrid_score DESC
LIMIT 10;

\echo ''
\echo '=============================================='
\echo 'BENCHMARK COMPARISON SUMMARY'
\echo '=============================================='

-- Summary statistics
SELECT
    'benchmark_vectors' AS table_name,
    count(*) AS rows,
    pg_size_pretty(pg_total_relation_size('benchmark_vectors')) AS size,
    (SELECT count(*) FROM pg_indexes WHERE tablename = 'benchmark_vectors') AS indexes
FROM benchmark_vectors
UNION ALL
SELECT 'benchmark_hyperbolic', count(*),
    pg_size_pretty(pg_total_relation_size('benchmark_hyperbolic')),
    (SELECT count(*) FROM pg_indexes WHERE tablename = 'benchmark_hyperbolic')
FROM benchmark_hyperbolic
UNION ALL
SELECT 'benchmark_graph_nodes', count(*),
    pg_size_pretty(pg_total_relation_size('benchmark_graph_nodes')),
    (SELECT count(*) FROM pg_indexes WHERE tablename = 'benchmark_graph_nodes')
FROM benchmark_graph_nodes
UNION ALL
SELECT 'benchmark_graph_edges', count(*),
    pg_size_pretty(pg_total_relation_size('benchmark_graph_edges')),
    (SELECT count(*) FROM pg_indexes WHERE tablename = 'benchmark_graph_edges')
FROM benchmark_graph_edges
UNION ALL
SELECT 'benchmark_documents', count(*),
    pg_size_pretty(pg_total_relation_size('benchmark_documents')),
    (SELECT count(*) FROM pg_indexes WHERE tablename = 'benchmark_documents')
FROM benchmark_documents
UNION ALL
SELECT 'benchmark_sparse', count(*),
    pg_size_pretty(pg_total_relation_size('benchmark_sparse')),
    (SELECT count(*) FROM pg_indexes WHERE tablename = 'benchmark_sparse')
FROM benchmark_sparse;

\echo ''
\echo 'Optimized benchmark completed!'
\echo '=============================================='
