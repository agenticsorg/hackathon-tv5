from __future__ import annotations

import asyncio
import os
from pathlib import Path
import pandas as pd


def _is_gcs(path: str | Path) -> bool:
    return str(path).startswith("gs://")


async def main():
    db_url = os.getenv("DATABASE_URL")
    if not db_url:
        raise SystemExit("Set DATABASE_URL")
    import asyncpg

    processed_env = os.getenv("PROCESSED_PREFIX", "data/processed")
    if _is_gcs(processed_env):
        movies_path = f"{processed_env}/movies_with_descriptions.parquet"
        df = pd.read_parquet(movies_path, storage_options={"token": "cloud"})
    else:
        processed = Path(processed_env)
        movies_path = processed / "movies_with_descriptions.parquet"
        if not movies_path.exists():
            raise FileNotFoundError("data/processed/movies_with_descriptions.parquet not found")
        df = pd.read_parquet(movies_path)
    cols = ["movie_id", "title", "genres", "overview", "release_year", "tmdb_id"]
    # Conform schema: derive release_year from release_date where present
    if "movieId" in df.columns:
        df["movie_id"] = df["movieId"].astype(int)
    if "release_year" not in df.columns:
        if "release_date" in df.columns:
            df["release_year"] = pd.to_datetime(df["release_date"], errors="coerce").dt.year
        else:
            df["release_year"] = None
    # Normalize dtypes and nulls for DB insert
    df["release_year"] = pd.to_numeric(df["release_year"], errors="coerce").astype("Int64")
    df["tmdb_id"] = pd.to_numeric(df.get("tmdbId", df.get("tmdb_id", None)), errors="coerce").astype("Int64") if ("tmdbId" in df.columns or "tmdb_id" in df.columns) else df.get("tmdb_id", None)
    # ensure strings
    df["title"] = df["title"].astype(str)
    df["genres"] = df.get("genres", "").astype(str)
    df["overview"] = df.get("overview", "").astype(str)
    for col in cols:
        if col not in df.columns:
            df[col] = None

    # Convert pandas NA to Python None
    def _py(v):
        if hasattr(pd, "isna") and pd.isna(v):
            return None
        return v
    rows = [(_py(a), _py(b), _py(c), _py(d), _py(e), _py(f)) for a,b,c,d,e,f in df[cols].itertuples(index=False, name=None)]
    conn = await asyncpg.connect(db_url)
    try:
        await conn.executemany(
            """
            INSERT INTO movies (movie_id, title, genres, overview, release_year, tmdb_id)
            VALUES ($1,$2,$3,$4,$5,$6)
            ON CONFLICT (movie_id) DO UPDATE SET
              title=EXCLUDED.title,
              genres=EXCLUDED.genres,
              overview=EXCLUDED.overview,
              release_year=EXCLUDED.release_year,
              tmdb_id=EXCLUDED.tmdb_id
            """,
            rows,
        )
    finally:
        await conn.close()
    print(f"Upserted {len(rows)} movies")


if __name__ == "__main__":
    asyncio.run(main())
