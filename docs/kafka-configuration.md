# Kafka Configuration - Media Gateway

## Overview

Media Gateway uses Apache Kafka for event streaming and asynchronous communication between microservices. This document describes the Kafka setup, topics, and configuration.

## Architecture

### Components

- **Zookeeper**: Coordination service required by Kafka (port 2181)
- **Kafka Broker**: Event streaming platform (ports 9092, 9093)
- **Kafka UI** (optional): Web interface for topic management (port 8090)
- **Schema Registry** (optional): Schema management for Avro (port 8081)
- **Kafka Connect** (optional): Integration connectors (port 8083)

### Network Configuration

- **Internal Communication**: `kafka:9092` (PLAINTEXT)
- **External/Host Access**: `localhost:9093` (PLAINTEXT_HOST)

## Topics

### Ingestion Topics

| Topic | Partitions | Retention | Description |
|-------|-----------|-----------|-------------|
| `content-ingested` | 5 | 30 days | New content ingestion events |
| `content-updated` | 3 | 7 days | Content metadata updates |
| `content-validation` | 3 | 2 days | Content validation events |

### Playback Topics

| Topic | Partitions | Retention | Description |
|-------|-----------|-----------|-------------|
| `playback-events` | 5 | 7 days | Media playback tracking |
| `playback-errors` | 3 | 30 days | Playback error events |

### User Activity Topics

| Topic | Partitions | Retention | Description |
|-------|-----------|-----------|-------------|
| `user-activity` | 5 | 30 days | User interaction events |
| `user-preferences` | 3 | 90 days | User preference changes |
| `user-sessions` | 3 | 7 days | Session lifecycle events |

### Discovery Topics

| Topic | Partitions | Retention | Description |
|-------|-----------|-----------|-------------|
| `search-queries` | 3 | 7 days | Search query events |
| `recommendation-events` | 5 | 30 days | Recommendation generation events |

### Sync Topics

| Topic | Partitions | Retention | Description |
|-------|-----------|-----------|-------------|
| `sync-state-changes` | 3 | 7 days | State synchronization events |
| `sync-conflicts` | 3 | 30 days | Conflict resolution events |

### Analytics Topics

| Topic | Partitions | Retention | Description |
|-------|-----------|-----------|-------------|
| `analytics-events` | 5 | 90 days | General analytics events |
| `quality-metrics` | 3 | 30 days | Quality scoring metrics |

### System Topics

| Topic | Partitions | Retention | Description |
|-------|-----------|-----------|-------------|
| `audit-logs` | 3 | 90 days | Audit trail events |
| `system-alerts` | 2 | 30 days | System alerts and notifications |
| `dlq-events` | 2 | 90 days | Dead letter queue for failed messages |

## Usage

### Basic Setup

Start Kafka and all services:

```bash
docker-compose up -d
```

The `kafka-setup.sh` script will automatically create all required topics when Kafka becomes healthy.

### Extended Setup with Kafka UI

Include the extended Kafka configuration:

```bash
docker-compose -f docker-compose.yml -f docker-compose.kafka.yml up -d
```

Access Kafka UI at: http://localhost:8090

### Manual Topic Creation

```bash
# Connect to Kafka container
docker exec -it mg-kafka bash

# Create a custom topic
kafka-topics --bootstrap-server kafka:9092 \
  --create \
  --topic my-custom-topic \
  --partitions 3 \
  --replication-factor 1 \
  --config retention.ms=604800000

# List all topics
kafka-topics --bootstrap-server kafka:9092 --list

# Describe a topic
kafka-topics --bootstrap-server kafka:9092 --describe --topic content-ingested
```

### Producer Example (Rust)

```rust
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

async fn produce_event() -> Result<(), Box<dyn std::error::Error>> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "kafka:9092")
        .set("message.timeout.ms", "5000")
        .create()?;

    let payload = r#"{"content_id": "abc123", "type": "movie"}"#;

    producer
        .send(
            FutureRecord::to("content-ingested")
                .payload(payload)
                .key("abc123"),
            Duration::from_secs(0),
        )
        .await?;

    Ok(())
}
```

### Consumer Example (Rust)

```rust
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::Message;

async fn consume_events() -> Result<(), Box<dyn std::error::Error>> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "kafka:9092")
        .set("group.id", "ingestion-service")
        .set("auto.offset.reset", "earliest")
        .create()?;

    consumer.subscribe(&["content-ingested"])?;

    loop {
        match consumer.recv().await {
            Ok(message) => {
                if let Some(payload) = message.payload() {
                    let content = std::str::from_utf8(payload)?;
                    println!("Received: {}", content);
                }
            }
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
}
```

