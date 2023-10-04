mod codegen;
pub mod generated;
pub mod parse;

use std::fs;

pub use codegen::generate_code;

pub fn generate(in_dir: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", in_dir);

    let mut services = Vec::new();

    for file in fs::read_dir(in_dir)? {
        let file = file?;
        let path = file.path();
        if let Some(ext) = path.extension() {
            if ext == "probe" {
                let file = fs::read_to_string(&file.path())?;
                let service = parse::Service::from_file(&file)?;
                services.push(service);
            }
        }
    }

    let mut file = fs::File::create(out_file)?;
    generate_code(&mut file, &services)?;

    Ok(())
}
