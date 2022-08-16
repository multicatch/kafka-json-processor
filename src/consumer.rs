use std::sync::mpsc::Sender;
use log::{error, info};
use rdkafka::consumer::StreamConsumer;
use rdkafka::Message;

pub async fn consumer_loop(consumer: StreamConsumer, tx: Sender<String>) {
    loop {
        match consumer.recv().await {
            Ok(message) => {
                let payload = String::from_utf8_lossy(message.payload().unwrap());
                let key = format!("{}:{}", message.topic(), message.partition());
                info!("Received message: [{}:{}] [{}] {}",
                        key, message.offset(), message.timestamp().to_millis().unwrap_or(0), payload
                    );

                tx.send(payload.into()).unwrap();
            }
            Err(e) => {
                error!("Cannot consume message! Reason: {}", e);
            }
        }
    }
}