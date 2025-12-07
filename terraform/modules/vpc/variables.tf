# VPC Module Variables

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
  description = "CIDR range for private subnet (Cloud SQL, Memorystore)"
  type        = string
  default     = "10.1.0.0/20"
}
