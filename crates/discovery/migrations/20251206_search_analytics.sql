-- Search Analytics Schema
-- Time-series optimized tables for search query tracking and analytics

CREATE TABLE IF NOT EXISTS search_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query_hash VARCHAR(64) NOT NULL,
    query_text VARCHAR(500) NOT NULL,
    user_id_hash VARCHAR(64),
    result_count INTEGER NOT NULL,
    latency_ms INTEGER NOT NULL,
    filters_applied JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS search_clicks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    search_event_id UUID REFERENCES search_events(id) ON DELETE CASCADE,
    content_id UUID NOT NULL,
    position INTEGER NOT NULL,
    clicked_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS popular_searches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query_text VARCHAR(500) NOT NULL,
    period_type VARCHAR(10) NOT NULL,
    period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    search_count INTEGER NOT NULL DEFAULT 0,
    avg_results FLOAT,
    avg_latency_ms FLOAT,
    ctr FLOAT,
    UNIQUE(query_text, period_type, period_start)
);

-- Indexes for time-series queries
CREATE INDEX IF NOT EXISTS idx_search_events_time ON search_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_search_events_query_hash ON search_events(query_hash);
CREATE INDEX IF NOT EXISTS idx_search_events_result_count ON search_events(result_count) WHERE result_count = 0;
CREATE INDEX IF NOT EXISTS idx_search_clicks_event ON search_clicks(search_event_id);
CREATE INDEX IF NOT EXISTS idx_search_clicks_time ON search_clicks(clicked_at DESC);
CREATE INDEX IF NOT EXISTS idx_popular_period ON popular_searches(period_type, period_start DESC);
CREATE INDEX IF NOT EXISTS idx_popular_count ON popular_searches(period_type, search_count DESC);

-- Partitioning for time-series data (optional, for high-volume scenarios)
-- ALTER TABLE search_events SET (autovacuum_vacuum_scale_factor = 0.0);
-- ALTER TABLE search_events SET (autovacuum_vacuum_threshold = 5000);
