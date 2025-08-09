use clap::Parser;
use env_logger::Env;
use log::{error, info};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::signal;

mod server;

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
async fn main() {
    let args = Args::parse();

    // Init logger
    env_logger::Builder::from_env(Env::default().default_filter_or(if args.verbose {
        "debug"
    } else {
        "warn"
    }))
    .init();

    // 创建关闭信号
    let shutdown_signal = Arc::new(AtomicBool::new(false));

    // 启动HTTP服务器
    let server_handle = server::start(args.port, shutdown_signal.clone());

    // 这里可以启动其他线程
    // let other_handle = start_other_service();

    // 等待信号
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("received SIGINT signal");
        }
        _ = sigterm.recv() => {
            info!("received SIGTERM signal");
        }
    }

    // 设置关闭信号
    shutdown_signal.store(true, Ordering::Relaxed);

    // 等待服务器task结束
    if let Err(e) = server_handle.await {
        error!("server task join failed: {:?}", e);
    }

    // 等待其他线程结束
    // if let Err(e) = other_handle.join() {
    //     error!("other thread join failed: {:?}", e);
    // }
}
