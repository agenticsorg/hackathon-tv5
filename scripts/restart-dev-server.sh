#!/bin/bash
# Restart Next.js dev server with cache clearing

set -e

echo "🔄 Restarting Next.js dev servers..."

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Kill any running Next.js processes
echo "Stopping existing Next.js processes..."
pkill -f "next dev" || true
sleep 2

# Clear .next cache directories
echo "Clearing .next cache directories..."
rm -rf apps/www/.next
rm -rf platform/apps/marketing/.next

echo -e "${GREEN}✅ Cache cleared${NC}"
echo ""
echo -e "${BLUE}To start the dev server, run:${NC}"
echo "  cd apps/www && npm run dev"
echo "  OR"
echo "  cd platform/apps/marketing && npm run dev"
echo ""
echo -e "${BLUE}Then test with:${NC}"
echo "  ./scripts/test-universal-agent-compatibility.sh http://localhost:3000"
