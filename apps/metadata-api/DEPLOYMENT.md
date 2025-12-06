# Nexus-UMMID Metadata API - Deployment Guide

## Overview

Production deployment configuration for the Nexus-UMMID Entertainment Discovery Metadata API on Google Cloud Run.

**Target Infrastructure:**
- Platform: Google Cloud Run (Managed)
- GCP Project: `agentics-foundation25lon-1899`
- Region: `us-central1`
- Scaling: 1-100 instances
- Resources: 512Mi RAM, 1 CPU

## Files Created

### 1. Dockerfile
Multi-stage Docker build optimized for Cloud Run:
- **Stage 1 (Builder):** Compiles TypeScript, installs dependencies
- **Stage 2 (Production):** Minimal runtime image with Node.js 20 Alpine
- **Size Target:** <200MB
- **Security:** Non-root user (nodejs:1001)
- **Health Check:** Built-in HTTP health endpoint

### 2. .dockerignore
Optimizes Docker build context by excluding:
- node_modules (installed during build)
- Test files and coverage reports
- Development configuration
- Documentation and source control files

### 3. cloudbuild.yaml
Google Cloud Build pipeline with 4 stages:
1. **Build:** Creates optimized Docker image with caching
2. **Push:** Uploads to Artifact Registry (SHA + latest tags)
3. **Deploy:** Deploys to Cloud Run with production settings
4. **Verify:** Runs health check to confirm deployment

### 4. .env.production
Production environment template with:
- GCP project configuration
- Vertex AI (Gemini) settings
- Firebase/Firestore connection
- AgentDB learning system settings
- Platform connector configurations
- Feature flags and performance tuning

### 5. package.json (Updated)
Added deployment scripts:
- `docker:build` - Build Docker image locally
- `docker:run` - Run container locally with .env
- `docker:run:prod` - Run with production config
- `docker:stop` - Stop running container
- `docker:clean` - Remove image
- `gcp:build` - Trigger Cloud Build
- `gcp:deploy` - Deploy to Cloud Run

## Local Development & Testing

### Build and Test Docker Image Locally

```bash
cd apps/metadata-api

# Build the Docker image
npm run docker:build

# Run locally (development)
npm run docker:run

# Run with production config
npm run docker:run:prod

# Test health endpoint
curl http://localhost:8080/health

# Stop container
npm run docker:stop
```

### Manual Docker Commands

```bash
# Build with custom tag
docker build -t metadata-api:test .

# Run with custom environment
docker run -p 8080:8080 \
  -e NODE_ENV=production \
  -e GCP_PROJECT=agentics-foundation25lon-1899 \
  metadata-api:test

# Inspect image size
docker images metadata-api

# Check running containers
docker ps

# View logs
docker logs <container-id>
```

## Cloud Deployment

### Prerequisites

1. **GCP Authentication:**
```bash
gcloud auth login
gcloud config set project agentics-foundation25lon-1899
```

2. **Enable Required APIs:**
```bash
gcloud services enable cloudbuild.googleapis.com
gcloud services enable run.googleapis.com
gcloud services enable artifactregistry.googleapis.com
gcloud services enable aiplatform.googleapis.com
gcloud services enable firestore.googleapis.com
```

3. **Create Artifact Registry Repository:**
```bash
gcloud artifacts repositories create metadata-api \
  --repository-format=docker \
  --location=us-central1 \
  --description="Nexus-UMMID Metadata API Docker images"
```

4. **Create Service Account:**
```bash
gcloud iam service-accounts create metadata-api-sa \
  --display-name="Metadata API Service Account"

# Grant necessary permissions
gcloud projects add-iam-policy-binding agentics-foundation25lon-1899 \
  --member="serviceAccount:metadata-api-sa@agentics-foundation25lon-1899.iam.gserviceaccount.com" \
  --role="roles/aiplatform.user"

gcloud projects add-iam-policy-binding agentics-foundation25lon-1899 \
  --member="serviceAccount:metadata-api-sa@agentics-foundation25lon-1899.iam.gserviceaccount.com" \
  --role="roles/datastore.user"
```

### Deploy to Cloud Run

#### Option 1: Using Cloud Build (Recommended)

```bash
cd apps/metadata-api

# Trigger automated build and deploy pipeline
npm run gcp:build

# Or manually:
gcloud builds submit --config=cloudbuild.yaml
```

This will:
- Build Docker image with caching
- Push to Artifact Registry
- Deploy to Cloud Run
- Run health check verification

#### Option 2: Direct Cloud Run Deployment

```bash
cd apps/metadata-api

# Deploy from source (Cloud Run builds automatically)
npm run gcp:deploy

# Or manually:
gcloud run deploy nexus-ummid-metadata-api \
  --source . \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --min-instances 1 \
  --max-instances 100 \
  --memory 512Mi \
  --cpu 1 \
  --timeout 300s \
  --port 8080 \
  --service-account metadata-api-sa@agentics-foundation25lon-1899.iam.gserviceaccount.com
```

#### Option 3: Deploy Pre-built Image

```bash
# Build and push manually
docker build -t us-central1-docker.pkg.dev/agentics-foundation25lon-1899/metadata-api/nexus-ummid-metadata-api:v1.0.0 .
docker push us-central1-docker.pkg.dev/agentics-foundation25lon-1899/metadata-api/nexus-ummid-metadata-api:v1.0.0

# Deploy the image
gcloud run deploy nexus-ummid-metadata-api \
  --image us-central1-docker.pkg.dev/agentics-foundation25lon-1899/metadata-api/nexus-ummid-metadata-api:v1.0.0 \
  --region us-central1
```

### Configure Environment Variables

Set production environment variables in Cloud Run:

