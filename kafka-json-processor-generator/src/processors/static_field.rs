use std::collections::HashMap;
use crate::processors::{FIELD_KEY, ProcessorGenerationError};
use crate::processors::ProcessorGenerationError::RequiredConfigNotFound;

/// Generates a processor that adds a static field to JSON
///
/// This generates a function that adds a static field to a JSON from topic.
/// Available config options:
///  - "field" - static field name
///  - "value" - a string to put in this field
pub fn static_field(function_name: &str, config: &HashMap<String, String>) -> Result<String, ProcessorGenerationError> {
    let field_name = config.get(FIELD_KEY)
        .ok_or_else(|| RequiredConfigNotFound {
            function_name: function_name.to_string(),
            field_name: FIELD_KEY.to_string(),
            description: None,
        })?
        .replace('\"', "\\\"");

    let value = config.get("value")
        .ok_or_else(|| RequiredConfigNotFound {
            function_name: function_name.to_string(),
            field_name: "value".to_string(),
            description: None
        })?
        .replace('\"', "\\\"");

    Ok(FUNCTION_TEMPLATE
        .replace(FUNCTION_NAME, function_name)
        .replace(FIELD_NAME, &field_name)
        .replace(VALUE, &value)
    )
}

const FUNCTION_TEMPLATE: &str = r##"
fn %%FUNCTION_NAME%%(_input: &Value, message: &mut OutputMessage) -> ProcessingResult<()> {
    message.insert_val(&[ObjectKey::Key("%%FIELD_NAME%%".to_string())], Value::String("%%VALUE%%".to_string()))?;
    Ok(())
}
"##;

const FUNCTION_NAME: &str = "%%FUNCTION_NAME%%";
const FIELD_NAME: &str = "%%FIELD_NAME%%";
const VALUE: &str = "%%VALUE%%";