# Deploying RuVector on Google Cloud SQL

This guide walks you through deploying RuVector as a custom PostgreSQL extension on Google Cloud SQL. Cloud SQL provides managed PostgreSQL with automatic backups, high availability, and scaling.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Google Cloud Platform                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────────┐      ┌──────────────────────────────────┐   │
│   │  Cloud Run   │      │         Cloud SQL                 │   │
│   │  (API/App)   │─────►│   PostgreSQL 17 + RuVector        │   │
│   └──────────────┘      │   • 65+ SQL functions             │   │
│          │              │   • Vector search                 │   │
│          │              │   • GNN, Attention, Routing       │   │
│          ▼              │   • Automatic backups             │   │
│   ┌──────────────┐      │   • High availability             │   │
│   │   Users      │      └──────────────────────────────────┘   │
│   └──────────────┘                                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Prerequisites

- Google Cloud account with billing enabled
- `gcloud` CLI installed and authenticated
- Docker installed locally
- Project with Cloud SQL Admin API enabled

```bash
# Install gcloud CLI
curl -O https://dl.google.com/dl/cloudsdk/channels/rapid/downloads/google-cloud-cli-linux-x86_64.tar.gz
tar -xf google-cloud-cli-linux-x86_64.tar.gz
./google-cloud-sdk/install.sh

# Login and set project
gcloud auth login
gcloud config set project YOUR_PROJECT_ID

# Enable required APIs
gcloud services enable sqladmin.googleapis.com
gcloud services enable compute.googleapis.com
gcloud services enable artifactregistry.googleapis.com
```

---

## Method 1: Custom PostgreSQL Image with Cloud SQL (Recommended)

Cloud SQL supports custom database flags but not custom extensions directly. However, you can use **AlloyDB** or a **Compute Engine VM** with Cloud SQL Proxy for a fully managed experience.

### Step 1: Create a Compute Engine VM with RuVector

```bash
# Set variables
export PROJECT_ID=$(gcloud config get-value project)
export ZONE=us-central1-a
export INSTANCE_NAME=ruvector-db

# Create a VM with Container-Optimized OS
gcloud compute instances create-with-container $INSTANCE_NAME \
  --project=$PROJECT_ID \
  --zone=$ZONE \
  --machine-type=e2-standard-4 \
  --container-image=gcr.io/$PROJECT_ID/ruvector:latest \
  --container-restart-policy=always \
  --container-env=POSTGRES_USER=ruvector,POSTGRES_PASSWORD=YOUR_SECURE_PASSWORD,POSTGRES_DB=ruvector_db \
  --boot-disk-size=100GB \
  --boot-disk-type=pd-ssd \
  --tags=postgres-server \
  --metadata=google-logging-enabled=true
```

### Step 2: Configure Firewall Rules

```bash
# Allow PostgreSQL traffic (restrict to your IP in production)
gcloud compute firewall-rules create allow-postgres \
  --direction=INGRESS \
  --priority=1000 \
  --network=default \
  --action=ALLOW \
  --rules=tcp:5432 \
  --source-ranges=0.0.0.0/0 \
  --target-tags=postgres-server

# For production, restrict to specific IPs:
# --source-ranges=YOUR_IP/32
```

### Step 3: Get the External IP

```bash
gcloud compute instances describe $INSTANCE_NAME \
  --zone=$ZONE \
  --format='get(networkInterfaces[0].accessConfigs[0].natIP)'
```

### Step 4: Connect to RuVector

```bash
# Connect via psql
psql -h EXTERNAL_IP -U ruvector -d ruvector_db

# Verify RuVector is loaded
SELECT ruvector_version();
-- Returns: 0.2.5

# List all RuVector functions
SELECT proname FROM pg_proc WHERE proname LIKE 'ruvector_%' ORDER BY proname;
```

---

## Method 2: Cloud SQL with Proxy + Compute Engine Sidecar

For true Cloud SQL managed experience with RuVector extension processing:

### Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Your App      │────►│  Compute Engine │────►│   Cloud SQL     │
│   (Cloud Run)   │     │  (RuVector API) │     │   (PostgreSQL)  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                              │
                              │ RuVector functions
                              │ run here
                              ▼
                        ┌─────────────────┐
                        │ Vector Search   │
                        │ GNN Processing  │
                        │ Agent Routing   │
                        └─────────────────┘
