use super::{ContentEvent, EventError, EventResult, KafkaConfig};
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use super::metrics::ProducerMetrics;

const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_MS: u64 = 100;

pub struct KafkaEventProducer {
    producer: Arc<FutureProducer>,
    config: KafkaConfig,
    metrics: ProducerMetrics,
}

impl KafkaEventProducer {
    pub fn new(config: KafkaConfig) -> EventResult<Self> {
        info!(
            brokers = %config.brokers,
            topic_prefix = %config.topic_prefix,
            "Initializing Kafka event producer"
        );

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("message.timeout.ms", config.message_timeout_ms.to_string())
            .set("request.timeout.ms", config.request_timeout_ms.to_string())
            .set("enable.idempotence", config.enable_idempotence.to_string())
            .set("acks", "all")
            .set("retries", "10")
            .set("max.in.flight.requests.per.connection", "5")
            .set("compression.type", "snappy")
            .create()
            .map_err(|e| EventError::ConfigError(e.to_string()))?;

        let metrics = ProducerMetrics::new();

        Ok(Self {
            producer: Arc::new(producer),
            config,
            metrics,
        })
    }

    pub fn from_env() -> EventResult<Self> {
        let config = KafkaConfig::from_env()?;
        Self::new(config)
    }

    async fn publish_with_confirmation(&self, event: &ContentEvent) -> EventResult<()> {
        let topic = self.config.topic_for_event(event.event_type());
        let payload = serde_json::to_string(event)?;
        let key = event.content_id().to_string();

        let start = std::time::Instant::now();

        debug!(
            content_id = %event.content_id(),
            event_type = %event.event_type(),
            correlation_id = %event.correlation_id(),
            topic = %topic,
            payload_size = payload.len(),
            "Publishing event to Kafka"
        );

        let record = FutureRecord::to(&topic).payload(&payload).key(&key);

        let timeout = Timeout::After(Duration::from_millis(self.config.message_timeout_ms));

        match self.producer.send(record, timeout).await {
            Ok((partition, offset)) => {
                let duration = start.elapsed();
                self.metrics.observe_delivery_latency(duration);
                self.metrics.increment_events_sent(&topic);

                debug!(
                    topic = %topic,
                    partition = partition,
                    offset = offset,
                    latency_ms = duration.as_millis(),
                    "Event delivered successfully"
                );
                Ok(())
            }
            Err((kafka_error, _)) => {
                self.metrics.increment_events_failed(&topic);
                error!(
                    topic = %topic,
                    error = %kafka_error,
                    "Failed to deliver event"
                );
                Err(EventError::DeliveryFailed(kafka_error.to_string()))
            }
        }
    }

    pub async fn publish_event(&self, event: ContentEvent) -> EventResult<()> {
        let mut retries = 0;

        loop {
            match self.publish_with_confirmation(&event).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    retries += 1;

                    if retries >= MAX_RETRIES {
                        error!(
                            content_id = %event.content_id(),
                            event_type = %event.event_type(),
                            retries = retries,
                            error = %e,
                            "Failed to publish event after max retries"
                        );
                        return Err(e);
                    }

                    let delay = Duration::from_millis(BASE_RETRY_DELAY_MS * 2u64.pow(retries - 1));

                    warn!(
                        content_id = %event.content_id(),
                        event_type = %event.event_type(),
                        retry = retries,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "Retrying event publication"
                    );

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    pub async fn publish_batch(&self, events: Vec<ContentEvent>) -> EventResult<()> {
        info!(count = events.len(), "Publishing event batch");

        for event in events {
            self.publish_event(event).await?;
        }

        Ok(())
    }

    pub async fn health_check(&self) -> EventResult<bool> {
        let metadata = self
            .producer
            .client()
            .fetch_metadata(None, Duration::from_secs(5))
            .map_err(|e| EventError::BrokerUnavailable(e.to_string()))?;

        let broker_count = metadata.brokers().len();
        debug!(broker_count = broker_count, "Kafka health check passed");

        Ok(broker_count > 0)
    }

    pub fn in_flight_count(&self) -> i32 {
        self.producer.in_flight_count()
    }
}

#[async_trait::async_trait]
impl super::EventProducer for KafkaEventProducer {
    async fn publish_event(&self, event: ContentEvent) -> EventResult<()> {
        self.publish_event(event).await
    }

    async fn publish_batch(&self, events: Vec<ContentEvent>) -> EventResult<()> {
        self.publish_batch(events).await
    }

    async fn health_check(&self) -> EventResult<bool> {
        self.health_check().await
    }
}
