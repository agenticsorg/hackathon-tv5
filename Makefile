# Media Gateway Development Makefile
# Common targets for development workflow

.PHONY: help setup teardown build test test-integration clean format lint check run docker-up docker-down docker-logs migrate seed

# Default target
.DEFAULT_GOAL := help

# Database connection string
DATABASE_URL ?= postgresql://mediagateway:localdev123@localhost:5432/media_gateway

# ============================================================================
# Help
# ============================================================================

help: ## Show this help message
	@echo 'Media Gateway Development Makefile'
	@echo ''
	@echo 'Usage:'
	@echo '  make <target>'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ============================================================================
# Development Environment
# ============================================================================

setup: ## Setup development environment (Docker, DB, migrations, seed data)
	@echo "Setting up development environment..."
	@chmod +x scripts/dev-setup.sh
	@./scripts/dev-setup.sh

teardown: ## Teardown development environment
	@echo "Tearing down development environment..."
	@chmod +x scripts/dev-teardown.sh
	@./scripts/dev-teardown.sh

teardown-all: ## Teardown environment and remove all volumes and images
	@echo "Tearing down development environment (removing all data)..."
	@chmod +x scripts/dev-teardown.sh
	@./scripts/dev-teardown.sh --all

# ============================================================================
# Building
# ============================================================================

build: ## Build all Rust crates
	@echo "Building Media Gateway workspace..."
	cargo build

build-release: ## Build all crates in release mode
	@echo "Building Media Gateway workspace (release)..."
	cargo build --release

build-sona: ## Build SONA service
	@echo "Building SONA service..."
	cargo build -p media-gateway-sona

build-auth: ## Build Auth service
	@echo "Building Auth service..."
	cargo build -p media-gateway-auth

build-discovery: ## Build Discovery service
	@echo "Building Discovery service..."
	cargo build -p media-gateway-discovery

# ============================================================================
# Testing
# ============================================================================

test: ## Run all unit tests
	@echo "Running unit tests..."
	cargo test --lib --bins

test-integration: ## Run integration tests (requires running database)
	@echo "Running integration tests..."
	@echo "NOTE: Ensure database is running (make docker-up)"
	DATABASE_URL=$(DATABASE_URL) cargo test --test '*' -- --test-threads=1

test-sona: ## Run SONA tests
	@echo "Running SONA tests..."
	cargo test -p media-gateway-sona

test-auth: ## Run Auth tests
	@echo "Running Auth tests..."
	cargo test -p media-gateway-auth

test-all: test test-integration ## Run all tests (unit + integration)

test-coverage: ## Generate test coverage report
	@echo "Generating test coverage..."
	cargo tarpaulin --out Html --output-dir coverage

# ============================================================================
# Code Quality
# ============================================================================

format: ## Format code using rustfmt
	@echo "Formatting code..."
	cargo fmt --all

format-check: ## Check code formatting
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

lint: ## Run clippy linter
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

check: format-check lint ## Run all code quality checks
	@echo "Running cargo check..."
	cargo check --all-targets --all-features

# ============================================================================
# Database
# ============================================================================

migrate: ## Run database migrations
	@echo "Running database migrations..."
	@chmod +x scripts/run-migrations.sh
	@./scripts/run-migrations.sh

seed: ## Load seed data into database
	@echo "Loading seed data..."
	@docker-compose exec -T postgres psql -U mediagateway -d media_gateway -f - < scripts/seed-data.sql

sqlx-prepare: ## Prepare SQLx metadata for offline builds
	@echo "Preparing SQLx metadata..."
	@export DATABASE_URL=$(DATABASE_URL); \
	for crate in crates/auth crates/discovery crates/sona crates/sync crates/ingestion crates/playback; do \
		if [ -d "$$crate" ]; then \
			echo "  Preparing $$crate..."; \
			(cd "$$crate" && cargo sqlx prepare) || true; \
		fi; \
	done

db-reset: ## Reset database (teardown + setup)
	@echo "Resetting database..."
	@make teardown-all
	@make setup

# ============================================================================
# Docker
# ============================================================================

docker-up: ## Start Docker Compose services
	@echo "Starting Docker Compose services..."
	docker-compose up -d

docker-down: ## Stop Docker Compose services
	@echo "Stopping Docker Compose services..."
	docker-compose down

docker-logs: ## Show Docker Compose logs
	docker-compose logs -f

docker-logs-sona: ## Show SONA service logs
	docker-compose logs -f sona

docker-logs-auth: ## Show Auth service logs
	docker-compose logs -f auth

docker-logs-postgres: ## Show PostgreSQL logs
	docker-compose logs -f postgres

docker-ps: ## Show running Docker containers
	docker-compose ps

docker-restart: ## Restart Docker Compose services
	@echo "Restarting Docker Compose services..."
	docker-compose restart

# ============================================================================
# Running Services
# ============================================================================

run-sona: ## Run SONA service locally
	@echo "Running SONA service..."
	DATABASE_URL=$(DATABASE_URL) cargo run -p media-gateway-sona

run-auth: ## Run Auth service locally
	@echo "Running Auth service..."
	DATABASE_URL=$(DATABASE_URL) cargo run -p media-gateway-auth

run-discovery: ## Run Discovery service locally
	@echo "Running Discovery service..."
	DATABASE_URL=$(DATABASE_URL) cargo run -p media-gateway-discovery

# ============================================================================
# Cleanup
# ============================================================================

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	cargo clean

clean-all: clean teardown-all ## Clean everything (build artifacts + Docker volumes)

# ============================================================================
# Development Workflow
# ============================================================================

dev: setup docker-up ## Complete dev setup (alias for setup + docker-up)
	@echo "Development environment ready!"

quick-test: format lint test ## Quick test workflow (format + lint + unit tests)

ci: check test ## CI workflow (check + test)

# ============================================================================
# Documentation
# ============================================================================

docs: ## Generate and open project documentation
	@echo "Generating documentation..."
	cargo doc --no-deps --open

docs-all: ## Generate documentation with dependencies
	@echo "Generating documentation with dependencies..."
	cargo doc --open

# ============================================================================
# Utility
# ============================================================================

watch: ## Watch for file changes and rebuild
	@echo "Watching for changes..."
	cargo watch -x build

watch-test: ## Watch for file changes and run tests
	@echo "Watching for changes and running tests..."
	cargo watch -x test

install-tools: ## Install development tools
	@echo "Installing development tools..."
	cargo install cargo-watch
	cargo install cargo-tarpaulin
	cargo install sqlx-cli --no-default-features --features postgres

status: ## Show development environment status
	@echo "=== Development Environment Status ==="
	@echo ""
	@echo "Docker Services:"
	@docker-compose ps || echo "Docker Compose not running"
	@echo ""
	@echo "Database Connection:"
	@psql $(DATABASE_URL) -c "SELECT version();" 2>/dev/null || echo "Cannot connect to database"
	@echo ""
	@echo "Rust Toolchain:"
	@rustc --version
	@cargo --version
