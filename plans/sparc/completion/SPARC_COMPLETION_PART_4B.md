# SPARC Completion Phase - Part 4B: Operational Procedures

**Version:** 1.0.0
**Phase:** SPARC Completion (Phase 5)
**Date:** 2025-12-06
**Status:** Complete

---

## Executive Summary

This document specifies the day-to-day operational procedures for managing the Media Gateway platform in production. It covers service management, incident response, maintenance procedures, and on-call operations.

---

## 1. Service Operations

### 1.1 Service Restart Procedures

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SERVICE RESTART PROCEDURES                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ROLLING RESTART (No Downtime):                                            │
│   ──────────────────────────────                                             │
│   Use for: Routine restarts, config updates, minor issues                  │
│                                                                              │
│   Step 1: Verify current state                                              │
│   $ kubectl get pods -l app=<service> -n production                        │
│   $ kubectl get deployment <service> -n production                         │
│                                                                              │
│   Step 2: Initiate rolling restart                                          │
│   $ kubectl rollout restart deployment/<service> -n production             │
│                                                                              │
│   Step 3: Monitor rollout                                                    │
│   $ kubectl rollout status deployment/<service> -n production              │
│                                                                              │
│   Step 4: Verify health                                                      │
│   $ kubectl get pods -l app=<service> -n production                        │
│   $ curl -s https://api.mediagateway.io/health | jq                        │
│                                                                              │
│   Expected duration: 2-5 minutes per service                                │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   HARD RESTART (Brief Downtime):                                            │
│   ──────────────────────────────                                             │
│   Use for: Stuck pods, memory issues, critical bugs                        │
│   WARNING: Causes brief service interruption                               │
│                                                                              │
│   Step 1: Scale down                                                         │
│   $ kubectl scale deployment/<service> --replicas=0 -n production          │
│                                                                              │
│   Step 2: Wait for termination                                              │
│   $ kubectl get pods -l app=<service> -n production -w                     │
│   (Wait until no pods remain)                                               │
│                                                                              │
│   Step 3: Scale up                                                           │
│   $ kubectl scale deployment/<service> --replicas=<original> -n production │
│                                                                              │
│   Step 4: Verify health                                                      │
│   $ kubectl rollout status deployment/<service> -n production              │
│                                                                              │
│   Expected duration: 1-3 minutes                                            │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   SINGLE POD RESTART:                                                        │
│   ────────────────────                                                       │
│   Use for: Debugging specific pod, isolating issues                        │
│                                                                              │
│   $ kubectl delete pod <pod-name> -n production                            │
│   (Deployment will automatically create replacement)                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Service Scaling Procedures

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SERVICE SCALING PROCEDURES                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   MANUAL SCALE UP:                                                           │
│   ─────────────────                                                          │
│   When: Traffic spike expected, HPA not responding fast enough             │
│                                                                              │
│   Step 1: Check current state                                               │
│   $ kubectl get hpa <service> -n production                                │
│   $ kubectl get deployment <service> -n production                         │
│                                                                              │
│   Step 2: Scale up                                                           │
│   $ kubectl scale deployment/<service> --replicas=<new-count> -n production│
│                                                                              │
│   Step 3: Verify scaling                                                     │
│   $ kubectl get pods -l app=<service> -n production -w                     │
│   (Wait for all pods to be Ready)                                           │
│                                                                              │
│   Step 4: Update HPA min if needed                                          │
│   $ kubectl patch hpa <service> -n production \                            │
│       -p '{"spec":{"minReplicas":<new-min>}}'                              │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   MANUAL SCALE DOWN:                                                         │
│   ──────────────────                                                         │
│   When: Traffic returned to normal, cost optimization                       │
│   WARNING: Ensure traffic levels support reduced capacity                  │
│                                                                              │
│   Step 1: Verify current traffic                                            │
│   Check Grafana: Request rate per pod                                       │
│   Ensure: < 70% CPU utilization at target replica count                    │
│                                                                              │
│   Step 2: Scale down gradually                                              │
│   $ kubectl scale deployment/<service> --replicas=<new-count> -n production│
│                                                                              │
│   Step 3: Monitor after scaling                                             │
│   Watch for 15 minutes:                                                     │
│   • Error rate stable                                                       │
│   • Latency stable                                                          │
│   • CPU < 80%                                                               │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   Service Scaling Limits:                                                    │
│   ───────────────────────                                                    │
│   Service              Min    Max    Scale Trigger                          │
│   ─────────────────────────────────────────────────                         │
│   API Gateway          3      20     CPU 70%, RPS 1000/pod                 │
│   Auth Service         3      10     CPU 70%, RPS 200/pod                  │
│   Content Service      3      15     CPU 70%, RPS 500/pod                  │
│   Search Service       2      10     CPU 70%, Latency 300ms                │
│   SONA Engine          2      8      CPU 60%, Latency 3ms                  │
│   Sync Service         3      12     Connections 5000/pod                  │
│   MCP Server           2      10     CPU 70%, RPS 100/pod                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Configuration Updates

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CONFIGURATION UPDATE PROCEDURES                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   CONFIGMAP UPDATE:                                                          │
│   ─────────────────                                                          │
│   Use for: Non-sensitive configuration changes                              │
│                                                                              │
│   Step 1: Edit ConfigMap                                                    │
│   $ kubectl edit configmap <service>-config -n production                  │
│   OR                                                                         │
│   $ kubectl apply -f configs/<service>-config.yaml                         │
│                                                                              │
│   Step 2: Trigger pod restart (if not using auto-reload)                   │
│   $ kubectl rollout restart deployment/<service> -n production             │
│                                                                              │
│   Step 3: Verify configuration                                              │
│   $ kubectl exec -it <pod> -n production -- env | grep <key>               │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   SECRET UPDATE:                                                             │
│   ──────────────                                                             │
│   Use for: API keys, credentials, certificates                             │
│                                                                              │
│   Step 1: Update in Secret Manager                                          │
│   $ gcloud secrets versions add <secret-name> \                            │
│       --data-file=<secret-file>                                             │
│                                                                              │
│   Step 2: Trigger External Secrets refresh                                  │
│   $ kubectl annotate externalsecret <secret> \                             │
│       force-sync=$(date +%s) -n production                                  │
│                                                                              │
│   Step 3: Restart pods to pick up new secret                               │
│   $ kubectl rollout restart deployment/<service> -n production             │
│                                                                              │
│   Step 4: Verify secret                                                      │
│   $ kubectl exec -it <pod> -n production -- \                              │
│       cat /secrets/<secret-file>                                            │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   FEATURE FLAG UPDATE:                                                       │
│   ─────────────────────                                                      │
│   Use for: Feature toggles, A/B tests, gradual rollouts                    │
│                                                                              │
│   Step 1: Update in LaunchDarkly/Flagsmith                                 │
│   (Use dashboard or API)                                                    │
│                                                                              │
│   Step 2: Verify propagation                                                │
│   $ curl -s https://api.mediagateway.io/internal/flags | jq .<flag>       │
│                                                                              │
│   No pod restart required (real-time update)                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Database Operations

