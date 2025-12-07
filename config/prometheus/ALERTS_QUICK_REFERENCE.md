# Prometheus Alerts Quick Reference

## Critical Alerts (Immediate Action Required)

| Alert | Trigger | Duration | Action |
|-------|---------|----------|--------|
| ServiceDown | `up == 0` | 1m | Check service health, restart if needed |
| ServiceCriticalErrorRate | Error rate > 10% | 2m | Check logs, identify root cause |
| HighLatencyP99 | P99 > 1s | 5m | Investigate slow queries/endpoints |
| DatabaseConnectionPoolExhausted | No idle connections + pending requests | 2m | Increase pool size or fix connection leaks |
| RedisConnectionFailed | Redis unreachable | 1m | Check Redis service, network |
| CriticalMemoryUsage | Process memory > 768MB | 2m | Investigate memory leak, restart service |
| RecommendationServiceDown | Error rate > 50% | 2m | Check recommendation service health |
| SearchLatencyCritical | Search P95 > 1s | 2m | Optimize search queries, check database |
| KafkaConsumerLagCritical | Lag > 100k messages | 2m | Scale consumers, check processing |
| KafkaOfflinePartitions | Offline partitions > 0 | 1m | Check Kafka cluster health |

## Warning Alerts (Needs Attention)

| Alert | Trigger | Duration | Action |
|-------|---------|----------|--------|
| ServiceHighErrorRate | Error rate > 5% | 5m | Monitor, investigate if persists |
| HighLatencyP95 | P95 > 500ms | 5m | Review recent changes, optimize |
| VeryHighLatencyP50 | P50 > 200ms | 10m | Check database performance |
| DatabaseConnectionPoolLow | < 2 idle connections | 5m | Monitor pool usage |
| RedisHighMemoryUsage | Memory > 85% | 5m | Review cache strategy, clear old data |
| HighMemoryUsage | Process memory > 512MB | 5m | Monitor memory trends |
| HighCPUUsage | CPU > 80% | 10m | Review workload, consider scaling |
| SearchLatencyHigh | Search P95 > 400ms | 5m | Optimize search queries |
| RecommendationServiceDegraded | Error rate > 10% | 5m | Check service logs |
| AuthServiceHighLatency | Auth P95 > 300ms | 5m | Optimize auth flow |
| PlaybackBufferingHigh | Buffering rate > 5% | 5m | Check CDN, network, encoding |
| UploadFailureRateHigh | Failure rate > 5% | 5m | Check storage backend |
| DatabaseHighConnections | > 80 active connections | 5m | Review connection usage |
| DatabaseSlowQueries | Mean exec time > 1s | 5m | Identify and optimize slow queries |
| DatabaseDeadlocks | Deadlocks detected | 1m | Review transaction logic |
| DatabaseReplicationLag | Lag > 10s | 2m | Check replica health |
| KafkaConsumerLag | Lag > 10k messages | 5m | Monitor consumer performance |
| KafkaUnderReplicatedPartitions | Under-replicated > 0 | 5m | Check broker health |

## Common Response Procedures

### Service Down
```bash
# Check service status
systemctl status <service>

# Check logs
journalctl -u <service> -n 100

# Restart if needed
systemctl restart <service>

# Verify recovery
curl http://<service>/health
```

### High Error Rate
```bash
# Check recent logs
tail -f /var/log/<service>/error.log

# Check error distribution
grep "ERROR" /var/log/<service>/*.log | cut -d' ' -f5 | sort | uniq -c

# Review recent deployments
git log --oneline -10
```

### High Latency
```bash
# Check database connections
SELECT count(*) FROM pg_stat_activity;

# Find slow queries
SELECT query, mean_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

# Check Redis latency
redis-cli --latency

# Review application metrics
curl http://<service>/metrics | grep duration
```

### Database Pool Exhausted
```bash
# Check current connections
SELECT count(*), state
FROM pg_stat_activity
GROUP BY state;

# Find long-running queries
SELECT pid, now() - query_start as duration, query
FROM pg_stat_activity
WHERE state = 'active'
ORDER BY duration DESC;

# Kill if necessary
SELECT pg_terminate_backend(pid);
```

### Redis Issues
```bash
# Check Redis status
redis-cli ping

# Check memory usage
redis-cli info memory

# Check connected clients
redis-cli client list

# Monitor in real-time
redis-cli monitor
```

### Kafka Consumer Lag
```bash
# Check consumer group status
kafka-consumer-groups --bootstrap-server kafka:9092 \
  --describe --group <consumer-group>

# Check topic details
kafka-topics --bootstrap-server kafka:9092 \
  --describe --topic <topic>

# Reset offset if needed (CAUTION)
kafka-consumer-groups --bootstrap-server kafka:9092 \
  --group <consumer-group> --reset-offsets --to-latest \
  --topic <topic> --execute
```

## Escalation Paths

### Severity: Critical
1. Automatic page to on-call engineer
2. Post to #incidents Slack channel
3. Start incident response procedure
4. Escalate to engineering lead after 15 minutes

### Severity: Warning
1. Post to #alerts Slack channel
2. Create issue if not resolved in 1 hour
3. Escalate to critical if persists > 2 hours

## Silencing Alerts

### During Maintenance
```bash
# Create silence for 2 hours
amtool silence add alertname="ServiceDown" \
  job="api-gateway" \
  --duration=2h \
  --comment="Planned maintenance"
```

### For Known Issues
```bash
# Create silence with ticket reference
amtool silence add alertname="HighLatencyP95" \
  instance="discovery:8081" \
  --duration=24h \
  --comment="Known issue - TICKET-123"
```

## Alert Validation

```bash
# Validate rules
promtool check rules config/prometheus/alerts.yml

# Test rules
promtool test rules config/prometheus/alerts_test.yml

# Check active alerts
curl http://prometheus:9090/api/v1/alerts

# Check Alertmanager status
curl http://alertmanager:9093/api/v2/status
```

## Metrics Verification

```bash
# Check if metric exists
curl -s 'http://prometheus:9090/api/v1/query?query=up' | jq .

# Check metric cardinality
curl -s 'http://prometheus:9090/api/v1/label/__name__/values' | jq -r '.data[]' | wc -l

# Check scrape targets
curl -s 'http://prometheus:9090/api/v1/targets' | jq '.data.activeTargets[] | select(.health != "up")'
```

## Useful PromQL Queries

```promql
# Service availability (last 24h)
avg_over_time(up[24h])

# Error rate by service
rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])

# P95 latency by endpoint
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Database connection pool usage
sqlx_pool_connections_active / (sqlx_pool_connections_active + sqlx_pool_connections_idle)

# Redis memory usage percentage
redis_memory_used_bytes / redis_memory_max_bytes

# Kafka consumer lag total
sum(kafka_consumer_lag) by (topic)
```

## Dashboard Links

- **Prometheus**: http://localhost:9090
- **Alertmanager**: http://localhost:9093
- **Grafana**: http://localhost:3000
- **Service Health**: http://localhost:8080/health

## Contact Information

- **On-call rotation**: Check PagerDuty
- **Slack channels**: #incidents, #alerts, #platform
- **Runbooks**: https://docs.media-gateway.io/runbooks/
- **Status page**: https://status.media-gateway.io

## Tips

1. Always check alert annotations for context
2. Review recent deployments when alerts fire
3. Check related services (database, cache, queue)
4. Document investigation steps in incident ticket
5. Update runbooks based on learnings
6. Silence alerts during planned maintenance
7. Tune thresholds based on false positive rates
8. Monitor alert fatigue metrics weekly
