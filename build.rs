fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .file_descriptor_set_path("src/cyclingtracker.bin")
        .compile(&["proto/cyclingtracker.proto"], &["proto"])
        .unwrap();

    Ok(())
}
