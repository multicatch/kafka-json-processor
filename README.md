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

## Test your processor

You may want to test if the generated processor is correct before deploying it.
To test it, in the generated project, create a `simulations` directory. 
In this directory, create another directory (or directories, depending on the `template.yaml`) with the name `${input_topic}_${output_topic}`,
where `${input_topic}` is the name of the input_topic from `template.yaml` and `${output_topic}` is the name of the output_topic from `template.yaml`.

For example, if you have the following stream in your `tempate.yaml`:
```yaml
streams:
  - input_topic: sometopic
    output_topic: target
```

Then create the following directory structure:
```text
<project_directory>
 > simulations
 | > sometopic_target
```

In the `${input_topic}_${output_topic}` directory (in this case - `sometopic_target`), create text files with the input message and expected output.
For example, given the template [`all_processors.yaml`](template-examples/all_processors.yaml) (see template-examples), I have prepared some test data in [`simulations/in_out`](simulations/in_out).
The test files always have two headers - `[Input]` (for input JSON) and `[Expected]` (for expected processed message).

To run the simulation, run `cargo test` in the generated project. 
See [`kjp-generator/tests/integration_test.rs`](kjp-generator/tests/integration_test.rs) for a complete example.