use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use log::{error, info, LevelFilter};
use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{Consumer, StreamConsumer};
use tokio::runtime::Builder;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    info!("Starting kafka-json-processor...");

    let runtime = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(4)
        .build()
        .unwrap();

    runtime.block_on(async {
        let config_path = "./processor.properties";
        let config_file = File::open(config_path).unwrap();

        let consumer_conf = create_client_config(config_file).unwrap();
        let consumer: StreamConsumer = consumer_conf.create().unwrap();

        consumer.subscribe(&["aaaa"]).unwrap();

        loop {
            match consumer.recv().await {
                Ok(message) => {
                    let payload = String::from_utf8_lossy(message.payload().unwrap());
                    info!("Received message: [{}:{}:{}] [{}] {}",
                        message.topic(), message.partition(), message.offset(), message.timestamp().to_millis().unwrap_or(0), payload
                    );
                }
                Err(e) => {
                    error!("Cannot consume message! Reason: {}", e);
                }
            }
        }
    });
}

fn create_client_config(file: File) -> Result<ClientConfig, Box<dyn Error>> {
    let mut consumer_config = ClientConfig::new();

    for line in BufReader::new(&file).lines() {
        let cur_line: String = line?.trim().to_string();
        if cur_line.starts_with('#') || cur_line.is_empty() {
            continue;
        }

        let key_value: Vec<&str> = cur_line.split('=').collect();
        let key = key_value.first().ok_or("malformed key")?;
        let value = key_value.get(1).ok_or("malformed value")?;

        consumer_config.set(key.to_string(), value.to_string());
    }

    Ok(consumer_config)
}