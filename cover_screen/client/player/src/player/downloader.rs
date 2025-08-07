use log::{debug, info};
use reqwest::header::CONTENT_TYPE;
use std::{error::Error, fs, path::PathBuf};
use uuid::Uuid;

const TMP_FILE_DIR: &str = "/tmp/cover_screen_download";

pub struct Downloader {}

impl Downloader {
    /// Download resource from url and save to /tmp, return (resource path, content-type)
    /// # Arguments
    /// * `url` - The url of the resource to download
    /// # Returns
    /// * `Result<(PathBuf, String), Box<dyn Error>>` - The result of the operation
    pub async fn from_url(url: &str) -> Result<(PathBuf, String), Box<dyn Error>> {
        info!("download resource: {}", url);

        let response = reqwest::get(url).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("Download failed: HTTP {}", status).into());
        }

        // 获取 Content-Type
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        // 获取扩展名
        let ext = ext_from_content_type(&content_type)
            .ok_or(format!("unknown content-type: {}", content_type))?;
        debug!("get ext: {} from content-type: {}", ext, content_type);

        // 确保目录存在
        fs::create_dir_all(&TMP_FILE_DIR)?;

        // 保存文件
        let filename = format!("{}/{}.{}", TMP_FILE_DIR, Uuid::new_v4(), ext);
        let path = PathBuf::from(&filename);
        debug!("save file: {}", path.display());
        let bytes = response.bytes().await?;
        tokio::fs::write(&path, &bytes).await?;

        Ok((path, content_type))
    }

    pub fn cleanup() -> Result<(), Box<dyn Error>> {
        debug!("cleanup tmp files at: {}", TMP_FILE_DIR);

        let path = PathBuf::from(TMP_FILE_DIR);
        if path.exists() {
            fs::remove_dir_all(path)?;
        }

        Ok(())
    }
}

/// 从 Content-Type 获取扩展名
fn ext_from_content_type(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        "image/bmp" => Some("bmp"),
        "image/tiff" => Some("tiff"),
        _ => None,
    }
}
