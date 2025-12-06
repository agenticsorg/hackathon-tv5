-- RuVector TV5 Benchmark Suite with Raft Consensus and Scale Testing
-- Optimized for distributed vector operations, consensus algorithms, and federated learning

-- ============================================================================
-- SETUP: TV5 Distributed Benchmark Environment
-- ============================================================================

DROP SCHEMA IF EXISTS tv5_bench CASCADE;
CREATE SCHEMA tv5_bench;
SET search_path TO tv5_bench, public;

-- Performance settings for scale testing
SET max_parallel_workers_per_gather = 4;
SET parallel_tuple_cost = 0.001;
SET work_mem = '512MB';
SET maintenance_work_mem = '1GB';
SET effective_cache_size = '4GB';
SET jit = on;

\timing on

-- ============================================================================
-- 1. RAFT CONSENSUS DATA STRUCTURES
-- ============================================================================

-- Raft node states
CREATE TYPE raft_state AS ENUM ('follower', 'candidate', 'leader');

-- Raft log entries
CREATE TABLE raft_log (
    term int NOT NULL,
    log_index bigint NOT NULL,
    command jsonb NOT NULL,
    committed boolean DEFAULT false,
    created_at timestamptz DEFAULT now(),
    PRIMARY KEY (term, log_index)
);

-- Raft cluster nodes
CREATE TABLE raft_nodes (
    node_id text PRIMARY KEY,
    state raft_state DEFAULT 'follower',
    current_term int DEFAULT 0,
    voted_for text,
    last_heartbeat timestamptz DEFAULT now(),
    commit_index bigint DEFAULT 0,
    last_applied bigint DEFAULT 0,
    -- Vector embedding for node similarity (routing optimization)
    capability_vector vector(64)
);

-- Raft vote records
CREATE TABLE raft_votes (
    term int NOT NULL,
    candidate_id text NOT NULL,
    voter_id text NOT NULL,
    granted boolean NOT NULL,
    vote_time timestamptz DEFAULT now(),
    PRIMARY KEY (term, voter_id)
);

-- ============================================================================
-- 2. SCALE TESTING DATA STRUCTURES
-- ============================================================================

-- Scale vectors table with shard-based logical partitioning
CREATE TABLE scale_vectors (
    shard_id int NOT NULL,
    id bigserial,
    embedding vector(384),
    embedding_arr float8[],
    metadata jsonb,
    created_at timestamptz DEFAULT now(),
    PRIMARY KEY (shard_id, id)
) WITH (fillfactor = 90);

-- Index for shard-based queries
CREATE INDEX scale_vectors_shard_idx ON scale_vectors(shard_id);

-- Federated agent states (distributed learning)
CREATE TABLE federated_agents (
    agent_id text PRIMARY KEY,
    shard_id int NOT NULL,
    embedding vector(384),
    quality float8 DEFAULT 0.0,
    task_count int DEFAULT 0,
    last_sync timestamptz DEFAULT now(),
    state jsonb DEFAULT '{}'::jsonb
);

-- Distributed consensus metrics
CREATE TABLE consensus_metrics (
    metric_id serial PRIMARY KEY,
    cluster_id text NOT NULL,
    round_number int NOT NULL,
    consensus_score float8,
    participant_count int,
    latency_ms float8,
    throughput_ops float8,
    timestamp timestamptz DEFAULT now()
);

-- ============================================================================
-- 3. RAFT CONSENSUS FUNCTIONS
-- ============================================================================

-- Request vote RPC
CREATE OR REPLACE FUNCTION raft_request_vote(
    p_term int,
    p_candidate_id text,
    p_voter_id text,
    p_last_log_index bigint,
    p_last_log_term int
)
RETURNS boolean AS $$
DECLARE
    voter_term int;
    voter_voted_for text;
    voter_last_index bigint;
    voter_last_term int;
    vote_granted boolean := false;
BEGIN
    -- Get voter state
    SELECT current_term, voted_for INTO voter_term, voter_voted_for
    FROM raft_nodes WHERE node_id = p_voter_id;

    -- Get voter's last log entry
    SELECT COALESCE(max(log_index), 0), COALESCE(max(term), 0)
    INTO voter_last_index, voter_last_term
    FROM raft_log WHERE committed = true;

    -- Grant vote if:
    -- 1. Candidate term >= voter term
    -- 2. Voter hasn't voted or voted for this candidate
    -- 3. Candidate log is at least as up-to-date
    IF p_term >= voter_term AND
       (voter_voted_for IS NULL OR voter_voted_for = p_candidate_id) AND
       (p_last_log_term > voter_last_term OR
        (p_last_log_term = voter_last_term AND p_last_log_index >= voter_last_index))
    THEN
        vote_granted := true;

        -- Update voter state
        UPDATE raft_nodes
        SET current_term = p_term,
            voted_for = p_candidate_id,
            state = 'follower'
        WHERE node_id = p_voter_id;
    END IF;

    -- Record vote
    INSERT INTO raft_votes (term, candidate_id, voter_id, granted)
    VALUES (p_term, p_candidate_id, p_voter_id, vote_granted)
    ON CONFLICT (term, voter_id) DO UPDATE SET granted = vote_granted;

    RETURN vote_granted;
