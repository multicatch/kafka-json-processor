use log::LevelFilter;
use serde_json::Value;
use kafka_json_processor::processor::{OutputMessage, ProcessingResult};
use kafka_json_processor::run_processor;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    run_processor("aaaa".to_string(), "bbbb".to_string(), &[&add_static_field]);
}

fn add_static_field(_input: &Value, message: &mut OutputMessage) -> ProcessingResult<()> {
    message.insert_str("abc".to_string(), "xyz".to_string());
    Ok(())
}
