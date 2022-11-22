use std::collections::BTreeMap;
use crate::processors::Processor;
use crate::Template;

pub fn generate_cargo(template: &Template, core_path: Option<String>) -> String {
    CARGO_TOML
        .replace(PROJECT_NAME, &template.name
            .replace(' ', "-")
            .to_lowercase(),
        )
        .replace(CORE_VERSION, &core_path
            .map(|path| format!("{{ path = \"{}\" }}", path))
            .unwrap_or_else(|| "\"0.1.0\"".to_string())
        )
}

pub fn generate_main(streams: BTreeMap<(String, String), Vec<Processor>>) -> String {
    let streams_config: String = streams.iter()
        .map(|((input_topic, output_topic), processors)| {
            let processor_list: String = processors.iter()
                .map(|processor| format!("&{}, ", processor.function_name))
                .collect();

            SINGLE_STREAM
                .replace(INPUT_TOPIC, input_topic)
                .replace(OUTPUT_TOPIC, output_topic)
                .replace(PROCESSORS, &processor_list)
        })
        .collect();

    let function_names: String =  streams.iter()
        .flat_map(|(_, processors)| {
            processors.iter()
                .map(|p| format!("{}, ",p.function_name))
        })
        .collect();

    let functions: String = streams.into_iter()
        .flat_map(|(_, processors)| {
            processors.into_iter()
                .map(|p| p.function_body)
        })
        .collect();

    format!("{}{}{}",
            MAIN.replace(STREAMS, &streams_config),
            functions,
            SIMULATIONS
                .replace(FUNCTION_IMPORTS, &function_names)
                .replace(STREAMS, &streams_config)
    )
}

const CARGO_TOML: &str = r##"[package]
name = "%%PROJECT_NAME%%"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
log = "0.4.17"
env_logger = "0.9.0"
serde_json = "1.0.83"
kafka_json_processor_core = %%CORE_VERSION%%
"##;

const PROJECT_NAME: &str = "%%PROJECT_NAME%%";
const CORE_VERSION: &str = "%%CORE_VERSION%%";

const MAIN: &str = r##"#![allow(unused_variables, unused_imports)]

use std::collections::HashMap;
use log::{LevelFilter, trace, debug, error, info, warn};
use serde_json::Value;
use kafka_json_processor_core::processor::{ObjectKey, ObjectTree, OutputMessage, ProcessingResult};
use kafka_json_processor_core::{run_processor, Stream};
use kafka_json_processor_core::formatters::json::pretty_json;
use kafka_json_processor_core::formatters::xml::pretty_xml;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let mut streams = HashMap::new();
%%STREAMS%%

    run_processor(streams);
}
"##;

const STREAMS: &str = "%%STREAMS%%";

const SINGLE_STREAM: &str = r##"
    streams.insert("%%INPUT_TOPIC%%_%%OUTPUT_TOPIC%%".to_string(), Stream {
        source_topic: "%%INPUT_TOPIC%%".to_string(),
        target_topic: "%%OUTPUT_TOPIC%%".to_string(),
        processors: &[%%PROCESSORS%%],
    });"##;

const INPUT_TOPIC: &str = "%%INPUT_TOPIC%%";
const OUTPUT_TOPIC: &str = "%%OUTPUT_TOPIC%%";
const PROCESSORS: &str = "%%PROCESSORS%%";

const SIMULATIONS: &str = r##"

#[cfg(test)]
mod simulations {
    use std::collections::HashMap;
    use log::LevelFilter;
    use kafka_json_processor_core::simulation::simulate_streams_from_default_folder;
    use kafka_json_processor_core::Stream;
    use crate::{%%FUNCTION_IMPORTS%%};

    #[test]
    fn simulate_streams() {
        env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .init();

        let mut streams: HashMap<String, Stream> = HashMap::new();
%%STREAMS%%

        simulate_streams_from_default_folder(streams);
    }
}
"##;

const FUNCTION_IMPORTS: &str = "%%FUNCTION_IMPORTS%%";


