# Exogenesis Omega - Deployment Guide

This directory contains Docker and Kubernetes deployment configurations for the Exogenesis Omega distributed TV recommendation system.

## 📁 Directory Structure

```
deploy/
├── docker/                          # Docker configurations
│   ├── Dockerfile.constellation     # Constellation server (<100MB)
│   ├── Dockerfile.federation        # Federation worker
│   ├── docker-compose.yml          # Local development
│   └── docker-compose.prod.yml     # Production cluster
│
├── kubernetes/                      # Kubernetes manifests
│   ├── namespace.yaml              # omega-system namespace
│   ├── constellation-deployment.yaml
│   ├── constellation-service.yaml
│   ├── ruvector-statefulset.yaml
│   ├── federation-cronjob.yaml
│   ├── configmap.yaml
│   ├── secret.yaml                 # Template (DO NOT commit actual secrets)
│   ├── hpa.yaml                    # HorizontalPodAutoscaler
│   └── pdb.yaml                    # PodDisruptionBudget
│
└── helm/                            # Helm charts
    └── omega-constellation/
        ├── Chart.yaml
        ├── values.yaml             # Default values
        ├── values-prod.yaml        # Production overrides
        └── templates/
            ├── _helpers.tpl
            ├── deployment.yaml
            ├── service.yaml
            └── configmap.yaml
```

---

## 🐳 Docker Deployment

### Local Development

1. **Start the development cluster:**

```bash
cd deploy/docker

# Set environment variables
export POSTGRES_PASSWORD="your_strong_password"
export GRAFANA_PASSWORD="admin"

# Start services
docker-compose up -d

# View logs
docker-compose logs -f constellation-1

# Check health
curl http://localhost:8080/health
```

2. **Services available:**
   - Constellation 1: gRPC `localhost:50051`, HTTP `localhost:8080`, Metrics `localhost:9090`
   - Constellation 2: gRPC `localhost:50052`, HTTP `localhost:8081`, Metrics `localhost:9091`
   - Constellation 3: gRPC `localhost:50053`, HTTP `localhost:8082`, Metrics `localhost:9092`
   - RuVector-Postgres: `localhost:5432`
   - Prometheus: `localhost:9093`
   - Grafana: `localhost:3000` (admin/admin)

3. **Stop services:**

```bash
docker-compose down

# Remove volumes (reset data)
docker-compose down -v
```

### Production Cluster (Docker Swarm)

1. **Initialize Swarm:**

```bash
docker swarm init
```

2. **Deploy production stack:**

```bash
cd deploy/docker

# Set production secrets
export POSTGRES_PASSWORD="$(openssl rand -base64 32)"
export VERSION="v0.1.0"
export SHARD_ID=1
export REGION="us-east-1"

# Deploy stack
docker stack deploy -c docker-compose.prod.yml omega-prod

# Check services
docker service ls

# Scale constellation servers
docker service scale omega-prod_constellation=10
```

3. **Monitor:**

```bash
# Service logs
docker service logs -f omega-prod_constellation

# Check Raft status
docker exec omega-ruvector-primary psql -U omega -c "SELECT * FROM raft_status();"
```

---

## ☸️ Kubernetes Deployment

### Prerequisites

- Kubernetes 1.25+
- kubectl configured
- Storage class `fast-ssd` for NVMe volumes
- (Optional) cert-manager for TLS
- (Optional) Prometheus Operator

### Quick Deployment

1. **Create namespace:**

```bash
kubectl apply -f kubernetes/namespace.yaml
```

2. **Create secrets:**

```bash
# Generate password
export POSTGRES_PASSWORD="$(openssl rand -base64 32)"

# Create secret from template
cat kubernetes/secret.yaml | \
  sed "s/<REPLACE_WITH_STRONG_PASSWORD>/$POSTGRES_PASSWORD/g" | \
  kubectl apply -f -
```

3. **Deploy components:**

```bash
# ConfigMaps
kubectl apply -f kubernetes/configmap.yaml

# RuVector-Postgres StatefulSet
kubectl apply -f kubernetes/ruvector-statefulset.yaml

# Wait for database
kubectl wait --for=condition=ready pod -l app=ruvector-postgres -n omega-system --timeout=300s

# Constellation servers
kubectl apply -f kubernetes/constellation-deployment.yaml
kubectl apply -f kubernetes/constellation-service.yaml

# Federation worker
kubectl apply -f kubernetes/federation-cronjob.yaml

# Autoscaling and availability
kubectl apply -f kubernetes/hpa.yaml
kubectl apply -f kubernetes/pdb.yaml
```

