use std::io;

#[derive(Debug, Clone)]
pub struct ImuData {
    pub accel: [f32; 3],
    pub gyro: [f32; 3],
    pub mag: [f32; 3],
    pub temp: f32,
}

pub trait Imu {
    fn name(&self) -> String;
    fn imu_data(&self) -> ImuData;
    fn init(&self) -> io::Result<()>;
    fn deinit(&self) -> io::Result<()>;
    fn sample_rate(&self) -> u32;
}
