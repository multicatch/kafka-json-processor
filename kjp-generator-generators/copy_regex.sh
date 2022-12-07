#!/usr/bin/env bash

source "$(dirname "$0")/util/params.sh" || exit 255

function_source="fn %%FUNCTION_NAME%%(input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    lazy_static! {
       static ref REGEX: regex::Regex = regex::Regex::new(r#\"%%PATTERN%%\"#).unwrap();
    }

    if let Some(source) = input.get_val(##JSONPATH(%%SOURCE_FIELD%%)##)?
        .as_str()
        .map(|v| v.to_string()) {

        let capture = REGEX.captures_iter(&source)
            .next()
            .and_then(|c| c.get(%%CAPTURE_GROUP%%))
            .map(|v| v.as_str().to_string())
            .ok_or_else(|| ErrorKind::ProcessorSkipped {
                reason: format!(\"Failed to extract anything from field using regex /{}/ (capture group {}).\", r#\"%%PATTERN%%\"#, %%CAPTURE_GROUP%%)
            })?;

        message.insert_val(##JSONPATH(%%TARGET_FIELD%%)##, Value::String(capture))?;
    }
    Ok(())
}
"

required_param_to_var source_field
required_param_to_var target_field
required_param_to_var pattern

capture_group=0
optional_param_to_var group capture_group

function_source="${function_source//"%%SOURCE_FIELD%%"/$source_field}"
function_source="${function_source//"%%TARGET_FIELD%%"/$target_field}"
function_source="${function_source//"%%PATTERN%%"/$pattern}"
function_source="${function_source//"%%CAPTURE_GROUP%%"/$capture_group}"
function_source="${function_source//"%%FUNCTION_NAME%%"/$kjp_function_name}"

echo "OK"
echo "$function_source"
exit 0