```

### Step 1: Create Cloud SQL Instance

```bash
# Create a Cloud SQL PostgreSQL instance
gcloud sql instances create ruvector-sql \
  --database-version=POSTGRES_17 \
  --tier=db-standard-2 \
  --region=us-central1 \
  --storage-size=100GB \
  --storage-type=SSD \
  --availability-type=REGIONAL \
  --backup-start-time=03:00 \
  --maintenance-window-day=SUN \
  --maintenance-window-hour=04

# Set root password
gcloud sql users set-password postgres \
  --instance=ruvector-sql \
  --password=YOUR_SECURE_PASSWORD

# Create database
gcloud sql databases create ruvector_db --instance=ruvector-sql
```

### Step 2: Create RuVector API Service

Create a lightweight API that connects to Cloud SQL and provides RuVector functions:

```javascript
// ruvector-api/server.js
const express = require('express');
const { Pool } = require('pg');
const ruvector = require('ruvector');

const app = express();
app.use(express.json());

// Connect to Cloud SQL via Unix socket
const pool = new Pool({
  user: process.env.DB_USER,
  password: process.env.DB_PASS,
  database: process.env.DB_NAME,
  host: `/cloudsql/${process.env.INSTANCE_CONNECTION_NAME}`,
});

// Vector search endpoint
app.post('/search', async (req, res) => {
  const { query_embedding, limit = 10 } = req.body;

  // Use RuVector's JavaScript SDK for vector operations
  const results = await ruvector.search(query_embedding, { limit });
  res.json(results);
});

// Semantic routing endpoint
app.post('/route', async (req, res) => {
  const { query } = req.body;
  const agent = await ruvector.route(query);
  res.json({ agent });
});

// GNN-enhanced search
app.post('/gnn-search', async (req, res) => {
  const { query_embedding, use_gnn = true } = req.body;
  const results = await ruvector.gnnSearch(query_embedding, { use_gnn });
  res.json(results);
});

const PORT = process.env.PORT || 8080;
app.listen(PORT, () => console.log(`RuVector API running on port ${PORT}`));
```

### Step 3: Deploy API to Cloud Run

```bash
# Build and push the API image
docker build -t gcr.io/$PROJECT_ID/ruvector-api:latest ./ruvector-api
docker push gcr.io/$PROJECT_ID/ruvector-api:latest

# Deploy to Cloud Run with Cloud SQL connection
gcloud run deploy ruvector-api \
  --image gcr.io/$PROJECT_ID/ruvector-api:latest \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --add-cloudsql-instances $PROJECT_ID:us-central1:ruvector-sql \
  --set-env-vars "DB_USER=postgres,DB_PASS=YOUR_PASSWORD,DB_NAME=ruvector_db,INSTANCE_CONNECTION_NAME=$PROJECT_ID:us-central1:ruvector-sql"
```

---

## Method 3: Full RuVector on Compute Engine (Production Ready)

For full PostgreSQL extension support with all 65+ functions:

### Step 1: Push Image to Google Container Registry

```bash
# Tag and push the RuVector image
docker tag ruvnet/ruvector-postgres:latest gcr.io/$PROJECT_ID/ruvector:latest
docker push gcr.io/$PROJECT_ID/ruvector:latest
```

### Step 2: Create Instance Template

```bash
# Create an instance template for auto-scaling
gcloud compute instance-templates create-with-container ruvector-template \
  --machine-type=e2-standard-4 \
  --container-image=gcr.io/$PROJECT_ID/ruvector:latest \
  --container-restart-policy=always \
  --container-env=POSTGRES_USER=ruvector,POSTGRES_PASSWORD=YOUR_SECURE_PASSWORD,POSTGRES_DB=ruvector_db \
  --boot-disk-size=200GB \
  --boot-disk-type=pd-ssd \
  --tags=ruvector-db \
  --metadata=google-logging-enabled=true \
  --container-mount-host-path=host-path=/mnt/disks/pgdata,mount-path=/var/lib/postgresql/data
