#!/usr/bin/env bash
set -euo pipefail

# Creates/updates Kaggle (and optionally TMDB) secrets.
# Usage:
#   KAGGLE_JSON=$HOME/.kaggle/kaggle.json ./scripts/setup_secrets.sh
# Optional (not required with TMDB Kaggle dataset):
#   TMDB_API_KEY=... ./scripts/setup_secrets.sh

if [[ -n "${TMDB_API_KEY:-}" ]]; then
  if gcloud secrets describe tmdb-api-key >/dev/null 2>&1; then
    echo -n "$TMDB_API_KEY" | gcloud secrets versions add tmdb-api-key --data-file=- >/dev/null
  else
    gcloud secrets create tmdb-api-key >/dev/null
    echo -n "$TMDB_API_KEY" | gcloud secrets versions add tmdb-api-key --data-file=- >/dev/null
  fi
  echo "Updated secret: tmdb-api-key"
else
  echo "TMDB_API_KEY not set; skipping (not required if using TMDB Kaggle dataset)."
fi

if [[ -n "${KAGGLE_JSON:-}" ]]; then
  if [[ ! -f "$KAGGLE_JSON" ]]; then
    echo "KAGGLE_JSON path does not exist: $KAGGLE_JSON" >&2
    exit 1
  fi
  if gcloud secrets describe kaggle-credentials >/dev/null 2>&1; then
    gcloud secrets versions add kaggle-credentials --data-file="$KAGGLE_JSON" >/dev/null
  else
    gcloud secrets create kaggle-credentials >/dev/null
    gcloud secrets versions add kaggle-credentials --data-file="$KAGGLE_JSON" >/dev/null
  fi
  echo "Updated secret: kaggle-credentials"
else
  echo "KAGGLE_JSON not set; skipping."
fi
