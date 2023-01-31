use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use crossbeam_channel::bounded;
use log::{info, warn, error, debug, trace};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::BaseProducer;
use rdkafka::{Offset, TopicPartitionList};
use tokio::runtime::{Builder, Runtime};
use tokio::time::interval;
use crate::config::Config;
use crate::consumer::consumer_loop;
use crate::journal::{MessageOffsetHolder, OffsetKey};
use crate::processor::{Processor, SerializedOutputMessage};
use crate::producer::producer_loop;

pub mod config;
mod consumer;
mod producer;
pub mod processor;
pub mod formatters;
pub mod simulation;
pub mod error;
pub mod journal;

#[derive(Clone)]
pub struct Stream {
    pub source_topic: String,
    pub target_topic: String,
    pub processors: &'static [Processor],
}

pub enum PendingMessage {
    Received,
    Processed {
        id: String,
        topic: String,
        offset: MessageOffset,
        message: SerializedOutputMessage,
    },
}

pub struct MessageOffset {
    topic: String,
    partition: i32,
    offset: i64,
}

pub fn run_processor(streams: HashMap<String, Stream>) {
    info!("Starting kafka-json-processor...");

    let config_env = "KAFKA_PROCESSOR_CONFIG_PATH";
    let default_config_path = "./processor.properties";
    let config_path = std::env::var(config_env)
        .unwrap_or_else(|_| {
            info!("Environment variable {} not found, using default config path.", config_env);
            default_config_path.to_string()
        });

    info!("Reading config from {}", config_path);
    let config = Config::read_from(default_config_path).unwrap();

    loop {
        debug!("Starting runtime...");

        let config = config.clone();

        let runtime = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(config.internal_config.worker_threads)
            .build()
            .unwrap();

        if let Err(e) = runtime.block_on(async {
            run_processing_tasks(
                &runtime,
                config,
                streams.clone(),
            ).await
        }) {
            warn!("RUNTIME ERROR: {:?}. Restarting...", e);
        }
    }
}

macro_rules! exec_or_retry_in_10s {
    ($supplier:expr) => {
        match $supplier {
            Ok(value) => value,
            Err(e) => {
                error!("Connection error: [{}], retrying in 10 seconds...", e);
                std::thread::sleep(Duration::from_secs(10));
                return Ok(());
            }
        }
    };
}

async fn run_processing_tasks(
    runtime: &Runtime,
    config: Config,
    streams: HashMap<String, Stream>,
) -> Result<(), Box<dyn Error>> {
    let consumer: StreamConsumer = exec_or_retry_in_10s!(config.consumer_config.create());
    let producer: BaseProducer = exec_or_retry_in_10s!(config.producer_config.create());

    let offset_holder = MessageOffsetHolder::with_offsets_in(config.internal_config.journal_path)?;
    let offset_holder = Arc::new(offset_holder);
    let producer_offset_holder = offset_holder.clone();

    show_streams_and_subscribe(&consumer, &streams, offset_holder.offsets())?;

    let (tx, rx) = bounded(config.internal_config.channel_capacity);
    runtime.spawn(async move {
        producer_loop(
            producer,
            rx,
            config.internal_config.queue_size,
            Duration::from_millis(config.internal_config.queue_slowdown_ms as u64),
            producer_offset_holder,
        ).await;
    });

    runtime.spawn(async move {
        journal_flush_loop(offset_holder).await;
    });

    consumer_loop(consumer, tx, runtime, streams).await
}

fn show_streams_and_subscribe(consumer: &StreamConsumer, streams: &HashMap<String, Stream>, offsets: HashMap<OffsetKey, i64>) -> Result<(), Box<dyn Error>> {
    streams.values()
        .for_each(|stream| {
           info!("Stream [{}] --> [{}]: {} processor(s).", stream.source_topic, stream.target_topic, stream.processors.len());
        });

    let topics: Vec<&str> = streams.keys()
        .map(|key| key.as_str())
        .collect();

    consumer.subscribe(&topics)?;

    let mut topic_list = TopicPartitionList::new();

    // add topics with saved offsets to topic_list and store those topics in a vec
    let topics_with_offsets: Vec<String> = offsets.into_iter()
        .filter(|(offset_key, _)| {
            // topic is valid if it's associated with known stream
            topics.contains(&offset_key.0.as_str())
        })
        .filter_map(|(offset_key, offset)| {
            trace!("Adding topic [Topic: {}] [Partition: {}] with offset {}", &offset_key.0, offset_key.1, offset);

            topic_list.add_partition_offset(&offset_key.0, offset_key.1, Offset::Offset(offset))
                .map_err(|e| {
                    error!("Cannot assign topic with offset {offset} [Topic: {}] [Partition: {}]. Reason: {e}", &offset_key.0, offset_key.1)
                })
                .map(|_| {
                    offset_key.0
                })
                .ok()
        })
        .collect();

    topics.iter()
        .filter(|topic| !topics_with_offsets.contains(&topic.to_string()))
        .for_each(|topic| {
            trace!("Adding unassigned topic [Topic: {topic}]");
            topic_list.add_topic_unassigned(topic);
        });

    if let Err(e) = consumer.assign(&topic_list) {
        error!("Cannot assign topics. Topics or offsets are incorrect. Will continue anyway. Reason for failure: {e}");
    }

    Ok(())
}

/// Runs a loop that flushes offsets every 30 seconds.
///
/// Flushing saves current offsets to a journal on disk.
/// It is used to subscribe to topics from given offsets in case of crash or client id change
/// (when we cannot be sure where we finished during last run).
async fn journal_flush_loop(offset_holder: Arc<MessageOffsetHolder>) {
    let mut journal_interval = interval(Duration::from_secs(30));

    loop {
        journal_interval.tick().await;

        debug!("Timeout, flushing journal");
        offset_holder.flush();
    }
}