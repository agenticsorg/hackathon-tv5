#!/bin/bash
# Media Gateway Development Environment Teardown Script
# Run: chmod +x scripts/dev-teardown.sh && ./scripts/dev-teardown.sh

set -e

echo "=== Media Gateway Development Teardown ==="

# Parse command-line arguments
CLEAN_VOLUMES=false
CLEAN_IMAGES=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --volumes)
            CLEAN_VOLUMES=true
            shift
            ;;
        --images)
            CLEAN_IMAGES=true
            shift
            ;;
        --all)
            CLEAN_VOLUMES=true
            CLEAN_IMAGES=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--volumes] [--images] [--all]"
            echo "  --volumes: Remove all Docker volumes (deletes all data)"
            echo "  --images:  Remove Docker images"
            echo "  --all:     Remove everything (volumes + images)"
            exit 1
            ;;
    esac
done

# Stop all services
echo "Stopping all Media Gateway services..."
docker-compose down

if [ "$CLEAN_VOLUMES" = true ]; then
    echo ""
    echo "WARNING: Removing all Docker volumes. This will delete all data!"
    read -p "Are you sure? (yes/no): " confirmation
    if [ "$confirmation" = "yes" ]; then
        echo "Removing Docker volumes..."
        docker-compose down -v
        echo "Volumes removed."
    else
        echo "Skipping volume removal."
    fi
fi

if [ "$CLEAN_IMAGES" = true ]; then
    echo ""
    echo "Removing Docker images..."

    # Remove Media Gateway images
    docker images | grep mg- | awk '{print $3}' | xargs -r docker rmi -f || true

    # Remove dangling images
    docker image prune -f

    echo "Images removed."
fi

# Clean up any orphaned networks
echo ""
echo "Cleaning up orphaned networks..."
docker network prune -f

# Optional: Clean up system
read -p "Run Docker system prune to free up space? (yes/no): " cleanup_confirmation
if [ "$cleanup_confirmation" = "yes" ]; then
    echo "Running Docker system prune..."
    docker system prune -f
fi

echo ""
echo "=== Teardown Complete ==="
echo ""

if [ "$CLEAN_VOLUMES" = true ]; then
    echo "All services stopped and data volumes removed."
    echo "Next setup will start with a fresh database."
else
    echo "All services stopped. Data volumes preserved."
    echo "Run with --volumes to remove all data."
fi

echo ""
echo "To restart the development environment, run:"
echo "  ./scripts/dev-setup.sh"
