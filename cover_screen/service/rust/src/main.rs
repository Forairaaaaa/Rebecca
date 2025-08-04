mod cover_screen;

use cover_screen::{screen::scan_screens, socket::ScreenSocket};
use std::error::Error;
use std::sync::Arc;
use tokio::{signal, sync::Notify, task};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let screens = scan_screens()?;

    let shutdown_notify = Arc::new(Notify::new());
    let mut workers = Vec::new();

    for (i, screen) in screens.iter().enumerate() {
        let screen_name = format!("screen{}", i);
        let mut socket = ScreenSocket::new(&screen, &screen_name, None).await?;
        let notify = shutdown_notify.clone();

        let worker = task::spawn(async move {
            tokio::select! {
                _ = async {
                    loop {
                        socket.listen().await;
                    }
                } => {}

                _ = notify.notified() => {
                    println!("{} shutdown...", screen_name);
                }
            }
        });
        workers.push(worker);
    }

    signal::ctrl_c().await?;
    println!("Received Ctrl+C, shutting down...");

    shutdown_notify.notify_waiters();

    for worker in workers {
        worker.await?;
    }

    println!("Cleaning up...");
    cover_screen::socket::cleanup();

    println!("Shutdown complete.");
    Ok(())
}
