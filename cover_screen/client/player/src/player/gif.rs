use crate::cover_screen::CoverScreen;
use crate::player::image::{draw_image_from_data, resize_image};
use crate::player::types::ResizeMode;
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, Delay};
use log::debug;
use std::fs::File;
use std::io::BufReader;
use std::{error::Error, path::Path};

pub async fn play_gif<P: AsRef<Path>>(
    screen: &mut impl CoverScreen,
    path: P,
    resize_mode: ResizeMode,
) -> Result<(), Box<dyn Error>> {
    debug!("play gif from {}", path.as_ref().display());

    let image_data_frames = convert_to_image_data_frames(screen, path, resize_mode)?;

    for (image_data, delay) in image_data_frames {
        draw_image_from_data(screen, &image_data, screen.width(), screen.height()).await?;
        tokio::time::sleep(delay.into()).await;
    }

    Ok(())
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
        let resized_img = resize_image(&img, width, height, &resize_mode);
        let image_data = resized_img.to_rgba8().into_raw();
        image_data_frames.push((image_data, delay));
    }

    Ok(image_data_frames)
}
