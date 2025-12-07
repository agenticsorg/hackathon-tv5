-- TVDB Self-Learning Recommendation System - RuVector PostgreSQL Schema
-- Uses ruvector extension for SIMD-optimized vector operations

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- ruvector is already enabled by default in ruvnet/ruvector-postgres

-- ============================================================================
-- USERS
-- ============================================================================

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE,
    username VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_active_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    settings JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_last_active ON users(last_active_at);

-- ============================================================================
-- CONTENT (Series & Movies from TVDB)
-- ============================================================================

CREATE TABLE IF NOT EXISTS content (
    id VARCHAR(50) PRIMARY KEY, -- TVDB ID
    content_type VARCHAR(20) NOT NULL CHECK (content_type IN ('series', 'movie')),
    title VARCHAR(500) NOT NULL,
    year INTEGER,
    overview TEXT,
    genres TEXT[], -- Array of genre names
    rating DECIMAL(3,1), -- TVDB score
    status VARCHAR(50),
    network_id INTEGER,
    network_name VARCHAR(200),
    original_language VARCHAR(10),
    original_country VARCHAR(10),
    image_url TEXT,
    first_aired DATE,
    last_aired DATE,

    -- Vector embedding for semantic search (using ruvector type)
    embedding ruvector(384), -- 384-dim for all-MiniLM-L6-v2

    -- Metadata
    tvdb_last_updated TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    -- Full text search
    search_vector tsvector GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(overview, '')), 'B') ||
        setweight(to_tsvector('english', coalesce(array_to_string(genres, ' '), '')), 'C')
    ) STORED
);

CREATE INDEX IF NOT EXISTS idx_content_type ON content(content_type);
CREATE INDEX IF NOT EXISTS idx_content_genres ON content USING GIN(genres);
CREATE INDEX IF NOT EXISTS idx_content_year ON content(year);
CREATE INDEX IF NOT EXISTS idx_content_rating ON content(rating DESC);
CREATE INDEX IF NOT EXISTS idx_content_network ON content(network_id);
CREATE INDEX IF NOT EXISTS idx_content_search ON content USING GIN(search_vector);

-- ============================================================================
-- USER PREFERENCES
-- ============================================================================

CREATE TABLE IF NOT EXISTS user_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,

    -- Preference vector (aggregated from watch history)
    preference_vector ruvector(384),

    -- Genre weights (JSON object: genre -> weight 0-1)
    genre_weights JSONB DEFAULT '{}'::jsonb,

    -- Network weights (JSON object: network -> weight 0-1)
    network_weights JSONB DEFAULT '{}'::jsonb,

    -- Preferred content settings
    preferred_content_type VARCHAR(20) DEFAULT 'both'
        CHECK (preferred_content_type IN ('series', 'movie', 'both')),
    preferred_languages TEXT[] DEFAULT ARRAY['en'],

    -- Content rating preferences (parental controls)
    max_content_rating VARCHAR(20),
    excluded_genres TEXT[] DEFAULT ARRAY[]::TEXT[],

    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================================================
-- WATCH HISTORY
-- ============================================================================

CREATE TABLE IF NOT EXISTS watch_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content_id VARCHAR(50) NOT NULL REFERENCES content(id) ON DELETE CASCADE,

    -- Episode tracking (for series)
    episode_id INTEGER,
    season_number INTEGER,
    episode_number INTEGER,

    -- Watch progress
    watched_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    duration_seconds INTEGER NOT NULL, -- How long they watched
    completion_percentage DECIMAL(5,2) DEFAULT 0, -- 0-100

    -- Context
    platform VARCHAR(50), -- web, mobile, tv
    device_type VARCHAR(50),

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_watch_history_user ON watch_history(user_id);
CREATE INDEX IF NOT EXISTS idx_watch_history_content ON watch_history(content_id);
CREATE INDEX IF NOT EXISTS idx_watch_history_user_time ON watch_history(user_id, watched_at DESC);
CREATE INDEX IF NOT EXISTS idx_watch_history_recent ON watch_history(watched_at DESC);

-- ============================================================================
-- RATINGS
-- ============================================================================

CREATE TABLE IF NOT EXISTS ratings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content_id VARCHAR(50) NOT NULL REFERENCES content(id) ON DELETE CASCADE,
    rating DECIMAL(3,1) NOT NULL CHECK (rating >= 0 AND rating <= 10),
    rated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    UNIQUE(user_id, content_id)
);

