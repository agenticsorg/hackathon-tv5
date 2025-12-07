# GKE Module Variables

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

variable "network_name" {
  description = "VPC network name"
  type        = string
}

variable "subnet_name" {
  description = "Subnet name for GKE nodes"
  type        = string
}

variable "pods_range_name" {
  description = "Name of secondary IP range for pods"
  type        = string
  default     = "gke-pods"
}

variable "services_range_name" {
  description = "Name of secondary IP range for services"
  type        = string
  default     = "gke-services"
}

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
  description = "Start time for maintenance window (HH:MM format)"
  type        = string
  default     = "03:00"
}

variable "release_channel" {
  description = "GKE release channel (RAPID, REGULAR, STABLE)"
  type        = string
  default     = "REGULAR"

  validation {
    condition     = contains(["RAPID", "REGULAR", "STABLE"], var.release_channel)
    error_message = "Release channel must be RAPID, REGULAR, or STABLE."
  }
}

variable "workload_identity_namespaces" {
  description = "List of Kubernetes namespaces for Workload Identity"
  type        = list(string)
  default     = ["auth", "discovery", "ingestion", "sync", "default"]
}

variable "vpc_connection_dependency" {
  description = "VPC connection dependency for proper resource ordering"
  type        = any
  default     = null
}
