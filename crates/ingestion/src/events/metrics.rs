use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge, HistogramVec,
    IntCounterVec, IntGauge,
};
use std::time::Duration;

lazy_static::lazy_static! {
    static ref EVENTS_SENT_TOTAL: IntCounterVec = register_int_counter_vec!(
        "kafka_events_sent_total",
        "Total number of events successfully sent to Kafka",
        &["topic"]
    )
    .unwrap();

    static ref EVENTS_FAILED_TOTAL: IntCounterVec = register_int_counter_vec!(
        "kafka_events_failed_total",
        "Total number of events that failed to send to Kafka",
        &["topic"]
    )
    .unwrap();

    static ref DELIVERY_LATENCY_SECONDS: HistogramVec = register_histogram_vec!(
        "kafka_delivery_latency_seconds",
        "Histogram of Kafka delivery latencies in seconds",
        &["topic"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap();

    static ref PRODUCER_QUEUE_SIZE: IntGauge = register_int_gauge!(
        "kafka_producer_queue_size",
        "Current number of messages in the producer queue"
    )
    .unwrap();
}

#[derive(Clone)]
pub struct ProducerMetrics;

impl ProducerMetrics {
    pub fn new() -> Self {
        Self
    }

    pub fn increment_events_sent(&self, topic: &str) {
        EVENTS_SENT_TOTAL.with_label_values(&[topic]).inc();
    }

    pub fn increment_events_failed(&self, topic: &str) {
        EVENTS_FAILED_TOTAL.with_label_values(&[topic]).inc();
    }

    pub fn observe_delivery_latency(&self, duration: Duration) {
        DELIVERY_LATENCY_SECONDS
            .with_label_values(&["all"])
            .observe(duration.as_secs_f64());
    }

    pub fn set_producer_queue_size(&self, size: i64) {
        PRODUCER_QUEUE_SIZE.set(size);
    }
}

impl Default for ProducerMetrics {
    fn default() -> Self {
        Self::new()
    }
}
