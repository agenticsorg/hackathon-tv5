#!/bin/bash
# Nexus-UMMID Demo UI Deployment Script
# Run this from Google Cloud Shell or a machine with gcloud configured

set -e

# Configuration
PROJECT_ID="agentics-foundation25lon-1899"
REGION="us-central1"
SERVICE_NAME="nexus-ummid-demo"
IMAGE_NAME="demo-ui"
ARTIFACT_REGISTRY="${REGION}-docker.pkg.dev"

echo "🚀 Deploying Nexus-UMMID Demo UI to Cloud Run..."
echo "   Project: ${PROJECT_ID}"
echo "   Region: ${REGION}"
echo "   Service: ${SERVICE_NAME}"
echo ""

# Set project
gcloud config set project ${PROJECT_ID}

# Create Artifact Registry repository if it doesn't exist
echo "📦 Ensuring Artifact Registry repository exists..."
gcloud artifacts repositories describe ${IMAGE_NAME} \
    --location=${REGION} 2>/dev/null || \
gcloud artifacts repositories create ${IMAGE_NAME} \
    --repository-format=docker \
    --location=${REGION} \
    --description="Nexus-UMMID Demo UI container images"

# Configure Docker for Artifact Registry
echo "🔐 Configuring Docker authentication..."
gcloud auth configure-docker ${ARTIFACT_REGISTRY} --quiet

# Build image
echo "🔨 Building Docker image..."
IMAGE_TAG="${ARTIFACT_REGISTRY}/${PROJECT_ID}/${IMAGE_NAME}/${SERVICE_NAME}:latest"
docker build -t ${IMAGE_TAG} .

# Push image
echo "📤 Pushing image to Artifact Registry..."
docker push ${IMAGE_TAG}

# Deploy to Cloud Run
echo "☁️ Deploying to Cloud Run..."
gcloud run deploy ${SERVICE_NAME} \
    --image ${IMAGE_TAG} \
    --region ${REGION} \
    --platform managed \
    --allow-unauthenticated \
    --memory 256Mi \
    --cpu 1 \
    --min-instances 0 \
    --max-instances 10 \
    --port 8080

# Get service URL
echo ""
echo "✅ Deployment complete!"
echo ""
SERVICE_URL=$(gcloud run services describe ${SERVICE_NAME} --region ${REGION} --format 'value(status.url)')
echo "🌐 Demo UI URL: ${SERVICE_URL}"
echo ""
echo "Share this URL with your team!"
