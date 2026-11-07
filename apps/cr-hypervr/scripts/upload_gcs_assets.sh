#!/usr/bin/env bash
set -euo pipefail

# Robust uploader with verification and detailed logs.
# Usage (pass explicit buckets; defaults still point to ${PROJECT_ID}-${REGION}-datasets/models):
#   PROJECT_ID=agentics-foundation25lon-1809 REGION=europe-west2 \
#   DATA_BUCKET=gs://agentics-foundation25lon-1809-europe-west2-datasets-20251207 \
#   MODEL_BUCKET=gs://agentics-foundation25lon-1809-europe-west2-models-20251207 \
#   bash scripts/upload_gcs_assets.sh

ts() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

PROJECT_ID=${PROJECT_ID:-agentics-foundation25lon-1809}
REGION=${REGION:-europe-west2}
DATA_BUCKET=${DATA_BUCKET:-gs://${PROJECT_ID}-${REGION}-datasets}
MODEL_BUCKET=${MODEL_BUCKET:-gs://${PROJECT_ID}-${REGION}-models}

GSDBG=${GSDBG:-0}
GSFLAGS=(-m)
if [[ "$GSDBG" == "1" ]]; then
  GSFLAGS=(-m -D)
fi

echo "[$(ts)] Start upload to GCS"
echo "Project:   $PROJECT_ID"
echo "Region:    $REGION"
echo "Data bkt:  $DATA_BUCKET"
echo "Model bkt: $MODEL_BUCKET"

ensure_bucket() {
  local B=$1
  if gsutil ls -b "$B" >/dev/null 2>&1; then
    echo "[$(ts)] Bucket exists: $B"
  else
    echo "[$(ts)] Creating bucket: $B"
    gsutil mb -l "$REGION" "$B"
  fi
}

ensure_bucket "$DATA_BUCKET"
ensure_bucket "$MODEL_BUCKET"

local_count_size() {
  local P=$1
  local cnt size
  cnt=$(find "$P" -type f | wc -l | tr -d ' ')
  size=$(du -sk "$P" | awk '{print $1}') # KiB
  echo "$cnt files, ${size}KiB"
}

remote_count_size() {
  local U=$1
  local cnt size
  cnt=$(gsutil ls -r "$U" 2>/dev/null | grep -v '/$' | wc -l | tr -d ' ')
  size=$(gsutil du -s "$U" 2>/dev/null | awk '{print $1}')
  echo "$cnt objects, ${size}B"
}

upload_dir() {
  local SRC=$1 DST=$2
  echo "[$(ts)] Uploading directory: $SRC -> $DST"
  echo "         Local:  $(local_count_size "$SRC")"
  gsutil "${GSFLAGS[@]}" rsync -r -c "$SRC" "$DST"
  echo "         Remote: $(remote_count_size "$DST")"
}

upload_file_verify() {
  local SRC=$1 DST=$2  # DST ends with / or object path
  local base=$(basename "$SRC")
  local OBJ=$DST
  if [[ "$DST" =~ /$ ]]; then OBJ="${DST}${base}"; fi
  echo "[$(ts)] Uploading file: $SRC -> $OBJ"
  gsutil "${GSFLAGS[@]}" cp -n "$SRC" "$OBJ"
  # Verify MD5 if available
  if command -v openssl >/dev/null 2>&1; then
    local lmd5; lmd5=$(openssl md5 -binary "$SRC" | base64 | tr -d '[:space:]')
    local rmd5; rmd5=$(gsutil stat "$OBJ" | awk -F": " '/Hash \(md5\)/{print $2}' | tr -d '[:space:]')
    echo "         Local MD5:  $lmd5"
    echo "         Remote MD5: $rmd5"
    if [[ -n "$rmd5" && "$lmd5" == "$rmd5" ]]; then
      echo "         Verify: OK"
    else
      echo "         Verify: MISMATCH or unavailable" >&2
      return 1
    fi
  else
    echo "         MD5 verify skipped (openssl not found)"
  fi
}

# Upload TMDB CSV
if [[ -f data/tmdb/TMDB_movie_dataset_v11.csv ]]; then
  upload_file_verify data/tmdb/TMDB_movie_dataset_v11.csv "$DATA_BUCKET/data/tmdb/"
else
  echo "[$(ts)] WARN: TMDB CSV missing locally; skipping" >&2
fi

# Upload MovieLens directory
if [[ -d data/movielens/ml-25m ]]; then
  upload_dir data/movielens/ml-25m "$DATA_BUCKET/data/movielens/ml-25m/"
else
  echo "[$(ts)] WARN: MovieLens directory missing; skipping" >&2
fi

# Upload base MiniLM model directory
if [[ -d models/base-minilm ]]; then
  upload_dir models/base-minilm "$MODEL_BUCKET/models/base-minilm/"
else
  echo "[$(ts)] WARN: base-minilm directory missing; skipping" >&2
fi

echo "[$(ts)] Upload completed"
