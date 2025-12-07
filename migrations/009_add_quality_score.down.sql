-- Drop index
DROP INDEX IF EXISTS idx_content_quality_score;

-- Drop column
ALTER TABLE content DROP COLUMN IF EXISTS quality_score;
