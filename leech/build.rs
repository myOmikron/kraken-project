use std::env;

use probe_config::ProbeFileDirectory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    probe_config::generate(
        &[
            (
                "src/modules/service_detection/probe_files/tcp",
                ProbeFileDirectory::Tcp,
            ),
            (
                "src/modules/service_detection/probe_files/udp",
                ProbeFileDirectory::Udp,
            ),
        ],
        &format!("{}/generated_probes.rs", env::var("OUT_DIR")?),
    )
    .map_err(|err| err.to_string())?; // use Display implementation in error messages
    Ok(())
}
