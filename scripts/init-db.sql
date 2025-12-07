-- Media Gateway Database Initialization Script
-- This script runs automatically when the PostgreSQL container starts

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create schemas
CREATE SCHEMA IF NOT EXISTS content;
CREATE SCHEMA IF NOT EXISTS users;
CREATE SCHEMA IF NOT EXISTS sync;

-- Content tables
CREATE TABLE IF NOT EXISTS content.items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity_id VARCHAR(255),
    title VARCHAR(500) NOT NULL,
    overview TEXT,
    release_year INTEGER,
    content_type VARCHAR(50) NOT NULL,
    genres TEXT[],
    platforms TEXT[],
    popularity_score FLOAT DEFAULT 0.0,
    embedding FLOAT[],
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS content.availability (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content_id UUID REFERENCES content.items(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,
    region VARCHAR(10) NOT NULL,
    available BOOLEAN DEFAULT true,
    subscription_required BOOLEAN DEFAULT false,
    rental_price DECIMAL(10,2),
    purchase_price DECIMAL(10,2),
    expires_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(content_id, platform, region)
);

-- User tables
CREATE TABLE IF NOT EXISTS users.profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(255),
    avatar_url TEXT,
    subscription_tier VARCHAR(50) DEFAULT 'free',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS users.preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users.profiles(id) ON DELETE CASCADE,
    genre_weights JSONB DEFAULT '{}',
    platform_preferences TEXT[],
    watch_history_size INTEGER DEFAULT 0,
    embedding FLOAT[],
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id)
);

CREATE TABLE IF NOT EXISTS users.interactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users.profiles(id) ON DELETE CASCADE,
    content_id UUID REFERENCES content.items(id) ON DELETE CASCADE,
    interaction_type VARCHAR(50) NOT NULL,
    rating FLOAT,
    watch_progress FLOAT,
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Sync tables
CREATE TABLE IF NOT EXISTS sync.watchlists (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users.profiles(id) ON DELETE CASCADE,
    content_id UUID REFERENCES content.items(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ DEFAULT NOW(),
    hlc_timestamp BIGINT NOT NULL,
    device_id VARCHAR(255),
    UNIQUE(user_id, content_id)
);

CREATE TABLE IF NOT EXISTS sync.playback_positions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users.profiles(id) ON DELETE CASCADE,
    content_id UUID REFERENCES content.items(id) ON DELETE CASCADE,
    position_seconds INTEGER DEFAULT 0,
    duration_seconds INTEGER,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    hlc_timestamp BIGINT NOT NULL,
    device_id VARCHAR(255),
    UNIQUE(user_id, content_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_content_items_entity ON content.items(entity_id);
CREATE INDEX IF NOT EXISTS idx_content_items_type ON content.items(content_type);
CREATE INDEX IF NOT EXISTS idx_content_items_popularity ON content.items(popularity_score DESC);
CREATE INDEX IF NOT EXISTS idx_availability_platform ON content.availability(platform, region);
CREATE INDEX IF NOT EXISTS idx_availability_expires ON content.availability(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_email ON users.profiles(email);
CREATE INDEX IF NOT EXISTS idx_interactions_user ON users.interactions(user_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_watchlist_user ON sync.watchlists(user_id, added_at DESC);
CREATE INDEX IF NOT EXISTS idx_playback_user ON sync.playback_positions(user_id);

-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA content TO mediagateway;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA users TO mediagateway;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA sync TO mediagateway;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA content TO mediagateway;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA users TO mediagateway;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA sync TO mediagateway;
