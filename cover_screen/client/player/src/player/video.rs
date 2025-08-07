use crate::cover_screen::CoverScreen;
use crate::player::{BppConverter, ResizeMode};
use log::{debug, info};
use std::{error::Error, path::Path, process::Stdio};
use tokio::process::Command;

pub struct VideoPlayer {}

impl VideoPlayer {
    /// Play video from target
    /// # Arguments
    /// * `screen` - The screen to render the video to
    /// * `target` - The target video file
    /// * `resize_mode` - The resize mode to apply to the video
    /// * `repeat` - Whether to play the video in loop
    /// # Returns
    pub async fn from_target<P: AsRef<Path>>(
        screen: &mut impl CoverScreen,
        target: P,
        resize_mode: ResizeMode,
        repeat: bool,
    ) -> Result<(), Box<dyn Error>> {
        info!("play target video {}", target.as_ref().display());

        let screen_width = screen.width();
        let screen_height = screen.height();
        let screen_bpp = screen.bpp();

        // 构建ffmpeg命令
        let mut cmd = Command::new("ffmpeg");

        // 输入文件
        cmd.arg("-i").arg(target.as_ref());

        // 循环播放
        if repeat {
            cmd.arg("-stream_loop").arg("-1");
        }

        // 输出格式：原始RGB数据
        cmd.arg("-f").arg("rawvideo");
        cmd.arg("-pix_fmt").arg("rgba");

        // 获取视频信息
        let video_info = Self::get_video_info(target.as_ref()).await?;
        let (video_width, video_height, fps) = video_info;

        debug!(
            "video size: {}x{}, screen size: {}x{}, resize mode: {:?}, fps: {}",
            video_width, video_height, screen_width, screen_height, &resize_mode, fps
        );

        // 根据resize_mode设置ffmpeg的缩放参数
        Self::add_resize_parameters(&mut cmd, &resize_mode, screen_width, screen_height);

        // 使用原视频帧率
        cmd.arg("-r").arg(&fps.to_string());

        // 输出到stdout
        cmd.arg("-");

        // 设置stdout为管道
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        debug!("ffmpeg command: {:?}", cmd);

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        // 启动stderr读取任务（用于日志）
        let stderr_handle = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut buffer = [0; 1024];
            let mut stderr = tokio::io::BufReader::new(stderr);
            while let Ok(n) = stderr.read(&mut buffer).await {
                if n == 0 {
                    break;
                }
                if let Ok(s) = std::str::from_utf8(&buffer[..n]) {
                    debug!("ffmpeg stderr: {}", s.trim());
                }
            }
        });

        // 读取视频帧数据
        let frame_size = (screen_width * screen_height * 4) as usize; // RGBA = 4 bytes per pixel
        let mut frame_buffer = vec![0u8; frame_size];
        let mut stdout = tokio::io::BufReader::new(stdout);

        // 帧率控制：使用定时器
        let frame_interval = tokio::time::Duration::from_secs_f64(1.0 / fps);
        let mut interval = tokio::time::interval(frame_interval);

        loop {
            // 等待下一帧时间
            interval.tick().await;

            // 读取并处理一帧
            if let Err(e) = Self::process_frame(
                &mut stdout,
                &mut frame_buffer,
                screen,
                screen_width,
                screen_height,
                screen_bpp,
            )
            .await
            {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    // 视频播放结束
                    break;
                } else {
                    return Err(e.into());
                }
            }
        }

        // 等待ffmpeg进程结束
        let _ = child.wait().await;
        let _ = stderr_handle.await;

        Ok(())
    }

    /// 获取视频信息
    async fn get_video_info<P: AsRef<Path>>(
        video_path: P,
    ) -> Result<(u32, u32, f64), Box<dyn Error>> {
        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_streams")
            .arg(video_path.as_ref())
            .output()
            .await?;

        if !output.status.success() {
            return Err("Failed to get video info".into());
        }

        let output_str = String::from_utf8(output.stdout)?;
        let json: serde_json::Value = serde_json::from_str(&output_str)?;

        // 查找视频流
        if let Some(streams) = json["streams"].as_array() {
            for stream in streams {
                if stream["codec_type"] == "video" {
                    let width = stream["width"].as_u64().unwrap_or(0) as u32;
                    let height = stream["height"].as_u64().unwrap_or(0) as u32;

                    // 获取帧率
                    let fps = Self::parse_frame_rate(stream);

                    debug!("video size: {}x{}, fps: {}", width, height, fps);
                    return Ok((width, height, fps));
                }
            }
        }

        Err("No video stream found".into())
    }

    /// 解析帧率字符串，如 "30000/1001"
    fn parse_frame_rate(stream: &serde_json::Value) -> f64 {
        const DEFAULT_FPS: f64 = 30.0;

        let r_frame_rate = match stream["r_frame_rate"].as_str() {
            Some(rate) => rate,
            None => return DEFAULT_FPS,
        };

        let parts: Vec<&str> = r_frame_rate.split('/').collect();
        if parts.len() != 2 {
            return DEFAULT_FPS;
        }

        let num = match parts[0].parse::<f64>() {
            Ok(n) => n,
            Err(_) => return DEFAULT_FPS,
        };

        let den = match parts[1].parse::<f64>() {
            Ok(d) => d,
            Err(_) => return DEFAULT_FPS,
        };

        if den == 0.0 { DEFAULT_FPS } else { num / den }
    }

    /// 根据resize_mode添加ffmpeg的缩放参数
    fn add_resize_parameters(
        cmd: &mut Command,
        resize_mode: &ResizeMode,
        screen_width: u32,
        screen_height: u32,
    ) {
        match resize_mode {
            ResizeMode::Stretch => {
                // 直接拉伸到屏幕尺寸
                cmd.arg("-s")
                    .arg(format!("{}x{}", screen_width, screen_height));
            }
            ResizeMode::Letterbox => {
                // 等比缩放并居中，周围填充黑色
                let filter = format!(
                    "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:black",
                    screen_width, screen_height, screen_width, screen_height
                );
                cmd.arg("-vf").arg(filter);
            }
            ResizeMode::Fill => {
                // 等比缩放填满屏幕，可能裁剪
                let filter = format!(
                    "scale={}:{}:force_original_aspect_ratio=increase,crop={}:{}",
                    screen_width, screen_height, screen_width, screen_height
                );
                cmd.arg("-vf").arg(filter);
            }
        }
    }

    /// 处理单帧数据
    async fn process_frame(
        stdout: &mut tokio::io::BufReader<tokio::process::ChildStdout>,
        frame_buffer: &mut [u8],
        screen: &mut impl CoverScreen,
        screen_width: u32,
        screen_height: u32,
        screen_bpp: u32,
    ) -> Result<(), std::io::Error> {
        use tokio::io::AsyncReadExt;

        // 读取一帧数据
        stdout.read_exact(frame_buffer).await?;

        // 转换bpp格式
        let converted = if screen_bpp != 32 {
            BppConverter::convert(
                frame_buffer,
                screen_width,
                screen_height,
                32, // RGBA
                screen_bpp as u8,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        } else {
            frame_buffer.to_vec()
        };

        // 推送到屏幕
        screen.frame_buffer().copy_from_slice(&converted);
        screen.push_frame().await?;

        Ok(())
    }
}