```

### Step 3: Create Managed Instance Group (Optional - for HA)

```bash
# Create a managed instance group
gcloud compute instance-groups managed create ruvector-group \
  --template=ruvector-template \
  --size=1 \
  --zone=$ZONE

# Set up health check
gcloud compute health-checks create tcp ruvector-health \
  --port=5432 \
  --check-interval=10s \
  --timeout=5s \
  --healthy-threshold=2 \
  --unhealthy-threshold=3
```

### Step 4: Set Up Load Balancer (Optional)

```bash
# Create a TCP load balancer for PostgreSQL
gcloud compute backend-services create ruvector-backend \
  --protocol=TCP \
  --health-checks=ruvector-health \
  --global

gcloud compute backend-services add-backend ruvector-backend \
  --instance-group=ruvector-group \
  --instance-group-zone=$ZONE \
  --global

gcloud compute target-tcp-proxies create ruvector-proxy \
  --backend-service=ruvector-backend

gcloud compute forwarding-rules create ruvector-lb \
  --global \
  --target-tcp-proxy=ruvector-proxy \
  --ports=5432
```

---

## Connecting Your Application

### From Cloud Run / Cloud Functions

```javascript
const { Pool } = require('pg');

// For Compute Engine deployment
const pool = new Pool({
  host: 'EXTERNAL_IP_OR_INTERNAL_IP',
  port: 5432,
  user: 'ruvector',
  password: 'YOUR_PASSWORD',
  database: 'ruvector_db',
});

// Example: Semantic search
async function semanticSearch(queryEmbedding, limit = 10) {
  const result = await pool.query(`
    SELECT id, title,
           embedding <-> $1::vector AS distance
    FROM documents
    ORDER BY distance
    LIMIT $2
  `, [JSON.stringify(queryEmbedding), limit]);

  return result.rows;
}

// Example: Agent routing
async function routeQuery(queryEmbedding) {
  const result = await pool.query(`
    SELECT ruvector_route($1::vector)
  `, [JSON.stringify(queryEmbedding)]);

  return result.rows[0].ruvector_route;
}
```

### From Python

```python
import psycopg2
import numpy as np

conn = psycopg2.connect(
    host="EXTERNAL_IP",
    port=5432,
    user="ruvector",
    password="YOUR_PASSWORD",
    database="ruvector_db"
)

def semantic_search(query_embedding, limit=10):
    with conn.cursor() as cur:
        cur.execute("""
            SELECT id, title,
                   embedding <-> %s::vector AS distance
            FROM documents
            ORDER BY distance
            LIMIT %s
        """, (str(query_embedding.tolist()), limit))
        return cur.fetchall()

def enable_learning():
    with conn.cursor() as cur:
        cur.execute("SELECT ruvector_enable_learning(true)")
        conn.commit()
```

---

## Security Best Practices

### 1. Use Private IP

```bash
# Create a VPC-native cluster
gcloud compute networks create ruvector-vpc --subnet-mode=custom

gcloud compute networks subnets create ruvector-subnet \
  --network=ruvector-vpc \
  --region=us-central1 \
  --range=10.0.0.0/24

# Deploy VM with private IP only
gcloud compute instances create ruvector-db \
  --no-address \
  --network=ruvector-vpc \
  --subnet=ruvector-subnet \
  ...
```

### 2. Use Cloud SQL Auth Proxy

```bash
# Download the proxy
curl -o cloud-sql-proxy https://storage.googleapis.com/cloud-sql-connectors/cloud-sql-proxy/v2.8.1/cloud-sql-proxy.linux.amd64
chmod +x cloud-sql-proxy

# Run the proxy
./cloud-sql-proxy --port 5432 $PROJECT_ID:us-central1:ruvector-sql
```

### 3. Enable SSL/TLS

```bash
# Force SSL connections
gcloud sql instances patch ruvector-sql --require-ssl
```

### 4. Use Secret Manager for Credentials

```bash
# Store password in Secret Manager
echo -n "YOUR_PASSWORD" | gcloud secrets create ruvector-db-password --data-file=-

