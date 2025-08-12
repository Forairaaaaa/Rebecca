use crate::devices::imu::{Imu, ImuData};
use log::{debug, warn};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct IioImu {
    name: String,
    device_path: PathBuf,
    // IIO channel file paths for reading sensor data
    accel_x_path: Option<PathBuf>,
    accel_y_path: Option<PathBuf>,
    accel_z_path: Option<PathBuf>,
    gyro_x_path: Option<PathBuf>,
    gyro_y_path: Option<PathBuf>,
    gyro_z_path: Option<PathBuf>,
    mag_x_path: Option<PathBuf>,
    mag_y_path: Option<PathBuf>,
    mag_z_path: Option<PathBuf>,
    temp_path: Option<PathBuf>,
    // Scale factors for converting raw values
    accel_scale: f32,
    gyro_scale: f32,
    mag_scale: f32,
    temp_scale: f32,
    // Optional offset for temperature when using raw channel
    temp_offset: f32,
    // Whether temperature path points to an already-processed input value
    temp_is_input: bool,
    // Sample rate for the IMU device
    sample_rate: u32,
}

impl IioImu {
    pub fn new(name: &str) -> Option<Self> {
        debug!("searching for iio device: {}", name);

        // Scan /sys/bus/iio/devices for matching device
        let iio_devices_path = Path::new("/sys/bus/iio/devices");
        if !iio_devices_path.exists() {
            warn!(
                "iio devices path does not exist: {}",
                iio_devices_path.display()
            );
            return None;
        }

        let entries = match fs::read_dir(iio_devices_path) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("failed to read IIO devices directory: {}", e);
                return None;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let device_path = entry.path();

            // Check if this is an IIO device directory (iio:deviceN)
            let dir_name = device_path.file_name()?.to_str()?;
            if !dir_name.starts_with("iio:device") {
                continue;
            }

            // Read device name
            let name_path = device_path.join("name");
            let device_name = match fs::read_to_string(&name_path) {
                Ok(content) => content.trim().to_string(),
                Err(_) => continue,
            };

            // Check if this matches our target device name
            if device_name != name {
                continue;
            }

            debug!("found matching iio device at: {}", device_path.display());

            // Initialize the ImuFromIio instance
            let mut imu = IioImu {
                name: name.to_string(),
                device_path: device_path.clone(),
                accel_x_path: None,
                accel_y_path: None,
                accel_z_path: None,
                gyro_x_path: None,
                gyro_y_path: None,
                gyro_z_path: None,
                mag_x_path: None,
                mag_y_path: None,
                mag_z_path: None,
                temp_path: None,
                accel_scale: 1.0,
                gyro_scale: 1.0,
                mag_scale: 1.0,
                temp_scale: 1.0,
                temp_offset: 0.0,
                temp_is_input: false,
                sample_rate: 30,
            };

            // Scan for available channels and scales
            if let Err(e) = imu.scan_channels() {
                warn!("failed to scan iio channels for {}: {}", name, e);
                return None;
            }

            return Some(imu);
        }

