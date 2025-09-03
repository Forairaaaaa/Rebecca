use crate::devices::backlight::Backlight;
use log::debug;
use std::io;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MockBacklight {
    name: String,
    max_brightness: u32,
    brightness: Arc<RwLock<f32>>, // normalized 0.0~1.0
}

impl MockBacklight {
    pub fn new(name: &str, max_brightness: u32) -> Self {
        Self {
            name: name.to_string(),
            max_brightness,
            brightness: Arc::new(RwLock::new(0.6)),
        }
    }
}

impl Backlight for MockBacklight {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn device_type(&self) -> String {
        "mock".to_string()
    }

    fn max_brightness(&self) -> u32 {
        self.max_brightness
    }

    fn get_brightness(&self) -> io::Result<f32> {
        // Use try_read to avoid blocking in sync context
        let brightness = self
            .brightness
            .try_read()
            .map_err(|_| io::Error::new(io::ErrorKind::WouldBlock, "brightness lock busy"))?;
        let brightness_val = *brightness;
        debug!(
            "mock backlight {} get brightness: {}",
            self.name, brightness_val
        );
        Ok(brightness_val)
    }

    fn set_brightness(&self, brightness: f32) -> io::Result<()> {
        let brightness = brightness.clamp(0.0, 1.0);
        // Use try_write to avoid blocking in sync context
        let mut brightness_guard = self
            .brightness
            .try_write()
            .map_err(|_| io::Error::new(io::ErrorKind::WouldBlock, "brightness lock busy"))?;
        *brightness_guard = brightness;
        debug!(
            "mock backlight {} set brightness to: {}",
            self.name, brightness
        );
        Ok(())
    }

    fn init(&self) -> io::Result<()> {
        debug!("initializing mock backlight device: {}", self.name);
        Ok(())
    }

    fn deinit(&self) -> io::Result<()> {
        debug!("deinitializing mock backlight device: {}", self.name);
        Ok(())
    }
}
