# Auth Service Deployment Guide

## Quick Start

### Prerequisites

1. **Redis** (version 6.0+)
```bash
docker run -d --name redis -p 6379:6379 redis:7-alpine
```

2. **RSA Keys** (RS256 for JWT)
```bash
# Generate private key
openssl genrsa -out jwt_private_key.pem 2048

# Extract public key
openssl rsa -in jwt_private_key.pem -pubout -out jwt_public_key.pem

# Move to secrets directory
mkdir -p /secrets
mv jwt_private_key.pem jwt_public_key.pem /secrets/
chmod 600 /secrets/jwt_private_key.pem
```

3. **Environment Variables**
```bash
cp .env.example .env
# Edit .env with your configuration
```

### Build

```bash
cargo build --release --bin auth-service
```

### Run

```bash
export RUST_LOG=info
cargo run --release --bin auth-service
```

The service will start on `http://0.0.0.0:8084`

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --bin auth-service

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/auth-service /usr/local/bin/

EXPOSE 8084
CMD ["auth-service"]
```

### Build and Run

```bash
docker build -t media-gateway-auth:latest .

docker run -d \
  --name auth-service \
  -p 8084:8084 \
  -e BIND_ADDRESS=0.0.0.0:8084 \
  -e REDIS_URL=redis://redis:6379 \
  -v /secrets:/secrets \
  media-gateway-auth:latest
```

## Kubernetes Deployment

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: auth-service-config
data:
  BIND_ADDRESS: "0.0.0.0:8084"
  JWT_ISSUER: "https://api.mediagateway.io"
  JWT_AUDIENCE: "mediagateway-users"
  REDIS_URL: "redis://redis-service:6379"
```

### Secret

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: auth-service-secrets
type: Opaque
data:
  jwt-private-key: <base64-encoded-pem>
  jwt-public-key: <base64-encoded-pem>
```

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: auth-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: auth-service
  template:
    metadata:
      labels:
        app: auth-service
    spec:
      containers:
      - name: auth-service
        image: media-gateway-auth:latest
        ports:
        - containerPort: 8084
        envFrom:
        - configMapRef:
            name: auth-service-config
        volumeMounts:
        - name: jwt-keys
          mountPath: /secrets
          readOnly: true
        livenessProbe:
          httpGet:
            path: /health
            port: 8084
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8084
          initialDelaySeconds: 5
          periodSeconds: 10
      volumes:
      - name: jwt-keys
        secret:
          secretName: auth-service-secrets
```

### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: auth-service
spec:
  selector:
    app: auth-service
  ports:
  - protocol: TCP
    port: 8084
    targetPort: 8084
  type: ClusterIP
```

## GCP Cloud Run Deployment

```bash
# Build for Cloud Run
gcloud builds submit --tag gcr.io/PROJECT_ID/auth-service

# Deploy
gcloud run deploy auth-service \
  --image gcr.io/PROJECT_ID/auth-service \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --set-env-vars REDIS_URL=redis://REDIS_IP:6379 \
  --set-secrets JWT_PRIVATE_KEY_PATH=/secrets/jwt_private_key.pem:jwt-private-key:latest,JWT_PUBLIC_KEY_PATH=/secrets/jwt_public_key.pem:jwt-public-key:latest
```

## Monitoring

### Health Check

```bash
curl http://localhost:8084/health
```

Response:
```json
{
  "status": "healthy",
  "service": "auth-service",
  "version": "0.1.0"
}
```

### Metrics (Prometheus)

Add to your Prometheus scrape config:

```yaml
scrape_configs:
  - job_name: 'auth-service'
    static_configs:
      - targets: ['auth-service:8084']
```

### Logging

Logs are output in JSON format for Cloud Logging:

```json
{
  "timestamp": "2025-12-06T00:00:00.000Z",
  "level": "INFO",
  "message": "Token exchange successful",
  "target": "auth_service::server",
  "user_id": "user-123"
}
```

## Security Considerations

1. **TLS Termination**: Use a reverse proxy (nginx, Cloud Load Balancer)
2. **Rate Limiting**: Implement at API Gateway level
3. **Key Rotation**: Rotate JWT keys every 90 days
4. **Secret Management**: Use GCP Secret Manager in production
5. **Network Policies**: Restrict access to Redis and database

## Performance Tuning

### Redis Connection Pool

```rust
// In production, use connection pooling
redis::Client::open(redis_url)?
    .get_multiplexed_connection_manager()
    .await?
```

### Actix-web Workers

```bash
# Set worker count (default = CPU cores)
export ACTIX_WORKERS=4
```

### Database Connection Pool

```rust
// SQLx connection pool configuration
sqlx::postgres::PgPoolOptions::new()
    .max_connections(20)
    .connect(&database_url)
    .await?
```

## Troubleshooting

### Redis Connection Failed

```bash
# Check Redis is running
redis-cli ping

# Check Redis URL
echo $REDIS_URL
```

### JWT Key Not Found

```bash
# Verify key files exist
ls -l /secrets/jwt_*.pem

# Check permissions
chmod 600 /secrets/jwt_private_key.pem
chmod 644 /secrets/jwt_public_key.pem
```

### Port Already in Use

```bash
# Change bind address
export BIND_ADDRESS=0.0.0.0:8085
```

## Production Checklist

- [ ] TLS certificates configured
- [ ] JWT keys generated and stored in Secret Manager
- [ ] Redis cluster deployed (high availability)
- [ ] Database migrations applied
- [ ] Environment variables set
- [ ] Health checks configured
- [ ] Monitoring and alerting set up
- [ ] Rate limiting enabled at gateway
- [ ] CORS configured for web clients
- [ ] OAuth providers registered
- [ ] Backup and recovery procedures tested

---

**Last Updated:** 2025-12-06
**Version:** 0.1.0
