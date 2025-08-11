mod imu;

use clap::Parser;
use colored::Colorize;
use env_logger::Env;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::{signal, sync::Notify};

#[derive(Parser, Debug)]
#[command(version, about, long_about = "A bridge to get IMU data easily")]
struct Args {
    /// ID of the IMU, e.g. imu0, if not provided, list all available IMUs
    imu: Option<String>,

    /// Rebecca-HAL server host
    #[arg(long, default_value = "localhost")]
    host: String,

    /// Rebecca-HAL server port
    #[arg(short, long, default_value_t = 12580)]
    port: u16,

    /// Verbose mode
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Init logger
    env_logger::Builder::from_env(Env::default().default_filter_or(if args.verbose {
        "debug"
    } else {
        "info"
    }))
    .init();

    debug!("get args: {:#?}", args);

    // If no IMU provided
    if args.imu.is_none() {
        let imus = imu::list_imu(&args.host, args.port).await?;
        print!("Available IMUs: ");
        for imu in imus {
            print!("{} ", imu.green());
        }
        println!();
        return Ok(());
    }

    // Create IMU socket
    let mut imu_socket = imu::ImuSocket::new(&args.imu.unwrap(), &args.host, args.port).await?;

    // Wait for signal
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("received SIGINT signal");
        }
        _ = sigterm.recv() => {
            info!("received SIGTERM signal");
        }
        _ = imu_socket.listen() => {
            error!("imu socket listen error");
        }
    }

    Ok(())
}