4. **Verify deployment:**

```bash
# Check pods
kubectl get pods -n omega-system

# Check services
kubectl get svc -n omega-system

# View logs
kubectl logs -f -l app=constellation -n omega-system

# Check database
kubectl exec -it ruvector-postgres-0 -n omega-system -- psql -U omega -c "SELECT version();"
```

5. **Get external IP:**

```bash
# gRPC endpoint (for TV devices)
kubectl get svc constellation-grpc-lb -n omega-system

# Management API
kubectl get svc constellation-api-lb -n omega-system
```

### Scaling

```bash
# Scale constellation servers
kubectl scale deployment constellation-server -n omega-system --replicas=15

# Add read replicas (if configured)
kubectl scale statefulset ruvector-postgres-read -n omega-system --replicas=5
```

### Monitoring

```bash
# Port-forward Prometheus
kubectl port-forward -n omega-system svc/prometheus 9090:9090

# Port-forward Grafana
kubectl port-forward -n omega-system svc/grafana 3000:3000
```

---

## 🎯 Helm Deployment (Recommended)

### Installation

1. **Install development environment:**

```bash
cd deploy/helm

helm install omega ./omega-constellation \
  --namespace omega-system \
  --create-namespace \
  --set ruvectorPostgres.auth.password="$(openssl rand -base64 32)" \
  --set monitoring.grafana.adminPassword="admin"
```

2. **Install production environment:**

```bash
helm install omega ./omega-constellation \
  --namespace omega-system \
  --create-namespace \
  -f omega-constellation/values-prod.yaml \
  --set ruvectorPostgres.auth.existingSecret=ruvector-postgres-secret \
  --set monitoring.grafana.existingSecret=grafana-admin-secret
```

### Upgrade

```bash
# Upgrade with new image
helm upgrade omega ./omega-constellation \
  --namespace omega-system \
  --set constellation.image.tag=v0.2.0

# Rollback
helm rollback omega 1 -n omega-system
```

### Uninstall

```bash
helm uninstall omega -n omega-system

# Delete PVCs (if needed)
kubectl delete pvc -l app=ruvector-postgres -n omega-system
```

### Customization

Create your own values file:

```yaml
# my-values.yaml
global:
  environment: staging
  region: us-west-2

constellation:
  replicaCount: 5
  shard:
    id: 25
    region: us-west-2

ruvectorPostgres:
  replicas: 3
  persistence:
    data:
      size: 1Ti
```

```bash
helm install omega ./omega-constellation \
  -f omega-constellation/values-prod.yaml \
  -f my-values.yaml
```

---

## 📊 Architecture Highlights

### Constellation Server
- **Replicas:** 10 per shard (5-20 with autoscaling)
- **Resources:** 8-16 CPU, 32-64Gi RAM
- **Ports:** 50051 (gRPC), 8080 (HTTP), 9090 (Metrics)
- **Image Size:** <100MB (multi-stage Rust build)

### RuVector-Postgres
- **Topology:** 3-node Raft cluster
- **Storage:** 2Ti data + 100Gi WAL per node
- **Resources:** 8-16 CPU, 32-64Gi RAM
- **Features:** HNSW indexing, GNN learning, adaptive compression

### Federation Worker
- **Schedule:** Hourly CronJob
- **Function:** Cross-shard pattern aggregation
- **Resources:** 4-8 CPU, 16-32Gi RAM

---

## 🔒 Security Considerations

### Secrets Management

**Never commit actual secrets!** Use one of these approaches:

1. **Kubernetes Secrets:**
```bash
kubectl create secret generic ruvector-postgres-secret \
  --from-literal=username=omega \
  --from-literal=password="$(openssl rand -base64 32)" \
  -n omega-system
```

2. **External Secrets Operator:**
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: ruvector-postgres-secret
spec:
  secretStoreRef:
    name: aws-secrets-manager
  target:
    name: ruvector-postgres-secret
  data:
    - secretKey: password
      remoteRef:
        key: /omega/production/postgres-password
