# Staging Environment Outputs

# VPC Outputs
output "vpc_network_name" {
  description = "VPC network name"
  value       = module.vpc.network_name
}

output "vpc_network_id" {
  description = "VPC network ID"
  value       = module.vpc.network_id
}

# GKE Outputs
output "gke_cluster_name" {
  description = "GKE cluster name"
  value       = module.gke.cluster_name
}

output "gke_cluster_endpoint" {
  description = "GKE cluster endpoint"
  value       = module.gke.cluster_endpoint
  sensitive   = true
}

output "gke_cluster_ca_certificate" {
  description = "GKE cluster CA certificate"
  value       = module.gke.cluster_ca_certificate
  sensitive   = true
}

output "gke_workload_identity_pool" {
  description = "Workload Identity pool"
  value       = module.gke.workload_identity_pool
}

# Cloud SQL Outputs
output "cloudsql_instance_name" {
  description = "Cloud SQL instance name"
  value       = module.cloudsql.instance_name
}

output "cloudsql_private_ip" {
  description = "Cloud SQL private IP address"
  value       = module.cloudsql.private_ip_address
}

output "cloudsql_connection_name" {
  description = "Cloud SQL connection name"
  value       = module.cloudsql.instance_connection_name
}

output "cloudsql_database_name" {
  description = "Database name"
  value       = module.cloudsql.database_name
}

output "cloudsql_database_user" {
  description = "Database user"
  value       = module.cloudsql.database_user
}

# Memorystore Outputs
output "redis_instance_name" {
  description = "Redis instance name"
  value       = module.memorystore.instance_name
}

output "redis_host" {
  description = "Redis host"
  value       = module.memorystore.host
}

output "redis_port" {
  description = "Redis port"
  value       = module.memorystore.port
}

# Secret Manager Outputs
output "jwt_secret_id" {
  description = "JWT secret ID"
  value       = module.secrets.jwt_secret_id
}

output "encryption_key_secret_id" {
  description = "Encryption key secret ID"
  value       = module.secrets.encryption_key_secret_id
}

# Security Outputs
output "security_policy_name" {
  description = "Cloud Armor security policy name"
  value       = module.security.security_policy_name
}

# Connection Information
output "connection_info" {
  description = "Connection information for all services"
  value = {
    gke = {
      cluster_name = module.gke.cluster_name
      endpoint     = module.gke.cluster_endpoint
      region       = var.region
    }
    database = {
      host         = module.cloudsql.private_ip_address
      port         = 5432
      database     = module.cloudsql.database_name
      user         = module.cloudsql.database_user
    }
    redis = {
      host = module.memorystore.host
      port = module.memorystore.port
    }
  }
  sensitive = true
}
