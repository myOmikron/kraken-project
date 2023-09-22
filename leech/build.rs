fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["../proto/attacks.proto"], &["../proto/"])?;
    probe_config::generate(
        "src/modules/service_detection/probe_files",
        "src/modules/service_detection/generated.rs",
    )?;
    Ok(())
}
