# Secret Manager Module Outputs

output "jwt_secret_id" {
  description = "JWT secret ID"
  value       = google_secret_manager_secret.jwt_secret.secret_id
}

output "jwt_secret_name" {
  description = "JWT secret full name"
  value       = google_secret_manager_secret.jwt_secret.name
}

output "encryption_key_secret_id" {
  description = "Encryption key secret ID"
  value       = google_secret_manager_secret.encryption_key.secret_id
}

output "encryption_key_secret_name" {
  description = "Encryption key secret full name"
  value       = google_secret_manager_secret.encryption_key.name
}

output "oauth_google_client_id_secret_id" {
  description = "OAuth Google client ID secret ID"
  value       = google_secret_manager_secret.oauth_google_client_id.secret_id
}

output "oauth_google_client_secret_secret_id" {
  description = "OAuth Google client secret secret ID"
  value       = google_secret_manager_secret.oauth_google_client_secret.secret_id
}

output "oauth_apple_client_id_secret_id" {
  description = "OAuth Apple client ID secret ID"
  value       = google_secret_manager_secret.oauth_apple_client_id.secret_id
}

output "oauth_apple_client_secret_secret_id" {
  description = "OAuth Apple client secret secret ID"
  value       = google_secret_manager_secret.oauth_apple_client_secret.secret_id
}

output "email_api_key_secret_id" {
  description = "Email API key secret ID"
  value       = google_secret_manager_secret.email_api_key.secret_id
}

output "app_secret_ids" {
  description = "Map of application secret IDs"
  value = {
    for k, v in google_secret_manager_secret.app_secrets :
    k => v.secret_id
  }
}

output "all_secret_ids" {
  description = "List of all secret IDs"
  value = concat(
    [
      google_secret_manager_secret.jwt_secret.secret_id,
      google_secret_manager_secret.encryption_key.secret_id,
      google_secret_manager_secret.oauth_google_client_id.secret_id,
      google_secret_manager_secret.oauth_google_client_secret.secret_id,
      google_secret_manager_secret.oauth_apple_client_id.secret_id,
      google_secret_manager_secret.oauth_apple_client_secret.secret_id,
      google_secret_manager_secret.email_api_key.secret_id
    ],
    [for s in google_secret_manager_secret.app_secrets : s.secret_id]
  )
}
