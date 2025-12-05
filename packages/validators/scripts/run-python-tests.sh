#!/usr/bin/env bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}Running Python validator tests...${NC}"

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALIDATORS_DIR="$(dirname "$SCRIPT_DIR")"

cd "$VALIDATORS_DIR"

# Ensure virtual environment exists
if [ ! -d ".venv" ]; then
    echo -e "${BLUE}Virtual environment not found, setting up...${NC}"
    ./scripts/setup-python-env.sh
fi

# Activate virtual environment
source .venv/bin/activate

# Run validator tests
echo -e "${BLUE}Testing Python validator...${NC}"

# Test with sample llms.txt if it exists
if [ -f "../../apps/www/public/llms.txt" ]; then
    echo -e "${BLUE}Validating apps/www/public/llms.txt...${NC}"
    python validate-arw.py ../../apps/www/public/llms.txt --schema ../schemas/arw_model.json
    echo -e "${GREEN}Validation successful!${NC}"
else
    echo -e "${RED}Sample file not found at apps/www/public/llms.txt${NC}"
    exit 1
fi

echo -e "${GREEN}All Python tests passed!${NC}"
