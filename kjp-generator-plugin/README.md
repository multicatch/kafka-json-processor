# Kafka-json-processor generator plugin framework

This is a project that lets you create a plugin for [kjp-generator](../kjp-generator-generators) in Rust.
This library contains logic for parsing input parameters, handling errors and some handy utilities.

## Handy utilities

* `GeneratorError` for signaling errors (DO use this one for error for correct error handling),
* `json_path_to_object_key(&str) -> String` for generating `&[ObjectKey]` for use with [kafka-json-processor-core](../kafka-json-processor-core) from JSONPath (parses JSONPath).

## How to create custom plugin

See [examples](examples) for practical guide how to use `kjp-generator-plugin`. 
For technical details how plugins work, see [kjp-generator documentation](../kjp-generator/README.md).


