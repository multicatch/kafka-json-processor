use std::collections::HashMap;
use std::{env, io};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Write;
use regex::Regex;
use crate::GeneratorError::InvalidGeneratorArguments;

pub struct ProcessorParams {
    pub function_name: String,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum GeneratorError {
    InvalidGeneratorArguments {
        args: Vec<String>,
        description: String,
    },
    RequiredConfigNotFound {
        function_name: String,
        field_name: String,
        description: Option<String>,
    },
    OtherError {
        description: String,
    },
}

impl GeneratorError {
    pub fn new_from<E: Error>(err: E) -> GeneratorError {
        GeneratorError::OtherError {
            description: format!("{err}")
        }
    }
}

impl Display for GeneratorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidGeneratorArguments { args, description } =>
                write!(f, "{} \nProvided arguments:{:?}", description, args),

            GeneratorError::RequiredConfigNotFound { function_name, field_name, description } =>
                write!(f, "Processor required config that was missing in template. Function: {}, missing field: {}. Description: {}",
                       function_name, field_name, description.clone().unwrap_or_else(|| "N/A".to_string())
                ),

            GeneratorError::OtherError { description } =>
                write!(f, "{}", description),
        }
    }
}

impl Error for GeneratorError {}

pub fn read_params() -> Result<ProcessorParams, GeneratorError> {
    let args: Vec<String> = env::args().collect();
    if args.is_empty() {
        return Err(InvalidGeneratorArguments {
            args,
            description: "No processor generator arguments specified. Specify at least 1 argument (function_name).".to_string(),
        });
    }

    if args.len() % 2 == 1 {
        return Err(InvalidGeneratorArguments {
            args,
            description: "Processor arguments are invalid. Cannot construct the config map (wrong number of arguments).".to_string(),
        });
    }

    let function_name = args[1].clone();

    let config: HashMap<String, String> = args.into_iter()
        .skip(2)
        .collect::<Vec<String>>()
        .chunks_exact(2)
        .into_iter()
        .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
        .collect();

    Ok(ProcessorParams {
        function_name,
        config,
    })
}

pub fn return_generated<F>(generate_function: F)
    where F: FnOnce(ProcessorParams) -> Result<String, GeneratorError> {

    let result = read_params()
        .and_then(generate_function);

    let mut output = String::new();
    match result {
        Ok(string) => {
            output.push_str("OK\n");
            output.push_str(&string);
        },
        Err(e) => {
            output.push_str("ERR\n");
            output.push_str(&format!("{e}"));
        },
    }

    {
        let mut lock = io::stdout().lock();
        lock.write_all(output.as_bytes()).unwrap();
        lock.flush().unwrap();
    }
}

/// Function to generate valid ObjectKey accessor from JSONPath.
///
/// The generated accessor can be used as a part of processor function.
/// This accessor is an interpreted version of JSONPath, which speeds up the processing.
///
/// ```rust
/// # use kjp_generator_plugin::json_path_to_object_key;
/// let string = json_path_to_object_key("$[0].phoneNumbers[1][test].type");
/// assert_eq!("&[Index(0), Key(\"phoneNumbers\".to_string()), Index(1), Key(\"test\".to_string()), Key(\"type\".to_string())]", string);
/// ```
pub fn json_path_to_object_key(jsonpath: &str) -> String {
    if !jsonpath.starts_with('$') {
        return format!("&[Key(\"{}\".to_string())]", jsonpath.escape_for_json())
    }

    let result: Vec<String> = Regex::new(r"[.\[\]]")
        .unwrap()
        .split(jsonpath)
        .skip(1)
        .filter(|s| !s.is_empty())
        .map(|s| match s.parse::<i64>() {
            Ok(num) => format!("Index({})", num),
            Err(_) => format!("Key(\"{}\".to_string())", s.escape_for_json()),
        })
        .collect();

    format!("&[{}]", result.join(", "))
}

pub trait JsonFieldName {
    fn escape_for_json(&self) -> String;
}

impl JsonFieldName for String {
    fn escape_for_json(&self) -> String {
        self.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }
}

impl JsonFieldName for &str {
    fn escape_for_json(&self) -> String {
        self.to_string()
            .escape_for_json()
    }
}