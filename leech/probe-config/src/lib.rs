mod codegen;
pub mod generated;
pub mod parse;

use std::fs;

pub use codegen::generate_code;

pub fn generate(in_dirs: &[&str], out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut services = Vec::new();

    for dir in in_dirs {
        println!("cargo:rerun-if-changed={}", dir);
        for file in fs::read_dir(dir)? {
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
    }

    let mut file = fs::File::create(out_file)?;
    generate_code(&mut file, &services)?;

    Ok(())
}
