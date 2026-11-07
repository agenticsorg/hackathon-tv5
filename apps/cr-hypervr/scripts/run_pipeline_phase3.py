from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def _is_gcs(p: str) -> bool:
    return str(p).startswith("gs://")


def _parquet_exists(path: str) -> bool:
    try:
        import pandas as pd  # noqa: F401
        # We rely on fsspec to resolve gs://. Just try a metadata read.
        # Using pyarrow, this will error fast if missing.
        pd.read_parquet(path, columns=[], engine="pyarrow")  # type: ignore[arg-type]
        return True
    except Exception:
        return False


def _require_phase2_outputs(processed: str) -> None:
    # Accept either name for the movies parquet (pipeline writes the first)
    movies_candidates = [
        f"{processed}/movies_with_descriptions.parquet",
        f"{processed}/movies_enriched.parquet",
    ]
    profiles = f"{processed}/user_profiles.parquet"
    triplets = f"{processed}/triplets/triplets_10k.parquet"

    has_movies = any(_parquet_exists(m) for m in movies_candidates)
    has_profiles = _parquet_exists(profiles)
    has_triplets = _parquet_exists(triplets)

    if has_movies and has_profiles and has_triplets:
        return

    # Optionally run Phase 2 to produce missing outputs
    if os.getenv("RUN_PHASE2_IF_MISSING", "").lower() in ("1", "true", "yes"): 
        print("Phase 2 outputs missing — invoking scripts/run_pipeline_phase2.py ...")
        env = os.environ.copy()
        # Respect GCS_* envs if user set them
        subprocess.check_call(["python", "scripts/run_pipeline_phase2.py"], env=env)
        # Re-check
        has_movies = any(_parquet_exists(m) for m in movies_candidates)
        has_profiles = _parquet_exists(profiles)
        has_triplets = _parquet_exists(triplets)
        if has_movies and has_profiles and has_triplets:
            return

    missing = []
    if not has_movies:
        missing.append("movies_with_descriptions.parquet")
    if not has_profiles:
        missing.append("user_profiles.parquet")
    if not has_triplets:
        missing.append("triplets/triplets_10k.parquet")
    raise SystemExit(
        "Phase 3 requires Phase 2 outputs. Missing: " + ", ".join(missing)
    )


def main() -> None:
    # Locations
    processed = (
        os.getenv("GCS_PROCESSED_PREFIX")
        or os.getenv("PROCESSED_DIR")
        or os.getenv("PROCESSED_PREFIX")
        or "data/processed"
    )
    base_model_dir = os.getenv("BASE_MODEL_DIR", "models/base-minilm")
    output_dir = os.getenv("OUTPUT_DIR", "models/movie-minilm-v1")

    # Validate inputs; if using local FS, ensure directories exist
    if not _is_gcs(processed):
        Path(processed).mkdir(parents=True, exist_ok=True)
    if not _is_gcs(base_model_dir):
        Path(base_model_dir).mkdir(parents=True, exist_ok=True)
    Path(output_dir).mkdir(parents=True, exist_ok=True)

    # Ensure Phase 2 outputs are present (or produce them if allowed)
    _require_phase2_outputs(processed)

    # Training config
    epochs = os.getenv("EPOCHS", "1")
    batch_size = os.getenv("BATCH_SIZE", "64")
    use_triplet = os.getenv("USE_TRIPLET", os.getenv("USE_TRIPLET_LOSS", "0"))

    env = os.environ.copy()
    env.update(
        {
            "BASE_MODEL_DIR": base_model_dir,
            "PROCESSED_DIR": processed,
            "OUTPUT_DIR": output_dir,
            "EPOCHS": str(epochs),
            "BATCH_SIZE": str(batch_size),
            "USE_TRIPLET": str(use_triplet or "0"),
        }
    )

    if os.getenv("SKIP_TRAIN", "").lower() in ("1", "true", "yes"):
        print("[Phase 3] Skipping fine-tuning per SKIP_TRAIN")
    else:
        print("[Phase 3] Starting fine-tuning…")
        subprocess.check_call([sys.executable, "training/train_finetune.py"], env=env)

    if os.getenv("SKIP_ONNX_EXPORT", "").lower() not in ("1", "true", "yes"):
        print("[Phase 3] Exporting ONNX…")
        subprocess.check_call([sys.executable, "training/onnx_export.py"], env=env)
    else:
        print("[Phase 3] Skipping ONNX export per SKIP_ONNX_EXPORT")

    if os.getenv("SKIP_QUANTIZE", "").lower() not in ("1", "true", "yes"):
        print("[Phase 3] Quantizing ONNX to INT8…")
        subprocess.check_call([sys.executable, "training/quantize_int8.py"], env=env)
    else:
        print("[Phase 3] Skipping quantization per SKIP_QUANTIZE")

    print("[Phase 3] Completed. Artifacts under:")
    print(f"  - output_dir = {output_dir}")
    print(f"  - model.onnx and model-int8.onnx if export/quantize enabled")

    # Optional: upload artifacts to GCS
    upload_uri = (
        os.getenv("MODEL_UPLOAD_URI")
        or os.getenv("GCS_MODEL_UPLOAD_URI")
        or (
            f"{os.getenv('GCS_MODELS_BUCKET').rstrip('/')}/models/movie-minilm-v1"
            if os.getenv("GCS_MODELS_BUCKET")
            else None
        )
    )
    if upload_uri and upload_uri.startswith("gs://"):
        try:
            import fsspec
            from pathlib import PurePosixPath

            fs = fsspec.filesystem("gcs")
            print(f"[Phase 3] Uploading artifacts to {upload_uri} …")
            base = Path(output_dir)
            for local in base.rglob("*"):
                if local.is_dir():
                    continue
                rel = local.relative_to(base)
                tgt = str(PurePosixPath(upload_uri.strip("/")) / str(rel))
                # Ensure parent dir on GCS
                parent = str(PurePosixPath(tgt).parent)
                try:
                    fs.mkdir(parent)
                except Exception:
                    pass
                with open(local, "rb") as fsrc, fs.open(tgt, "wb") as fdst:  # type: ignore[attr-defined]
                    fdst.write(fsrc.read())
            print("[Phase 3] Upload complete.")
        except Exception as e:
            print(f"[Phase 3] WARN: Upload to GCS failed: {e}")
    elif upload_uri:
        print(f"[Phase 3] WARN: Unsupported upload URI: {upload_uri}")


if __name__ == "__main__":
    main()
