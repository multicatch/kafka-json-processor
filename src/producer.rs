use std::time::Duration;
use crossbeam_channel::Receiver;
use log::{debug, error, info, trace, warn};
use rdkafka::error::KafkaError;
use rdkafka::error::RDKafkaErrorCode::{InvalidTopic, QueueFull, UnknownTopic, UnknownTopicOrPartition};
use rdkafka::producer::{Producer, BaseProducer, BaseRecord};
use rdkafka::util::Timeout;
use crate::{PendingMessage, SerializedOutputMessage};

pub async fn producer_loop(producer: BaseProducer, topic: &str, rx: Receiver<PendingMessage>, queue_size: usize, queue_slowdown_time: Duration) {
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

                if let SentMessage::ShouldSkipMessage = send_loop(&producer, topic, &id, message, queue_size, queue_slowdown_time) {
                    // Message not sent, so
                    continue;
                }

                while producer.in_flight_count() as f64 >= 0.95f64 * queue_size as f64 {
                    warn!("Producer queue is almost full ({}/{}). Halting production for {}s.", producer.in_flight_count(), queue_size, queue_slowdown_time.as_secs());
                    // This flush will block current thread for queue_slowdown_time or less.
                    producer.flush(Timeout::After(queue_slowdown_time));
                    info!("Restarting production (pending: {})...", producer.in_flight_count());
                }
            }
        }
    }
}

fn send_loop(producer: &BaseProducer, topic: &str, id: &str, message: SerializedOutputMessage, queue_size: usize, queue_slowdown_time: Duration) -> SentMessage {
    loop {
        match send_and_recover(
            producer,
            topic,
            id,
            message.key.clone(),
            message.message.clone(),
            queue_size,
            queue_slowdown_time
        ) {
            SentMessage::Ok |
            SentMessage::ShouldIgnore => { return SentMessage::Ok; },
            SentMessage::ShouldSkipMessage => { return SentMessage::ShouldSkipMessage; },
            SentMessage::CanRetry => { /* try, try again */ },
        }
    };
}

fn send_and_recover(
    producer: &BaseProducer,
    topic: &str,
    id: &str,
    key: String,
    payload: String,
    queue_size: usize,
    queue_slowdown_time: Duration,
) -> SentMessage {
    if let Err(KafkaError::MessageProduction(err)) = send(producer, topic, key, payload) {
        if err == QueueFull {
            warn!(
                "[{id}] Producer queue is full. Try changing producer config (bigger queue, lower linger.ms). Output queue: {}/{}, slowing down producer for {}s.",
                producer.in_flight_count(), queue_size, queue_slowdown_time.as_secs()
            );
            // This flush will block current thread for queue_slowdown_time or less.
            producer.flush(Timeout::After(queue_slowdown_time));
            SentMessage::CanRetry
        } else if err == UnknownTopic || err == UnknownTopicOrPartition || err == InvalidTopic {
            error!("[{id}] Topic [{}] is invalid (or not existent in Kafka). Message will be lost!", topic);
            SentMessage::ShouldSkipMessage
        } else {
            SentMessage::ShouldIgnore
        }
    } else {
        SentMessage::Ok
    }
}

enum SentMessage {
    Ok,
    ShouldIgnore,
    ShouldSkipMessage,
    CanRetry
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