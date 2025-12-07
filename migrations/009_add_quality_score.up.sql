-- Add quality score column to content table
ALTER TABLE content ADD COLUMN quality_score REAL NOT NULL DEFAULT 0.0;

-- Create index for quality score queries
CREATE INDEX idx_content_quality_score ON content(quality_score);

-- Update existing content with initial quality scores (will be recalculated by batch job)
UPDATE content SET quality_score = 0.5 WHERE quality_score = 0.0;