### 2.1 PostgreSQL Operations

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    POSTGRESQL OPERATIONS                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   CONNECTION TO DATABASE:                                                    │
│   ───────────────────────                                                    │
│   Via Cloud SQL Proxy (recommended):                                        │
│   $ cloud_sql_proxy -instances=<project>:us-central1:media-gateway-db=tcp:5432 &
│   $ psql -h localhost -U admin -d media_gateway                            │
│                                                                              │
│   Via kubectl port-forward:                                                 │
│   $ kubectl port-forward svc/postgres 5432:5432 -n production              │
│   $ psql -h localhost -U admin -d media_gateway                            │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   QUERY PERFORMANCE INVESTIGATION:                                          │
│   ─────────────────────────────────                                          │
│   Find slow queries:                                                         │
│   SELECT query, calls, mean_time, total_time                               │
│   FROM pg_stat_statements                                                   │
│   ORDER BY mean_time DESC LIMIT 10;                                         │
│                                                                              │
│   Check for blocking queries:                                               │
│   SELECT blocked_locks.pid AS blocked_pid,                                 │
│          blocking_locks.pid AS blocking_pid,                               │
│          blocked_activity.query AS blocked_query                           │
│   FROM pg_locks blocked_locks                                               │
│   JOIN pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
│   JOIN pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
│   WHERE NOT blocked_locks.granted;                                          │
│                                                                              │
│   Check connection usage:                                                    │
│   SELECT count(*), state FROM pg_stat_activity GROUP BY state;             │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   KILL RUNAWAY QUERY:                                                        │
│   ────────────────────                                                       │
│   Find the process:                                                          │
│   SELECT pid, now() - pg_stat_activity.query_start AS duration, query      │
│   FROM pg_stat_activity                                                     │
│   WHERE state != 'idle' ORDER BY duration DESC;                            │
│                                                                              │
│   Cancel gracefully:                                                         │
│   SELECT pg_cancel_backend(<pid>);                                          │
│                                                                              │
│   Force terminate (if cancel fails):                                        │
│   SELECT pg_terminate_backend(<pid>);                                       │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   FAILOVER TO READ REPLICA:                                                  │
│   ─────────────────────────                                                  │
│   Step 1: Verify replica is in sync                                         │
│   $ gcloud sql instances describe media-gateway-db-replica                 │
│   Check: replicaConfiguration.failoverTarget: true                         │
│                                                                              │
│   Step 2: Promote replica (DESTRUCTIVE - breaks replication)               │
│   $ gcloud sql instances promote-replica media-gateway-db-replica          │
│                                                                              │
│   Step 3: Update connection strings                                         │
│   (Update secrets to point to new primary)                                 │
│                                                                              │
│   Step 4: Restart services                                                  │
│   $ kubectl rollout restart deployment --all -n production                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Redis Operations

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    REDIS OPERATIONS                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   CONNECTION TO REDIS:                                                       │
│   ────────────────────                                                       │
│   Via kubectl exec:                                                          │
│   $ kubectl exec -it <app-pod> -n production -- redis-cli -h redis         │
│                                                                              │
│   Via port-forward:                                                          │
│   $ kubectl port-forward svc/redis 6379:6379 -n production                 │
│   $ redis-cli -h localhost                                                  │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   CACHE INSPECTION:                                                          │
│   ─────────────────                                                          │
│   Check memory usage:                                                        │
│   > INFO memory                                                              │
│                                                                              │
│   Check key count:                                                           │
│   > DBSIZE                                                                   │
│                                                                              │
│   Find large keys:                                                           │
│   > redis-cli --bigkeys                                                     │
│                                                                              │
│   Check hit ratio:                                                           │
│   > INFO stats | grep keyspace                                              │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   CACHE FLUSH PROCEDURES:                                                    │
│   ────────────────────────                                                   │
│   Flush specific key pattern:                                               │
│   > SCAN 0 MATCH "content:*" COUNT 1000                                    │
│   (Then DEL each key)                                                       │
│                                                                              │
│   Flush single database:                                                     │
│   > FLUSHDB                                                                  │
│   WARNING: Removes all keys in current database                            │
│                                                                              │
│   Flush all databases:                                                       │
│   > FLUSHALL                                                                 │
│   WARNING: Removes ALL keys - use with extreme caution                     │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   RATE LIMITER RESET:                                                        │
│   ────────────────────                                                       │
│   Reset rate limit for specific user:                                       │
│   > DEL rate_limit:user:<user_id>                                          │
│                                                                              │
│   Reset rate limit for IP:                                                  │
│   > DEL rate_limit:ip:<ip_address>                                         │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   SESSION MANAGEMENT:                                                        │
│   ────────────────────                                                       │
│   Find user sessions:                                                        │
│   > KEYS session:user:<user_id>:*                                          │
│                                                                              │
│   Invalidate all sessions for user:                                        │
│   > DEL session:user:<user_id>:*                                           │
│                                                                              │
│   Check session TTL:                                                         │
│   > TTL session:<session_id>                                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Qdrant Operations

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    QDRANT OPERATIONS                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   CONNECTION TO QDRANT:                                                      │
│   ─────────────────────                                                      │
│   Via port-forward:                                                          │
│   $ kubectl port-forward svc/qdrant 6333:6333 -n production                │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   COLLECTION STATUS:                                                         │
│   ──────────────────                                                         │
│   List collections:                                                          │
│   $ curl -s http://localhost:6333/collections | jq                         │
│                                                                              │
│   Get collection info:                                                       │
│   $ curl -s http://localhost:6333/collections/content_embeddings | jq      │
│                                                                              │
│   Check cluster status:                                                      │
│   $ curl -s http://localhost:6333/cluster | jq                             │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   INDEX REBUILD:                                                             │
│   ──────────────                                                             │
│   Trigger HNSW index optimization:                                          │
│   $ curl -X POST http://localhost:6333/collections/content_embeddings \    │
│       -H "Content-Type: application/json" \                                 │
│       -d '{"optimizers_config": {"indexing_threshold": 0}}'                │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   SNAPSHOT MANAGEMENT:                                                       │
│   ─────────────────────                                                      │
│   Create snapshot:                                                           │
│   $ curl -X POST http://localhost:6333/collections/content_embeddings/snapshots
│                                                                              │
│   List snapshots:                                                            │
│   $ curl http://localhost:6333/collections/content_embeddings/snapshots    │
│                                                                              │
│   Restore from snapshot:                                                     │
│   $ curl -X PUT http://localhost:6333/collections/content_embeddings/snapshots/recover \
│       -H "Content-Type: application/json" \                                 │
│       -d '{"location": "<snapshot_name>"}'                                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Incident Response

