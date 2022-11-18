use std::fs::remove_dir_all;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use kafka_json_processor_generator::read_and_parse_and_generate;

#[test]
fn generate_and_build() {
    env_logger::builder().is_test(true).init();
    let root = root_project_dir();

    let input_template = root.join("kafka-json-processor-generator/test-examples/correct.yaml");
    let output_dir = root.join("test-output");

    let result = read_and_parse_and_generate(
        input_template,
        output_dir.clone(),
        Some("../kafka-json-processor-core".to_string())
    );

    assert!(result.is_ok(), "Generation failed. {}", result.err().unwrap());

    let exit_status = Command::new("cargo")
        .args(["build"])
        .current_dir(output_dir.clone())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to run cargo build");

    assert!(exit_status.success(), "Cargo build failed with status {exit_status}");

    let exit_status = Command::new("cargo")
        .args(["test"])
        .current_dir(output_dir.clone())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to run cargo test");

    assert!(exit_status.success(), "Cargo test failed with status {exit_status}");

    println!("Removing test-output...");
    remove_dir_all(output_dir).unwrap();
}

fn root_project_dir() -> PathBuf {
    let working_dir = PathBuf::from(".");
    let mut working_dir = working_dir.canonicalize().unwrap();
    let in_generator_dir = working_dir.ends_with("kafka-json-processor-generator");

    if in_generator_dir {
        working_dir.pop();
    }

    working_dir
}