# Cloud Armor Security Module - Media Gateway
# Provides DDoS protection and WAF for application endpoints

# Cloud Armor security policy
resource "google_compute_security_policy" "policy" {
  name    = "${var.project_name}-${var.environment}-security-policy"
  project = var.project_id

  description = "Cloud Armor security policy for ${var.project_name} ${var.environment}"

  # Default rule - allow all traffic (will be customized by specific rules)
  rule {
    action   = "allow"
    priority = "2147483647"
    match {
      versioned_expr = "SRC_IPS_V1"
      config {
        src_ip_ranges = ["*"]
      }
    }
    description = "Default rule - allow all"
  }

  # Rate limiting rule - protect against DDoS
  rule {
    action   = "rate_based_ban"
    priority = "1000"
    match {
      versioned_expr = "SRC_IPS_V1"
      config {
        src_ip_ranges = ["*"]
      }
    }
    rate_limit_options {
      conform_action = "allow"
      exceed_action  = "deny(429)"
      enforce_on_key = "IP"
      ban_duration_sec = var.ban_duration_sec
      rate_limit_threshold {
        count        = var.rate_limit_threshold_count
        interval_sec = var.rate_limit_threshold_interval
      }
    }
    description = "Rate limiting rule - ${var.rate_limit_threshold_count} requests per ${var.rate_limit_threshold_interval}s"
  }

  # Block known bad IP ranges
  dynamic "rule" {
    for_each = var.blocked_ip_ranges
    content {
      action   = "deny(403)"
      priority = 2000 + rule.key
      match {
        versioned_expr = "SRC_IPS_V1"
        config {
          src_ip_ranges = [rule.value]
        }
      }
      description = "Block IP range: ${rule.value}"
    }
  }

  # Allow specific trusted IP ranges (e.g., office, CI/CD)
  dynamic "rule" {
    for_each = var.allowed_ip_ranges
    content {
      action   = "allow"
      priority = 3000 + rule.key
      match {
        versioned_expr = "SRC_IPS_V1"
        config {
          src_ip_ranges = [rule.value]
        }
      }
      description = "Allow trusted IP range: ${rule.value}"
    }
  }

  # SQL injection protection
  rule {
    action   = "deny(403)"
    priority = "4000"
    match {
      expr {
        expression = "evaluatePreconfiguredExpr('sqli-stable')"
      }
    }
    description = "SQL injection protection"
  }

  # XSS protection
  rule {
    action   = "deny(403)"
    priority = "4001"
    match {
      expr {
        expression = "evaluatePreconfiguredExpr('xss-stable')"
      }
    }
    description = "XSS protection"
  }

  # Local file inclusion protection
  rule {
    action   = "deny(403)"
    priority = "4002"
    match {
      expr {
        expression = "evaluatePreconfiguredExpr('lfi-stable')"
      }
    }
    description = "Local file inclusion protection"
  }

  # Remote code execution protection
  rule {
    action   = "deny(403)"
    priority = "4003"
    match {
      expr {
        expression = "evaluatePreconfiguredExpr('rce-stable')"
      }
    }
    description = "Remote code execution protection"
  }

  # Protocol attack protection
  rule {
    action   = "deny(403)"
    priority = "4004"
    match {
      expr {
        expression = "evaluatePreconfiguredExpr('protocolattack-stable')"
      }
    }
    description = "Protocol attack protection"
  }

  # Session fixation protection
  rule {
    action   = "deny(403)"
    priority = "4005"
    match {
      expr {
        expression = "evaluatePreconfiguredExpr('sessionfixation-stable')"
      }
    }
    description = "Session fixation protection"
  }

  # Adaptive protection (auto-detect and mitigate attacks)
  dynamic "adaptive_protection_config" {
    for_each = var.enable_adaptive_protection ? [1] : []
    content {
      layer_7_ddos_defense_config {
        enable          = true
        rule_visibility = "STANDARD"
      }
    }
  }

  # Advanced options
  advanced_options_config {
    json_parsing = "STANDARD"
    log_level    = var.log_level
  }
}

# Backend service example (for reference - actual backend services defined in GKE)
# This shows how to attach the security policy to a backend service
# resource "google_compute_backend_service" "example" {
#   name                  = "${var.project_name}-${var.environment}-backend"
#   protocol              = "HTTP"
#   port_name             = "http"
#   timeout_sec           = 30
#   security_policy       = google_compute_security_policy.policy.id
#
#   backend {
#     group = google_compute_instance_group.example.id
#   }
# }
