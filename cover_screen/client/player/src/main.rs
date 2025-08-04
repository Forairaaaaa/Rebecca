mod cover_screen;
mod player;

use cover_screen::CoverScreen;
use player::color_bar::draw_color_bar;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut screen = CoverScreen::new("screen1").await?;

    draw_color_bar(&mut screen).await?;

    Ok(())
}
