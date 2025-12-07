# Cloud SQL Module Variables

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
  description = "Environment name (dev, staging, prod)"
  type        = string

  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be dev, staging, or prod."
  }
}

variable "region" {
  description = "GCP region"
  type        = string
  default     = "us-central1"
}

variable "replica_region" {
  description = "GCP region for read replica"
  type        = string
  default     = "us-east1"
}

variable "database_version" {
  description = "PostgreSQL version"
  type        = string
  default     = "POSTGRES_15"
}

variable "tier" {
  description = "Cloud SQL tier"
  type        = string
  default     = "db-custom-2-7680" # 2 vCPU, 7.5GB RAM
}

variable "availability_type" {
  description = "Availability type (ZONAL or REGIONAL)"
  type        = string
  default     = "REGIONAL"

  validation {
    condition     = contains(["ZONAL", "REGIONAL"], var.availability_type)
    error_message = "Availability type must be ZONAL or REGIONAL."
  }
}

variable "disk_size" {
  description = "Disk size in GB"
  type        = number
  default     = 100
}

variable "disk_autoresize_limit" {
  description = "Maximum disk size in GB for autoresize"
  type        = number
  default     = 500
}

variable "database_name" {
  description = "Default database name"
  type        = string
  default     = "media_gateway"
}

variable "additional_databases" {
  description = "List of additional databases to create"
  type        = list(string)
  default     = []
}

variable "db_user" {
  description = "Database user name"
  type        = string
  default     = "media_gateway_app"
}

variable "network_self_link" {
  description = "VPC network self link"
  type        = string
}

variable "vpc_connection_dependency" {
  description = "VPC connection dependency for proper resource ordering"
  type        = any
  default     = null
}

variable "enable_replica" {
  description = "Enable read replica"
  type        = bool
  default     = false
}

variable "database_flags" {
  description = "Database flags for PostgreSQL configuration"
  type = list(object({
    name  = string
    value = string
  }))
  default = [
    {
      name  = "max_connections"
      value = "200"
    },
    {
      name  = "shared_buffers"
      value = "262144" # 256MB in 8KB pages
    },
    {
      name  = "effective_cache_size"
      value = "786432" # 768MB in 8KB pages
    },
    {
      name  = "maintenance_work_mem"
      value = "65536" # 64MB in KB
    },
    {
      name  = "checkpoint_completion_target"
      value = "0.9"
    },
    {
      name  = "wal_buffers"
      value = "2048" # 16MB in 8KB pages
    },
    {
      name  = "default_statistics_target"
      value = "100"
    },
    {
      name  = "random_page_cost"
      value = "1.1"
    },
    {
      name  = "effective_io_concurrency"
      value = "200"
    },
    {
      name  = "work_mem"
      value = "5242" # ~5MB in KB
    },
    {
      name  = "min_wal_size"
      value = "1024" # 1GB in MB
    },
    {
      name  = "max_wal_size"
      value = "4096" # 4GB in MB
    }
  ]
}
