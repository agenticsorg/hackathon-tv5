-- RuVector PostgreSQL Feature Benchmark Suite
-- This benchmark tests hyperbolic embeddings, GNN layers, attention mechanisms,
-- sparse vectors, and quantization - all features of the RuVector extension.

-- ============================================================================
-- SETUP: Create benchmark tables and helper functions
-- ============================================================================

DROP SCHEMA IF EXISTS ruvector_bench CASCADE;
CREATE SCHEMA ruvector_bench;
SET search_path TO ruvector_bench, public;

-- Enable timing
\timing on

-- ============================================================================
-- 1. HYPERBOLIC EMBEDDINGS (Poincaré Ball Model)
-- ============================================================================

-- Poincaré distance function
CREATE OR REPLACE FUNCTION poincare_distance(a float8[], b float8[], curvature float8 DEFAULT -1.0)
RETURNS float8 AS $$
DECLARE
    diff_norm_sq float8;
    a_norm_sq float8;
    b_norm_sq float8;
    num float8;
    denom float8;
    c float8;
BEGIN
    c := abs(curvature);

    -- Calculate norms
    SELECT sum(x*x) INTO a_norm_sq FROM unnest(a) x;
    SELECT sum(x*x) INTO b_norm_sq FROM unnest(b) x;
    SELECT sum((a[i] - b[i])^2) INTO diff_norm_sq
    FROM generate_subscripts(a, 1) i;

    -- Poincaré distance formula
    num := 2.0 * c * diff_norm_sq;
    denom := (1.0 - c * a_norm_sq) * (1.0 - c * b_norm_sq);

    -- Clamp for numerical stability
    IF denom <= 0 THEN
        RETURN 'infinity'::float8;
    END IF;

    RETURN (1.0 / sqrt(c)) * asinh(sqrt(num / denom));
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

-- Lorentz/Hyperboloid distance function
CREATE OR REPLACE FUNCTION lorentz_distance(a float8[], b float8[])
RETURNS float8 AS $$
DECLARE
    minkowski_prod float8;
BEGIN
    -- Minkowski inner product: -x0*y0 + x1*y1 + x2*y2 + ...
    SELECT -a[1]*b[1] + sum(a[i]*b[i])
    INTO minkowski_prod
    FROM generate_subscripts(a, 1) i WHERE i > 1;

    -- Distance = acosh(-<x,y>_L)
    IF minkowski_prod >= -1 THEN
        RETURN 0;
    END IF;

    RETURN acosh(-minkowski_prod);
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

-- Möbius addition in Poincaré ball
CREATE OR REPLACE FUNCTION mobius_add(a float8[], b float8[], curvature float8 DEFAULT -1.0)
RETURNS float8[] AS $$
DECLARE
    c float8;
    a_norm_sq float8;
    b_norm_sq float8;
    ab_dot float8;
    num_coef_a float8;
    num_coef_b float8;
    denom float8;
    result float8[];
BEGIN
    c := abs(curvature);

    SELECT sum(x*x) INTO a_norm_sq FROM unnest(a) x;
    SELECT sum(x*x) INTO b_norm_sq FROM unnest(b) x;
    SELECT sum(a[i]*b[i]) INTO ab_dot FROM generate_subscripts(a, 1) i;

    num_coef_a := 1.0 + 2.0*c*ab_dot + c*b_norm_sq;
    num_coef_b := 1.0 - c*a_norm_sq;
    denom := 1.0 + 2.0*c*ab_dot + c*c*a_norm_sq*b_norm_sq;

    SELECT array_agg((num_coef_a * a[i] + num_coef_b * b[i]) / denom ORDER BY i)
    INTO result
    FROM generate_subscripts(a, 1) i;

    RETURN result;
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

-- Exponential map (tangent space to Poincaré ball)
CREATE OR REPLACE FUNCTION exp_map(x float8[], v float8[], curvature float8 DEFAULT -1.0)
RETURNS float8[] AS $$
DECLARE
    c float8;
    v_norm float8;
    x_norm_sq float8;
    lambda_x float8;
    coef float8;
    scaled_v float8[];
