-- Rollback initial schema migration

DROP TABLE IF EXISTS rate_limits;
DROP TABLE IF EXISTS refresh_tokens;
DROP TABLE IF EXISTS oauth_tokens;
DROP TABLE IF EXISTS user_devices;
DROP TABLE IF EXISTS watch_progress;
DROP TABLE IF EXISTS user_watchlist;
DROP TABLE IF EXISTS series_metadata;
DROP TABLE IF EXISTS platform_availability;
DROP TABLE IF EXISTS content_ratings;
DROP TABLE IF EXISTS credits;
DROP TABLE IF EXISTS content_moods;
DROP TABLE IF EXISTS content_themes;
DROP TABLE IF EXISTS content_genres;
DROP TABLE IF EXISTS platform_ids;
DROP TABLE IF EXISTS external_ids;
DROP TABLE IF EXISTS content;
DROP TABLE IF EXISTS user_preferences;
DROP TABLE IF EXISTS users;

DROP EXTENSION IF EXISTS pg_trgm;
DROP EXTENSION IF EXISTS "uuid-ossp";
