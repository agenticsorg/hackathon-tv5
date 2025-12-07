-- Migration: Add LoRA Adapters table
-- Description: Storage for UserLoRAAdapter models with versioning support
-- Target: <2ms retrieval latency with proper indexing

-- LoRA Adapters table
CREATE TABLE lora_adapters (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    adapter_name VARCHAR(100) NOT NULL DEFAULT 'default',
    version INTEGER NOT NULL DEFAULT 1,
    weights BYTEA NOT NULL,
    size_bytes BIGINT NOT NULL,
    training_iterations INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, adapter_name, version)
);

-- Indexes for <2ms retrieval latency
-- Primary lookup: get latest adapter for user
CREATE INDEX idx_lora_adapters_user_name_version
ON lora_adapters(user_id, adapter_name, version DESC);

-- Fast user lookup for list operations
CREATE INDEX idx_lora_adapters_user_updated
ON lora_adapters(user_id, updated_at DESC);

-- Statistics and monitoring
CREATE INDEX idx_lora_adapters_created
ON lora_adapters(created_at DESC);

-- Comments for documentation
COMMENT ON TABLE lora_adapters IS 'Storage for user-specific LoRA adapter models with versioning';
COMMENT ON COLUMN lora_adapters.user_id IS 'Reference to users table';
COMMENT ON COLUMN lora_adapters.adapter_name IS 'Name/identifier for adapter (e.g., "default", "experimental")';
COMMENT ON COLUMN lora_adapters.version IS 'Auto-incrementing version number per (user_id, adapter_name)';
COMMENT ON COLUMN lora_adapters.weights IS 'Bincode-serialized UserLoRAAdapter (~40KB for rank=8)';
COMMENT ON COLUMN lora_adapters.size_bytes IS 'Size of weights blob in bytes';
COMMENT ON COLUMN lora_adapters.training_iterations IS 'Number of training iterations completed';

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_lora_adapters_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_lora_adapters_updated_at
    BEFORE UPDATE ON lora_adapters
    FOR EACH ROW
    EXECUTE FUNCTION update_lora_adapters_updated_at();
