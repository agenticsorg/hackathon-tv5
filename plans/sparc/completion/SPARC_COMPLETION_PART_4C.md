# SPARC Completion Phase - Part 4C: Disaster Recovery Procedures

**Version:** 1.0.0
**Phase:** SPARC Completion (Phase 5)
**Date:** 2025-12-06
**Status:** Complete

---

## Executive Summary

This document specifies the disaster recovery (DR) procedures for the Media Gateway platform. It defines RTO/RPO targets, backup strategies, failover procedures, and recovery validation tests to ensure business continuity in the event of a disaster.

---

## 1. Disaster Recovery Overview

### 1.1 DR Strategy Summary

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    DISASTER RECOVERY STRATEGY                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Recovery Objectives:                                                       │
│   ────────────────────                                                       │
│   • RTO (Recovery Time Objective): 30 minutes                              │
│   • RPO (Recovery Point Objective): 5 minutes                              │
│   • Target Availability: 99.9%                                              │
│                                                                              │
│   DR Architecture:                                                           │
│   ─────────────────                                                          │
│   Primary Region: us-central1 (Iowa)                                        │
│   DR Region: us-east1 (South Carolina)                                      │
│   Replication: Asynchronous (PostgreSQL, Qdrant)                           │
│   DNS Failover: Cloud DNS with health checks                               │
│                                                                              │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                        NORMAL OPERATIONS                          │    │
│   ├───────────────────────────────────────────────────────────────────┤    │
│   │                                                                   │    │
│   │   us-central1 (PRIMARY)              us-east1 (DR)               │    │
│   │   ┌─────────────────────┐           ┌─────────────────────┐      │    │
│   │   │   GKE Autopilot     │           │   GKE Standby       │      │    │
│   │   │   (Active)          │           │   (Warm Standby)    │      │    │
│   │   └─────────────────────┘           └─────────────────────┘      │    │
│   │   ┌─────────────────────┐           ┌─────────────────────┐      │    │
│   │   │   Cloud SQL         │──Async──▶│   Cloud SQL         │      │    │
│   │   │   (Primary)         │  Replica  │   (Read Replica)    │      │    │
│   │   └─────────────────────┘           └─────────────────────┘      │    │
│   │   ┌─────────────────────┐           ┌─────────────────────┐      │    │
│   │   │   Redis HA          │           │   Redis (Cold)      │      │    │
│   │   │   (Active)          │           │   (Not provisioned) │      │    │
│   │   └─────────────────────┘           └─────────────────────┘      │    │
│   │   ┌─────────────────────┐           ┌─────────────────────┐      │    │
│   │   │   Qdrant             │──Async──▶│   Qdrant            │      │    │
│   │   │   (Primary)         │  Backup   │   (Standby)         │      │    │
│   │   └─────────────────────┘           └─────────────────────┘      │    │
│   │                                                                   │    │
│   │   Traffic: 100% ─────────▶ us-central1                          │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Disaster Scenarios

| Scenario | RTO | RPO | Procedure |
|----------|-----|-----|-----------|
| Single pod failure | 30s | 0 | Automatic (K8s) |
| Single node failure | 2 min | 0 | Automatic (GKE) |
| Single zone failure | 5 min | 0 | Automatic (Multi-zone) |
| Database primary failure | 5 min | <1 min | Automatic failover |
| Redis failure | 2 min | Cache only | Automatic (HA) |
| Regional outage | 30 min | <5 min | Manual failover |
| Data corruption | 1 hr | <5 min | PITR restore |
| Ransomware/Security | 4 hr | <5 min | Clean restore |

---

## 2. Backup Procedures

