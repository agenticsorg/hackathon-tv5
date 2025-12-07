-- API Keys Table Migration
-- Run this migration to create the api_keys table

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    key_prefix VARCHAR(12) NOT NULL,
    key_hash VARCHAR(64) NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    rate_limit_per_minute INTEGER DEFAULT 60,
    expires_at TIMESTAMP WITH TIME ZONE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    revoked_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(key_prefix)
);

CREATE INDEX idx_api_keys_user ON api_keys(user_id);
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix) WHERE revoked_at IS NULL;

COMMENT ON TABLE api_keys IS 'API keys for programmatic access to Media Gateway';
COMMENT ON COLUMN api_keys.key_prefix IS 'First 12 characters of API key for identification (mg_live_xxxx)';
COMMENT ON COLUMN api_keys.key_hash IS 'SHA-256 hash of full API key';
COMMENT ON COLUMN api_keys.scopes IS 'Array of scopes: read:content, read:recommendations, write:watchlist, write:progress, admin:full';
COMMENT ON COLUMN api_keys.rate_limit_per_minute IS 'Request limit per minute for this key';
COMMENT ON COLUMN api_keys.last_used_at IS 'Last time this key was used for authentication';
