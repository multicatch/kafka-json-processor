use std::sync::mpsc::Sender;
use log::{debug, error, trace};
use rdkafka::consumer::StreamConsumer;
use rdkafka::Message;
use tokio::runtime::Runtime;
use crate::processor::{process_payload, Processor};

pub async fn consumer_loop(consumer: StreamConsumer, tx: Sender<String>, runtime: &Runtime, processors: &'static [Processor]) {
    loop {
        match consumer.recv().await {
            Ok(message) => {
                let payload = message.payload().unwrap().to_vec();
                let key = format!("{}:{}@{}({})",
                                  message.topic(),
                                  message.partition(),
                                  message.offset(),
                                  message.timestamp().to_millis().unwrap_or(0)
                );

                debug!("[{key}] Received message.");
                trace!("[{key}] Message: {}", String::from_utf8_lossy(&payload));

                spawn_task(runtime, tx.clone(), key, payload, processors);
            }
            Err(e) => {
                error!("Cannot consume message! Reason: {e}");
            }
        }
    }
}

fn spawn_task(runtime: &Runtime, tx: Sender<String>, key: String, payload: Vec<u8>, processors: &'static [Processor]) {
    runtime.spawn(async move {
        match process_payload(key.clone(), &payload, processors) {
            Ok(processed) => {
                trace!("[{key}] Output: {}", processed.message);
                tx.send(processed.message).unwrap();
            }
            Err(e) => {
                error!("[{key}] Processing error: {e}. Message will be ignored and lost.");
            }
        };
    });
}