### 3.1 Incident Severity Levels

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    INCIDENT SEVERITY LEVELS                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   P1 - CRITICAL                                                              │
│   ─────────────                                                              │
│   Definition:                                                                │
│   • Complete service outage                                                 │
│   • Data breach or security incident                                        │
│   • >50% of users affected                                                  │
│   • Critical business function unavailable                                  │
│                                                                              │
│   Response:                                                                  │
│   • Response time: 15 minutes                                               │
│   • Resolution target: 1 hour                                               │
│   • On-call: Immediately page                                               │
│   • Escalation: VP notified within 30 min                                  │
│   • Communication: Status page updated every 15 min                        │
│                                                                              │
│   Examples:                                                                  │
│   • API Gateway completely down                                             │
│   • Database corrupted                                                      │
│   • Authentication failing for all users                                    │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   P2 - HIGH                                                                  │
│   ────────                                                                   │
│   Definition:                                                                │
│   • Major feature unavailable                                               │
│   • 10-50% of users affected                                               │
│   • Significant performance degradation                                    │
│   • Workaround available but painful                                       │
│                                                                              │
│   Response:                                                                  │
│   • Response time: 30 minutes                                               │
│   • Resolution target: 4 hours                                              │
│   • On-call: Page during business hours, alert after hours                 │
│   • Escalation: Tech lead notified within 1 hour                           │
│   • Communication: Status page updated every 30 min                        │
│                                                                              │
│   Examples:                                                                  │
│   • Search returning incorrect results                                      │
│   • Cross-device sync delayed by minutes                                   │
│   • Recommendations not loading                                            │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   P3 - MEDIUM                                                                │
│   ──────────                                                                 │
│   Definition:                                                                │
│   • Minor feature degraded                                                  │
│   • <10% of users affected                                                 │
│   • Easy workaround available                                              │
│   • Isolated issue                                                          │
│                                                                              │
│   Response:                                                                  │
│   • Response time: 4 hours                                                  │
│   • Resolution target: 24 hours                                             │
│   • On-call: Alert, no page                                                │
│   • Escalation: Team lead notified                                         │
│   • Communication: Status page if customer-facing                          │
│                                                                              │
│   Examples:                                                                  │
│   • Single platform integration slow                                        │
│   • Specific content type not loading                                      │
│   • UI glitch on specific browser                                          │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   P4 - LOW                                                                   │
│   ────────                                                                   │
│   Definition:                                                                │
│   • Minor issue                                                              │
│   • Cosmetic problem                                                        │
│   • Enhancement request                                                     │
│   • Documentation issue                                                     │
│                                                                              │
│   Response:                                                                  │
│   • Response time: Next business day                                        │
│   • Resolution target: Next sprint                                          │
│   • On-call: No action                                                      │
│   • Escalation: Add to backlog                                             │
│   • Communication: None                                                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Incident Response Workflow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    INCIDENT RESPONSE WORKFLOW                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────┐                                                            │
│   │   DETECT    │  Alert received / Customer report / Monitoring           │
│   └──────┬──────┘                                                            │
│          │                                                                   │
│          ▼                                                                   │
│   ┌─────────────┐                                                            │
│   │   TRIAGE    │  Assess severity, impact, assign owner                   │
│   └──────┬──────┘                                                            │
│          │                                                                   │
│          ▼                                                                   │
│   ┌─────────────┐                                                            │
│   │  RESPOND    │  Mitigate impact, gather information                     │
│   └──────┬──────┘                                                            │
│          │                                                                   │
│          ▼                                                                   │
│   ┌─────────────┐                                                            │
│   │  RESOLVE    │  Fix root cause, verify resolution                       │
│   └──────┬──────┘                                                            │
│          │                                                                   │
│          ▼                                                                   │
│   ┌─────────────┐                                                            │
│   │   REVIEW    │  Post-mortem, action items, documentation               │
│   └─────────────┘                                                            │
│                                                                              │
│                                                                              │
│   TRIAGE CHECKLIST:                                                          │
│   ─────────────────                                                          │
│   □ What is the customer impact?                                            │
│   □ How many users are affected?                                            │
│   □ Is there a workaround?                                                  │
│   □ When did this start?                                                    │
│   □ Were there any recent changes?                                          │
│   □ What is the severity level?                                             │
│   □ Who is the incident owner?                                              │
│                                                                              │
│   RESPOND CHECKLIST:                                                         │
│   ──────────────────                                                         │
│   □ Create incident channel: #inc-YYYYMMDD-<brief-desc>                    │
│   □ Update status page                                                      │
│   □ Notify stakeholders                                                     │
│   □ Gather logs and metrics                                                │
│   □ Identify potential causes                                               │
│   □ Apply immediate mitigation                                              │
│                                                                              │
│   RESOLVE CHECKLIST:                                                         │
│   ─────────────────                                                          │
│   □ Root cause identified                                                   │
│   □ Fix deployed                                                            │
│   □ Verification complete                                                   │
│   □ Customer impact ended                                                   │
│   □ Status page updated                                                     │
│   □ Stakeholders notified                                                   │
│                                                                              │
│   REVIEW CHECKLIST:                                                          │
│   ─────────────────                                                          │
│   □ Timeline documented                                                     │
│   □ Root cause documented                                                   │
│   □ Action items created                                                    │
│   □ Post-mortem scheduled                                                   │
│   □ Runbooks updated                                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.3 Incident Communication Templates

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    INCIDENT COMMUNICATION TEMPLATES                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   INCIDENT DECLARATION (Slack):                                              │
│   ──────────────────────────────                                             │
│   🚨 **INCIDENT DECLARED** - [Severity]                                     │
│   **Time:** [HH:MM UTC]                                                     │
│   **Description:** [Brief description]                                      │
│   **Impact:** [Customer impact]                                             │
│   **Owner:** @[name]                                                        │
│   **Channel:** #inc-YYYYMMDD-[brief-name]                                  │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   STATUS UPDATE (Every 15-30 min for P1/P2):                                │
│   ─────────────────────────────────────────                                  │
│   📊 **STATUS UPDATE** - [HH:MM UTC]                                        │
│   **Current Status:** [Investigating / Identified / Mitigating / Resolved]│
│   **What we know:** [Summary]                                              │
│   **What we're doing:** [Current actions]                                  │
│   **Next update:** [Time]                                                  │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   INCIDENT RESOLVED (Slack):                                                 │
│   ─────────────────────────                                                  │
│   ✅ **INCIDENT RESOLVED**                                                  │
│   **Time:** [HH:MM UTC]                                                     │
│   **Duration:** [X hours Y minutes]                                        │
│   **Root Cause:** [Brief description]                                       │
│   **Resolution:** [What fixed it]                                          │
│   **Post-mortem:** Scheduled for [date/time]                               │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   CUSTOMER EMAIL (for major incidents):                                      │
│   ─────────────────────────────────────                                      │
│   Subject: Media Gateway Service Update                                     │
│                                                                              │
│   Dear Customer,                                                             │
│                                                                              │
│   We experienced an issue with [service] between [time] and [time] UTC.    │
│                                                                              │
│   During this time, you may have experienced [impact].                      │
│                                                                              │
│   The issue has been resolved and service has been restored.               │
│                                                                              │
│   We apologize for any inconvenience this may have caused. We are          │
│   conducting a thorough review to prevent similar issues in the future.   │
│                                                                              │
│   If you have any questions, please contact support@mediagateway.io.       │
│                                                                              │
│   Sincerely,                                                                 │
│   The Media Gateway Team                                                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. On-Call Operations

