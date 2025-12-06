#!/bin/bash
# Nexus-UMMID Cloud Run Deployment Script
set -e

# Source gcloud
source /root/google-cloud-sdk/path.bash.inc

PROJECT_ID="agentics-foundation25lon-1899"
REGION="us-central1"
SERVICE_NAME="nexus-ummid-api"
IMAGE_NAME="nexus-ummid-api"

echo "🚀 Nexus-UMMID Cloud Run Deployment"
echo "===================================="
echo "Project: $PROJECT_ID"
echo "Region: $REGION"
echo "Service: $SERVICE_NAME"
echo ""

# Get access token
echo "🔑 Obtaining access token..."
ACCESS_TOKEN=$(gcloud auth application-default print-access-token 2>/dev/null)
if [ -z "$ACCESS_TOKEN" ]; then
    echo "❌ Failed to get access token"
    exit 1
fi
echo "✅ Access token obtained"

# Enable APIs
echo ""
echo "📡 Enabling required APIs..."

for API in run.googleapis.com artifactregistry.googleapis.com cloudbuild.googleapis.com; do
    echo "  - Enabling $API..."
    RESULT=$(curl -s -X POST "https://serviceusage.googleapis.com/v1/projects/$PROJECT_ID/services/$API:enable" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        -H "Content-Type: application/json")
    if echo "$RESULT" | grep -q "error"; then
        echo "    ⚠️  $(echo $RESULT | jq -r '.error.message // "Unknown error"')"
    else
        echo "    ✅ Enabled"
    fi
done

# Wait for APIs to propagate
echo ""
echo "⏳ Waiting for APIs to activate (10s)..."
sleep 10

# Create Artifact Registry repository
echo ""
echo "📦 Creating Artifact Registry repository..."
REPO_RESULT=$(curl -s -X POST "https://artifactregistry.googleapis.com/v1/projects/$PROJECT_ID/locations/$REGION/repositories?repositoryId=nexus-ummid" \
    -H "Authorization: Bearer $ACCESS_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "format": "DOCKER",
        "description": "Nexus-UMMID container images"
    }')

if echo "$REPO_RESULT" | grep -q "already exists"; then
    echo "✅ Repository already exists"
elif echo "$REPO_RESULT" | grep -q "error"; then
    echo "⚠️  $(echo $REPO_RESULT | jq -r '.error.message // "Repository creation issue"')"
else
    echo "✅ Repository created"
fi

# Configure Docker for Artifact Registry
echo ""
echo "🐳 Configuring Docker authentication..."
gcloud auth configure-docker $REGION-docker.pkg.dev --quiet 2>/dev/null || true

# Build image using Cloud Build
echo ""
echo "🔨 Building container image with Cloud Build..."
cd /home/user/hackathon-tv5/apps/metadata-api

# Submit build
BUILD_RESULT=$(curl -s -X POST "https://cloudbuild.googleapis.com/v1/projects/$PROJECT_ID/builds" \
    -H "Authorization: Bearer $ACCESS_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
        \"source\": {
            \"storageSource\": {
                \"bucket\": \"${PROJECT_ID}_cloudbuild\",
                \"object\": \"source.tar.gz\"
            }
        },
        \"steps\": [
            {
                \"name\": \"gcr.io/cloud-builders/docker\",
                \"args\": [\"build\", \"-t\", \"$REGION-docker.pkg.dev/$PROJECT_ID/nexus-ummid/$IMAGE_NAME:latest\", \".\"]
            },
            {
                \"name\": \"gcr.io/cloud-builders/docker\",
                \"args\": [\"push\", \"$REGION-docker.pkg.dev/$PROJECT_ID/nexus-ummid/$IMAGE_NAME:latest\"]
            }
        ],
        \"images\": [\"$REGION-docker.pkg.dev/$PROJECT_ID/nexus-ummid/$IMAGE_NAME:latest\"]
    }")

echo "Build submitted: $(echo $BUILD_RESULT | jq -r '.metadata.build.id // "check Cloud Console"')"

echo ""
echo "✅ Deployment script completed!"
echo ""
echo "📋 Next steps:"
echo "  1. Check Cloud Build status in GCP Console"
echo "  2. Deploy to Cloud Run when build completes:"
echo "     gcloud run deploy $SERVICE_NAME \\"
echo "       --image $REGION-docker.pkg.dev/$PROJECT_ID/nexus-ummid/$IMAGE_NAME:latest \\"
echo "       --region $REGION \\"
echo "       --allow-unauthenticated"
