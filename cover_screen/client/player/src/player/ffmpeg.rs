use crate::cover_screen::CoverScreen;
use crate::player::ResizeMode;
use log::debug;
use std::{error::Error, path::Path, process::Stdio};
use tokio::process::{Child, ChildStderr, ChildStdout, Command};

pub struct FFmpeg {}

impl FFmpeg {
    /// 构建基础的ffmpeg命令
    pub fn build_command() -> Command {
        Command::new("ffmpeg")
    }

    /// 添加循环播放参数
    pub fn add_loop_parameter(cmd: &mut Command, repeat: bool) {
        if repeat {
            cmd.arg("-stream_loop").arg("-1");
        }
    }

    /// 添加输入文件
    pub fn add_input_file<P: AsRef<Path>>(cmd: &mut Command, path: P) {
        cmd.arg("-i").arg(path.as_ref());
    }

    /// 添加测试源输入（用于生成彩条等测试图案）
    pub fn add_test_source_input(cmd: &mut Command, source: &str) {
        cmd.arg("-f").arg("lavfi").arg("-i").arg(source);
    }

    /// 添加原始视频输出参数
    pub fn add_raw_output_params(cmd: &mut Command) {
        cmd.arg("-f").arg("rawvideo");
    }

    /// 根据目标bpp设置像素格式
    pub fn set_pixel_format(cmd: &mut Command, target_bpp: u32) {
        match target_bpp {
            16 => {
                cmd.arg("-pix_fmt").arg("rgb565");
            }
            24 => {
                cmd.arg("-pix_fmt").arg("rgb24");
            }
            32 => {
                cmd.arg("-pix_fmt").arg("rgba");
            }
            _ => {
                // 默认使用rgba，后续在Rust中转换
                cmd.arg("-pix_fmt").arg("rgba");
            }
        }
    }

    /// 添加单帧输出参数（用于图像处理）
    pub fn add_single_frame_params(cmd: &mut Command) {
        cmd.arg("-frames:v").arg("1");
    }