END;
$$ LANGUAGE plpgsql;

-- Append entries RPC (log replication)
CREATE OR REPLACE FUNCTION raft_append_entries(
    p_term int,
    p_leader_id text,
    p_prev_log_index bigint,
    p_prev_log_term int,
    p_entries jsonb[],
    p_leader_commit bigint,
    p_follower_id text
)
RETURNS boolean AS $$
DECLARE
    follower_term int;
    prev_entry_term int;
    i int;
    entry jsonb;
    new_index bigint;
BEGIN
    -- Get follower state
    SELECT current_term INTO follower_term
    FROM raft_nodes WHERE node_id = p_follower_id;

    -- Reject if term is stale
    IF p_term < follower_term THEN
        RETURN false;
    END IF;

    -- Check prev log entry matches
    IF p_prev_log_index > 0 THEN
        SELECT term INTO prev_entry_term
        FROM raft_log WHERE log_index = p_prev_log_index;

        IF prev_entry_term IS NULL OR prev_entry_term != p_prev_log_term THEN
            RETURN false;
        END IF;
    END IF;

    -- Append new entries
    new_index := p_prev_log_index;
    FOREACH entry IN ARRAY p_entries LOOP
        new_index := new_index + 1;
        INSERT INTO raft_log (term, log_index, command, committed)
        VALUES (p_term, new_index, entry, false)
        ON CONFLICT (term, log_index) DO UPDATE SET command = entry;
    END LOOP;

    -- Update commit index
    IF p_leader_commit > 0 THEN
        UPDATE raft_log SET committed = true
        WHERE log_index <= p_leader_commit;
    END IF;

    -- Update follower state
    UPDATE raft_nodes
    SET current_term = p_term,
        state = 'follower',
        last_heartbeat = now(),
        commit_index = p_leader_commit
    WHERE node_id = p_follower_id;

    RETURN true;
END;
$$ LANGUAGE plpgsql;

-- Leader election
CREATE OR REPLACE FUNCTION raft_elect_leader(p_cluster_size int DEFAULT 5)
RETURNS TABLE(leader_id text, term int, votes_received int) AS $$
DECLARE
    candidate record;
    vote_count int;
    majority int;
BEGIN
    majority := p_cluster_size / 2 + 1;

    -- Find candidate with most votes in current term
    FOR candidate IN
        SELECT n.node_id, n.current_term,
               count(v.voter_id) FILTER (WHERE v.granted) as votes
        FROM raft_nodes n
        LEFT JOIN raft_votes v ON v.term = n.current_term AND v.candidate_id = n.node_id
        WHERE n.state = 'candidate'
        GROUP BY n.node_id, n.current_term
        ORDER BY votes DESC, n.current_term DESC
        LIMIT 1
    LOOP
        IF candidate.votes >= majority THEN
            -- Promote to leader
            UPDATE raft_nodes
            SET state = 'leader'
            WHERE node_id = candidate.node_id;

            RETURN QUERY SELECT candidate.node_id, candidate.current_term, candidate.votes::int;
        END IF;
    END LOOP;

    RETURN;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 4. DISTRIBUTED VECTOR OPERATIONS
-- ============================================================================

-- Parallel vector insert across shards
CREATE OR REPLACE FUNCTION scale_insert_vectors(
    p_shard_id int,
    p_count int,
    p_dimension int DEFAULT 384
)
RETURNS bigint AS $$
DECLARE
    inserted_count bigint;
BEGIN
    INSERT INTO scale_vectors (shard_id, embedding_arr)
    SELECT p_shard_id,
           (SELECT array_agg(random() * 2 - 1) FROM generate_series(1, p_dimension))
    FROM generate_series(1, p_count);

    GET DIAGNOSTICS inserted_count = ROW_COUNT;

    -- Update vector column
    UPDATE scale_vectors
    SET embedding = embedding_arr::vector
    WHERE shard_id = p_shard_id AND embedding IS NULL;

    RETURN inserted_count;
END;
$$ LANGUAGE plpgsql;

-- Distributed vector search across shards
CREATE OR REPLACE FUNCTION scale_search_vectors(
    p_query vector,
    p_k int DEFAULT 10,
    p_shards int[] DEFAULT NULL
)
RETURNS TABLE(shard_id int, id bigint, distance float8) AS $$
BEGIN
    RETURN QUERY
    SELECT sv.shard_id, sv.id, sv.embedding <=> p_query AS dist
    FROM scale_vectors sv
    WHERE p_shards IS NULL OR sv.shard_id = ANY(p_shards)
    ORDER BY dist
    LIMIT p_k;
END;
$$ LANGUAGE plpgsql PARALLEL SAFE;

