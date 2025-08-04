use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FbInfo {
    pub path: PathBuf,
    pub name: Option<String>,
    pub resolution: Option<(u32, u32)>,
    pub bpp: Option<u32>,
    pub device_path: Option<PathBuf>,
}

pub fn scan_fb_devices() -> io::Result<Vec<FbInfo>> {
    let mut result = Vec::new();

    for entry in fs::read_dir("/sys/class/graphics")? {
        let entry = entry?;
        let entry_path = entry.path();

        let fb_name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if !fb_name.starts_with("fb") {
            continue;
        }

        let path = PathBuf::from(format!("/dev/{}", fb_name));

        let name = read_to_string(entry_path.join("name")).ok();
        let bpp = read_to_string(entry_path.join("bits_per_pixel"))
            .ok()
            .and_then(|s| s.trim().parse().ok());
        let resolution = read_to_string(entry_path.join("virtual_size"))
            .ok()
            .and_then(|s| {
                let mut parts = s.trim().split(',');
                Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
            });

        let device_path = fs::read_link(entry_path.join("device"))
            .ok()
            .map(|p| fs::canonicalize(&p).unwrap_or_else(|_| p));

        result.push(FbInfo {
            path,
            name,
            resolution,
            bpp,
            device_path,
        });
    }

    Ok(result)
}

fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut s = String::new();
    fs::File::open(path)?.read_to_string(&mut s)?;
    Ok(s.trim().to_string())
}
