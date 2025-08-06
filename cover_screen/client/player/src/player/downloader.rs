use reqwest::header::CONTENT_TYPE;
use std::{error::Error, path::PathBuf};
use uuid::Uuid;

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

/// 下载资源并保存到 /tmp，返回 (资源路径, content-type)
pub async fn download_resource(url: &str) -> Result<(PathBuf, String), Box<dyn Error>> {
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
    let filename = format!("/tmp/cover_screen_download_{}.{}", Uuid::new_v4(), ext);
    let path = PathBuf::from(&filename);

    // 写入文件
    let bytes = response.bytes().await?;
    tokio::fs::write(&path, &bytes).await?;

    Ok((path, content_type))
}