CREATE INDEX IF NOT EXISTS idx_ratings_user ON ratings(user_id);
CREATE INDEX IF NOT EXISTS idx_ratings_content ON ratings(content_id);
CREATE INDEX IF NOT EXISTS idx_ratings_high ON ratings(rating DESC);

-- ============================================================================
-- USER INTERACTIONS
-- ============================================================================

CREATE TABLE IF NOT EXISTS user_interactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content_id VARCHAR(50) REFERENCES content(id) ON DELETE SET NULL,

    interaction_type VARCHAR(50) NOT NULL CHECK (interaction_type IN (
        'view', 'click', 'search', 'add_watchlist', 'remove_watchlist', 'share', 'dismiss'
    )),

    -- Context
    source VARCHAR(100), -- recommendation, search, browse, etc.
    position INTEGER, -- Position in list when clicked
    session_id UUID,

    -- Additional metadata
    metadata JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_interactions_user ON user_interactions(user_id);
CREATE INDEX IF NOT EXISTS idx_interactions_content ON user_interactions(content_id);
CREATE INDEX IF NOT EXISTS idx_interactions_type ON user_interactions(interaction_type);
CREATE INDEX IF NOT EXISTS idx_interactions_time ON user_interactions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_interactions_session ON user_interactions(session_id);

-- ============================================================================
-- WATCHLIST
-- ============================================================================

CREATE TABLE IF NOT EXISTS watchlist (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content_id VARCHAR(50) NOT NULL REFERENCES content(id) ON DELETE CASCADE,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    priority INTEGER DEFAULT 0, -- User-defined ordering
    notes TEXT,

    UNIQUE(user_id, content_id)
);

CREATE INDEX IF NOT EXISTS idx_watchlist_user ON watchlist(user_id);
CREATE INDEX IF NOT EXISTS idx_watchlist_user_priority ON watchlist(user_id, priority DESC, added_at DESC);

-- ============================================================================
-- RECOMMENDATION PATTERNS (Learning from AgentDB)
-- ============================================================================

