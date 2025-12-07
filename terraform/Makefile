# Terraform Makefile for Media Gateway
# Simplifies common Terraform operations

.PHONY: help init plan apply destroy fmt validate clean dev staging prod

# Default environment
ENV ?= dev

# Terraform commands
TERRAFORM := terraform
TF_DIR := environments/$(ENV)

help: ## Show this help message
	@echo "Media Gateway Terraform Commands"
	@echo ""
	@echo "Usage: make [target] ENV=[dev|staging|prod]"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

init: ## Initialize Terraform
	@echo "Initializing Terraform for $(ENV)..."
	cd $(TF_DIR) && $(TERRAFORM) init

plan: ## Generate and show execution plan
	@echo "Planning Terraform changes for $(ENV)..."
	cd $(TF_DIR) && $(TERRAFORM) plan -out=tfplan

apply: ## Apply Terraform changes
	@echo "Applying Terraform changes for $(ENV)..."
	cd $(TF_DIR) && $(TERRAFORM) apply tfplan

apply-auto: ## Apply without confirmation (use with caution)
	@echo "Auto-applying Terraform changes for $(ENV)..."
	cd $(TF_DIR) && $(TERRAFORM) apply -auto-approve

destroy: ## Destroy Terraform-managed infrastructure
	@echo "WARNING: This will destroy all resources in $(ENV)!"
	@read -p "Are you sure? Type 'yes' to confirm: " confirm && [ "$$confirm" = "yes" ]
	cd $(TF_DIR) && $(TERRAFORM) destroy

fmt: ## Format Terraform files
	@echo "Formatting Terraform files..."
	$(TERRAFORM) fmt -recursive .

validate: ## Validate Terraform configuration
	@echo "Validating Terraform configuration for $(ENV)..."
	cd $(TF_DIR) && $(TERRAFORM) validate

clean: ## Clean Terraform files
	@echo "Cleaning Terraform files for $(ENV)..."
	rm -rf $(TF_DIR)/.terraform
	rm -f $(TF_DIR)/.terraform.lock.hcl
	rm -f $(TF_DIR)/tfplan
	rm -f $(TF_DIR)/*.tfstate*

output: ## Show Terraform outputs
	@echo "Terraform outputs for $(ENV):"
	cd $(TF_DIR) && $(TERRAFORM) output

output-json: ## Show Terraform outputs in JSON
	@echo "Terraform outputs (JSON) for $(ENV):"
	cd $(TF_DIR) && $(TERRAFORM) output -json

# Environment shortcuts
dev: ## Set environment to dev
	@$(MAKE) ENV=dev

staging: ## Set environment to staging
	@$(MAKE) ENV=staging

prod: ## Set environment to prod
	@$(MAKE) ENV=prod

# GKE connection
connect-gke: ## Connect kubectl to GKE cluster
	@echo "Connecting to GKE cluster in $(ENV)..."
	@CLUSTER=$$(cd $(TF_DIR) && $(TERRAFORM) output -raw gke_cluster_name 2>/dev/null) && \
	PROJECT=$$(cd $(TF_DIR) && $(TERRAFORM) output -raw project_id 2>/dev/null || echo "YOUR_PROJECT_ID") && \
	gcloud container clusters get-credentials $$CLUSTER --region us-central1 --project $$PROJECT

# Cost estimation (requires infracost)
cost: ## Estimate infrastructure costs
	@echo "Estimating costs for $(ENV)..."
	@command -v infracost >/dev/null 2>&1 || { echo "infracost not installed. Install from https://www.infracost.io/"; exit 1; }
	cd $(TF_DIR) && infracost breakdown --path .

# Security scan (requires tfsec)
security: ## Run security scan
	@echo "Running security scan..."
	@command -v tfsec >/dev/null 2>&1 || { echo "tfsec not installed. Install from https://github.com/aquasecurity/tfsec"; exit 1; }
	tfsec .

# Module documentation
docs: ## Generate module documentation
	@echo "Generating module documentation..."
	@command -v terraform-docs >/dev/null 2>&1 || { echo "terraform-docs not installed. Install from https://terraform-docs.io/"; exit 1; }
	@for module in modules/*; do \
		echo "Generating docs for $$module..."; \
		terraform-docs markdown table $$module > $$module/README.md; \
	done

# All environments init
init-all: ## Initialize all environments
	@$(MAKE) init ENV=dev
	@$(MAKE) init ENV=staging
	@$(MAKE) init ENV=prod

# All environments plan
plan-all: ## Plan all environments
	@$(MAKE) plan ENV=dev
	@$(MAKE) plan ENV=staging
	@$(MAKE) plan ENV=prod

# Upgrade providers
upgrade: ## Upgrade Terraform providers
	@echo "Upgrading Terraform providers for $(ENV)..."
	cd $(TF_DIR) && $(TERRAFORM) init -upgrade
