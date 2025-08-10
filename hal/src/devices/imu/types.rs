use std::io;

#[derive(Debug, Clone)]
pub struct ImuData {
    pub accel: [f32; 3], // normalized to g
    pub gyro: [f32; 3],  // radians/s
    pub mag: [f32; 3],   // normalized to gauss
    pub temp: f32,       // Celsius
    pub quaternion: [f32; 4],
    pub euler_angles: [f32; 3], // radians
}

pub trait Imu {
    fn name(&self) -> String;
    fn imu_data(&self) -> ImuData;
    fn init(&self) -> io::Result<()>;
    fn deinit(&self) -> io::Result<()>;
    fn sample_rate(&self) -> u32;
}
