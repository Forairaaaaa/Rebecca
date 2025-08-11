mod imu;

use clap::{Parser, ValueEnum};
use colored::Colorize;
use env_logger::Env;
use log::{debug, error, info};
use tokio::signal;

#[derive(Debug, Clone, ValueEnum)]
enum Command {
    Info,
    Start,
    Stop,
    Read,
}

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

    /// Command to execute
    #[arg(default_value = "info")]
    command: Command,
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

    match args.command {
        Command::Info => {
            let device_info =
                imu::get_device_info(&args.imu.unwrap(), &args.host, args.port).await?;
            println!("{:#?}", device_info);
            return Ok(());
        }
        Command::Start => {
            imu::start_imu_data_publishing(&args.imu.unwrap(), &args.host, args.port).await?;
            return Ok(());
        }
        Command::Stop => {
            imu::stop_imu_data_publishing(&args.imu.unwrap(), &args.host, args.port).await?;
            return Ok(());
        }
        Command::Read => {}
    }

    // Create IMU socket
    let mut imu_socket = imu::ImuSocket::new(&args.imu.unwrap(), &args.host, args.port).await?;

    // Wait for signal
    info!("start to listen imu data");
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
