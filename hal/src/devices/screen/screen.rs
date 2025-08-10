use crate::common::Emoji;
use crate::devices::{
    API_REGISTER, ApiRoute,
    screen::{FrameBufferScreen, ScreenSocket},
};
use hyper::{Method, Response};
use log::{info, warn};
use std::io;
use std::sync::Arc;
use tokio::{sync::Notify, task};

/// Start screen service to handle cover screen devices
/// # Arguments
/// * `device_infos` - A mutable reference to a vector of `DeviceInfo` for appending new screen devices
/// * `shutdown_notify` - A notify clone for shutdown signal
/// # Returns
/// A `task::JoinHandle` that can be used to wait for the screen service to shutdown
pub async fn start_screen_service(
    shutdown_notify: Arc<Notify>,
) -> io::Result<task::JoinHandle<()>> {
    // Create screens
    let screens = FrameBufferScreen::new()?;
    let mut screen_sockets: Vec<ScreenSocket> = Vec::new();

    // Create screen sockets
    for (i, screen) in screens.into_iter().enumerate() {
        let screen_socket = ScreenSocket::new(Box::new(screen), format!("screen{}", i)).await?;
        let screen_info = screen_socket.get_device_info();

        // Add device to device list
        API_REGISTER.add_device(screen_socket.id.clone()).await;

        // Register get info api
        match API_REGISTER
            .add_api(
                ApiRoute {
                    path: format!("/{}/info", screen_socket.id),
                    method: Method::GET,
                    description: format!("{} Get device info", Emoji::INFO),
                },
                Box::new(move |_request| {
                    let screen_info = screen_info.clone();
                    Box::pin(async move { Response::new(screen_info) })
                }),
            )
            .await
        {
            Ok(_) => {}
            Err(e) => {
                warn!("add api failed: {}", e);
            }
        }

        screen_sockets.push(screen_socket);
    }

    // Start screen service
    let handle = task::spawn(async move {
        let mut workers = Vec::new();

        for mut screen_socket in screen_sockets {
            let notify = shutdown_notify.clone();

            let worker = task::spawn(async move {
                tokio::select! {
                    _ = async {
                        loop {
                            screen_socket.listen().await;
                        }
                    } => {}

                    _ = notify.notified() => {
                        info!("{} shutdown...", screen_socket.id);
                    }
                }
            });

            workers.push(worker);
        }

        for worker in workers {
            worker.await.unwrap();
        }

        info!("screen service shutdown complete");
    });

    Ok(handle)
}
