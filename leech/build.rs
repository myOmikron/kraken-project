use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    probe_config::generate(
        "src/modules/service_detection/probe_files",
        &format!("{}/generated_probes.rs", env::var("OUT_DIR")?),
    )?;
    Ok(())
}
