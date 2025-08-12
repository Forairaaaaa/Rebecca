use crate::devices::imu::{Imu, ImuData};
use log::debug;
use std::io;
use std::time::Instant;

pub struct MockImu {
    name: String,
    sample_rate: u32,
    start: Instant,
}

impl MockImu {
    pub fn new() -> Self {
        Self {
            name: "mock".to_string(),
            sample_rate: 50,
            start: Instant::now(),
        }
    }
}

impl Imu for MockImu {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn imu_data(&self) -> ImuData {
        let speed_scale = 0.5;

        let t = self.start.elapsed().as_secs_f32() * speed_scale;

        // 模拟多轴慢速旋转
        let yaw = 0.2 * t; // Z 轴旋转
        let pitch = 0.1 * t.sin() * 0.5; // X 轴缓慢摆动
        let roll = 0.1 * t.cos() * 0.5; // Y 轴缓慢摆动

        // 欧拉角 -> 四元数
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();

        let qw = cr * cp * cy + sr * sp * sy;
        let qx = sr * cp * cy - cr * sp * sy;
        let qy = cr * sp * cy + sr * cp * sy;
        let qz = cr * cp * sy - sr * sp * cy;

        // 模拟加速度（重力 + 少量噪声）
        let accel = [
            0.0 + 0.02 * (t * 3.0).sin(),
            0.0 + 0.02 * (t * 2.0).cos(),
            -1.0 + 0.02 * (t * 1.5).sin(),
        ];

        // 模拟陀螺仪（绕 XYZ 缓慢旋转）
        let gyro = [0.05 * (t * 0.5).cos(), 0.05 * (t * 0.3).sin(), 0.05];

        // 模拟磁力计（固定方向 + 少量噪声）
        let mag = [
            0.3 + 0.01 * (t * 2.0).cos(),
            0.0 + 0.01 * (t * 1.3).sin(),
            0.5 + 0.01 * (t * 1.7).cos(),
        ];

        // 温度（单位：毫摄氏度）
        let temp = 25_000.0 + 500.0 * (t * 0.1).sin();

        ImuData {
            accel,
            gyro,
            mag,
            temp,
            quaternion: [qw, qx, qy, qz],
            euler_angles: [yaw, pitch, roll],
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
