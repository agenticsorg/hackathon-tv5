-- Add parental controls to users table
ALTER TABLE users ADD COLUMN parental_controls JSONB DEFAULT NULL;

-- Create index for faster lookups on users with parental controls enabled
CREATE INDEX idx_users_parental_controls_enabled
ON users ((parental_controls->>'enabled'))
WHERE parental_controls IS NOT NULL;
