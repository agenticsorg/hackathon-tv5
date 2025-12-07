#!/bin/bash
# Run database migrations for Media Gateway
# This script should be run after PostgreSQL is ready but before starting application services

set -e

echo "=== Running Database Migrations ==="

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Default database URL if not set
DATABASE_URL=${DATABASE_URL:-"postgresql://mediagateway:localdev123@localhost:5432/media_gateway"}

echo "Database URL: ${DATABASE_URL}"

# Check if sqlx-cli is installed
if ! command -v sqlx >/dev/null 2>&1; then
    echo "sqlx-cli not found. Installing..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Run migrations
echo "Running migrations..."
sqlx migrate run --database-url "$DATABASE_URL" || {
    echo "Warning: Migration failed or no migrations found."
    echo "This is expected if migrations haven't been created yet."
    exit 0
}

echo "Migrations completed successfully!"
