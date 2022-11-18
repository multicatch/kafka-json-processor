mod static_field;
mod copy_field;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::processors::ProcessorGenerationError::{GeneratorUnknown, OtherError, RequiredConfigNotFound};
use crate::Stream;

#[derive(Eq, PartialEq, Debug)]
pub struct Processor {
    pub function_name: String,
    pub function_body: String,
}

#[derive(Debug)]
pub enum ProcessorGenerationError {
    RequiredConfigNotFound {
        function_name: String,
        field_name: String,
        description: Option<String>,
    },
    GeneratorUnknown {
        name: String,
    },
    #[allow(dead_code)] // API for any other case, currently unused, but needed for extensions
    OtherError {
        function_name: String,
        error: Box<dyn Error>
    }
}

impl Display for ProcessorGenerationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RequiredConfigNotFound { function_name, field_name, description } =>
                write!(f, "Processor required config that was missing in template. Function: {}, missing field: {}. Description: {}",
                       function_name, field_name, description.clone().unwrap_or_else(|| "N/A".to_string())
                ),
            GeneratorUnknown { name } =>
                write!(f, "Failed to generate function. Generator is unknown: {}", name),

            OtherError { function_name, error } =>
                write!(f, "Processor generation error. Function: {}. Error: {}", function_name, error)
        }
    }
}

impl Error for ProcessorGenerationError {}

pub type ProcessorFn = &'static (dyn Fn(&str, &HashMap<String, String>) -> Result<String, ProcessorGenerationError> + Sync + Send);

/// Creates a map of code generators.
///
/// This creates a dictionary of all available processor types.
/// Each generator is a function that generates a processor function body,
/// which will be used for processing JSON messages in a given stream.
pub fn create_processor_generators() -> HashMap<String, ProcessorFn> {
    let mut m: HashMap<String, ProcessorFn> = HashMap::new();
    m.insert("static_field".to_string(), &static_field::static_field);
    m.insert("copy_field".to_string(), &copy_field::copy_field);
    m
}

pub const FIELD_KEY: &str = "field";
pub const KIND_KEY: &str = "kind";

pub fn generate_processors(stream: Stream, generators: &HashMap<String, ProcessorFn>) -> Result<Vec<Processor>, Box<dyn Error>> {
    stream.processors.iter()
        .enumerate()
        .map(|(index, config)| {
            let kind = config.get(KIND_KEY)
                .ok_or_else(|| RequiredConfigNotFound {
                    function_name: generate_function_name(&stream, index, "UNKNOWN"),
                    field_name: KIND_KEY.to_string(),
                    description: None
                })?;

            let generate_source = generators.get(kind)
                .ok_or_else(|| GeneratorUnknown {
                    name: kind.to_string()
                })?;

            let function_name = generate_function_name(&stream, index, kind);
            Ok(Processor {
                function_name: function_name.clone(),
                function_body: generate_source(&function_name, config)?,
            })
        })
        .collect()
}

fn generate_function_name(stream: &Stream, index: usize, kind: &str) -> String {
    format!("{}_{}_{}_{}", stream.input_topic, stream.output_topic, index, kind)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::{generate_processors, Stream};
    use crate::processors::{Processor, ProcessorFn, ProcessorGenerationError};

    #[test]
    fn should_generate_function() {
        let stream = Stream {
            input_topic: "abc".to_string(),
            output_topic: "def".to_string(),
            processors: vec![
                HashMap::from([
                    ("kind".to_string(), "test_generator".to_string()),
                ])
            ]
        };
        let mut generators: HashMap<String, ProcessorFn> = HashMap::new();
        generators.insert("test_generator".to_string(), &test_generator);

        let result = generate_processors(stream, &generators);
        assert!(result.is_ok());
        assert_eq!(vec![
            Processor {
                function_name: "abc_def_0_test_generator".to_string(),
                function_body: "result function".to_string()
            },
        ], result.unwrap());
    }

    fn test_generator(_function_name: &str, _config: &HashMap<String, String>) -> Result<String, ProcessorGenerationError> {
        Ok("result function".to_string())
    }
}