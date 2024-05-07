mod codegen;
mod parse;
mod schema;

use std::error::Error;
use std::fs;

pub use codegen::generate_code;
pub use schema::Prevalence;
pub use schema::Probe;
pub use schema::ProbeFile;

pub use crate::parse::parse_file;
pub use crate::parse::CheckProbeError;
pub use crate::parse::ParseError;
pub use crate::parse::ParseErrorKind;
pub use crate::parse::ProbeFileDirectory;

pub fn generate(
    in_dirs: &[(&str, ProbeFileDirectory)],
    out_file: &str,
) -> Result<(), Box<dyn Error>> {
    let mut services = Vec::new();

    for (dir, kind) in in_dirs.iter().copied() {
        println!("cargo:rerun-if-changed={}", dir);
        let mut files = fs::read_dir(dir)?
            .map(|result| result.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        files.sort();
        for path in files {
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
