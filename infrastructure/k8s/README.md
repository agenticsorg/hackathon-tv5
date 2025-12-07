# Kubernetes Deployment Configuration

This directory contains Kubernetes manifests for deploying the Media Gateway platform on Google Kubernetes Engine (GKE).

## Directory Structure

```
infrastructure/k8s/
├── namespace.yaml              # Namespace definitions (prod/staging)
├── services/                   # Service deployments
│   ├── api-gateway.yaml
│   ├── discovery-service.yaml
│   ├── sona-engine.yaml
│   ├── sync-service.yaml
│   ├── auth-service.yaml
│   ├── mcp-server.yaml
│   └── ingestion-service.yaml
├── ingress.yaml               # Ingress with Cloud Armor
├── configmaps/                # Configuration
│   └── app-config.yaml
├── secrets/                   # External secrets
│   └── external-secrets.yaml
├── network-policies/          # Zero-trust networking
│   └── default.yaml
├── kustomization.yaml         # Kustomize configuration
└── README.md                  # This file
```

## Prerequisites

1. GKE cluster with Workload Identity enabled
2. External Secrets Operator installed
3. GCP Secret Manager configured
4. Cloud Armor security policy created
5. Static IP address reserved

## Deployment

### Using kubectl

```bash
# Deploy to production
kubectl apply -f namespace.yaml
kubectl apply -f configmaps/ -n media-gateway-prod
kubectl apply -f secrets/ -n media-gateway-prod
kubectl apply -f services/ -n media-gateway-prod
kubectl apply -f network-policies/ -n media-gateway-prod
kubectl apply -f ingress.yaml

# Deploy to staging
kubectl apply -f namespace.yaml
kubectl apply -f configmaps/ -n media-gateway-staging
kubectl apply -f secrets/ -n media-gateway-staging
kubectl apply -f services/ -n media-gateway-staging
kubectl apply -f network-policies/ -n media-gateway-staging
```

### Using Kustomize

```bash
# Production
kustomize build . | kubectl apply -f -

# Staging
kustomize build . | kubectl apply -f - --namespace media-gateway-staging
```

## Service Configuration

### API Gateway
- **Port**: 8080
- **Replicas**: 3-10 (HPA)
- **Resources**: 500m-1000m CPU, 512Mi-1Gi memory
- **Health**: /health endpoint

### Discovery Service
- **Port**: 8081
- **Replicas**: 2-8 (HPA)
- **Resources**: 250m-500m CPU, 256Mi-512Mi memory

### SONA Engine
- **Port**: 8082
- **Replicas**: 2-6 (HPA)
- **Resources**: 500m-1000m CPU, 1Gi-2Gi memory

### Sync Service
- **Port**: 8083 (HTTP), 8084 (WebSocket)
- **Replicas**: 2-6 (HPA)
- **Resources**: 250m-500m CPU, 256Mi-512Mi memory

### Auth Service
- **Port**: 8084
- **Replicas**: 2-4 (HPA)
- **Resources**: 250m-500m CPU, 256Mi-512Mi memory

### MCP Server
- **Port**: 3000 (HTTP), 3001 (SSE)
- **Replicas**: 1-4 (HPA)
- **Resources**: 250m-500m CPU, 256Mi-512Mi memory

### Ingestion Service
- **Port**: 8085
- **Replicas**: 1-3 (HPA)
- **Resources**: 250m-500m CPU, 256Mi-512Mi memory

## Network Policies

Zero-trust networking is enforced with NetworkPolicies:

- Default deny all ingress/egress
- Explicit allow rules for service communication
- DNS access allowed to kube-system
- Database/Redis access allowed where needed
- Monitoring allowed from monitoring namespace

## Security

### Cloud Armor
- SQL injection protection
- XSS protection
- Local file inclusion protection
- Remote code execution protection
- Rate limiting (1000 req/min per IP)

### Workload Identity
Each service uses a dedicated GCP service account with minimal permissions.

### External Secrets
Secrets are stored in GCP Secret Manager and synchronized to Kubernetes using External Secrets Operator.

### Network Policies
Zero-trust networking with explicit allow rules.

## Monitoring

All services expose Prometheus metrics on port 9090+:
- api-gateway: 9090
- discovery-service: 9091
- sona-engine: 9092
- sync-service: 9093
- auth-service: 9094
- mcp-server: 9095
- ingestion-service: 9096

## Health Checks

All services implement:
- Liveness probe: /health endpoint
- Readiness probe: /health endpoint
- Startup probe: /health endpoint (where applicable)

## Autoscaling

HPA is configured for all services with:
- CPU-based scaling
- Memory-based scaling
- Custom metrics (e.g., WebSocket connections for sync-service)

## Rolling Updates

All deployments use rolling update strategy:
- maxSurge: 1
- maxUnavailable: 0

This ensures zero-downtime deployments.

## Troubleshooting

### Check pod status
```bash
kubectl get pods -n media-gateway-prod
```

### View logs
```bash
kubectl logs -f deployment/api-gateway -n media-gateway-prod
```

### Check HPA status
```bash
kubectl get hpa -n media-gateway-prod
```

### Debug network policies
```bash
kubectl describe networkpolicy api-gateway-policy -n media-gateway-prod
```

### Test service connectivity
```bash
kubectl run test --image=curlimages/curl:latest --rm -i --restart=Never -n media-gateway-prod -- \
  curl -f http://api-gateway.media-gateway-prod.svc.cluster.local:8080/health
```

## Backup and Recovery

### Backup current state
```bash
kubectl get all -n media-gateway-prod -o yaml > backup.yaml
```

### Restore from backup
```bash
kubectl apply -f backup.yaml
```

## CI/CD Integration

GitHub Actions workflow automatically:
1. Builds Docker images
2. Pushes to GCR
3. Updates Kubernetes manifests
4. Deploys to staging/production
5. Runs smoke tests
6. Performs canary deployments

See `.github/workflows/ci-cd.yaml` for details.