        warn!("iio device '{}' not found", name);
        None
    }

    fn scan_channels(&mut self) -> io::Result<()> {
        let entries = fs::read_dir(&self.device_path)?;

        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_str().unwrap_or("");

            // Look for raw data files and their corresponding scales
            match file_name_str {
                // Accelerometer channels
                "in_accel_x_raw" => {
                    self.accel_x_path = Some(entry.path());
                    self.accel_scale = self.read_scale("in_accel_scale").unwrap_or(1.0);
                }
                "in_accel_y_raw" => {
                    self.accel_y_path = Some(entry.path());
                }
                "in_accel_z_raw" => {
                    self.accel_z_path = Some(entry.path());
                }

                // Gyroscope channels
                "in_anglvel_x_raw" => {
                    self.gyro_x_path = Some(entry.path());
                    self.gyro_scale = self.read_scale("in_anglvel_scale").unwrap_or(1.0);
                }
                "in_anglvel_y_raw" => {
                    self.gyro_y_path = Some(entry.path());
                }
                "in_anglvel_z_raw" => {
                    self.gyro_z_path = Some(entry.path());
                }

                // Magnetometer channels
                "in_magn_x_raw" => {
                    self.mag_x_path = Some(entry.path());
                    self.mag_scale = self.read_scale("in_magn_scale").unwrap_or(1.0);
                }
                "in_magn_y_raw" => {
                    self.mag_y_path = Some(entry.path());
                }
                "in_magn_z_raw" => {
                    self.mag_z_path = Some(entry.path());
                }

                // Temperature channel
                "in_temp_raw" => {
                    self.temp_path = Some(entry.path());
                    self.temp_scale = self.read_scale("in_temp_scale").unwrap_or(1.0);
                    self.temp_offset = self.read_offset("in_temp_offset").unwrap_or(0.0);
                    self.temp_is_input = false;
                }
                "in_temp_input" => {
                    self.temp_path = Some(entry.path());
                    self.temp_is_input = true;
                }
                "in_temp_scale" => {
                    self.temp_scale = self.read_scale("in_temp_scale").unwrap_or(1.0);
                }
                "in_temp_offset" => {
                    self.temp_offset = self.read_offset("in_temp_offset").unwrap_or(0.0);
                }

                // Sample rate files
                "sampling_frequency" => {
                    if let Some(rate) = self.read_sample_rate(&entry.path()) {
                        self.sample_rate = rate;
                    }
                }
                "in_accel_sampling_frequency" => {
                    if let Some(rate) = self.read_sample_rate(&entry.path()) {
                        self.sample_rate = rate;
                    }
                }

                _ => {}
            }
        }

        debug!("scanned iio channels for device: {}", self.name);
        Ok(())
    }

    fn read_scale(&self, scale_file: &str) -> Option<f32> {
        let scale_path = self.device_path.join(scale_file);
        fs::read_to_string(scale_path).ok()?.trim().parse().ok()
    }

    fn read_offset(&self, offset_file: &str) -> Option<f32> {
        let offset_path = self.device_path.join(offset_file);
        fs::read_to_string(offset_path).ok()?.trim().parse().ok()
    }

    fn read_sample_rate(&self, path: &PathBuf) -> Option<u32> {
        fs::read_to_string(path).ok()?.trim().parse().ok()
    }

    fn read_raw_value(&self, path: &Option<PathBuf>) -> f32 {
        match path {
            Some(p) => fs::read_to_string(p)
                .ok()
                .and_then(|s| s.trim().parse::<i32>().ok())
                .unwrap_or(0) as f32,
            None => 0.0,
        }
    }

    fn read_value_as_f32(&self, path: &Option<PathBuf>) -> f32 {
        match path {
            Some(p) => match fs::read_to_string(p) {
                Ok(content) => {
                    let trimmed = content.trim();
                    if let Ok(v) = trimmed.parse::<f32>() {
                        v
                    } else if let Ok(v) = trimmed.parse::<i64>() {
                        v as f32
                    } else if let Ok(v) = trimmed.parse::<i32>() {
                        v as f32
                    } else {
                        0.0
                    }
                }
                Err(_) => 0.0,
            },
            None => 0.0,
        }
    }
}

impl Imu for IioImu {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn imu_data(&self) -> ImuData {
        // Read raw values and apply scaling
        let accel_x = self.read_raw_value(&self.accel_x_path) * self.accel_scale;
        let accel_y = self.read_raw_value(&self.accel_y_path) * self.accel_scale;
        let accel_z = self.read_raw_value(&self.accel_z_path) * self.accel_scale;

        let gyro_x = self.read_raw_value(&self.gyro_x_path) * self.gyro_scale;
        let gyro_y = self.read_raw_value(&self.gyro_y_path) * self.gyro_scale;
        let gyro_z = self.read_raw_value(&self.gyro_z_path) * self.gyro_scale;

        let mag_x = self.read_raw_value(&self.mag_x_path) * self.mag_scale;
        let mag_y = self.read_raw_value(&self.mag_y_path) * self.mag_scale;
        let mag_z = self.read_raw_value(&self.mag_z_path) * self.mag_scale;

        let temp = if self.temp_is_input {
            // Already processed by driver; use as-is
            self.read_value_as_f32(&self.temp_path)
        } else {
            // Apply offset and scale for raw channel
            (self.read_raw_value(&self.temp_path) + self.temp_offset) * self.temp_scale
        };

        ImuData {
            accel: [accel_x, accel_y, accel_z],
            gyro: [gyro_x, gyro_y, gyro_z],
            mag: [mag_x, mag_y, mag_z],
            temp,
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
