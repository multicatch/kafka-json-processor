use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;
use log::{error, info, LevelFilter};
use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::producer::{Producer, BaseProducer, BaseRecord};
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

        let (consumer_conf, producer_conf) = create_config(config_file).unwrap();
        let consumer: StreamConsumer = consumer_conf.create().unwrap();
        let producer: BaseProducer = producer_conf.create().unwrap();

        consumer.subscribe(&["aaaa"]).unwrap();

        loop {
            match consumer.recv().await {
                Ok(message) => {
                    let payload = String::from_utf8_lossy(message.payload().unwrap());
                    let key = format!("{}:{}", message.topic(), message.partition());
                    info!("Received message: [{}:{}] [{}] {}",
                        key, message.offset(), message.timestamp().to_millis().unwrap_or(0), payload
                    );
                    send(&producer, "bbbb", key, payload.into()).unwrap();
                }
                Err(e) => {
                    error!("Cannot consume message! Reason: {}", e);
                }
            }
        }
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


fn send(producer: &BaseProducer, topic: &str, key: String, payload: String) -> Result<(), KafkaError> {
    let result = producer.send(
        BaseRecord::to(topic)
            .key(&key)
            .payload(&payload),
    );
    if let Err((e, _)) = &result {
        error!("Could not send message [{}]! Reason: {}, queue: {}", key, e, producer.in_flight_count());
    } else {
        producer.poll(Duration::from_millis(0));
    }

    result.map_err(|(e, _)| e)
}