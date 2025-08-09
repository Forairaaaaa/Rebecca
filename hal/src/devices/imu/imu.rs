use crate::devices::imu::{Imu, ImuFromIio};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Notify, task};

pub async fn start_imu_service(shutdown_notify: Arc<Notify>) -> io::Result<task::JoinHandle<()>> {
    let mpu6500_iio = ImuFromIio::new("mpu6500".to_string()).ok_or(io::Error::new(
        io::ErrorKind::Other,
        "failed to create imu from iio by name: mpu6500",
    ))?;

    mpu6500_iio.init().unwrap();

    let handle = task::spawn(async move {
        tokio::select! {
            _ = shutdown_notify.notified() => {
                mpu6500_iio.deinit().unwrap();
            }
            _ = async {
                loop {
                    let imu_data = mpu6500_iio.imu_data();
                    println!("imu data: {:?}", imu_data);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            } => {}
        }
    });

    Ok(handle)
}