### 4.1 On-Call Rotation

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ON-CALL ROTATION STRUCTURE                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Primary On-Call:                                                           │
│   ────────────────                                                           │
│   • Rotation: Weekly (Monday 09:00 to Monday 09:00)                        │
│   • Responsibility: First responder to all alerts                          │
│   • Response time: 15 minutes for P1, 30 minutes for P2                   │
│   • Availability: 24/7 during rotation                                     │
│                                                                              │
│   Secondary On-Call:                                                         │
│   ──────────────────                                                         │
│   • Rotation: Same week as primary                                          │
│   • Responsibility: Backup if primary unavailable                          │
│   • Escalation: After 15 minutes of no primary response                    │
│   • SME support: Provide domain expertise                                  │
│                                                                              │
│   On-Call Schedule:                                                          │
│   ─────────────────                                                          │
│   Week 1: Engineer A (Primary), Engineer B (Secondary)                     │
│   Week 2: Engineer B (Primary), Engineer C (Secondary)                     │
│   Week 3: Engineer C (Primary), Engineer D (Secondary)                     │
│   Week 4: Engineer D (Primary), Engineer A (Secondary)                     │
│   (Rotate through team of 4-6 engineers)                                   │
│                                                                              │
│   On-Call Requirements:                                                      │
│   ─────────────────────                                                      │
│   □ Laptop with VPN access available                                       │
│   □ Phone with PagerDuty app installed                                     │
│   □ Internet connectivity                                                  │
│   □ Within 15 minutes of a workspace                                       │
│   □ Not traveling without backup arranged                                  │
│   □ Completed on-call training                                             │
│                                                                              │
│   Handoff Procedure:                                                         │
│   ──────────────────                                                         │
│   Monday 09:00:                                                              │
│   1. Outgoing on-call sends summary of week's issues                       │
│   2. Incoming on-call acknowledges receipt                                 │
│   3. Review any ongoing issues                                             │
│   4. Verify access to all systems                                          │
│   5. Test paging mechanism                                                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 On-Call Runbook

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ON-CALL RESPONSE RUNBOOK                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   WHEN PAGED:                                                                │
│   ───────────                                                                │
│   1. Acknowledge the page in PagerDuty                                     │
│   2. Open laptop and connect to VPN                                        │
│   3. Review the alert details                                              │
│   4. Check #on-call Slack channel                                          │
│   5. Open relevant dashboards                                              │
│                                                                              │
│   FIRST 5 MINUTES:                                                           │
│   ────────────────                                                           │
│   □ Understand the alert                                                    │
│   □ Check service health dashboard                                         │
│   □ Verify alert is not a false positive                                   │
│   □ Assess customer impact                                                 │
│   □ Determine severity level                                               │
│                                                                              │
│   IF REAL INCIDENT:                                                          │
│   ─────────────────                                                          │
│   □ Declare incident in #on-call                                           │
│   □ Create incident channel                                                │
│   □ Update status page if customer-facing                                  │
│   □ Start investigation                                                    │
│   □ Apply known mitigations from runbook                                   │
│   □ Escalate if needed                                                     │
│                                                                              │
│   COMMON ISSUES & FIRST ACTIONS:                                            │
│   ──────────────────────────────                                             │
│                                                                              │
│   High Error Rate:                                                          │
│   → Check logs for errors: kubectl logs -l app=<service>                   │
│   → Check recent deployments: argocd app history <app>                     │
│   → Check external dependencies status                                     │
│   → Rollback if recent deployment: kubectl rollout undo                    │
│                                                                              │
│   High Latency:                                                              │
│   → Check CPU/Memory: kubectl top pods                                     │
│   → Check database connections                                             │
│   → Check cache hit rate                                                   │
│   → Scale up if resource constrained                                       │
│                                                                              │
│   Service Down:                                                              │
│   → Check pod status: kubectl get pods                                     │
│   → Check pod events: kubectl describe pod <pod>                          │
│   → Check node status: kubectl get nodes                                   │
│   → Restart if stuck: kubectl rollout restart                             │
│                                                                              │
│   Database Issues:                                                           │
│   → Check connections: SELECT count(*) FROM pg_stat_activity;             │
│   → Check replication lag                                                  │
│   → Kill long-running queries                                              │
│   → Failover to replica if primary unhealthy                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.3 Escalation Procedures

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ESCALATION PROCEDURES                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   WHEN TO ESCALATE:                                                          │
│   ─────────────────                                                          │
│   • Issue persists >30 minutes                                              │
│   • You're stuck and need help                                              │
│   • Customer impact is severe                                               │
│   • Security incident suspected                                             │
│   • Data loss possible                                                      │
│   • Expertise needed you don't have                                        │
│                                                                              │
│   ESCALATION PATH:                                                           │
│   ────────────────                                                           │
│                                                                              │
│   Level 1: Secondary On-Call                                                │
│   ────────────────────────────                                               │
│   Method: PagerDuty escalation policy                                       │
│   When: Primary needs support or is unavailable                            │
│   Contact: Automatic after 15 min no-ack or manual escalation             │
│                                                                              │
│   Level 2: Tech Lead / SME                                                  │
│   ─────────────────────────                                                  │
│   Method: Direct Slack DM or phone                                         │
│   When: Issue requires architectural knowledge                             │
│   Contacts:                                                                  │
│   • Backend: [name] - [phone]                                              │
│   • Platform: [name] - [phone]                                             │
│   • Database: [name] - [phone]                                             │
│   • Security: [name] - [phone]                                             │
│                                                                              │
│   Level 3: Engineering Manager                                               │
│   ───────────────────────────                                                │
│   Method: Phone call                                                         │
│   When: Issue >1 hour, team coordination needed                            │
│   Contact: [name] - [phone]                                                │
│                                                                              │
│   Level 4: VP Engineering                                                    │
│   ────────────────────────                                                   │
│   Method: Phone call                                                         │
│   When: P1 >30 min, major customer impact, security breach                │
│   Contact: [name] - [phone]                                                │
│                                                                              │
│   Level 5: Executive                                                         │
│   ─────────────────                                                          │
│   Method: Phone call                                                         │
│   When: Company-wide impact, data breach, extended outage                  │
│   Contacts:                                                                  │
│   • CTO: [name] - [phone]                                                  │
│   • CEO: [name] - [phone]                                                  │
│                                                                              │
│   EXTERNAL ESCALATION:                                                       │
│   ────────────────────                                                       │
│   GCP Support: support.google.com (Premium support)                        │
│   PubNub Support: support.pubnub.com                                       │
│   Spotify Support: developer.spotify.com/support                           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 5. Log Investigation Procedures

