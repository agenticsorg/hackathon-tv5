# Cloud SQL Module Outputs

output "instance_name" {
  description = "Cloud SQL instance name"
  value       = google_sql_database_instance.primary.name
}

output "instance_connection_name" {
  description = "Cloud SQL instance connection name"
  value       = google_sql_database_instance.primary.connection_name
}

output "instance_self_link" {
  description = "Cloud SQL instance self link"
  value       = google_sql_database_instance.primary.self_link
}

output "private_ip_address" {
  description = "Private IP address of the Cloud SQL instance"
  value       = google_sql_database_instance.primary.private_ip_address
}

output "database_name" {
  description = "Database name"
  value       = google_sql_database.database.name
}

output "database_user" {
  description = "Database user name"
  value       = google_sql_user.default.name
}

output "database_password_secret_id" {
  description = "Secret Manager secret ID for database password"
  value       = google_secret_manager_secret.db_password.secret_id
}

output "database_password_secret_version" {
  description = "Secret Manager secret version for database password"
  value       = google_secret_manager_secret_version.db_password.name
  sensitive   = true
}

output "replica_instance_name" {
  description = "Cloud SQL replica instance name"
  value       = var.enable_replica ? google_sql_database_instance.replica[0].name : null
}

output "replica_connection_name" {
  description = "Cloud SQL replica instance connection name"
  value       = var.enable_replica ? google_sql_database_instance.replica[0].connection_name : null
}

output "replica_private_ip_address" {
  description = "Private IP address of the Cloud SQL replica instance"
  value       = var.enable_replica ? google_sql_database_instance.replica[0].private_ip_address : null
}

output "connection_string" {
  description = "PostgreSQL connection string (without password)"
  value       = "postgresql://${google_sql_user.default.name}@${google_sql_database_instance.primary.private_ip_address}:5432/${google_sql_database.database.name}"
  sensitive   = true
}