```bash
gcloud run services update nexus-ummid-metadata-api \
  --region us-central1 \
  --set-env-vars="NODE_ENV=production,GCP_PROJECT=agentics-foundation25lon-1899,VERTEX_AI_MODEL=gemini-1.5-flash-002"
```

For sensitive values, use Secret Manager:

```bash
# Create secret
echo -n "your-api-key" | gcloud secrets create api-key --data-file=-

# Grant access to service account
gcloud secrets add-iam-policy-binding api-key \
  --member="serviceAccount:metadata-api-sa@agentics-foundation25lon-1899.iam.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"

# Mount secret as environment variable
gcloud run services update nexus-ummid-metadata-api \
  --region us-central1 \
  --update-secrets="API_KEY=api-key:latest"
```

## Monitoring & Verification

### Check Deployment Status

```bash
# Get service details
gcloud run services describe nexus-ummid-metadata-api \
  --region us-central1 \
  --format json

# Get service URL
gcloud run services describe nexus-ummid-metadata-api \
  --region us-central1 \
  --format='value(status.url)'
```

### Test Deployed Service

```bash
# Get service URL
SERVICE_URL=$(gcloud run services describe nexus-ummid-metadata-api \
  --region us-central1 \
  --format='value(status.url)')

# Health check
curl $SERVICE_URL/health

# API info
curl $SERVICE_URL/

# Test metadata endpoint
curl $SERVICE_URL/api/v1/metadata
```

### View Logs

```bash
# Real-time logs
gcloud run services logs tail nexus-ummid-metadata-api \
  --region us-central1

# Filter by severity
gcloud run services logs read nexus-ummid-metadata-api \
  --region us-central1 \
  --filter='severity>=ERROR' \
  --limit 50
```

### Monitor Metrics

```bash
# Open Cloud Console Metrics
gcloud run services describe nexus-ummid-metadata-api \
  --region us-central1 \
  --format='get(status.url)' | \
  sed 's|https://||' | \
  xargs -I {} echo "https://console.cloud.google.com/run/detail/us-central1/nexus-ummid-metadata-api/metrics?project=agentics-foundation25lon-1899"
```

## Performance Optimization

### Current Configuration

- **Memory:** 512Mi (sufficient for 400M+ users with caching)
- **CPU:** 1 vCPU (auto-scales based on load)
- **Min Instances:** 1 (reduces cold starts)
- **Max Instances:** 100 (handles traffic spikes)
- **Concurrency:** 80 requests per instance
- **Timeout:** 300 seconds (5 minutes)

### Scaling Adjustments

For higher load:

```bash
gcloud run services update nexus-ummid-metadata-api \
  --region us-central1 \
  --memory 1Gi \
  --cpu 2 \
  --min-instances 3 \
  --max-instances 200 \
  --concurrency 100
```

For cost optimization:

```bash
gcloud run services update nexus-ummid-metadata-api \
  --region us-central1 \
  --min-instances 0 \
  --max-instances 50 \
  --concurrency 80
```

## Security Best Practices

1. **Non-root Container:** Runs as `nodejs` user (UID 1001)
2. **Service Account:** Uses dedicated IAM service account with minimal permissions
3. **Secrets Management:** Sensitive values stored in Secret Manager
4. **HTTPS Only:** Cloud Run enforces HTTPS automatically
5. **Security Headers:** Helmet.js middleware enabled
6. **Authentication:** Supports API key and Firebase Auth

## Troubleshooting

### Build Fails

```bash
# Check build logs
gcloud builds log --stream

# Validate Dockerfile syntax
docker build --no-cache -t test .
```

### Deployment Fails

```bash
# Check service logs
gcloud run services logs tail nexus-ummid-metadata-api --region us-central1

# Verify service account permissions
gcloud projects get-iam-policy agentics-foundation25lon-1899 \
  --flatten="bindings[].members" \
  --filter="bindings.members:metadata-api-sa@agentics-foundation25lon-1899.iam.gserviceaccount.com"
```

### Health Check Fails

```bash
# Test locally first
docker run -p 8080:8080 metadata-api
curl http://localhost:8080/health

# Check Cloud Run startup logs
gcloud run services logs read nexus-ummid-metadata-api \
  --region us-central1 \
  --limit 100
```

### High Latency

```bash
# Check instance scaling
gcloud run services describe nexus-ummid-metadata-api \
  --region us-central1 \
  --format='value(spec.template.metadata.annotations)'

# Increase min instances to reduce cold starts
gcloud run services update nexus-ummid-metadata-api \
  --region us-central1 \
  --min-instances 3
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Deploy to Cloud Run

on:
  push:
    branches: [main]
    paths:
      - 'apps/metadata-api/**'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: google-github-actions/auth@v1
        with:
          credentials_json: ${{ secrets.GCP_SA_KEY }}

      - uses: google-github-actions/setup-gcloud@v1

      - name: Build and Deploy
        run: |
          cd apps/metadata-api
          gcloud builds submit --config=cloudbuild.yaml
```

## Cost Estimation

**Cloud Run Pricing (us-central1):**
- CPU: $0.00002400 per vCPU-second
- Memory: $0.00000250 per GiB-second
- Requests: $0.40 per million requests

**Example monthly cost (1M requests, avg 100ms response):**
- CPU: ~$4
- Memory: ~$1
- Requests: ~$0.40
- **Total: ~$5-6/month**

## Support & Documentation

- **Cloud Run Docs:** https://cloud.google.com/run/docs
- **Artifact Registry:** https://cloud.google.com/artifact-registry/docs
- **Cloud Build:** https://cloud.google.com/build/docs
- **Vertex AI:** https://cloud.google.com/vertex-ai/docs

---

**Last Updated:** 2025-12-06
**Version:** 1.0.0
**Maintainer:** mondweep
