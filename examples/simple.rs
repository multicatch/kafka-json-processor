use std::collections::HashMap;
use log::LevelFilter;
use serde_json::Value;
use kafka_json_processor::processor::{OutputMessage, ProcessingResult};
use kafka_json_processor::{run_processor, Stream};

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let mut streams = HashMap::new();
    streams.insert("aaaa".to_string(), Stream {
        source_topic: "aaaa".to_string(),
        target_topic: "bbbb".to_string(),
        processors: &[&add_static_field],
    });

    run_processor(streams);
}

fn add_static_field(_input: &Value, message: &mut OutputMessage) -> ProcessingResult<()> {
    message.insert_str("abc".to_string(), "xyz".to_string());
    Ok(())
}
