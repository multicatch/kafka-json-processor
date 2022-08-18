use std::error::Error;
use std::time::Duration;
use crossbeam_channel::bounded;
use log::{info, warn};
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

pub enum PendingMessage {
    Received,
    Processed {
        id: String,
        message: SerializedOutputMessage,
    },
}

pub fn run_processor(input_topic: String, output_topic: String, processors: &'static [Processor]) {
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
                input_topic.clone(),
                output_topic.clone(),
                processors
            ).await
        }) {
            warn!("RUNTIME ERROR: {:?}. Restarting...", e);
        }
    }
}

async fn run_processing_tasks(
    runtime: &Runtime,
    config: Config,
    input_topic: String,
    output_topic: String,
    processors: &'static [Processor]
) -> Result<(), Box<dyn Error>> {
    let consumer: StreamConsumer = config.consumer_config.create()?;
    let producer: BaseProducer = config.producer_config.create()?;

    consumer.subscribe(&[&input_topic])?;

    let (tx, rx) = bounded(config.internal_config.channel_capacity);
    runtime.spawn(async move {
        producer_loop(
            producer,
            &output_topic,
            rx,
            config.internal_config.queue_size,
            Duration::from_millis(config.internal_config.queue_slowdown_ms as u64)
        ).await;
    });

    consumer_loop(consumer, tx, runtime, processors).await
}