```

3. **Sealed Secrets:**
```bash
kubeseal --format=yaml < secret.yaml > sealed-secret.yaml
kubectl apply -f sealed-secret.yaml
```

### TLS Configuration

1. **Using cert-manager:**
```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create ClusterIssuer
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
      - http01:
          ingress:
            class: nginx
EOF

# Enable in Helm
helm upgrade omega ./omega-constellation \
  --set tls.enabled=true \
  --set tls.certManager.enabled=true \
  --set tls.certManager.issuer=letsencrypt-prod
```

2. **Using existing certificates:**
```bash
kubectl create secret tls constellation-tls \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key \
  -n omega-system

helm upgrade omega ./omega-constellation \
  --set tls.enabled=true \
  --set tls.existingSecret=constellation-tls
```

---

## 📈 Monitoring & Observability

### Prometheus Metrics

Key metrics exposed:

- `constellation_devices_active` - Active device count
- `constellation_sync_requests_total` - Total sync requests
- `constellation_sync_latency_ms` - Sync latency histogram
- `constellation_pattern_quality` - Pattern quality distribution
- `constellation_grpc_connections_active` - Active gRPC connections
- `postgres_connections_active` - Database connections
- `ruvector_vector_search_latency_ms` - Vector search performance

### Grafana Dashboards

Pre-configured dashboards are included in Helm charts:

1. **Omega System Overview**
   - Active devices
   - Sync rate
   - Pattern quality
   - Resource utilization

2. **Constellation Metrics**
   - Request rate and latency
   - gRPC connection pool
   - Error rates
   - Cache hit rates

3. **Database Metrics**
   - Query performance
   - Connection pool usage
   - Raft consensus health
   - Storage utilization

Access Grafana:
```bash
kubectl port-forward -n omega-system svc/grafana 3000:3000
# Open http://localhost:3000
```

---

## 🔧 Troubleshooting

### Common Issues

1. **Pods stuck in Pending:**
```bash
# Check events
kubectl describe pod <pod-name> -n omega-system

# Check storage class
kubectl get storageclass

# Check resource availability
kubectl top nodes
```

2. **Database connection errors:**
```bash
# Check PostgreSQL readiness
kubectl exec -it ruvector-postgres-0 -n omega-system -- pg_isready -U omega

# Check Raft status
kubectl exec -it ruvector-postgres-0 -n omega-system -- \
  psql -U omega -c "SELECT * FROM raft_status();"

# View logs
kubectl logs -f ruvector-postgres-0 -n omega-system
```

3. **High memory usage:**
```bash
# Check resource usage
kubectl top pods -n omega-system

# Adjust limits in Helm
helm upgrade omega ./omega-constellation \
  --set constellation.resources.limits.memory=48Gi
```

4. **Sync latency issues:**
```bash
# Check metrics
kubectl port-forward -n omega-system svc/prometheus 9090:9090
# Query: histogram_quantile(0.99, constellation_sync_latency_ms_bucket)

# Scale up
kubectl scale deployment constellation-server -n omega-system --replicas=15
```

### Debug Mode

Enable verbose logging:

```bash
helm upgrade omega ./omega-constellation \
  --set config.logging.level=debug
```

Or for specific pod:
```bash
kubectl set env deployment/constellation-server RUST_LOG=debug -n omega-system
```

---

## 🚀 Production Checklist

- [ ] Secrets externalized (not in Git)
- [ ] TLS enabled for all services
- [ ] Resource limits configured
- [ ] Autoscaling enabled (HPA)
- [ ] Pod disruption budgets configured
- [ ] Network policies applied
- [ ] Monitoring and alerting setup
- [ ] Backup strategy implemented
- [ ] Disaster recovery plan documented
- [ ] Load testing completed
- [ ] Security scanning passed (Trivy, Falco)
- [ ] Multi-region deployment (if required)

---

## 📚 Additional Resources

- [Architecture Documentation](../../specs/ARCHITECTURE.md)
- [Implementation Plan](../../specs/IMPLEMENTATION_PLAN.md)
- [RuVector-Postgres Documentation](https://github.com/ruvnet/ruvector-postgres)
- [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/configuration/overview/)
- [Helm Documentation](https://helm.sh/docs/)

---

**Generated:** 2024
**Version:** 0.1.0
**Maintainers:** Exogenesis Omega Team
