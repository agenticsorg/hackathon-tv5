from __future__ import annotations

from pathlib import Path
import os
import pandas as pd
import numpy as np
import glob
try:
    import gcsfs  # noqa: F401
except Exception:
    gcsfs = None


def _storage_options(path: str | Path) -> dict | None:
    p = str(path)
    return {"token": "cloud"} if p.startswith("gs://") else None


def _list_ratings_parts(processed_dir: str) -> list[str]:
    pat = f"{processed_dir}/ratings_enriched-*.parquet"
    # Try gcsfs glob
    if str(processed_dir).startswith("gs://"):
        try:
            import gcsfs  # type: ignore
            fs = gcsfs.GCSFileSystem()
            matches = sorted(fs.glob(pat))
            if matches:
                return [m if m.startswith("gs://") else ("gs://" + m) for m in matches]
        except Exception:
            pass
    else:
        files = sorted(glob.glob(pat))
        if files:
            return files
    # Fallback: sequential probe
    return [f"{processed_dir}/ratings_enriched-{i:05d}.parquet" for i in range(0, 200)]


def generate_triplets(
    processed_dir: str = os.getenv("GCS_PROCESSED_PREFIX", "data/processed"),
    out_dir: str = os.getenv("GCS_TRIPLETS_PREFIX", "data/processed/triplets"),
    user_sample: int | None = 10_000,
    random_state: int = 42,
) -> None:
    # Avoid Path round-tripping for GCS URIs; Path("gs://...") becomes "gs:/..."
    out_is_gcs = str(out_dir).startswith("gs://")
    if not out_is_gcs:
        Path(out_dir).mkdir(parents=True, exist_ok=True)

    # Load movies metadata (small enough)
    movies_path = f"{processed_dir}/movies_with_descriptions.parquet" if str(processed_dir).startswith("gs://") else Path(processed_dir) / "movies_with_descriptions.parquet"
    movies = pd.read_parquet(movies_path, storage_options=_storage_options(movies_path))

    # Split positives and negatives
    # Build quick genre map if available
    # genres may be JSON-like text; keep as raw string match for simplicity
    movie_genres = movies.set_index("movieId")["genres"].to_dict()
    rng = np.random.default_rng(random_state)
    trip_rows: list[tuple[int, int, int]] = []  # (user_id, pos_movie, neg_movie)

    parts = _list_ratings_parts(processed_dir)
    if not parts:
        # Fallback to single-file
        parts = [f"{processed_dir}/ratings_enriched.parquet"]

    for pth in parts:
        df = pd.read_parquet(pth, storage_options=_storage_options(pth), columns=["user_id", "movieId", "rating"])
        positives = df[df["rating"] >= 4.0]
        negatives = df[df["rating"] <= 2.0]
        if positives.empty or negatives.empty:
            continue
        pos_by_user = positives.groupby("user_id")["movieId"].apply(list).to_dict()
        neg_by_user = negatives.groupby("user_id")["movieId"].apply(list).to_dict()
        for u, pos_list in pos_by_user.items():
            neg_list = neg_by_user.get(u)
            if not neg_list:
                continue
            p = rng.choice(pos_list)
            p_genres = str(movie_genres.get(int(p), ""))
            candidates = [n for n in neg_list if any(tok in str(movie_genres.get(int(n), "")) for tok in p_genres.split())]
            if not candidates:
                candidates = neg_list
            n = rng.choice(candidates)
            trip_rows.append((int(u), int(p), int(n)))
            if user_sample is not None and len(trip_rows) >= user_sample:
                break
        if user_sample is not None and len(trip_rows) >= user_sample:
            break

    df = pd.DataFrame(trip_rows, columns=["user_id", "pos_movie_id", "neg_movie_id"])
    out_path = (f"{out_dir}/triplets_10k.parquet" if out_is_gcs else str(Path(out_dir) / "triplets_10k.parquet"))
    df.to_parquet(out_path, storage_options=_storage_options(out_path))
    print(f"Saved {len(df):,} triplets to {out_path}")


if __name__ == "__main__":
    generate_triplets()
