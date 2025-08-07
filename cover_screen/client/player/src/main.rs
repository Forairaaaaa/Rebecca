mod cover_screen;
mod player;

use clap::Parser;
use cover_screen::SocketCoverScreen;
use log::{debug, error};
use player::{ColorBar, Downloader, GifPlayer, ImageRenderer, ResizeMode};
use std::{error::Error, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the cover screen, e.g. screen0 for /tmp/cover_screen/screen0.json
    screen: String,

    /// Set if the target resource is a URL
    #[arg(short, long)]
    url: bool,

    /// Render resize mode
    #[arg(short, long, value_enum, default_value_t = ResizeMode::Fill)]
    resize_mode: ResizeMode,

    /// Play in loop
    #[arg(short, long, default_value_t = true)]
    repeat: bool,

    /// Is target resource a video
    #[arg(short, long, default_value_t = false)]
    video: bool,

    /// Target resource path, e.g. ~/wtf.png, if not provided, draw color bar
    #[arg(default_value = None)]
    resource: Option<PathBuf>,
}

const IMAGE_EXTS: [&str; 6] = ["jpg", "jpeg", "png", "webp", "bmp", "tiff"];
const GIF_EXT: &str = "gif";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = Args::parse();
    debug!("get args: {:#?}", args);

    let mut screen = SocketCoverScreen::new(&args.screen).await?;

    // If resource is provided
    if let Some(mut resource) = args.resource {
        let resource_ext: String;

        if args.url {
            let (path, ext) = Downloader::from_url(resource.to_str().unwrap()).await?;
            resource = path;
            resource_ext = ext;
        } else {
            resource_ext = resource
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default()
                .to_string();
        }

        // Map renderer
        match resource_ext.as_str() {
            ext if IMAGE_EXTS.contains(&ext) => {
                ImageRenderer::from_file(&mut screen, resource, args.resize_mode).await?;
            }
            ext if ext == GIF_EXT => {
                GifPlayer::from_file(&mut screen, resource, args.resize_mode, args.repeat).await?;
            }
            _ => {
                error!("unsupported extension: {}", resource.display());
            }
        }

        Downloader::cleanup()?;
    }
    // If not, draw color bar
    else {
        ColorBar::draw(&mut screen).await?;
    }

    Ok(())
}
