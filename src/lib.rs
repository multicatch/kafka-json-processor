use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use crossbeam_channel::bounded;
use log::{info, warn, error, debug};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::BaseProducer;
use tokio::runtime::{Builder, Runtime};
use crate::config::Config;
use crate::consumer::consumer_loop;
use crate::processor::{Processor, SerializedOutputMessage};
use crate::producer::producer_loop;

pub mod config;
mod consumer;
mod producer;
pub mod processor;

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
        message: SerializedOutputMessage,
    },
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

    show_streams_and_subscribe(&consumer, &streams)?;

    let (tx, rx) = bounded(config.internal_config.channel_capacity);
    runtime.spawn(async move {
        producer_loop(
            producer,
            rx,
            config.internal_config.queue_size,
            Duration::from_millis(config.internal_config.queue_slowdown_ms as u64),
        ).await;
    });

    consumer_loop(consumer, tx, runtime, streams).await
}

fn show_streams_and_subscribe(consumer: &StreamConsumer, streams: &HashMap<String, Stream>) -> Result<(), Box<dyn Error>> {
    streams.values()
        .for_each(|stream| {
           info!("Stream [{}] --> [{}]: {} processor(s).", stream.source_topic, stream.target_topic, stream.processors.len());
        });

    let topics: Vec<&str> = streams.keys()
        .map(|key| key.as_str())
        .collect();

    consumer.subscribe(&topics)?;

    Ok(())
}