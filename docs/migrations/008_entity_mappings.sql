-- Entity Resolution Mappings Migration
-- Migration: 008_entity_mappings
-- Purpose: Add entity_mappings table for persistent cross-platform content resolution

-- Create entity_mappings table
CREATE TABLE IF NOT EXISTS entity_mappings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    external_id VARCHAR(100) NOT NULL,
    id_type VARCHAR(20) NOT NULL,
    entity_id VARCHAR(100) NOT NULL,
    confidence FLOAT NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(external_id, id_type)
);

-- Create indexes for optimized lookups
CREATE INDEX IF NOT EXISTS idx_entity_mappings_external ON entity_mappings(external_id, id_type);
CREATE INDEX IF NOT EXISTS idx_entity_mappings_entity ON entity_mappings(entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_mappings_type ON entity_mappings(id_type);

-- Add comment
COMMENT ON TABLE entity_mappings IS 'Stores mappings between external IDs (EIDR, IMDB, TMDB) and internal entity IDs for cross-platform content resolution';
