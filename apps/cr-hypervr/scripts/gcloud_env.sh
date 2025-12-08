#!/usr/bin/env bash
set -euo pipefail

# Use a repo-local Cloud SDK config to avoid $HOME permission issues.
export CLOUDSDK_CONFIG="${CLOUDSDK_CONFIG:-$(pwd)/.gcloud}"
mkdir -p "$CLOUDSDK_CONFIG"

if [[ -n "${PROJECT_ID:-}" ]]; then
  gcloud config set core/project "$PROJECT_ID" >/dev/null
fi
if [[ -n "${REGION:-}" ]]; then
  gcloud config set compute/region "$REGION" >/dev/null
fi

echo "CLOUDSDK_CONFIG=$CLOUDSDK_CONFIG"
gcloud config list 2>/dev/null || true

# Stable bucket envs (point to existing dated buckets by default; no reupload)
# Users may export these to override.
PROJECT_ID=${PROJECT_ID:-$(gcloud config get-value core/project 2>/dev/null)}
REGION=${REGION:-$(gcloud config get-value compute/region 2>/dev/null)}
DATE_SUFFIX=${DATE_SUFFIX:-20251207}

export GCS_DATA_BUCKET=${GCS_DATA_BUCKET:-gs://${PROJECT_ID}-${REGION}-datasets-${DATE_SUFFIX}}
export GCS_MODELS_BUCKET=${GCS_MODELS_BUCKET:-gs://${PROJECT_ID}-${REGION}-models-${DATE_SUFFIX}}
export GCS_EMB_BUCKET=${GCS_EMB_BUCKET:-gs://${PROJECT_ID}-${REGION}-embeddings-${DATE_SUFFIX}}

echo "Buckets:"
echo "  GCS_DATA_BUCKET=$GCS_DATA_BUCKET"
echo "  GCS_MODELS_BUCKET=$GCS_MODELS_BUCKET"
echo "  GCS_EMB_BUCKET=$GCS_EMB_BUCKET"
