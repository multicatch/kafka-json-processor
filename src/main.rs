use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use log::{info, LevelFilter};
use rdkafka::{ClientConfig};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{BaseProducer};
use tokio::runtime::Builder;
use kafka_json_processor::consumer::consumer_loop;
use kafka_json_processor::producer::producer_loop;

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

        let (consumer_conf, producer_conf) = create_config(config_file).unwrap();
        let consumer: StreamConsumer = consumer_conf.create().unwrap();
        let producer: BaseProducer = producer_conf.create().unwrap();

        consumer.subscribe(&["aaaa"]).unwrap();

        let (tx, rx) = mpsc::channel();
        runtime.spawn(async move {
            producer_loop(producer, "bbbb", rx).await;
        });
        consumer_loop(consumer, tx).await;
    });
}

fn create_config(file: File) -> Result<(ClientConfig, ClientConfig), Box<dyn Error>> {
    let mut consumer_config = ClientConfig::new();
    let mut producer_config = ClientConfig::new();

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
            consumer_config.set(
                key.strip_prefix("consumer.").unwrap().to_string(),
                value.to_string(),
            );
        } else if key.starts_with("producer.") {
            producer_config.set(
                key.strip_prefix("producer.").unwrap().to_string(),
                value.to_string(),
            );
        } else {
            consumer_config.set(key.to_string(), value.to_string());
            producer_config.set(key.to_string(), value.to_string());
        }
    }

    Ok((consumer_config, producer_config))
}