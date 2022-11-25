use std::collections::HashMap;
use log::LevelFilter;
use serde_json::Value;
use kafka_json_processor_core::processor::{ObjectTree, OutputMessage};
use kafka_json_processor_core::{run_processor, Stream};
use kafka_json_processor_core::error::ProcessingError;
use kafka_json_processor_core::formatters::json::pretty_json;
use kafka_json_processor_core::formatters::xml::pretty_xml;
use kafka_json_processor_core::processor::ObjectKey::Key;

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

fn add_static_field(_input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    message.insert_val(&[Key("static_field".to_string())], Value::String("example".to_string()))?;
    Ok(())
}

fn format_xml_field(input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    if let Some(xml) = input.get_val(&[Key("xml2".to_string())])?
        .as_str()
        .map(|v| v.to_string()) {

        message.insert_val(&[Key("pretty_xml".to_string())], Value::String(pretty_xml(xml)))?;
    }
    Ok(())
}

fn format_json_field(input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    if let Some(json) = input.get_val(&[Key("json".to_string())])?
        .as_str()
        .map(|v| v.to_string())  {

        message.insert_val(&[Key("pretty_json".to_string())], Value::String(pretty_json(json)))?;
    }
    Ok(())
}

#[cfg(test)]
mod simulations {
    use std::collections::HashMap;
    use log::LevelFilter;
    use kafka_json_processor_core::simulation::simulate_streams_from_default_folder;
    use kafka_json_processor_core::Stream;
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