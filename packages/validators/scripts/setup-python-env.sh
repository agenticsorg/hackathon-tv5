#!/usr/bin/env bash
set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Trap handler for cleanup on interrupt
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo -e "${RED}Setup interrupted. Cleaning up corrupted venv...${NC}"
        rm -rf .venv
    fi
}
trap cleanup EXIT INT TERM

echo -e "${BLUE}Setting up Python virtual environment...${NC}"

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALIDATORS_DIR="$(dirname "$SCRIPT_DIR")"

cd "$VALIDATORS_DIR"

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}Error: python3 not found. Please install Python 3.8 or later.${NC}"
    exit 1
fi

# Get Python version
PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
echo -e "${GREEN}Found Python ${PYTHON_VERSION}${NC}"

# Check for minimum Python version (3.8+)
PYTHON_MAJOR=$(echo "$PYTHON_VERSION" | cut -d. -f1)
PYTHON_MINOR=$(echo "$PYTHON_VERSION" | cut -d. -f2)
if [ "$PYTHON_MAJOR" -lt 3 ] || ([ "$PYTHON_MAJOR" -eq 3 ] && [ "$PYTHON_MINOR" -lt 8 ]); then
    echo -e "${RED}Error: Python 3.8+ required, found ${PYTHON_VERSION}${NC}"
    exit 1
fi

# Validate or recreate virtual environment
if [ -d ".venv" ]; then
    # Check if venv is valid by testing for activate script
    if [ ! -f ".venv/bin/activate" ]; then
        echo -e "${YELLOW}Corrupted virtual environment detected, removing...${NC}"
        rm -rf .venv
    else
        echo -e "${GREEN}Valid virtual environment found${NC}"
    fi
fi

# Create virtual environment if needed
if [ ! -d ".venv" ]; then
    echo -e "${BLUE}Creating virtual environment...${NC}"

    # Use timeout to prevent hanging (5 minute timeout)
    if command -v timeout &> /dev/null; then
        timeout 300 python3 -m venv .venv || {
            echo -e "${RED}Virtual environment creation failed or timed out${NC}"
            rm -rf .venv
            exit 1
        }
    else
        python3 -m venv .venv || {
            echo -e "${RED}Virtual environment creation failed${NC}"
            rm -rf .venv
            exit 1
        }
    fi

    # Verify activation script exists
    if [ ! -f ".venv/bin/activate" ]; then
        echo -e "${RED}Virtual environment creation incomplete${NC}"
        rm -rf .venv
        exit 1
    fi

    echo -e "${GREEN}Virtual environment created successfully${NC}"
fi

# Activate virtual environment
echo -e "${BLUE}Activating virtual environment...${NC}"
source .venv/bin/activate

# Verify activation
if [ -z "${VIRTUAL_ENV:-}" ]; then
    echo -e "${RED}Failed to activate virtual environment${NC}"
    exit 1
fi
echo -e "${GREEN}Virtual environment activated${NC}"

# Upgrade pip with timeout and error handling
echo -e "${BLUE}Upgrading pip...${NC}"
python -m pip install --upgrade pip --timeout 60 --retries 3 || {
    echo -e "${YELLOW}Warning: pip upgrade failed, continuing with existing version${NC}"
}

# Create requirements.txt if it doesn't exist
if [ ! -f "requirements.txt" ]; then
    echo -e "${BLUE}Creating requirements.txt...${NC}"
    cat > requirements.txt <<EOF
jsonschema>=4.17.0
pyyaml>=6.0
EOF
    echo -e "${GREEN}requirements.txt created${NC}"
fi

# Install dependencies with timeout and error handling
if [ -f "requirements.txt" ]; then
    echo -e "${BLUE}Installing dependencies from requirements.txt...${NC}"
    pip install -r requirements.txt --timeout 60 --retries 3 || {
        echo -e "${RED}Failed to install dependencies${NC}"
        exit 1
    }
    echo -e "${GREEN}Dependencies installed successfully${NC}"
fi

# Verify installation
echo -e "${BLUE}Verifying installation...${NC}"
python -c "import jsonschema, yaml; print('All dependencies available')" || {
    echo -e "${RED}Dependency verification failed${NC}"
    exit 1
}

# Disable cleanup trap on success
trap - EXIT

echo -e "${GREEN}Python environment setup complete!${NC}"
echo -e "${YELLOW}To activate the virtual environment manually, run:${NC}"
echo -e "  source .venv/bin/activate"
