-- Initialize Omega Database
-- This script runs on first PostgreSQL startup

-- Enable RuVector extension
CREATE EXTENSION IF NOT EXISTS ruvector;

-- Create patterns table
CREATE TABLE IF NOT EXISTS patterns (
    id UUID PRIMARY KEY,
    device_id VARCHAR(255) NOT NULL,
    embedding ruvector(384) NOT NULL,
    success_rate REAL NOT NULL,
    sample_count INTEGER NOT NULL DEFAULT 1,
    context JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create HNSW index on embeddings for fast similarity search
-- TODO: Re-enable when RuVector HNSW support is available
-- CREATE INDEX IF NOT EXISTS patterns_embedding_idx
-- ON patterns
-- USING hnsw (embedding ruvector_cosine_ops)
-- WITH (m = 32, ef_construction = 200);

-- Create index on device_id for fast device queries
CREATE INDEX IF NOT EXISTS patterns_device_id_idx
ON patterns (device_id);

-- Create index on success_rate for quality filtering
CREATE INDEX IF NOT EXISTS patterns_success_rate_idx
ON patterns (success_rate);

-- Create partial index for high-quality patterns
CREATE INDEX IF NOT EXISTS patterns_high_quality_idx
ON patterns (success_rate)
WHERE success_rate >= 0.8;

-- Create devices table for tracking device state
CREATE TABLE IF NOT EXISTS devices (
    device_id VARCHAR(255) PRIMARY KEY,
    shard_id INTEGER NOT NULL,
    last_sync_at TIMESTAMP WITH TIME ZONE,
    last_sync_version BIGINT DEFAULT 0,
    pattern_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create index on shard_id
CREATE INDEX IF NOT EXISTS devices_shard_id_idx
ON devices (shard_id);

-- Create content embeddings table
CREATE TABLE IF NOT EXISTS content_embeddings (
    content_id VARCHAR(255) PRIMARY KEY,
    title TEXT NOT NULL,
    embedding ruvector(384) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create HNSW index on content embeddings
-- TODO: Re-enable when RuVector HNSW support is available
-- CREATE INDEX IF NOT EXISTS content_embeddings_idx
-- ON content_embeddings
-- USING hnsw (embedding ruvector_cosine_ops)
-- WITH (m = 32, ef_construction = 200);

-- Create trending table for global trends
CREATE TABLE IF NOT EXISTS trending_content (
    content_id VARCHAR(255) PRIMARY KEY,
    trending_score REAL NOT NULL,
    region VARCHAR(50),
    time_window TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create index on trending_score
CREATE INDEX IF NOT EXISTS trending_score_idx
ON trending_content (trending_score DESC);

-- Create index on time_window
CREATE INDEX IF NOT EXISTS trending_time_window_idx
ON trending_content (time_window DESC);

-- Create federation state table
CREATE TABLE IF NOT EXISTS federation_state (
    id SERIAL PRIMARY KEY,
    round_number BIGINT NOT NULL,
    patterns_aggregated INTEGER NOT NULL,
    trends_detected INTEGER NOT NULL,
    quality_avg REAL NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) NOT NULL
);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for updated_at
CREATE TRIGGER update_patterns_updated_at
BEFORE UPDATE ON patterns
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_devices_updated_at
BEFORE UPDATE ON devices
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Create function for pattern quality scoring
CREATE OR REPLACE FUNCTION calculate_pattern_quality(
    p_success_rate REAL,
    p_sample_count INTEGER
)
RETURNS REAL AS $$
BEGIN
    -- Weighted quality: success_rate * log(sample_count + 1)
    RETURN p_success_rate * LOG(p_sample_count + 1);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO omega;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO omega;

-- Insert sample content (optional, for testing)
-- TODO: Fix ruvector casting syntax before enabling
-- INSERT INTO content_embeddings (content_id, title, embedding, metadata)
-- VALUES
--     ('content-1', 'Sample Movie 1', array_fill(0.5::real, ARRAY[384])::ruvector, '{"genre": "action", "year": 2024}'::jsonb),
--     ('content-2', 'Sample Series 1', array_fill(0.3::real, ARRAY[384])::ruvector, '{"genre": "drama", "year": 2024}'::jsonb)
-- ON CONFLICT (content_id) DO NOTHING;

-- Create view for high-quality patterns
CREATE OR REPLACE VIEW high_quality_patterns AS
SELECT
    id,
    device_id,
    embedding,
    success_rate,
    sample_count,
    calculate_pattern_quality(success_rate, sample_count) as quality_score,
    context,
    created_at
FROM patterns
WHERE success_rate >= 0.8
ORDER BY quality_score DESC;

-- Create view for device statistics
CREATE OR REPLACE VIEW device_stats AS
SELECT
    d.device_id,
    d.shard_id,
    d.pattern_count,
    COUNT(p.id) as actual_pattern_count,
    AVG(p.success_rate) as avg_success_rate,
    MAX(p.created_at) as last_pattern_at
FROM devices d
LEFT JOIN patterns p ON d.device_id = p.device_id
GROUP BY d.device_id, d.shard_id, d.pattern_count;

-- Vacuum and analyze for optimization
VACUUM ANALYZE;
