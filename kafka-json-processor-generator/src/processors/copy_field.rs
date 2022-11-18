use std::collections::HashMap;
use crate::processors::ProcessorGenerationError;
use crate::processors::ProcessorGenerationError::RequiredConfigNotFound;

/// Generates a processor that copies a field to another field
///
/// Available config options:
///  - "source_field" - source field name (field that should be copied)
///  - "target_field" - name of field to copy value to
pub fn copy_field(function_name: &str, config: &HashMap<String, String>) -> Result<String, ProcessorGenerationError> {
    let source_field = config.get("source_field")
        .ok_or_else(|| RequiredConfigNotFound {
            function_name: function_name.to_string(),
            field_name: "source_field".to_string(),
            description: None,
        })?
        .replace('\"', "\\\"");

    let target_field = config.get("target_field")
        .ok_or_else(|| RequiredConfigNotFound {
            function_name: function_name.to_string(),
            field_name: "target_field".to_string(),
            description: None
        })?
        .replace('\"', "\\\"");

    Ok(FUNCTION_TEMPLATE
        .replace(FUNCTION_NAME, function_name)
        .replace(SOURCE_FIELD, &source_field)
        .replace(TARGET_FIELD, &target_field)
    )
}

const FUNCTION_TEMPLATE: &str = r##"
fn %%FUNCTION_NAME%%(input: &Value, message: &mut OutputMessage) -> ProcessingResult<()> {
    if let Some(value) = input.get("%%SOURCE_FIELD%%")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string()) {

        message.insert_str("%%TARGET_FIELD%%".to_string(), value);
    } else {
        debug!("Field [%%SOURCE_FIELD%%] is not present - skipping field copy");
    }
    Ok(())
}
"##;

const FUNCTION_NAME: &str = "%%FUNCTION_NAME%%";
const SOURCE_FIELD: &str = "%%SOURCE_FIELD%%";
const TARGET_FIELD: &str = "%%TARGET_FIELD%%";