-- Federated embedding aggregation (FedAvg algorithm)
CREATE OR REPLACE FUNCTION federated_aggregate(
    p_cluster_id text,
    p_min_quality float8 DEFAULT 0.7
)
RETURNS vector AS $$
DECLARE
    agg_embedding float8[];
    total_weight float8;
    dim int := 384;
BEGIN
    -- Weighted average based on quality scores
    SELECT
        array_agg(sum_val / total_w ORDER BY idx),
        max(total_w)
    INTO agg_embedding, total_weight
    FROM (
        SELECT
            idx,
            sum(embedding[idx]::float8 * quality) AS sum_val,
            sum(quality) AS total_w
        FROM federated_agents, generate_series(1, dim) AS idx
        WHERE quality >= p_min_quality
        GROUP BY idx
    ) AS aggregated;

    IF total_weight IS NULL OR total_weight = 0 THEN
        RETURN NULL;
    END IF;

    RETURN agg_embedding::vector;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 5. CONSENSUS METRICS FUNCTIONS
-- ============================================================================

-- Calculate consensus score (agreement ratio)
CREATE OR REPLACE FUNCTION calc_consensus_score(
    p_votes boolean[],
    p_weights float8[] DEFAULT NULL
)
RETURNS float8 AS $$
DECLARE
    total_weight float8;
    agree_weight float8;
BEGIN
    IF p_weights IS NULL THEN
        -- Equal weights
        RETURN (SELECT count(*) FILTER (WHERE v) FROM unnest(p_votes) v)::float8 / array_length(p_votes, 1);
    ELSE
        -- Weighted consensus
        SELECT
            sum(w),
            sum(CASE WHEN v THEN w ELSE 0 END)
        INTO total_weight, agree_weight
        FROM unnest(p_votes, p_weights) AS t(v, w);

        RETURN agree_weight / NULLIF(total_weight, 0);
    END IF;
END;
$$ LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE;

-- Byzantine fault tolerance check
CREATE OR REPLACE FUNCTION check_byzantine_tolerance(
    p_total_nodes int,
    p_faulty_nodes int
)
RETURNS TABLE(
    is_tolerant boolean,
    max_faulty int,
    required_honest int,
    safety_margin float8
) AS $$
BEGIN
    -- BFT requires n >= 3f + 1
    RETURN QUERY
    SELECT
        p_total_nodes >= 3 * p_faulty_nodes + 1,
        (p_total_nodes - 1) / 3,
        2 * ((p_total_nodes - 1) / 3) + 1,
        (p_total_nodes - 3 * p_faulty_nodes - 1)::float8 / p_total_nodes;
END;
$$ LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE;

-- ============================================================================
-- 6. BENCHMARK DATA POPULATION
-- ============================================================================

\echo 'Populating TV5 benchmark data...'

-- Initialize Raft cluster (5 nodes)
INSERT INTO raft_nodes (node_id, state, current_term, capability_vector)
SELECT
    'node-' || i,
    CASE WHEN i = 1 THEN 'leader'::raft_state ELSE 'follower'::raft_state END,
    1,
    (SELECT array_agg(random())::vector(64) FROM generate_series(1, 64))
FROM generate_series(1, 5) i;

-- Populate initial Raft log
INSERT INTO raft_log (term, log_index, command, committed)
SELECT
    1,
    i,
    jsonb_build_object('op', 'set', 'key', 'key-' || i, 'value', random()),
    true
FROM generate_series(1, 1000) i;

-- Populate scale vectors across 8 shards (10K per shard = 80K total)
DO $$
BEGIN
    FOR shard IN 0..7 LOOP
        PERFORM scale_insert_vectors(shard, 10000, 384);
        RAISE NOTICE 'Shard % populated with 10K vectors', shard;
    END LOOP;
END $$;

-- Create HNSW index for vector search
CREATE INDEX scale_vectors_hnsw_idx ON scale_vectors
USING hnsw (embedding vector_cosine_ops) WITH (m = 16, ef_construction = 64);

-- Initialize federated agents (100 agents across 10 shards)
INSERT INTO federated_agents (agent_id, shard_id, embedding, quality, task_count)
SELECT
    'agent-' || i,
    i % 10,
    (SELECT array_agg(random() * 2 - 1)::vector(384) FROM generate_series(1, 384)),
    0.5 + random() * 0.5,  -- Quality between 0.5 and 1.0
    floor(random() * 100)::int
FROM generate_series(1, 100) i;

-- Record baseline consensus metrics
INSERT INTO consensus_metrics (cluster_id, round_number, consensus_score, participant_count, latency_ms, throughput_ops)
SELECT
    'cluster-1',
    i,
    0.7 + random() * 0.3,
    5,
    1 + random() * 10,
    1000 + random() * 9000
FROM generate_series(1, 100) i;

ANALYZE;

\echo ''
\echo '=============================================='
\echo 'TV5 Benchmark Suite Ready'
\echo '=============================================='
\echo ''
