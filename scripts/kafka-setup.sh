#!/bin/bash
# Kafka Topic Setup Script
# Creates required topics for Media Gateway platform
# Usage: Automatically run via docker-compose.kafka.yml

set -e

KAFKA_BOOTSTRAP_SERVERS="${KAFKA_BOOTSTRAP_SERVERS:-kafka:9092}"
REPLICATION_FACTOR="${KAFKA_REPLICATION_FACTOR:-1}"
DEFAULT_PARTITIONS="${KAFKA_DEFAULT_PARTITIONS:-3}"

echo "=========================================="
echo "Media Gateway - Kafka Topic Setup"
echo "=========================================="
echo "Bootstrap Servers: $KAFKA_BOOTSTRAP_SERVERS"
echo "Replication Factor: $REPLICATION_FACTOR"
echo "Default Partitions: $DEFAULT_PARTITIONS"
echo ""

# Wait for Kafka to be ready
echo "Waiting for Kafka broker to be ready..."
cub kafka-ready -b "$KAFKA_BOOTSTRAP_SERVERS" 1 60

echo ""
echo "Creating Kafka topics..."

# Function to create topic with custom retention and partitions
create_topic() {
    local topic_name=$1
    local partitions=${2:-$DEFAULT_PARTITIONS}
    local retention_hours=${3:-168}  # 7 days default
    local retention_ms=$((retention_hours * 3600000))

    echo "Creating topic: $topic_name"
    kafka-topics --bootstrap-server "$KAFKA_BOOTSTRAP_SERVERS" \
        --create \
        --if-not-exists \
        --topic "$topic_name" \
        --partitions "$partitions" \
        --replication-factor "$REPLICATION_FACTOR" \
        --config retention.ms="$retention_ms" \
        --config compression.type=snappy \
        --config cleanup.policy=delete

    if [ $? -eq 0 ]; then
        echo "✓ Topic '$topic_name' created successfully (partitions=$partitions, retention=${retention_hours}h)"
    else
        echo "✗ Failed to create topic '$topic_name'"
        return 1
    fi
}

# Ingestion Topics
echo ""
echo "--- Ingestion Topics ---"
create_topic "content-ingested" 5 720        # 30 days retention, 5 partitions
create_topic "content-updated" 3 168         # 7 days retention
create_topic "content-validation" 3 48       # 2 days retention

# Playback Topics
echo ""
echo "--- Playback Topics ---"
create_topic "playback-events" 5 168         # 7 days retention, 5 partitions
create_topic "playback-errors" 3 720         # 30 days retention

# User Activity Topics
echo ""
echo "--- User Activity Topics ---"
create_topic "user-activity" 5 720           # 30 days retention, 5 partitions
create_topic "user-preferences" 3 2160       # 90 days retention
create_topic "user-sessions" 3 168           # 7 days retention

# Discovery Topics
echo ""
echo "--- Discovery Topics ---"
create_topic "search-queries" 3 168          # 7 days retention
create_topic "recommendation-events" 5 720   # 30 days retention

# Sync Topics
echo ""
echo "--- Sync Topics ---"
create_topic "sync-state-changes" 3 168      # 7 days retention
create_topic "sync-conflicts" 3 720          # 30 days retention

# Analytics Topics
echo ""
echo "--- Analytics Topics ---"
create_topic "analytics-events" 5 2160       # 90 days retention, 5 partitions
create_topic "quality-metrics" 3 720         # 30 days retention

# System Topics
echo ""
echo "--- System Topics ---"
create_topic "audit-logs" 3 2160             # 90 days retention
create_topic "system-alerts" 2 720           # 30 days retention

# Dead Letter Queue
echo ""
echo "--- Dead Letter Queue ---"
create_topic "dlq-events" 2 2160             # 90 days retention

echo ""
echo "=========================================="
echo "Listing all topics:"
echo "=========================================="
kafka-topics --bootstrap-server "$KAFKA_BOOTSTRAP_SERVERS" --list

echo ""
echo "=========================================="
echo "Topic details:"
echo "=========================================="
kafka-topics --bootstrap-server "$KAFKA_BOOTSTRAP_SERVERS" --describe

echo ""
echo "=========================================="
echo "Kafka topic setup completed successfully!"
echo "=========================================="
