use crate::cover_screen::CoverScreen;
use log::{debug, info};
use std::io;

pub struct MockCoverScreen {
    width: u32,
    height: u32,
    bpp: u32,
    frame_buffer: Vec<u8>,
}

impl MockCoverScreen {
    pub fn new(width: u32, height: u32, bpp: u32) -> io::Result<Self> {
        // Raylib maybe
        // ...

        let frame_buffer = create_frame_buffer(width, height, bpp);

        Ok(Self {
            width,
            height,
            bpp,
            frame_buffer,
        })
    }
}

fn create_frame_buffer(width: u32, height: u32, bpp: u32) -> Vec<u8> {
    let size_in_bytes = (width * height * bpp / 8) as usize;

    info!(
        "create frame buffer: {} x {} {} bytes",
        width, height, size_in_bytes
    );

    vec![0u8; size_in_bytes]
}
