from pipeline.user_profiles import build_user_profiles
from pipeline.triplets import generate_triplets
import subprocess
import sys
import os


def main():
    # Assume data-join job produced movies_with_descriptions + ratings_enriched-*.parquet
    # If not present, attempt a join on a small chunk by setting a tiny chunksize to reduce memory.
    processed = os.getenv("GCS_PROCESSED_PREFIX", "data/processed")
    data_prefix = os.getenv("GCS_DATA_PREFIX")
    # Heuristic: if movies parquet is missing, run join (chunked)
    need_join = False
    import pandas as pd
    from pathlib import Path
    movies_path = f"{processed}/movies_with_descriptions.parquet" if str(processed).startswith("gs://") else Path(processed) / "movies_with_descriptions.parquet"
    try:
        pd.read_parquet(movies_path, storage_options={"token": "cloud"} if str(movies_path).startswith("gs://") else None)
    except Exception:
        need_join = True
    if need_join:
        os.environ.setdefault("JOIN_RATINGS_CHUNKSIZE", "250000")
        subprocess.check_call([sys.executable, "scripts/join_datasets.py"])  # chunked join

    profiles_path = os.getenv(
        "GCS_PROFILES_PATH",
        (processed + "/user_profiles.parquet") if str(processed).startswith("gs://") else "data/processed/user_profiles.parquet",
    )
    min_ratings = int(os.getenv("MIN_RATINGS", "10"))
    build_user_profiles(processed_dir=processed, out_path=profiles_path, min_ratings=min_ratings)
    triplets_out = os.getenv("GCS_TRIPLETS_PREFIX", "data/processed/triplets")
    # Allow overriding triplet sample size. Set TRIPLET_USER_SAMPLE=all for full dataset.
    samp_env = os.getenv("TRIPLET_USER_SAMPLE", "10000")
    user_sample = None if str(samp_env).lower() in ("all", "none", "0", "-1") else int(samp_env)
    generate_triplets(processed_dir=processed, out_dir=triplets_out, user_sample=user_sample)

    # Always build and validate hyperedges at the end of Phase 2
    os.environ.setdefault("PROCESSED_PREFIX", processed)
    subprocess.check_call([sys.executable, "scripts/build_hyperedges.py"])  # writes parquet (+optional DB insert)
    subprocess.check_call([sys.executable, "scripts/validate_hyperedges.py"])  # exits non-zero on mismatch


if __name__ == "__main__":
    main()
