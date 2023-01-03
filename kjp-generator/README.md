# Kafka-json-processor project generator

This utility can be used to generate a Rust project that can be compiled into an executable with all preconfigured JSON processors.
The executable will read from Kafka topic, process the messages and write to another Kafka topic.

The project is generated based on *a template*. 
The template is a configuration file instructing the generator how would you like to process the messages.

## Preparing template.yml

See the following example of "Example processor" project:

```yaml
name: "Example processor"
streams:
- input_topic: in
  output_topic: out

  processors:
    - generator: static_field
      field: $.hello
      value: world
    - generator: copy_field
      source_field: $.abc[1]
      target_field: $.def
```

In this file, we define one stream that will process messages from "in" topic and put processed messages into "out". 

**A stream** is a single pipeline for processing messages originating from one topic and going into another topic.
A single stream contains a list of **processors**. 
The processors are functions that will add or change fields to the output message.

In this example, we define two processors:
* The first one will add a `static_field` to the output message. Desired `field` is defined by JSONPath and the static value is defined by `value`.
* The second one will `copy_field` from an input message to an output message. It will copy from `source_field` (defined by JSONPath) to `target_field`.

Notice that in the template we do not use the term *processor*, *processor kind* or *processor type* to specify what function to use in a pipeline.
The reason is that we actually **generate** the functions for your target project. 
So this file (`template.yml`) actually defines how to generate the project, and thus we use different generators for the desired behavior.

## What are the generators?

The generators are scripts or executables that output function source based on given parameters.
You can write your own script for custom functions. You can also write custom executable (or "plugin" - see [kjp-generator-plugin](../kjp-generator-plugin)).

Kafka-json-processor has a few ready-to-use generators - you can use [kjp-generator-generators](../kjp-generator-generators).

By default, kjp-generator uses "./generators" path for finding available generators. 
If you wish to specify a custom path, use this argument option:

```text
  -g, --generators-path <GENERATORS_PATH>
          Custom path to processor generators.
          
          Put all processor generators in this directory. This directory will be scanned for available files and those files will be used as executable plugins to generate any code requested by `generator` option in your `template.yaml`.

          [default: ./generators]
```

Example: `./kjp-generator -g ./kjp-generator-generators -t ./template.yml -o output-directory`

## How to generate a project?

You will need:
* a template (`template.yml`),
* a `kjp-generator` executable,
* generators (eg. [kjp-generator-generators](../kjp-generator-generators)).

Run `kjp-generator` with the following required options:

```text
  -t, --template <TEMPLATE>
          Path to template file (YAML).
          
          A template file is a configuration file that will be used to generate the final project. 'The final project' - Rust project with generated code to process messages from selected Kafka topics.

  -o, --output <OUTPUT>
          Output directory of generated project.
          
          This will be the directory where the project with message processors will be generated. This project will contain generated code to process messages basing on the supplied template. The code will need to be compiled afterwards.
```

Those options are optional, but keep those in mind:
```text
  -g, --generators-path <GENERATORS_PATH>
          Custom path to processor generators.
          
          Put all processor generators in this directory. This directory will be scanned for available files and those files will be used as executable plugins to generate any code requested by `generator` option in your `template.yaml`.
                    
          [default: ./generators]
          
  -c, --core-path <CORE_PATH>
          Custom path to kafka_json_processor_core.
          
          kafka_json_processor_code is a dependency that contains code that will prevent boilerplate in the generated project. By default, it will use hardcoded version from `crates.io`. If it doesn't work (or you want to use custom core), supply a path to the source code of kafka_json_processor_code.
```

Example:
`./kjp-generator -g ./kjp-generator-generators -t ./template.yml -o output-directory`

See also:
* [template examples](../template-examples)
* [generation test](tests/integration_test.rs)

## Developer's Guide To Plugins

During code generation phase, kjp-generator will try to execute generators with the following arguments:
* generated function name,
* generator options from `template.yml` as argument pairs ("$name" "$value").

For example, the following processor configuration will execute:
```yaml
- input_topic: in
  output_topic: out

  processors:
    - generator: static_field
      field: $.hello
      value: world
```

Will execute the following script:
```sh
./static_field.sh in_out_static_0_field 'generator' 'static_field' 'field' '$.hello' 'value' 'world'
```

Kjp-generator expects the plugin to output a valid UTF-8 string in stdout with the following format:

* first line: 1) `OK` or 2) `ERR`,
* rest of stdout: 1) function source or 2) error message.

Line separators: `\n`.
The plugin termination is treated as the end of function generation - the stdout is collected after this event.
Invalid output is treated as an error.

The plugin does need to interpret JSONPath - it can just wrap it in `##JSONPATH(...)##` to instruct the kjp-generator that it wants it converted to `&[ObjectKey]`.
For example: `##JSONPATH($.hello[1].world)##` will become `&[Key(\"hello\".to_string()), Index(1), Key(\"world\".to_string())]`.

Output examples:

```text
OK
fn in_out_static_0_field(_input: &Value, message: &mut OutputMessage) -> Result<(), ProcessingError> {
    message.insert_val(##JSONPATH($.hello)##, Value::String("world".to_string()))?;
    Ok(())
}

```

```text
ERR
Generator required property that was missing in config: source_field
```