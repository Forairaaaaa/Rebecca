use crate::cover_screen::CoverScreen;
use crate::player::convertor::convert_bpp;
use crate::player::types::ResizeMode;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use log::debug;
use std::{error::Error, path::Path};

pub async fn draw_image_from_data(
    screen: &mut impl CoverScreen,
    image_data: &[u8],
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    let src_bpp = 32; // RGBA8 格式
    let dst_bpp = screen.bpp() as u8;

    // 如果 bpp 不匹配，转换 bpp
    let converted = if src_bpp != dst_bpp {
        convert_bpp(image_data, width, height, src_bpp, dst_bpp)?
    } else {
        image_data.to_vec()
    };

    // 复制数据并推送
    screen.frame_buffer().copy_from_slice(&converted);
    screen.push_frame().await?;

    Ok(())
}

pub async fn draw_image_from_file<P: AsRef<Path>>(
    screen: &mut impl CoverScreen,
    path: P,
    resize_mode: ResizeMode,
) -> Result<(), Box<dyn Error>> {
    debug!("draw image: {}", path.as_ref().display());

    let width = screen.width();
    let height = screen.height();
    let img = image::open(path)?;

    let resized_img = resize_image(&img, width, height, &resize_mode);

    let src_data = resized_img.to_rgba8().into_raw();
    draw_image_from_data(screen, &src_data, width, height).await?;

    Ok(())
}

const RESIZE_FILTER: image::imageops::FilterType = image::imageops::FilterType::Triangle;

pub fn resize_image(
    img: &DynamicImage,
    screen_width: u32,
    screen_height: u32,
    resize_mode: &ResizeMode,
) -> DynamicImage {
    debug!("resize image in {:?}", resize_mode);
    match resize_mode {
        ResizeMode::Stretch => resize_stretch(img, screen_width, screen_height),
        ResizeMode::Letterbox => resize_letterbox(img, screen_width, screen_height),
        ResizeMode::Fill => resize_fill(img, screen_width, screen_height),
    }
}

fn resize_stretch(img: &DynamicImage, screen_w: u32, screen_h: u32) -> DynamicImage {
    img.resize_exact(screen_w, screen_h, RESIZE_FILTER)
}

fn resize_letterbox(img: &DynamicImage, screen_w: u32, screen_h: u32) -> DynamicImage {
    let (img_w, img_h) = img.dimensions();
    let img_ratio = img_w as f32 / img_h as f32;
    let screen_ratio = screen_w as f32 / screen_h as f32;

    let (target_w, target_h) = if img_ratio > screen_ratio {
        let w = screen_w;
        let h = ((w as f32) / img_ratio).round() as u32;
        (w, h)
    } else {
        let h = screen_h;
        let w = ((h as f32) * img_ratio).round() as u32;
        (w, h)
    };

    let resized = img.resize(target_w, target_h, RESIZE_FILTER);
    let mut full_image = ImageBuffer::from_pixel(screen_w, screen_h, Rgba([0, 0, 0, 255]));

    let offset_x = (screen_w - target_w) / 2;
    let offset_y = (screen_h - target_h) / 2;
    image::imageops::overlay(&mut full_image, &resized, offset_x.into(), offset_y.into());

    DynamicImage::ImageRgba8(full_image)
}

fn resize_fill(img: &DynamicImage, screen_w: u32, screen_h: u32) -> DynamicImage {
    let (img_w, img_h) = img.dimensions();
    let img_ratio = img_w as f32 / img_h as f32;
    let screen_ratio = screen_w as f32 / screen_h as f32;

    // 填充模式：等比缩放填满屏幕，可能裁剪图片
    let (target_w, target_h) = if img_ratio > screen_ratio {
        // 图片更宽，以高度为准，宽度会被裁剪
        let h = screen_h;
        let w = ((h as f32) * img_ratio).round() as u32;
        (w, h)
    } else {
        // 图片更高，以宽度为准，高度会被裁剪
        let w = screen_w;
        let h = ((w as f32) / img_ratio).round() as u32;
        (w, h)
    };

    // 先等比缩放到目标尺寸
    let resized = img.resize(target_w, target_h, RESIZE_FILTER);

    // 创建屏幕大小的图像
    let mut full_image = ImageBuffer::from_pixel(screen_w, screen_h, Rgba([0, 0, 0, 255]));

    // 计算居中偏移，负值表示需要裁剪
    let offset_x = ((screen_w as i32 - target_w as i32) / 2) as i32;
    let offset_y = ((screen_h as i32 - target_h as i32) / 2) as i32;

    // 将缩放后的图片覆盖到屏幕图像上，超出部分会被裁剪
    image::imageops::overlay(&mut full_image, &resized, offset_x.into(), offset_y.into());

    DynamicImage::ImageRgba8(full_image)
}