    /// 根据resize_mode添加ffmpeg的缩放参数
    pub fn add_resize_parameters(
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

    /// 添加帧率参数
    pub fn add_framerate_param(cmd: &mut Command, fps: f64) {
        cmd.arg("-r").arg(&fps.to_string());
    }

    /// 添加输出到stdout
    pub fn add_stdout_output(cmd: &mut Command) {
        cmd.arg("-");
    }

    /// 设置stdout和stderr管道
    pub fn set_pipes(cmd: &mut Command) {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
    }

    /// 启动ffmpeg进程
    pub async fn spawn_process(
        cmd: &mut Command,
    ) -> Result<(Child, ChildStdout, ChildStderr), Box<dyn Error>> {
        debug!("ffmpeg command: {:?}", cmd);

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        Ok((child, stdout, stderr))
    }

    /// 启动stderr读取任务
    pub fn spawn_stderr_reader(stderr: ChildStderr) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
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
        })
    }

    /// 获取媒体文件的帧率
    pub async fn get_media_fps<P: AsRef<Path>>(media_path: P) -> Result<f64, Box<dyn Error>> {
        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_streams")
            .arg(media_path.as_ref())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(30.0); // 默认帧率
        }

        let output_str = String::from_utf8(output.stdout)?;
        let json: serde_json::Value = serde_json::from_str(&output_str)?;

        // 查找视频流
        if let Some(streams) = json["streams"].as_array() {
            for stream in streams {
                if stream["codec_type"] == "video" {
                    // 尝试获取帧率
                    if let Some(r_frame_rate) = stream["r_frame_rate"].as_str() {
                        let parts: Vec<&str> = r_frame_rate.split('/').collect();
                        if parts.len() == 2 {
                            if let (Ok(num), Ok(den)) =
                                (parts[0].parse::<f64>(), parts[1].parse::<f64>())
                            {
                                if den != 0.0 {
                                    return Ok(num / den);
                                }
                            }
                        }
                    }
                    // 尝试获取duration和nb_frames
                    if let (Some(duration), Some(nb_frames)) = (
                        stream["duration"]
                            .as_str()
                            .and_then(|s| s.parse::<f64>().ok()),
                        stream["nb_frames"]
                            .as_str()
                            .and_then(|s| s.parse::<f64>().ok()),
                    ) {
                        if duration > 0.0 && nb_frames > 0.0 {
                            return Ok(nb_frames / duration);
                        }
                    }
                }
            }
        }

        Ok(30.0) // 默认帧率
    }

    /// 获取媒体文件的尺寸和帧率
    pub async fn get_media_info<P: AsRef<Path>>(
        media_path: P,
    ) -> Result<(u32, u32, f64), Box<dyn Error>> {
        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_streams")
            .arg(media_path.as_ref())
            .output()
            .await?;

        if !output.status.success() {
            return Err("Failed to get media info".into());
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

                    debug!("media size: {}x{}, fps: {}", width, height, fps);
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

    /// 执行单次ffmpeg命令（用于图像处理）
    pub async fn execute_single_frame<P: AsRef<Path>>(
        media_path: P,
        resize_mode: &ResizeMode,
        screen_width: u32,
        screen_height: u32,
        target_bpp: u32,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut cmd = Self::build_command();
        Self::add_input_file(&mut cmd, media_path);
        Self::add_raw_output_params(&mut cmd);
        Self::set_pixel_format(&mut cmd, target_bpp);
        Self::add_single_frame_params(&mut cmd);
        Self::add_resize_parameters(&mut cmd, resize_mode, screen_width, screen_height);
        Self::add_stdout_output(&mut cmd);
        Self::set_pipes(&mut cmd);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ffmpeg failed: {}", stderr).into());
        }

        let frame_data = output.stdout;
        let bytes_per_pixel = target_bpp / 8;
        let frame_size = (screen_width * screen_height * bytes_per_pixel) as usize;

        if frame_data.len() != frame_size {
            return Err(format!(
                "Invalid frame size: expected {} ({}x{}x{}), got {}",
                frame_size,
                screen_width,
                screen_height,
                bytes_per_pixel,
                frame_data.len()
            )
            .into());
        }

        Ok(frame_data)
    }

    /// 执行测试源命令（用于生成彩条等测试图案）
    pub async fn execute_test_source(
        source: &str,
        screen_width: u32,
        screen_height: u32,
        target_bpp: u32,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut cmd = Self::build_command();
        Self::add_test_source_input(&mut cmd, source);
        Self::add_raw_output_params(&mut cmd);
        Self::set_pixel_format(&mut cmd, target_bpp);
        Self::add_single_frame_params(&mut cmd);
        Self::add_stdout_output(&mut cmd);
        Self::set_pipes(&mut cmd);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ffmpeg failed: {}", stderr).into());
        }

        let frame_data = output.stdout;
        let bytes_per_pixel = target_bpp / 8;
        let frame_size = (screen_width * screen_height * bytes_per_pixel) as usize;

        if frame_data.len() != frame_size {
            return Err(format!(
                "Invalid frame size: expected {} ({}x{}x{}), got {}",
                frame_size,
                screen_width,
                screen_height,
                bytes_per_pixel,
                frame_data.len()
            )
            .into());
        }

        Ok(frame_data)
    }

    /// 执行流式ffmpeg命令（用于视频和GIF播放）
    pub async fn execute_stream<P: AsRef<Path>, S: CoverScreen>(
        media_path: P,
        resize_mode: &ResizeMode,
        screen_width: u32,
        screen_height: u32,
        screen_bpp: u32,
        fps: f64,
        repeat: bool,
        screen: &mut S,
    ) -> Result<(), Box<dyn Error>> {
        // 构建ffmpeg命令
        let mut cmd = Self::build_command();

        // 循环播放（必须在输入文件之前）
        Self::add_loop_parameter(&mut cmd, repeat);

        // 输入文件
        Self::add_input_file(&mut cmd, media_path);

        // 输出格式：原始RGB数据
        Self::add_raw_output_params(&mut cmd);

        // 设置像素格式
        Self::set_pixel_format(&mut cmd, screen_bpp);

        // 根据resize_mode设置ffmpeg的缩放参数
        Self::add_resize_parameters(&mut cmd, resize_mode, screen_width, screen_height);

        // 使用媒体原始帧率
        Self::add_framerate_param(&mut cmd, fps);

        // 输出到stdout
        Self::add_stdout_output(&mut cmd);

        // 设置stdout为管道
        Self::set_pipes(&mut cmd);

        // 启动ffmpeg进程
        let (mut child, stdout, stderr) = Self::spawn_process(&mut cmd).await?;

        // 启动stderr读取任务（用于日志）
        Self::spawn_stderr_reader(stderr);

        // 创建帧率控制定时器
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs_f64(1.0 / fps));

        // 读取帧数据
        let bytes_per_pixel = screen_bpp / 8;
        let frame_size = (screen_width * screen_height * bytes_per_pixel) as usize;
        let mut frame_buffer = vec![0u8; frame_size];
        let mut stdout = tokio::io::BufReader::new(stdout);

        loop {
            use tokio::io::AsyncReadExt;

            // 等待下一帧时间
            interval.tick().await;

            // 读取一帧数据
            match stdout.read_exact(&mut frame_buffer).await {
                Ok(_) => {
                    // 推送到屏幕（ffmpeg已经输出正确的bpp格式）
                    screen.frame_buffer().copy_from_slice(&frame_buffer);
                    screen.push_frame().await?;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        // 媒体播放结束
                        break;
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }

        // 等待进程结束
        child.wait().await?;
        Ok(())
    }
}
