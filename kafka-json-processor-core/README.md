# Kafka-json-processor-core

This is a core dependency for [`kafka-json-processor`](../README.md). 
It contains core features of kafka-json-processor projects generated with [kafka-json-processor generator](../kjp-generator).

## Features

* built-in functions for managing reading from and writing to Kafka topics,
* configuration,
* functions for reading/serializing JSON,
* errors, type definitions for reducing boilerplate in generated projects,
* pretty XML and pretty JSON formatters,
* stream simulator.

## How to use?

Most of the time, it will probably be used for generated projects only.
Thanks to this core dependency, [kafka-json-processor generator](../kjp-generator) generated project with one file - `main.rs`, which is still human-readable (and you can tweak some functions before compiling).

But nothing's stopping you from implementing your custom kafka-json-processor by hand! See [examples](./examples).

## Simulations

To test streams in a "dry" environment, you can use *simulations*. 
This is a test utility included in this core that lets you test whether JSON messages are processed correctly (before running your compiled kafka-json-processor).

For simulations, prepare following directory structure:

```text
<project_directory>
 > simulations
 | > stream_name
```

For generated projects, `stream_name` will be `${input_topic}_${output_topic}` (eg. `in_out`), but you can have stream of any name in your custom kafka-json-processor.
Prepare a `HashMap<String, Stream>` of streams and run simulation using `kafka_json_processor_core::simulation::simulate_streams_from_default_folder`.

At the beginning of simulation, the simulator will look for all files in the directory and will try to:
* deserialize `[Input]` JSON,
* run all processors in stream with given input message,
* assert that output message equals `[Expected]` message (by comparing JSON-s, not raw serialized strings).

Examples:
* [message definitions for simulations](../simulations)
* [simple simulation implementation in tests](examples/simple.rs)

## Configuring kafka-json-processor

By default, kafka-json-processor will look for [`./processor.properties`](../processor.properties). 
You can change the default location by setting the `KAFKA_PROCESSOR_CONFIG_PATH` environment variable.

This file contains configuration for Kafka client (rdkafka) and kafka-json-processor specific options.

For rdkafka configuration [see documentation](https://docs.confluent.io/5.5.0/clients/librdkafka/md_CONFIGURATION.html).
Prefixing rdkafka properties with `consumer.` or `producer.` will apply the property to consumer or producer only.
Non-prefixed properties will be applied to both clients.

Kafka-json-processor specific options:

```properties
# Worker threads - how many threads to use for processing.
# Default: 4
processor.worker.threads=4

# Received messages are passed by a channel to worker threads. If the processors are too slow, the channel fill up.
# Default: 50
processor.channel.capacity=50

# The producer queue size. Processed messages are queued to be sent to Kafka. Producing will slow down if the queue fills up.
# You should set this option to the same value as producer.queue.buffering.max.messages.
# Default: 100000
processor.queue.size=100000

# Slow down time. When the producer queue is filled up above 95%, then the message production will be paused for the following time.
# This does not mean that processing will be paused too!
# Default: 10000 (10s)
processor.queue.slowdown.ms=10000
```