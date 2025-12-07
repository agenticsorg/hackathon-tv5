# Memorystore Redis Module - Media Gateway
# Provides Redis 7 cache with HA and private service access

resource "google_redis_instance" "cache" {
  name               = "${var.project_name}-${var.environment}-redis"
  display_name       = "${var.project_name} ${var.environment} Redis Cache"
  tier               = var.tier
  memory_size_gb     = var.memory_size_gb
  region             = var.region
  project            = var.project_id
  redis_version      = var.redis_version
  reserved_ip_range  = var.reserved_ip_range
  connect_mode       = "PRIVATE_SERVICE_ACCESS"
  authorized_network = var.network_id

  # High availability configuration
  replica_count          = var.tier == "STANDARD_HA" ? var.replica_count : 0
  read_replicas_mode     = var.tier == "STANDARD_HA" && var.replica_count > 0 ? "READ_REPLICAS_ENABLED" : "READ_REPLICAS_DISABLED"

  # Redis configuration
  redis_configs = var.redis_configs

  # Persistence configuration
  persistence_config {
    persistence_mode    = var.persistence_mode
    rdb_snapshot_period = var.persistence_mode == "RDB" ? var.rdb_snapshot_period : null
  }

  # Maintenance policy
  maintenance_policy {
    weekly_maintenance_window {
      day = "SUNDAY"
      start_time {
        hours   = 3
        minutes = 0
      }
    }
  }

  # Transit encryption
  transit_encryption_mode = var.transit_encryption_mode
  auth_enabled            = var.auth_enabled

  # Labels
  labels = {
    environment = var.environment
    project     = var.project_name
    managed_by  = "terraform"
  }

  depends_on = [var.vpc_connection_dependency]
}

# Store Redis AUTH string in Secret Manager if auth is enabled
resource "google_secret_manager_secret" "redis_auth" {
  count = var.auth_enabled ? 1 : 0

  secret_id = "${var.project_name}-${var.environment}-redis-auth"
  project   = var.project_id

  replication {
    auto {}
  }

  labels = {
    environment = var.environment
    project     = var.project_name
  }
}

resource "google_secret_manager_secret_version" "redis_auth" {
  count = var.auth_enabled ? 1 : 0

  secret      = google_secret_manager_secret.redis_auth[0].id
  secret_data = google_redis_instance.cache.auth_string
}
