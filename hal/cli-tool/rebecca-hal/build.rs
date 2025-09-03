use std::io::Result;
fn main() -> Result<()> {
    // prost_build::compile_protos(&["src/imu_data.proto"], &["src/"])?;
    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .compile_protos(&["src/imu_data.proto"], &["src/"])
        .unwrap();
    Ok(())
}
