from __future__ import annotations

from pathlib import Path
import pandas as pd


def normalize_title(t: str) -> str:
    return (t or "").strip().lower()


def enrich(
    processed_dir: str = "data/processed",
    tmdb_csv_path: str = "data/tmdb/movies_metadata.csv",
    out_path: str = "data/processed/movies_enriched.parquet",
) -> None:
    movies_pq = Path(processed_dir) / "movies.parquet"
    if not movies_pq.exists():
        raise FileNotFoundError("Run netflix_parser.build() first to generate movies.parquet")

    movies = pd.read_parquet(movies_pq)
    movies["title_norm"] = movies["title"].map(normalize_title)

    if Path(tmdb_csv_path).exists():
        tm = pd.read_csv(tmdb_csv_path, low_memory=False)
        # Keep relevant fields
        keep = [
            "id",
            "title",
            "overview",
            "genres",
            "release_date",
            "vote_average",
            "popularity",
        ]
        tm = tm[keep]
        tm["title_norm"] = tm["title"].map(normalize_title)

        # Naive title-based join (improve later with year-based matching)
        merged = movies.merge(tm, on="title_norm", how="left", suffixes=("", "_tmdb"))
        merged.rename(columns={"id": "tmdb_id"}, inplace=True)
        merged.to_parquet(out_path)
        print(f"Enriched movies saved to {out_path}")
    else:
        # Fallback: save base movies without enrichment
        movies.to_parquet(out_path)
        print("TMDB CSV not found; saved base movies without enrichment.")


if __name__ == "__main__":
    enrich()

