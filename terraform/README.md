# Media Gateway - Terraform Infrastructure

This directory contains Terraform infrastructure-as-code for deploying Media Gateway on Google Cloud Platform.

## Architecture Overview

The infrastructure consists of:

- **VPC Network**: Custom VPC with Private Google Access and Cloud NAT
- **GKE Autopilot**: Managed Kubernetes cluster with Workload Identity
- **Cloud SQL**: PostgreSQL 15 with HA mode and automated backups
- **Memorystore**: Redis 7 with HA and persistence
- **Secret Manager**: Centralized secret management
- **Cloud Armor**: DDoS protection and WAF

## Directory Structure

```
terraform/
├── modules/                    # Reusable Terraform modules
│   ├── vpc/                   # VPC network configuration
│   ├── gke/                   # GKE Autopilot cluster
│   ├── cloudsql/              # PostgreSQL database
│   ├── memorystore/           # Redis cache
│   ├── secrets/               # Secret Manager
│   └── security/              # Cloud Armor security
├── environments/              # Environment-specific configurations
│   ├── dev/                   # Development environment
│   ├── staging/               # Staging environment
│   └── prod/                  # Production environment
└── README.md                  # This file
```

## Prerequisites

1. **GCP Project**: Create a GCP project for each environment
2. **Terraform**: Install Terraform >= 1.5.0
3. **gcloud CLI**: Install and authenticate with GCP
4. **Permissions**: Ensure you have the following IAM roles:
   - Compute Admin
   - Kubernetes Engine Admin
   - Cloud SQL Admin
   - Secret Manager Admin
   - Service Networking Admin

## Quick Start

### 1. Set up GCP credentials

```bash
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

### 2. Configure backend (optional but recommended)

For production use, configure a GCS backend for state management:

```bash
# Create GCS bucket for Terraform state
gsutil mb -p YOUR_PROJECT_ID -c STANDARD -l us-central1 gs://media-gateway-terraform-state-dev

# Enable versioning
gsutil versioning set on gs://media-gateway-terraform-state-dev
```

Update the backend configuration in `environments/{env}/main.tf`:

```hcl
backend "gcs" {
  bucket = "media-gateway-terraform-state-dev"
  prefix = "dev/state"
}
```

### 3. Initialize and deploy (Development)

```bash
cd environments/dev

# Update terraform.tfvars with your project ID
# project_id = "your-gcp-project-id-dev"

# Initialize Terraform
terraform init

# Review the plan
terraform plan

# Apply the configuration
terraform apply
```

## Environment Configuration

### Development (dev)

- **Purpose**: Local development and testing
- **Cost Optimized**: ZONAL availability, smaller instances
- **Cloud SQL**: db-custom-1-3840 (1 vCPU, 3.75GB RAM), ZONAL
- **Redis**: BASIC tier, 1GB memory
- **Security**: Relaxed rate limits, verbose logging

**Estimated Monthly Cost**: ~$150-$250

### Staging (staging)

- **Purpose**: Pre-production testing
- **Production-like**: REGIONAL availability, moderate capacity
- **Cloud SQL**: db-custom-2-7680 (2 vCPU, 7.5GB RAM), REGIONAL
- **Redis**: STANDARD_HA tier, 4GB memory, 1 replica
- **Security**: Production-level security with adaptive protection

**Estimated Monthly Cost**: ~$800-$1,200

### Production (prod)

- **Purpose**: Production workloads
- **Full HA**: REGIONAL availability, full capacity
- **Cloud SQL**: db-custom-4-15360 (4 vCPU, 15GB RAM), REGIONAL + read replica
- **Redis**: STANDARD_HA tier, 6GB memory, 2 replicas
- **Security**: Full Cloud Armor protection, adaptive DDoS defense
- **Release Channel**: STABLE

**Estimated Monthly Cost**: ~$2,500-$3,500

## Module Documentation

### VPC Module

Creates a custom VPC network with:
- Separate subnets for GKE and private services
- Private Google Access enabled
- Cloud NAT for outbound internet access
- VPC peering for Cloud SQL and Memorystore

**Inputs**: See `modules/vpc/variables.tf`
**Outputs**: See `modules/vpc/outputs.tf`

### GKE Module

Creates a GKE Autopilot cluster with:
- Workload Identity for secure service access
- Private cluster configuration
- Network policies enabled
- Binary Authorization (prod only)
- Managed Prometheus monitoring

**Inputs**: See `modules/gke/variables.tf`
**Outputs**: See `modules/gke/outputs.tf`

### Cloud SQL Module

Creates a PostgreSQL 15 database with:
- Private IP only (no public IP)
- Automated backups with point-in-time recovery
- SSL/TLS encryption in transit
- Optimized PostgreSQL flags
- Optional read replica for production

**Inputs**: See `modules/cloudsql/variables.tf`
**Outputs**: See `modules/cloudsql/outputs.tf`

### Memorystore Module

Creates a Redis 7 cache with:
- Private service access
- High availability (STANDARD_HA tier)
- RDB persistence snapshots
- Transit encryption
- Redis AUTH enabled

**Inputs**: See `modules/memorystore/variables.tf`
**Outputs**: See `modules/memorystore/outputs.tf`

### Secrets Module

Creates Secret Manager secrets for:
- JWT signing keys
- Encryption keys
- OAuth provider credentials
- Email service API keys
- Database passwords (auto-generated)

**Inputs**: See `modules/secrets/variables.tf`
**Outputs**: See `modules/secrets/outputs.tf`

### Security Module

Creates Cloud Armor security policy with:
- Rate limiting and DDoS protection
- OWASP Top 10 protection (SQL injection, XSS, etc.)
- Adaptive Layer 7 DDoS defense
- Custom IP allow/block lists

**Inputs**: See `modules/security/variables.tf`
**Outputs**: See `modules/security/outputs.tf`

## Deployment Workflow

### Initial Deployment

```bash
# 1. Navigate to environment
cd environments/dev

