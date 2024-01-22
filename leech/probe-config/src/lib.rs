mod codegen;
pub mod generated;
pub mod parse;

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs;

pub use codegen::generate_code;

use crate::parse::ParseError;

#[derive(Debug)]
struct FileParseError {
    file: String,
    error: ParseError,
}

impl Display for FileParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error in {}: {}", self.file, self.error)
    }
}

impl Error for FileParseError {}

pub fn generate(in_dirs: &[&str], out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut services = Vec::new();

    for dir in in_dirs {
        println!("cargo:rerun-if-changed={}", dir);
        for file in fs::read_dir(dir)? {
            let file = file?;
            let path = file.path();
            if let Some(ext) = path.extension() {
                if ext == "probe" {
                    let path = path
                        .to_str()
                        .ok_or("build script path should always be convertible to string")?;
                    let content = fs::read_to_string(&file.path())?;
                    let service = parse::Service::from_file(path, &content).map_err(
                        |parse_error: ParseError| FileParseError {
                            file: String::from(path),
                            error: parse_error,
                        },
                    )?;
                    services.push(service);
                }
            }
        }
    }

    let mut file = fs::File::create(out_file)?;
    generate_code(&mut file, &services)?;

    Ok(())
}