BEGIN
    c := abs(curvature);

    SELECT sqrt(sum(t*t)) INTO v_norm FROM unnest(v) t;
    SELECT sum(t*t) INTO x_norm_sq FROM unnest(x) t;

    IF v_norm < 1e-10 THEN
        RETURN x;
    END IF;

    lambda_x := 2.0 / (1.0 - c * x_norm_sq);
    coef := tanh(sqrt(c) * lambda_x * v_norm / 2.0) / (sqrt(c) * v_norm);

    SELECT array_agg(v[i] * coef ORDER BY i)
    INTO scaled_v
    FROM generate_subscripts(v, 1) i;

    RETURN mobius_add(x, scaled_v, curvature);
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

-- ============================================================================
-- 2. GRAPH NEURAL NETWORK LAYERS
-- ============================================================================

-- GCN Layer (Graph Convolutional Network)
CREATE OR REPLACE FUNCTION gcn_layer(
    node_features float8[][],
    adjacency float8[][],
    weights float8[][]
)
RETURNS float8[][] AS $$
DECLARE
    n int := array_length(node_features, 1);
    d_in int := array_length(node_features, 2);
    d_out int := array_length(weights, 2);
    normalized_adj float8[][];
    result float8[][];
    degree float8[];
    i int;
    j int;
    k int;
    temp_sum float8;
BEGIN
    -- Calculate degree
    degree := ARRAY[]::float8[];
    FOR i IN 1..n LOOP
        SELECT sum(adjacency[i][j]) + 1 INTO temp_sum FROM generate_series(1, n) j;
        degree := array_append(degree, 1.0 / sqrt(temp_sum));
    END LOOP;

    -- D^-1/2 * A * D^-1/2 normalization (simplified)
    normalized_adj := adjacency;
    FOR i IN 1..n LOOP
        FOR j IN 1..n LOOP
            normalized_adj[i][j] := (adjacency[i][j] + CASE WHEN i=j THEN 1 ELSE 0 END) * degree[i] * degree[j];
        END LOOP;
    END LOOP;

    -- Aggregate: normalized_adj @ node_features
    -- Then multiply by weights
    result := ARRAY[]::float8[][];
    FOR i IN 1..n LOOP
        result := array_cat(result, ARRAY[ARRAY[]::float8[]]);
        FOR k IN 1..d_out LOOP
            temp_sum := 0;
            FOR j IN 1..n LOOP
                FOR l IN 1..d_in LOOP
                    temp_sum := temp_sum + normalized_adj[i][j] * node_features[j][l] * weights[l][k];
                END LOOP;
            END LOOP;
            result[i] := array_append(result[i], greatest(0, temp_sum)); -- ReLU
        END LOOP;
    END LOOP;

    RETURN result;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- GraphSAGE mean aggregator
CREATE OR REPLACE FUNCTION graphsage_mean(
    node_features float8[],
    neighbor_features float8[][]
)
RETURNS float8[] AS $$
DECLARE
    n_neighbors int := array_length(neighbor_features, 1);
    dim int := array_length(node_features, 1);
    agg_features float8[];
    concat_features float8[];
    i int;
    mean_val float8;
BEGIN
    IF n_neighbors IS NULL OR n_neighbors = 0 THEN
        -- Self-loop only
        RETURN node_features;
    END IF;

    -- Mean aggregation of neighbors
    agg_features := ARRAY[]::float8[];
    FOR i IN 1..dim LOOP
        SELECT avg(neighbor_features[j][i]) INTO mean_val
        FROM generate_series(1, n_neighbors) j;
        agg_features := array_append(agg_features, COALESCE(mean_val, 0));
    END LOOP;

    -- Concatenate self and aggregated neighbor features
    concat_features := node_features || agg_features;

    RETURN concat_features;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ============================================================================
-- 3. ATTENTION MECHANISMS
-- ============================================================================

-- Scaled dot-product attention
CREATE OR REPLACE FUNCTION scaled_dot_attention(
    query float8[],
    keys float8[][],
    values float8[][],
    scale float8 DEFAULT NULL
)
RETURNS float8[] AS $$
DECLARE
    n_keys int := array_length(keys, 1);
    dim int := array_length(query, 1);
    scores float8[];
    softmax_scores float8[];
    max_score float8;
    exp_sum float8;
    result float8[];
    i int;
    j int;
    dot_prod float8;
    s float8;
