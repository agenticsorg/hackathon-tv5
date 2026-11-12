#!/usr/bin/env bash
set -euo pipefail

# Default to repo-local Cloud SDK config to avoid $HOME perms issues
export CLOUDSDK_CONFIG="${CLOUDSDK_CONFIG:-$(pwd)/.gcloud}"
mkdir -p "$CLOUDSDK_CONFIG"

SERVICE_NAME=${SERVICE_NAME:-embedding-service}
PROJECT_ID=${PROJECT_ID:?set PROJECT_ID}
REGION=${REGION:-europe-west2}
AR_REPO=${AR_REPO:-embedding-service}
IMAGE=${IMAGE:-$REGION-docker.pkg.dev/$PROJECT_ID/$AR_REPO/api:latest}

INSTANCE_CONNECTION_NAME=${INSTANCE_CONNECTION_NAME:-$(gcloud sql instances describe embeddings-sql-$REGION --format='value(connectionName)' 2>/dev/null || true)}

gcloud run deploy "$SERVICE_NAME" \
  --image "$IMAGE" \
  --project "$PROJECT_ID" \
  --region "$REGION" \
  --platform managed \
  --allow-unauthenticated \
  --cpu 2 --memory 2Gi --max-instances 10 \
  --port 8080 \
  --add-cloudsql-instances "$INSTANCE_CONNECTION_NAME" \
  --set-secrets DATABASE_URL=database-url:latest \
  --set-env-vars ENVIRONMENT=prod,BASE_MODEL_DIR=models/base-minilm${MODEL_GCS_URI:+,MODEL_GCS_URI=${MODEL_GCS_URI}} \
  ${EXTRA_ARGS:-}
