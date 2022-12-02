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
    /// Path to template file (YAML)
    #[arg(short, long)]
    template: String,

    /// Output directory of generated project
    #[arg(short, long)]
    output: String,

    /// Custom path to kafka_json_processor_core
    #[arg(short, long)]
    core_path: Option<String>,

    /// Custom path to processor generators
    #[arg(short, long, default_value_t = String::from("./generators"))]
    generators_path: String,
}