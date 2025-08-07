use log::debug;

pub struct BppConverter {}

impl BppConverter {
    /// Convert bpp of image data
    /// # Arguments
    /// * `data` - The image data to convert
    /// * `width` - The width of the image
    /// * `height` - The height of the image
    /// * `src_bpp` - The source bpp of the image
    /// * `dst_bpp` - The destination bpp of the image
    pub fn convert(
        data: &[u8],
        width: u32,
        height: u32,
        src_bpp: u8,
        dst_bpp: u8,
    ) -> Result<Vec<u8>, &'static str> {
        // 检查 bpp 是否支持
        let src_bytes_per_pixel = src_bpp / 8;
        let dst_bytes_per_pixel = dst_bpp / 8;

        if src_bytes_per_pixel == dst_bytes_per_pixel {
            // 如果 bpp 相同，直接返回数据副本
            debug!("src_bpp == dst_bpp, return data");
            return Ok(data.to_vec());
        }

        let mut new_data =
            Vec::with_capacity((width * height * dst_bytes_per_pixel as u32) as usize);

        debug!("convert_bpp: src_bpp: {}, dst_bpp: {}", src_bpp, dst_bpp);

        match (src_bpp, dst_bpp) {
            (24, 32) => {
                // 从 24-bit RGB 转换为 32-bit RGBA
                debug!("convert 24-bit RGB to 32-bit RGBA");
                for i in (0..data.len()).step_by(src_bytes_per_pixel as usize) {
                    new_data.push(data[i]); // R
                    new_data.push(data[i + 1]); // G
                    new_data.push(data[i + 2]); // B
                    new_data.push(255); // A (设置为不透明)
                }
            }
            (32, 24) => {
                // 从 32-bit RGBA 转换为 24-bit RGB
                debug!("convert 32-bit RGBA to 24-bit RGB");
                for i in (0..data.len()).step_by(src_bytes_per_pixel as usize) {
                    new_data.push(data[i]); // R
                    new_data.push(data[i + 1]); // G
                    new_data.push(data[i + 2]); // B
                    // 忽略 data[i + 3] (Alpha 通道)
                }
            }
            (24, 16) => {
                // 从 24-bit RGB 转换为 16-bit RGB565
                debug!("convert 24-bit RGB to 16-bit RGB565");
                for i in (0..data.len()).step_by(src_bytes_per_pixel as usize) {
                    let r = data[i];
                    let g = data[i + 1];
                    let b = data[i + 2];
                    let rgb565 = rgb888_to_rgb565(r, g, b);
                    let bytes = rgb565.to_le_bytes();
                    new_data.push(bytes[0]);
                    new_data.push(bytes[1]);
                }
            }
            (16, 24) => {
                // 从 16-bit RGB565 转换为 24-bit RGB
                debug!("convert 16-bit RGB565 to 24-bit RGB");
                for i in (0..data.len()).step_by(src_bytes_per_pixel as usize) {
                    let rgb565 = u16::from_le_bytes([data[i], data[i + 1]]);
                    let (r, g, b) = rgb565_to_rgb888(rgb565);
                    new_data.push(r);
                    new_data.push(g);
                    new_data.push(b);
                }
            }
            (32, 16) => {
                // 从 32-bit RGBA 转换为 16-bit RGB565
                debug!("convert 32-bit RGBA to 16-bit RGB565");
                for i in (0..data.len()).step_by(src_bytes_per_pixel as usize) {
                    let r = data[i];
                    let g = data[i + 1];
                    let b = data[i + 2];
                    let rgb565 = rgb888_to_rgb565(r, g, b);
                    let bytes = rgb565.to_le_bytes();
                    new_data.push(bytes[0]);
                    new_data.push(bytes[1]);
                }
            }
            _ => {
                // 不支持的 bpp 转换
                return Err("Unsupported bpp conversion");
            }
        }

        Ok(new_data)
    }
}

fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | (b as u16 >> 3)
}

fn rgb565_to_rgb888(rgb565: u16) -> (u8, u8, u8) {
    let r = ((rgb565 >> 8) & 0xF8) as u8;
    let g = ((rgb565 >> 3) & 0xFC) as u8;
    let b = ((rgb565 << 3) & 0xF8) as u8;
    (r, g, b)
}
