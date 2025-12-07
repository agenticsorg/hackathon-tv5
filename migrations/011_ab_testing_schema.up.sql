-- A/B Testing Framework Database Schema
-- SONA Engine - Media Gateway
--
-- Run this migration to enable A/B testing functionality
-- Dependencies: None (standalone schema)

-- ============================================================================
-- Experiments Table
-- ============================================================================
-- Stores experiment configuration and lifecycle

CREATE TABLE IF NOT EXISTS experiments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    traffic_allocation FLOAT NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    CONSTRAINT valid_status CHECK (status IN ('draft', 'running', 'paused', 'completed')),
    CONSTRAINT valid_traffic CHECK (traffic_allocation >= 0.0 AND traffic_allocation <= 1.0)
);

CREATE INDEX idx_experiments_status ON experiments(status);
CREATE INDEX idx_experiments_created_at ON experiments(created_at);

COMMENT ON TABLE experiments IS 'A/B testing experiments with lifecycle management';
COMMENT ON COLUMN experiments.traffic_allocation IS 'Percentage of users to include (0.0-1.0)';
COMMENT ON COLUMN experiments.status IS 'draft, running, paused, or completed';

-- ============================================================================
-- Experiment Variants Table
-- ============================================================================
-- Stores variant configurations for each experiment

CREATE TABLE IF NOT EXISTS experiment_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    experiment_id UUID NOT NULL REFERENCES experiments(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    weight FLOAT NOT NULL DEFAULT 0.5,
    config JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_variant_name UNIQUE(experiment_id, name),
    CONSTRAINT valid_weight CHECK (weight >= 0.0 AND weight <= 1.0)
);

CREATE INDEX idx_variants_experiment ON experiment_variants(experiment_id);

COMMENT ON TABLE experiment_variants IS 'Variants for A/B tests with weighted distribution';
COMMENT ON COLUMN experiment_variants.weight IS 'Variant weight for distribution (must sum to 1.0 per experiment)';
COMMENT ON COLUMN experiment_variants.config IS 'JSON configuration for variant (e.g., algorithm parameters)';

-- ============================================================================
-- User Assignments Table
-- ============================================================================
-- Tracks which variant each user is assigned to (consistent hashing)

CREATE TABLE IF NOT EXISTS experiment_assignments (
    id BIGSERIAL PRIMARY KEY,
    experiment_id UUID NOT NULL REFERENCES experiments(id) ON DELETE CASCADE,
    variant_id UUID NOT NULL REFERENCES experiment_variants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_experiment UNIQUE(experiment_id, user_id)
);

CREATE INDEX idx_assignments_experiment ON experiment_assignments(experiment_id);
CREATE INDEX idx_assignments_user ON experiment_assignments(user_id);
CREATE INDEX idx_assignments_variant ON experiment_assignments(variant_id);

COMMENT ON TABLE experiment_assignments IS 'User-to-variant assignments using consistent hashing';
COMMENT ON COLUMN experiment_assignments.user_id IS 'User assigned to variant (deterministic via hashing)';

-- ============================================================================
-- Exposure Events Table
-- ============================================================================
-- Tracks when users see a variant (impressions)

CREATE TABLE IF NOT EXISTS experiment_exposures (
    id BIGSERIAL PRIMARY KEY,
    experiment_id UUID NOT NULL REFERENCES experiments(id) ON DELETE CASCADE,
    variant_id UUID NOT NULL REFERENCES experiment_variants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    exposed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    context JSONB
);

CREATE INDEX idx_exposures_experiment ON experiment_exposures(experiment_id);
CREATE INDEX idx_exposures_variant ON experiment_exposures(variant_id);
CREATE INDEX idx_exposures_user ON experiment_exposures(user_id);
CREATE INDEX idx_exposures_time ON experiment_exposures(exposed_at);

COMMENT ON TABLE experiment_exposures IS 'Tracks when users are exposed to variants';
COMMENT ON COLUMN experiment_exposures.context IS 'Additional context (endpoint, device, etc.)';

-- ============================================================================
-- Conversion Events Table
-- ============================================================================
-- Tracks goal completions (e.g., watch completion, click-through)

CREATE TABLE IF NOT EXISTS experiment_conversions (
    id BIGSERIAL PRIMARY KEY,
    experiment_id UUID NOT NULL REFERENCES experiments(id) ON DELETE CASCADE,
    variant_id UUID NOT NULL REFERENCES experiment_variants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    metric_name VARCHAR(255) NOT NULL,
    value FLOAT NOT NULL DEFAULT 1.0,
    converted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB
);

CREATE INDEX idx_conversions_experiment ON experiment_conversions(experiment_id);
CREATE INDEX idx_conversions_variant ON experiment_conversions(variant_id);
CREATE INDEX idx_conversions_metric ON experiment_conversions(metric_name);
CREATE INDEX idx_conversions_time ON experiment_conversions(converted_at);

COMMENT ON TABLE experiment_conversions IS 'Tracks goal completions for A/B testing';
COMMENT ON COLUMN experiment_conversions.metric_name IS 'Metric identifier (e.g., watch_completion, click_through)';
COMMENT ON COLUMN experiment_conversions.value IS 'Metric value (1.0 for binary, duration for continuous)';

-- ============================================================================
-- Helper Views
-- ============================================================================

-- View: Running experiments with variant counts
CREATE OR REPLACE VIEW running_experiments_summary AS
SELECT
    e.id,
    e.name,
    e.description,
    e.traffic_allocation,
    e.started_at,
    COUNT(DISTINCT v.id) as variant_count,
    COUNT(DISTINCT a.user_id) as assigned_users,
    COUNT(DISTINCT ex.user_id) as exposed_users
FROM experiments e
LEFT JOIN experiment_variants v ON e.id = v.experiment_id
LEFT JOIN experiment_assignments a ON e.id = a.experiment_id
LEFT JOIN experiment_exposures ex ON e.id = ex.experiment_id
WHERE e.status = 'running'
GROUP BY e.id, e.name, e.description, e.traffic_allocation, e.started_at;

COMMENT ON VIEW running_experiments_summary IS 'Quick overview of active experiments';

-- View: Variant performance metrics
CREATE OR REPLACE VIEW variant_performance AS
SELECT
    e.id as experiment_id,
    e.name as experiment_name,
    v.id as variant_id,
    v.name as variant_name,
    v.weight,
    COUNT(DISTINCT ex.user_id) as unique_users,
    COUNT(ex.id) as exposures,
    COUNT(DISTINCT c.id) as conversions,
    CASE
        WHEN COUNT(ex.id) > 0 THEN COUNT(DISTINCT c.id)::FLOAT / COUNT(ex.id)::FLOAT
        ELSE 0
    END as conversion_rate
FROM experiments e
JOIN experiment_variants v ON e.id = v.experiment_id
LEFT JOIN experiment_exposures ex ON v.id = ex.variant_id
LEFT JOIN experiment_conversions c ON v.id = c.variant_id
WHERE e.status = 'running'
GROUP BY e.id, e.name, v.id, v.name, v.weight;

COMMENT ON VIEW variant_performance IS 'Real-time variant performance metrics';