## Environment Variables

All services receive the following Kafka configuration:

```bash
KAFKA_BOOTSTRAP_SERVERS=kafka:9092
```

For external/host access (testing):

```bash
KAFKA_BOOTSTRAP_SERVERS=localhost:9093
```

## Configuration

### Kafka Broker Settings

```yaml
KAFKA_BROKER_ID: 1
KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: PLAINTEXT:PLAINTEXT,PLAINTEXT_HOST:PLAINTEXT
KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092,PLAINTEXT_HOST://localhost:9093
KAFKA_INTER_BROKER_LISTENER_NAME: PLAINTEXT
KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
KAFKA_TRANSACTION_STATE_LOG_MIN_ISR: 1
KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR: 1
KAFKA_AUTO_CREATE_TOPICS_ENABLE: "false"
KAFKA_LOG_RETENTION_HOURS: 168  # 7 days default
KAFKA_LOG_SEGMENT_BYTES: 1073741824  # 1GB
KAFKA_LOG_RETENTION_CHECK_INTERVAL_MS: 300000
```

### Zookeeper Settings

```yaml
ZOOKEEPER_CLIENT_PORT: 2181
ZOOKEEPER_TICK_TIME: 2000
ZOOKEEPER_SYNC_LIMIT: 2
```

## Health Checks

### Kafka Broker

```bash
# Via Docker healthcheck
docker exec mg-kafka kafka-broker-api-versions --bootstrap-server localhost:9092

# Manual check
docker exec mg-kafka kafka-topics --bootstrap-server localhost:9092 --list
```

### Zookeeper

```bash
# Via Docker healthcheck
docker exec mg-zookeeper nc -z localhost 2181

# Manual check
docker exec mg-zookeeper zkCli.sh -server localhost:2181 ls /
```

## Monitoring

### Kafka UI

Access the web interface at http://localhost:8090 (when using extended compose file)

Features:
- Topic browsing and management
- Consumer group monitoring
- Message viewing
- Configuration management

### Metrics

Kafka metrics are exposed via JMX and can be scraped by Prometheus:

```yaml
# Add to prometheus.yml
- job_name: 'kafka'
  static_configs:
    - targets: ['kafka:9997']
```

## Production Considerations

### Replication

For production, increase replication factor:

```bash
KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 3
KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR: 3
```

### Multiple Brokers

Add additional Kafka brokers to `docker-compose.yml`:

```yaml
kafka-2:
  image: confluentinc/cp-kafka:7.5.0
  environment:
    KAFKA_BROKER_ID: 2
    # ... other configs
```

### Security

Enable SASL/SSL authentication:

```yaml
KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: SASL_SSL:SASL_SSL
KAFKA_SASL_ENABLED_MECHANISMS: PLAIN
KAFKA_SASL_MECHANISM_INTER_BROKER_PROTOCOL: PLAIN
```

### Partitioning Strategy

- High-throughput topics: 5+ partitions
- Order-sensitive topics: 1 partition or partition by key
- Consumer parallelism: partitions ≥ consumer instances

## Troubleshooting

### Connection Issues

```bash
# Check if Kafka is running
docker ps | grep kafka

# Check Kafka logs
docker logs mg-kafka

# Test connection from host
docker exec mg-kafka kafka-broker-api-versions --bootstrap-server localhost:9092
```

### Topic Not Found

```bash
# List all topics
docker exec mg-kafka kafka-topics --bootstrap-server localhost:9092 --list

# Recreate topics
docker exec mg-kafka /scripts/kafka-setup.sh
```

### Consumer Lag

```bash
# Check consumer group lag
docker exec mg-kafka kafka-consumer-groups \
  --bootstrap-server localhost:9092 \
  --describe \
  --group <group-id>
```

### Disk Space

```bash
# Check Kafka data volume usage
docker exec mg-kafka df -h /var/lib/kafka/data

# Clean up old segments (be careful)
docker exec mg-kafka kafka-configs \
  --bootstrap-server localhost:9092 \
  --entity-type topics \
  --entity-name <topic-name> \
  --alter \
  --add-config retention.ms=3600000
```

## References

- [Apache Kafka Documentation](https://kafka.apache.org/documentation/)
- [Confluent Platform Documentation](https://docs.confluent.io/)
- [rdkafka Rust Client](https://docs.rs/rdkafka/)
- [Kafka Best Practices](https://kafka.apache.org/documentation/#bestpractices)

## Next Steps

1. Review topic schema in your service code
2. Implement producer/consumer patterns
3. Configure consumer groups appropriately
4. Set up monitoring and alerting
5. Plan for schema evolution strategy
