use crate::cover_screen::CoverScreen;
use crate::player::{FFmpeg, ResizeMode};
use log::{debug, info};
use std::{error::Error, path::Path};

pub struct ColorBar {}

impl ColorBar {
    /// Draw color bar using ffmpeg
    /// # Arguments
    /// * `cover_screen` - The cover screen to draw the color bar to
    /// # Returns
    /// * `Result<(), Box<dyn Error>>` - The result of the operation
    pub async fn draw(cover_screen: &mut impl CoverScreen) -> Result<(), Box<dyn Error>> {
        info!("draw color bar with ffmpeg");

        let width = cover_screen.width();
        let height = cover_screen.height();
        let target_bpp = cover_screen.bpp();

        // 使用ffmpeg生成SMPTE彩条，直接输出目标bpp格式
        let source = format!("testsrc=duration=1:size={}x{}:rate=1", width, height);
        let frame_data = FFmpeg::execute_test_source(&source, width, height, target_bpp).await?;

        // 将数据复制到目标 frame buffer
        let frame_buffer = cover_screen.frame_buffer();
        frame_buffer[..frame_data.len()].copy_from_slice(&frame_data);

        cover_screen.push_frame().await?;

        Ok(())
    }
}

pub struct GifPlayer {}

impl GifPlayer {
    /// Play gif from file using ffmpeg
    /// # Arguments
    /// * `screen` - The screen to render the gif to
    /// * `path` - The path to the gif file
    /// * `resize_mode` - The resize mode to apply to the gif
    /// * `repeat` - Whether to play the gif in loop
    /// # Returns
    /// * `Result<(), Box<dyn Error>>` - The result of the operation
    pub async fn from_file<P: AsRef<Path>>(
        screen: &mut impl CoverScreen,
        path: P,
        resize_mode: ResizeMode,
        repeat: bool,
    ) -> Result<(), Box<dyn Error>> {
        info!("play gif with ffmpeg: {}", path.as_ref().display());

        let screen_width = screen.width();
        let screen_height = screen.height();
        let screen_bpp = screen.bpp();

        let fps = FFmpeg::get_media_fps(&path).await?;
        debug!("gif fps: {}", fps);

        FFmpeg::execute_stream(
            path,
            &resize_mode,
            screen_width,
            screen_height,
            screen_bpp,
            fps,
            repeat,
            screen,
        )
        .await
    }
}

pub struct ImageRenderer {}

impl ImageRenderer {
    /// Render image from file using ffmpeg
    /// # Arguments
    /// * `screen` - The screen to render the image to
    /// * `path` - The path to the image file
    /// * `resize_mode` - The resize mode to apply to the image
    /// # Returns
    /// * `Result<(), Box<dyn Error>>` - The result of the operation
    pub async fn from_file<P: AsRef<Path>>(
        screen: &mut impl CoverScreen,
        path: P,
        resize_mode: ResizeMode,
    ) -> Result<(), Box<dyn Error>> {
        debug!("draw image with ffmpeg: {}", path.as_ref().display());

        let screen_width = screen.width();
        let screen_height = screen.height();
        let screen_bpp = screen.bpp();

        let frame_data = FFmpeg::execute_single_frame(
            path,
            &resize_mode,
            screen_width,
            screen_height,
            screen_bpp,
        )
        .await?;

        screen.frame_buffer().copy_from_slice(&frame_data);
        screen.push_frame().await?;

        Ok(())
    }
}

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

        let (video_width, video_height, fps) = FFmpeg::get_media_info(&target).await?;

        debug!(
            "video size: {}x{}, screen size: {}x{}, resize mode: {:?}, fps: {}",
            video_width, video_height, screen_width, screen_height, &resize_mode, fps
        );

        FFmpeg::execute_stream(
            target,
            &resize_mode,
            screen_width,
            screen_height,
            screen_bpp,
            fps,
            repeat,
            screen,
        )
        .await
    }
}
