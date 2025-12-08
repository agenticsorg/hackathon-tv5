from __future__ import annotations

from pathlib import Path
import os
import pandas as pd
import math


def _storage_options(path: str | Path) -> dict | None:
    p = str(path)
    return {"token": "cloud"} if p.startswith("gs://") else None


def main():
    project_root = Path(__file__).parent.parent
    data_prefix = os.getenv("DATA_PREFIX") or os.getenv("GCS_DATA_PREFIX")
    processed_prefix = os.getenv("PROCESSED_PREFIX") or os.getenv("GCS_PROCESSED_PREFIX")

    if data_prefix:
        if str(data_prefix).startswith("gs://"):
            tmdb_csv = f"{data_prefix}/tmdb/TMDB_movie_dataset_v11.csv"
            links_csv = f"{data_prefix}/movielens/ml-25m/links.csv"
            ratings_csv = f"{data_prefix}/movielens/ml-25m/ratings.csv"
        else:
            dp = Path(str(data_prefix))
            tmdb_csv = dp / "tmdb/TMDB_movie_dataset_v11.csv"
            links_csv = dp / "movielens/ml-25m/links.csv"
            ratings_csv = dp / "movielens/ml-25m/ratings.csv"
    else:
        tmdb_csv = project_root / "data/tmdb/TMDB_movie_dataset_v11.csv"
        links_csv = project_root / "data/movielens/ml-25m/links.csv"
        ratings_csv = project_root / "data/movielens/ml-25m/ratings.csv"

    if processed_prefix:
        out_dir = processed_prefix
    else:
        out_dir = project_root / "data/processed"
        Path(out_dir).mkdir(parents=True, exist_ok=True)

    print("Loading TMDB (filtered columns)...")
    tmdb = pd.read_csv(
        tmdb_csv,
        storage_options=_storage_options(tmdb_csv),
        usecols=[c for c in ["imdb_id", "status", "overview", "title", "genres", "vote_average", "release_date"] if True],
    )
    tmdb = tmdb[tmdb["status"] == "Released"]
    # Keep modest descriptions locally; production datasets will far exceed this
    tmdb = tmdb[tmdb["overview"].notna() & (tmdb["overview"].astype(str).str.len() > 10)]
    tmdb["imdb_id_clean"] = tmdb["imdb_id"].astype(str).str.replace("tt", "", regex=False)
    tmdb["imdb_id_clean"] = pd.to_numeric(tmdb["imdb_id_clean"], errors="coerce")
    tmdb = tmdb.dropna(subset=["imdb_id_clean"])  # keep rows with parsed imdb

    print("Loading MovieLens links (small) and preparing mapping...")
    links = pd.read_csv(links_csv, storage_options=_storage_options(links_csv))
    links["imdbId"] = pd.to_numeric(links["imdbId"], errors="coerce")
    links = links.dropna(subset=["imdbId"])  # keep joinable

    print("Joining TMDB -> MovieLens (movies metadata only)...")
    movies_joined = pd.merge(
        tmdb[["imdb_id_clean", "title", "overview", "genres", "vote_average", "release_date"]],
        links[["movieId", "imdbId"]],
        left_on="imdb_id_clean",
        right_on="imdbId",
        how="inner",
    )

    keep_cols = ["movieId", "title", "overview", "genres", "vote_average", "release_date"]
    movies_keep = movies_joined[[c for c in keep_cols if c in movies_joined.columns]].copy()

    # Write movies metadata
    if isinstance(out_dir, str) and str(out_dir).startswith("gs://"):
        movies_keep.to_parquet(f"{out_dir}/movies_with_descriptions.parquet", storage_options=_storage_options(out_dir))
    else:
        Path(out_dir).mkdir(parents=True, exist_ok=True)
        Path(out_dir, "triplets").mkdir(exist_ok=True)
        movies_keep.to_parquet(Path(out_dir) / "movies_with_descriptions.parquet")

    # Stream ratings in chunks to avoid OOM; write partitioned enriched chunks
    print("Streaming ratings -> enriched parquet parts (chunked)...")
    part = 0
    chunksize = int(os.getenv("JOIN_RATINGS_CHUNKSIZE", "1000000"))
    usecols = ["userId", "movieId", "rating"]
    reader = pd.read_csv(ratings_csv, storage_options=_storage_options(ratings_csv), usecols=usecols, chunksize=chunksize)
    total_rows = 0
    for chunk in reader:
        total_rows += len(chunk)
        chunk = chunk[["userId", "movieId", "rating"]]
        enriched = chunk.merge(movies_keep[["movieId", "title", "genres"]], on="movieId", how="inner")
        enriched = enriched.rename(columns={"userId": "user_id"})
        if isinstance(out_dir, str) and str(out_dir).startswith("gs://"):
            outp = f"{out_dir}/ratings_enriched-{part:05d}.parquet"
            enriched.to_parquet(outp, storage_options=_storage_options(outp), index=False)
        else:
            outp = Path(out_dir) / f"ratings_enriched-{part:05d}.parquet"
            enriched.to_parquet(outp, index=False)
        print(f"Wrote part {part:05d} with {len(enriched):,} rows -> {outp}")
        part += 1

    print({
        "tmdb_descriptions": len(tmdb),
        "movies_matched": len(movies_keep),
        "ratings_rows_processed": total_rows,
        "ratings_parts": part,
    })


if __name__ == "__main__":
    main()
