#!/usr/bin/env bash
set -euo pipefail

# Deploy Cloud Run Jobs for data join and phase2 pipeline.
# Requirements: image pushed to Artifact Registry and service account for jobs.
# Usage:
#   PROJECT_ID=... REGION=europe-west2 AR_REPO=embedding-service JOBS_SA=embedding-jobs \
#   DATA_PREFIX=gs://<bucket>/data PROCESSED_PREFIX=gs://<bucket>/data/processed \
#   TRIPLETS_PREFIX=gs://<bucket>/triplets PROFILES_PATH=gs://<bucket>/data/processed/user_profiles.parquet \
#   ./scripts/deploy_jobs.sh

PROJECT_ID=${PROJECT_ID:?set PROJECT_ID}
REGION=${REGION:-europe-west2}
AR_REPO=${AR_REPO:-embedding-service}
IMAGE=${IMAGE:-$REGION-docker.pkg.dev/$PROJECT_ID/$AR_REPO/api:latest}
JOBS_SA=${JOBS_SA:-embedding-jobs@${PROJECT_ID}.iam.gserviceaccount.com}
INSTANCE_CONNECTION_NAME=${INSTANCE_CONNECTION_NAME:-$(gcloud sql instances describe ${SQL_INSTANCE:-embeddings-sql-${REGION}} --format='value(connectionName)' 2>/dev/null || true)}

# Default to stable bucket envs if explicit prefixes not provided
DATA_PREFIX=${DATA_PREFIX:-${GCS_DATA_BUCKET}/data}
PROCESSED_PREFIX=${PROCESSED_PREFIX:-${GCS_EMB_BUCKET}/data/processed}
TRIPLETS_PREFIX=${TRIPLETS_PREFIX:-${GCS_EMB_BUCKET}/triplets}
PROFILES_PATH=${PROFILES_PATH:-${GCS_EMB_BUCKET}/data/processed/user_profiles.parquet}
SERVICE_URL=${SERVICE_URL:-$(gcloud run services describe embedding-service --project "$PROJECT_ID" --region="$REGION" --format='value(status.url)' 2>/dev/null || true)}

echo "Image:        $IMAGE"
echo "Jobs SA:      $JOBS_SA"
echo "Cloud SQL:    ${INSTANCE_CONNECTION_NAME:-[unset]}"
echo "Data prefix:  ${DATA_PREFIX:-[unset]}"
echo "Processed:    ${PROCESSED_PREFIX:-[unset]}"
echo "Triplets:     ${TRIPLETS_PREFIX:-[unset]}"
echo "Profiles:     ${PROFILES_PATH:-[unset]}"
echo "Service URL:  ${SERVICE_URL:-[unset]}"

common_env=(
  --set-env-vars PYTHONPATH=/app
  --set-env-vars GCS_DATA_PREFIX=${DATA_PREFIX}
  --set-env-vars GCS_PROCESSED_PREFIX=${PROCESSED_PREFIX}
  --set-env-vars GCS_TRIPLETS_PREFIX=${TRIPLETS_PREFIX}
  --set-env-vars GCS_PROFILES_PATH=${PROFILES_PATH}
  --set-env-vars GCS_MODELS_BUCKET=${GCS_MODELS_BUCKET:-}
  --set-env-vars TRIPLET_USER_SAMPLE=${TRIPLET_USER_SAMPLE:-all}
)

echo "Deploying job: data-join"
gcloud run jobs deploy data-join \
  --image "$IMAGE" \
  --project "$PROJECT_ID" \
  --region "$REGION" \
  --service-account "$JOBS_SA" \
  --cpu 2 --memory 4Gi \
  --task-timeout 3600 \
  --max-retries 1 \
  --command python \
  --args scripts/join_datasets.py \
  "${common_env[@]}"

echo "Deploying job: pipeline-phase2"
gcloud run jobs deploy pipeline-phase2 \
  --image "$IMAGE" \
  --project "$PROJECT_ID" \
  --region "$REGION" \
  --service-account "$JOBS_SA" \
  --cpu 4 --memory 8Gi \
  --task-timeout 14400 \
  --max-retries 1 \
  ${INSTANCE_CONNECTION_NAME:+--set-cloudsql-instances "$INSTANCE_CONNECTION_NAME"} \
  --set-secrets DATABASE_URL=database-url:latest \
  --command python \
  --args scripts/run_pipeline_phase2.py \
  --set-env-vars PROCESSED_PREFIX=${PROCESSED_PREFIX} \
  "${common_env[@]}"

echo "Deploying job: pipeline-phase3"
gcloud run jobs deploy pipeline-phase3 \
  --image "$IMAGE" \
  --project "$PROJECT_ID" \
  --region "$REGION" \
  --service-account "$JOBS_SA" \
  --cpu 4 --memory 8Gi \
  --task-timeout 21600 \
  --max-retries 1 \
  --set-env-vars BASE_MODEL_DIR=${BASE_MODEL_DIR:-models/base-minilm} \
  --set-env-vars OUTPUT_DIR=${OUTPUT_DIR:-models/movie-minilm-v1} \
  --set-env-vars EPOCHS=${EPOCHS:-1} \
  --set-env-vars BATCH_SIZE=${BATCH_SIZE:-64} \
  --set-env-vars RUN_PHASE2_IF_MISSING=${RUN_PHASE2_IF_MISSING:-false} \
  --command python \
  --args scripts/run_pipeline_phase3.py \
  "${common_env[@]}"

