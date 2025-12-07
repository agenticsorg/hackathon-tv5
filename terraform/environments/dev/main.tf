# Development Environment Configuration - Media Gateway

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.5"
    }
  }

  # Uncomment and configure backend for remote state
  # backend "gcs" {
  #   bucket = "media-gateway-terraform-state-dev"
  #   prefix = "dev/state"
  # }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

# VPC Module
module "vpc" {
  source = "../../modules/vpc"

  project_id   = var.project_id
  project_name = var.project_name
  environment  = var.environment
  region       = var.region

  gke_subnet_cidr     = var.gke_subnet_cidr
  gke_pods_cidr       = var.gke_pods_cidr
  gke_services_cidr   = var.gke_services_cidr
  private_subnet_cidr = var.private_subnet_cidr
}

# GKE Module
module "gke" {
  source = "../../modules/gke"

  project_id   = var.project_id
  project_name = var.project_name
  environment  = var.environment
  region       = var.region

  network_name                  = module.vpc.network_name
  subnet_name                   = module.vpc.gke_subnet_name
  pods_range_name               = module.vpc.pods_range_name
  services_range_name           = module.vpc.services_range_name
  master_ipv4_cidr_block        = var.master_ipv4_cidr_block
  master_authorized_networks    = var.master_authorized_networks
  maintenance_window_start      = var.maintenance_window_start
  release_channel               = var.release_channel
  workload_identity_namespaces  = var.workload_identity_namespaces
  vpc_connection_dependency     = module.vpc.private_vpc_connection

  depends_on = [module.vpc]
}

# Cloud SQL Module
module "cloudsql" {
  source = "../../modules/cloudsql"

  project_id   = var.project_id
  project_name = var.project_name
  environment  = var.environment
  region       = var.region

  database_version          = var.database_version
  tier                      = var.cloudsql_tier
  availability_type         = var.cloudsql_availability_type
  disk_size                 = var.cloudsql_disk_size
  disk_autoresize_limit     = var.cloudsql_disk_autoresize_limit
  database_name             = var.database_name
  additional_databases      = var.additional_databases
  db_user                   = var.db_user
  network_self_link         = module.vpc.network_self_link
  enable_replica            = var.enable_cloudsql_replica
  replica_region            = var.replica_region
  vpc_connection_dependency = module.vpc.private_vpc_connection

  depends_on = [module.vpc]
}

# Memorystore Redis Module
module "memorystore" {
  source = "../../modules/memorystore"

  project_id   = var.project_id
  project_name = var.project_name
  environment  = var.environment
  region       = var.region

  tier                      = var.redis_tier
  memory_size_gb            = var.redis_memory_size_gb
  redis_version             = var.redis_version
  reserved_ip_range         = var.redis_reserved_ip_range
  network_id                = module.vpc.network_id
  replica_count             = var.redis_replica_count
  persistence_mode          = var.redis_persistence_mode
  rdb_snapshot_period       = var.redis_rdb_snapshot_period
  transit_encryption_mode   = var.redis_transit_encryption_mode
  auth_enabled              = var.redis_auth_enabled
  vpc_connection_dependency = module.vpc.private_vpc_connection

  depends_on = [module.vpc]
}

# Secret Manager Module
module "secrets" {
  source = "../../modules/secrets"

  project_id                         = var.project_id
  project_name                       = var.project_name
  environment                        = var.environment
  workload_identity_service_accounts = module.gke.workload_identity_service_accounts

  depends_on = [module.gke]
}

# Cloud Armor Security Module
module "security" {
  source = "../../modules/security"

  project_id   = var.project_id
  project_name = var.project_name
  environment  = var.environment

  rate_limit_threshold_count    = var.rate_limit_threshold_count
  rate_limit_threshold_interval = var.rate_limit_threshold_interval
  ban_duration_sec              = var.ban_duration_sec
  blocked_ip_ranges             = var.blocked_ip_ranges
  allowed_ip_ranges             = var.allowed_ip_ranges
  enable_adaptive_protection    = var.enable_adaptive_protection
  log_level                     = var.log_level
}

# Enable required APIs
resource "google_project_service" "required_apis" {
  for_each = toset([
    "compute.googleapis.com",
    "container.googleapis.com",
    "sqladmin.googleapis.com",
    "redis.googleapis.com",
    "secretmanager.googleapis.com",
    "servicenetworking.googleapis.com",
    "cloudresourcemanager.googleapis.com",
    "iam.googleapis.com",
    "monitoring.googleapis.com",
    "logging.googleapis.com",
  ])

  project            = var.project_id
  service            = each.value
  disable_on_destroy = false
}
