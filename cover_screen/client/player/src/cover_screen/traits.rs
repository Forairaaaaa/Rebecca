use std::io;

pub trait CoverScreen {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn bpp(&self) -> u32;
    fn frame_buffer(&mut self) -> &mut Vec<u8>;
    async fn push_frame(&mut self) -> io::Result<()>;
}
