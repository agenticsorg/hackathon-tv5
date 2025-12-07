# Performance Testing Framework

k6-based performance testing suite for the Media Gateway platform.

## Overview

This framework provides comprehensive performance testing including baseline, stress, spike, and soak tests for all microservices.

## Prerequisites

```bash
# Install k6
# macOS
brew install k6

# Linux
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# Docker
docker pull grafana/k6:latest
```

## Test Types

### 1. Baseline Test (`baseline.js`)

**Purpose**: Establish performance baseline under normal load

**Configuration**:
- 10,000 virtual users
- 1,000 RPS target
- 30-minute duration
- Tests all main endpoints

**Usage**:
```bash
k6 run tests/performance/k6/baseline.js

# With custom environment
BASE_URL=https://api.example.com k6 run tests/performance/k6/baseline.js

# Output to InfluxDB
k6 run --out influxdb=http://localhost:8086/k6 tests/performance/k6/baseline.js
```

**Metrics to Monitor**:
- p95 response times should meet SLA targets
- Error rate < 1%
- Steady-state throughput ≥ 1000 RPS

### 2. Stress Test (`stress.js`)

**Purpose**: Find system breaking point by gradually increasing load

**Configuration**:
- 20,000 virtual users (2x normal)
- 3,500 RPS peak
- Gradual 5-minute ramp-up stages

**Usage**:
```bash
k6 run tests/performance/k6/stress.js

# With relaxed thresholds
k6 run --no-thresholds tests/performance/k6/stress.js
```

**What to Analyze**:
- At what load does performance degrade?
- How does error rate change under stress?
- Which service becomes the bottleneck first?
- Resource utilization at breaking point

### 3. Spike Test (`spike.js`)

**Purpose**: Test system behavior under sudden traffic spikes

**Configuration**:
- 100,000 users sudden load (10x normal)
- 30-second ramp to peak
- Measures system recovery

**Usage**:
```bash
k6 run tests/performance/k6/spike.js

# Monitor recovery metrics
k6 run --out json=spike-results.json tests/performance/k6/spike.js
```

**Key Questions**:
- Does the system survive the spike?
- How long to recover to normal performance?
- Are there cascading failures?
- Do circuit breakers work correctly?

### 4. Soak Test (`soak.js`)

**Purpose**: Detect memory leaks and degradation over extended periods

**Configuration**:
- 24-hour sustained load
- 10,000 constant VUs
- Memory leak detection
- Resource monitoring

**Usage**:
```bash
# Run in background (recommended)
nohup k6 run tests/performance/k6/soak.js > soak-test.log 2>&1 &

# Monitor progress
tail -f soak-test.log

# Docker for long-running tests
docker run -i --rm \
  -v $(pwd):/tests \
  grafana/k6:latest \
  run /tests/performance/k6/soak.js
```

**What to Watch**:
- Response times should remain stable
- Memory usage should not trend upward
- No connection leaks
- Error rates stay consistent

## Configuration

### Environment Variables

```bash
# Service URLs
export BASE_URL=http://localhost:8080
export AUTH_URL=http://localhost:8081
export DISCOVERY_URL=http://localhost:8082
export SONA_URL=http://localhost:8083
export SYNC_URL=http://localhost:8084
export PLAYBACK_URL=http://localhost:8085
export INGESTION_URL=http://localhost:8086

# Test credentials
export TEST_EMAIL=test@example.com
export TEST_PASSWORD=testpass123
```

### Performance Targets (from SPARC)

| Service | Target | Threshold |
|---------|--------|-----------|
| Search API | p95 < 500ms | p99 < 1s |
| SONA Recommendations | p95 < 5ms | p99 < 10ms |
| Sync Operations | p95 < 100ms | p99 < 200ms |
| Auth Operations | p95 < 50ms | p99 < 100ms |
| Playback | p95 < 200ms | p99 < 500ms |
| Ingestion | p95 < 1s | p99 < 2s |

## Monitoring and Visualization

### InfluxDB + Grafana Setup

```bash
# Start InfluxDB
docker run -d -p 8086:8086 \
  --name influxdb \
  -v influxdb-data:/var/lib/influxdb2 \
  influxdb:2.0

# Create k6 database
docker exec influxdb influx setup \
  --username admin \
  --password admin123 \
  --org k6 \
  --bucket k6 \
  --retention 0 \
  --force

# Start Grafana
docker run -d -p 3000:3000 \
  --name grafana \
  -v grafana-data:/var/lib/grafana \
  grafana/grafana
```

### Run Tests with InfluxDB Output

```bash
k6 run --out influxdb=http://localhost:8086/k6 tests/performance/k6/baseline.js
```

### Import Grafana Dashboard

1. Open Grafana at http://localhost:3000
2. Add InfluxDB data source
3. Import k6 dashboard (ID: 2587)

## Analysis and Reporting

### Generate HTML Report

```bash
# Install k6-reporter
npm install -g k6-to-junit

# Run test with summary export
k6 run --summary-export=summary.json tests/performance/k6/baseline.js

# Convert to HTML
k6-to-junit summary.json > results.html
```

### Key Metrics to Track

1. **Response Time**:
   - Average, p50, p95, p99
   - Trends over time
   - Per-endpoint breakdown

2. **Throughput**:
   - Requests per second
   - Data transferred
   - Successful vs failed requests

3. **Error Rate**:
   - HTTP errors (4xx, 5xx)
   - Network errors
   - Timeout errors

4. **Resource Utilization**:
   - CPU usage
   - Memory usage
   - Connection count
   - Database connections

## Troubleshooting

### High Error Rates

```bash
# Check service health
curl http://localhost:8081/health

# Review service logs
docker logs <service-name>

# Reduce load to find threshold
k6 run --vus 1000 --duration 5m tests/performance/k6/baseline.js
```

### Slow Response Times

```bash
# Enable detailed metrics
k6 run --http-debug="full" tests/performance/k6/baseline.js

# Profile individual endpoint
k6 run --http-debug tests/performance/k6/baseline.js 2>&1 | grep "request_url"
```

### Memory Issues

```bash
# Monitor during soak test
docker stats

# Check for leaks
k6 run --vus 100 --duration 1h tests/performance/k6/soak.js
```

## Best Practices

1. **Run Tests Regularly**: Include performance tests in CI/CD pipeline
2. **Isolate Environment**: Use dedicated test environment
3. **Warm Up Services**: Run warm-up requests before actual test
4. **Monitor Resources**: Track CPU, memory, database connections
5. **Baseline First**: Always establish baseline before stress/spike tests
6. **Analyze Trends**: Compare results over time, not just pass/fail
7. **Test Realistic Scenarios**: Use production-like data and patterns
8. **Document Results**: Keep historical results for regression analysis

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Performance Tests

on:
  schedule:
    - cron: '0 2 * * *' # Daily at 2 AM
  workflow_dispatch:

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup k6
        run: |
          sudo gpg -k
          sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
          echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
          sudo apt-get update
          sudo apt-get install k6

      - name: Run baseline test
        env:
          BASE_URL: ${{ secrets.PERF_BASE_URL }}
        run: |
          k6 run --summary-export=baseline-summary.json tests/performance/k6/baseline.js

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: performance-results
          path: baseline-summary.json
```

## Support

For issues or questions:
- Review test logs: `k6 run --verbose tests/performance/k6/<test>.js`
- Check k6 documentation: https://k6.io/docs/
- Review service health endpoints
- Contact DevOps team for infrastructure issues
