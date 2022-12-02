pub mod processors;
pub mod project;

use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs::{create_dir_all, File, read_to_string};
use std::io::Write;
use std::path::Path;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use crate::processors::{create_processor_generators, generate_processors};
use crate::project::{generate_cargo, generate_main};

/// Reads template, parses it and generated project based on it.
pub fn read_and_parse_and_generate<P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path>>(
    template_path: P1,
    output_path: P2,
    core_path: Option<String>,
    generators_path: P3,
) -> Result<(), Box<dyn Error>> {
    let template = read_template(template_path)?;

    info!("Generating output project in {}", output_path.as_ref().display());
    create_directories(&output_path)?;

    let output_path = output_path.as_ref();

    let cargo = generate_cargo(&template, core_path);
    let cargo_file = output_path.join("Cargo.toml");
    {
        info!("Generation of Cargo.toml finished. Writing Cargo.toml.");
        let mut cargo_file = File::create(cargo_file)?;
        cargo_file.write_all(cargo.as_bytes())?;
    }

    let generators = create_processor_generators(generators_path)?;
    let streams = template.streams.into_iter()
        .map(|stream| {
            let processors = generate_processors(stream.clone(), &generators)?;
            Ok(((stream.input_topic, stream.output_topic), processors))
        })
        .collect::<Result<BTreeMap<_, _>, Box<dyn Error>>>()?;

    let main = generate_main(streams);
    let main_file = output_path.join("src").join("main.rs");
    {
        info!("Generation of main.rs finished. Writing main.rs.");
        let mut main_file = File::create(main_file)?;
        main_file.write_all(main.as_bytes())?;
    }

    info!("All files generated.");

    Ok(())
}

fn create_directories<P: AsRef<Path>>(base_path: P) -> Result<(), Box<dyn Error>> {
    let path = base_path.as_ref();
    debug!("Creating directory: {}", path.display());
    create_dir_all(path)?;

    let src_dir = path.join("src");
    debug!("Creating directory: {}", src_dir.display());
    create_dir_all(&src_dir)?;

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
                            ("kind".to_string(), "static_field".to_string()),
                            ("field".to_string(), "$.hello".to_string()),
                            ("value".to_string(), "world".to_string()),
                        ]),
                        HashMap::from([
                            ("kind".to_string(), "copy_field".to_string()),
                            ("source_field".to_string(), "$.abc[1]".to_string()),
                            ("target_field".to_string(), "$.def".to_string()),
                        ]),
                    ]
                }
            ]
        }, result.unwrap())
    }
}