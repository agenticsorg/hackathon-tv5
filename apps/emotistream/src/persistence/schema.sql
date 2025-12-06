-- EmotiStream PostgreSQL Schema
-- This schema supports the RL-based content recommendation system
-- Uses ruvector extension for SIMD-optimized emotion vector operations

-- Enable ruvector extension (provides vector similarity, temporal compression, attention mechanisms)
CREATE EXTENSION IF NOT EXISTS ruvector;

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100),
    date_of_birth DATE,
    preferences JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for email lookups
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Emotion analyses table
CREATE TABLE IF NOT EXISTS emotion_analyses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    input_text TEXT NOT NULL,
    valence REAL NOT NULL CHECK (valence >= -1 AND valence <= 1),
    arousal REAL NOT NULL CHECK (arousal >= -1 AND arousal <= 1),
    stress_level REAL NOT NULL CHECK (stress_level >= 0 AND stress_level <= 1),
    primary_emotion VARCHAR(50) NOT NULL,
    confidence REAL NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    emotion_vector REAL[] DEFAULT ARRAY[]::REAL[],
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_emotion_user ON emotion_analyses(user_id);
CREATE INDEX IF NOT EXISTS idx_emotion_created ON emotion_analyses(created_at DESC);

-- Emotion embeddings table (using ruvector for fast similarity search)
-- Stores 8-dimensional emotion vectors for content and user states
CREATE TABLE IF NOT EXISTS emotion_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(20) NOT NULL CHECK (entity_type IN ('user_state', 'content', 'desired_state')),
    entity_id VARCHAR(255) NOT NULL,
    embedding ruvector(8),  -- 8-dim emotion vector: [joy, sadness, anger, fear, surprise, disgust, trust, anticipation]
    valence REAL,
    arousal REAL,
    stress REAL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_embeddings_entity ON emotion_embeddings(entity_type, entity_id);

-- User emotion history with EMA tracking (temporal patterns)
CREATE TABLE IF NOT EXISTS user_emotion_ema (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    ema_embedding ruvector(8),  -- Exponential moving average of user's emotional state
    alpha REAL DEFAULT 0.3,     -- EMA smoothing factor
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id)
);

-- Content catalog table
CREATE TABLE IF NOT EXISTS content (
    id VARCHAR(100) PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL,
    duration_minutes INTEGER DEFAULT 30,
    emotional_profile JSONB DEFAULT '{}',
    tags TEXT[] DEFAULT ARRAY[]::TEXT[],
    thumbnail_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_content_category ON content(category);

-- Feedback/experiences table
-- Note: Foreign keys relaxed for MVP to allow flexible content sources (TMDB, mock)
CREATE TABLE IF NOT EXISTS feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    content_id VARCHAR(255) NOT NULL,
    content_title VARCHAR(500),
    session_id VARCHAR(255),

    -- Emotion before watching
    emotion_before_valence REAL,
    emotion_before_arousal REAL,
    emotion_before_stress REAL,
    emotion_before_primary VARCHAR(50),
    emotion_before_confidence REAL,

    -- Emotion after watching
    emotion_after_valence REAL,
    emotion_after_arousal REAL,
    emotion_after_stress REAL,
    emotion_after_primary VARCHAR(50),
    emotion_after_confidence REAL,

    -- Feedback metrics
    star_rating INTEGER CHECK (star_rating >= 1 AND star_rating <= 5),
    completed BOOLEAN DEFAULT FALSE,
    watch_duration_ms INTEGER DEFAULT 0,
    total_duration_ms INTEGER DEFAULT 0,

    -- RL metrics
    reward REAL,
    q_value_before REAL,
    q_value_after REAL,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_feedback_user ON feedback(user_id);
CREATE INDEX IF NOT EXISTS idx_feedback_content ON feedback(content_id);
CREATE INDEX IF NOT EXISTS idx_feedback_created ON feedback(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_feedback_user_content ON feedback(user_id, content_id);

-- Q-values table (RL policy storage)
-- Note: Foreign keys relaxed for MVP
CREATE TABLE IF NOT EXISTS q_values (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    state_key VARCHAR(255) NOT NULL,
    content_id VARCHAR(255) NOT NULL,
    q_value REAL NOT NULL DEFAULT 0,
    visit_count INTEGER DEFAULT 0,
    last_updated TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(user_id, state_key, content_id)
);

CREATE INDEX IF NOT EXISTS idx_qvalues_user_state ON q_values(user_id, state_key);

-- Watch sessions table
-- Note: Foreign keys relaxed for MVP
CREATE TABLE IF NOT EXISTS watch_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    content_id VARCHAR(255) NOT NULL,

    -- State before viewing
    state_before JSONB,
    desired_state JSONB,

    -- Session tracking
    started_at TIMESTAMPTZ DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    watch_duration_ms INTEGER DEFAULT 0,

    -- Status
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'completed', 'abandoned'))
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON watch_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON watch_sessions(user_id, status) WHERE status = 'active';

