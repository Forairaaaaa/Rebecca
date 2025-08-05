mod cover_screen;
mod player;

use cover_screen::SocketCoverScreen;
use player::draw_color_bar;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut screen = SocketCoverScreen::new("screen1").await?;

    draw_color_bar(&mut screen).await?;

    Ok(())
}
