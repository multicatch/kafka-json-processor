#!/usr/bin/env bash

source "$(dirname "$0")/util/params.sh" || exit 255

function_source="fn %%FUNCTION_NAME%%(input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    if let Some(value) = input.get_val(##JSONPATH(%%SOURCE_FIELD%%)##)?
        .as_str()
        .map(|v| v.to_string()) {

        message.insert_val(##JSONPATH(%%TARGET_FIELD%%)##, Value::String(value))?;
    }
    Ok(())
}
"

required_param_to_var source_field
required_param_to_var target_field

function_source="${function_source//"%%SOURCE_FIELD%%"/$source_field}"
function_source="${function_source//"%%TARGET_FIELD%%"/$target_field}"
function_source="${function_source//"%%FUNCTION_NAME%%"/$kjp_function_name}"

echo "OK"
echo "$function_source"
exit 0