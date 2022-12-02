use std::fs;
use std::fs::{create_dir_all, remove_dir_all};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use log::LevelFilter;
use kjp_generator::read_and_parse_and_generate;

#[test]
fn generate_and_build() {
    env_logger::builder().is_test(true)
        .filter_level(LevelFilter::Debug)
        .init();
    let root = root_project_dir();

    let input_template = root.join("kjp-generator/test-examples/correct.yaml");
    let output_dir = root.join("test-output");
    let generator_dir = root.join("test-generators");

    copy_generators(root.join("target/debug"), &generator_dir);

    let result = read_and_parse_and_generate(
        input_template,
        output_dir.clone(),
        Some("../kafka-json-processor-core".to_string()),
        &generator_dir,
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
    remove_dir_all(generator_dir).unwrap();
}

fn root_project_dir() -> PathBuf {
    let working_dir = PathBuf::from(".");
    let mut working_dir = working_dir.canonicalize().unwrap();
    let in_generator_dir = working_dir.ends_with("kjp-generator");

    if in_generator_dir {
        working_dir.pop();
    }

    working_dir
}

fn copy_generators<P1: AsRef<Path>, P2: AsRef<Path>>(source: P1, target: P2) {
    if target.as_ref().exists() {
        remove_dir_all(&target).unwrap();
    }

    create_dir_all(&target).unwrap();

    let source = source.as_ref();
    let target = target.as_ref();
    fs::copy(source.join("copy_field"), target.join("copy_field")).unwrap();
    fs::copy(source.join("static_field"), target.join("static_field")).unwrap();
}