#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use crate::processors::Processor;
    use crate::project::{generate_cargo, generate_main};
    use crate::Template;

    #[test]
    fn should_generate_main() {
        let mut streams = BTreeMap::new();
        streams.insert(("abc".to_string(), "def".to_string()), vec![
            Processor {
                function_name: "function_1".to_string(),
                function_body: r##"
fn function_1(_input: &Value, _message: &mut OutputMessage) -> ProcessingResult<()> {
    Ok(())
}"##.to_string(),
            },
            Processor {
                function_name: "function_2".to_string(),
                function_body: r##"
fn function_2(_input: &Value, _message: &mut OutputMessage) -> ProcessingResult<()> {
    Ok(())
}"##.to_string(),
            },
        ]);

        streams.insert(("topic1".to_string(), "topic2".to_string()), vec![
            Processor {
                function_name: "function_3".to_string(),
                function_body: r##"
fn function_3(_input: &Value, _message: &mut OutputMessage) -> ProcessingResult<()> {
    Ok(())
}"##.to_string(),
            },
            Processor {
                function_name: "function_4".to_string(),
                function_body: r##"
fn function_4(_input: &Value, _message: &mut OutputMessage) -> ProcessingResult<()> {
    Ok(())
}"##.to_string(),
            },
        ]);

        let main = generate_main(streams);
        assert_eq!(r##"#![allow(unused_variables, unused_imports)]

use std::collections::HashMap;
use log::{LevelFilter, trace, debug, error, info, warn};
use serde_json::Value;
use kafka_json_processor_core::processor::{ObjectKey, ObjectTree, OutputMessage, ProcessingResult};
use kafka_json_processor_core::{run_processor, Stream};
use kafka_json_processor_core::formatters::json::pretty_json;
use kafka_json_processor_core::formatters::xml::pretty_xml;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let mut streams = HashMap::new();

    streams.insert("abc_def".to_string(), Stream {
        source_topic: "abc".to_string(),
        target_topic: "def".to_string(),
        processors: &[&function_1, &function_2, ],
    });
    streams.insert("topic1_topic2".to_string(), Stream {
        source_topic: "topic1".to_string(),
        target_topic: "topic2".to_string(),
        processors: &[&function_3, &function_4, ],
    });

    run_processor(streams);
}

fn function_1(_input: &Value, _message: &mut OutputMessage) -> ProcessingResult<()> {
    Ok(())
}
fn function_2(_input: &Value, _message: &mut OutputMessage) -> ProcessingResult<()> {
    Ok(())
}
fn function_3(_input: &Value, _message: &mut OutputMessage) -> ProcessingResult<()> {
    Ok(())
}
fn function_4(_input: &Value, _message: &mut OutputMessage) -> ProcessingResult<()> {
    Ok(())
}

#[cfg(test)]
mod simulations {
    use std::collections::HashMap;
    use log::LevelFilter;
    use kafka_json_processor_core::simulation::simulate_streams_from_default_folder;
    use kafka_json_processor_core::Stream;
    use crate::{function_1, function_2, function_3, function_4, };

    #[test]
    fn simulate_streams() {
        env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .init();

        let mut streams: HashMap<String, Stream> = HashMap::new();

    streams.insert("abc_def".to_string(), Stream {
        source_topic: "abc".to_string(),
        target_topic: "def".to_string(),
        processors: &[&function_1, &function_2, ],
    });
    streams.insert("topic1_topic2".to_string(), Stream {
        source_topic: "topic1".to_string(),
        target_topic: "topic2".to_string(),
        processors: &[&function_3, &function_4, ],
    });

        simulate_streams_from_default_folder(streams);
    }
}
"##, main);
    }

    #[test]
    fn should_generate_cargo() {
        let actual = generate_cargo(&Template {
            name: "sample-project Abcdef".to_string(),
            streams: vec![],
        }, None);

        assert_eq!(r##"[package]
name = "sample-project-abcdef"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
log = "0.4.17"
env_logger = "0.9.0"
serde_json = "1.0.83"
kafka_json_processor_core = "0.1.0"
"##, actual);
    }
}