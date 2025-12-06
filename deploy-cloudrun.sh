#!/bin/bash
# EmotiStream - Google Cloud Run Deployment Script
#
# Prerequisites:
#   1. Google Cloud SDK installed: https://cloud.google.com/sdk/docs/install
#   2. Docker installed
#   3. Run: gcloud auth login (opens browser for authentication)
#
# Usage:
#   ./deploy-cloudrun.sh [PROJECT_ID] [REGION]
#
# Example:
#   ./deploy-cloudrun.sh my-hackathon-project us-central1

set -e

# Configuration
PROJECT_ID="${1:-emotistream-hackathon}"
REGION="${2:-us-central1}"
BACKEND_SERVICE="emotistream-api"
FRONTEND_SERVICE="emotistream-web"

echo "=========================================="
echo "EmotiStream Cloud Run Deployment"
echo "=========================================="
echo "Project: $PROJECT_ID"
echo "Region: $REGION"
echo ""

# Check if gcloud is installed
if ! command -v gcloud &> /dev/null; then
    echo "ERROR: gcloud CLI not installed"
    echo "Install from: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Check if user is authenticated
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | head -n1 > /dev/null 2>&1; then
    echo "Not authenticated. Running gcloud auth login..."
    gcloud auth login
fi

# Set project
echo "Setting project to $PROJECT_ID..."
gcloud config set project $PROJECT_ID

# Enable required APIs
echo "Enabling required APIs..."
gcloud services enable cloudbuild.googleapis.com
gcloud services enable run.googleapis.com
gcloud services enable artifactregistry.googleapis.com

# Create Artifact Registry repository if it doesn't exist
echo "Setting up Artifact Registry..."
gcloud artifacts repositories create emotistream \
    --repository-format=docker \
    --location=$REGION \
    --description="EmotiStream Docker images" \
    2>/dev/null || echo "Repository already exists"

# Configure Docker for Artifact Registry
echo "Configuring Docker authentication..."
gcloud auth configure-docker ${REGION}-docker.pkg.dev --quiet

# ==========================================
# Deploy Backend
# ==========================================
echo ""
echo "Building and deploying backend..."
cd apps/emotistream

# Build and push backend image
docker build -t ${REGION}-docker.pkg.dev/${PROJECT_ID}/emotistream/${BACKEND_SERVICE}:latest .
docker push ${REGION}-docker.pkg.dev/${PROJECT_ID}/emotistream/${BACKEND_SERVICE}:latest

# Deploy to Cloud Run
gcloud run deploy $BACKEND_SERVICE \
    --image ${REGION}-docker.pkg.dev/${PROJECT_ID}/emotistream/${BACKEND_SERVICE}:latest \
    --platform managed \
    --region $REGION \
    --allow-unauthenticated \
    --memory 512Mi \
    --cpu 1 \
    --min-instances 0 \
    --max-instances 10 \
    --port 8080 \
    --set-env-vars "NODE_ENV=production"

# Get backend URL
BACKEND_URL=$(gcloud run services describe $BACKEND_SERVICE --region $REGION --format="value(status.url)")
echo "Backend deployed at: $BACKEND_URL"

cd ../..

# ==========================================
# Deploy Frontend
# ==========================================
echo ""
echo "Building and deploying frontend..."
cd apps/emotistream-web

# Copy Cloud Run config
cp next.config.cloudrun.ts next.config.ts

# Build with backend URL
docker build \
    --build-arg NEXT_PUBLIC_API_URL="${BACKEND_URL}/api/v1" \
    -t ${REGION}-docker.pkg.dev/${PROJECT_ID}/emotistream/${FRONTEND_SERVICE}:latest .

docker push ${REGION}-docker.pkg.dev/${PROJECT_ID}/emotistream/${FRONTEND_SERVICE}:latest

# Deploy to Cloud Run
gcloud run deploy $FRONTEND_SERVICE \
    --image ${REGION}-docker.pkg.dev/${PROJECT_ID}/emotistream/${FRONTEND_SERVICE}:latest \
    --platform managed \
    --region $REGION \
    --allow-unauthenticated \
    --memory 512Mi \
    --cpu 1 \
    --min-instances 0 \
    --max-instances 10 \
    --port 8080

# Get frontend URL
FRONTEND_URL=$(gcloud run services describe $FRONTEND_SERVICE --region $REGION --format="value(status.url)")

cd ../..

# ==========================================
# Summary
# ==========================================
echo ""
echo "=========================================="
echo "Deployment Complete!"
echo "=========================================="
echo ""
echo "Frontend: $FRONTEND_URL"
echo "Backend:  $BACKEND_URL"
echo "API:      ${BACKEND_URL}/api/v1"
echo ""
echo "Test endpoints:"
echo "  curl ${BACKEND_URL}/api/v1/health"
echo "  curl ${BACKEND_URL}/api/v1/rl/exploration-rate"
echo ""
echo "To view logs:"
echo "  gcloud run logs read $BACKEND_SERVICE --region $REGION"
echo "  gcloud run logs read $FRONTEND_SERVICE --region $REGION"
