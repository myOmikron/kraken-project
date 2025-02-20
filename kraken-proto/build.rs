fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        // Required for protoc versions < 3.15
        // 3.15 stabilized the feature and enabled it by default
        // This was relevant for the CI
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["./proto/attacks.proto"], &["./proto/"])?;
    Ok(())
}
