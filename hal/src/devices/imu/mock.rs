use crate::devices::imu::{Imu, ImuData};
use log::debug;
use std::io;

pub struct MockImu {
    name: String,
    sample_rate: u32,
}

impl MockImu {
    pub fn new() -> Self {
        Self {
            name: "mock".to_string(),
            sample_rate: 50,
        }
    }
}

impl Imu for MockImu {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn imu_data(&self) -> ImuData {
        ImuData {
            accel: [0.0, 0.0, 0.0],
            gyro: [0.0, 0.0, 0.0],
            mag: [0.0, 0.0, 0.0],
            temp: 0.0,
            quaternion: [0.0, 0.0, 0.0, 0.0],
            euler_angles: [0.0, 0.0, 0.0],
        }
    }

    fn init(&self) -> io::Result<()> {
        debug!("init iio imu device: {}", self.name);
        Ok(())
    }

    fn deinit(&self) -> io::Result<()> {
        debug!("deinit iio imu device: {}", self.name);
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
