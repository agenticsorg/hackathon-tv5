#!/usr/bin/env bash
set -euo pipefail

# Default to repo-local Cloud SDK config to avoid $HOME perms issues
export CLOUDSDK_CONFIG="${CLOUDSDK_CONFIG:-$(pwd)/.gcloud}"
mkdir -p "$CLOUDSDK_CONFIG"

# Creates core infra: bucket, service accounts, Cloud SQL instance/db/user, database-url secret.
# Usage:
#   PROJECT_ID=... REGION=europe-west2 BUCKET_NAME=... SQL_INSTANCE=... DB_NAME=movies DB_USER=app_user ./scripts/provision_core.sh
# Optional:
#   DB_PASSWORD (auto-generated if empty), AR_REPO (defaults embedding-service), RUNTIME_SA, JOBS_SA

PROJECT_ID=${PROJECT_ID:-$(gcloud config get-value core/project 2>/dev/null)}
REGION=${REGION:-$(gcloud config get-value compute/region 2>/dev/null)}
AR_REPO=${AR_REPO:-embedding-service}
BUCKET_NAME=${BUCKET_NAME:-${PROJECT_ID}-${REGION}-embeddings}
SQL_INSTANCE=${SQL_INSTANCE:-embeddings-sql-${REGION}}
DB_NAME=${DB_NAME:-movies}
DB_USER=${DB_USER:-app_user}
DB_PASSWORD=${DB_PASSWORD:-}
RUNTIME_SA_NAME=${RUNTIME_SA:-embedding-service}
JOBS_SA_NAME=${JOBS_SA:-embedding-jobs}

if [[ -z "$PROJECT_ID" || -z "$REGION" ]]; then
  echo "PROJECT_ID/REGION not set. Set env vars or run 'gcloud config set project/region'." >&2
  exit 1
fi

RUNTIME_SA_EMAIL="${RUNTIME_SA_NAME}@${PROJECT_ID}.iam.gserviceaccount.com"
JOBS_SA_EMAIL="${JOBS_SA_NAME}@${PROJECT_ID}.iam.gserviceaccount.com"

echo "== Summary =="
echo "Project:        $PROJECT_ID"
echo "Region:         $REGION"
echo "AR Repo:        $AR_REPO"
echo "Bucket:         $BUCKET_NAME"
echo "SQL Instance:   $SQL_INSTANCE"
echo "DB:             $DB_NAME"
echo "DB User:        $DB_USER"
echo "Runtime SA:     $RUNTIME_SA_EMAIL"
echo "Jobs SA:        $JOBS_SA_EMAIL"

if [[ -z "$DB_PASSWORD" ]]; then
  if command -v openssl >/dev/null 2>&1; then
    DB_PASSWORD=$(openssl rand -base64 20 | tr -d '=+' | cut -c1-20)
  else
    DB_PASSWORD=$(head -c 24 /dev/urandom | base64 | tr -d '=+' | cut -c1-20)
  fi
  echo "Generated DB password (not printed)."
fi

echo "== Creating service accounts (idempotent) =="
gcloud iam service-accounts describe "$RUNTIME_SA_EMAIL" >/dev/null 2>&1 || \
  gcloud iam service-accounts create "$RUNTIME_SA_NAME" --display-name="Embedding Service Runtime"
gcloud iam service-accounts describe "$JOBS_SA_EMAIL" >/dev/null 2>&1 || \
  gcloud iam service-accounts create "$JOBS_SA_NAME" --display-name="Embedding Jobs"

echo "== Granting roles =="
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:$RUNTIME_SA_EMAIL" --role="roles/cloudsql.client" --quiet >/dev/null
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:$RUNTIME_SA_EMAIL" --role="roles/secretmanager.secretAccessor" --quiet >/dev/null
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:$RUNTIME_SA_EMAIL" --role="roles/artifactregistry.reader" --quiet >/dev/null

gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:$JOBS_SA_EMAIL" --role="roles/cloudsql.client" --quiet >/dev/null
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:$JOBS_SA_EMAIL" --role="roles/secretmanager.secretAccessor" --quiet >/dev/null
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:$JOBS_SA_EMAIL" --role="roles/storage.admin" --quiet >/dev/null

echo "== Creating GCS bucket (idempotent) =="
if gsutil ls -b "gs://$BUCKET_NAME" >/dev/null 2>&1; then
  echo "Bucket exists: gs://$BUCKET_NAME"
else
  gsutil mb -l "$REGION" "gs://$BUCKET_NAME"
fi
gsutil -q cp /dev/null "gs://$BUCKET_NAME/models/.keep" || true
gsutil -q cp /dev/null "gs://$BUCKET_NAME/data/netflix/.keep" || true
gsutil -q cp /dev/null "gs://$BUCKET_NAME/data/tmdb/.keep" || true
gsutil -q cp /dev/null "gs://$BUCKET_NAME/data/processed/.keep" || true
gsutil -q cp /dev/null "gs://$BUCKET_NAME/embeddings/.keep" || true

echo "== Creating Cloud SQL instance/database/user (idempotent) =="
if gcloud sql instances describe "$SQL_INSTANCE" --project "$PROJECT_ID" >/dev/null 2>&1; then
  echo "SQL instance exists: $SQL_INSTANCE"
else
  gcloud sql instances create "$SQL_INSTANCE" \
    --database-version=POSTGRES_15 --cpu=2 --memory=7680MB \
    --region="$REGION" --availability-type=ZONAL --quiet
fi

if gcloud sql databases describe "$DB_NAME" --instance "$SQL_INSTANCE" >/dev/null 2>&1; then
  echo "Database exists: $DB_NAME"
else
  gcloud sql databases create "$DB_NAME" --instance "$SQL_INSTANCE" --quiet
fi

if gcloud sql users list --instance "$SQL_INSTANCE" --format="value(name)" | grep -qx "$DB_USER"; then
  echo "User exists: $DB_USER (updating password)"
  gcloud sql users set-password "$DB_USER" --instance "$SQL_INSTANCE" --password "$DB_PASSWORD" --quiet
else
  gcloud sql users create "$DB_USER" --instance "$SQL_INSTANCE" --password "$DB_PASSWORD" --quiet
fi

INSTANCE_CONNECTION_NAME=$(gcloud sql instances describe "$SQL_INSTANCE" --format='value(connectionName)')
DB_URL="postgresql://$DB_USER:$DB_PASSWORD@/$DB_NAME?host=/cloudsql/$INSTANCE_CONNECTION_NAME"

echo "== Writing database-url secret (idempotent) =="
if gcloud secrets describe database-url >/dev/null 2>&1; then
  echo -n "$DB_URL" | gcloud secrets versions add database-url --data-file=- >/dev/null
else
  gcloud secrets create database-url >/dev/null
  echo -n "$DB_URL" | gcloud secrets versions add database-url --data-file=- >/dev/null
fi

echo "== Summary Outputs =="
echo "BUCKET_NAME=$BUCKET_NAME"
echo "SQL_INSTANCE=$SQL_INSTANCE"
echo "INSTANCE_CONNECTION_NAME=$INSTANCE_CONNECTION_NAME"
echo "DB_NAME=$DB_NAME"
echo "DB_USER=$DB_USER"
echo "DB_PASSWORD=[REDACTED]"
echo "DATABASE_URL stored in Secret Manager: database-url (latest)"
