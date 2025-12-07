#!/bin/bash
# Media Gateway Local Development Setup Script
# Run: chmod +x scripts/dev-setup.sh && ./scripts/dev-setup.sh

set -e

echo "=== Media Gateway Development Setup ==="

# Check dependencies
command -v docker >/dev/null 2>&1 || { echo "Docker is required but not installed. Aborting."; exit 1; }
command -v docker-compose >/dev/null 2>&1 || { echo "Docker Compose is required but not installed. Aborting."; exit 1; }

# Create .env if it doesn't exist
if [ ! -f .env ]; then
    echo "Creating .env from .env.example..."
    cp .env.example .env
    echo "Please update .env with your API keys and configuration."
fi

# Start infrastructure services only first
echo "Starting infrastructure services (postgres, redis, qdrant)..."
docker-compose up -d postgres redis qdrant

# Wait for services to be healthy
echo "Waiting for infrastructure to be ready..."

# Wait for PostgreSQL
echo -n "Waiting for PostgreSQL..."
until docker-compose exec -T postgres pg_isready -U mediagateway -d media_gateway >/dev/null 2>&1; do
    echo -n "."
    sleep 2
done
echo " Ready!"

# Wait for Redis
echo -n "Waiting for Redis..."
until docker-compose exec -T redis redis-cli ping >/dev/null 2>&1; do
    echo -n "."
    sleep 2
done
echo " Ready!"

# Wait for Qdrant
echo -n "Waiting for Qdrant..."
until curl -sf http://localhost:6333/health >/dev/null 2>&1; do
    echo -n "."
    sleep 2
done
echo " Ready!"

# Run database migrations
echo ""
echo "Running database migrations..."
if [ -f "scripts/run-migrations.sh" ]; then
    ./scripts/run-migrations.sh
else
    echo "Warning: scripts/run-migrations.sh not found. Skipping migrations."
    echo "You may need to run migrations manually before starting application services."
fi

# Load seed data if available
echo ""
echo "Loading seed data..."
if [ -f "scripts/seed-data.sql" ]; then
    docker-compose exec -T postgres psql -U mediagateway -d media_gateway -f - < scripts/seed-data.sql
    echo "Seed data loaded successfully!"
else
    echo "Note: scripts/seed-data.sql not found. Skipping seed data."
fi

# Generate SQLx metadata for offline builds
echo ""
echo "Generating SQLx metadata for offline builds..."
if command -v cargo-sqlx >/dev/null 2>&1; then
    # Set DATABASE_URL for sqlx
    export DATABASE_URL="postgresql://mediagateway:localdev123@localhost:5432/media_gateway"

    # Prepare SQLx for each crate that uses it
    for crate in crates/auth crates/discovery crates/sona crates/sync crates/ingestion crates/playback; do
        if [ -d "$crate" ]; then
            echo "  Preparing SQLx for $crate..."
            (cd "$crate" && cargo sqlx prepare --check >/dev/null 2>&1 || cargo sqlx prepare) || true
        fi
    done
    echo "SQLx metadata generation complete!"
else
    echo "Note: cargo-sqlx not installed. Install with: cargo install sqlx-cli"
    echo "      Skipping SQLx metadata generation."
fi

# Start application services
echo ""
echo "Starting application services..."
docker-compose up -d

# Wait for application services
echo ""
echo "Waiting for application services to be healthy..."

services=("api-gateway:8080" "discovery:8081" "sona:8082" "auth:8083" "sync:8084" "ingestion:8085" "playback:8086")
for service in "${services[@]}"; do
    name="${service%%:*}"
    port="${service##*:}"
    echo -n "Waiting for $name..."
    max_attempts=30
    attempt=0
    until curl -sf http://localhost:${port}/health >/dev/null 2>&1 || [ $attempt -eq $max_attempts ]; do
        echo -n "."
        sleep 2
        ((attempt++))
    done
    if [ $attempt -eq $max_attempts ]; then
        echo " Timeout! (Check logs: docker-compose logs $name)"
    else
        echo " Ready!"
    fi
done

echo ""
echo "=== All services are running! ==="
echo ""
echo "Infrastructure endpoints:"
echo "  PostgreSQL: localhost:5432"
echo "  Redis:      localhost:6379"
echo "  Qdrant:     localhost:6333 (REST), localhost:6334 (gRPC)"
echo ""
echo "Application endpoints:"
echo "  API Gateway: http://localhost:8080"
echo "  Discovery:   http://localhost:8081"
echo "  SONA:        http://localhost:8082"
echo "  Auth:        http://localhost:8083"
echo "  Sync:        http://localhost:8084"
echo "  Ingestion:   http://localhost:8085"
echo "  Playback:    http://localhost:8086"
echo ""
echo "Commands:"
echo "  Stop services:        docker-compose down"
echo "  View logs:            docker-compose logs -f [service-name]"
echo "  Restart service:      docker-compose restart [service-name]"
echo "  Development mode:     docker-compose -f docker-compose.yml -f docker-compose.dev.yml up"
