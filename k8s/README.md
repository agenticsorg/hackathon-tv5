# Media Gateway Kubernetes Manifests

This directory contains Kubernetes manifests for deploying Media Gateway microservices to GKE Autopilot.

## Directory Structure

```
k8s/
├── base/                           # Base manifests (environment-agnostic)
│   ├── kustomization.yaml         # Base kustomization config
│   ├── namespace.yaml             # Namespace definition
│   ├── common/                    # Common resources
│   │   ├── configmap.yaml         # Shared configuration
│   │   ├── secrets.yaml           # Secret templates
│   │   └── network-policy.yaml    # Network policies
│   ├── api-gateway/               # API Gateway (port 8080)
│   ├── auth-service/              # Auth Service (port 8084)
│   ├── discovery-service/         # Discovery Service (port 8081)
│   ├── sona-service/              # SONA Engine (port 8082)
│   ├── sync-service/              # Sync Service (port 8083)
│   ├── ingestion-service/         # Ingestion Service (port 8085)
│   ├── playback-service/          # Playback Service (port 8086)
│   └── mcp-server/                # MCP Server (port 3000)
└── overlays/                       # Environment-specific overlays
    ├── dev/                       # Development environment
    ├── staging/                   # Staging environment
    └── prod/                      # Production environment
```

## Service Ports

| Service | Port | Component |
|---------|------|-----------|
| API Gateway | 8080 | Gateway |
| Auth Service | 8084 | Authentication |
| Discovery Service | 8081 | Catalog |
| SONA Service | 8082 | Recommendation |
| Sync Service | 8083 | Sync |
| Ingestion Service | 8085 | Ingestion |
| Playback Service | 8086 | Playback |
| MCP Server | 3000 | Orchestration |

All services expose metrics on port 9090.

## Prerequisites

1. GKE Autopilot cluster
2. `kubectl` configured
3. `kustomize` installed (v4.0+)
4. Docker images pushed to GCR

## Deployment

### 1. Update Configuration

Before deploying, update the following:

**Secrets** (`k8s/base/common/secrets.yaml`):
```bash
# Replace all REPLACE_ME values with actual secrets
# OR use Google Secret Manager with External Secrets Operator
```

**Image Registry** (in overlay kustomization files):
```yaml
# Replace PROJECT_ID with your GCP project ID
images:
  - name: gcr.io/PROJECT_ID/api-gateway
    newTag: v1.0.0
```

### 2. Deploy to Development

```bash
# Preview changes
kubectl kustomize k8s/overlays/dev

# Apply
kubectl apply -k k8s/overlays/dev

# Verify
kubectl get all -n media-gateway
```

### 3. Deploy to Staging

```bash
kubectl apply -k k8s/overlays/staging
```

### 4. Deploy to Production

```bash
kubectl apply -k k8s/overlays/prod
```

## Resource Specifications

### Development Environment
- 1 replica per service
- Reduced resource limits (200m CPU, 256Mi memory)
- Debug logging enabled
- Tracing disabled

### Staging Environment
- 2 replicas per service
- Moderate HPA limits (5-8 max replicas)
- Info-level logging
- 50% trace sampling

### Production Environment
- 3 replicas for critical services (API Gateway, Discovery, Playback)
- 2 replicas for other services
- Full HPA limits (8-15 max replicas)
- Warn-level logging
- 10% trace sampling
- Increased PodDisruptionBudgets

## Autoscaling

All services include HorizontalPodAutoscaler (HPA) with:
- CPU target: 70-75%
- Memory target: 80-85%
- Scale-down stabilization: 5 minutes
- Scale-up stabilization: 0 seconds

Example HPA ranges:
- API Gateway: 2-10 replicas
- Discovery Service: 2-12 replicas
- Playback Service: 2-15 replicas
- Auth Service: 2-8 replicas

## Security Features

### Pod Security
- `runAsNonRoot: true`
- `readOnlyRootFilesystem: true`
- `seccompProfile: RuntimeDefault`
- Drop all capabilities
- Non-privileged containers (UID 1000)

### Network Policies
- Internal traffic allowed within namespace
- External traffic only to API Gateway
- MCP Server isolated (internal only)
- DNS, PostgreSQL, Redis access allowed

### Secret Management
Base manifests include placeholder secrets. For production:

**Option 1: Google Secret Manager**
```bash
# Install External Secrets Operator
helm repo add external-secrets https://charts.external-secrets.io
helm install external-secrets external-secrets/external-secrets -n external-secrets-system
```

