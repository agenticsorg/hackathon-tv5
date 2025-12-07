# Cloud SQL Module - Media Gateway
# Provides PostgreSQL 15 database with HA and automated backups

resource "random_id" "db_name_suffix" {
  byte_length = 4
}

resource "google_sql_database_instance" "primary" {
  name             = "${var.project_name}-${var.environment}-db-${random_id.db_name_suffix.hex}"
  database_version = var.database_version
  region           = var.region
  project          = var.project_id

  settings {
    tier              = var.tier
    availability_type = var.availability_type
    disk_type         = "PD_SSD"
    disk_size         = var.disk_size
    disk_autoresize   = true
    disk_autoresize_limit = var.disk_autoresize_limit

    # Backup configuration
    backup_configuration {
      enabled                        = true
      start_time                     = "02:00"
      point_in_time_recovery_enabled = var.environment != "dev"
      transaction_log_retention_days = var.environment == "prod" ? 7 : 3
      backup_retention_settings {
        retained_backups = var.environment == "prod" ? 30 : 7
        retention_unit   = "COUNT"
      }
    }

    # IP configuration - private IP only
    ip_configuration {
      ipv4_enabled                                  = false
      private_network                               = var.network_self_link
      enable_private_path_for_google_cloud_services = true
      require_ssl                                   = true
    }

    # Maintenance window
    maintenance_window {
      day          = 7 # Sunday
      hour         = 3
      update_track = var.environment == "prod" ? "stable" : "canary"
    }

    # Insights configuration
    insights_config {
      query_insights_enabled  = true
      query_string_length     = 1024
      record_application_tags = true
      record_client_address   = true
    }

    # Database flags for PostgreSQL optimization
    dynamic "database_flags" {
      for_each = var.database_flags
      content {
        name  = database_flags.value.name
        value = database_flags.value.value
      }
    }

    # User labels
    user_labels = {
      environment = var.environment
      project     = var.project_name
      managed_by  = "terraform"
    }
  }

  deletion_protection = var.environment == "prod" ? true : false

  depends_on = [var.vpc_connection_dependency]
}

# Database
resource "google_sql_database" "database" {
  name     = var.database_name
  instance = google_sql_database_instance.primary.name
  project  = var.project_id

  charset   = "UTF8"
  collation = "en_US.UTF8"
}

# Additional databases for multi-tenancy or microservices
resource "google_sql_database" "additional_databases" {
  for_each = toset(var.additional_databases)

  name     = each.value
  instance = google_sql_database_instance.primary.name
  project  = var.project_id

  charset   = "UTF8"
  collation = "en_US.UTF8"
}

# Database user
resource "random_password" "db_password" {
  length  = 32
  special = true
}

resource "google_sql_user" "default" {
  name     = var.db_user
  instance = google_sql_database_instance.primary.name
  password = random_password.db_password.result
  project  = var.project_id
}

# Store password in Secret Manager
resource "google_secret_manager_secret" "db_password" {
  secret_id = "${var.project_name}-${var.environment}-db-password"
  project   = var.project_id

  replication {
    auto {}
  }

  labels = {
    environment = var.environment
    project     = var.project_name
  }
}

resource "google_secret_manager_secret_version" "db_password" {
  secret      = google_secret_manager_secret.db_password.id
  secret_data = random_password.db_password.result
}

# Read replica for production (optional)
resource "google_sql_database_instance" "replica" {
  count = var.enable_replica ? 1 : 0

  name                 = "${var.project_name}-${var.environment}-db-replica-${random_id.db_name_suffix.hex}"
  database_version     = var.database_version
  region               = var.replica_region
  project              = var.project_id
  master_instance_name = google_sql_database_instance.primary.name

  replica_configuration {
    failover_target = false
  }

  settings {
    tier              = var.tier
    availability_type = "ZONAL"
    disk_type         = "PD_SSD"
    disk_autoresize   = true

    ip_configuration {
      ipv4_enabled                                  = false
      private_network                               = var.network_self_link
      enable_private_path_for_google_cloud_services = true
      require_ssl                                   = true
    }

    insights_config {
      query_insights_enabled  = true
      query_string_length     = 1024
      record_application_tags = true
      record_client_address   = true
    }

    user_labels = {
      environment = var.environment
      project     = var.project_name
      managed_by  = "terraform"
      role        = "replica"
    }
  }

  deletion_protection = var.environment == "prod" ? true : false

  depends_on = [google_sql_database_instance.primary]
}
