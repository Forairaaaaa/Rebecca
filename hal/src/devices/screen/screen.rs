use crate::common::Emoji;
use crate::devices::{
    API_REGISTER, ApiRoute,
    screen::{FrameBufferScreen, ScreenSocket},
};
use hyper::{Method, Response, header::CONTENT_TYPE};
use log::{error, info, warn};
use std::io;
use std::sync::Arc;
use tokio::{sync::Notify, task};

/// Start screen service to handle cover screen devices
/// # Arguments
/// * `host` - The host for ZMQ socket to bind to
/// * `shutdown_notify` - A notify clone for shutdown signal
/// # Returns
/// A `task::JoinHandle` that can be used to wait for the screen service to shutdown
pub async fn start_screen_service(
    host: String,
    shutdown_notify: Arc<Notify>,
) -> io::Result<task::JoinHandle<()>> {
    // Create screens
    let screens = FrameBufferScreen::new()?;
    let mut screen_sockets: Vec<ScreenSocket> = Vec::new();

    // Create screen sockets
    for (i, screen) in screens.into_iter().enumerate() {
        let screen_socket =
            ScreenSocket::new(Box::new(screen), format!("screen{}", i), host.clone()).await?;
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
                    Box::pin(async move {
                        Response::builder()
                            .header(CONTENT_TYPE, "application/json; charset=utf-8")
                            .body(screen_info)
                            .unwrap()
                    })
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
            worker.await.unwrap_or_else(|e| {
                error!("await screen worker error: {}", e);
            });
        }

        info!("screen service shutdown complete");
    });

    Ok(handle)
}
