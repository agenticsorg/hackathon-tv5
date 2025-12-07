# GKE Module Outputs

output "cluster_id" {
  description = "GKE cluster ID"
  value       = google_container_cluster.primary.id
}

output "cluster_name" {
  description = "GKE cluster name"
  value       = google_container_cluster.primary.name
}

output "cluster_endpoint" {
  description = "GKE cluster endpoint"
  value       = google_container_cluster.primary.endpoint
  sensitive   = true
}

output "cluster_ca_certificate" {
  description = "GKE cluster CA certificate"
  value       = google_container_cluster.primary.master_auth[0].cluster_ca_certificate
  sensitive   = true
}

output "workload_identity_pool" {
  description = "Workload Identity pool"
  value       = "${var.project_id}.svc.id.goog"
}

output "workload_identity_service_accounts" {
  description = "Map of namespace to Workload Identity service account emails"
  value = {
    for ns in var.workload_identity_namespaces :
    ns => google_service_account.workload_identity[ns].email
  }
}

output "cluster_version" {
  description = "GKE cluster version"
  value       = google_container_cluster.primary.master_version
}

output "cluster_location" {
  description = "GKE cluster location"
  value       = google_container_cluster.primary.location
}
