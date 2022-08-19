use std::collections::HashMap;
use std::error::Error;
use std::{fs, io};
use std::io::ErrorKind;
use std::path::Path;
use std::time::Instant;
use log::{debug, error, info, warn};
use serde_json::Value;
use crate::processor::process_payload;
use crate::Stream;

pub fn simulate_streams_from_default_folder(streams: HashMap<String, Stream>) {
    simulate_streams(streams, "simulations");
}

pub fn simulate_streams<P: AsRef<Path>>(streams: HashMap<String, Stream>, base_path: P) {
    for (source, stream) in streams {
        if let Err(e) = find_samples_and_simulate(&base_path.as_ref().join(source), &stream) {
            error!("Error during simulation: {}", e);
        }
    }
}

fn find_samples_and_simulate<P: AsRef<Path>>(simulation_path: P, stream: &Stream) -> Result<(), Box<dyn Error>> {
    let path = simulation_path.as_ref();
    let source = &stream.source_topic;

    info!("Stream [{}] --> [{}]: Beginning simulation. Looking for samples in: {}",
        source, stream.target_topic, path.display());

    if !path.exists() {
        warn!("Simulation dir [{}] does not exist, aborting simulation.", path.display());
        return Ok(());
    }

    for (i, entry) in fs::read_dir(path)?.enumerate() {
        let entry = entry?;
        let file_path = entry.path();

        let (input, expected) = read_data_from(&file_path)?;

        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let msg_id = format!("simulation_{}_{}", source, file_name);

        info!("====== [{}#{}: {}] ======", source, i, file_name);
        run_single_simulation(msg_id, stream, input, expected);
    }

    Ok(())
}

const INPUT_HEADER: &str = "[Input]";
const EXPECTED_HEADER: &str = "[Expected]";

fn read_data_from<P: AsRef<Path>>(path: P) -> Result<(String, String), io::Error> {
    let file_input = fs::read_to_string(&path)?;
    let mut lines = file_input.lines();

    if !matches!(&lines.next(), Some(INPUT_HEADER)) {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("File format error [{}]: Missing [{INPUT_HEADER}] part.", path.as_ref().display()
        )));
    }

    let input = lines.clone()
        .take_while(|line| line != &"[Expected]");

    let input_lines = input.clone().count();

    let input: String = input
        .map(|s| s.to_string() + "\n")
        .collect();

    let output: String = lines
        .skip(input_lines + 1)
        .map(|s| s.to_string() + "\n")
        .collect();

    if input.trim().is_empty() || output.trim().is_empty() {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("File format error [{}]: Missing [{INPUT_HEADER}] or [{EXPECTED_HEADER}] part.", path.as_ref().display()
        )));
    }

    Ok((input, output))
}

