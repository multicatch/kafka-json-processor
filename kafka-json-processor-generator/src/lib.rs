mod processors;

use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::processors::{create_processor_generators, generate_processors};

pub fn read_and_parse_and_generate<P1: AsRef<Path>, P2: AsRef<Path>>(
    template_path: P1, _output_path: P2
) -> Result<(), Box<dyn Error>> {
    let template = read_template(template_path)?;

    let generators = create_processor_generators();

    template.streams.into_iter()
        .for_each(|stream| {
            let _ = generate_processors(stream, &generators);
        });

    Ok(())
}

fn read_template<P: AsRef<Path>>(path: P) -> Result<Template, Box<dyn Error>> {
    let file_content = read_to_string(path)?;
    let template = serde_yaml::from_str(&file_content)?;
    Ok(template)
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Template {
    name: String,
    streams: Vec<Stream>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Stream {
    input_topic: String,
    output_topic: String,
    processors: Vec<HashMap<String, String>>
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::{read_template, Stream, Template};

    #[test]
    fn should_read() {
        let result = read_template("test-examples/correct.yaml");

        assert!(result.is_ok());
        assert_eq!(Template {
            name: "Example processor".to_string(),
            streams: vec![
                Stream {
                    input_topic: "in".to_string(),
                    output_topic: "out".to_string(),
                    processors: vec![
                        HashMap::from([
                            ("kind".to_string(), "xml_formatter".to_string()),
                            ("field".to_string(), "sample_xml".to_string())
                        ])
                    ]
                }
            ]
        }, result.unwrap())
    }
}