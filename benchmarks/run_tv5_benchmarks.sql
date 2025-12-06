-- TV5 Raft Consensus and Scale Benchmark Execution
-- Tests distributed vector operations, consensus algorithms, and federation

SET search_path TO tv5_bench, public;
SET max_parallel_workers_per_gather = 4;
SET work_mem = '512MB';
SET jit = on;

\echo ''
\echo '=============================================='
\echo 'TV5 BENCHMARK 1: Raft Consensus Operations'
\echo '=============================================='

\echo ''
\echo '1.1 Leader Election Simulation (5 nodes)'
EXPLAIN ANALYZE
WITH election AS (
    -- Simulate vote requests from node-2 as candidate
    SELECT raft_request_vote(2, 'node-2', voter, 1000, 1) as vote_granted
    FROM unnest(ARRAY['node-1', 'node-3', 'node-4', 'node-5']) AS voter
)
SELECT
    count(*) FILTER (WHERE vote_granted) AS votes_for,
    count(*) FILTER (WHERE NOT vote_granted) AS votes_against,
    count(*) FILTER (WHERE vote_granted) >= 3 AS majority_reached
FROM election;

\echo ''
\echo '1.2 Log Replication (1000 entries)'
EXPLAIN ANALYZE
SELECT
    count(*) AS total_entries,
    count(*) FILTER (WHERE committed) AS committed_entries,
    max(log_index) AS latest_index,
    avg(log_index) AS avg_index
FROM raft_log;

\echo ''
\echo '1.3 Append Entries Performance'
EXPLAIN ANALYZE
SELECT raft_append_entries(
    2,
    'node-1',
    1000,
    1,
    ARRAY[
        '{"op": "set", "key": "test-1", "value": 1}'::jsonb,
        '{"op": "set", "key": "test-2", "value": 2}'::jsonb,
        '{"op": "set", "key": "test-3", "value": 3}'::jsonb
    ],
    1003,
    'node-2'
);

\echo ''
\echo '1.4 Cluster State Query'
EXPLAIN ANALYZE
SELECT
    node_id,
    state,
    current_term,
    commit_index,
    now() - last_heartbeat AS time_since_heartbeat
FROM raft_nodes
ORDER BY current_term DESC, node_id;

\echo ''
\echo '=============================================='
\echo 'TV5 BENCHMARK 2: Distributed Scale Operations'
\echo '=============================================='

\echo ''
\echo '2.1 Parallel Vector Count (80K vectors, 8 shards)'
EXPLAIN ANALYZE
SELECT
    shard_id,
    count(*) AS vector_count,
    pg_size_pretty(pg_total_relation_size('scale_vectors_p' || (shard_id % 16))) AS partition_size
FROM scale_vectors
GROUP BY shard_id
ORDER BY shard_id;

\echo ''
\echo '2.2 Cross-Shard Vector Search (top-10)'
EXPLAIN ANALYZE
SELECT * FROM scale_search_vectors(
    (SELECT embedding FROM scale_vectors WHERE shard_id = 0 LIMIT 1),
    10,
    ARRAY[0, 1, 2, 3]  -- Search first 4 shards
);

\echo ''
\echo '2.3 Full Cluster Search (all 8 shards, top-10)'
EXPLAIN ANALYZE
SELECT * FROM scale_search_vectors(
    (SELECT embedding FROM scale_vectors WHERE shard_id = 4 LIMIT 1),
    10,
    NULL  -- All shards
);

\echo ''
\echo '2.4 Partition-Parallel Aggregation'
EXPLAIN ANALYZE
SELECT
    shard_id,
    avg(embedding <=> (SELECT embedding FROM scale_vectors LIMIT 1)) AS avg_distance,
    min(embedding <=> (SELECT embedding FROM scale_vectors LIMIT 1)) AS min_distance,
    max(embedding <=> (SELECT embedding FROM scale_vectors LIMIT 1)) AS max_distance
FROM scale_vectors
GROUP BY shard_id
ORDER BY shard_id;

\echo ''
\echo '=============================================='
\echo 'TV5 BENCHMARK 3: Federated Learning'
\echo '=============================================='

\echo ''
\echo '3.1 Federated Agent Status'
EXPLAIN ANALYZE
SELECT
    shard_id,
    count(*) AS agent_count,
    avg(quality) AS avg_quality,
    sum(task_count) AS total_tasks,
    min(last_sync) AS oldest_sync
FROM federated_agents
GROUP BY shard_id
ORDER BY shard_id;

\echo ''
\echo '3.2 Quality-Filtered Aggregation (FedAvg)'
EXPLAIN ANALYZE
WITH qualified_agents AS (
    SELECT agent_id, embedding, quality
    FROM federated_agents
    WHERE quality >= 0.7
)
SELECT
    count(*) AS qualified_count,
    avg(quality) AS avg_quality,
    (SELECT count(*) FROM federated_agents) AS total_agents
FROM qualified_agents;

\echo ''
\echo '3.3 Agent Similarity Clustering'
EXPLAIN ANALYZE
SELECT
    a.agent_id AS agent_a,
    b.agent_id AS agent_b,
    a.embedding <=> b.embedding AS distance
