# Memorystore Redis Module Variables

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

variable "tier" {
  description = "Redis tier (BASIC or STANDARD_HA)"
  type        = string
  default     = "STANDARD_HA"

  validation {
    condition     = contains(["BASIC", "STANDARD_HA"], var.tier)
    error_message = "Tier must be BASIC or STANDARD_HA."
  }
}

variable "memory_size_gb" {
  description = "Redis memory size in GB"
  type        = number
  default     = 6

  validation {
    condition     = var.memory_size_gb >= 1 && var.memory_size_gb <= 300
    error_message = "Memory size must be between 1 and 300 GB."
  }
}

variable "redis_version" {
  description = "Redis version"
  type        = string
  default     = "REDIS_7_0"
}

variable "reserved_ip_range" {
  description = "Reserved IP range for Redis (CIDR notation)"
  type        = string
  default     = "10.2.0.0/29"
}

variable "network_id" {
  description = "VPC network ID for authorized network"
  type        = string
}

variable "vpc_connection_dependency" {
  description = "VPC connection dependency for proper resource ordering"
  type        = any
  default     = null
}

variable "replica_count" {
  description = "Number of read replicas (0-5)"
  type        = number
  default     = 1

  validation {
    condition     = var.replica_count >= 0 && var.replica_count <= 5
    error_message = "Replica count must be between 0 and 5."
  }
}

variable "redis_configs" {
  description = "Redis configuration parameters"
  type        = map(string)
  default = {
    maxmemory-policy           = "allkeys-lru"
    notify-keyspace-events     = "Ex"
    timeout                    = "300"
    tcp-keepalive              = "60"
    maxmemory-samples          = "5"
    lfu-log-factor             = "10"
    lfu-decay-time             = "1"
    activedefrag               = "yes"
    active-defrag-ignore-bytes = "104857600" # 100MB
  }
}

variable "persistence_mode" {
  description = "Persistence mode (DISABLED or RDB)"
  type        = string
  default     = "RDB"

  validation {
    condition     = contains(["DISABLED", "RDB"], var.persistence_mode)
    error_message = "Persistence mode must be DISABLED or RDB."
  }
}

variable "rdb_snapshot_period" {
  description = "RDB snapshot period (ONE_HOUR, SIX_HOURS, TWELVE_HOURS, TWENTY_FOUR_HOURS)"
  type        = string
  default     = "TWELVE_HOURS"

  validation {
    condition     = contains(["ONE_HOUR", "SIX_HOURS", "TWELVE_HOURS", "TWENTY_FOUR_HOURS"], var.rdb_snapshot_period)
    error_message = "RDB snapshot period must be ONE_HOUR, SIX_HOURS, TWELVE_HOURS, or TWENTY_FOUR_HOURS."
  }
}

variable "transit_encryption_mode" {
  description = "Transit encryption mode (DISABLED or SERVER_AUTHENTICATION)"
  type        = string
  default     = "SERVER_AUTHENTICATION"

  validation {
    condition     = contains(["DISABLED", "SERVER_AUTHENTICATION"], var.transit_encryption_mode)
    error_message = "Transit encryption mode must be DISABLED or SERVER_AUTHENTICATION."
  }
}

variable "auth_enabled" {
  description = "Enable Redis AUTH"
  type        = bool
  default     = true
}
