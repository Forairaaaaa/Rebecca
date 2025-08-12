use crate::common::Emoji;
use crate::devices::imu::{IioImu, Imu, ImuData, MockImu, socket::ImuSocket};
use crate::devices::{API_REGISTER, ApiRoute};
use hyper::{Method, Response, StatusCode, header::CONTENT_TYPE};
use log::{error, info, warn};
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

fn add_custom_imus(imus: &mut Vec<Box<dyn Imu + Send + Sync + 'static>>) {
    // Try to create mpu6500 from iio
    if let Some(mpu6500_iio) = IioImu::new("mpu6500") {
        imus.push(Box::new(mpu6500_iio));
    } else {
        error!("failed to create imu from iio by name: mpu6500");
        return;
    }
}

/// Start IMU service
/// # Arguments
/// * `host` - The host for ZMQ socket to bind to
/// * `shutdown_notify` - A notify clone for shutdown signal
/// * `mock_imu` - Whether to create mock IMU for api test
/// # Returns
/// A `task::JoinHandle` that can be used to wait for the service to shutdown
pub async fn start_imu_service(
    host: &str,
    shutdown_notify: Arc<Notify>,
    mock_imu: bool,
) -> io::Result<task::JoinHandle<()>> {
    let mut imus: Vec<Box<dyn Imu + Send + Sync + 'static>> = Vec::new();

    if mock_imu {
        info!("create mock imu");
        imus.push(Box::new(MockImu::new()));
    }

    add_custom_imus(&mut imus);

    // Create imu sockets
    let mut imu_sockets: Vec<Arc<ImuSocket>> = Vec::new();
    for (i, imu) in imus.into_iter().enumerate() {
        let imu_socket =
            ImuSocket::new(imu, format!("imu{}", i), host, Arc::new(on_imu_data)).await?;

        let imu_socket = Arc::new(imu_socket);

        register_device(&imu_socket).await;

        imu_sockets.push(imu_socket);
    }

    // Start imu service
    let handle = task::spawn(async move {
        let mut workers = Vec::new();

        for imu_socket in imu_sockets {
            let notify = shutdown_notify.clone();

            let worker = task::spawn(async move {
                let imu_socket = Arc::clone(&imu_socket);
                tokio::select! {
                    _ = notify.notified() => {
                        info!("imu service shutdown...");
                        if let Err(e) = imu_socket.stop().await {
                            warn!("failed to stop imu socket: {}", e);
                        }
                    }
                }
                info!("imu service shutdown complete");
            });

            workers.push(worker);
        }

        for worker in workers {
            worker.await.unwrap_or_else(|e| {
                error!("await imu worker error: {}", e);
            });
        }

        info!("imu service shutdown complete");
    });

    Ok(handle)
}