fn run_single_simulation(msg_id: String, stream: &Stream, input: String, expected_output: String) {
    let interval = Instant::now();

    debug!("[{msg_id}] Simulation started.");
    let result = process_payload(msg_id.clone(), input.as_bytes(), stream.processors);
    info!("[{msg_id}] Simulation finished in {}ms", interval.elapsed().as_millis());

    if let Err(e) = &result {
        error!("[{msg_id}] FAILED: {}", e);
        assert!(result.is_ok());
    }

    let message = result.unwrap();

    let expected: Value = serde_json::from_str(&expected_output).unwrap();
    let actual: Value = serde_json::from_str(&message.message).unwrap();

    assert_eq!(actual, expected,
               "FAILED. Expected: {}, actual: {}",
               expected_output.replace('\n', ""),
               message.message.replace('\n', "")
    );
    info!("[{}] PASSED.", msg_id);
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use log::LevelFilter;
    use serde_json::Value;
    use crate::formatters::json::pretty_json;
    use crate::formatters::xml::pretty_xml;
    use crate::processor::{OutputMessage, ProcessingResult};
    use crate::simulation::{read_data_from, simulate_streams_from_default_folder};
    use crate::Stream;

    #[test]
    fn try_simulate_streams() {
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

    #[test]
    fn should_read_data() {
        let result = read_data_from("simulations/example/basic.sample");
        assert!(result.is_ok());
        let (input, expected) = result.unwrap();

        assert_eq!(input, r#"{
    "xml": "<?xml version=\"1.0\" encoding=\"UTF-8\"?><breakfast_menu><!-- comment --><!-- comment after comment --><food>  <name>Belgian Waffles</name><!-- comment 2 -->    <price>$5.95</price><description>Two of our famous Belgian Waffles with plenty of real maple syrup</description><calories>650</calories></food><food><name>Strawberry Belgian Waffles</name><price>$7.95</price><description>Light Belgian waffles covered with strawberries and whipped cream</description><calories>900</calories></food><food><name>Berry-Berry Belgian Waffles</name><price>$8.95</price><description>Light Belgian waffles covered with an assortment of fresh berries and whipped cream</description><calories>900</calories></food><food><name>French Toast</name><price>$4.50</price><description>Thick slices made from our homemade sourdough bread</description><calories>600</calories></food><food><name>Homestyle Breakfast</name><price>$6.95</price><description>Two eggs, bacon or sausage, toast, and our ever-popular hash browns</description><calories>950</calories></food></breakfast_menu>",
    "json": "{\"glossary\":[{\"title\":\"example glossary\",\"GlossDiv\":{\"title\":\"S\",\"available\":true,\"number\":123.22,\"GlossList\":{\"GlossEntry\":{\"ID\":\"SGML\",\"SortAs\":\"SGML\",\"GlossTerm\":\"Standard Generalized Markup Language\",\"Acronym\":\"SGML\",\"Abbrev\":\"ISO 8879:1986\",\"GlossDef\":{\"para\":\"A meta-markup language, used to create markup languages such as \\\"DocBook\\\".\",\"GlossSeeAlso\":[\"GML\",\"XML\"]},\"GlossSee\":\"markup\"}}}}]}",
    "other_field": "ignored"
}

"#);

        assert_eq!(expected, r#"{
	"static_field": "example",
	"pretty_json": "{\n  \"glossary\": [\n    {\n      \"title\": \"example glossary\",\n      \"GlossDiv\": {\n        \"title\": \"S\",\n        \"available\": true,\n        \"number\": 123.22,\n        \"GlossList\": {\n          \"GlossEntry\": {\n            \"ID\": \"SGML\",\n            \"SortAs\": \"SGML\",\n            \"GlossTerm\": \"Standard Generalized Markup Language\",\n            \"Acronym\": \"SGML\",\n            \"Abbrev\": \"ISO 8879:1986\",\n            \"GlossDef\": {\n              \"para\": \"A meta-markup language, used to create markup languages such as \\\"DocBook\\\".\",\n              \"GlossSeeAlso\": [\n                \"GML\",\n                \"XML\"\n              ]\n            },\n            \"GlossSee\": \"markup\"\n          }\n        }\n      }\n    }\n  ]\n}",
	"pretty_xml": "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<breakfast_menu>\n<!-- comment -->\n<!-- comment after comment -->\n  <food>  \n    <name>Belgian Waffles</name>\n  <!-- comment 2 -->    \n    <price>$5.95</price>\n    <description>Two of our famous Belgian Waffles with plenty of real maple syrup</description>\n    <calories>650</calories>\n  </food>\n  <food>\n    <name>Strawberry Belgian Waffles</name>\n    <price>$7.95</price>\n    <description>Light Belgian waffles covered with strawberries and whipped cream</description>\n    <calories>900</calories>\n  </food>\n  <food>\n    <name>Berry-Berry Belgian Waffles</name>\n    <price>$8.95</price>\n    <description>Light Belgian waffles covered with an assortment of fresh berries and whipped cream</description>\n    <calories>900</calories>\n  </food>\n  <food>\n    <name>French Toast</name>\n    <price>$4.50</price>\n    <description>Thick slices made from our homemade sourdough bread</description>\n    <calories>600</calories>\n  </food>\n  <food>\n    <name>Homestyle Breakfast</name>\n    <price>$6.95</price>\n    <description>Two eggs, bacon or sausage, toast, and our ever-popular hash browns</description>\n    <calories>950</calories>\n  </food>\n</breakfast_menu>"
}
"#);
    }
}