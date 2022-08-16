use std::sync::mpsc::Receiver;
use std::time::Duration;
use log::{debug, error};
use rdkafka::error::KafkaError;
use rdkafka::producer::{Producer, BaseProducer, BaseRecord};

pub async fn producer_loop(producer: BaseProducer, topic: &str, rx: Receiver<String>) {
    while let Ok(payload) = rx.recv() {
        debug!("Producing message: {payload}");
        send(&producer, topic, "".to_string(), payload).unwrap();
    }
}

fn send(producer: &BaseProducer, topic: &str, key: String, payload: String) -> Result<(), KafkaError> {
    producer.send(
        BaseRecord::to(topic)
            .key(&key)
            .payload(&payload),
    )
        .map(|_| {
            producer.poll(Duration::from_millis(0));
        })
        .map_err(|(e, _)| {
            error!("Could not send message [{}]! Reason: {}, queue: {}", key, e, producer.in_flight_count());
            e
        })
}