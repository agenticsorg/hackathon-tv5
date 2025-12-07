-- Media Gateway PostgreSQL Schema
-- Version: 1.0.0

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    external_auth_id VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(100),
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_premium BOOLEAN DEFAULT FALSE,
    subscription_tier VARCHAR(20) DEFAULT 'free'
);

-- User preferences
CREATE TABLE user_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    favorite_genres TEXT[] DEFAULT '{}',
    disliked_genres TEXT[] DEFAULT '{}',
    preferred_languages TEXT[] DEFAULT '{en}',
    subscribed_platforms TEXT[] DEFAULT '{}',
    max_content_rating VARCHAR(10),
    preferred_video_quality VARCHAR(10) DEFAULT 'HD',
    autoplay_next BOOLEAN DEFAULT TRUE,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Content table
CREATE TABLE content (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content_type VARCHAR(20) NOT NULL,
    title VARCHAR(500) NOT NULL,
    original_title VARCHAR(500),
    overview TEXT,
    tagline VARCHAR(500),
    release_date DATE,
    premiere_date DATE,
    runtime_minutes INTEGER,
    popularity_score FLOAT DEFAULT 0.5,
    average_rating FLOAT DEFAULT 0.0,
    vote_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_updated TIMESTAMPTZ DEFAULT NOW()
);

-- External IDs
CREATE TABLE external_ids (
    content_id UUID PRIMARY KEY REFERENCES content(id) ON DELETE CASCADE,
    eidr_id VARCHAR(50),
    imdb_id VARCHAR(20),
    tmdb_id INTEGER,
    tvdb_id INTEGER,
    gracenote_tms_id VARCHAR(30)
);

-- Platform-specific IDs
CREATE TABLE platform_ids (
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    platform VARCHAR(20) NOT NULL,
    platform_content_id VARCHAR(100) NOT NULL,
    PRIMARY KEY (content_id, platform)
);

-- Content genres
CREATE TABLE content_genres (
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    genre VARCHAR(50) NOT NULL,
    PRIMARY KEY (content_id, genre)
);

-- Content themes and moods
CREATE TABLE content_themes (
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    theme VARCHAR(100) NOT NULL,
    PRIMARY KEY (content_id, theme)
);

CREATE TABLE content_moods (
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    mood VARCHAR(50) NOT NULL,
    PRIMARY KEY (content_id, mood)
);

-- Credits (cast and crew)
CREATE TABLE credits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    person_name VARCHAR(200) NOT NULL,
    role_type VARCHAR(20) NOT NULL, -- actor, director, writer, producer
    character_name VARCHAR(200),
    order_index INTEGER DEFAULT 0
);

-- Content ratings by region
CREATE TABLE content_ratings (
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    region VARCHAR(5) NOT NULL,
    rating VARCHAR(20) NOT NULL,
    advisory_notes TEXT,
    PRIMARY KEY (content_id, region)
);

-- Platform availability
CREATE TABLE platform_availability (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    platform VARCHAR(20) NOT NULL,
    region VARCHAR(5) NOT NULL,
    availability_type VARCHAR(20) NOT NULL,
    price_cents INTEGER,
    currency VARCHAR(3),
    deep_link TEXT NOT NULL,
    web_fallback TEXT NOT NULL,
    video_qualities TEXT[] DEFAULT '{}',
    audio_tracks TEXT[] DEFAULT '{}',
    subtitle_tracks TEXT[] DEFAULT '{}',
    available_from TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ
);

-- Series metadata
CREATE TABLE series_metadata (
    content_id UUID PRIMARY KEY REFERENCES content(id) ON DELETE CASCADE,
    season_count INTEGER,
    episode_count INTEGER,
    status VARCHAR(20),
    network VARCHAR(100),
    air_day VARCHAR(20),
    air_time TIME
);

-- User watchlist (CRDT-compatible)
CREATE TABLE user_watchlist (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    hlc_timestamp BIGINT NOT NULL,
    operation_tag UUID NOT NULL UNIQUE,
    is_removed BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (user_id, content_id)
);

-- Watch progress (LWW Register)
CREATE TABLE watch_progress (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    content_id UUID REFERENCES content(id) ON DELETE CASCADE,
    position_seconds INTEGER NOT NULL DEFAULT 0,
    duration_seconds INTEGER,
    completion_rate FLOAT DEFAULT 0.0,
    last_watched TIMESTAMPTZ DEFAULT NOW(),
    hlc_timestamp BIGINT NOT NULL,
    device_id UUID,
    PRIMARY KEY (user_id, content_id)
);

-- User devices
CREATE TABLE user_devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    device_name VARCHAR(100) NOT NULL,
    device_type VARCHAR(20) NOT NULL,
    platform VARCHAR(50),
    capabilities TEXT[] DEFAULT '{}',
    push_token TEXT,
    last_seen TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

-- OAuth tokens
CREATE TABLE oauth_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(20) NOT NULL,
    access_token_hash VARCHAR(64) NOT NULL,
    refresh_token_hash VARCHAR(64),
    expires_at TIMESTAMPTZ,
    scopes TEXT[] DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Refresh tokens
CREATE TABLE refresh_tokens (
    token_hash VARCHAR(64) PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    device_id UUID REFERENCES user_devices(id),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

-- Rate limit tracking
CREATE TABLE rate_limits (
    key VARCHAR(255) PRIMARY KEY,
    count INTEGER DEFAULT 0,
    window_start TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_content_type ON content(content_type);
CREATE INDEX idx_content_release_date ON content(release_date);
CREATE INDEX idx_content_popularity ON content(popularity_score DESC);
CREATE INDEX idx_content_title_trgm ON content USING gin(title gin_trgm_ops);
CREATE INDEX idx_external_ids_imdb ON external_ids(imdb_id);
CREATE INDEX idx_external_ids_tmdb ON external_ids(tmdb_id);
CREATE INDEX idx_platform_avail_content ON platform_availability(content_id);
CREATE INDEX idx_platform_avail_platform_region ON platform_availability(platform, region);
CREATE INDEX idx_watchlist_user ON user_watchlist(user_id) WHERE NOT is_removed;
CREATE INDEX idx_watch_progress_user ON watch_progress(user_id);
CREATE INDEX idx_credits_content ON credits(content_id);
CREATE INDEX idx_users_email ON users(email);