# Access in Cloud Run
gcloud run deploy ... --set-secrets=DB_PASS=ruvector-db-password:latest
```

---

## Monitoring & Logging

### Enable Cloud Monitoring

```bash
# View logs
gcloud logging read "resource.type=gce_instance AND resource.labels.instance_id=INSTANCE_ID" --limit=50

# Create uptime check
gcloud monitoring uptime create ruvector-uptime \
  --display-name="RuVector Database" \
  --resource-type=gce-instance \
  --resource-labels=project_id=$PROJECT_ID,instance_id=INSTANCE_ID,zone=$ZONE
```

### Custom Metrics

```sql
-- Query to monitor index health
SELECT ruvector_learning_stats();

-- Check memory usage
SELECT ruvector_memory_stats();

-- Monitor agent routing performance
SELECT ruvector_routing_stats();
```

---

## Cost Optimization

| Configuration | Monthly Cost (est.) | Use Case |
|---------------|---------------------|----------|
| e2-micro (1 vCPU, 1GB) | ~$6 | Development/Testing |
| e2-standard-2 (2 vCPU, 8GB) | ~$50 | Small production |
| e2-standard-4 (4 vCPU, 16GB) | ~$100 | Medium production |
| n2-standard-8 (8 vCPU, 32GB) | ~$250 | Large production |
| n2-highmem-16 (16 vCPU, 128GB) | ~$700 | Enterprise |

### Cost-Saving Tips

1. **Use preemptible VMs** for non-critical workloads (80% cheaper)
2. **Schedule shutdowns** for dev/test instances
3. **Use committed use discounts** for production (up to 57% off)
4. **Right-size instances** based on actual usage

```bash
# Create preemptible instance (for dev/test)
gcloud compute instances create ruvector-dev \
  --preemptible \
  --machine-type=e2-standard-2 \
  ...
```

---

## Troubleshooting

### Connection Refused

```bash
# Check if container is running
gcloud compute ssh $INSTANCE_NAME --command="docker ps"

# Check logs
gcloud compute ssh $INSTANCE_NAME --command="docker logs \$(docker ps -q)"
```

### Extension Not Loading

```bash
# Verify RuVector files exist
gcloud compute ssh $INSTANCE_NAME --command="docker exec \$(docker ps -q) ls /usr/share/postgresql/17/extension/"
```

### Performance Issues

```sql
-- Check HNSW index parameters
SHOW hnsw.ef_search;

-- Increase for better accuracy (slower)
SET hnsw.ef_search = 200;

-- Check index stats
SELECT * FROM pg_stat_user_indexes WHERE indexrelname LIKE '%hnsw%';
```

---

## Quick Reference

### Essential Commands

```bash
# Deploy RuVector
gcloud compute instances create-with-container ruvector-db \
  --container-image=ruvnet/ruvector-postgres:latest \
  --machine-type=e2-standard-4 \
  --zone=us-central1-a

# Get IP
gcloud compute instances describe ruvector-db --zone=us-central1-a --format='get(networkInterfaces[0].accessConfigs[0].natIP)'

# Connect
psql -h EXTERNAL_IP -U ruvector -d ruvector_db

# SSH into VM
gcloud compute ssh ruvector-db --zone=us-central1-a

# View container logs
gcloud compute ssh ruvector-db --command="docker logs \$(docker ps -q) --tail 100"

# Restart container
gcloud compute ssh ruvector-db --command="docker restart \$(docker ps -q)"

# Delete instance
gcloud compute instances delete ruvector-db --zone=us-central1-a
```

---

## Next Steps

1. **Set up backups**: Configure Cloud Storage for PostgreSQL backups
2. **Enable monitoring**: Set up Prometheus/Grafana or Cloud Monitoring
3. **Configure SSL**: Enable TLS for secure connections
4. **Set up replication**: Configure read replicas for scaling
5. **Implement CI/CD**: Automate deployments with Cloud Build

For more information:
- [RuVector Documentation](https://github.com/ruvnet/ruvector)
- [Google Cloud SQL Documentation](https://cloud.google.com/sql/docs)
- [PostgreSQL on GCE Best Practices](https://cloud.google.com/architecture/postgresql-deployment-on-gce)
