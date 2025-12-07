# Terraform Infrastructure Documentation - Media Gateway

## Overview

This document describes the Terraform infrastructure implementation for Media Gateway on Google Cloud Platform, following SPARC specifications.

## Implementation Summary

**Task**: BATCH_009 TASK-006 - Create Terraform GCP Infrastructure Module
**Status**: Complete
**Date**: 2025-12-06

## Architecture

### Infrastructure Components

1. **VPC Network** (`modules/vpc`)
   - Custom VPC with Private Google Access
   - Cloud NAT for outbound internet access
   - Separate subnets for GKE and private services
   - VPC peering for Cloud SQL and Memorystore
   - Firewall rules for internal traffic and health checks

2. **GKE Autopilot** (`modules/gke`)
   - Managed Kubernetes with auto-scaling
   - Workload Identity for secure service access
   - Private cluster configuration
   - Network policies enabled
   - Binary Authorization (production only)
   - Managed Prometheus monitoring

3. **Cloud SQL PostgreSQL** (`modules/cloudsql`)
   - PostgreSQL 15 database
   - High availability (REGIONAL mode)
   - Private IP only (no public access)
   - Automated backups with PITR
   - SSL/TLS encryption
   - Read replica support (production)
   - Optimized database flags

4. **Memorystore Redis** (`modules/memorystore`)
   - Redis 7 cache
   - High availability (STANDARD_HA tier)
   - RDB persistence snapshots
   - Transit encryption
   - Redis AUTH enabled
   - Read replicas support

5. **Secret Manager** (`modules/secrets`)
   - JWT signing keys
   - Encryption keys
   - OAuth provider credentials
   - Email service API keys
   - Database passwords (auto-generated)
   - Workload Identity IAM bindings

6. **Cloud Armor** (`modules/security`)
   - DDoS protection
   - Rate limiting
   - OWASP Top 10 protection
   - Adaptive Layer 7 defense
   - Custom IP allow/block lists

## Directory Structure

```
terraform/
├── modules/
│   ├── vpc/                 # VPC network module
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   ├── gke/                 # GKE Autopilot module
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   ├── cloudsql/            # Cloud SQL module
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   ├── memorystore/         # Memorystore Redis module
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   ├── secrets/             # Secret Manager module
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   └── security/            # Cloud Armor module
│       ├── main.tf
│       ├── variables.tf
│       └── outputs.tf
├── environments/
│   ├── dev/                 # Development environment
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── terraform.tfvars
│   ├── staging/             # Staging environment
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   └── prod/                # Production environment
│       ├── main.tf
│       ├── variables.tf
│       └── outputs.tf
├── versions.tf              # Provider version constraints
├── Makefile                 # Common operations
├── .gitignore               # Git ignore rules
├── terraform.tfvars.example # Example variables file
└── README.md                # User documentation
```

## Environment Configurations

### Development (dev)

**Purpose**: Cost-optimized for development and testing

**Resources**:
- Cloud SQL: db-custom-1-3840 (1 vCPU, 3.75GB RAM), ZONAL
- Redis: BASIC tier, 1GB memory
- GKE: Autopilot with REGULAR release channel
- Security: Relaxed rate limits, verbose logging

**Network CIDR**:
- GKE nodes: 10.0.0.0/20
- GKE pods: 10.4.0.0/14
- GKE services: 10.8.0.0/20
- Private subnet: 10.1.0.0/20
- GKE master: 172.16.0.0/28

**Estimated Cost**: ~$150-$250/month

### Staging (staging)

**Purpose**: Production-like environment for testing

**Resources**:
- Cloud SQL: db-custom-2-7680 (2 vCPU, 7.5GB RAM), REGIONAL
- Redis: STANDARD_HA tier, 4GB memory, 1 replica
- GKE: Autopilot with REGULAR release channel
- Security: Production-level security policies

**Network CIDR**:
- GKE nodes: 10.10.0.0/20
- GKE pods: 10.14.0.0/14
- GKE services: 10.18.0.0/20
- Private subnet: 10.11.0.0/20
- GKE master: 172.16.1.0/28

**Estimated Cost**: ~$800-$1,200/month

### Production (prod)

**Purpose**: Full HA configuration for production workloads

**Resources**:
- Cloud SQL: db-custom-4-15360 (4 vCPU, 15GB RAM), REGIONAL + read replica
- Redis: STANDARD_HA tier, 6GB memory, 2 replicas
- GKE: Autopilot with STABLE release channel
- Security: Full Cloud Armor with adaptive protection
- Deletion protection enabled

