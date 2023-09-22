mod codegen;
mod parse;

use std::{fs, io};

pub use codegen::generate_code;
pub use parse::*;

pub fn generate(in_dir: &str, out_file: &str) -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", in_dir);

    let mut services = Vec::new();

    for file in fs::read_dir(&in_dir)? {
        let file = fs::read_to_string(&file.unwrap().path())?;
        let service = Service::from_file(&file).expect("Failed to parse probes");
        services.push(service);
    }

    let mut file = fs::File::create(out_file)?;
    generate_code(&mut file, &services)
}