FROM federated_agents a, federated_agents b
WHERE a.agent_id < b.agent_id
    AND a.shard_id = b.shard_id
    AND a.embedding <=> b.embedding < 0.5
ORDER BY distance
LIMIT 20;

\echo ''
\echo '=============================================='
\echo 'TV5 BENCHMARK 4: Consensus Metrics'
\echo '=============================================='

\echo ''
\echo '4.1 Consensus Score Distribution'
EXPLAIN ANALYZE
SELECT
    width_bucket(consensus_score, 0, 1, 10) AS bucket,
    count(*) AS count,
    avg(consensus_score) AS avg_score,
    avg(latency_ms) AS avg_latency
FROM consensus_metrics
GROUP BY bucket
ORDER BY bucket;

\echo ''
\echo '4.2 Byzantine Fault Tolerance Analysis'
SELECT * FROM check_byzantine_tolerance(5, 1);
SELECT * FROM check_byzantine_tolerance(7, 2);
SELECT * FROM check_byzantine_tolerance(10, 3);

\echo ''
\echo '4.3 Consensus Evolution Over Time'
EXPLAIN ANALYZE
SELECT
    round_number / 10 AS epoch,
    avg(consensus_score) AS avg_consensus,
    avg(latency_ms) AS avg_latency,
    avg(throughput_ops) AS avg_throughput
FROM consensus_metrics
GROUP BY round_number / 10
ORDER BY epoch;

\echo ''
\echo '=============================================='
\echo 'TV5 BENCHMARK 5: Node Capability Routing'
\echo '=============================================='

\echo ''
\echo '5.1 Capability-Based Node Selection'
EXPLAIN ANALYZE
WITH query_capability AS (
    SELECT (SELECT array_agg(random())::vector(64) FROM generate_series(1, 64)) AS cap
)
SELECT
    node_id,
    state,
    capability_vector <=> cap AS capability_distance
FROM raft_nodes, query_capability
WHERE state != 'follower' OR current_term = (SELECT max(current_term) FROM raft_nodes)
ORDER BY capability_distance
LIMIT 3;

\echo ''
\echo '5.2 Load-Balanced Shard Selection'
EXPLAIN ANALYZE
SELECT
    shard_id,
    count(*) AS vector_count,
    count(*)::float8 / sum(count(*)) OVER () AS load_fraction
FROM scale_vectors
GROUP BY shard_id
ORDER BY vector_count ASC
LIMIT 4;

\echo ''
\echo '=============================================='
\echo 'TV5 BENCHMARK 6: Combined Operations'
\echo '=============================================='

\echo ''
\echo '6.1 Consensus + Vector Search (Raft-coordinated query)'
EXPLAIN ANALYZE
WITH leader AS (
    SELECT node_id FROM raft_nodes WHERE state = 'leader' LIMIT 1
),
search_result AS (
    SELECT sv.shard_id, sv.id, sv.embedding <=> (
        SELECT embedding FROM scale_vectors ORDER BY random() LIMIT 1
    ) AS distance
    FROM scale_vectors sv
    ORDER BY distance
    LIMIT 10
)
SELECT
    l.node_id AS coordinator,
    s.shard_id,
    s.id,
    s.distance
FROM leader l, search_result s;

\echo ''
\echo '6.2 Federated Search with Quality Weighting'
EXPLAIN ANALYZE
WITH agent_queries AS (
    SELECT
        fa.agent_id,
        fa.quality,
        (SELECT sv.id FROM scale_vectors sv
         WHERE sv.shard_id = fa.shard_id
         ORDER BY sv.embedding <=> fa.embedding
         LIMIT 1) AS nearest_vector
    FROM federated_agents fa
    WHERE fa.quality >= 0.8
)
SELECT
    agent_id,
    quality,
    nearest_vector
FROM agent_queries
ORDER BY quality DESC
LIMIT 10;

\echo ''
\echo '=============================================='
\echo 'TV5 BENCHMARK SUMMARY'
\echo '=============================================='

SELECT 'raft_nodes' AS table_name, count(*) AS rows,
       pg_size_pretty(pg_total_relation_size('raft_nodes')) AS size
FROM raft_nodes
UNION ALL
SELECT 'raft_log', count(*), pg_size_pretty(pg_total_relation_size('raft_log'))
FROM raft_log
UNION ALL
SELECT 'scale_vectors', count(*), pg_size_pretty(pg_total_relation_size('scale_vectors'))
FROM scale_vectors
UNION ALL
SELECT 'federated_agents', count(*), pg_size_pretty(pg_total_relation_size('federated_agents'))
FROM federated_agents
UNION ALL
SELECT 'consensus_metrics', count(*), pg_size_pretty(pg_total_relation_size('consensus_metrics'))
FROM consensus_metrics;

\echo ''
\echo 'Scale Vector Distribution:'
SELECT shard_id, count(*) AS vectors FROM scale_vectors GROUP BY shard_id ORDER BY shard_id;

\echo ''
\echo 'TV5 Benchmark completed!'
\echo '=============================================='
