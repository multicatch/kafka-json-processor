use clap::{arg, Parser};
use log::{error, info};
use kjp_generator::read_and_parse_and_generate;

fn main() {
    env_logger::init();

    let args = Args::parse();

    match read_and_parse_and_generate(
        args.template,
        args.output,
        args.core_path,
        args.generators_path,
    ) {
        Ok(_) => info!("Project successfully created!"),
        Err(e) => error!("Generation failed. {e}")
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to template file (YAML).
    ///
    /// A template file is a configuration file that will be used to generate the final project.
    /// 'The final project' - Rust project with generated code to process messages from selected Kafka topics.
    #[arg(short, long)]
    template: String,

    /// Output directory of generated project.
    ///
    /// This will be the directory where the project with message processors will be generated.
    /// This project will contain generated code to process messages basing on the supplied template.
    /// The code will need to be compiled afterwards.
    #[arg(short, long)]
    output: String,

    /// Custom path to kafka_json_processor_core.
    ///
    /// kafka_json_processor_code is a dependency that contains code that will prevent boilerplate in the generated project.
    /// By default, it will use hardcoded version from `crates.io`.
    /// If it doesn't work (or you want to use custom core), supply a path to the source code of kafka_json_processor_code.
    #[arg(short, long)]
    core_path: Option<String>,

    /// Custom path to processor generators.
    ///
    /// Put all processor generators in this directory.
    /// This directory will be scanned for available files and those files will
    /// be used as executable plugins to generate any code requested by `generator` option in your `template.yaml`.
    #[arg(short, long, default_value_t = String::from("./generators"))]
    generators_path: String,
}