# Cloud Monitoring Setup - Nexus-UMMID Metadata API

Comprehensive monitoring and alerting configuration for production Cloud Run deployment.

## Overview

- **GCP Project**: `agentics-foundation25lon-1899`
- **Service**: `metadata-api` (Cloud Run)
- **SLO**: 99.9% availability (43.2 minutes downtime/month)
- **Error Budget**: 0.1% error rate

## Components

### 1. Alert Policies (`alerts.yaml`)

Production-grade alert policies with actionable thresholds:

| Alert | Threshold | Duration | Severity | Description |
|-------|-----------|----------|----------|-------------|
| **High P99 Latency** | > 500ms | 5 min | Warning | User experience degradation |
| **High Error Rate** | > 1% | 3 min | Critical | SLO at risk, error budget burn |
| **High Instance Count** | > 80 instances | 5 min | Warning | Cost spike or performance issue |
| **High Memory Usage** | > 80% | 5 min | Warning | Risk of OOM kills |
| **SLO Fast Burn Rate** | 10x error budget | 1 hour | Critical | Error budget exhaustion in 3 days |
| **Deployment Health** | Startup > 60s | 1 min | Critical | Broken deployment |

### 2. Monitoring Dashboard (`dashboard.json`)

Cloud Monitoring dashboard with:

- **Request Rate**: Real-time req/s by status code class
- **Latency Percentiles**: P50, P95, P99 tracking
- **Error Rates**: 4xx and 5xx as percentage
- **Instance Count**: Active Cloud Run instances
- **Memory/CPU**: Resource utilization
- **SLO Compliance**: Real-time availability tracking
- **Error Budget**: Remaining budget visualization
- **Logs Panel**: Recent errors

### 3. Metrics Middleware (`src/middleware/metrics.ts`)

Prometheus-compatible custom metrics:

```typescript
// Request duration histogram
metadata_api_request_duration_milliseconds{method, endpoint, status_code}

// Request counter
metadata_api_requests_total{method, endpoint, status_code}

// Error counter
metadata_api_errors_total{error_type}

// In-flight requests
metadata_api_requests_in_flight

// Service uptime
metadata_api_uptime_seconds
```

**Endpoints**:
- `/metrics` - Prometheus format
- `/metrics?format=json` - JSON summary
- `/health` - Health check with metrics

## Setup Instructions

### 1. Deploy Alert Policies

```bash
# Set project
gcloud config set project agentics-foundation25lon-1899

# Create notification channels first (if not exist)
gcloud alpha monitoring channels create \
  --display-name="Email - On-Call" \
  --type=email \
  --channel-labels=email_address=oncall@example.com

gcloud alpha monitoring channels create \
  --display-name="Slack - Alerts" \
  --type=slack \
  --channel-labels=url=https://hooks.slack.com/services/YOUR/WEBHOOK/URL

# Deploy alert policies
gcloud alpha monitoring policies create --policy-from-file=monitoring/alerts.yaml
```

### 2. Import Dashboard

