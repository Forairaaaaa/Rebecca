use crate::devices::imu::Imu;
use log::{debug, error, warn};
use prost::Message;
use regex::Regex;
use serde::Serialize;
use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::{sync::Notify, task, time};
use zeromq::{Socket, SocketSend};

// Protobuf message definition for IMU data
#[derive(Clone, PartialEq, Message)]
pub struct ImuDataProto {
    #[prost(float, repeated, tag = "1")]
    pub accel: Vec<f32>,
    #[prost(float, repeated, tag = "2")]
    pub gyro: Vec<f32>,
    #[prost(float, repeated, tag = "3")]
    pub mag: Vec<f32>,
    #[prost(float, tag = "4")]
    pub temp: f32,
    #[prost(uint64, tag = "5")]
    pub timestamp: u64,
}

pub struct ImuSocket {
    pub id: String,
    imu: Arc<dyn Imu + Send + Sync>,
    imu_data_port: u16,
    imu_data_socket: Arc<tokio::sync::Mutex<zeromq::PubSocket>>,
    is_running: Arc<AtomicBool>,
    update_task_handle: Arc<tokio::sync::Mutex<Option<task::JoinHandle<()>>>>,
    shutdown_notify: Arc<Notify>,
}

#[derive(Serialize, Debug)]
struct ImuSocketInfo {
    imu_data_port: u16,
    device_type: String,
    description: String,
}

impl ImuSocket {
    pub async fn new(imu: Box<dyn Imu + Send + Sync>, id: String) -> io::Result<Self> {
        debug!("creating imu socket for device: {}", id);

        // Create ZMQ PUB socket
        let mut imu_data_socket = zeromq::PubSocket::new();
        let ep = imu_data_socket
            .bind("tcp://127.0.0.1:0")
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("zmq bind failed: {}", e)))?
            .to_string();

        // Extract port from endpoint
        let imu_data_port = Regex::new(r":(\d+)$")
            .ok()
            .and_then(|re| re.captures(&ep))
            .and_then(|caps| caps.get(1))
            .and_then(|port_str| port_str.as_str().parse().ok())
            .unwrap_or_else(|| {
                error!("parse port from '{}' failed", ep);
                0
            });

        debug!("imu socket bound to port: {}", imu_data_port);

        Ok(Self {
            id,
            imu: Arc::from(imu),
            imu_data_port,
            imu_data_socket: Arc::new(tokio::sync::Mutex::new(imu_data_socket)),
            is_running: Arc::new(AtomicBool::new(false)),
            update_task_handle: Arc::new(tokio::sync::Mutex::new(None)),
            shutdown_notify: Arc::new(Notify::new()),
        })
    }

    pub async fn start(&self) -> io::Result<()> {
        if self.is_running.load(Ordering::Acquire) {
            warn!("imu socket {} is already running", self.id);
            return Ok(());
        }

        debug!("starting imu socket: {}", self.id);

        // Start the IMU device
        self.imu.init()?;

        // Set running flag
        self.is_running.store(true, Ordering::Release);

        // Start update task
        let mut task_handle = self.update_task_handle.lock().await;
        let handle = self.start_update_task().await;
        *task_handle = Some(handle);

        debug!("imu socket {} started successfully", self.id);
        Ok(())
    }

    pub async fn stop(&self) -> io::Result<()> {
        if !self.is_running.load(Ordering::Acquire) {
            warn!("imu socket {} is not running", self.id);
            return Ok(());
        }

        debug!("stopping imu socket: {}", self.id);

        // Set running flag to false
        self.is_running.store(false, Ordering::Release);

        // Notify shutdown
        self.shutdown_notify.notify_waiters();

        // Wait for update task to finish
        let mut task_handle = self.update_task_handle.lock().await;
        if let Some(handle) = task_handle.take() {
            handle.await.map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("task join failed: {}", e))
            })?;
        }

        // Stop the IMU device
        self.imu.deinit()?;

        debug!("imu socket {} stopped successfully", self.id);
        Ok(())
    }

    async fn start_update_task(&self) -> task::JoinHandle<()> {
        let imu = self.imu.clone();
        let socket = self.imu_data_socket.clone();
        let is_running = self.is_running.clone();
        let shutdown_notify = self.shutdown_notify.clone();
        let id = self.id.clone();

        task::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(33)); // ~30Hz (1000ms/30 ≈ 33ms)

            debug!("imu update task started for: {}", id);

            loop {
                tokio::select! {
                    _ = shutdown_notify.notified() => {
                        debug!("imu update task shutdown for: {}", id);
                        break;
                    }
                    _ = interval.tick() => {
                        if !is_running.load(Ordering::Acquire) {
                            break;
                        }

                        // Read IMU data
                        let imu_data = imu.imu_data();
                        // debug!("{} get imu data: {:?}", id, imu_data);

                        // Convert to protobuf message
                        let proto_msg = ImuDataProto {
                            accel: imu_data.accel.to_vec(),
                            gyro: imu_data.gyro.to_vec(),
                            mag: imu_data.mag.to_vec(),
                            temp: imu_data.temp,
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_micros() as u64,
                        };

                        // Serialize to bytes
                        let mut buf = Vec::new();
                        if let Err(e) = proto_msg.encode(&mut buf) {
                            error!("failed to encode imu data: {}", e);
                            continue;
                        }

                        // Send via ZMQ
                        let mut socket_guard = socket.lock().await;
                        if let Err(e) = socket_guard.send(buf.into()).await {
                            error!("failed to send imu data: {}", e);
                        }
                    }
                }
            }

            debug!("imu update task finished for: {}", id);
        })
    }

    pub fn get_device_info(&self) -> String {
        let imu_socket_info = ImuSocketInfo {
            imu_data_port: self.imu_data_port,
            device_type: self.imu.name(),
            description: "Subscribe IMU data from <imu_data_port> via ZMQ SUB socket. Data is published in protobuf format at 30Hz.".to_string(),
        };

        serde_json::to_string_pretty(&imu_socket_info).unwrap_or("wtf?🤡".to_string())
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Acquire)
    }
}
