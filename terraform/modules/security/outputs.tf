# Cloud Armor Security Module Outputs

output "security_policy_id" {
  description = "Cloud Armor security policy ID"
  value       = google_compute_security_policy.policy.id
}

output "security_policy_name" {
  description = "Cloud Armor security policy name"
  value       = google_compute_security_policy.policy.name
}

output "security_policy_self_link" {
  description = "Cloud Armor security policy self link"
  value       = google_compute_security_policy.policy.self_link
}

output "security_policy_fingerprint" {
  description = "Cloud Armor security policy fingerprint"
  value       = google_compute_security_policy.policy.fingerprint
}
