use crate::cover_screen::fb::scan_fb_devices;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use log::debug;

#[derive(Debug, Clone)]
pub struct ScreenInfo {
    pub path: PathBuf,
    pub name: String,
    pub resolution: (u32, u32),
    pub bpp: u32,
    pub device_path: PathBuf,
}

fn excluded_names() -> &'static [&'static str] {
    &[
        "drm-rp1-dsidrmf", // 主屏幕
    ]
}

pub fn scan_screens() -> io::Result<Vec<ScreenInfo>> {
    let fb_list = scan_fb_devices()?;
    let mut screens = Vec::new();

    for fb in fb_list {
        if fb.name.is_none()
            || fb.bpp.is_none()
            || fb.resolution.is_none()
            || fb.device_path.is_none()
        {
            continue;
        }

        if excluded_names()
            .iter()
            .any(|&n| n == fb.name.as_ref().unwrap())
        {
            continue;
        }

        screens.push(ScreenInfo {
            path: fb.path,
            name: fb.name.unwrap(),
            resolution: fb.resolution.unwrap(),
            bpp: fb.bpp.unwrap(),
            device_path: fb.device_path.unwrap(),
        });
    }

    Ok(screens)
}

pub fn push_frame_buffer(screen: &ScreenInfo, frame_buffer: &[u8]) -> io::Result<()> {
    debug!(
        "push frame buffer {} bytes to {}",
        frame_buffer.len(),
        screen.path.display()
    );

    let expected_len = (screen.resolution.0 * screen.resolution.1 * screen.bpp / 8) as usize;
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

    let mut file = OpenOptions::new().write(true).open(&screen.path)?;
    file.write_all(frame_buffer)?;

    Ok(())
}
