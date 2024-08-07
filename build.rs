fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .file_descriptor_set_path("src/fds/cyclingtracker.bin")
        .compile(
            &[format!(
                "{}/proto/cyclingtracker.proto",
                std::env::var("CARGO_MANIFEST_DIR").unwrap()
            )],
            &[format!(
                "{}/proto",
                std::env::var("CARGO_MANIFEST_DIR").unwrap()
            )],
        )
        .unwrap();

    Ok(())
}
