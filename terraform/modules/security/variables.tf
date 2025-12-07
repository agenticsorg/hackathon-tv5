# Cloud Armor Security Module Variables

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

variable "rate_limit_threshold_count" {
  description = "Number of requests allowed per interval"
  type        = number
  default     = 1000
}

variable "rate_limit_threshold_interval" {
  description = "Interval in seconds for rate limiting"
  type        = number
  default     = 60
}

variable "ban_duration_sec" {
  description = "Duration in seconds to ban offending IPs"
  type        = number
  default     = 600 # 10 minutes
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
  description = "Enable adaptive protection for Layer 7 DDoS defense"
  type        = bool
  default     = true
}

variable "log_level" {
  description = "Log level for Cloud Armor (NORMAL or VERBOSE)"
  type        = string
  default     = "NORMAL"

  validation {
    condition     = contains(["NORMAL", "VERBOSE"], var.log_level)
    error_message = "Log level must be NORMAL or VERBOSE."
  }
}
