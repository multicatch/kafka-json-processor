/// Example of kafka-json-processor-generator plugin, in Rust.
///
/// This plugin has the same purpose as ./kjp-generator-generators/static_field.sh.
/// The following code will generate a processor that adds a static field to the output JSON.
/// See the test at the end of this file for an example of generated code.
///
/// This executable needs to be run with the following arguments:
/// `./static_field $function_name field $field_name value $static_value`
/// Example:
/// `./static_field static_field_function field '$.hello' value 'Greetings, folks!'`
///
/// It will be activated by placing a compiled executable in the generators directory
/// (default: ./generators, can be set with --generators-path <GENERATORS_PATH> when using kjp_generator).

use kjp_generator_plugin::{GeneratorError, json_path_to_object_key, JsonFieldName, ProcessorParams, return_generated};
use kjp_generator_plugin::GeneratorError::RequiredConfigNotFound;

fn main() {
    return_generated(static_field);
}

/// Generates a processor that adds a static field to JSON
///
/// This generates a function that adds a static field to a JSON from topic.
/// Available config options:
///  - "field" - static field name
///  - "value" - a string to put in this field
pub fn static_field(params: ProcessorParams) -> Result<String, GeneratorError> {
    let function_name = params.function_name;
    let config = params.config;

    let target_field = json_path_to_object_key(
        config.get("field")
            .ok_or_else(|| RequiredConfigNotFound {
                function_name: function_name.to_string(),
                field_name: "field".to_string(),
                description: None,
            })?
    );

    let value = config.get("value")
        .ok_or_else(|| RequiredConfigNotFound {
            function_name: function_name.to_string(),
            field_name: "value".to_string(),
            description: None,
        })?
        .escape_for_json();

    Ok(FUNCTION_TEMPLATE
        .replace(FUNCTION_NAME, &function_name)
        .replace(TARGET_FIELD, &target_field)
        .replace(VALUE, &value)
    )
}

const FUNCTION_TEMPLATE: &str = r##"
fn %%FUNCTION_NAME%%(_input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    message.insert_val(%%TARGET_FIELD%%, Value::String("%%VALUE%%".to_string()))?;
    Ok(())
}
"##;

const FUNCTION_NAME: &str = "%%FUNCTION_NAME%%";
const TARGET_FIELD: &str = "%%TARGET_FIELD%%";
const VALUE: &str = "%%VALUE%%";

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use kjp_generator_plugin::ProcessorParams;
    use crate::static_field;

    #[test]
    fn should_generate_static_field() {
        let mut config = HashMap::new();
        config.insert("field".to_string(), "$.example[0]".to_string());
        config.insert("value".to_string(), "abcdef".to_string());
        let result = static_field(ProcessorParams {
            function_name: "abc1".to_string(),
            config,
        });
        assert_eq!(Ok(r##"
fn abc1(_input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    message.insert_val(&[Key("example".to_string()), Index(0)], Value::String("abcdef".to_string()))?;
    Ok(())
}
"##.to_string()), result);
    }
}