BEGIN
    -- Default scale
    IF scale IS NULL THEN
        s := 1.0 / sqrt(dim);
    ELSE
        s := scale;
    END IF;

    -- Compute attention scores
    scores := ARRAY[]::float8[];
    FOR i IN 1..n_keys LOOP
        SELECT sum(query[j] * keys[i][j]) * s INTO dot_prod
        FROM generate_series(1, dim) j;
        scores := array_append(scores, dot_prod);
    END LOOP;

    -- Softmax
    SELECT max(x) INTO max_score FROM unnest(scores) x;
    SELECT sum(exp(x - max_score)) INTO exp_sum FROM unnest(scores) x;

    softmax_scores := ARRAY[]::float8[];
    FOR i IN 1..n_keys LOOP
        softmax_scores := array_append(softmax_scores, exp(scores[i] - max_score) / exp_sum);
    END LOOP;

    -- Weighted sum of values
    result := ARRAY[]::float8[];
    FOR j IN 1..array_length(values, 2) LOOP
        SELECT sum(softmax_scores[i] * values[i][j]) INTO dot_prod
        FROM generate_series(1, n_keys) i;
        result := array_append(result, dot_prod);
    END LOOP;

    RETURN result;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Multi-head attention
CREATE OR REPLACE FUNCTION multi_head_attention(
    query float8[],
    keys float8[][],
    values float8[][],
    num_heads int DEFAULT 4
)
RETURNS float8[] AS $$
DECLARE
    dim int := array_length(query, 1);
    head_dim int := dim / num_heads;
    heads_output float8[][];
    h int;
    q_slice float8[];
    k_slices float8[][];
    v_slices float8[][];
    head_result float8[];
    final_result float8[];
    start_idx int;
    end_idx int;
    i int;
    j int;
BEGIN
    IF dim % num_heads != 0 THEN
        RAISE EXCEPTION 'Dimension must be divisible by num_heads';
    END IF;

    heads_output := ARRAY[]::float8[][];

    FOR h IN 0..num_heads-1 LOOP
        start_idx := h * head_dim + 1;
        end_idx := (h + 1) * head_dim;

        -- Slice query
        SELECT array_agg(query[i] ORDER BY i) INTO q_slice
        FROM generate_series(start_idx, end_idx) i;

        -- Slice keys
        k_slices := ARRAY[]::float8[][];
        FOR i IN 1..array_length(keys, 1) LOOP
            SELECT array_agg(keys[i][j] ORDER BY j) INTO q_slice
            FROM generate_series(start_idx, end_idx) j;
            k_slices := array_cat(k_slices, ARRAY[q_slice]);
        END LOOP;

        -- Slice values (same as keys for simplicity)
        v_slices := k_slices;

        -- Compute attention for this head
        head_result := scaled_dot_attention(q_slice, k_slices, v_slices);
        heads_output := array_cat(heads_output, ARRAY[head_result]);
    END LOOP;

    -- Concatenate heads
    final_result := ARRAY[]::float8[];
    FOR h IN 1..num_heads LOOP
        FOR i IN 1..head_dim LOOP
            final_result := array_append(final_result, heads_output[h][i]);
        END LOOP;
    END LOOP;

    RETURN final_result;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ============================================================================
-- 4. SPARSE VECTORS & BM25
-- ============================================================================

-- Sparse vector representation: ARRAY of (index, value) pairs as JSON
CREATE TYPE sparse_vector AS (
    indices int[],
    values float8[],
    dim int
);

-- Create sparse vector from dense
CREATE OR REPLACE FUNCTION to_sparse(dense float8[], threshold float8 DEFAULT 0.0)
RETURNS sparse_vector AS $$
DECLARE
    result sparse_vector;
    idx int;
BEGIN
    result.indices := ARRAY[]::int[];
    result.values := ARRAY[]::float8[];
    result.dim := array_length(dense, 1);

    FOR idx IN 1..result.dim LOOP
        IF abs(dense[idx]) > threshold THEN
            result.indices := array_append(result.indices, idx);
            result.values := array_append(result.values, dense[idx]);
        END IF;
    END LOOP;

    RETURN result;
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

