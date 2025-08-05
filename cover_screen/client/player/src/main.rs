mod cover_screen;
mod player;

use clap::Parser;
use cover_screen::SocketCoverScreen;
use log::debug;
use player::color_bar::draw_color_bar;
use player::image::{ResizeMode, draw_image};
use std::{error::Error, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the cover screen, e.g. screen0 for /tmp/cover_screen/screen0.json
    screen: String,

    /// Target resource path, e.g. ~/wtf.png, if not provided, draw color bar
    #[arg(default_value = None)]
    resource: Option<PathBuf>,

    /// Render resize mode
    #[arg(short, long, value_enum, default_value_t = ResizeMode::Letterbox)]
    resize_mode: ResizeMode,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = Args::parse();
    debug!("get args: {:#?}", args);

    let mut screen = SocketCoverScreen::new(&args.screen).await?;

    if let Some(resource) = args.resource {
        draw_image(&mut screen, resource, args.resize_mode).await?;
    } else {
        draw_color_bar(&mut screen).await?;
    }

    Ok(())
}
