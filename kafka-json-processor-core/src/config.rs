use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use log::warn;
use rdkafka::ClientConfig;

#[derive(Clone)]
pub struct Config {
    pub consumer_config: ClientConfig,
    pub producer_config: ClientConfig,
    pub internal_config: InternalConfig
}

#[derive(Clone)]
pub struct InternalConfig {
    pub worker_threads: usize,
    pub channel_capacity: usize,
    pub queue_slowdown_ms: usize,
    pub queue_size: usize,
    pub journal_enabled: bool,
    pub journal_path: String,
}

impl Default for InternalConfig {
    fn default() -> Self {
        InternalConfig {
            worker_threads: 4,
            channel_capacity: 50,
            queue_slowdown_ms: 10_000, // 10 s
            queue_size: 100_000,
            journal_enabled: true,
            journal_path: "./kjp_journal".to_string(),
        }
    }
}

impl Config {
    pub fn read_from<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
        let file = File::open(path.as_ref())?;
        let mut config = Config {
            consumer_config: ClientConfig::new(),
            producer_config: ClientConfig::new(),
            internal_config: InternalConfig::default(),
        };

        for line in BufReader::new(&file).lines() {
            let line: String = line?
                .trim_start()
                .to_string();

            if should_ignore(&line) {
                continue;
            }

            let (key, value) = key_value(&line)?;

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
            } else if key.starts_with("processor.") {
                set_internal(key, value, &mut config.internal_config)?;
            } else {
                config.consumer_config.set(key.to_string(), value.to_string());
                config.producer_config.set(key.to_string(), value.to_string());
            }
        }

        Ok(config)
    }
}


fn should_ignore(line: &str) -> bool {
    line.starts_with('#') || line.trim().is_empty()
}

fn key_value(line: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let key_value: Vec<&str> = line.split('=').collect();
    let key = key_value.first()
        .ok_or(format!("Illegal config entry (malformed key): {line}"))?;
    let value = key_value.get(1)
        .ok_or(format!("Illegal config entry (malformed value): {line}"))?;

    Ok((key, value))
}

fn set_internal(key: &str, value: &str, config: &mut InternalConfig) -> Result<(), Box<dyn Error>> {
    match key {
        "processor.channel.capacity" =>
            config.channel_capacity = value.parse()?,

        "processor.worker.threads" =>
            config.worker_threads = value.parse()?,

        "processor.queue.size" =>
            config.queue_size = value.parse()?,

        "processor.queue.slowdown.ms" =>
            config.queue_slowdown_ms = value.parse()?,

        "processor.journal.path" =>
            config.journal_path = value.parse()?,
        
        "processor.journal.enabled" =>
            config.journal_enabled = value.parse()?,

        _ => {
            warn!("Unknown config option: {key}={value}. Ignoring.")
        }
    }
    Ok(())
}