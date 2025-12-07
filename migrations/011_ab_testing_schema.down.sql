-- Drop views
DROP VIEW IF EXISTS variant_performance;
DROP VIEW IF EXISTS running_experiments_summary;

-- Drop indexes
DROP INDEX IF EXISTS idx_conversions_time;
DROP INDEX IF EXISTS idx_conversions_metric;
DROP INDEX IF EXISTS idx_conversions_variant;
DROP INDEX IF EXISTS idx_conversions_experiment;
DROP INDEX IF EXISTS idx_exposures_time;
DROP INDEX IF EXISTS idx_exposures_user;
DROP INDEX IF EXISTS idx_exposures_variant;
DROP INDEX IF EXISTS idx_exposures_experiment;
DROP INDEX IF EXISTS idx_assignments_variant;
DROP INDEX IF EXISTS idx_assignments_user;
DROP INDEX IF EXISTS idx_assignments_experiment;
DROP INDEX IF EXISTS idx_variants_experiment;
DROP INDEX IF EXISTS idx_experiments_created_at;
DROP INDEX IF EXISTS idx_experiments_status;

-- Drop tables
DROP TABLE IF EXISTS experiment_conversions;
DROP TABLE IF EXISTS experiment_exposures;
DROP TABLE IF EXISTS experiment_assignments;
DROP TABLE IF EXISTS experiment_variants;
DROP TABLE IF EXISTS experiments;
