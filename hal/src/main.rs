mod devices;
mod server;

use clap::Parser;
use devices::start_screen_service;
use env_logger::Env;
use log::{error, info};
use std::sync::Arc;
use tokio::{signal, sync::Notify, task::JoinHandle};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to listen on
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

    // Create shutdown notify
    let shutdown_notify = Arc::new(Notify::new());

    let mut tasks: Vec<JoinHandle<()>> = Vec::new();

    // Start screen service
    match start_screen_service(shutdown_notify.clone()).await {
        Ok(screen_handle) => tasks.push(screen_handle),
        Err(e) => error!("failed to start screen service: {}", e),
    }

    // Start HTTP server
    tasks.push(server::start_server(args.port, shutdown_notify.clone()));

    // Wait for signal
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("received SIGINT signal");
        }
        _ = sigterm.recv() => {
            info!("received SIGTERM signal");
        }
    }

    // Notify shutdown
    shutdown_notify.notify_waiters();

    // Wait for all tasks to finish
    for task in tasks {
        task.await?;
    }

    info!("shutdown complete.");
    Ok(())
}
