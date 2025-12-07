-- Media Gateway Development Seed Data
-- Provides test fixtures for local development and testing
--
-- Usage: psql -U mediagateway -d media_gateway -f seed-data.sql
-- Or via Docker: docker-compose exec -T postgres psql -U mediagateway -d media_gateway -f - < scripts/seed-data.sql

-- Start transaction
BEGIN;

-- ============================================================================
-- Test Users
-- ============================================================================

-- Insert test users with hashed passwords (password: "test123")
INSERT INTO users (id, email, password_hash, username, email_verified, created_at, updated_at)
VALUES
    ('00000000-0000-0000-0000-000000000001'::uuid, 'alice@test.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYJq.JxGKKK', 'alice', true, NOW(), NOW()),
    ('00000000-0000-0000-0000-000000000002'::uuid, 'bob@test.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYJq.JxGKKK', 'bob', true, NOW(), NOW()),
    ('00000000-0000-0000-0000-000000000003'::uuid, 'charlie@test.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYJq.JxGKKK', 'charlie', true, NOW(), NOW()),
    ('00000000-0000-0000-0000-000000000004'::uuid, 'diana@test.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYJq.JxGKKK', 'diana', false, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- User Preferences
-- ============================================================================

-- Insert user preferences for test users
INSERT INTO user_preferences (user_id, preferred_genres, preferred_languages, content_rating_limit, autoplay_enabled, subtitle_language)
VALUES
    ('00000000-0000-0000-0000-000000000001'::uuid, ARRAY['action', 'sci-fi', 'thriller'], ARRAY['en', 'es'], 'R', true, 'en'),
    ('00000000-0000-0000-0000-000000000002'::uuid, ARRAY['comedy', 'drama', 'romance'], ARRAY['en'], 'PG-13', true, 'en'),
    ('00000000-0000-0000-0000-000000000003'::uuid, ARRAY['documentary', 'history', 'biography'], ARRAY['en', 'fr'], 'PG', false, 'fr'),
    ('00000000-0000-0000-0000-000000000004'::uuid, ARRAY['animation', 'family', 'adventure'], ARRAY['en'], 'G', true, 'en')
ON CONFLICT (user_id) DO NOTHING;

-- ============================================================================
-- Content (Movies and Shows)
-- ============================================================================

-- Insert sample content
INSERT INTO content (id, title, content_type, description, release_year, genres, duration_minutes, rating, language, quality_score, created_at)
VALUES
    ('10000000-0000-0000-0000-000000000001'::uuid, 'The Matrix', 'movie', 'A computer hacker learns about the true nature of reality', 1999, ARRAY['action', 'sci-fi'], 136, 'R', 'en', 0.92, NOW()),
    ('10000000-0000-0000-0000-000000000002'::uuid, 'Inception', 'movie', 'A thief who steals corporate secrets through dream-sharing technology', 2010, ARRAY['action', 'sci-fi', 'thriller'], 148, 'PG-13', 'en', 0.95, NOW()),
    ('10000000-0000-0000-0000-000000000003'::uuid, 'The Office', 'series', 'A mockumentary on a group of office workers', 2005, ARRAY['comedy'], 22, 'TV-14', 'en', 0.89, NOW()),
    ('10000000-0000-0000-0000-000000000004'::uuid, 'Planet Earth', 'series', 'A documentary series about nature and wildlife', 2006, ARRAY['documentary'], 50, 'TV-G', 'en', 0.96, NOW()),
    ('10000000-0000-0000-0000-000000000005'::uuid, 'Toy Story', 'movie', 'Animated adventure about toys that come to life', 1995, ARRAY['animation', 'family', 'adventure'], 81, 'G', 'en', 0.94, NOW()),
    ('10000000-0000-0000-0000-000000000006'::uuid, 'The Dark Knight', 'movie', 'Batman fights the Joker in Gotham City', 2008, ARRAY['action', 'crime', 'thriller'], 152, 'PG-13', 'en', 0.97, NOW()),
    ('10000000-0000-0000-0000-000000000007'::uuid, 'Breaking Bad', 'series', 'A chemistry teacher turned methamphetamine manufacturer', 2008, ARRAY['drama', 'crime', 'thriller'], 47, 'TV-MA', 'en', 0.98, NOW()),
    ('10000000-0000-0000-0000-000000000008'::uuid, 'The Godfather', 'movie', 'The aging patriarch of an organized crime dynasty', 1972, ARRAY['drama', 'crime'], 175, 'R', 'en', 0.99, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Playback Progress
-- ============================================================================

-- Insert playback progress for test users
INSERT INTO playback_progress (user_id, content_id, progress_seconds, duration_seconds, completed, last_watched_at)
VALUES
    ('00000000-0000-0000-0000-000000000001'::uuid, '10000000-0000-0000-0000-000000000001'::uuid, 3600, 8160, false, NOW() - INTERVAL '2 hours'),
    ('00000000-0000-0000-0000-000000000001'::uuid, '10000000-0000-0000-0000-000000000002'::uuid, 8880, 8880, true, NOW() - INTERVAL '1 day'),
    ('00000000-0000-0000-0000-000000000002'::uuid, '10000000-0000-0000-0000-000000000003'::uuid, 900, 1320, false, NOW() - INTERVAL '3 hours'),
    ('00000000-0000-0000-0000-000000000003'::uuid, '10000000-0000-0000-0000-000000000004'::uuid, 2500, 3000, false, NOW() - INTERVAL '1 day')
ON CONFLICT (user_id, content_id) DO NOTHING;

-- ============================================================================
-- A/B Testing Experiments
-- ============================================================================

-- Insert sample experiments
INSERT INTO experiments (id, name, description, status, traffic_allocation, created_at, updated_at)
VALUES
    ('20000000-0000-0000-0000-000000000001'::uuid, 'recommendation_algorithm_v2', 'Test new collaborative filtering algorithm', 'running', 1.0, NOW() - INTERVAL '7 days', NOW()),
    ('20000000-0000-0000-0000-000000000002'::uuid, 'lora_boost_factor', 'Test different LoRA boost factors for personalization', 'running', 0.5, NOW() - INTERVAL '3 days', NOW()),
    ('20000000-0000-0000-0000-000000000003'::uuid, 'diversity_threshold', 'Test diversity threshold in recommendations', 'draft', 1.0, NOW() - INTERVAL '1 day', NOW())
ON CONFLICT (id) DO NOTHING;

-- Insert experiment variants
INSERT INTO experiment_variants (id, experiment_id, name, weight, config)
VALUES
    -- Recommendation algorithm experiment
    ('21000000-0000-0000-0000-000000000001'::uuid, '20000000-0000-0000-0000-000000000001'::uuid, 'control', 0.5, '{"algorithm": "collaborative_v1", "min_similarity": 0.3}'::jsonb),
    ('21000000-0000-0000-0000-000000000002'::uuid, '20000000-0000-0000-0000-000000000001'::uuid, 'treatment', 0.5, '{"algorithm": "collaborative_v2", "min_similarity": 0.4}'::jsonb),

    -- LoRA boost experiment
    ('21000000-0000-0000-0000-000000000003'::uuid, '20000000-0000-0000-0000-000000000002'::uuid, 'control', 0.33, '{"lora_boost": 0.3}'::jsonb),
    ('21000000-0000-0000-0000-000000000004'::uuid, '20000000-0000-0000-0000-000000000002'::uuid, 'medium_boost', 0.33, '{"lora_boost": 0.5}'::jsonb),
    ('21000000-0000-0000-0000-000000000005'::uuid, '20000000-0000-0000-0000-000000000002'::uuid, 'high_boost', 0.34, '{"lora_boost": 0.7}'::jsonb),

    -- Diversity threshold experiment
    ('21000000-0000-0000-0000-000000000006'::uuid, '20000000-0000-0000-0000-000000000003'::uuid, 'low', 0.5, '{"diversity_threshold": 0.3}'::jsonb),
    ('21000000-0000-0000-0000-000000000007'::uuid, '20000000-0000-0000-0000-000000000003'::uuid, 'high', 0.5, '{"diversity_threshold": 0.6}'::jsonb)
ON CONFLICT (id) DO NOTHING;

-- Insert experiment assignments
INSERT INTO experiment_assignments (experiment_id, user_id, variant_id, assigned_at)
VALUES
    ('20000000-0000-0000-0000-000000000001'::uuid, '00000000-0000-0000-0000-000000000001'::uuid, '21000000-0000-0000-0000-000000000001'::uuid, NOW() - INTERVAL '6 days'),
    ('20000000-0000-0000-0000-000000000001'::uuid, '00000000-0000-0000-0000-000000000002'::uuid, '21000000-0000-0000-0000-000000000002'::uuid, NOW() - INTERVAL '6 days'),
    ('20000000-0000-0000-0000-000000000002'::uuid, '00000000-0000-0000-0000-000000000001'::uuid, '21000000-0000-0000-0000-000000000004'::uuid, NOW() - INTERVAL '2 days'),
    ('20000000-0000-0000-0000-000000000002'::uuid, '00000000-0000-0000-0000-000000000003'::uuid, '21000000-0000-0000-0000-000000000003'::uuid, NOW() - INTERVAL '2 days')
ON CONFLICT (experiment_id, user_id) DO NOTHING;

-- Insert experiment exposures
INSERT INTO experiment_exposures (experiment_id, variant_id, user_id, exposed_at, context)
VALUES
    ('20000000-0000-0000-0000-000000000001'::uuid, '21000000-0000-0000-0000-000000000001'::uuid, '00000000-0000-0000-0000-000000000001'::uuid, NOW() - INTERVAL '5 days', '{"device": "mobile", "endpoint": "/recommendations"}'::jsonb),
    ('20000000-0000-0000-0000-000000000001'::uuid, '21000000-0000-0000-0000-000000000002'::uuid, '00000000-0000-0000-0000-000000000002'::uuid, NOW() - INTERVAL '5 days', '{"device": "web", "endpoint": "/recommendations"}'::jsonb),
    ('20000000-0000-0000-0000-000000000002'::uuid, '21000000-0000-0000-0000-000000000004'::uuid, '00000000-0000-0000-0000-000000000001'::uuid, NOW() - INTERVAL '1 day', '{"device": "mobile", "endpoint": "/recommendations"}'::jsonb)
ON CONFLICT DO NOTHING;

-- Insert experiment conversions
INSERT INTO experiment_conversions (experiment_id, variant_id, user_id, metric_name, value, converted_at, metadata)
VALUES
    ('20000000-0000-0000-0000-000000000001'::uuid, '21000000-0000-0000-0000-000000000001'::uuid, '00000000-0000-0000-0000-000000000001'::uuid, 'click_through', 1.0, NOW() - INTERVAL '5 days', '{"content_id": "10000000-0000-0000-0000-000000000001"}'::jsonb),
    ('20000000-0000-0000-0000-000000000001'::uuid, '21000000-0000-0000-0000-000000000002'::uuid, '00000000-0000-0000-0000-000000000002'::uuid, 'watch_completion', 0.85, NOW() - INTERVAL '4 days', '{"content_id": "10000000-0000-0000-0000-000000000003", "watch_percentage": 85}'::jsonb)
ON CONFLICT DO NOTHING;

-- ============================================================================
-- Audit Logs
-- ============================================================================

-- Insert sample audit logs
INSERT INTO audit_logs (user_id, action, resource_type, resource_id, ip_address, user_agent, status, metadata, created_at)
VALUES
    ('00000000-0000-0000-0000-000000000001'::uuid, 'login', 'session', '00000000-0000-0000-0000-000000000001'::uuid, '192.168.1.100', 'Mozilla/5.0', 'success', '{"method": "email"}'::jsonb, NOW() - INTERVAL '2 hours'),
    ('00000000-0000-0000-0000-000000000001'::uuid, 'playback_start', 'content', '10000000-0000-0000-0000-000000000001'::uuid, '192.168.1.100', 'Mozilla/5.0', 'success', '{"device": "mobile"}'::jsonb, NOW() - INTERVAL '1 hour'),
    ('00000000-0000-0000-0000-000000000002'::uuid, 'login', 'session', '00000000-0000-0000-0000-000000000002'::uuid, '192.168.1.101', 'Mozilla/5.0', 'success', '{"method": "oauth", "provider": "google"}'::jsonb, NOW() - INTERVAL '3 hours'),
    ('00000000-0000-0000-0000-000000000003'::uuid, 'update_profile', 'user', '00000000-0000-0000-0000-000000000003'::uuid, '192.168.1.102', 'Mozilla/5.0', 'success', '{"fields": ["email_verified"]}'::jsonb, NOW() - INTERVAL '1 day')
ON CONFLICT DO NOTHING;

-- Commit transaction
COMMIT;

-- Display summary
SELECT 'Seed data loaded successfully!' AS status;
SELECT COUNT(*) AS user_count FROM users;
SELECT COUNT(*) AS content_count FROM content;
SELECT COUNT(*) AS experiment_count FROM experiments;
SELECT COUNT(*) AS variant_count FROM experiment_variants;