# 2. Update variables
vim terraform.tfvars

# 3. Initialize
terraform init

# 4. Plan
terraform plan -out=tfplan

# 5. Review and apply
terraform apply tfplan
```

### Updating Infrastructure

```bash
# 1. Make changes to .tf files or variables

# 2. Plan changes
terraform plan -out=tfplan

# 3. Review diff carefully

# 4. Apply
terraform apply tfplan
```

### Destroying Infrastructure

```bash
# WARNING: This will destroy all resources
terraform destroy
```

## Outputs

After applying, retrieve important information:

```bash
# Get all outputs
terraform output

# Get specific output
terraform output gke_cluster_name

# Get sensitive output
terraform output -json connection_info | jq
```

## Connecting to GKE

```bash
# Configure kubectl
gcloud container clusters get-credentials $(terraform output -raw gke_cluster_name) \
  --region us-central1 \
  --project YOUR_PROJECT_ID

# Verify connection
kubectl get nodes
```

## Accessing Secrets

```bash
# List secrets
gcloud secrets list

# Get JWT secret
gcloud secrets versions access latest --secret="media-gateway-dev-jwt-secret"

# Get database password
gcloud secrets versions access latest --secret="media-gateway-dev-db-password"
```

## Cost Optimization Tips

1. **Development**: Use ZONAL availability and smaller instance types
2. **Auto-scaling**: GKE Autopilot automatically scales nodes based on workload
3. **Committed Use Discounts**: Consider CUDs for production workloads
4. **Cloud SQL**: Use ZONAL for non-production environments
5. **Redis**: Use BASIC tier for development

## Security Best Practices

1. **Never commit secrets**: Use Secret Manager, not git
2. **Least privilege**: Grant minimal IAM permissions
3. **Private networks**: All services use private IPs
4. **Encryption**: TLS in transit, encryption at rest
5. **Regular updates**: Keep Terraform and provider versions current
6. **State security**: Store Terraform state in GCS with versioning

## Monitoring and Logging

All resources are configured with:
- Cloud Monitoring integration
- Cloud Logging (structured logs)
- GKE Managed Prometheus
- Cloud SQL Insights
- VPC Flow Logs

## Troubleshooting

### Common Issues

**Issue**: API not enabled
```bash
# Enable required APIs
gcloud services enable compute.googleapis.com
gcloud services enable container.googleapis.com
gcloud services enable sqladmin.googleapis.com
```

**Issue**: Insufficient permissions
```bash
# Check current permissions
gcloud projects get-iam-policy YOUR_PROJECT_ID \
  --flatten="bindings[].members" \
  --filter="bindings.members:YOUR_EMAIL"
```

**Issue**: VPC peering conflicts
- Ensure IP ranges don't overlap
- Check reserved IP ranges in VPC peering

**Issue**: GKE connection timeout
- Verify master authorized networks
- Check firewall rules

## Maintenance

### Regular Tasks

- **Monthly**: Review Cloud SQL backups
- **Quarterly**: Update Terraform provider versions
- **Quarterly**: Review and rotate secrets
- **Annually**: Review IAM permissions

### Updates

```bash
# Update Terraform providers
terraform init -upgrade

# Update GKE version (automatic with release channel)
# Autopilot handles node upgrades automatically
```

## Support

For issues or questions:
1. Check module documentation
2. Review GCP Cloud Console logs
3. Consult GCP documentation
4. File an issue in the project repository

## License

This Terraform configuration is part of the Media Gateway project.
