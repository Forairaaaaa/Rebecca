use crate::devices::{
    DEVICE_MANAGER,
    imu::{Imu, ImuFromIio, socket::ImuSocket},
};
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
    let imu_socket = ImuSocket::new(Box::new(mpu6500_iio), "imu0".to_string()).await?;

    // Add device to manager
    match DEVICE_MANAGER
        .add_device(imu_socket.get_device_info())
        .await
    {
        Ok(_) => {
            info!("imu device added to manager: imu0");
        }
        Err(e) => {
            warn!("failed to add imu device to manager: {}", e);
        }
    }

    // Start the IMU socket
    // imu_socket.start().await?; // Stop by default

    let handle = task::spawn(async move {
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
