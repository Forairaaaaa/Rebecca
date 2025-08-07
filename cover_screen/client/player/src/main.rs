mod cover_screen;
mod player;

use clap::Parser;
use cover_screen::SocketCoverScreen;
use log::debug;
use player::{ColorBar, Downloader, GifPlayer, ImageRenderer, ResizeMode};
use std::{error::Error, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the cover screen, e.g. screen0 for /tmp/cover_screen/screen0.json
    screen: String,

    /// Set if the resource path is a URL
    #[arg(short, long)]
    url: bool,

    /// Target resource path, e.g. ~/wtf.png, if not provided, draw color bar
    #[arg(default_value = None)]
    resource: Option<PathBuf>,

    /// Render resize mode
    #[arg(short, long, value_enum, default_value_t = ResizeMode::Fill)]
    resize_mode: ResizeMode,

    /// Play in loop
    #[arg(short, long, default_value_t = true)]
    in_loop: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = Args::parse();
    debug!("get args: {:#?}", args);

    let mut screen = SocketCoverScreen::new(&args.screen).await?;

    if let Some(mut resource) = args.resource {
        if args.url {
            let (path, content_type) = Downloader::from_url(resource.to_str().unwrap()).await?;
            resource = path;
        }

        ImageRenderer::from_file(&mut screen, resource, args.resize_mode).await?;

        // GifPlayer::from_file(&mut screen, resource, args.resize_mode, args.in_loop).await?;

        Downloader::cleanup()?;
    } else {
        ColorBar::draw(&mut screen).await?;
    }

    Ok(())
}
