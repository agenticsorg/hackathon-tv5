#!/bin/bash
# Deployment script for AI Media Discovery Platform
# Usage: ./deploy.sh [command]

set -e

# Configuration
PROJECT_ID="${GCP_PROJECT_ID:-your-project-id}"
REGION="${GCP_REGION:-us-central1}"
SERVICE_NAME="media-discovery"
IMAGE_NAME="gcr.io/${PROJECT_ID}/${SERVICE_NAME}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Commands
case "${1:-help}" in
  setup)
    log_info "Setting up GCP project..."

    # Enable required APIs
    gcloud services enable \
      run.googleapis.com \
      cloudbuild.googleapis.com \
      containerregistry.googleapis.com \
      secretmanager.googleapis.com \
      firestore.googleapis.com \
      --project="${PROJECT_ID}"

    log_info "Creating secrets (you'll need to add values)..."

    # Create secrets if they don't exist
    gcloud secrets describe tmdb-access-token --project="${PROJECT_ID}" 2>/dev/null || \
      gcloud secrets create tmdb-access-token --project="${PROJECT_ID}"

    gcloud secrets describe openai-api-key --project="${PROJECT_ID}" 2>/dev/null || \
      gcloud secrets create openai-api-key --project="${PROJECT_ID}"

    log_info "Setup complete! Add secret values with:"
    echo "  gcloud secrets versions add tmdb-access-token --data-file=- <<< 'YOUR_TOKEN'"
    echo "  gcloud secrets versions add openai-api-key --data-file=- <<< 'YOUR_KEY'"
    ;;

  build)
    TAG="${2:-latest}"
    log_info "Building Docker image: ${IMAGE_NAME}:${TAG}"

    docker build -t "${IMAGE_NAME}:${TAG}" .
    ;;

  build-cloud)
    TAG="${2:-latest}"
    log_info "Building with Cloud Build: ${IMAGE_NAME}:${TAG}"

    gcloud builds submit --tag "${IMAGE_NAME}:${TAG}" --project="${PROJECT_ID}"
    ;;

  push)
    TAG="${2:-latest}"
    log_info "Pushing image to Container Registry..."

    docker push "${IMAGE_NAME}:${TAG}"
    ;;

  deploy)
    TAG="${2:-latest}"
    log_info "Deploying to Cloud Run..."

    gcloud run deploy "${SERVICE_NAME}" \
      --image "${IMAGE_NAME}:${TAG}" \
      --region "${REGION}" \
      --platform managed \
      --allow-unauthenticated \
      --memory 2Gi \
      --cpu 2 \
      --min-instances 0 \
      --max-instances 20 \
      --set-env-vars "NODE_ENV=production" \
      --set-secrets "NEXT_PUBLIC_TMDB_ACCESS_TOKEN=tmdb-access-token:latest,OPENAI_API_KEY=openai-api-key:latest" \
      --project="${PROJECT_ID}"

    URL=$(gcloud run services describe "${SERVICE_NAME}" --region="${REGION}" --format='value(status.url)' --project="${PROJECT_ID}")
    log_info "Deployed to: ${URL}"
    ;;

  deploy-ruvector)
    log_info "Deploying RuVector service..."

    # Deploy RuVector from the repos directory
    pushd ../../repos/ruvector-main/examples/google-cloud
    ./deploy.sh build Dockerfile.simple latest
    ./deploy.sh push latest
    ./deploy.sh deploy latest false
    popd

    RUVECTOR_URL=$(gcloud run services describe ruvector-benchmark --region="${REGION}" --format='value(status.url)' --project="${PROJECT_ID}")
    log_info "RuVector deployed to: ${RUVECTOR_URL}"
    log_info "Update RUVECTOR_ENDPOINT in your deployment."
    ;;

  logs)
    log_info "Fetching logs..."
    gcloud run services logs read "${SERVICE_NAME}" --region="${REGION}" --project="${PROJECT_ID}" --limit="${2:-100}"
    ;;

  status)
    log_info "Service status:"
    gcloud run services describe "${SERVICE_NAME}" --region="${REGION}" --project="${PROJECT_ID}" --format='yaml(status)'
    ;;

  url)
    URL=$(gcloud run services describe "${SERVICE_NAME}" --region="${REGION}" --format='value(status.url)' --project="${PROJECT_ID}")
    echo "${URL}"
    ;;

  cleanup)
    log_warn "This will delete the Cloud Run service. Continue? (y/N)"
    read -r confirm
    if [[ "${confirm}" == "y" ]]; then
      gcloud run services delete "${SERVICE_NAME}" --region="${REGION}" --project="${PROJECT_ID}" --quiet
      log_info "Service deleted."
    fi
    ;;

  help|*)
    echo "AI Media Discovery Platform - Deployment Script"
    echo ""
    echo "Usage: ./deploy.sh [command] [options]"
    echo ""
    echo "Commands:"
    echo "  setup           - Enable GCP APIs and create secrets"
    echo "  build [tag]     - Build Docker image locally"
    echo "  build-cloud [tag] - Build with Cloud Build"
    echo "  push [tag]      - Push image to Container Registry"
    echo "  deploy [tag]    - Deploy to Cloud Run"
    echo "  deploy-ruvector - Deploy RuVector service"
    echo "  logs [count]    - View service logs"
    echo "  status          - Show service status"
    echo "  url             - Get service URL"
    echo "  cleanup         - Delete the service"
    echo ""
    echo "Environment variables:"
    echo "  GCP_PROJECT_ID  - Google Cloud project ID"
    echo "  GCP_REGION      - Deployment region (default: us-central1)"
    ;;
esac
