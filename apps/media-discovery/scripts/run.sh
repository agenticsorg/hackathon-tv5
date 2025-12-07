#!/bin/bash

#################################################
# AI Media Discovery - Production Run Script
#
# This script builds and starts the Next.js app
# with all required environment configuration.
#################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║        AI Media Discovery - Production Startup            ║"
echo "║      Voice Search + TMDB Movie Database Integration       ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Navigate to app directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(dirname "$SCRIPT_DIR")"
cd "$APP_DIR"

echo -e "${BLUE}📁 Working directory: ${APP_DIR}${NC}"

# Check for required environment variables
echo -e "\n${YELLOW}🔍 Checking environment configuration...${NC}"

if [ -f .env ]; then
    echo -e "${GREEN}✓ Found .env file${NC}"
    source .env 2>/dev/null || true
else
    echo -e "${YELLOW}⚠ No .env file found. Creating from template...${NC}"

    if [ -n "$NEXT_PUBLIC_TMDB_ACCESS_TOKEN" ]; then
        echo "NEXT_PUBLIC_TMDB_ACCESS_TOKEN=$NEXT_PUBLIC_TMDB_ACCESS_TOKEN" > .env
        echo -e "${GREEN}✓ Created .env with TMDB token from environment${NC}"
    else
        echo -e "${RED}❌ NEXT_PUBLIC_TMDB_ACCESS_TOKEN not set${NC}"
        echo -e "${YELLOW}Please set TMDB token in .env file:${NC}"
        echo "NEXT_PUBLIC_TMDB_ACCESS_TOKEN=your_token_here"
        exit 1
    fi
fi

# Verify TMDB token exists
if [ -z "$NEXT_PUBLIC_TMDB_ACCESS_TOKEN" ]; then
    source .env 2>/dev/null || true
fi

if [ -z "$NEXT_PUBLIC_TMDB_ACCESS_TOKEN" ]; then
    echo -e "${RED}❌ NEXT_PUBLIC_TMDB_ACCESS_TOKEN not found in environment${NC}"
    exit 1
fi
echo -e "${GREEN}✓ TMDB API token configured${NC}"

# Install dependencies
echo -e "\n${YELLOW}📦 Installing dependencies...${NC}"
if command -v pnpm &> /dev/null; then
    pnpm install --frozen-lockfile 2>/dev/null || pnpm install
elif command -v npm &> /dev/null; then
    npm ci 2>/dev/null || npm install
else
    echo -e "${RED}❌ No package manager found (npm or pnpm required)${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Dependencies installed${NC}"

# Build options
BUILD_MODE="${1:-dev}"

if [ "$BUILD_MODE" = "prod" ] || [ "$BUILD_MODE" = "production" ]; then
    echo -e "\n${YELLOW}🔨 Building for production...${NC}"
    npm run build
    echo -e "${GREEN}✓ Production build complete${NC}"

    echo -e "\n${BLUE}🚀 Starting production server on port 3000...${NC}"
    echo -e "${GREEN}➜ Open http://localhost:3000 in your browser${NC}"
    echo -e "${YELLOW}   Press Ctrl+C to stop${NC}"
    echo ""
    npm run start
else
    echo -e "\n${BLUE}🚀 Starting development server on port 3000...${NC}"
    echo -e "${GREEN}➜ Open http://localhost:3000 in your browser${NC}"
    echo -e "${YELLOW}   Press Ctrl+C to stop${NC}"
    echo ""

    # Try to open browser (works on most systems)
    sleep 2 &
    (
        sleep 2
        if command -v xdg-open &> /dev/null; then
            xdg-open "http://localhost:3000" 2>/dev/null &
        elif command -v open &> /dev/null; then
            open "http://localhost:3000" 2>/dev/null &
        fi
    ) &

    npm run dev
fi
