# GKE Autopilot Module - Media Gateway
# Provides managed Kubernetes cluster with Workload Identity

resource "google_container_cluster" "primary" {
  name     = "${var.project_name}-${var.environment}-gke"
  location = var.region
  project  = var.project_id

  # Autopilot mode
  enable_autopilot = true

  network    = var.network_name
  subnetwork = var.subnet_name

  # IP allocation policy for VPC-native cluster
  ip_allocation_policy {
    cluster_secondary_range_name  = var.pods_range_name
    services_secondary_range_name = var.services_range_name
  }

  # Private cluster configuration
  private_cluster_config {
    enable_private_nodes    = true
    enable_private_endpoint = false
    master_ipv4_cidr_block  = var.master_ipv4_cidr_block

    master_global_access_config {
      enabled = true
    }
  }

  # Master authorized networks
  master_authorized_networks_config {
    dynamic "cidr_blocks" {
      for_each = var.master_authorized_networks
      content {
        cidr_block   = cidr_blocks.value.cidr_block
        display_name = cidr_blocks.value.display_name
      }
    }
  }

  # Workload Identity configuration
  workload_identity_config {
    workload_pool = "${var.project_id}.svc.id.goog"
  }

  # Security and compliance
  binary_authorization {
    evaluation_mode = var.environment == "prod" ? "PROJECT_SINGLETON_POLICY_ENFORCE" : "DISABLED"
  }

  # Network policy
  network_policy {
    enabled  = true
    provider = "PROVIDER_UNSPECIFIED"
  }

  # Logging and monitoring
  logging_config {
    enable_components = ["SYSTEM_COMPONENTS", "WORKLOADS"]
  }

  monitoring_config {
    enable_components = ["SYSTEM_COMPONENTS"]

    managed_prometheus {
      enabled = true
    }
  }

  # Maintenance window
  maintenance_policy {
    daily_maintenance_window {
      start_time = var.maintenance_window_start
    }
  }

  # Release channel
  release_channel {
    channel = var.release_channel
  }

  # Addons configuration
  addons_config {
    http_load_balancing {
      disabled = false
    }

    horizontal_pod_autoscaling {
      disabled = false
    }

    network_policy_config {
      disabled = false
    }

    gcp_filestore_csi_driver_config {
      enabled = true
    }

    gcs_fuse_csi_driver_config {
      enabled = true
    }
  }

  # DNS configuration
  dns_config {
    cluster_dns        = "CLOUD_DNS"
    cluster_dns_scope  = "VPC_SCOPE"
    cluster_dns_domain = "${var.environment}.${var.project_name}.local"
  }

  # Resource labels
  resource_labels = {
    environment = var.environment
    project     = var.project_name
    managed_by  = "terraform"
  }

  # Depends on VPC connection
  depends_on = [var.vpc_connection_dependency]
}

# Workload Identity binding for each namespace
resource "google_service_account" "workload_identity" {
  for_each = toset(var.workload_identity_namespaces)

  account_id   = "${var.project_name}-${var.environment}-${each.key}-wi"
  display_name = "Workload Identity for ${each.key} namespace"
  project      = var.project_id
}

resource "google_service_account_iam_member" "workload_identity_binding" {
  for_each = toset(var.workload_identity_namespaces)

  service_account_id = google_service_account.workload_identity[each.key].name
  role               = "roles/iam.workloadIdentityUser"
  member             = "serviceAccount:${var.project_id}.svc.id.goog[${each.key}/${each.key}-sa]"
}
