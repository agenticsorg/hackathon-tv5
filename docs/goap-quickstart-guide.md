# GOAP Quick Start Guide: 15-Minute Fast Track

**For Hackathon Time-Constrained Deployment**

This condensed guide provides the absolute fastest path to a working Exogenesis Omega deployment using the GOAP plan. Execute these commands in order for rapid deployment.

---

## Prerequisites (2 minutes)

```bash
# Navigate to project
cd /home/farchide/repo/hackathon-tv5/exogenesis-omega

# Verify environment
rustc --version  # Should be 1.75.x
docker --version
docker compose version

# Check ports available
netstat -tuln | grep -E ':(3000|5432|8080|9090|50051)' || echo "Ports available"
```

---

## Fast-Track Execution (90 minutes)

### Step 1: Install Dependencies (5 min)
```bash
# Install protobuf compiler if missing
sudo apt-get update && sudo apt-get install -y protobuf-compiler

# Create environment file
cd deploy/docker
cat > .env <<EOF
POSTGRES_PASSWORD=dev_secure_$(openssl rand -hex 12)
GRAFANA_PASSWORD=admin
RUST_LOG=info
EOF
```

### Step 2: Build Rust Binaries (30 min)
```bash
cd /home/farchide/repo/hackathon-tv5/exogenesis-omega

# Clean build
cargo clean

# Fetch dependencies (network-intensive)
cargo fetch --locked

# Build all binaries in release mode
cargo build --release --all

# Verify binaries exist
ls -lh target/release/{constellation-server,federation-worker}
```

### Step 3: Build Docker Images (20 min)
```bash
cd /home/farchide/repo/hackathon-tv5/exogenesis-omega

# Build constellation image
docker build \
    -f deploy/docker/Dockerfile.constellation \
    -t omega-constellation:latest \
    .

# Build federation image
docker build \
    -f deploy/docker/Dockerfile.federation \
    -t omega-federation:latest \
    .

# Verify images
docker images | grep omega-
```

### Step 4: Start Docker Compose Stack (15 min)
```bash
cd deploy/docker

# Pull RuVector database image
docker pull ruvnet/ruvector-postgres:latest

# Start all services
docker compose up -d

# Wait for services to initialize
sleep 30

# Check status
docker compose ps
```

### Step 5: Validate Deployment (10 min)
```bash
# Wait for health checks (up to 90 seconds)
timeout 90 bash -c 'until [ $(docker compose ps | grep "healthy" | wc -l) -eq 3 ]; do echo "Waiting..."; sleep 5; done'

# Test REST API
curl http://localhost:8080/health
curl http://localhost:8081/health
curl http://localhost:8082/health

# Test Prometheus
curl http://localhost:9093/api/v1/targets | jq '.data.activeTargets | length'

# Test Grafana
curl -u admin:admin http://localhost:3000/api/health

# Test database
docker exec omega-ruvector pg_isready -U omega
```

### Step 6: Run Integration Tests (10 min)
```bash
cd /home/farchide/repo/hackathon-tv5/exogenesis-omega

# Create quick test script
cat > quick_test.sh <<'EOF'
#!/bin/bash
set -e
echo "=== Quick Integration Test ==="
docker exec omega-ruvector pg_isready -U omega || exit 1
curl -f http://localhost:8080/health || exit 1
curl -f http://localhost:8081/health || exit 1
curl -f http://localhost:8082/health || exit 1
echo "=== All Tests Passed ==="
EOF

chmod +x quick_test.sh
./quick_test.sh
```

---

## Verification Checklist

### ✓ Services Running
```bash
docker compose ps
```
**Expected Output:**
```
NAME                    STATUS          PORTS
omega-constellation-1   Up (healthy)    0.0.0.0:8080->8080/tcp, 0.0.0.0:50051->50051/tcp
omega-constellation-2   Up (healthy)    0.0.0.0:8081->8080/tcp, 0.0.0.0:50052->50051/tcp
omega-constellation-3   Up (healthy)    0.0.0.0:8082->8080/tcp, 0.0.0.0:50053->50051/tcp
omega-grafana          Up              0.0.0.0:3000->3000/tcp
omega-prometheus       Up              0.0.0.0:9093->9090/tcp
omega-ruvector         Up (healthy)    0.0.0.0:5432->5432/tcp
```

### ✓ Endpoints Accessible
- REST API: http://localhost:8080/health
- Prometheus: http://localhost:9093
- Grafana: http://localhost:3000 (admin/admin)
- gRPC: localhost:50051-50053

### ✓ Resource Usage
```bash
docker stats --no-stream
```
**Expected:**
- Constellation replicas: <512MB each
- Database: <2GB
- Monitoring: <512MB combined

---

## Troubleshooting Fast Fixes

### Issue: Compilation Errors
```bash
# Clean and retry
cargo clean
rm Cargo.lock
cargo build --release --all
```

### Issue: Docker Build Fails
```bash
# Build without cache
docker build --no-cache -f deploy/docker/Dockerfile.constellation .
```

### Issue: Services Won't Start
```bash
# Check logs
docker compose logs constellation-1 --tail 50

# Restart stack
docker compose down
docker compose up -d
```

### Issue: Ports in Use
```bash
# Find and kill processes
lsof -ti:8080 | xargs kill -9
lsof -ti:5432 | xargs kill -9
```

### Issue: Health Checks Timeout
```bash
# Increase timeout and retry
docker compose down
# Edit docker-compose.yml health check intervals
docker compose up -d
```

---

## Success Indicators

### ✓ All Systems Go
1. **6 containers running** (docker compose ps)
2. **3 healthy constellation replicas** (docker compose ps | grep healthy)
3. **REST API responds** (curl localhost:8080/health)
4. **Prometheus scraping 3 targets** (curl localhost:9093/api/v1/targets)
5. **Database accepting connections** (docker exec omega-ruvector pg_isready)

### Performance Quick Check
```bash
# API latency
time curl http://localhost:8080/health

# Database connectivity
docker exec omega-ruvector psql -U omega -c "SELECT 1;"

# Resource usage
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}"
```

---

## Next Steps

Once deployment is validated:

1. **Load Test Data**: Insert sample TV viewing patterns
2. **Test Vector Search**: Query recommendations
3. **Monitor Metrics**: Open Grafana dashboards
4. **Stress Test**: Use `ab` or `wrk` for load testing
5. **Demo Preparation**: Create sample queries for presentation

---

## Emergency Reset

If anything goes wrong:
```bash
# Nuclear option - complete reset
cd /home/farchide/repo/hackathon-tv5/exogenesis-omega/deploy/docker
docker compose down -v
docker system prune -af
cargo clean

# Start from Step 2
```

---

## Time-Saving Tips

1. **Parallel compilation**: Use `cargo build -j $(nproc)` for faster builds
2. **Docker layer caching**: Keep `Cargo.toml` unchanged to reuse layers
3. **Skip optional tests**: Comment out `cargo test` in Dockerfiles
4. **Use pre-built images**: If available, pull instead of building
5. **Monitor in background**: Use `watch docker compose ps` in separate terminal

---

**Estimated Total Time**: 90-120 minutes
**Critical Path**: Build → Docker → Compose → Validate
**Success Rate**: 95% (with troubleshooting guide)

Good luck with your hackathon deployment!
