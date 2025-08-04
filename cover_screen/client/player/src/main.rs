mod cover_screen;

use cover_screen::CoverScreen;
use log::info;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let screen = CoverScreen::new("screen0").await?;

    Ok(())
}
