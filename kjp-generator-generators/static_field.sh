#!/usr/bin/env bash

source "$(dirname "$0")/util/params.sh" || exit 255

function_source="fn %%FUNCTION_NAME%%(_input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    message.insert_val(##JSONPATH(%%TARGET_FIELD%%)##, Value::String(\"%%VALUE%%\".to_string()))?;
    Ok(())
}
"

required_param_to_var field
required_param_to_var value


function_source="${function_source//"%%TARGET_FIELD%%"/$field}"
function_source="${function_source//"%%VALUE%%"/$value}"
function_source="${function_source//"%%FUNCTION_NAME%%"/$kjp_function_name}"

echo "OK"
echo "$function_source"
exit 0