#!/usr/bin/env bash
set -euo pipefail

# Default to repo-local Cloud SDK config to avoid $HOME perms issues
export CLOUDSDK_CONFIG="${CLOUDSDK_CONFIG:-$(pwd)/.gcloud}"
mkdir -p "$CLOUDSDK_CONFIG"

PROJECT_ID=${PROJECT_ID:-$(gcloud config get-value core/project 2>/dev/null)}
REGION=${REGION:-$(gcloud config get-value compute/region 2>/dev/null)}

echo "Project: ${PROJECT_ID}"
echo "Region:  ${REGION}"

echo "== Enabled services (key set) =="
for s in run.googleapis.com cloudbuild.googleapis.com artifactregistry.googleapis.com sqladmin.googleapis.com secretmanager.googleapis.com vpcaccess.googleapis.com storage.googleapis.com compute.googleapis.com; do
  printf "%-35s : " "$s"; gcloud services list --enabled --filter="NAME:$s" --format='value(NAME)' || true
done

echo "== Artifact Registry (europe-west2) =="
gcloud artifacts repositories list --location=europe-west2 --format='table(name,format,location)'

echo "== Service Accounts =="
gcloud iam service-accounts list --format='table(displayName,email)'

echo "== Bucket exists? =="
# Default to new datasets bucket with 20251207 suffix unless BUCKET_NAME provided
BUCKET_NAME=${BUCKET_NAME:-${PROJECT_ID}-europe-west2-datasets-20251207}
if gsutil ls -b gs://$BUCKET_NAME >/dev/null 2>&1; then
  echo "YES: gs://$BUCKET_NAME"
else
  echo "NO: (expected: gs://$BUCKET_NAME)"
fi

echo "== Cloud SQL instance =="
SQL_INSTANCE=${SQL_INSTANCE:-embeddings-sql-europe-west2}
gcloud sql instances describe "$SQL_INSTANCE" --format='table(name,region,state,backendType)' || true
echo "== Databases =="
gcloud sql databases list --instance="$SQL_INSTANCE" --format='table(name)'
echo "== Users =="
gcloud sql users list --instance="$SQL_INSTANCE" --format='table(name,type)'

echo "== Secrets =="
gcloud secrets list --format='table(name)' | sed -n '1,200p'
