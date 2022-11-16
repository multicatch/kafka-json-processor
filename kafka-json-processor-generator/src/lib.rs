use std::error::Error;
use std::path::Path;

pub fn parse_and_generate<P1: AsRef<Path>, P2: AsRef<Path>>(_template_path: P1, _output_path: P2)
    -> Result<(), Box<dyn Error>> {

    Ok(())
}