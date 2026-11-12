from __future__ import annotations

import asyncio
from typing import Any
import asyncpg
import numpy as np
from app.core.config import settings


class DB:
    def __init__(self, dsn: str | None = None) -> None:
        self._dsn = dsn or settings.database_url
        self._pool: asyncpg.Pool | None = None

    async def connect(self) -> None:
        if self._pool is None:
            if not self._dsn:
                import os
                env_dsn = os.getenv("DATABASE_URL")
                if env_dsn:
                    self._dsn = env_dsn
                else:
                    raise RuntimeError("DATABASE_URL not configured")
            self._pool = await asyncpg.create_pool(self._dsn, min_size=1, max_size=5)

    async def close(self) -> None:
        if self._pool is not None:
            await self._pool.close()
            self._pool = None

    async def fetch_similar(self, query_vec: np.ndarray, top_k: int = 10) -> list[dict[str, Any]]:
        await self.connect()
        assert self._pool is not None
        # Query movies by cosine distance, return movie and score
        q = """
        SELECT m.movie_id, m.title, m.genres, 1 - (e.embedding <=> $1::vector) AS score
        FROM movie_embeddings e
        JOIN movies m USING (movie_id)
        ORDER BY e.embedding <=> $1::vector
        LIMIT $2
        """
        def _vec_to_pgtext(v: np.ndarray) -> str:
            return "[" + ",".join(str(float(x)) for x in v.tolist()) + "]"
        vec = _vec_to_pgtext(query_vec.astype(float))
        async with self._pool.acquire() as conn:
            rows = await conn.fetch(q, vec, top_k)
        return [dict(r) for r in rows]

    async def fetch_user_profile_embedding(self, user_id: int, min_rating: float = 4.0) -> np.ndarray | None:
        """Return an average embedding of movies the user rated >= min_rating.
        Falls back to None if no vectors exist.
        """
        await self.connect()
        assert self._pool is not None
        q = """
        SELECT e.embedding
        FROM user_ratings r
        JOIN movie_embeddings e USING (movie_id)
        WHERE r.user_id = $1 AND r.rating >= $2
        LIMIT 1000
        """
        async with self._pool.acquire() as conn:
            rows = await conn.fetch(q, user_id, min_rating)
        if not rows:
            return None
        def _parse_vec(val: Any) -> np.ndarray:
            if isinstance(val, str):
                s = val.strip().strip("[]")
                parts = [p for p in s.split(",") if p.strip() != ""]
                return np.array([float(p) for p in parts], dtype=np.float32)
            # assume list-like of floats
            return np.array(list(val), dtype=np.float32)

        vecs = [_parse_vec(r["embedding"]) for r in rows]
        mean_vec = np.mean(np.stack(vecs, axis=0), axis=0)
        # Normalize to unit length for cosine search
        n = np.linalg.norm(mean_vec)
        if n > 0:
            mean_vec = mean_vec / n
        return mean_vec.astype(np.float32)

    async def fetch_genre_weights(self, movie_ids: list[int]) -> dict[int, float]:
        """Sum of genre-edge weights per movie for simple graph boost."""
        if not movie_ids:
            return {}
        await self.connect()
        assert self._pool is not None
        q = """
        SELECT src_id AS movie_id, COALESCE(SUM(weight),0) AS w
        FROM hyperedges
        WHERE src_kind='movie' AND dst_kind='genre' AND src_id = ANY($1::bigint[])
        GROUP BY src_id
        """
        async with self._pool.acquire() as conn:
            rows = await conn.fetch(q, movie_ids)
        return {int(r["movie_id"]): float(r["w"]) for r in rows}

    async def fetch_neighbors_cowatch(self, movie_ids: list[int], top_k: int = 100) -> dict[int, float]:
        """Return co‑watch neighbors aggregated across a set of seed movie_ids.

        Uses hyperedges where (src_kind='movie', dst_kind='movie').
        """
        if not movie_ids:
            return {}
        await self.connect()
        assert self._pool is not None
        q = (
            "SELECT dst_id AS movie_id, SUM(weight) AS w "
            "FROM hyperedges WHERE src_kind='movie' AND dst_kind='movie' AND src_id = ANY($1::bigint[]) "
            "GROUP BY dst_id ORDER BY SUM(weight) DESC LIMIT $2"
        )
        async with self._pool.acquire() as conn:
            rows = await conn.fetch(q, movie_ids, top_k)
        return {int(r["movie_id"]): float(r["w"]) for r in rows}

    async def fetch_neighbors_shared_genre(self, movie_ids: list[int], top_k: int = 200) -> dict[int, float]:
        """Return neighbors via shared genres.

        We derive genre nodes from movie->genre edges and then collect other
        movies pointing to those genres. Weight is sum of (w_src * w_dst).
        """
        if not movie_ids:
            return {}
        await self.connect()
        assert self._pool is not None
        q = (
            "SELECT he2.src_id AS movie_id, SUM(he1.weight * he2.weight) AS w "
            "FROM hyperedges he1 "
            "JOIN hyperedges he2 ON he1.dst_kind='genre' AND he2.dst_kind='genre' AND he2.dst_id=he1.dst_id "
            "WHERE he1.src_kind='movie' AND he1.src_id = ANY($1::bigint[]) "
            "GROUP BY he2.src_id ORDER BY w DESC LIMIT $2"
        )
        async with self._pool.acquire() as conn:
            rows = await conn.fetch(q, movie_ids, top_k)
        return {int(r["movie_id"]): float(r["w"]) for r in rows}

    async def fetch_movies_by_ids(self, movie_ids: list[int]) -> dict[int, dict]:
        if not movie_ids:
            return {}
        await self.connect()
        assert self._pool is not None
        q = "SELECT movie_id, title, genres FROM movies WHERE movie_id = ANY($1::int[])"
        async with self._pool.acquire() as conn:
            rows = await conn.fetch(q, movie_ids)
        return {int(r["movie_id"]): {"title": r["title"], "genres": r["genres"]} for r in rows}


_db_singleton: DB | None = None


def get_db() -> DB:
    global _db_singleton
    if _db_singleton is None:
        _db_singleton = DB()
    return _db_singleton
