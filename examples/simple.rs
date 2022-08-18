use std::collections::HashMap;
use log::LevelFilter;
use serde_json::Value;
use kafka_json_processor::processor::{OutputMessage, ProcessingResult};
use kafka_json_processor::{run_processor, Stream};
use kafka_json_processor::formatters::json::pretty_json;
use kafka_json_processor::formatters::xml::pretty_xml;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let mut streams = HashMap::new();
    streams.insert("aaaa".to_string(), Stream {
        source_topic: "aaaa".to_string(),
        target_topic: "bbbb".to_string(),
        processors: &[&add_static_field, &format_xml_field, &format_json_field],
    });

    run_processor(streams);
}

fn add_static_field(_input: &Value, message: &mut OutputMessage) -> ProcessingResult<()> {
    message.insert_str("abc".to_string(), "xyz".to_string());
    Ok(())
}

fn format_xml_field(input: &Value, message: &mut OutputMessage) -> ProcessingResult<()> {
    if let Some(xml) = input.get("xml")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string()) {

        message.insert_str("pretty_xml".to_string(), pretty_xml(xml));
    }
    Ok(())
}

fn format_json_field(input: &Value, message: &mut OutputMessage) -> ProcessingResult<()> {
    if let Some(json) = input.get("json")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string()) {

        message.insert_str("pretty_json".to_string(), pretty_json(json));
    }
    Ok(())
}