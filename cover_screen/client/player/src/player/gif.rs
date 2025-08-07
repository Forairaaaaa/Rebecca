use crate::cover_screen::CoverScreen;
use crate::player::ImageRenderer;
use crate::player::ResizeMode;
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, Delay};
use log::{debug, info};
use std::fs::File;
use std::io::BufReader;
use std::{error::Error, path::Path};

pub struct GifPlayer {}

impl GifPlayer {
    /// Play gif from file
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
        info!("play gif: {}", path.as_ref().display());

        info!("caching resized frames...");
        let image_data_frames = convert_to_image_data_frames(screen, path, resize_mode)?;

        info!("playing gif...");
        loop {
            for (image_data, delay) in &image_data_frames {
                ImageRenderer::from_image_data(
                    screen,
                    &image_data,
                    screen.width(),
                    screen.height(),
                )
                .await?;
                tokio::time::sleep(delay.clone().into()).await;
            }

            if !repeat {
                break;
            }
        }

        Ok(())
    }
}

fn convert_to_image_data_frames<P: AsRef<Path>>(
    screen: &mut impl CoverScreen,
    path: P,
    resize_mode: ResizeMode,
) -> Result<Vec<(Vec<u8>, Delay)>, Box<dyn Error>> {
    // Open and decode gif file
    let file_in = BufReader::new(File::open(path)?);
    let decoder = GifDecoder::new(file_in)?;
    let frames = decoder.into_frames();
    let frames = frames.collect_frames()?;

    // Resize and store frames
    let width = screen.width();
    let height = screen.height();

    let mut image_data_frames: Vec<(Vec<u8>, Delay)> = Vec::new();
    for frame in frames {
        let frame_data = frame.buffer();
        let delay = frame.delay();
        let left = frame.left();
        let top = frame.top();

        debug!(
            "raw frame info: size: {}x{}, delay: {:?}, left: {:?}, top: {:?}",
            frame_data.width(),
            frame_data.height(),
            delay,
            left,
            top
        );

        let img: image::DynamicImage = image::DynamicImage::ImageRgba8(frame_data.clone());
        let resized_img = ImageRenderer::resize(&img, width, height, &resize_mode);
        let image_data = resized_img.to_rgba8().into_raw();
        image_data_frames.push((image_data, delay));
    }

    Ok(image_data_frames)
}
