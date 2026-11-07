#!/usr/bin/env python3
from __future__ import annotations

import asyncio
import os
from pathlib import Path
from typing import Iterable, Set

import pandas as pd


def _storage_options(path: str | Path) -> dict | None:
    p = str(path)
    return {"token": "cloud"} if p.startswith("gs://") else None


async def _fetch_existing(conn, movie_ids: Iterable[int]) -> tuple[Set[int], Set[int]]:
    mids = list(set(int(x) for x in movie_ids))
    if not mids:
        return set(), set()
    rows1 = await conn.fetch("SELECT movie_id FROM movies WHERE movie_id = ANY($1::int[])", mids)
    rows2 = await conn.fetch("SELECT movie_id FROM movie_embeddings WHERE movie_id = ANY($1::int[])", mids)
    have_movies = {int(r["movie_id"]) for r in rows1}
    have_embs = {int(r["movie_id"]) for r in rows2}
    return have_movies, have_embs


async def main() -> int:
    db_url = os.getenv("DATABASE_URL")
    triplets_dir = os.getenv("GCS_TRIPLETS_PREFIX", "data/processed/triplets")
    if not db_url:
        print("Set DATABASE_URL")
        return 2
    trip_path = f"{triplets_dir}/triplets_10k.parquet"
    df = pd.read_parquet(trip_path, storage_options=_storage_options(trip_path))
    needed: Set[int] = set(map(int, df["pos_movie_id"].tolist())) | set(map(int, df["neg_movie_id"].tolist()))

    import asyncpg  # lazy import

    conn = await asyncpg.connect(db_url)
    try:
        have_movies, have_embs = await _fetch_existing(conn, needed)
    finally:
        await conn.close()

    missing_movies = needed - have_movies
    missing_embs = needed - have_embs
    print(f"Triplets movies referenced: {len(needed):,}")
    print(f"Present in movies table:    {len(have_movies):,} (missing {len(missing_movies):,})")
    print(f"With embeddings present:    {len(have_embs):,} (missing {len(missing_embs):,})")
    if missing_movies:
        print(f"Missing in movies table (sample): {sorted(list(missing_movies))[:10]}")
    if missing_embs:
        print(f"Missing embeddings (sample): {sorted(list(missing_embs))[:10]}")
    # Non-zero exit if any gaps
    return 0 if not missing_movies and not missing_embs else 1


if __name__ == "__main__":
    raise SystemExit(asyncio.run(main()))

