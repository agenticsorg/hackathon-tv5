-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Movies base table (enriched metadata subset)
CREATE TABLE IF NOT EXISTS movies (
  movie_id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  genres TEXT,
  overview TEXT,
  release_year INTEGER,
  tmdb_id INTEGER
);

-- Movie embeddings (384-d float32 vectors, unit-normalized for cosine)
CREATE TABLE IF NOT EXISTS movie_embeddings (
  movie_id INTEGER PRIMARY KEY REFERENCES movies(movie_id) ON DELETE CASCADE,
  embedding vector(384)
);

-- HNSW index for fast cosine similarity search
DROP INDEX IF EXISTS idx_movie_embeddings_hnsw;
CREATE INDEX idx_movie_embeddings_hnsw ON movie_embeddings USING hnsw (embedding vector_cosine_ops);

-- Optional: user cached embeddings
CREATE TABLE IF NOT EXISTS user_embeddings (
  user_id BIGINT PRIMARY KEY,
  embedding vector(384),
  updated_at TIMESTAMP DEFAULT now()
);

-- User ratings table (MovieLens compatible)
-- Kept minimal for analytics/pipeline joins; raw imports may live in GCS
CREATE TABLE IF NOT EXISTS user_ratings (
  user_id BIGINT NOT NULL,
  movie_id INTEGER NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
  rating NUMERIC(2,1) NOT NULL CHECK (rating >= 0.5 AND rating <= 5.0),
  rated_at TIMESTAMP,
  PRIMARY KEY (user_id, movie_id)
);
CREATE INDEX IF NOT EXISTS idx_user_ratings_user ON user_ratings(user_id);
CREATE INDEX IF NOT EXISTS idx_user_ratings_movie ON user_ratings(movie_id);

-- Hyperedges table to support graph-like relationships (e.g., co-watch, genre-affinity)
-- Flexible JSONB payload for features/weights
CREATE TABLE IF NOT EXISTS hyperedges (
  id BIGSERIAL PRIMARY KEY,
  src_kind TEXT NOT NULL,    -- e.g., 'user' or 'movie'
  src_id BIGINT NOT NULL,
  dst_kind TEXT NOT NULL,    -- e.g., 'movie' or 'genre'
  dst_id BIGINT NOT NULL,
  weight REAL DEFAULT 1.0,
  payload JSONB,
  created_at TIMESTAMP DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_hyperedges_src ON hyperedges(src_kind, src_id);
CREATE INDEX IF NOT EXISTS idx_hyperedges_dst ON hyperedges(dst_kind, dst_id);