-- Sparse dot product
CREATE OR REPLACE FUNCTION sparse_dot(a sparse_vector, b sparse_vector)
RETURNS float8 AS $$
DECLARE
    result float8 := 0;
    i int;
    j int;
    pos_a int := 1;
    pos_b int := 1;
    len_a int := array_length(a.indices, 1);
    len_b int := array_length(b.indices, 1);
BEGIN
    IF len_a IS NULL OR len_b IS NULL THEN
        RETURN 0;
    END IF;

    -- Merge-style intersection
    WHILE pos_a <= len_a AND pos_b <= len_b LOOP
        IF a.indices[pos_a] = b.indices[pos_b] THEN
            result := result + a.values[pos_a] * b.values[pos_b];
            pos_a := pos_a + 1;
            pos_b := pos_b + 1;
        ELSIF a.indices[pos_a] < b.indices[pos_b] THEN
            pos_a := pos_a + 1;
        ELSE
            pos_b := pos_b + 1;
        END IF;
    END LOOP;

    RETURN result;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- BM25 scoring function
CREATE OR REPLACE FUNCTION bm25_score(
    query_terms text[],
    document text,
    avg_doc_length float8 DEFAULT 100.0,
    total_docs int DEFAULT 1000,
    k1 float8 DEFAULT 1.2,
    b float8 DEFAULT 0.75
)
RETURNS float8 AS $$
DECLARE
    doc_terms text[];
    doc_length int;
    term text;
    tf int;
    df int;
    idf float8;
    score float8 := 0;
BEGIN
    -- Tokenize document
    doc_terms := string_to_array(lower(document), ' ');
    doc_length := array_length(doc_terms, 1);

    IF doc_length IS NULL THEN
        RETURN 0;
    END IF;

    FOREACH term IN ARRAY query_terms LOOP
        -- Term frequency in document
        SELECT count(*) INTO tf FROM unnest(doc_terms) t WHERE t = lower(term);

        IF tf > 0 THEN
            -- Inverse document frequency (simplified)
            df := greatest(1, total_docs / 10); -- Simplified estimate
            idf := ln((total_docs - df + 0.5) / (df + 0.5) + 1);

            -- BM25 formula
            score := score + idf * (tf * (k1 + 1)) /
                     (tf + k1 * (1 - b + b * doc_length / avg_doc_length));
        END IF;
    END LOOP;

    RETURN score;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ============================================================================
-- 5. VECTOR QUANTIZATION
-- ============================================================================

-- Binary quantization (1-bit per dimension)
CREATE OR REPLACE FUNCTION binary_quantize(vec float8[])
RETURNS bit varying AS $$
DECLARE
    result bit varying := '';
    val float8;
BEGIN
    FOREACH val IN ARRAY vec LOOP
        result := result || CASE WHEN val >= 0 THEN B'1' ELSE B'0' END;
    END LOOP;
    RETURN result;
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

-- Scalar quantization (8-bit per dimension)
CREATE OR REPLACE FUNCTION scalar_quantize(vec float8[], min_val float8 DEFAULT -1.0, max_val float8 DEFAULT 1.0)
RETURNS int[] AS $$
DECLARE
    result int[];
    val float8;
    normalized float8;
    quantized int;
BEGIN
    result := ARRAY[]::int[];

    FOREACH val IN ARRAY vec LOOP
        -- Normalize to 0-1 range
        normalized := (val - min_val) / (max_val - min_val);
        normalized := greatest(0, least(1, normalized));
        -- Quantize to 0-255
        quantized := floor(normalized * 255)::int;
        result := array_append(result, quantized);
    END LOOP;

    RETURN result;
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

