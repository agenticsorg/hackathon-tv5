# Memorystore Redis Module Outputs

output "instance_id" {
  description = "Redis instance ID"
  value       = google_redis_instance.cache.id
}

output "instance_name" {
  description = "Redis instance name"
  value       = google_redis_instance.cache.name
}

output "host" {
  description = "Redis host IP address"
  value       = google_redis_instance.cache.host
}

output "port" {
  description = "Redis port"
  value       = google_redis_instance.cache.port
}

output "read_endpoint" {
  description = "Redis read endpoint (for HA tier with read replicas)"
  value       = google_redis_instance.cache.read_endpoint
}

output "read_endpoint_port" {
  description = "Redis read endpoint port"
  value       = google_redis_instance.cache.read_endpoint_port
}

output "current_location_id" {
  description = "Current location ID of the Redis instance"
  value       = google_redis_instance.cache.current_location_id
}

output "persistence_iam_identity" {
  description = "IAM identity for persistence"
  value       = google_redis_instance.cache.persistence_iam_identity
}

output "auth_string" {
  description = "Redis AUTH string"
  value       = google_redis_instance.cache.auth_string
  sensitive   = true
}

output "auth_secret_id" {
  description = "Secret Manager secret ID for Redis AUTH string"
  value       = var.auth_enabled ? google_secret_manager_secret.redis_auth[0].secret_id : null
}

output "connection_string" {
  description = "Redis connection string"
  value       = "redis://${google_redis_instance.cache.host}:${google_redis_instance.cache.port}"
  sensitive   = true
}

output "read_connection_string" {
  description = "Redis read replica connection string (if available)"
  value       = google_redis_instance.cache.read_endpoint != "" ? "redis://${google_redis_instance.cache.read_endpoint}:${google_redis_instance.cache.read_endpoint_port}" : null
  sensitive   = true
}
