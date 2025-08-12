use crate::devices::screen::Screen;
use log::{debug, info, warn};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Frame buffer screen
/// 副屏的 fb 实现，可以将 buffer 写入 /dev/fbx
#[derive(Debug)]
pub struct FrameBufferScreen {
    bpp: u32,
    size: (u32, u32),
    path: PathBuf,
    device_path: PathBuf,
}

impl FrameBufferScreen {
    /// Scan and create screen for available frame buffer
    pub fn new() -> io::Result<Vec<Self>> {
        let mut fb_screens: Vec<Self> = Vec::new();

        // Iterate over all frame buffer directories
        for entry in fs::read_dir("/sys/class/graphics")? {
            let entry = entry?;
            let entry_path = entry.path();

            // Check fb directory name
            let Some(fb_dir_name) = entry_path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !fb_dir_name.starts_with("fb") {
                continue;
            }

            // get fb write path
            let path = PathBuf::from(format!("/dev/{}", fb_dir_name));

            // Get fb name
            let Some(name) = Self::read_to_string(entry_path.join("name")).ok() else {
                warn!(
                    "failed to read name of frame buffer: {}",
                    entry_path.display()
                );
                continue;
            };

            // Check excluded
            if Self::is_excluded(&name) {
                info!("excluded frame buffer of name: {}", name);
                continue;
            }

            // Get bpp
            let bpp = Self::read_to_string(entry_path.join("bits_per_pixel"))
                .ok()
                .and_then(|s| s.trim().parse().ok());
            let Some(bpp) = bpp else {
                warn!(
                    "failed to read bpp of frame buffer: {}",
                    entry_path.display()
                );
                continue;
            };

            // Get size
            let Some(size) = Self::read_to_string(entry_path.join("virtual_size"))
                .ok()
                .and_then(|s| {
                    let mut parts = s.trim().split(',');
                    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
                })
            else {
                warn!(
                    "failed to read size of frame buffer: {}",
                    entry_path.display()
                );
                continue;
            };

            // Get device path
            let Some(device_path) = fs::read_link(entry_path.join("device"))
                .ok()
                .map(|p| fs::canonicalize(&p).unwrap_or_else(|_| p))
            else {
                warn!(
                    "failed to read device path of frame buffer: {}",
                    entry_path.display()
                );
                continue;
            };

            let fb_screen = Self {
                bpp,
                size,
                path,
                device_path,
            };
            debug!("new frame buffer screen: {:#?}", fb_screen);

            fb_screens.push(fb_screen);
        }

        Ok(fb_screens)
    }

    fn is_excluded(name: &str) -> bool {
        static EXCLUDED_FB_NAMES: &[&str] = &[
            "drm-rp1-dsidrmf", // 主屏幕
        ];
        EXCLUDED_FB_NAMES.contains(&name)
    }

    fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
        let mut s = String::new();
        fs::File::open(path)?.read_to_string(&mut s)?;
        Ok(s.trim().to_string())
    }
}

impl Screen for FrameBufferScreen {
    fn bpp(&self) -> u32 {
        self.bpp
    }
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn device_type(&self) -> String {
        self.device_path.display().to_string()
    }

    fn push_frame_buffer(&self, frame_buffer: &[u8]) -> io::Result<()> {
        debug!(
            "push frame buffer {} bytes to {}",
            frame_buffer.len(),
            self.path.display()
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

        let mut file = fs::OpenOptions::new().write(true).open(&self.path)?;
        file.write_all(frame_buffer)?;

        Ok(())
    }
}
