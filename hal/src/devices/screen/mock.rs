use crate::devices::screen::Screen;
use log::debug;
use std::io;

/// Mock screen
/// 模拟副屏，可以在没有实际硬件的情况下模拟屏幕接口
#[derive(Debug)]
pub struct MockScreen {
    bpp: u32,
    size: (u32, u32),
}

impl MockScreen {
    pub fn new(width: u32, height: u32, bpp: u32) -> Self {
        Self {
            bpp,
            size: (width, height),
        }
    }
}

impl Screen for MockScreen {
    fn bpp(&self) -> u32 {
        self.bpp
    }
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn device_type(&self) -> String {
        "mock".to_string()
    }

    fn push_frame_buffer(&self, frame_buffer: &[u8]) -> io::Result<()> {
        debug!(
            "push frame buffer {} bytes to mock screen",
            frame_buffer.len(),
        );

        let expected_len = (self.size.0 * self.size.1 * self.bpp / 8) as usize;
        if frame_buffer.len() != expected_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Expected {} bytes, got {}",
                    expected_len,
                    frame_buffer.len()
                ),
            ));
        }

        // pass

        Ok(())
    }
}