echo "Jobs deployed. Use: gcloud run jobs run <name> --region=$REGION --wait"

echo "Deploying job: seed-movies"
gcloud run jobs deploy seed-movies \
  --image "$IMAGE" \
  --project "$PROJECT_ID" \
  --region "$REGION" \
  --service-account "$JOBS_SA" \
  --cpu 2 --memory 2Gi \
  --task-timeout 3600 \
  --max-retries 1 \
  ${INSTANCE_CONNECTION_NAME:+--set-cloudsql-instances "$INSTANCE_CONNECTION_NAME"} \
  --set-secrets DATABASE_URL=database-url:latest \
  --set-env-vars PROCESSED_PREFIX=${PROCESSED_PREFIX} \
  --set-env-vars GCS_PROCESSED_PREFIX=${PROCESSED_PREFIX} \
  --command python \
  --args scripts/seed_movies.py

echo "Deploying job: seed-embeddings"
gcloud run jobs deploy seed-embeddings \
  --image "$IMAGE" \
  --project "$PROJECT_ID" \
  --region "$REGION" \
  --service-account "$JOBS_SA" \
  --cpu 4 --memory 8Gi \
  --task-timeout 14400 \
  --max-retries 1 \
  ${INSTANCE_CONNECTION_NAME:+--set-cloudsql-instances "$INSTANCE_CONNECTION_NAME"} \
  --set-secrets DATABASE_URL=database-url:latest \
  --set-env-vars PROCESSED_PREFIX=${PROCESSED_PREFIX} \
  --set-env-vars GCS_PROCESSED_PREFIX=${PROCESSED_PREFIX} \
  --set-env-vars SERVICE_URL=${SERVICE_URL} \
  --set-env-vars BATCH_EMBED_SIZE=${BATCH_EMBED_SIZE:-256} \
  --set-env-vars UPSERT_CHUNK_SIZE=${UPSERT_CHUNK_SIZE:-1000} \
  --set-env-vars MOVIES_ROW_CHUNK=${MOVIES_ROW_CHUNK:-5000} \
  --set-env-vars MODEL_DIR=models/base-minilm \
  --set-env-vars EMBEDDING_BACKEND=st \
  --command python \
  --args scripts/seed_embeddings.py

echo "Deploying job: backfill-embeddings-db"
gcloud run jobs deploy backfill-embeddings-db \
  --image "$IMAGE" \
  --project "$PROJECT_ID" \
  --region "$REGION" \
  --service-account "$JOBS_SA" \
  --cpu 2 --memory 4Gi \
  --task-timeout 14400 \
  --max-retries 1 \
  ${INSTANCE_CONNECTION_NAME:+--set-cloudsql-instances "$INSTANCE_CONNECTION_NAME"} \
  --set-secrets DATABASE_URL=database-url:latest \
  --set-env-vars SERVICE_URL=${SERVICE_URL} \
  --set-env-vars EMBEDDING_BACKEND=st \
  --command python \
  --args scripts/backfill_embeddings_db.py

echo "Deploying job: validate-triplets"
gcloud run jobs deploy validate-triplets \
  --image "$IMAGE" \
  --project "$PROJECT_ID" \
  --region "$REGION" \
  --service-account "$JOBS_SA" \
  --cpu 1 --memory 1Gi \
  --task-timeout 1800 \
  --max-retries 1 \
  ${INSTANCE_CONNECTION_NAME:+--set-cloudsql-instances "$INSTANCE_CONNECTION_NAME"} \
  --set-secrets DATABASE_URL=database-url:latest \
  --set-env-vars GCS_TRIPLETS_PREFIX=${TRIPLETS_PREFIX} \
  --command python \
  --args scripts/validate_triplets_coverage.py

echo "Deploying job: validate-hyperedges"
gcloud run jobs deploy validate-hyperedges \
  --image "$IMAGE" \
  --project "$PROJECT_ID" \
  --region "$REGION" \
  --service-account "$JOBS_SA" \
  --cpu 1 --memory 2Gi \
  --task-timeout 3600 \
  --max-retries 1 \
  ${INSTANCE_CONNECTION_NAME:+--set-cloudsql-instances "$INSTANCE_CONNECTION_NAME"} \
  --set-secrets DATABASE_URL=database-url:latest \
  --set-env-vars PROCESSED_PREFIX=${PROCESSED_PREFIX} \
  --set-env-vars GCS_PROCESSED_PREFIX=${PROCESSED_PREFIX} \
  --command python \
  --args scripts/validate_hyperedges.py

echo "Deploying job: build-hyperedges"
gcloud run jobs deploy build-hyperedges \
  --image "$IMAGE" \
  --project "$PROJECT_ID" \
  --region "$REGION" \
  --service-account "$JOBS_SA" \
  --cpu 4 --memory 8Gi \
  --task-timeout 10800 \
  --max-retries 1 \
  ${INSTANCE_CONNECTION_NAME:+--set-cloudsql-instances "$INSTANCE_CONNECTION_NAME"} \
  --set-secrets DATABASE_URL=database-url:latest \
  --set-env-vars PROCESSED_PREFIX=${PROCESSED_PREFIX} \
  --set-env-vars GCS_PROCESSED_PREFIX=${PROCESSED_PREFIX} \
  --command python \
  --args scripts/build_hyperedges.py