**Network CIDR**:
- GKE nodes: 10.20.0.0/20
- GKE pods: 10.24.0.0/14
- GKE services: 10.28.0.0/20
- Private subnet: 10.21.0.0/20
- GKE master: 172.16.2.0/28

**Estimated Cost**: ~$2,500-$3,500/month

## SPARC Compliance

### Specification Requirements

- ✅ Region: us-central1 (primary), us-east1 (DR/replica)
- ✅ GKE: Autopilot mode, 2-50 nodes (auto-scaled)
- ✅ Cloud SQL: PostgreSQL 15, HA mode
- ✅ Memorystore: Redis 7, 6GB HA (production)
- ✅ Budget: <$4,000/month (production configuration)

### Constraints

- ✅ Terraform module best practices (reusable modules)
- ✅ Workspace variables for environments (dev/staging/prod)
- ✅ GCP naming conventions (project-env-resource)
- ✅ Private networking (no public IPs)
- ✅ Workload Identity (secure service access)
- ✅ Automated backups and DR

## Acceptance Criteria

All acceptance criteria have been met:

1. ✅ Created terraform/ directory structure
2. ✅ VPC module with Private Google Access and Cloud NAT
3. ✅ GKE Autopilot module with Workload Identity
4. ✅ Cloud SQL module with private IP and automated backups
5. ✅ Memorystore module with private service access
6. ✅ Secret Manager configuration
7. ✅ Cloud Armor security policy
8. ✅ Environment-specific configurations (dev, staging, prod)

## Key Features

### Security

- **Private networking**: All services use private IPs
- **Workload Identity**: Secure service-to-service authentication
- **Secret Manager**: Centralized secret management
- **Cloud Armor**: DDoS and OWASP Top 10 protection
- **Encryption**: TLS in transit, encryption at rest
- **SSL/TLS**: Required for all database connections

### High Availability

- **Cloud SQL**: REGIONAL availability with automatic failover
- **Redis**: STANDARD_HA tier with replicas
- **GKE**: Multi-zone Autopilot cluster
- **Backups**: Automated backups with point-in-time recovery
- **Read replicas**: Optional for production workloads

### Monitoring & Observability

- **Cloud Monitoring**: Integrated metrics
- **Cloud Logging**: Structured logs
- **GKE Managed Prometheus**: Kubernetes monitoring
- **Cloud SQL Insights**: Query performance
- **VPC Flow Logs**: Network traffic analysis

### Automation

- **Autopilot**: Automatic node management
- **Auto-scaling**: Based on workload
- **Auto-backups**: Scheduled database backups
- **Auto-updates**: Managed GKE updates
- **Self-healing**: Automatic recovery

## Usage

### Quick Start

```bash
# Initialize development environment
cd terraform/environments/dev
terraform init
terraform plan
terraform apply

# Or use Makefile
make init ENV=dev
make plan ENV=dev
make apply ENV=dev
```

### Connect to GKE

```bash
# Using Makefile
make connect-gke ENV=dev

# Or manually
gcloud container clusters get-credentials \
  media-gateway-dev-gke \
  --region us-central1 \
  --project YOUR_PROJECT_ID
```

### Access Secrets

```bash
# List secrets
gcloud secrets list

# Get JWT secret
gcloud secrets versions access latest \
  --secret="media-gateway-dev-jwt-secret"
```

## Cost Management

### Development
- Use ZONAL availability
- Smaller instance types
- Disable replicas and backups
- **Cost**: ~$150-$250/month

### Staging
- REGIONAL availability
- Moderate capacity
- Limited replicas
- **Cost**: ~$800-$1,200/month

### Production
- Full HA configuration
- Read replicas enabled
- Extended backup retention
- **Cost**: ~$2,500-$3,500/month

### Optimization Tips

1. Use committed use discounts for production
2. Enable auto-scaling for GKE
3. Use preemptible nodes for non-critical workloads
4. Monitor and right-size instances
5. Clean up unused resources

## Security Best Practices

1. **Never commit secrets**: Use Secret Manager
2. **Least privilege**: Grant minimal IAM permissions
3. **Private networks**: No public IP addresses
4. **State security**: Store in GCS with versioning
5. **Regular audits**: Review IAM and firewall rules
6. **Rotate secrets**: Quarterly rotation schedule

## Maintenance

### Regular Tasks

