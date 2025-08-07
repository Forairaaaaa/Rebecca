// https://en.wikipedia.org/wiki/SMPTE_color_bars

use crate::cover_screen::CoverScreen;
use crate::player::BppConverter;
use log::info;
use std::error::Error;

pub struct ColorBar {}

impl ColorBar {
    /// Draw color bar
    /// # Arguments
    /// * `cover_screen` - The cover screen to draw the color bar to
    /// # Returns
    /// * `Result<(), Box<dyn Error>>` - The result of the operation
    pub async fn draw(cover_screen: &mut impl CoverScreen) -> Result<(), Box<dyn Error>> {
        info!("draw color bar");

        let width = cover_screen.width();
        let height = cover_screen.height();
        let target_bpp = cover_screen.bpp();

        // 先构造自己的 frame buffer，使用 RGB888 (24 bpp) 格式
        let src_bpp = 24u8;
        let buffer_size = (width * height * (src_bpp / 8) as u32) as usize;
        let mut my_frame_buffer = vec![0u8; buffer_size];

        // 在自己的 frame buffer 上绘制
        draw_smpte_ecr1978_rgb888(&mut my_frame_buffer, width as usize, height as usize);

        // 检查目标 bpp，如果不匹配就转换
        let final_data = if src_bpp == target_bpp as u8 {
            my_frame_buffer
        } else {
            BppConverter::convert(&my_frame_buffer, width, height, src_bpp, target_bpp as u8)?
        };

        // 将数据复制到目标 frame buffer
        let frame_buffer = cover_screen.frame_buffer();
        frame_buffer[..final_data.len()].copy_from_slice(&final_data);

        cover_screen.push_frame().await?;

        Ok(())
    }
}

fn draw_smpte_ecr1978_rgb888(frame_buffer: &mut [u8], width: usize, height: usize) {
    let top_height = height * 2 / 3;
    let mid_height = height / 12;

    // 第一层
    let top_colors = [
        (192, 192, 192), // White
        (192, 192, 0),   // Yellow
        (0, 192, 192),   // Cyan
        (0, 192, 0),     // Green
        (192, 0, 192),   // Magenta
        (192, 0, 0),     // Red
        (0, 0, 192),     // Blue
    ];

    let bar_width = width / top_colors.len();

    for (i, &(r, g, b)) in top_colors.iter().enumerate() {
        let x_start = i * bar_width;
        let x_end = if i == top_colors.len() - 1 {
            width
        } else {
            x_start + bar_width
        };

        for y in 0..top_height {
            for x in x_start..x_end {
                let offset = (y * width + x) * 3;
                frame_buffer[offset] = r;
                frame_buffer[offset + 1] = g;
                frame_buffer[offset + 2] = b;
            }
        }
    }

    // 第二层
    let castellation_colors = [
        (0, 0, 255),     // Blue
        (0, 0, 0),       // Black (under Red)
        (255, 0, 255),   // Magenta
        (0, 0, 0),       // Black (under Green)
        (0, 255, 255),   // Cyan
        (0, 0, 0),       // Black (under Yellow)
        (192, 192, 192), // Gray (under White)
    ];

    let castellation_bar_width = width / castellation_colors.len();
    let mid_y_start = top_height;
    let mid_y_end = top_height + mid_height;

    for (i, &(r, g, b)) in castellation_colors.iter().enumerate() {
        let x_start = i * castellation_bar_width;
        let x_end = if i == castellation_colors.len() - 1 {
            width
        } else {
            x_start + castellation_bar_width
        };

        for y in mid_y_start..mid_y_end {
            for x in x_start..x_end {
                let offset = (y * width + x) * 3;
                frame_buffer[offset] = r;
                frame_buffer[offset + 1] = g;
                frame_buffer[offset + 2] = b;
            }
        }
    }

    // 第三层
    let bot_y_start = top_height + mid_height;
    let bot_y_end = height;

    for y in bot_y_start..bot_y_end {
        for x in 0..width {
            let (r, g, b) = get_bottom_bar_color(x, width);
            let offset = (y * width + x) * 3;
            frame_buffer[offset] = r;
            frame_buffer[offset + 1] = g;
            frame_buffer[offset + 2] = b;
        }
    }
}

fn get_bottom_bar_color(x: usize, width: usize) -> (u8, u8, u8) {
    let block_width = width / 6;
    let block = x / block_width;
    let rel_x = x % block_width;

    match block {
        0 => (0, 33, 76),     // -I (dark blue)
        1 => (192, 192, 192), // White
        2 => (50, 0, 106),    // +Q (dark purple)
        3 => (0, 0, 0),       // Black background + PLUGE
        4 | 5 | 6 => {
            // Inside PLUGE area
            let third = block_width / 3;
            match rel_x {
                r if r < third => (9, 9, 9),        // PLUGE 1: below black
                r if r < 2 * third => (19, 19, 19), // PLUGE 2: black
                _ => (29, 29, 29),                  // PLUGE 3: above black
            }
        }
        _ => (0, 0, 0), // Fallback black
    }
}
