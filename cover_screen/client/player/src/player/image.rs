use crate::cover_screen::CoverScreen;
use crate::player::convertor::convert_bpp;
use clap::ValueEnum;
use image::{GenericImageView, ImageBuffer, Rgba, RgbaImage};
use log::info;
use std::{error::Error, path::Path};

#[derive(Debug, Clone, ValueEnum)]
pub enum ResizeMode {
    Stretch,   // 拉伸
    Letterbox, // 等比缩放，居中显示
    Fill,      // 等比缩放，填满屏幕（可能裁剪）
}

pub async fn draw_image<P: AsRef<Path>>(
    screen: &mut impl CoverScreen,
    path: P,
    resize_mode: ResizeMode,
) -> Result<(), Box<dyn Error>> {
    info!("draw image: {}", path.as_ref().display());

    // 加载图片
    let img = image::open(path)?;

    let (screen_width, screen_height) = (screen.width(), screen.height());

    // 调整图像尺寸
    info!("resize image in {:?}", resize_mode);
    let resized_img = match resize_mode {
        ResizeMode::Stretch => resize_stretch(&img, screen_width, screen_height),
        ResizeMode::Letterbox => resize_letterbox(&img, screen_width, screen_height),
        ResizeMode::Fill => resize_fill(&img, screen_width, screen_height),
    };

    // 获取像素格式：RGBA8
    let src_data = resized_img.as_raw();

    let src_bpp = 32;
    let dst_bpp = screen.bpp() as u8;

    // 转换像素格式（bpp）
    let converted = if src_bpp != dst_bpp {
        convert_bpp(src_data, screen_width, screen_height, src_bpp, dst_bpp)?
    } else {
        src_data.clone()
    };

    // 将转换后的数据写入 frame buffer
    let fb = screen.frame_buffer();
    if fb.len() != converted.len() {
        fb.resize(converted.len(), 0); // 确保大小一致
    }
    fb.copy_from_slice(&converted);

    // 调用 push_frame 显示
    screen.push_frame().await?;

    Ok(())
}

const RESIZE_FILTER: image::imageops::FilterType = image::imageops::FilterType::Triangle;

fn resize_stretch(img: &image::DynamicImage, screen_w: u32, screen_h: u32) -> RgbaImage {
    img.resize_exact(screen_w, screen_h, RESIZE_FILTER)
        .to_rgba8()
}

fn resize_letterbox(img: &image::DynamicImage, screen_w: u32, screen_h: u32) -> RgbaImage {
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

    full_image
}

fn resize_fill(img: &image::DynamicImage, screen_w: u32, screen_h: u32) -> RgbaImage {
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

    full_image
}