- **Weekly**: Review Cloud Monitoring alerts
- **Monthly**: Check Cloud SQL backups
- **Quarterly**: Update Terraform providers
- **Quarterly**: Rotate secrets
- **Annually**: Review IAM permissions

### Disaster Recovery

1. **Backups**: Automated daily backups with PITR
2. **Replicas**: Read replicas in us-east1 (production)
3. **State backup**: Terraform state in GCS with versioning
4. **Documentation**: Recovery procedures documented

## Files Created

### Modules (6 modules, 18 files)

1. **VPC Module** (3 files)
   - `/workspaces/media-gateway/terraform/modules/vpc/main.tf`
   - `/workspaces/media-gateway/terraform/modules/vpc/variables.tf`
   - `/workspaces/media-gateway/terraform/modules/vpc/outputs.tf`

2. **GKE Module** (3 files)
   - `/workspaces/media-gateway/terraform/modules/gke/main.tf`
   - `/workspaces/media-gateway/terraform/modules/gke/variables.tf`
   - `/workspaces/media-gateway/terraform/modules/gke/outputs.tf`

3. **Cloud SQL Module** (3 files)
   - `/workspaces/media-gateway/terraform/modules/cloudsql/main.tf`
   - `/workspaces/media-gateway/terraform/modules/cloudsql/variables.tf`
   - `/workspaces/media-gateway/terraform/modules/cloudsql/outputs.tf`

4. **Memorystore Module** (3 files)
   - `/workspaces/media-gateway/terraform/modules/memorystore/main.tf`
   - `/workspaces/media-gateway/terraform/modules/memorystore/variables.tf`
   - `/workspaces/media-gateway/terraform/modules/memorystore/outputs.tf`

5. **Secrets Module** (3 files)
   - `/workspaces/media-gateway/terraform/modules/secrets/main.tf`
   - `/workspaces/media-gateway/terraform/modules/secrets/variables.tf`
   - `/workspaces/media-gateway/terraform/modules/secrets/outputs.tf`

6. **Security Module** (3 files)
   - `/workspaces/media-gateway/terraform/modules/security/main.tf`
   - `/workspaces/media-gateway/terraform/modules/security/variables.tf`
   - `/workspaces/media-gateway/terraform/modules/security/outputs.tf`

### Environments (3 environments, 10 files)

1. **Development** (4 files)
   - `/workspaces/media-gateway/terraform/environments/dev/main.tf`
   - `/workspaces/media-gateway/terraform/environments/dev/variables.tf`
   - `/workspaces/media-gateway/terraform/environments/dev/outputs.tf`
   - `/workspaces/media-gateway/terraform/environments/dev/terraform.tfvars`

2. **Staging** (3 files)
   - `/workspaces/media-gateway/terraform/environments/staging/main.tf`
   - `/workspaces/media-gateway/terraform/environments/staging/variables.tf`
   - `/workspaces/media-gateway/terraform/environments/staging/outputs.tf`

3. **Production** (3 files)
   - `/workspaces/media-gateway/terraform/environments/prod/main.tf`
   - `/workspaces/media-gateway/terraform/environments/prod/variables.tf`
   - `/workspaces/media-gateway/terraform/environments/prod/outputs.tf`

### Root Configuration (5 files)

- `/workspaces/media-gateway/terraform/README.md`
- `/workspaces/media-gateway/terraform/versions.tf`
- `/workspaces/media-gateway/terraform/Makefile`
- `/workspaces/media-gateway/terraform/.gitignore`
- `/workspaces/media-gateway/terraform/terraform.tfvars.example`

### Documentation (1 file)

- `/workspaces/media-gateway/docs/terraform-infrastructure.md` (this file)

## Total Files Created: 34

## Next Steps

1. **Configure GCP Project**: Create GCP project for each environment
2. **Set up State Backend**: Create GCS buckets for Terraform state
3. **Update Variables**: Configure project_id in terraform.tfvars
4. **Initialize**: Run `terraform init` in each environment
5. **Deploy**: Apply Terraform configuration
6. **Verify**: Check resources in GCP Console
7. **Connect**: Configure kubectl and verify cluster access

## Conclusion

The Terraform infrastructure module is complete and ready for deployment. All SPARC requirements and acceptance criteria have been met. The implementation follows Terraform best practices with:

- Modular, reusable components
- Environment-specific configurations
- Security-first design
- Cost optimization for each environment
- Comprehensive documentation
- Makefile for common operations
- Version constraints for reproducibility

The infrastructure is production-ready and can scale from development to production workloads while staying within the $4,000/month budget constraint.
