use crate::common::Emoji;
use crate::devices::{
    API_REGISTER, ApiRoute,
    screen::{FrameBufferScreen, MockScreen, Screen, ScreenSocket},
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
/// * `mock_screen` - Whether to create mock screen for api test
/// # Returns
/// A `task::JoinHandle` that can be used to wait for the screen service to shutdown
pub async fn start_screen_service(
    host: &str,
    shutdown_notify: Arc<Notify>,
    mock_screen: bool,
) -> io::Result<task::JoinHandle<()>> {
    let mut screens: Vec<Box<dyn Screen + Send + Sync + 'static>> = Vec::new();

    // Create fb screens
    let fb_screens = FrameBufferScreen::new()?;
    screens.extend(
        fb_screens
            .into_iter()
            .map(|screen| Box::new(screen) as Box<dyn Screen + Send + Sync + 'static>),
    );

    // Create mock screens
    if mock_screen {
        info!("create mock screens");
        screens.push(Box::new(MockScreen::new(320, 240, 16)));
        screens.push(Box::new(MockScreen::new(320, 240, 16)));
    }

    // Create screen sockets
    let mut screen_sockets: Vec<ScreenSocket> = Vec::new();
    for (i, screen) in screens.into_iter().enumerate() {
        let screen_socket = ScreenSocket::new(screen, format!("screen{}", i), host).await?;
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
