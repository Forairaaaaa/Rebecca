use crate::devices::{
    API_REGISTER, ApiRoute,
    imu::{ImuFromIio, socket::ImuSocket},
};
use hyper::{Method, Response};
use log::{info, warn};
use std::io;
use std::sync::Arc;
use tokio::{sync::Notify, task};

pub async fn start_imu_service(shutdown_notify: Arc<Notify>) -> io::Result<task::JoinHandle<()>> {
    // Try to create IMU from IIO
    let mpu6500_iio = ImuFromIio::new("mpu6500".to_string()).ok_or(io::Error::new(
        io::ErrorKind::Other,
        "failed to create imu from iio by name: mpu6500",
    ))?;

    // Create IMU socket
    let imu_socket = Arc::new(ImuSocket::new(Box::new(mpu6500_iio), "imu0".to_string()).await?);

    let imu_socket_clone = Arc::clone(&imu_socket);
    let imu_socket_clone2 = Arc::clone(&imu_socket);

    // Register get info api
    match API_REGISTER
        .add_api(
            ApiRoute {
                path: format!("/{}/info", imu_socket.id),
                method: Method::GET,
                description: "Get device info".to_string(),
            },
            Box::new(move |_request| {
                let imu_socket = Arc::clone(&imu_socket_clone);
                Box::pin(async move { Response::new(imu_socket.get_device_info()) })
            }),
        )
        .await
    {
        Ok(_) => {}
        Err(e) => {
            warn!("add api failed: {}", e);
        }
    }

    // Start the IMU socket
    // imu_socket.start().await?; // Stop by default

    let handle = task::spawn(async move {
        let imu_socket = Arc::clone(&imu_socket_clone2);
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
