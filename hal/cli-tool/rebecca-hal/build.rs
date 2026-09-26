use std::io::Result;

fn main() -> Result<()> {
    let schema = "../../service/src/imu_data.proto";
    println!("cargo:rerun-if-changed={schema}");
    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .compile_protos(&[schema], &["../../service/src"])?;
    Ok(())
}
