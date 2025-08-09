use std::io;

/// Screen trait
pub trait Screen: Send {
    fn bpp(&self) -> u32;
    fn size(&self) -> (u32, u32);
    fn device_type(&self) -> String;
    fn push_frame_buffer(&self, frame_buffer: &[u8]) -> io::Result<()>;
}
