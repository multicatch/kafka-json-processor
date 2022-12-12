# Kafka JSON Processor

A processor that reads JSONs from Kafka topics, processes them and puts them in other selected Kafka topic.

```text
Kafka topic #1 -> JSON message -> kafka-json-processor -> new JSON message -> Kafka topic #2
```

Processors used by kafka-json-processor are configured by a template.yaml file. 
You can customize the process of processing messages or extend processors with custom ones.

In fact this utility generates Rust code based on a template in YAML. 
Compiled code in customized cases is generally faster, 
as it does not need to interpret the config and runs instructions directly.

This project is split into the following subprojects:
* [The generator](kjp-generator) - generates a project based on `template.yaml` and available processor generators.
* [The processor generators](kjp-generator-generators) - a set of scripts with code generators with some predefined functions for your custom processor.
* [The plugin framework](kjp-generator-plugin) - base for your custom code generator (if you want to write it in Rust, not as a script).
* [The core dependency](kafka-json-processor-core) - used in generated projects, prevents boilerplate.

## How to use?

In short, the steps to run your custom processor are the following:

1. Prepare your `template.yaml` with your desired processor configuration (e.g. copy field, extract date from message etc. - [see example](template-examples/basic.yaml)).
2. Generate JSON processor with [the generator](kjp-generator) (and [processor generators](kjp-generator-generators), you can also use your own) and compile the generated project.
3. Prepare `processor.properties` with rdkafka (Kafka client) configuration - [see example](./processor.properties) (put this file in the same directory as your executable).
4. Run your executable (to see logs set the following environment variable: `RUST_LOG=info`, e.g. in bash you can just run `RUST_LOG=info ./your_executable`).

