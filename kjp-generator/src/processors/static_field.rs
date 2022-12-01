use std::collections::HashMap;
use crate::processors::{FIELD_KEY, json_path_to_object_key, JsonFieldName, ProcessorGenerationError};
use crate::processors::ProcessorGenerationError::RequiredConfigNotFound;

/// Generates a processor that adds a static field to JSON
///
/// This generates a function that adds a static field to a JSON from topic.
/// Available config options:
///  - "field" - static field name
///  - "value" - a string to put in this field
pub fn static_field(function_name: &str, config: &HashMap<String, String>) -> Result<String, ProcessorGenerationError> {
    let target_field = json_path_to_object_key(
        config.get(FIELD_KEY)
            .ok_or_else(|| RequiredConfigNotFound {
                function_name: function_name.to_string(),
                field_name: FIELD_KEY.to_string(),
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
        .replace(FUNCTION_NAME, function_name)
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
    use crate::processors::static_field::static_field;

    #[test]
    fn should_generate_static_field() {
        let mut config = HashMap::new();
        config.insert("field".to_string(), "$.example[0]".to_string());
        config.insert("value".to_string(), "abcdef".to_string());
        let result = static_field("abc1", &config);
        assert_eq!(Ok(r##"
fn abc1(_input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    message.insert_val(&[Key("example".to_string()), Index(0)], Value::String("abcdef".to_string()))?;
    Ok(())
}
"##.to_string()), result);
    }
}