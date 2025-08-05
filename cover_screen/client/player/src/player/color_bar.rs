// https://en.wikipedia.org/wiki/SMPTE_color_bars

use crate::cover_screen::CoverScreen;
use log::info;
use std::error::Error;

pub async fn draw_color_bar(cover_screen: &mut impl CoverScreen) -> Result<(), Box<dyn Error>> {
    info!("draw color bar");

    let width = cover_screen.width();
    let height = cover_screen.height();
    let bpp = cover_screen.bpp();
    let frame_buffer = cover_screen.frame_buffer();

    draw_smpte_ecr1978(frame_buffer, width as usize, height as usize, bpp as usize);

    cover_screen.push_frame().await?;

    Ok(())
}

fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | (b as u16 >> 3)
}

fn draw_smpte_ecr1978(frame_buffer: &mut [u8], width: usize, height: usize, bpp: usize) {
    if bpp != 16 {
        panic!("Only RGB565 supported");
    }

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
        let color565 = rgb888_to_rgb565(r, g, b).to_le_bytes();
        let x_start = i * bar_width;
        let x_end = if i == top_colors.len() - 1 {
            width
        } else {
            x_start + bar_width
        };

        for y in 0..top_height {
            for x in x_start..x_end {
                let offset = (y * width + x) * 2;
                frame_buffer[offset] = color565[0];
                frame_buffer[offset + 1] = color565[1];
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
        let color565 = rgb888_to_rgb565(r, g, b).to_le_bytes();
        let x_start = i * castellation_bar_width;
        let x_end = if i == castellation_colors.len() - 1 {
            width
        } else {
            x_start + castellation_bar_width
        };

        for y in mid_y_start..mid_y_end {
            for x in x_start..x_end {
                let offset = (y * width + x) * 2;
                frame_buffer[offset] = color565[0];
                frame_buffer[offset + 1] = color565[1];
            }
        }
    }

    // 第三层
    let bot_y_start = top_height + mid_height;
    let bot_y_end = height;

    for y in bot_y_start..bot_y_end {
        for x in 0..width {
            let (r, g, b) = get_bottom_bar_color(x, width);
            let color565 = rgb888_to_rgb565(r, g, b).to_le_bytes();
            let offset = (y * width + x) * 2;
            frame_buffer[offset] = color565[0];
            frame_buffer[offset + 1] = color565[1];
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
