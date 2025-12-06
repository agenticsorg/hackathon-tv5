# Quick Reference: Scaling to 400M Users
## One-Page Architecture Cheat Sheet

**Last Updated:** 2025-12-06

---

## At-a-Glance Architecture Decisions

### User Scale → Technology Mapping

| User Scale | Database | Caching | CDN | ML Inference | Monthly Cost |
|------------|----------|---------|-----|--------------|--------------|
| 0-1M | PostgreSQL | Redis single | Cloudflare Free | None | $2K |
| 1M-10M | PostgreSQL + Replicas | Redis Cluster | Cloudflare Pro | Batch (daily) | $20K |
| 10M-50M | PostgreSQL + Citus | Redis + Cassandra | CloudFront | Real-time (origin) | $80K |
| 50M-100M | **CockroachDB** | Redis + Cassandra | Cloudflare Ent | Edge + Origin | $150K |
| 100M-200M | CockroachDB Multi | Multi-tier | Custom CDN (start) | Edge Inference | $250K |
| 200M-400M+ | **CockroachDB + Cassandra** | **3-tier** | **Custom CDN** | **Optimized Edge** | **$320K** |

---

## Critical Scaling Thresholds

### When to Migrate: Database

```
PostgreSQL → CockroachDB
WHEN: 50M users OR need global consistency
WHY: Auto-sharding, multi-region, ACID guarantees
COST: +$30K/month
EFFORT: 3 months (dual-write, shadow, cutover)
```

### When to Migrate: CDN

```
Cloudflare → Custom CDN (Netflix-style)
WHEN: 200M users OR 500TB+ bandwidth/month
WHY: 50-70% cost savings at scale
COST: $500K capex, then $12K/month opex (vs $20K)
EFFORT: 6 months (ISP negotiations, deployment)
```

### When to Migrate: Architecture

```
Monolith → Microservices
WHEN: 3+ teams OR 1M users
WHY: Team independence, better scaling
COST: +$20K/month (initial complexity)
EFFORT: 4 months (gradual extraction)

Single Region → Multi-Region
WHEN: 10M users OR >20% international
WHY: Lower latency, compliance, DR
COST: +$80K/month
EFFORT: 2 months (infra + data replication)
```

---

## Technology Stack Cheat Sheet

### Tier 1: Must-Have Technologies

```yaml
compute:
  choice: Kubernetes (EKS/GKE)
  why: Industry standard, auto-scaling, portability
  alternative: ECS (AWS-only), Cloud Run (serverless)

transactional_db:
  choice: CockroachDB (at 50M+ users)
  why: Auto-sharding, strong consistency, multi-region
  alternative: Vitess (MySQL compatibility), Spanner (Google-only)

analytics_db:
  choice: Cassandra
  why: 1M+ writes/sec, linear scaling, time-series optimized
  alternative: ScyllaDB (C++ rewrite, faster)

cache:
  choice: Redis Cluster
  why: Sub-ms latency, rich data structures, pub/sub
  alternative: Memcached (simpler, faster for pure cache)

object_storage:
  choice: S3 (AWS) or GCS (Google Cloud)
  why: 11 nines durability, cheap, integrated ecosystem
  alternative: MinIO (self-hosted)

cdn:
  choice: Cloudflare (0-200M), Custom (200M+)
  why: DDoS protection, edge compute, cost at scale
  alternative: Akamai (enterprise), Fastly (real-time)
```

### Tier 2: ML/AI Technologies

```yaml
training:
  choice: PyTorch + Ray
  why: Best ecosystem, distributed training, research-to-prod
  alternative: TensorFlow (Google ecosystem)

serving:
  choice: vLLM (LLM), NVIDIA Triton (multi-framework)
  why: 3-5x throughput, continuous batching, production-ready
  alternative: TorchServe (PyTorch-only), TFServing (TF-only)

feature_store:
  choice: Feast
  why: Open source, online+offline, Kubernetes-native
  alternative: Tecton (managed, expensive)

experimentation:
  choice: MLflow + Custom A/B framework
  why: Model registry, versioning, simple
  alternative: Kubeflow (complex, full platform)
```

