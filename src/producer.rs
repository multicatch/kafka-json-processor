use std::time::Duration;
use crossbeam_channel::Receiver;
use log::{debug, error, trace};
use rdkafka::error::KafkaError;
use rdkafka::producer::{Producer, BaseProducer, BaseRecord};
use crate::PendingMessage;

pub async fn producer_loop(producer: BaseProducer, topic: &str, rx: Receiver<PendingMessage>) {
    while let Ok(pending) = rx.recv() {
        match pending {
            PendingMessage::Received => {
                // Producer ready to receive messages.
                // This message is used to slow down consumer if the producer is not able to keep up.
                // Sender<PendingMessage> uses a bounded channel, so it will block if it's full (that is,
                // when Receiver<PendingMessage> did not read objects from queue).
            }
            PendingMessage::Processed { id, message } => {
                debug!("[{id}] Producing message [{}]", message.key);
                trace!("[{id}] Produced: {}", message.message);
                send(&producer, topic, message.key, message.message).unwrap();
            }
        }
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