**Via Console**:
1. Go to [Cloud Monitoring Dashboards](https://console.cloud.google.com/monitoring/dashboards)
2. Click "Create Dashboard" → "Import from JSON"
3. Upload `monitoring/dashboard.json`
4. Save

**Via gcloud CLI**:
```bash
gcloud monitoring dashboards create --config-from-file=monitoring/dashboard.json
```

### 3. Enable Metrics Middleware

Update `src/index.ts`:

```typescript
import { metricsMiddleware, metricsHandler, healthCheckWithMetrics } from './middleware/metrics';

// Add metrics middleware (before routes)
app.use(metricsMiddleware);

// Add metrics endpoint
app.get('/metrics', metricsHandler);

// Enhanced health check
app.get('/health', healthCheckWithMetrics);
```

### 4. Verify Metrics Collection

```bash
# Local testing
npm run dev

# Check metrics endpoint
curl http://localhost:8080/metrics
curl http://localhost:8080/metrics?format=json

# After deployment to Cloud Run
curl https://metadata-api-[hash]-uc.a.run.app/metrics
```

### 5. Test Alerts

```bash
# Simulate high latency
ab -n 1000 -c 50 https://metadata-api-[hash]-uc.a.run.app/api/v1/metadata

# Simulate errors (should trigger error rate alert)
for i in {1..100}; do
  curl https://metadata-api-[hash]-uc.a.run.app/api/v1/metadata/invalid-id
done
```

## Alert Descriptions

### High P99 Latency
**What it means**: 99% of requests are slower than 500ms
**Impact**: Poor user experience, potential SLO violation
**Response Time**: 15 minutes
**Runbook**: https://docs.nexus-ummid.io/runbooks/high-latency

**Investigation**:
1. Check Cloud Run dashboard for CPU/memory pressure
2. Review slow query logs from Firestore
3. Check external API connector latency
4. Verify no network issues

**Resolution**:
- Scale up Cloud Run instances
- Optimize database queries
- Add caching for hot data
- Optimize external API calls

### High Error Rate
**What it means**: More than 1% of requests returning 5xx errors
**Impact**: SLO violation, rapid error budget burn
**Response Time**: Immediate (paged)
**Runbook**: https://docs.nexus-ummid.io/runbooks/high-error-rate

**Investigation**:
```bash
# View error logs
gcloud logging read "resource.type=cloud_run_revision AND \
  resource.labels.service_name=metadata-api AND \
  severity>=ERROR" \
  --limit 50 \
  --format json
```

**Resolution**:
- Rollback if deployment-related
- Fix critical bugs
- Add circuit breakers for failing dependencies
- Increase error handling

### High Instance Count
**What it means**: Using 80+ of max 100 instances
**Impact**: High costs, possible performance issues
**Response Time**: 30 minutes
**Runbook**: https://docs.nexus-ummid.io/runbooks/high-instance-count

**Investigation**:
1. Check if traffic surge is legitimate
2. Review request rate trends
3. Check average request duration
4. Look for memory leaks

**Resolution**:
- Increase max instances if legitimate traffic
- Enable Cloud Armor rate limiting for attacks
- Optimize code performance
- Fix memory leaks

### High Memory Usage
**What it means**: Instances using >80% of allocated memory
**Impact**: Risk of OOM kills, crashes
**Response Time**: 30 minutes
**Runbook**: https://docs.nexus-ummid.io/runbooks/high-memory-usage

**Investigation**:
```bash
# Check memory trends
gcloud monitoring read "resource.type=cloud_run_revision AND \
  metric.type=run.googleapis.com/container/memory/utilizations" \
  --filter "resource.labels.service_name=metadata-api"
```

**Resolution**:
- Increase Cloud Run memory allocation (currently 512Mi)
- Fix memory leaks
- Implement cache eviction
- Optimize data structures

## SLO Tracking

### Availability SLO: 99.9%

**Error Budget**: 0.1% = 43.2 minutes/month

**Calculation**:
```
Error Budget Minutes Remaining = 43.2 - (Downtime This Month)
Error Budget % = (Successful Requests / Total Requests)
```

**Burn Rate**:
- **Normal**: 1x (100% error budget used over 30 days)
- **Warning**: 5x (budget exhausted in 6 days)
- **Critical**: 10x (budget exhausted in 3 days)

**Query Error Budget**:
```bash
gcloud monitoring read \
  "resource.type=cloud_run_revision AND \
   resource.labels.service_name=metadata-api AND \
   metric.type=run.googleapis.com/request_count" \
  --filter "metric.labels.response_code_class!=2xx" \
  --aggregate sum \
  --start-time "$(date -u -d '30 days ago' +%Y-%m-%dT%H:%M:%SZ)"
```

## Notification Channels

Configure these notification channels:

1. **Email - On-Call**: `oncall@example.com`
2. **Slack - Alerts**: Webhook to #alerts channel
3. **PagerDuty - Critical**: For critical alerts only

**Create channels**:
```bash
# Email
gcloud alpha monitoring channels create \
  --display-name="Email - On-Call" \
  --type=email \
  --channel-labels=email_address=oncall@nexus-ummid.io

# Slack
gcloud alpha monitoring channels create \
  --display-name="Slack - Alerts" \
  --type=slack \
  --channel-labels=url=YOUR_WEBHOOK_URL
```

## Cost Optimization

**Monitoring Costs**:
- Alert policies: Free (first 500 conditions)
- Dashboard: Free
- Metrics ingestion: $0.258/MiB (150 MiB free/month)
- Log ingestion: $0.50/GiB (first 50 GiB free/month)

**Estimated Monthly Cost**: $5-$10 for 400M+ user scale

## Runbook Links

- [High Latency Runbook](https://docs.nexus-ummid.io/runbooks/high-latency)
- [High Error Rate Runbook](https://docs.nexus-ummid.io/runbooks/high-error-rate)
- [High Instance Count Runbook](https://docs.nexus-ummid.io/runbooks/high-instance-count)
- [High Memory Usage Runbook](https://docs.nexus-ummid.io/runbooks/high-memory-usage)
- [SLO Burn Rate Runbook](https://docs.nexus-ummid.io/runbooks/slo-burn-rate)
- [Deployment Health Runbook](https://docs.nexus-ummid.io/runbooks/deployment-health)

## Testing Alerts

### Load Testing
```bash
# Install Apache Bench
sudo apt-get install apache2-utils

# Generate load
ab -n 10000 -c 100 https://metadata-api-[hash]-uc.a.run.app/api/v1/metadata
```

### Error Injection
```bash
# Trigger 5xx errors (requires test endpoint)
for i in {1..1000}; do
  curl -X POST https://metadata-api-[hash]-uc.a.run.app/test/error-500
done
```

## Maintenance

### Weekly
- Review alert noise (false positives)
- Check SLO compliance
- Review dashboard for anomalies

### Monthly
- Review and adjust alert thresholds
- Update runbooks based on incidents
- Analyze error budget consumption
- Review cost vs. value of alerts

## Troubleshooting

### Alerts Not Firing
```bash
# Check alert policy status
gcloud alpha monitoring policies list

# Test notification channel
gcloud alpha monitoring channels test <CHANNEL_ID>
```

### Metrics Not Appearing
```bash
# Check if metrics are being written
gcloud monitoring read "resource.type=cloud_run_revision AND \
  metric.type=custom.googleapis.com/metadata-api/request_duration_milliseconds"

# Check service logs
gcloud logging read "resource.type=cloud_run_revision AND \
  resource.labels.service_name=metadata-api"
```

### Dashboard Not Loading
1. Verify project permissions
2. Check dashboard JSON syntax
3. Ensure all metric types exist
4. Verify resource names are correct

## Support

- **Documentation**: https://docs.nexus-ummid.io/monitoring
- **Incidents**: Create incident in PagerDuty
- **Questions**: #metadata-api-oncall Slack channel

---

**Last Updated**: 2025-12-06
**Maintained By**: Platform Engineering Team
**Version**: 1.0.0
