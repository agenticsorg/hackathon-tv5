# Secret Manager Module Variables

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

variable "app_secrets" {
  description = "Map of application secrets to create"
  type = map(object({
    category = optional(string, "general")
  }))
  default = {}
}

variable "workload_identity_service_accounts" {
  description = "Map of Workload Identity service account emails to grant access"
  type        = map(string)
  default     = {}
}