-- Hamming distance for binary vectors
CREATE OR REPLACE FUNCTION hamming_distance(a bit varying, b bit varying)
RETURNS int AS $$
BEGIN
    RETURN length(replace((a # b)::text, '0', ''));
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

-- ============================================================================
-- 6. BENCHMARK DATA GENERATION
-- ============================================================================

-- Generate random vector
CREATE OR REPLACE FUNCTION random_vector(dim int)
RETURNS float8[] AS $$
SELECT array_agg(random() * 2 - 1)
FROM generate_series(1, dim);
$$ LANGUAGE SQL VOLATILE;

-- Generate random unit vector (for hyperbolic space)
CREATE OR REPLACE FUNCTION random_unit_vector(dim int, max_norm float8 DEFAULT 0.9)
RETURNS float8[] AS $$
DECLARE
    vec float8[];
    norm float8;
    scale float8;
BEGIN
    vec := random_vector(dim);
    SELECT sqrt(sum(x*x)) INTO norm FROM unnest(vec) x;
    scale := (random() * max_norm) / norm;
    SELECT array_agg(x * scale ORDER BY i) INTO vec
    FROM unnest(vec) WITH ORDINALITY AS t(x, i);
    RETURN vec;
END;
$$ LANGUAGE plpgsql VOLATILE;

-- ============================================================================
-- 7. BENCHMARK TABLES
-- ============================================================================

-- Dense vectors table
CREATE TABLE benchmark_vectors (
    id serial PRIMARY KEY,
    embedding vector(384),
    embedding_arr float8[],
    metadata jsonb
);

-- Hyperbolic embeddings table
CREATE TABLE benchmark_hyperbolic (
    id serial PRIMARY KEY,
    poincare_embedding float8[],
    hierarchy_level int
);

-- Graph nodes for GNN
CREATE TABLE benchmark_graph_nodes (
    id serial PRIMARY KEY,
    features float8[],
    node_type text
);

-- Graph edges
CREATE TABLE benchmark_graph_edges (
    id serial PRIMARY KEY,
    source_id int REFERENCES benchmark_graph_nodes(id),
    target_id int REFERENCES benchmark_graph_nodes(id),
    weight float8 DEFAULT 1.0
);

-- Documents for BM25
CREATE TABLE benchmark_documents (
    id serial PRIMARY KEY,
    content text,
    title text
);

-- Sparse vectors
CREATE TABLE benchmark_sparse (
    id serial PRIMARY KEY,
    sparse_vec sparse_vector,
    original_dim int
);

-- ============================================================================
-- 8. POPULATE BENCHMARK DATA
-- ============================================================================

-- Insert test vectors
INSERT INTO benchmark_vectors (embedding_arr, metadata)
SELECT
    random_vector(384),
    jsonb_build_object('idx', i, 'category', 'test')
FROM generate_series(1, 10000) i;

-- Convert to pgvector format
UPDATE benchmark_vectors
SET embedding = embedding_arr::vector;

-- Insert hyperbolic embeddings
INSERT INTO benchmark_hyperbolic (poincare_embedding, hierarchy_level)
SELECT
    random_unit_vector(32, 0.95),
    (random() * 5)::int
FROM generate_series(1, 5000);

-- Insert graph nodes
INSERT INTO benchmark_graph_nodes (features, node_type)
SELECT
    random_vector(64),
    CASE (random() * 3)::int WHEN 0 THEN 'user' WHEN 1 THEN 'item' ELSE 'tag' END
FROM generate_series(1, 1000);

-- Insert graph edges (random connectivity)
INSERT INTO benchmark_graph_edges (source_id, target_id, weight)
SELECT
    (random() * 999 + 1)::int,
    (random() * 999 + 1)::int,
    random()
FROM generate_series(1, 5000)
WHERE (random() * 999 + 1)::int != (random() * 999 + 1)::int;

-- Insert documents
INSERT INTO benchmark_documents (content, title)
SELECT
    'This is document ' || i || ' about machine learning vectors embeddings neural networks ' ||
    CASE (i % 5) WHEN 0 THEN 'attention transformers' WHEN 1 THEN 'graph networks'
    WHEN 2 THEN 'hyperbolic geometry' WHEN 3 THEN 'sparse retrieval' ELSE 'quantization' END,
    'Document ' || i
FROM generate_series(1, 1000) i;

-- Insert sparse vectors
INSERT INTO benchmark_sparse (sparse_vec, original_dim)
SELECT
    to_sparse(random_vector(1000), 0.8),
    1000
FROM generate_series(1, 2000);

-- ============================================================================
-- BENCHMARK EXECUTION
-- ============================================================================

\echo '=============================================='
\echo 'RuVector Feature Benchmark Results'
\echo '=============================================='
\echo ''