CREATE TABLE IF NOT EXISTS recommendation_patterns (
    id SERIAL PRIMARY KEY,
    pattern_type VARCHAR(50) NOT NULL,
    approach TEXT NOT NULL,

    -- Context matching
    user_segment VARCHAR(20), -- new, casual, regular, power
    time_of_day VARCHAR(20),
    day_of_week VARCHAR(20),
    platform VARCHAR(50),
    content_type_preference VARCHAR(20),

    -- Performance metrics
    success_rate DECIMAL(5,4) DEFAULT 0.5,
    total_uses INTEGER DEFAULT 0,
    avg_reward DECIMAL(5,4) DEFAULT 0,

    -- Pattern embedding for similarity search
    embedding ruvector(384),

    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX IF NOT EXISTS idx_patterns_type ON recommendation_patterns(pattern_type);
CREATE INDEX IF NOT EXISTS idx_patterns_success ON recommendation_patterns(success_rate DESC);

-- ============================================================================
-- LEARNING FEEDBACK
-- ============================================================================

CREATE TABLE IF NOT EXISTS learning_feedback (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content_id VARCHAR(50) REFERENCES content(id) ON DELETE SET NULL,
    pattern_id INTEGER REFERENCES recommendation_patterns(id) ON DELETE SET NULL,

    -- Outcome
    was_successful BOOLEAN NOT NULL,
    reward DECIMAL(5,4) NOT NULL, -- -1 to 1
    user_action VARCHAR(50) NOT NULL, -- watched, skipped, rated, etc.

    -- Context at time of recommendation
    recommendation_position INTEGER,
    recommendation_source VARCHAR(100),

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_feedback_user ON learning_feedback(user_id);
CREATE INDEX IF NOT EXISTS idx_feedback_pattern ON learning_feedback(pattern_id);
CREATE INDEX IF NOT EXISTS idx_feedback_time ON learning_feedback(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_feedback_success ON learning_feedback(was_successful);

-- ============================================================================
-- REFLEXION EPISODES (For self-critique learning)
-- ============================================================================

CREATE TABLE IF NOT EXISTS reflexion_episodes (
    id SERIAL PRIMARY KEY,
    context TEXT NOT NULL,
    action TEXT NOT NULL,
    outcome TEXT NOT NULL,
    reward DECIMAL(5,4) NOT NULL,
    self_critique TEXT,

    -- Embedding for similarity search
    embedding ruvector(384),

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_reflexion_reward ON reflexion_episodes(reward DESC);

-- ============================================================================
-- SYNC TRACKING (For TVDB updates)
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync_status (
    id SERIAL PRIMARY KEY,
    sync_type VARCHAR(50) NOT NULL, -- series, movies, episodes
    last_sync_timestamp BIGINT NOT NULL, -- Unix timestamp from TVDB
    items_synced INTEGER DEFAULT 0,
    status VARCHAR(20) DEFAULT 'completed',
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sync_type ON sync_status(sync_type, created_at DESC);

-- ============================================================================
-- HELPER FUNCTIONS (Using RuVector)
-- ============================================================================

-- Function to find similar content by vector using ruvector_cosine_distance
CREATE OR REPLACE FUNCTION find_similar_content(
    p_content_id VARCHAR(50),
    p_limit INTEGER DEFAULT 10,
    p_threshold DECIMAL DEFAULT 0.5
)
RETURNS TABLE (
    content_id VARCHAR(50),
    title VARCHAR(500),
    similarity DECIMAL
) AS $$
DECLARE
    query_embedding ruvector(384);
BEGIN
    -- Get the embedding for the query content
    SELECT embedding INTO query_embedding FROM content WHERE id = p_content_id;

    IF query_embedding IS NULL THEN
        RETURN;
    END IF;

    RETURN QUERY
    SELECT
        c.id,
        c.title,
        (1 - ruvector_cosine_distance(c.embedding, query_embedding))::DECIMAL as similarity
    FROM content c
    WHERE c.id != p_content_id
    AND c.embedding IS NOT NULL
    AND (1 - ruvector_cosine_distance(c.embedding, query_embedding)) >= p_threshold
    ORDER BY ruvector_cosine_distance(c.embedding, query_embedding)
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- Function to get personalized recommendations using ruvector
CREATE OR REPLACE FUNCTION get_personalized_recommendations(
    p_user_id UUID,
    p_limit INTEGER DEFAULT 20,
    p_content_type VARCHAR(20) DEFAULT NULL
)
RETURNS TABLE (
    content_id VARCHAR(50),
    title VARCHAR(500),
    content_type VARCHAR(20),
    similarity DECIMAL,
    genres TEXT[]
) AS $$
DECLARE
    v_pref_vector ruvector(384);
BEGIN
    -- Get user preference vector
    SELECT preference_vector INTO v_pref_vector
    FROM user_preferences
    WHERE user_id = p_user_id;

    IF v_pref_vector IS NULL THEN
        -- Cold start: return popular content
        RETURN QUERY
        SELECT
            c.id,
            c.title,
            c.content_type,
            c.rating::DECIMAL,
            c.genres
        FROM content c
        WHERE (p_content_type IS NULL OR c.content_type = p_content_type)
        ORDER BY c.rating DESC NULLS LAST
        LIMIT p_limit;
    ELSE
        -- Personalized recommendations using ruvector distance
        RETURN QUERY
        SELECT
            c.id,
            c.title,
            c.content_type,
            (1 - ruvector_cosine_distance(c.embedding, v_pref_vector))::DECIMAL as similarity,
            c.genres
        FROM content c
        WHERE c.embedding IS NOT NULL
        AND (p_content_type IS NULL OR c.content_type = p_content_type)
        AND c.id NOT IN (
            SELECT wh.content_id
            FROM watch_history wh
            WHERE wh.user_id = p_user_id
        )
        ORDER BY ruvector_cosine_distance(c.embedding, v_pref_vector)
        LIMIT p_limit;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Enable self-learning on content table
SELECT ruvector_enable_learning('content', '{"algorithm": "q_learning", "reward_decay": 0.95}'::jsonb);

-- ============================================================================
-- INITIAL DATA
-- ============================================================================

-- Insert default recommendation patterns
INSERT INTO recommendation_patterns (pattern_type, approach, user_segment, success_rate) VALUES
('cold_start', 'Show popular and trending content for new users', 'new', 0.6),
('genre_match', 'Match user top genres for regular users', 'regular', 0.75),
('similar_content', 'Use vector similarity for personalized recommendations', 'power', 0.8),
('time_based', 'Recommend series in evening, movies in morning', 'casual', 0.65),
('network_based', 'Recommend from user preferred networks', 'regular', 0.7)
ON CONFLICT DO NOTHING;

-- Show table summary
SELECT 'Schema initialized successfully' as status;
SELECT table_name, pg_size_pretty(pg_total_relation_size(quote_ident(table_name))) as size
FROM information_schema.tables
WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
ORDER BY table_name;
