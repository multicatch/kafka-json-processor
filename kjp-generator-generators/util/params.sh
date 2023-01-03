#!/usr/bin/env bash

_kjp_param_count="$#"
if [[ "$_kjp_param_count" -eq 0 ]]; then
  echo "ERR"
  echo "No processor generator arguments specified. Specify at least 1 argument (function_name)."
  exit 1
fi

_kjp_param_count="$#"
if [[ $((_kjp_param_count%2)) -eq 0 ]]; then
  echo "ERR"
  echo "Processor arguments are invalid. Cannot construct the config map (wrong number of arguments)."
  exit 1
fi

export kjp_function_name="$1"
shift

while [[ "$#" -gt 0 ]]; do
  declare "kjp_params_$1=$2"
  shift
  shift
done

required_param_to_var() {
    local name=$1
    local var_name
    if [[ "$#" -eq 1 ]]; then
      var_name="$name"
    else
      var_name="$2"
    fi

    local i="kjp_params_$name"
    local param_val
    param_val=$(printf '%s' "${!i}")

    if [[ -z "$param_val" ]]; then
      echo "ERR"
      printf 'Generator required property that was missing in config: %s\n' "$name"
      exit 1
    fi

    export "$var_name=$param_val"
}

optional_param_to_var() {
    local name=$1
    local var_name
    if [[ "$#" -eq 1 ]]; then
      var_name="$name"
    else
      var_name="$2"
    fi

    local i="kjp_params_$name"
    local param_val
    param_val=$(printf '%s' "${!i}")

    if [[ -n "$param_val" ]]; then
      export "$var_name=$param_val"
    fi
}