-- Insert default content catalog
INSERT INTO content (id, title, description, category, duration_minutes, emotional_profile, tags) VALUES
    ('meditation-calm', 'Guided Meditation: Finding Calm', 'A gentle meditation for relaxation', 'meditation', 15, '{"targetValence": 0.3, "targetArousal": -0.5, "targetStress": 0.1}', ARRAY['relaxation', 'calm', 'meditation']),
    ('comedy-standup', 'Stand-Up Comedy Hour', 'Hilarious stand-up performances', 'comedy', 60, '{"targetValence": 0.8, "targetArousal": 0.4, "targetStress": 0.2}', ARRAY['comedy', 'laughter', 'entertainment']),
    ('nature-doc', 'Planet Earth: Ocean Deep', 'Stunning ocean documentary', 'documentary', 45, '{"targetValence": 0.5, "targetArousal": 0.1, "targetStress": 0.15}', ARRAY['nature', 'documentary', 'ocean']),
    ('action-thriller', 'Edge of Tomorrow', 'High-octane action adventure', 'action', 120, '{"targetValence": 0.6, "targetArousal": 0.8, "targetStress": 0.4}', ARRAY['action', 'thriller', 'adventure']),
    ('drama-inspiring', 'The Pursuit of Happyness', 'Inspiring true story of perseverance', 'drama', 117, '{"targetValence": 0.7, "targetArousal": 0.3, "targetStress": 0.3}', ARRAY['drama', 'inspiring', 'biography']),
    ('music-relaxing', 'Classical Piano Collection', 'Soothing piano performances', 'music', 90, '{"targetValence": 0.4, "targetArousal": -0.4, "targetStress": 0.05}', ARRAY['music', 'classical', 'relaxing']),
    ('yoga-morning', 'Morning Yoga Flow', 'Energizing morning yoga routine', 'fitness', 30, '{"targetValence": 0.5, "targetArousal": 0.2, "targetStress": 0.1}', ARRAY['yoga', 'fitness', 'morning']),
    ('cooking-comfort', 'Comfort Food Recipes', 'Easy comfort food cooking show', 'cooking', 45, '{"targetValence": 0.6, "targetArousal": 0.1, "targetStress": 0.15}', ARRAY['cooking', 'food', 'comfort']),
    ('travel-adventure', 'Adventures in Japan', 'Cultural travel exploration', 'travel', 50, '{"targetValence": 0.7, "targetArousal": 0.5, "targetStress": 0.2}', ARRAY['travel', 'culture', 'adventure']),
    ('scifi-epic', 'Interstellar Journey', 'Mind-bending space epic', 'scifi', 150, '{"targetValence": 0.5, "targetArousal": 0.6, "targetStress": 0.35}', ARRAY['scifi', 'space', 'epic'])
ON CONFLICT (id) DO NOTHING;

-- Function to update timestamps
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for users table
DROP TRIGGER IF EXISTS users_updated_at ON users;
CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();