### 2.1 PostgreSQL Backup Strategy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    POSTGRESQL BACKUP STRATEGY                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Automated Backups (Cloud SQL):                                            │
│   ──────────────────────────────                                             │
│   Type: Full backup                                                          │
│   Frequency: Daily at 03:00 UTC                                             │
│   Retention: 7 days                                                          │
│   Location: Multi-region (us)                                               │
│   Encryption: Google-managed key                                            │
│                                                                              │
│   Point-in-Time Recovery:                                                    │
│   ─────────────────────────                                                  │
│   Type: Transaction log archival                                            │
│   Frequency: Continuous                                                      │
│   Retention: 7 days                                                          │
│   RPO: <5 minutes                                                           │
│                                                                              │
│   Cross-Region Replication:                                                  │
│   ─────────────────────────                                                  │
│   Type: Async read replica                                                  │
│   Target: us-east1                                                          │
│   Lag: <1 minute typically                                                  │
│   Promotable: Yes (for DR)                                                  │
│                                                                              │
│   Manual Backup (Before Major Changes):                                     │
│   ──────────────────────────────────────                                     │
│   Trigger: Before schema migrations, major deployments                      │
│   Command:                                                                   │
│   $ gcloud sql backups create \                                             │
│       --instance=media-gateway-db \                                         │
│       --description="Pre-migration backup $(date +%Y%m%d)"                 │
│                                                                              │
│   Backup Verification:                                                       │
│   ────────────────────                                                       │
│   Frequency: Monthly                                                         │
│   Process: Restore to test instance, run validation queries                │
│   Documentation: Record restore time and data integrity check              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Redis Backup Strategy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    REDIS BACKUP STRATEGY                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   NOTE: Redis is primarily used for caching. Data can be reconstructed    │
│   from PostgreSQL. Full backup is optional but recommended.                │
│                                                                              │
│   RDB Snapshots (Memorystore):                                              │
│   ────────────────────────────                                               │
│   Type: Point-in-time snapshot                                              │
│   Frequency: Daily                                                           │
│   Retention: 7 days                                                          │
│   Location: Same region                                                      │
│                                                                              │
│   Export Procedure:                                                          │
│   ─────────────────                                                          │
│   $ gcloud redis instances export \                                         │
│       gs://media-gateway-backups/redis/$(date +%Y%m%d).rdb \               │
│       --instance=media-gateway-redis \                                      │
│       --region=us-central1                                                  │
│                                                                              │
│   Recovery Priority:                                                         │
│   ──────────────────                                                         │
│   • Sessions: Rebuild from PostgreSQL session table                        │
│   • Cache: Let application repopulate on demand                            │
│   • Rate limits: Reset (acceptable for DR scenario)                        │
│                                                                              │
│   Cache Warming (Post-Recovery):                                            │
│   ───────────────────────────────                                            │
│   $ ./scripts/cache-warm.sh --popular-content --user-sessions             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Qdrant Backup Strategy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    QDRANT BACKUP STRATEGY                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Collection Snapshots:                                                      │
│   ─────────────────────                                                      │
│   Type: Full collection snapshot                                            │
│   Frequency: Daily at 04:00 UTC                                             │
│   Retention: 7 days                                                          │
│   Location: Cloud Storage (multi-region)                                    │
│                                                                              │
│   Backup Command:                                                            │
│   ───────────────                                                            │
│   # Create snapshot for each collection                                     │
│   for collection in content_embeddings user_preferences; do                │
│     curl -X POST "http://qdrant:6333/collections/$collection/snapshots"    │
│   done                                                                       │
│                                                                              │
│   # Copy to Cloud Storage                                                   │
│   gsutil -m cp -r /qdrant/snapshots/* \                                    │
│       gs://media-gateway-backups/qdrant/$(date +%Y%m%d)/                   │
│                                                                              │
│   Cross-Region Replication:                                                  │
│   ─────────────────────────                                                  │
│   Method: Scheduled snapshot copy to DR region                             │
│   Frequency: Every 4 hours                                                  │
│   Destination: gs://media-gateway-backups-dr/qdrant/                       │
│                                                                              │
│   Recovery Time:                                                             │
│   ──────────────                                                             │
│   Snapshot restore: ~10 minutes for 500K vectors                           │
│   Index rebuild: ~20 minutes (HNSW optimization)                           │
│   Total: ~30 minutes                                                        │
│                                                                              │
│   Alternative: Regeneration                                                  │
│   ────────────────────────                                                   │
│   If backup unavailable, regenerate embeddings from PostgreSQL content     │
│   Time: ~2-4 hours for full regeneration                                   │
│   Script: ./scripts/regenerate-embeddings.sh                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.4 Application State Backup

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    APPLICATION STATE BACKUP                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Kubernetes Manifests:                                                      │
│   ─────────────────────                                                      │
│   Location: Git repository (GitOps)                                        │
│   Recovery: Re-apply from ArgoCD                                           │
│                                                                              │
│   Secrets:                                                                   │
│   ────────                                                                   │
│   Location: Google Secret Manager                                           │
│   Replication: Automatic (global)                                          │
│   Recovery: Access from any region                                         │
│                                                                              │
│   ConfigMaps:                                                                │
│   ───────────                                                                │
│   Location: Git repository                                                  │
│   Recovery: Re-apply from ArgoCD                                           │
│                                                                              │
│   SSL Certificates:                                                          │
│   ─────────────────                                                          │
│   Type: Google-managed                                                      │
│   Recovery: Automatic provisioning                                         │
│                                                                              │
│   Feature Flags:                                                             │
│   ──────────────                                                             │
│   Location: LaunchDarkly (SaaS)                                            │
│   Recovery: Automatic (managed service)                                    │
│                                                                              │
│   DNS Configuration:                                                         │
│   ──────────────────                                                         │
│   Location: Cloud DNS                                                       │
│   Recovery: Export and re-import zone file                                 │
│   Backup command:                                                            │
│   $ gcloud dns record-sets export zone.yaml \                              │
│       --zone=mediagateway-zone                                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Regional Failover Procedures

### 3.1 Failover Decision Matrix

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    FAILOVER DECISION MATRIX                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   AUTOMATIC FAILOVER (No Human Decision):                                   │
│   ───────────────────────────────────────                                    │
│   • Pod crash → Kubernetes restarts pod                                    │
│   • Node failure → GKE reschedules pods                                    │
│   • Zone failure → Traffic routes to healthy zones                         │
│   • Database primary failure → Cloud SQL automatic failover               │
│   • Redis primary failure → Memorystore automatic failover                │
│                                                                              │
│   MANUAL FAILOVER (Human Decision Required):                                │
│   ──────────────────────────────────────────                                 │
│   • Complete regional outage (us-central1)                                 │
│   • Extended regional degradation (>30 min)                                │
│   • Data corruption requiring restore                                      │
│   • Security incident requiring isolation                                  │
│                                                                              │
│   Decision Authority:                                                        │
│   ───────────────────                                                        │
│   • During business hours: Engineering Lead + VP Engineering              │
│   • After hours: On-call Engineer + Engineering Manager                   │
│   • Emergency: Any senior engineer (document immediately)                  │
│                                                                              │
│   Decision Criteria:                                                         │
│   ──────────────────                                                         │
│   □ Primary region health checks failing >5 minutes                        │
│   □ GCP status page confirms regional issue                               │
│   □ No ETA from GCP or ETA >30 minutes                                    │
│   □ Customer impact is significant                                        │
│   □ Failover cost/risk acceptable                                          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Regional Failover Procedure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    REGIONAL FAILOVER PROCEDURE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   PHASE 1: DECISION & COMMUNICATION (5 minutes)                             │
│   ──────────────────────────────────────────────                             │
│                                                                              │
│   Step 1.1: Confirm regional issue                                          │
│   □ Check GCP status: https://status.cloud.google.com                      │
│   □ Verify health checks failing in us-central1                           │
│   □ Confirm DR region (us-east1) is healthy                               │
│                                                                              │
│   Step 1.2: Declare failover                                                │
│   Post to #incident channel:                                                │
│   "🚨 REGIONAL FAILOVER INITIATED                                          │
│    Primary: us-central1 (DOWN)                                             │
│    Target: us-east1                                                        │
│    Commander: @[name]                                                       │
│    ETA: 30 minutes"                                                        │
│                                                                              │
│   Step 1.3: Update status page                                              │
│   Status: Major Outage                                                      │
│   Message: "We are experiencing a regional outage and are failing over    │
│   to our disaster recovery site. Services will be restored shortly."      │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   PHASE 2: DATABASE FAILOVER (10 minutes)                                   │
│   ────────────────────────────────────────                                   │
│                                                                              │
│   Step 2.1: Promote read replica to primary                                │
│   $ gcloud sql instances promote-replica media-gateway-db-replica          │
│                                                                              │
│   Step 2.2: Wait for promotion                                              │
│   $ gcloud sql instances describe media-gateway-db-replica | grep state   │
│   (Wait for state: RUNNABLE)                                               │
│                                                                              │
│   Step 2.3: Update connection secret                                        │
│   $ gcloud secrets versions add db-host \                                  │
│       --data-file=<(echo -n "<new-db-ip>")                                 │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   PHASE 3: APPLICATION FAILOVER (10 minutes)                                │
│   ──────────────────────────────────────────                                 │
│                                                                              │
│   Step 3.1: Scale up DR GKE cluster                                        │
│   $ gcloud container clusters resize media-gateway-dr \                    │
│       --num-nodes=3 --zone=us-east1-b                                      │
│                                                                              │
│   Step 3.2: Deploy applications                                             │
│   $ kubectl config use-context gke_media-gateway_us-east1_dr              │
│   $ argocd app sync --all                                                  │
│                                                                              │
│   Step 3.3: Wait for pods to be ready                                       │
│   $ kubectl wait --for=condition=Ready pods --all -n production            │
│                                                                              │
│   Step 3.4: Provision Redis                                                 │
│   $ gcloud redis instances create media-gateway-redis-dr \                 │
│       --size=6 --region=us-east1 --tier=standard                           │
│                                                                              │
│   Step 3.5: Restore Qdrant from snapshot                                   │
│   $ ./scripts/qdrant-restore.sh --region=us-east1 --latest                │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   PHASE 4: DNS FAILOVER (5 minutes)                                         │
│   ─────────────────────────────────                                          │
│                                                                              │
│   Step 4.1: Update DNS to point to DR                                       │
│   $ gcloud dns record-sets update api.mediagateway.io \                    │
│       --zone=mediagateway-zone \                                            │
│       --type=A \                                                            │
│       --rrdatas=<dr-lb-ip> \                                               │
│       --ttl=60                                                              │
│                                                                              │
│   Step 4.2: Verify DNS propagation                                          │
│   $ dig api.mediagateway.io                                                │
│   (Verify new IP returned)                                                  │
│                                                                              │
│   Step 4.3: Test endpoints                                                  │
│   $ curl -s https://api.mediagateway.io/health | jq                       │
│   $ ./scripts/smoke-test.sh                                                │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   PHASE 5: VERIFICATION & COMMUNICATION (5 minutes)                         │
│   ──────────────────────────────────────────────────                         │
│                                                                              │
│   Step 5.1: Verify all services healthy                                     │
│   $ kubectl get pods -n production                                         │
│   $ curl -s https://api.mediagateway.io/health                            │
│   $ ./scripts/smoke-test.sh --full                                         │
│                                                                              │
│   Step 5.2: Update status page                                              │
│   Status: Operational                                                        │
│   Message: "Services have been restored. We are running in our DR site.   │
│   Some users may experience brief delays as caches warm up."              │
│                                                                              │
│   Step 5.3: Notify stakeholders                                             │
│   Post to #incident:                                                        │
│   "✅ FAILOVER COMPLETE                                                     │
│    DR region: us-east1 (ACTIVE)                                            │
│    All services operational                                                 │
│    Monitoring continues"                                                    │
│                                                                              │
│   TOTAL TIME: ~30 minutes                                                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.3 Failback Procedure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    FAILBACK PROCEDURE                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   PREREQUISITES:                                                             │
│   ──────────────                                                             │
│   □ Primary region (us-central1) confirmed healthy                         │
│   □ Stable operations in DR for minimum 4 hours                            │
│   □ Off-peak traffic time selected                                         │
│   □ Team available for monitoring                                          │
│   □ Stakeholder approval obtained                                          │
│                                                                              │
│   PHASE 1: PREPARE PRIMARY REGION (2 hours)                                 │
│   ─────────────────────────────────────────                                  │
│                                                                              │
│   Step 1.1: Create new database in primary region                          │
│   $ gcloud sql instances create media-gateway-db-new \                     │
│       --database-version=POSTGRES_15 \                                     │
│       --region=us-central1 \                                                │
│       --tier=db-custom-2-7680 \                                            │
│       --availability-type=REGIONAL                                         │
│                                                                              │
│   Step 1.2: Set up replication from DR                                      │
│   $ gcloud sql instances patch media-gateway-db-replica \                  │
│       --master-instance-name=media-gateway-db-dr                           │
│                                                                              │
│   Step 1.3: Wait for sync                                                   │
│   Monitor replication lag until <30 seconds                                │
│                                                                              │
│   Step 1.4: Scale up primary GKE cluster                                   │
│   $ gcloud container clusters resize media-gateway-prod \                  │
│       --num-nodes=3 --region=us-central1                                   │
│                                                                              │
│   Step 1.5: Deploy applications                                             │
│   $ kubectl config use-context gke_media-gateway_us-central1_prod         │
│   $ argocd app sync --all                                                  │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   PHASE 2: TRAFFIC MIGRATION (30 minutes)                                   │
│   ────────────────────────────────────────                                   │
│                                                                              │
│   Step 2.1: Add primary to load balancer                                   │
│   Configure weighted routing: DR 90%, Primary 10%                          │
│                                                                              │
│   Step 2.2: Monitor for 15 minutes                                          │
│   Verify errors and latency are acceptable                                 │
│                                                                              │
│   Step 2.3: Increase primary traffic                                        │
│   DR 50%, Primary 50% → Wait 10 min                                        │
│   DR 10%, Primary 90% → Wait 10 min                                        │
│   DR 0%, Primary 100%                                                       │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   PHASE 3: DATABASE CUTOVER (10 minutes)                                    │
│   ──────────────────────────────────────                                     │
│                                                                              │
│   Step 3.1: Stop writes to DR database                                      │
│   (Application already pointing to primary)                                │
│                                                                              │
│   Step 3.2: Verify final sync                                               │
│   Confirm replication lag = 0                                              │
│                                                                              │
│   Step 3.3: Promote primary database                                        │
│   $ gcloud sql instances promote-replica media-gateway-db-new              │
│                                                                              │
│   Step 3.4: Update connection strings                                       │
│   Point to new primary                                                      │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   PHASE 4: CLEANUP (1 hour)                                                 │
│   ─────────────────────────                                                  │
│                                                                              │
│   Step 4.1: Scale down DR region                                            │
│   $ gcloud container clusters resize media-gateway-dr --num-nodes=0       │
│                                                                              │
│   Step 4.2: Reconfigure DR replication                                     │
│   Create new read replica in us-east1 from new primary                    │
│                                                                              │
│   Step 4.3: Update DNS TTL back to normal                                  │
│   Increase TTL from 60s to 300s                                            │
│                                                                              │
│   Step 4.4: Document lessons learned                                        │
│   Schedule post-incident review                                            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Data Recovery Procedures

### 4.1 Point-in-Time Recovery

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    POINT-IN-TIME RECOVERY (PITR)                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   USE CASE: Recover from data corruption, accidental deletion              │
│                                                                              │
│   Step 1: Identify recovery target time                                     │
│   ──────────────────────────────────────                                     │
│   Review logs to find the last known good state                            │
│   Example: Data corruption detected at 14:30, recover to 14:25             │
│                                                                              │
│   Step 2: Create restored instance                                          │
│   ─────────────────────────────────                                          │
│   $ gcloud sql instances clone media-gateway-db \                          │
│       media-gateway-db-restored \                                           │
│       --point-in-time="2024-12-06T14:25:00Z"                               │
│                                                                              │
│   Step 3: Wait for restore completion                                       │
│   ───────────────────────────────────                                        │
│   $ gcloud sql instances describe media-gateway-db-restored               │
│   (Wait for state: RUNNABLE)                                               │
│   Estimated time: 15-30 minutes depending on size                          │
│                                                                              │
│   Step 4: Verify restored data                                              │
│   ────────────────────────────                                               │
│   $ psql -h <restored-db-ip> -U admin -d media_gateway                     │
│   > SELECT count(*) FROM users;                                             │
│   > SELECT max(updated_at) FROM content;                                    │
│   Compare counts with expected values                                       │
│                                                                              │
│   Step 5: Stop production traffic                                           │
│   ─────────────────────────────                                              │
│   Display maintenance page                                                  │
│   $ kubectl scale deployment --all --replicas=0 -n production              │
│                                                                              │
│   Step 6: Switch to restored database                                       │
│   ──────────────────────────────────                                         │
│   Update secrets to point to restored instance                             │
│   $ gcloud secrets versions add db-host \                                  │
│       --data-file=<(echo -n "<restored-db-ip>")                            │
│                                                                              │
│   Step 7: Restart services                                                  │
│   ────────────────────────                                                   │
│   $ kubectl scale deployment --all --replicas=<original> -n production     │
│   $ kubectl rollout status deployment --all -n production                  │
│                                                                              │
│   Step 8: Verify and remove maintenance page                                │
│   ─────────────────────────────────────────                                  │
│   $ ./scripts/smoke-test.sh --full                                         │
│                                                                              │
│   RECOVERY TIME: 30-60 minutes                                              │
│   DATA LOSS: Up to 5 minutes (RPO)                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Full Database Restore

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    FULL DATABASE RESTORE                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   USE CASE: Complete data loss, corrupt backups require older restore      │
│                                                                              │
│   Step 1: Identify backup to restore                                        │
│   ────────────────────────────────                                           │
│   $ gcloud sql backups list --instance=media-gateway-db                    │
│                                                                              │
│   Step 2: Create new instance from backup                                   │
│   ─────────────────────────────────────                                      │
│   $ gcloud sql instances restore-backup media-gateway-db \                 │
│       --backup-id=<backup-id>                                               │
│                                                                              │
│   Or restore to new instance:                                               │
│   $ gcloud sql instances create media-gateway-db-restored \                │
│       --source-instance=media-gateway-db \                                  │
│       --source-backup=<backup-id>                                           │
│                                                                              │
│   Step 3: Verify restored data                                              │
│   ────────────────────────────                                               │
│   Connect and run validation queries                                        │
│                                                                              │
│   Step 4: Switch production to restored instance                            │
│   ─────────────────────────────────────────────                              │
│   Follow steps 5-8 from PITR procedure                                     │
│                                                                              │
│   Step 5: Regenerate derived data                                           │
│   ─────────────────────────────────                                          │
│   # Regenerate embeddings if backup is old                                 │
│   $ ./scripts/regenerate-embeddings.sh --incremental                       │
│                                                                              │
│   # Rebuild search indexes if needed                                        │
│   $ ./scripts/reindex-search.sh                                            │
│                                                                              │
│   RECOVERY TIME: 1-2 hours                                                  │
│   DATA LOSS: Since last backup (up to 24 hours)                            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.3 Selective Data Recovery

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SELECTIVE DATA RECOVERY                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   USE CASE: Recover specific table/rows without full restore               │
│                                                                              │
│   Method 1: Restore backup to temp instance                                │
│   ──────────────────────────────────────────                                 │
│   Step 1: Create temporary restore instance                                │
│   $ gcloud sql instances clone media-gateway-db \                          │
│       media-gateway-db-temp \                                               │
│       --point-in-time="<timestamp>"                                        │
│                                                                              │
│   Step 2: Export specific data                                              │
│   $ pg_dump -h <temp-db-ip> -U admin -d media_gateway \                   │
│       --table=users --data-only > users_backup.sql                         │
│                                                                              │
│   Step 3: Import to production                                              │
│   $ psql -h <prod-db-ip> -U admin -d media_gateway \                      │
│       -c "TRUNCATE users;"  # If replacing                                 │
│   $ psql -h <prod-db-ip> -U admin -d media_gateway \                      │
│       < users_backup.sql                                                    │
│                                                                              │
│   Step 4: Delete temp instance                                              │
│   $ gcloud sql instances delete media-gateway-db-temp                      │
│                                                                              │
│   ───────────────────────────────────────────────────────────────────────── │
│                                                                              │
│   Method 2: Query backup directly                                           │
│   ────────────────────────────────                                           │
│   Connect to backup instance (read-only):                                  │
│   $ psql -h <backup-ip> -U admin -d media_gateway                          │
│   > SELECT * FROM users WHERE id = 'specific-id';                          │
│                                                                              │
│   Export specific records:                                                  │
│   > \copy (SELECT * FROM users WHERE id = 'specific-id') TO 'recovery.csv' │
│                                                                              │
│   Import to production:                                                     │
│   > \copy users FROM 'recovery.csv'                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 5. DR Testing Procedures

### 5.1 DR Test Schedule

| Test Type | Frequency | Duration | Impact |
|-----------|-----------|----------|--------|
| Backup verification | Monthly | 2 hours | None |
| Database failover | Quarterly | 1 hour | None (read replica) |
| Application failover | Quarterly | 2 hours | Maintenance window |
| Full DR test | Annually | 4 hours | Scheduled downtime |
| Tabletop exercise | Semi-annually | 2 hours | None |

### 5.2 DR Test Procedure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    QUARTERLY DR TEST PROCEDURE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   PRE-TEST (1 week before):                                                 │
│   ──────────────────────────                                                 │
│   □ Schedule maintenance window                                             │
│   □ Notify stakeholders                                                    │
│   □ Update status page (scheduled maintenance)                             │
│   □ Prepare rollback plan                                                  │
│   □ Assign test team                                                       │
│                                                                              │
│   TEST EXECUTION:                                                            │
│   ───────────────                                                            │
│                                                                              │
│   Phase 1: Backup Verification (30 min)                                     │
│   □ List recent backups                                                    │
│   □ Restore latest backup to test instance                                 │
│   □ Run data validation queries                                            │
│   □ Compare row counts with production                                     │
│   □ Delete test instance                                                   │
│                                                                              │
│   Phase 2: Database Failover Test (30 min)                                  │
│   □ Verify read replica is in sync                                         │
│   □ Promote read replica to primary                                        │
│   □ Update application connection strings                                  │
│   □ Verify application connectivity                                        │
│   □ Verify data integrity                                                  │
│   □ Create new read replica                                                │
│   □ Restore original configuration                                         │
│                                                                              │
│   Phase 3: Application Failover Test (45 min)                              │
│   □ Scale up DR cluster                                                    │
│   □ Deploy applications to DR                                              │
│   □ Verify all services healthy                                            │
│   □ Run smoke tests against DR                                             │
│   □ Shift 10% traffic to DR                                               │
│   □ Monitor for 10 minutes                                                 │
│   □ Shift back to primary                                                  │
│   □ Scale down DR cluster                                                  │
│                                                                              │
│   Phase 4: Full Failover Simulation (Optional - Annual)                    │
│   □ Simulate primary region unavailable                                    │
│   □ Execute full failover procedure                                        │
│   □ Operate from DR for 30 minutes                                        │
│   □ Execute failback procedure                                             │
│                                                                              │
│   POST-TEST:                                                                 │
│   ──────────                                                                 │
│   □ Document results                                                        │
│   □ Note any issues encountered                                            │
│   □ Update runbooks if needed                                              │
│   □ Schedule follow-up for issues                                          │
│   □ Update DR test report                                                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.3 DR Test Report Template

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    DR TEST REPORT                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Test Date: _____________________                                          │
│   Test Type: □ Monthly  □ Quarterly  □ Annual                              │
│   Test Lead: _____________________                                          │
│   Participants: _____________________                                       │
│                                                                              │
│   TEST RESULTS:                                                              │
│   ─────────────                                                              │
│                                                                              │
│   Backup Verification:                                                       │
│   ├── Status: □ PASS  □ FAIL                                               │
│   ├── Restore time: _____ minutes                                          │
│   ├── Data validation: □ PASS  □ FAIL                                      │
│   └── Notes: _____________________________________                         │
│                                                                              │
│   Database Failover:                                                         │
│   ├── Status: □ PASS  □ FAIL                                               │
│   ├── Failover time: _____ minutes                                         │
│   ├── Data loss: _____ seconds                                             │
│   └── Notes: _____________________________________                         │
│                                                                              │
│   Application Failover:                                                      │
│   ├── Status: □ PASS  □ FAIL                                               │
│   ├── Failover time: _____ minutes                                         │
│   ├── Services recovered: ___/___                                          │
│   └── Notes: _____________________________________                         │
│                                                                              │
│   RTO/RPO Verification:                                                      │
│   ├── RTO Target: 30 minutes | Actual: _____ minutes                       │
│   ├── RPO Target: 5 minutes  | Actual: _____ minutes                       │
│   └── Status: □ MET  □ NOT MET                                             │
│                                                                              │
│   ISSUES IDENTIFIED:                                                         │
│   ──────────────────                                                         │
│   1. _________________________________________________                     │
│   2. _________________________________________________                     │
│   3. _________________________________________________                     │
│                                                                              │
│   ACTION ITEMS:                                                              │
│   ─────────────                                                              │
│   1. _________________________ Owner: _______ Due: _______                 │
│   2. _________________________ Owner: _______ Due: _______                 │
│   3. _________________________ Owner: _______ Due: _______                 │
│                                                                              │
│   SIGN-OFF:                                                                  │
│   ─────────                                                                  │
│   Test Lead: _________________ Date: _______                                │
│   Operations Lead: _________________ Date: _______                          │
│   Engineering Lead: _________________ Date: _______                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Summary

This Disaster Recovery Procedures document provides:

✅ **DR Strategy** - RTO/RPO targets, architecture, scenario matrix
✅ **Backup Procedures** - PostgreSQL, Redis, Qdrant, application state
✅ **Failover Procedures** - Regional failover and failback steps
✅ **Data Recovery** - PITR, full restore, selective recovery
✅ **DR Testing** - Schedule, procedures, and report templates

**Next Document**: SPARC_COMPLETION_PART_5A.md - Success Metrics Framework

---

**Document Status:** Complete
**Related Documents**:
- SPARC_COMPLETION_PART_4A.md (Launch Day Runbook)
- SPARC_COMPLETION_PART_4B.md (Operational Procedures)
- SPARC_ARCHITECTURE_INFRASTRUCTURE.md (GCP Infrastructure)

---

END OF DISASTER RECOVERY PROCEDURES
