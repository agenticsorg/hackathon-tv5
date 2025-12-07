# Prometheus Alert Rules

This directory contains Prometheus alerting rules for the Media Gateway platform.

## Files

- `alerts.yml` - Comprehensive alert rule definitions organized by category

## Alert Categories

### 1. Availability Alerts (`media-gateway-availability`)
- **ServiceDown** - Critical: Service is completely down (1m)
- **ServiceHighErrorRate** - Warning: Error rate > 5% (5m)
- **ServiceCriticalErrorRate** - Critical: Error rate > 10% (2m)

### 2. Latency Alerts (`media-gateway-latency`)
- **HighLatencyP95** - Warning: P95 latency > 500ms (5m)
- **HighLatencyP99** - Critical: P99 latency > 1s (5m)
- **VeryHighLatencyP50** - Warning: P50 latency > 200ms (10m)

### 3. Resource Alerts (`media-gateway-resources`)
- **DatabaseConnectionPoolExhausted** - Critical: No idle connections (2m)
- **DatabaseConnectionPoolLow** - Warning: < 2 idle connections (5m)
- **RedisConnectionFailed** - Critical: Redis unreachable (1m)
- **RedisHighMemoryUsage** - Warning: Redis memory > 85% (5m)
- **HighMemoryUsage** - Warning: Process memory > 512MB (5m)
- **CriticalMemoryUsage** - Critical: Process memory > 768MB (2m)
- **HighCPUUsage** - Warning: CPU > 80% (10m)

### 4. Business Logic Alerts (`media-gateway-business`)
- **SearchLatencyHigh** - Warning: Search P95 > 400ms SLO (5m)
- **SearchLatencyCritical** - Critical: Search P95 > 1s (2m)
- **RecommendationServiceDegraded** - Warning: Error rate > 10% (5m)
- **RecommendationServiceDown** - Critical: Error rate > 50% (2m)
- **AuthServiceHighLatency** - Warning: Auth P95 > 300ms (5m)
- **PlaybackBufferingHigh** - Warning: Buffering rate > 5% (5m)
- **UploadFailureRateHigh** - Warning: Upload failure rate > 5% (5m)

### 5. Database Alerts (`media-gateway-database`)
- **DatabaseHighConnections** - Warning: > 80 active connections (5m)
- **DatabaseSlowQueries** - Warning: Mean execution time > 1s (5m)
- **DatabaseDeadlocks** - Warning: Deadlocks detected (1m)
- **DatabaseReplicationLag** - Warning: Replication lag > 10s (2m)

### 6. Kafka Alerts (`media-gateway-kafka`)
- **KafkaConsumerLag** - Warning: Lag > 10k messages (5m)
- **KafkaConsumerLagCritical** - Critical: Lag > 100k messages (2m)
- **KafkaOfflinePartitions** - Critical: Offline partitions detected (1m)
- **KafkaUnderReplicatedPartitions** - Warning: Under-replicated partitions (5m)

## Severity Levels

- **Critical**: Immediate action required, service degradation or outage
- **Warning**: Attention needed, potential issues developing

## Alert Labels

All alerts include:
- `severity`: critical or warning
- `category`: availability, performance, resources, business, database, messaging

## Alert Annotations

All alerts provide:
- `summary`: Brief description of the alert
- `description`: Detailed information with metric values
- `runbook_url`: Link to runbook for remediation steps

## Evaluation Intervals

- Most rule groups: 30s
- Ensures timely detection while balancing resource usage

## For Durations

Alert durations are tuned based on severity:
- **Critical alerts**: 1-2 minutes (fast response needed)
- **Warning alerts**: 5-10 minutes (avoid false positives)

## Metric Dependencies

These alerts expect the following metrics to be available:

### HTTP Metrics
- `up` - Service availability
- `http_requests_total` - Request counts by status
- `http_request_duration_seconds_bucket` - Request latency histograms

### Database Metrics (SQLx)
- `sqlx_pool_connections_idle` - Idle connection count
- `sqlx_pool_connections_active` - Active connection count
- `sqlx_pool_connections_pending` - Pending connection requests

### Redis Metrics
- `redis_up` - Redis availability
- `redis_memory_used_bytes` - Current memory usage
- `redis_memory_max_bytes` - Maximum configured memory

### Process Metrics
- `process_resident_memory_bytes` - Process memory usage
- `process_cpu_seconds_total` - CPU time

### Business Metrics
- `search_request_duration_seconds_bucket` - Search latency
- `recommendation_requests_total` - Recommendation requests by status
- `playback_buffer_events_total` - Playback buffering events
- `upload_requests_total` - Upload requests by status

### PostgreSQL Metrics
- `pg_stat_database_numbackends` - Active connections
- `pg_stat_statements_mean_exec_time` - Query execution time
- `pg_stat_database_deadlocks` - Deadlock count
- `pg_replication_lag` - Replication lag in seconds

### Kafka Metrics
- `kafka_consumer_lag` - Consumer lag by topic
- `kafka_server_replica_manager_offline_replica_count` - Offline partitions
- `kafka_server_replica_manager_under_replicated_partitions` - Under-replicated partitions

## Configuration in Prometheus

The `prometheus.yml` configuration references this file:

```yaml
rule_files:
  - /etc/prometheus/alerts.yml
```

## Docker Volume Mounting

When running Prometheus in Docker, mount this file:

```yaml
volumes:
  - ./config/prometheus/alerts.yml:/etc/prometheus/alerts.yml:ro
```

## Testing Alerts

Validate alert rule syntax:

```bash
promtool check rules config/prometheus/alerts.yml
```

Test alert evaluation:

```bash
promtool test rules config/prometheus/alerts_test.yml
```

## Alert Manager Integration

These alerts are designed to work with Alertmanager for routing and notifications. Configure Alertmanager in `prometheus.yml`:

```yaml
alerting:
  alertmanagers:
    - static_configs:
        - targets:
            - alertmanager:9093
```

## Runbook URLs

All runbook URLs point to `https://docs.media-gateway.io/runbooks/`. Update these URLs to point to your actual runbook documentation.

## Maintenance

- Review alert thresholds quarterly based on SLO changes
- Update for durations based on false positive rates
- Add new alerts as new metrics become available
- Archive unused alerts when features are deprecated

## Related Documentation

- [Prometheus Configuration](../prometheus.yml)
- [Grafana Dashboards](../grafana/)
- [Alertmanager Configuration](../alertmanager/)
- [SLO Definitions](../../docs/slo.md)