---

## Architecture Patterns by Use Case

### Social Network / Feed-Based App

```
Pattern: Write-heavy, eventual consistency OK

Stack:
  - CockroachDB: User profiles, follows
  - Cassandra: Posts, likes, comments (denormalized)
  - Redis: Feed cache (pre-computed)
  - Kafka: Event stream (new post → fan-out)

Feed Generation:
  - Background jobs write feeds to Redis
  - API reads pre-computed feed from cache
  - Fallback to real-time aggregation if cache miss

Cost at 100M users: $180K/month
```

### E-Commerce / Marketplace

```
Pattern: Read-heavy, strong consistency for transactions

Stack:
  - CockroachDB: Users, orders, inventory
  - Redis: Product catalog cache, shopping cart
  - Elasticsearch: Search (product discovery)
  - S3: Product images

Transactions:
  - ACID guarantees for order placement
  - Optimistic locking for inventory
  - Saga pattern for distributed transactions

Cost at 100M users: $220K/month
```

### Video Streaming / Media

```
Pattern: Bandwidth-heavy, geo-distributed

Stack:
  - CockroachDB: User accounts, subscriptions
  - Cassandra: View history, watch time
  - Custom CDN: Video delivery (Netflix Open Connect)
  - S3 + Glacier: Video storage (transcoded assets)

Video Delivery:
  - Adaptive bitrate streaming (HLS/DASH)
  - 95% of traffic from ISP-edge caches
  - Pre-positioning popular content

Cost at 100M users: $400K/month (bandwidth-dominated)
```

### Recommendation / AI-Powered

```
Pattern: ML-heavy, personalization at scale

Stack:
  - CockroachDB: User profiles, preferences
  - Cassandra: User events, ML features
  - Redis: Real-time recommendations cache
  - vLLM: LLM-based recommendations
  - Kafka: Event streaming for ML pipelines

ML Pipeline:
  - Real-time events → Kafka → Feature store
  - Batch jobs (Spark) → Model training
  - Edge inference (quantized models) → Low latency
  - A/B testing for model evaluation

Cost at 100M users: $280K/month (ML-heavy)
```

---

## Performance Targets by Scale

### Latency Budgets

```yaml
0-10M_users:
  api_p95: 200ms
  page_load: 2s
  db_query: 50ms

10M-100M_users:
  api_p95: 100ms
  page_load: 1.5s
  db_query: 20ms

100M-400M_users:
  api_p95: 50ms
  page_load: 1s
  db_query: 10ms
  ml_inference: 50ms (regional), 10ms (edge)
```

### Availability Targets

```yaml
0-10M_users:
  slo: 99.9% (8.76 hours/year)
  strategy: Single region + backups

10M-100M_users:
  slo: 99.95% (4.38 hours/year)
  strategy: Multi-region active-passive

100M-400M_users:
  slo: 99.99% (52 minutes/year)
  strategy: Multi-region active-active + auto-failover
```

---

## Cost Optimization Playbook

### Quick Wins (0-3 months, 30% savings)

1. **Reserved Instances**
   - Apply to: Databases, Kubernetes nodes
   - Savings: 30-50%
   - Effort: Low (just purchase)

2. **Spot Instances**
   - Apply to: ML training, batch jobs
   - Savings: 70-90%
   - Effort: Medium (handle interruptions)

3. **Right-Sizing**
   - Tool: AWS Compute Optimizer, GCP Recommender
   - Savings: 20-30%
   - Effort: Low (resize instances)

### Medium Wins (3-12 months, 15% additional)

4. **Database Tiering**
   - Move cold data to Glacier/Archive
   - Savings: 40% on storage
   - Effort: Medium (lifecycle policies)

