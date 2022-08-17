use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crossbeam_channel::bounded;
use log::{info, warn};
use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::BaseProducer;
use tokio::runtime::{Builder, Runtime};
use crate::consumer::consumer_loop;
use crate::processor::{Processor, SerializedOutputMessage};
use crate::producer::producer_loop;

pub mod consumer;
pub mod producer;
pub mod processor;

pub enum PendingMessage {
    Received,
    Processed {
        id: String,
        message: SerializedOutputMessage,
    },
}

#[derive(Clone)]
pub struct Config {
    pub consumer_config: ClientConfig,
    pub producer_config: ClientConfig,
    pub worker_threads: usize,
    pub channel_capacity: usize,
}

impl Config {
    pub fn read_from<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
        let file = File::open(path.as_ref())?;
        create_config(file)
    }
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
            .worker_threads(config.worker_threads)
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

    let (tx, rx) = bounded(50);
    runtime.spawn(async move {
        producer_loop(producer, &output_topic, rx).await;
    });

    consumer_loop(consumer, tx, runtime, processors).await
}

fn create_config(file: File) -> Result<Config, Box<dyn Error>> {
    let mut config = Config {
        consumer_config: ClientConfig::new(),
        producer_config: ClientConfig::new(),
        worker_threads: 4,
        channel_capacity: 50
    };

    for line in BufReader::new(&file).lines() {
        let current_line: String = line?
            .trim_start()
            .to_string();

        if current_line.starts_with('#') || current_line.trim().is_empty() {
            continue;
        }

        let key_value: Vec<&str> = current_line.split('=').collect();
        let key = key_value.first()
            .ok_or(format!("Illegal config entry (malformed key): {current_line}"))?;
        let value = key_value.get(1)
            .ok_or(format!("Illegal config entry (malformed value): {current_line}"))?;

        if key.starts_with("consumer.") {
            config.consumer_config.set(
                key.strip_prefix("consumer.").unwrap().to_string(),
                value.to_string(),
            );
        } else if key.starts_with("producer.") {
            config.producer_config.set(
                key.strip_prefix("producer.").unwrap().to_string(),
                value.to_string(),
            );
        } else if !key.starts_with("processor.") {
            config.consumer_config.set(key.to_string(), value.to_string());
            config.producer_config.set(key.to_string(), value.to_string());
        } else {
            match *key {
                "processor.channel.capacity" =>
                    config.channel_capacity = value.parse()?,

                "processor.worker.threads" =>
                    config.worker_threads = value.parse()?,

                _ => {}
            }
        }
    }

    Ok(config)
}