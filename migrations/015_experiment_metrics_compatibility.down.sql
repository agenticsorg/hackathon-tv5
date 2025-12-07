-- Drop function
DROP FUNCTION IF EXISTS get_experiment_summary(UUID);

-- Drop indexes
DROP INDEX IF EXISTS idx_exposures_user_time;
DROP INDEX IF EXISTS idx_conversions_user_time;
DROP INDEX IF EXISTS idx_conversions_experiment_metric;

-- Drop view
DROP VIEW IF EXISTS experiment_metrics;
