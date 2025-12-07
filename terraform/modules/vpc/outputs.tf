# VPC Module Outputs

output "network_id" {
  description = "VPC network ID"
  value       = google_compute_network.vpc.id
}

output "network_name" {
  description = "VPC network name"
  value       = google_compute_network.vpc.name
}

output "network_self_link" {
  description = "VPC network self link"
  value       = google_compute_network.vpc.self_link
}

output "gke_subnet_name" {
  description = "GKE subnet name"
  value       = google_compute_subnetwork.gke_subnet.name
}

output "gke_subnet_self_link" {
  description = "GKE subnet self link"
  value       = google_compute_subnetwork.gke_subnet.self_link
}

output "private_subnet_name" {
  description = "Private subnet name"
  value       = google_compute_subnetwork.private_subnet.name
}

output "private_subnet_self_link" {
  description = "Private subnet self link"
  value       = google_compute_subnetwork.private_subnet.self_link
}

output "pods_range_name" {
  description = "Name of the pods secondary IP range"
  value       = "gke-pods"
}

output "services_range_name" {
  description = "Name of the services secondary IP range"
  value       = "gke-services"
}

output "router_name" {
  description = "Cloud Router name"
  value       = google_compute_router.router.name
}

output "nat_name" {
  description = "Cloud NAT name"
  value       = google_compute_router_nat.nat.name
}

output "private_vpc_connection" {
  description = "Private VPC connection for services"
  value       = google_service_networking_connection.private_vpc_connection.network
}
