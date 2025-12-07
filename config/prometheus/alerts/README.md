# Prometheus Alert Rules

This directory contains Prometheus alert rule definitions for the media-gateway services.

## Alert Files

### service-alerts.yml

Core service monitoring alerts organized into the following groups:

#### 1. Service Availability
- **ServiceDown** (critical): Service is unreachable for 2+ minutes
- **HighErrorRate** (warning): HTTP 5xx error rate > 5% for 5 minutes
- **CriticalErrorRate** (critical): HTTP 5xx error rate > 10% for 2 minutes

#### 2. Service Performance
- **HighLatency** (warning): P95 latency > 500ms for 10 minutes
- **CriticalLatency** (critical): P99 latency > 1s for 5 minutes

#### 3. Database
- **DatabaseDown** (critical): PostgreSQL unreachable for 1+ minute
- **DatabaseConnectionPoolExhausted** (critical): No idle connections, requests pending for 2+ minutes
- **DatabaseConnectionPoolLow** (warning): Less than 2 idle connections for 5+ minutes

#### 4. Resources
- **HighMemoryUsage** (warning): Memory usage > 85% for 10 minutes
- **CriticalMemoryUsage** (critical): Memory usage > 95% for 5 minutes
- **HighCPUUsage** (warning): CPU usage > 90% for 10 minutes
- **CriticalCPUUsage** (critical): CPU usage > 95% for 5 minutes

#### 5. Cache (Redis)
- **RedisDown** (critical): Redis unreachable for 1+ minute
- **RedisHighMemoryUsage** (warning): Redis memory > 85% for 5 minutes
- **RedisCriticalMemoryUsage** (critical): Redis memory > 95% for 2 minutes

#### 6. Health Checks
- **ServiceUnhealthy** (warning): Health check failing for 3+ minutes
- **ServiceDegraded** (info): Health check duration > 2s average

## Severity Levels

- **critical**: Requires immediate attention, service degradation or outage
- **warning**: Should be investigated, potential issues developing
- **info**: Informational, may indicate areas for optimization

## Integration

Include this file in your Prometheus configuration:

```yaml
# prometheus.yml
rule_files:
  - /etc/prometheus/alerts/*.yml
```

## Runbook Links

Each alert includes a runbook URL pointing to:
`https://docs.media-gateway.io/runbooks/{alert-type}`

Ensure runbooks are created for each alert type with:
- Cause analysis
- Impact assessment
- Resolution steps
- Prevention measures

## Testing Alerts

To validate alert syntax:

```bash
promtool check rules config/prometheus/alerts/service-alerts.yml
```

To test alert evaluation:

```bash
promtool test rules config/prometheus/alerts/service-alerts.yml
```
