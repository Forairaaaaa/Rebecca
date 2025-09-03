mod backlight;
mod imu;

use clap::{Parser, Subcommand};
use env_logger::Env;
use log::debug;

#[derive(Parser, Debug)]
#[command(version, about, long_about = "Rebecca HAL CLI helper tool")]
struct Args {
    /// Rebecca-HAL server host
    #[arg(long, default_value = "localhost")]
    host: String,

    /// Rebecca-HAL server port
    #[arg(short, long, default_value_t = 12580)]
    port: u16,

    /// Verbose mode
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// IMU related commands
    Imu {
        /// ID of the IMU, e.g. imu0, if not provided, list all available IMUs
        device_id: Option<String>,

        #[command(subcommand)]
        command: Option<imu::ImuCommand>,
    },

    /// Backlight related commands
    Backlight {
        /// ID of the backlight, e.g. backlight0, if not provided, list all available backlights
        device_id: Option<String>,

        #[command(subcommand)]
        command: Option<backlight::BacklightCommand>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Init logger
    env_logger::Builder::from_env(Env::default().default_filter_or(if args.verbose {
        "debug"
    } else {
        "warn"
    }))
    .init();

    debug!("get args: {:#?}", args);

    match args.subcommand {
        SubCommand::Imu { device_id, command } => {
            imu::handle_imu_command(device_id, command, &args.host, args.port).await?;
        }
        SubCommand::Backlight { device_id, command } => {
            backlight::handle_backlight_command(device_id, command, &args.host, args.port).await?;
        }
    }

    Ok(())
}
