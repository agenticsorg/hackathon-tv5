# Development Environment Variables

variable "project_id" {
  description = "GCP Project ID"
  type        = string
}

variable "project_name" {
  description = "Project name for resource naming"
  type        = string
  default     = "media-gateway"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "dev"
}

variable "region" {
  description = "GCP region"
  type        = string
  default     = "us-central1"
}

variable "replica_region" {
  description = "GCP region for replicas"
  type        = string
  default     = "us-east1"
}

# VPC Variables
variable "gke_subnet_cidr" {
  description = "CIDR range for GKE nodes subnet"
  type        = string
  default     = "10.0.0.0/20"
}

variable "gke_pods_cidr" {
  description = "CIDR range for GKE pods"
  type        = string
  default     = "10.4.0.0/14"
}

variable "gke_services_cidr" {
  description = "CIDR range for GKE services"
  type        = string
  default     = "10.8.0.0/20"
}

variable "private_subnet_cidr" {
  description = "CIDR range for private subnet"
  type        = string
  default     = "10.1.0.0/20"
}

# GKE Variables
variable "master_ipv4_cidr_block" {
  description = "CIDR block for GKE master"
  type        = string
  default     = "172.16.0.0/28"
}

variable "master_authorized_networks" {
  description = "List of master authorized networks"
  type = list(object({
    cidr_block   = string
    display_name = string
  }))
  default = []
}

variable "maintenance_window_start" {
  description = "Start time for maintenance window"
  type        = string
  default     = "03:00"
}

variable "release_channel" {
  description = "GKE release channel"
  type        = string
  default     = "REGULAR"
}

variable "workload_identity_namespaces" {
  description = "List of Kubernetes namespaces for Workload Identity"
  type        = list(string)
  default     = ["auth", "discovery", "ingestion", "sync", "default"]
}

# Cloud SQL Variables
variable "database_version" {
  description = "PostgreSQL version"
  type        = string
  default     = "POSTGRES_15"
}

variable "cloudsql_tier" {
  description = "Cloud SQL tier"
  type        = string
  default     = "db-custom-1-3840" # 1 vCPU, 3.75GB RAM (smaller for dev)
}

variable "cloudsql_availability_type" {
  description = "Cloud SQL availability type"
  type        = string
  default     = "ZONAL" # ZONAL for dev to save costs
}

variable "cloudsql_disk_size" {
  description = "Disk size in GB"
  type        = number
  default     = 50
}

variable "cloudsql_disk_autoresize_limit" {
  description = "Maximum disk size in GB for autoresize"
  type        = number
  default     = 200
}

variable "database_name" {
  description = "Default database name"
  type        = string
  default     = "media_gateway"
}

variable "additional_databases" {
  description = "List of additional databases"
  type        = list(string)
  default     = []
}

variable "db_user" {
  description = "Database user name"
  type        = string
  default     = "media_gateway_app"
}

variable "enable_cloudsql_replica" {
  description = "Enable read replica"
  type        = bool
  default     = false # Disabled for dev
}

# Memorystore Redis Variables
variable "redis_tier" {
  description = "Redis tier"
  type        = string
  default     = "BASIC" # BASIC for dev to save costs
}

variable "redis_memory_size_gb" {
  description = "Redis memory size in GB"
  type        = number
  default     = 1 # Smaller for dev
}

variable "redis_version" {
  description = "Redis version"
  type        = string
  default     = "REDIS_7_0"
}

variable "redis_reserved_ip_range" {
  description = "Reserved IP range for Redis"
  type        = string
  default     = "10.2.0.0/29"
}

variable "redis_replica_count" {
  description = "Number of read replicas"
  type        = number
  default     = 0 # No replicas for dev
}

variable "redis_persistence_mode" {
  description = "Persistence mode"
  type        = string
  default     = "DISABLED" # Disabled for dev
}

variable "redis_rdb_snapshot_period" {
  description = "RDB snapshot period"
  type        = string
  default     = "TWENTY_FOUR_HOURS"
}

variable "redis_transit_encryption_mode" {
  description = "Transit encryption mode"
  type        = string
  default     = "DISABLED" # Disabled for dev for easier debugging
}

variable "redis_auth_enabled" {
  description = "Enable Redis AUTH"
  type        = bool
  default     = false # Disabled for dev for easier debugging
}

# Cloud Armor Variables
variable "rate_limit_threshold_count" {
  description = "Number of requests allowed per interval"
  type        = number
  default     = 2000 # More permissive for dev
}

variable "rate_limit_threshold_interval" {
  description = "Interval in seconds for rate limiting"
  type        = number
  default     = 60
}

variable "ban_duration_sec" {
  description = "Duration in seconds to ban offending IPs"
  type        = number
  default     = 300 # 5 minutes for dev
}

variable "blocked_ip_ranges" {
  description = "List of IP ranges to block"
  type        = list(string)
  default     = []
}

variable "allowed_ip_ranges" {
  description = "List of trusted IP ranges to always allow"
  type        = list(string)
  default     = []
}

variable "enable_adaptive_protection" {
  description = "Enable adaptive protection"
  type        = bool
  default     = false # Disabled for dev
}

variable "log_level" {
  description = "Log level for Cloud Armor"
  type        = string
  default     = "VERBOSE" # More verbose for dev debugging
}
