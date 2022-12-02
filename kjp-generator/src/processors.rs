use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use log::{debug, info, trace};
use regex::{Captures, Regex};
use kjp_generator_plugin::json_path_to_object_key;
use crate::processors::ProcessorGenerationError::{GeneratorUnknown, RequiredConfigNotFound};
use crate::Stream;

#[derive(Eq, PartialEq, Debug)]
pub struct Processor {
    pub function_name: String,
    pub function_body: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ProcessorGenerationError {
    RequiredConfigNotFound {
        function_name: String,
        field_name: String,
        description: Option<String>,
    },
    GeneratorUnknown {
        name: String,
    },
    GeneratorError {
        description: String,
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
            ProcessorGenerationError::GeneratorError { description } =>
                write!(f, "Failed to generate function. {description}"),
        }
    }
}

impl Error for ProcessorGenerationError {}

/// Creates a map of code generators from given path.
///
/// This creates a dictionary of all available processor types.
/// Each generator is a program that generates a processor function body,
/// which will be used for processing JSON messages in a given stream.
///
/// This function will scan given path for files. Each file will be treated as a separate generator.
/// Each generator will have the name of corresponding file, but without extension.
/// In case of name conflict, last found generator will overwrite previous ones.
///
/// Please be careful what directory you use, as the generation process runs executables from the directory.
pub fn create_processor_generators<P: AsRef<Path>>(generators_path: P) -> Result<HashMap<String, PathBuf>, Box<dyn Error>> {
    info!("Loading available generators from: {:?}", generators_path.as_ref());

    let m: HashMap<String, PathBuf> = fs::read_dir(&generators_path)?
        .into_iter()
        .filter_map(|entry| entry.map_err(|err|  {
            info!("Cannot read file in [{:?}]: {}", generators_path.as_ref(), err);
            err
        }).ok())
        .filter(|entry| entry.file_type()
            .map(|e| e.is_file())
            .unwrap_or(false)
        )
        .filter_map(|entry| {
            let generator_name = entry.path().file_stem()?.to_str()?.to_string();
            let generator_path = entry.path();
            info!("Generator found: {} [{:?}]", generator_name, generator_path);

            Some((generator_name, generator_path))
        })
        .collect();

    Ok(m)
}

pub const FIELD_KEY: &str = "field";
pub const GENERATOR_KEY: &str = "generator";

/// Generates code for all processors in a stream.
///
/// This function generates a vec of [`Processor`], which contains function name and function source.
/// Each element represents a processor function which will be running as a part of the stream
/// in the target JSON processor executable.
pub fn generate_processors(stream: Stream, generators: &HashMap<String, PathBuf>) -> Result<Vec<Processor>, Box<dyn Error>> {
    debug!("Generating processors...");
    stream.processors.iter()
        .enumerate()
        .map(|(index, config)| {
            let generator_name = config.get(GENERATOR_KEY)
                .ok_or_else(|| RequiredConfigNotFound {
                    function_name: generate_function_name(&stream, index, "UNKNOWN"),
                    field_name: GENERATOR_KEY.to_string(),
                    description: None
                })?;

            let generator_path = generators.get(generator_name)
                .ok_or_else(|| GeneratorUnknown {
                    name: generator_name.to_string()
                })?;

            let function_name = generate_function_name(&stream, index, generator_name);
            debug!("Generating processor [{}] (generator: {})", function_name, generator_name);
            Ok(Processor {
                function_name: function_name.clone(),
                function_body: generate_source(generator_path, &function_name, config)?,
            })
        })
        .collect()
}

fn generate_function_name(stream: &Stream, index: usize, generator_name: &str) -> String {
    format!("{}_{}_{}_{}", stream.input_topic, stream.output_topic, index, generator_name)
}

fn generate_source<P: AsRef<OsStr>>(generator_path: P, function_name: &str, config: &HashMap<String, String>)
    -> Result<String, ProcessorGenerationError> {

    let generator_path_str = generator_path.as_ref().to_str().unwrap_or("");

    let mut args = vec![function_name];
    config.iter()
        .for_each(|(key, value)| {
            args.push(key);
            args.push(value);
        });

    trace!("Running [{:?}] with arguments {:?}", generator_path.as_ref(), args);

    let output = Command::new(&generator_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|err| ProcessorGenerationError::GeneratorError {
            description: format!("Generator error process failed [{}]: {}", generator_path_str, err),
        })?;

    let result = String::from_utf8(output.stdout)
        .map_err(|err| ProcessorGenerationError::GeneratorError {
            description: format!("Cannot read output of [{}] (not a valid UTF-8 string): {}", generator_path_str, err),
        })?;

    if !output.status.success() {
        return Err(ProcessorGenerationError::GeneratorError {
            description: format!("[{}] Process finished without success (status: [{}], output: [{}])",
                generator_path_str, output.status, result
            )
        })
    }

    trace!("[{} output] {}", generator_path_str, result);

    let jsonpath_regex = Regex::new("(##JSONPATH\\(.*\\)##)").unwrap();

    interpret_child_output(generator_path_str, result)
        .map(|source| {
            jsonpath_regex.replace_all(&source, |caps: &Captures| {
                trace!("Replacing {} with actual object tree accessor.", &caps[1]);
                let capture = &caps[1];
                let capture = &capture["##JSONPATH(".len()..capture.len()-")##".len()];
                json_path_to_object_key(capture)
            }).to_string()
        })
}

fn interpret_child_output(generator_path_str: &str, output: String) -> Result<String, ProcessorGenerationError> {
    if output.is_empty() {
        return Err(ProcessorGenerationError::GeneratorError {
            description: format!("[{}] Process output is empty.", generator_path_str),
        })
    }

    let string: String = output.lines()
        .skip(1)
        .collect::<Vec<&str>>()
        .join("\n");

    if output.starts_with("OK") {
        Ok(string)
    } else {
        Err(ProcessorGenerationError::GeneratorError {
            description: format!("[{}] {}.", generator_path_str, string),
        })
    }
}