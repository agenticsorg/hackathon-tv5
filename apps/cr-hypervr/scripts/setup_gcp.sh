#!/usr/bin/env bash
set -euo pipefail

PROJECT_ID=${PROJECT_ID:?set PROJECT_ID}
REGION=${REGION:-europe-west2}

echo "Setting project and region..."
gcloud config set project "$PROJECT_ID"
gcloud config set compute/region "$REGION"

echo "Enabling required services..."
gcloud services enable \
  run.googleapis.com \
  cloudbuild.googleapis.com \
  artifactregistry.googleapis.com \
  sqladmin.googleapis.com \
  secretmanager.googleapis.com \
  vpcaccess.googleapis.com \
  storage.googleapis.com \
  compute.googleapis.com

echo "Setup complete. Create bucket/SQL and deploy per mainPRD.md."