### 5.1 Log Access and Search

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    LOG INVESTIGATION PROCEDURES                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   LOG ACCESS:                                                                │
│   ───────────                                                                │
│   Cloud Logging Console:                                                     │
│   https://console.cloud.google.com/logs/query                              │
│                                                                              │
│   Via kubectl:                                                               │
│   $ kubectl logs -l app=<service> -n production --tail=100 -f              │
│                                                                              │
│   Via gcloud:                                                                │
│   $ gcloud logging read \                                                   │
│       'resource.type="k8s_container" AND resource.labels.namespace_name="production"'
│       --limit=100 --format=json                                             │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   COMMON LOG QUERIES:                                                        │
│   ────────────────────                                                       │
│                                                                              │
│   Find errors in last hour:                                                 │
│   resource.type="k8s_container"                                             │
│   resource.labels.namespace_name="production"                               │
│   severity>=ERROR                                                           │
│   timestamp>="2024-12-06T08:00:00Z"                                        │
│                                                                              │
│   Find by trace ID:                                                          │
│   trace="projects/media-gateway/traces/<trace-id>"                         │
│                                                                              │
│   Find by user ID:                                                           │
│   jsonPayload.userId="<user-id>"                                           │
│                                                                              │
│   Find by request ID:                                                        │
│   jsonPayload.requestId="<request-id>"                                     │
│                                                                              │
│   Find 5xx errors:                                                           │
│   jsonPayload.status>=500                                                   │
│                                                                              │
│   Find slow requests:                                                        │
│   jsonPayload.latencyMs>1000                                               │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   LOG FIELDS:                                                                │
│   ───────────                                                                │
│   Standard fields:                                                           │
│   • timestamp - When the log was written                                   │
│   • severity - DEBUG, INFO, WARNING, ERROR, CRITICAL                       │
│   • resource.labels.container_name - Service name                          │
│   • resource.labels.pod_name - Pod name                                    │
│                                                                              │
│   Application fields (in jsonPayload):                                      │
│   • requestId - Unique request identifier                                  │
│   • userId - Authenticated user ID                                         │
│   • traceId - Distributed trace ID                                         │
│   • latencyMs - Request duration                                           │
│   • status - HTTP status code                                              │
│   • method - HTTP method                                                   │
│   • path - Request path                                                    │
│   • error - Error message (if any)                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Troubleshooting Workflows

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TROUBLESHOOTING WORKFLOWS                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   HIGH ERROR RATE INVESTIGATION:                                            │
│   ──────────────────────────────                                             │
│   1. Check which endpoints are erroring                                     │
│      Query: severity>=ERROR | groupBy path | count                         │
│                                                                              │
│   2. Check error types                                                       │
│      Query: severity>=ERROR | groupBy jsonPayload.error | count            │
│                                                                              │
│   3. Check if related to specific users                                     │
│      Query: severity>=ERROR | groupBy jsonPayload.userId | count           │
│                                                                              │
│   4. Check downstream dependencies                                          │
│      Query: jsonPayload.downstream="true" AND severity>=ERROR              │
│                                                                              │
│   5. Compare with recent deployments                                        │
│      $ argocd app history <app> | head -5                                  │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   HIGH LATENCY INVESTIGATION:                                               │
│   ────────────────────────────                                               │
│   1. Find slow requests                                                      │
│      Query: jsonPayload.latencyMs>1000 | orderBy latencyMs desc            │
│                                                                              │
│   2. Check database query times                                             │
│      Query: jsonPayload.dbLatencyMs>100                                    │
│                                                                              │
│   3. Check external API times                                               │
│      Query: jsonPayload.externalApiMs>500                                  │
│                                                                              │
│   4. Check cache performance                                                │
│      Look at Redis metrics in Grafana                                      │
│                                                                              │
│   5. Get trace for slow request                                             │
│      Copy traceId and view in Cloud Trace                                  │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   USER-REPORTED ISSUE INVESTIGATION:                                        │
│   ────────────────────────────────                                           │
│   1. Get user ID and approximate time                                      │
│                                                                              │
│   2. Search logs by user ID                                                 │
│      Query: jsonPayload.userId="<id>" AND timestamp>="<time>"              │
│                                                                              │
│   3. Find errors for that user                                              │
│      Query: jsonPayload.userId="<id>" AND severity>=ERROR                 │
│                                                                              │
│   4. Trace the user's requests                                              │
│      Get requestId, then: trace="<traceId>"                                │
│                                                                              │
│   5. Check user's data                                                      │
│      SELECT * FROM users WHERE id = '<id>';                                │
│      SELECT * FROM user_sessions WHERE user_id = '<id>';                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Summary

This Operational Procedures document provides:

✅ **Service Operations** - Restart, scaling, and configuration procedures
✅ **Database Operations** - PostgreSQL, Redis, and Qdrant management
✅ **Incident Response** - Severity levels, workflow, and communication
✅ **On-Call Operations** - Rotation, runbook, and escalation
✅ **Log Investigation** - Queries and troubleshooting workflows

**Next Document**: SPARC_COMPLETION_PART_4C.md - Disaster Recovery Procedures

---

**Document Status:** Complete
**Related Documents**:
- SPARC_COMPLETION_PART_4A.md (Launch Day Runbook)
- SPARC_ARCHITECTURE_INFRASTRUCTURE.md (GCP Infrastructure)
- SPARC_ARCHITECTURE_PART_4.md (Deployment Architecture)

---

END OF OPERATIONAL PROCEDURES
