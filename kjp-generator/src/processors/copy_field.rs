use kjp_generator_plugin::{GeneratorError, json_path_to_object_key, JsonFieldName, ProcessorParams, return_generated};
use kjp_generator_plugin::GeneratorError::RequiredConfigNotFound;

#[allow(dead_code)]
fn main() {
    return_generated(copy_field);
}

/// Generates a processor that copies a field to another field
///
/// Available config options:
///  - "source_field" - source field name (field that should be copied)
///  - "target_field" - name of field to copy value to
pub fn copy_field(params: ProcessorParams) -> Result<String, GeneratorError> {
    let function_name = params.function_name;
    let config = params.config;

    let raw_source_field = config.get("source_field")
        .ok_or_else(|| RequiredConfigNotFound {
            function_name: function_name.to_string(),
            field_name: "source_field".to_string(),
            description: None,
        })?;

    let source_field = json_path_to_object_key(raw_source_field);

    let target_field = json_path_to_object_key(config.get("target_field")
            .ok_or_else(|| RequiredConfigNotFound {
                function_name: function_name.to_string(),
                field_name: "target_field".to_string(),
                description: None,
            })?
    );

    Ok(FUNCTION_TEMPLATE
        .replace(FUNCTION_NAME, &function_name)
        .replace(SOURCE_FIELD, &source_field)
        .replace(RAW_SOURCE_FIELD, &raw_source_field.escape_for_json())
        .replace(TARGET_FIELD, &target_field)
    )
}

const FUNCTION_TEMPLATE: &str = r##"
fn %%FUNCTION_NAME%%(input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    if let Some(value) = input.get_val(%%SOURCE_FIELD%%)?
        .as_str()
        .map(|v| v.to_string()) {

        message.insert_val(%%TARGET_FIELD%%, Value::String(value))?;
    }
    Ok(())
}
"##;

const FUNCTION_NAME: &str = "%%FUNCTION_NAME%%";
const SOURCE_FIELD: &str = "%%SOURCE_FIELD%%";
const RAW_SOURCE_FIELD: &str = "%%RAW_SOURCE_FIELD%%";
const TARGET_FIELD: &str = "%%TARGET_FIELD%%";

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use kjp_generator_plugin::ProcessorParams;
    use crate::processors::copy_field::copy_field;

    #[test]
    fn should_generate_copy_field() {
        let mut config = HashMap::new();
        config.insert("source_field".to_string(), "$[te\"st]".to_string());
        config.insert("target_field".to_string(), "$[0].xd".to_string());
        let result = copy_field(ProcessorParams {
            function_name: "function1".to_string(),
            config
        });

        assert_eq!(Ok(r##"
fn function1(input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    if let Some(value) = input.get_val(&[Key("te\"st".to_string())])?
        .as_str()
        .map(|v| v.to_string()) {

        message.insert_val(&[Index(0), Key("xd".to_string())], Value::String(value))?;
    }
    Ok(())
}
"##.to_string()), result);
    }
}