**Option 2: Kubernetes Secrets**
```bash
# Create secrets manually
kubectl create secret generic media-gateway-secrets \
  --from-literal=DATABASE_URL=postgresql://... \
  --from-literal=JWT_SECRET=... \
  -n media-gateway
```

## Monitoring

### Prometheus Integration
All services expose metrics on port 9090 with annotations:
```yaml
prometheus.io/scrape: "true"
prometheus.io/port: "9090"
prometheus.io/path: "/metrics"
```

### Health Checks
- Liveness probe: `/health` endpoint
- Readiness probe: `/ready` endpoint
- Initial delay: 30s (liveness), 10s (readiness)

## High Availability

### Pod Disruption Budgets
- Minimum 1 available pod per service
- Production: 2 available for critical services

### Anti-Affinity
Pods prefer different nodes using soft anti-affinity:
```yaml
podAntiAffinity:
  preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      topologyKey: kubernetes.io/hostname
```

## Volume Mounts

All services mount:
- `/tmp` - emptyDir for temporary files
- `/app/cache` - emptyDir for caching

Additional mounts:
- SONA Service: `/app/models` (1Gi)
- Discovery Service: `/app/cache` (2Gi)
- Ingestion Service: `/app/staging` (5Gi)

## Updating Images

### Using Kustomize
```bash
cd k8s/overlays/prod
kustomize edit set image gcr.io/PROJECT_ID/api-gateway:v1.1.0
kubectl apply -k .
```

### Using kubectl
```bash
kubectl set image deployment/api-gateway \
  api-gateway=gcr.io/PROJECT_ID/api-gateway:v1.1.0 \
  -n media-gateway
```

## Rollback

```bash
# View rollout history
kubectl rollout history deployment/api-gateway -n media-gateway

# Rollback to previous version
kubectl rollout undo deployment/api-gateway -n media-gateway

# Rollback to specific revision
kubectl rollout undo deployment/api-gateway --to-revision=2 -n media-gateway
```

## Troubleshooting

### View Logs
```bash
# Single pod
kubectl logs -f deployment/api-gateway -n media-gateway

# All pods
kubectl logs -f -l app=api-gateway -n media-gateway

# Previous container
kubectl logs deployment/api-gateway --previous -n media-gateway
```

### Describe Resources
```bash
kubectl describe deployment api-gateway -n media-gateway
kubectl describe pod -l app=api-gateway -n media-gateway
kubectl describe hpa api-gateway -n media-gateway
```

### Debug Networking
```bash
# Check service endpoints
kubectl get endpoints -n media-gateway

# Test connectivity
kubectl run -it --rm debug --image=busybox --restart=Never -n media-gateway -- sh
wget -O- http://api-gateway:8080/health
```

### Check Resource Usage
```bash
kubectl top pods -n media-gateway
kubectl top nodes
```

## GKE Autopilot Considerations

1. **Resource Requests Required**: Autopilot requires resource requests for all containers
2. **Ephemeral Storage**: Specify ephemeral-storage requests/limits
3. **No Node Pools**: Autopilot manages nodes automatically
4. **Workload Identity**: Use for GCP service access
5. **Automatic Updates**: Nodes auto-update on schedule

## CI/CD Integration

### GitHub Actions Example
```yaml
- name: Deploy to GKE
  run: |
    gcloud container clusters get-credentials CLUSTER_NAME --region REGION
    kubectl apply -k k8s/overlays/prod
    kubectl rollout status deployment/api-gateway -n media-gateway
```

### Using Skaffold
```yaml
apiVersion: skaffold/v2beta29
kind: Config
deploy:
  kustomize:
    paths:
      - k8s/overlays/dev
```

## Best Practices

1. **Never commit secrets**: Use Secret Manager or encrypted secrets
2. **Use resource limits**: Prevent resource exhaustion
3. **Enable monitoring**: Prometheus, Grafana, Cloud Monitoring
4. **Test in staging**: Always test changes before production
5. **Use namespaces**: Isolate environments
6. **Tag images properly**: Use semantic versioning
7. **Document changes**: Update this README
8. **Review security**: Regular security audits

## Additional Resources

- [GKE Autopilot Documentation](https://cloud.google.com/kubernetes-engine/docs/concepts/autopilot-overview)
- [Kustomize Documentation](https://kustomize.io/)
- [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/configuration/overview/)
- [Pod Security Standards](https://kubernetes.io/docs/concepts/security/pod-security-standards/)

## Support

For issues or questions:
1. Check logs: `kubectl logs -f deployment/SERVICE_NAME -n media-gateway`
2. Check events: `kubectl get events -n media-gateway`
3. Review this documentation
4. Contact the platform team
