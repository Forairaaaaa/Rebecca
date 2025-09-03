use crate::devices::backlight::Backlight;
use log::{debug, warn};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
pub struct SysfsBacklight {
    name: String,
    device_path: PathBuf,
    brightness_path: PathBuf,
    max_brightness_path: PathBuf,
    max_brightness: u32,
}

impl SysfsBacklight {
    pub fn new(name: &str) -> Option<Self> {
        debug!("searching for backlight device: {}", name);

        // Scan /sys/class/backlight for matching device
        let backlight_devices_path = Path::new("/sys/class/backlight");
        if !backlight_devices_path.exists() {
            warn!(
                "backlight devices path does not exist: {}",
                backlight_devices_path.display()
            );
            return None;
        }

        let entries = match fs::read_dir(backlight_devices_path) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("failed to read backlight devices directory: {}", e);
                return None;
            }
        };

        for entry in entries.flatten() {
            let device_name = entry.file_name().to_string_lossy().to_string();
            if device_name.contains(name) {
                let device_path = entry.path();
                let brightness_path = device_path.join("brightness");
                let max_brightness_path = device_path.join("max_brightness");

                // Check if required files exist
                if !brightness_path.exists() || !max_brightness_path.exists() {
                    warn!("backlight device {} missing required files", device_name);
                    continue;
                }

                // Read max brightness
                let max_brightness = match fs::read_to_string(&max_brightness_path) {
                    Ok(content) => match content.trim().parse::<u32>() {
                        Ok(val) => val,
                        Err(e) => {
                            warn!("failed to parse max_brightness: {}", e);
                            continue;
                        }
                    },
                    Err(e) => {
                        warn!("failed to read max_brightness: {}", e);
                        continue;
                    }
                };

                debug!(
                    "found backlight device: {} at {}",
                    device_name,
                    device_path.display()
                );
                return Some(Self {
                    name: device_name,
                    device_path,
                    brightness_path,
                    max_brightness_path,
                    max_brightness,
                });
            }
        }

        warn!("backlight device not found: {}", name);
        None
    }

    /// Get all available backlight devices
    pub fn get_all_devices() -> Vec<Self> {
        let mut devices = Vec::new();
        let backlight_devices_path = Path::new("/sys/class/backlight");

        if !backlight_devices_path.exists() {
            warn!("backlight devices path does not exist");
            return devices;
        }

        let entries = match fs::read_dir(backlight_devices_path) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("failed to read backlight devices directory: {}", e);
                return devices;
            }
        };

        for entry in entries.flatten() {
            let device_name = entry.file_name().to_string_lossy().to_string();
            if let Some(device) = Self::new(&device_name) {
                devices.push(device);
            }
        }

        devices
    }
}

impl Backlight for SysfsBacklight {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn device_type(&self) -> String {
        format!("/sys/class/backlight/{}", self.name)
    }

    fn max_brightness(&self) -> u32 {
        self.max_brightness
    }

    fn get_brightness(&self) -> io::Result<f32> {
        let content = fs::read_to_string(&self.brightness_path)?;
        let brightness = content.trim().parse::<u32>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("parse brightness failed: {}", e),
            )
        })?;

        // Normalize to 0.0~1.0
        Ok(brightness as f32 / self.max_brightness as f32)
    }

    fn set_brightness(&self, brightness: f32) -> io::Result<()> {
        // Clamp to 0.0~1.0
        let brightness = brightness.clamp(0.0, 1.0);

        // Convert to raw value
        let raw_brightness = (brightness * self.max_brightness as f32).round() as u32;

        fs::write(&self.brightness_path, raw_brightness.to_string())?;
        debug!(
            "set backlight {} brightness to {} (raw: {})",
            self.name, brightness, raw_brightness
        );

        Ok(())
    }

    fn init(&self) -> io::Result<()> {
        debug!("initializing backlight device: {}", self.name);
        // For sysfs backlight, no special initialization is needed
        Ok(())
    }

    fn deinit(&self) -> io::Result<()> {
        debug!("deinitializing backlight device: {}", self.name);
        // For sysfs backlight, no special cleanup is needed
        Ok(())
    }
}
