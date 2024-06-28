fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .type_attribute(".uselesspackage.Point", "#[derive(Eq)]")
        .type_attribute(".uselesspackage.Point", "#[derive(Hash)]")
        .file_descriptor_set_path("src/useless.bin")
        .compile(&["proto/useless.proto"], &["proto"])
        .unwrap();
    Ok(())
}
