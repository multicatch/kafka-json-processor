use std::collections::HashMap;
use crossbeam_channel::Sender;
use log::{debug, error, trace, warn};
use rdkafka::consumer::StreamConsumer;
use rdkafka::Message;
use tokio::runtime::Runtime;
use crate::{PendingMessage, Stream};
use crate::processor::{process_payload, ProcessingResult};

pub async fn consumer_loop(consumer: StreamConsumer, tx: Sender<PendingMessage>, runtime: &Runtime, streams: HashMap<String, Stream>)
                           -> ProcessingResult<()>
{
    loop {
        match consumer.recv().await {
            Ok(message) => {
                tx.send(PendingMessage::Received).unwrap();

                let payload = message.payload().unwrap().to_vec();
                let key = format!("{}:{}@{}({})",
                                  message.topic(),
                                  message.partition(),
                                  message.offset(),
                                  message.timestamp().to_millis().unwrap_or(0)
                );

                debug!("[{key}] Received message.");
                trace!("[{key}] Message: {}", String::from_utf8_lossy(&payload));

                if let Some(stream) = streams.get(message.topic()) {
                    spawn_task(runtime, tx.clone(), key, payload, stream.clone());
                } else {
                    warn!("[{key}] Topic {} is unsupported! Ignoring message.", message.topic());
                }
            }
            Err(e) => {
                error!("Cannot consume message! Reason: {e}");
            }
        }
    }
}

fn spawn_task(runtime: &Runtime, tx: Sender<PendingMessage>, key: String, payload: Vec<u8>, stream: Stream) {
    runtime.spawn(async move {
        match process_payload(key.clone(), &payload, stream.processors) {
            Ok(processed) => {
                trace!("[{key}] Output: {}", processed.message);
                tx.send(PendingMessage::Processed {
                    id: key,
                    topic: stream.target_topic.clone(),
                    message: processed,
                }).unwrap();
            }
            Err(e) => {
                error!("[{key}] Processing error: {e}. Message will be ignored and lost.");
            }
        };
    });
}