mod codegen;
pub mod generated;
mod parse;
mod schema;

use std::error::Error;
use std::fs;

pub use codegen::generate_code;
pub use schema::{Prevalence, Probe, ProbeFile, Protocol};

pub use crate::parse::{
    parse_file, CheckProbeError, ParseError, ParseErrorKind, ProbeFileDirectory,
};

pub fn generate(
    in_dirs: &[(&str, ProbeFileDirectory)],
    out_file: &str,
) -> Result<(), Box<dyn Error>> {
    let mut services = Vec::new();

    for (dir, kind) in in_dirs.iter().copied() {
        println!("cargo:rerun-if-changed={}", dir);
        for file in fs::read_dir(dir)? {
            let file = file?;
            let path = file.path();
            if let Some(ext) = path.extension() {
                if ext == "probe" {
                    services.push(parse_file(path, kind)?);
                }
            }
        }
    }

    let mut file = fs::File::create(out_file)?;
    generate_code(&mut file, &services)?;

    Ok(())
}
