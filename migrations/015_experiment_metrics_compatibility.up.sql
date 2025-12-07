-- Experiment Metrics Compatibility Layer
-- Media Gateway - SONA Engine
--
-- This migration ensures backward compatibility with code expecting experiment_metrics table
-- while the actual storage uses experiment_exposures and experiment_conversions.
--
-- Dependencies: 011_ab_testing_schema.sql

-- ============================================================================
-- Compatibility View: experiment_metrics
-- ============================================================================
-- Provides unified view combining exposures and conversions for legacy code

CREATE OR REPLACE VIEW experiment_metrics AS
SELECT
    id,
    experiment_id,
    variant_id,
    user_id,
    'exposure' as metric_name,
    1.0 as metric_value,
    exposed_at as recorded_at
FROM experiment_exposures
UNION ALL
SELECT
    id,
    experiment_id,
    variant_id,
    user_id,
    metric_name,
    value as metric_value,
    converted_at as recorded_at
FROM experiment_conversions;

COMMENT ON VIEW experiment_metrics IS 'Unified view of exposures and conversions for backward compatibility';

-- ============================================================================
-- Additional Indexes for Performance
-- ============================================================================

-- Index on experiment_id and metric_name for faster metric queries
CREATE INDEX IF NOT EXISTS idx_conversions_experiment_metric
ON experiment_conversions(experiment_id, metric_name);

-- Index on user_id and converted_at for user activity tracking
CREATE INDEX IF NOT EXISTS idx_conversions_user_time
ON experiment_conversions(user_id, converted_at);

-- Index on user_id and exposed_at for exposure tracking
CREATE INDEX IF NOT EXISTS idx_exposures_user_time
ON experiment_exposures(user_id, exposed_at);

-- ============================================================================
-- Helper Functions
-- ============================================================================

-- Function to get experiment summary
CREATE OR REPLACE FUNCTION get_experiment_summary(exp_id UUID)
RETURNS TABLE (
    variant_id UUID,
    variant_name VARCHAR,
    total_exposures BIGINT,
    total_conversions BIGINT,
    conversion_rate FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        v.id as variant_id,
        v.name as variant_name,
        COUNT(DISTINCT e.id) as total_exposures,
        COUNT(DISTINCT c.id) as total_conversions,
        CASE
            WHEN COUNT(DISTINCT e.id) > 0 THEN
                COUNT(DISTINCT c.id)::FLOAT / COUNT(DISTINCT e.id)::FLOAT
            ELSE 0.0
        END as conversion_rate
    FROM experiment_variants v
    LEFT JOIN experiment_exposures e ON v.id = e.variant_id
    LEFT JOIN experiment_conversions c ON v.id = c.variant_id
    WHERE v.experiment_id = exp_id
    GROUP BY v.id, v.name
    ORDER BY v.name;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION get_experiment_summary IS 'Get summary statistics for all variants in an experiment';