5. **ML Model Compression**
   - Quantize to 4-bit/8-bit
   - Savings: 40% on inference
   - Effort: High (model accuracy testing)

6. **Multi-Cloud Arbitrage**
   - GCP for ML, AWS for storage
   - Savings: 15-25%
   - Effort: High (multi-cloud management)

### Big Wins (12-24 months, 50% additional)

7. **Custom CDN**
   - Deploy at ISP edge
   - Savings: 50-70% on CDN
   - Effort: Very high ($500K capex, ISP negotiations)

8. **Custom Silicon**
   - AWS Inferentia, Google TPU
   - Savings: 60-70% on ML inference
   - Effort: Medium (model conversion)

9. **In-House Data Centers**
   - Only at massive scale (1B+ users)
   - Savings: 50% vs. cloud
   - Effort: Very high ($10M capex, 24-month payback)

---

## One-Liners for Common Questions

**Q: When to move to microservices?**
A: When you have 3+ teams OR 1M+ users. Not before.

**Q: When to deploy multi-region?**
A: When >20% users are international OR you need <100ms global latency.

**Q: CockroachDB or Cassandra?**
A: CockroachDB for transactions (ACID), Cassandra for analytics (eventual).

**Q: Build or buy for ML infrastructure?**
A: Buy (vLLM, Triton) until 100M users, then optimize/build custom.

**Q: How much should infrastructure cost?**
A: $0.50-1.00 per user/month at 1-10M users, $0.20-0.30 at 100M+.

**Q: What's the #1 scaling bottleneck?**
A: Database writes. Solution: Sharding + caching + eventual consistency where possible.

**Q: Should we use serverless?**
A: Yes for variable workloads (Lambda/Cloud Run), no for predictable high-throughput.

**Q: How many engineers to support 100M users?**
A: 40-60 engineers (backend, frontend, ML, SRE, data engineering).

---

## Emergency Playbook

### Database is at 90% CPU

```bash
# Immediate (< 1 hour)
1. Add read replicas (route reads to replicas)
2. Enable connection pooling (PgBouncer)
3. Scale up instance size (vertical scale)

# Short-term (< 1 week)
4. Add database indices for slow queries
5. Implement query caching (Redis)
6. Optimize N+1 queries

# Long-term (< 1 month)
7. Implement database sharding
8. Migrate to CockroachDB (auto-sharding)
9. Move analytics to Cassandra
```

### CDN Costs Spiking

```bash
# Immediate (< 1 day)
1. Check for bot traffic (block bad actors)
2. Increase cache TTL (reduce origin requests)
3. Enable compression (Gzip/Brotli)

# Short-term (< 1 week)
4. Implement smarter cache invalidation
5. Add CloudFront/Cloudflare signed URLs
6. Negotiate better rates with CDN provider

# Long-term (< 6 months)
7. Deploy custom CDN at ISP edge
8. Implement P2P delivery (WebRTC)
```

### ML Inference Latency High

```bash
# Immediate (< 1 day)
1. Increase inference server instances
2. Enable request batching (static batching)
3. Add Redis cache for common requests

# Short-term (< 2 weeks)
4. Implement model quantization (8-bit)
5. Deploy vLLM (continuous batching)
6. Add edge inference (simple requests)

# Long-term (< 2 months)
7. Model pruning and compression
8. Custom silicon (AWS Inferentia)
9. Multi-tier inference strategy
```

---

## Further Reading

- **Full Analysis**: `/docs/MASSIVE_SCALE_ARCHITECTURE_ANALYSIS.md`
- **Technology Decisions**: `/docs/TECHNOLOGY_DECISION_MATRIX.md`
- **Implementation Plan**: `/docs/IMPLEMENTATION_ROADMAP.md`

---

**Print this page and keep it handy for architecture discussions!**

---

**Version:** 1.0
**Last Updated:** 2025-12-06
