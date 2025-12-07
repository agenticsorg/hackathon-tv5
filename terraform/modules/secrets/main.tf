# Secret Manager Module - Media Gateway
# Provides centralized secret management for application credentials

# Application secrets
resource "google_secret_manager_secret" "app_secrets" {
  for_each = var.app_secrets

  secret_id = "${var.project_name}-${var.environment}-${each.key}"
  project   = var.project_id

  replication {
    auto {}
  }

  labels = {
    environment = var.environment
    project     = var.project_name
    managed_by  = "terraform"
    category    = lookup(each.value, "category", "general")
  }
}

# JWT signing key
resource "random_password" "jwt_secret" {
  length  = 64
  special = true
}

resource "google_secret_manager_secret" "jwt_secret" {
  secret_id = "${var.project_name}-${var.environment}-jwt-secret"
  project   = var.project_id

  replication {
    auto {}
  }

  labels = {
    environment = var.environment
    project     = var.project_name
    managed_by  = "terraform"
    category    = "authentication"
  }
}

resource "google_secret_manager_secret_version" "jwt_secret" {
  secret      = google_secret_manager_secret.jwt_secret.id
  secret_data = random_password.jwt_secret.result
}

# Encryption key for sensitive data
resource "random_password" "encryption_key" {
  length  = 32
  special = false
}

resource "google_secret_manager_secret" "encryption_key" {
  secret_id = "${var.project_name}-${var.environment}-encryption-key"
  project   = var.project_id

  replication {
    auto {}
  }

  labels = {
    environment = var.environment
    project     = var.project_name
    managed_by  = "terraform"
    category    = "encryption"
  }
}

resource "google_secret_manager_secret_version" "encryption_key" {
  secret      = google_secret_manager_secret.encryption_key.id
  secret_data = random_password.encryption_key.result
}

# OAuth provider secrets (placeholders - update with actual values)
resource "google_secret_manager_secret" "oauth_google_client_id" {
  secret_id = "${var.project_name}-${var.environment}-oauth-google-client-id"
  project   = var.project_id

  replication {
    auto {}
  }

  labels = {
    environment = var.environment
    project     = var.project_name
    managed_by  = "terraform"
    category    = "oauth"
    provider    = "google"
  }
}

resource "google_secret_manager_secret" "oauth_google_client_secret" {
  secret_id = "${var.project_name}-${var.environment}-oauth-google-client-secret"
  project   = var.project_id

  replication {
    auto {}
  }

  labels = {
    environment = var.environment
    project     = var.project_name
    managed_by  = "terraform"
    category    = "oauth"
    provider    = "google"
  }
}

resource "google_secret_manager_secret" "oauth_apple_client_id" {
  secret_id = "${var.project_name}-${var.environment}-oauth-apple-client-id"
  project   = var.project_id

  replication {
    auto {}
  }

  labels = {
    environment = var.environment
    project     = var.project_name
    managed_by  = "terraform"
    category    = "oauth"
    provider    = "apple"
  }
}

resource "google_secret_manager_secret" "oauth_apple_client_secret" {
  secret_id = "${var.project_name}-${var.environment}-oauth-apple-client-secret"
  project   = var.project_id

  replication {
    auto {}
  }

  labels = {
    environment = var.environment
    project     = var.project_name
    managed_by  = "terraform"
    category    = "oauth"
    provider    = "apple"
  }
}

# Email service credentials (e.g., SendGrid, AWS SES)
resource "google_secret_manager_secret" "email_api_key" {
  secret_id = "${var.project_name}-${var.environment}-email-api-key"
  project   = var.project_id

  replication {
    auto {}
  }

  labels = {
    environment = var.environment
    project     = var.project_name
    managed_by  = "terraform"
    category    = "email"
  }
}

# IAM bindings for Workload Identity to access secrets
resource "google_secret_manager_secret_iam_member" "workload_identity_access" {
  for_each = var.workload_identity_service_accounts

  secret_id = google_secret_manager_secret.jwt_secret.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${each.value}"
}

resource "google_secret_manager_secret_iam_member" "workload_identity_encryption" {
  for_each = var.workload_identity_service_accounts

  secret_id = google_secret_manager_secret.encryption_key.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${each.value}"
}

# Grant access to OAuth secrets
resource "google_secret_manager_secret_iam_member" "workload_identity_oauth_google_id" {
  for_each = var.workload_identity_service_accounts

  secret_id = google_secret_manager_secret.oauth_google_client_id.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${each.value}"
}

resource "google_secret_manager_secret_iam_member" "workload_identity_oauth_google_secret" {
  for_each = var.workload_identity_service_accounts

  secret_id = google_secret_manager_secret.oauth_google_client_secret.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${each.value}"
}

resource "google_secret_manager_secret_iam_member" "workload_identity_email" {
  for_each = var.workload_identity_service_accounts

  secret_id = google_secret_manager_secret.email_api_key.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${each.value}"
}
