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
    message.insert_str("static_field".to_string(), "example".to_string());
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

#[cfg(test)]
mod simulations {
    use std::collections::HashMap;
    use log::LevelFilter;
    use kafka_json_processor::simulation::simulate_streams_from_default_folder;
    use kafka_json_processor::Stream;
    use crate::{add_static_field, format_json_field, format_xml_field};

    #[test]
    fn example() {
        env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Trace)
            .init();

        let mut streams: HashMap<String, Stream> = HashMap::new();
        streams.insert("example".to_string(), Stream {
            source_topic: "example".to_string(),
            target_topic: "example".to_string(),
            processors: &[&add_static_field, &format_xml_field, &format_json_field],
        });

        simulate_streams_from_default_folder(streams);
    }
}