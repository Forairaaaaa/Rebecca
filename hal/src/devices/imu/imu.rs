use crate::common::Emoji;
use crate::devices::imu::{ImuFromIio, socket::ImuSocket, types::ImuData};
use crate::devices::{API_REGISTER, ApiRoute};
use hyper::{Method, Response, StatusCode, header::CONTENT_TYPE};
use log::{info, warn};
use std::io;
use std::sync::Arc;
use tokio::{sync::Notify, task};

// 批量克隆
macro_rules! arc_clones {
    ($arc_var:ident, $($name:ident),*) => {
        $( let $name = Arc::clone(&$arc_var); )*
    };
}

// 注册设备
async fn register_device(imu_socket: &Arc<ImuSocket>) {
    arc_clones!(
        imu_socket,
        imu_socket_clone1,
        imu_socket_clone2,
        imu_socket_clone3,
        imu_socket_clone4
    );

    let success_response = || -> Response<String> {
        Response::builder()
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body("ok👍".to_string())
            .unwrap()
    };
    let error_response = |e: io::Error| {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(e.to_string())
            .unwrap()
    };

    // Add device to device list
    API_REGISTER.add_device(imu_socket.id.clone()).await;

    // Get info
    if let Err(e) = API_REGISTER
        .add_api(
            ApiRoute {
                path: format!("/{}/info", imu_socket.id),
                method: Method::GET,
                description: format!("{} Get device info.", Emoji::INFO),
            },
            Box::new(move |_request| {
                let imu_socket = Arc::clone(&imu_socket_clone1);
                Box::pin(async move {
                    Response::builder()
                        .header(CONTENT_TYPE, "application/json; charset=utf-8")
                        .body(imu_socket.get_device_info())
                        .unwrap()
                })
            }),
        )
        .await
    {
        warn!("add api failed: {}", e);
    }

    // Get protobuf schema (text/plain)
    if let Err(e) = API_REGISTER
        .add_api(
            ApiRoute {
                path: format!("/{}/schema", imu_socket.id),
                method: Method::GET,
                description: format!("{} Get IMU data protobuf schema.", Emoji::FORMAT),
            },
            Box::new(move |_request| {
                let imu_socket = Arc::clone(&imu_socket_clone4);
                Box::pin(async move {
                    Response::builder()
                        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                        .body(imu_socket.get_schema())
                        .unwrap()
                })
            }),
        )
        .await
    {
        warn!("add api failed: {}", e);
    }

    // Start publishing data
    if let Err(e) = API_REGISTER
        .add_api(
            ApiRoute {
                path: format!("/{}/start", imu_socket.id),
                method: Method::GET,
                description: format!("{} Start publishing data.", Emoji::START),
            },
            Box::new(move |_request| {
                let imu_socket = Arc::clone(&imu_socket_clone2);
                Box::pin(async move {
                    match imu_socket.start().await {
                        Ok(_) => success_response(),
                        Err(e) => error_response(e),
                    }
                })
            }),
        )
        .await
    {
        warn!("add api failed: {}", e);
    }

    // Stop publishing data
    if let Err(e) = API_REGISTER
        .add_api(
            ApiRoute {
                path: format!("/{}/stop", imu_socket.id),
                method: Method::GET,
                description: format!("{} Stop publishing data.", Emoji::STOP),
            },
            Box::new(move |_request| {
                let imu_socket = Arc::clone(&imu_socket_clone3);
                Box::pin(async move {
                    match imu_socket.stop().await {
                        Ok(_) => success_response(),
                        Err(e) => error_response(e),
                    }
                })
            }),
        )
        .await
    {
        warn!("add api failed: {}", e);
    }
}

fn on_imu_data(_imu_data: &mut ImuData) {
    // Not adjustment needed
}

/// Start IMU service
/// # Arguments
/// * `host` - The host for ZMQ socket to bind to
/// * `shutdown_notify` - A notify clone for shutdown signal
/// # Returns
/// A `task::JoinHandle` that can be used to wait for the service to shutdown
pub async fn start_imu_service(
    host: String,
    shutdown_notify: Arc<Notify>,
) -> io::Result<task::JoinHandle<()>> {
    // Try to create IMU from IIO
    let mpu6500_iio = ImuFromIio::new("mpu6500".to_string()).ok_or(io::Error::new(
        io::ErrorKind::Other,
        "failed to create imu from iio by name: mpu6500",
    ))?;

    // Create IMU socket
    let imu_socket = Arc::new(
        ImuSocket::new(
            Box::new(mpu6500_iio),
            "imu0".to_string(),
            host,
            Arc::new(on_imu_data),
        )
        .await?,
    );
    arc_clones!(imu_socket, imu_socket_task);

    register_device(&imu_socket).await;

    let handle = task::spawn(async move {
        let imu_socket = Arc::clone(&imu_socket_task);
        tokio::select! {
            _ = shutdown_notify.notified() => {
                info!("imu service shutdown...");
                if let Err(e) = imu_socket.stop().await {
                    warn!("failed to stop imu socket: {}", e);
                }
            }
        }

        info!("imu service shutdown complete");
    });

    Ok(handle)
}
