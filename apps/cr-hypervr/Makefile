PY ?= python3

.PHONY: gcp-provision gcp-secrets gcp-verify gcp-build gcp-deploy gcp-deploy-infra gcp-jobs-deploy gcp-job-run-phase2 gcp-job-run-phase3 db-apply-cloudsql export-openapi

# GCP: Core provisioning (Artifact Registry, bucket, Cloud SQL, SAs, secret)
gcp-provision:
	bash scripts/provision_core.sh

# GCP: Create/update Secret Manager entries (e.g., DATABASE_URL, Kaggle creds)
gcp-secrets:
	bash scripts/setup_secrets.sh

# GCP: Sanity checks (services, AR repo, Cloud SQL, SAs)
gcp-verify:
	bash scripts/gcp_verify.sh

# GCP: Build container in Cloud Build and push to Artifact Registry
gcp-build:
	gcloud builds submit --region=$${REGION:-europe-west2} --config=cloudbuild.yaml --substitutions=_REGION=$${REGION:-europe-west2}

# GCP: Deploy primary API service (embedding-service)
gcp-deploy:
	bash scripts/deploy_cloud_run.sh

# GCP: Deploy infra-service (graph‑focused, POST‑only)
gcp-deploy-infra:
	bash scripts/deploy_graph_service.sh

# GCP: Deploy Cloud Run Jobs for data pipelines and validation
gcp-jobs-deploy:
	bash scripts/deploy_jobs.sh

# Run Phase 2 (join → profiles → hyperedges + validation) as a Cloud Run Job
gcp-job-run-phase2:
	PROJECT_ID=$${PROJECT_ID} REGION=$${REGION:-europe-west2} bash -lc 'gcloud beta run jobs execute pipeline-phase2 --region $$REGION'

# Run Phase 3 (fine‑tune → ONNX → INT8) as a Cloud Run Job
gcp-job-run-phase3:
	PROJECT_ID=$${PROJECT_ID} REGION=$${REGION:-europe-west2} bash -lc 'gcloud beta run jobs execute pipeline-phase3 --region $$REGION'

# Apply schema + pgvector to Cloud SQL (uses Cloud SQL connect)
db-apply-cloudsql:
	bash scripts/db_apply_cloudsql.sh

# Export OpenAPI JSON (writes openapi.json in repo root)
export-openapi:
	$(PY) scripts/export_openapi.py
