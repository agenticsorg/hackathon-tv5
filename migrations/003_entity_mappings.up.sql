-- Entity Resolution Mappings
-- Stores mappings between external IDs and internal entity IDs

CREATE TABLE entity_mappings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    external_id VARCHAR(100) NOT NULL,
    id_type VARCHAR(20) NOT NULL,
    entity_id VARCHAR(100) NOT NULL,
    confidence FLOAT NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(external_id, id_type)
);

CREATE INDEX idx_entity_mappings_external ON entity_mappings(external_id, id_type);
CREATE INDEX idx_entity_mappings_entity ON entity_mappings(entity_id);
CREATE INDEX idx_entity_mappings_type ON entity_mappings(id_type);
