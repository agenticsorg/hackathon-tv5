#!/usr/bin/env python3
from __future__ import annotations

import asyncio
import os
from pathlib import Path
import pandas as pd


async def seed(database_url: str, df: pd.DataFrame) -> None:
    import asyncpg

    conn = await asyncpg.connect(database_url)
    try:
        norm = []
        for row in df[["user_id", "movie_id", "rating", "rated_at"]].itertuples(index=False, name=None):
            uid, mid, rating, ts = row
            ts_norm = None
            if not pd.isna(ts):
                try:
                    ts_norm = pd.to_datetime(ts).to_pydatetime()
                except Exception:
                    ts_norm = None
            norm.append((int(uid), int(mid), float(rating), ts_norm))
        await conn.executemany(
            """
            INSERT INTO user_ratings (user_id, movie_id, rating, rated_at)
            VALUES ($1,$2,$3,$4)
            ON CONFLICT (user_id, movie_id) DO UPDATE SET rating=EXCLUDED.rating, rated_at=COALESCE(EXCLUDED.rated_at, user_ratings.rated_at)
            """,
            norm,
        )
    finally:
        await conn.close()


def main() -> None:
    db_url = os.getenv("DATABASE_URL")
    if not db_url:
        raise SystemExit("Set DATABASE_URL")
    processed = Path(os.getenv("PROCESSED_PREFIX", "data/sample-processed"))
    parts = sorted(processed.glob("ratings_enriched-*.parquet"))
    if not parts:
        raise FileNotFoundError(f"No ratings_enriched-*.parquet in {processed}")
    frames = []
    for p in parts:
        df = pd.read_parquet(p)
        if "userId" in df.columns:
            df = df.rename(columns={"userId": "user_id"})
        if "movieId" in df.columns:
            df = df.rename(columns={"movieId": "movie_id"})
        df["rated_at"] = pd.NaT
        frames.append(df[["user_id", "movie_id", "rating", "rated_at"]])
    merged = pd.concat(frames, ignore_index=True)
    asyncio.run(seed(db_url, merged))
    print(f"Upserted {len(merged):,} user_ratings rows")


if __name__ == "__main__":
    main()
