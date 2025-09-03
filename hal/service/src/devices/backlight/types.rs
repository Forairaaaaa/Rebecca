use std::io;

/// Backlight device trait
pub trait Backlight {
    fn name(&self) -> String;
    fn device_type(&self) -> String;
    fn max_brightness(&self) -> u32;
    fn get_brightness(&self) -> io::Result<f32>; // normalized to 0.0~1.0
    fn set_brightness(&self, brightness: f32) -> io::Result<()>; // normalized to 0.0~1.0
    fn init(&self) -> io::Result<()>;
    fn deinit(&self) -> io::Result<()>;
}
