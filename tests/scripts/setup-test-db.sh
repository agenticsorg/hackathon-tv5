#!/bin/bash
# Setup test database for integration tests

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Setting up Media Gateway Test Database${NC}"

# Default values
DB_HOST="${POSTGRES_HOST:-localhost}"
DB_PORT="${POSTGRES_PORT:-5432}"
DB_USER="${POSTGRES_USER:-postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:-postgres}"
DB_NAME="media_gateway_test"

# Check if PostgreSQL is running
echo -e "${YELLOW}Checking PostgreSQL connection...${NC}"
if ! pg_isready -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" > /dev/null 2>&1; then
    echo -e "${RED}ERROR: PostgreSQL is not running on $DB_HOST:$DB_PORT${NC}"
    echo "Please start PostgreSQL and try again."
    exit 1
fi

echo -e "${GREEN}✓ PostgreSQL is running${NC}"

# Drop existing test database if it exists
echo -e "${YELLOW}Dropping existing test database (if exists)...${NC}"
PGPASSWORD="$DB_PASSWORD" dropdb -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" --if-exists "$DB_NAME"

# Create test database
echo -e "${YELLOW}Creating test database...${NC}"
PGPASSWORD="$DB_PASSWORD" createdb -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" "$DB_NAME"

echo -e "${GREEN}✓ Test database created${NC}"

# Export DATABASE_URL for migrations
export DATABASE_URL="postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"

# Run migrations
echo -e "${YELLOW}Running migrations...${NC}"
cd "$(dirname "$0")/../.."
sqlx migrate run --source migrations

echo -e "${GREEN}✓ Migrations completed${NC}"

# Verify setup
echo -e "${YELLOW}Verifying setup...${NC}"
PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "\dt" > /dev/null

echo -e "${GREEN}✓ Setup verified${NC}"

echo ""
echo -e "${GREEN}Test database setup complete!${NC}"
echo ""
echo "Database URL: $DATABASE_URL"
echo ""
echo "To run tests:"
echo "  export DATABASE_URL=\"$DATABASE_URL\""
echo "  cargo test --package media-gateway-tests"
