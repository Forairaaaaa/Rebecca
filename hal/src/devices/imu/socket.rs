use crate::common::Emoji;
use crate::devices::imu::{Imu, ImuData};
use ahrs::{Ahrs, Madgwick};
use log::{debug, error, info, warn};
use nalgebra::Vector3;
use prost::Message;
use regex::Regex;
use serde::Serialize;
use std::f64;
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
    #[prost(uint64, tag = "1")]
    pub timestamp: u64,
    #[prost(float, repeated, tag = "2")]
    pub accel: Vec<f32>,
    #[prost(float, repeated, tag = "3")]
    pub gyro: Vec<f32>,
    #[prost(float, repeated, tag = "4")]
    pub mag: Vec<f32>,
    #[prost(float, tag = "5")]
    pub temp: f32,
    #[prost(float, repeated, tag = "6")]
    pub quaternion: Vec<f32>,
    #[prost(float, repeated, tag = "7")]
    pub euler_angles: Vec<f32>,
}

// Human-readable protobuf schema to be exposed via schema API
const IMU_DATA_PROTO_SCHEMA: &str = r#"syntax = "proto3";

message ImuDataProto {
  uint64        timestamp = 1; // microseconds since UNIX_EPOCH
  repeated float accel = 2;   // ax, ay, az
  repeated float gyro  = 3;   // gx, gy, gz
  repeated float mag   = 4;   // mx, my, mz
  float         temp   = 5;   // milli-degree Celsius
  repeated float quaternion = 6; // quaternion (4 floats)
  repeated float euler_angles = 7; // yaw, pitch, roll (radians)
}
"#;

// On imu data callback for data's final adjustment before publishing
pub type OnImuData = Arc<dyn Fn(&mut ImuData) + Send + Sync>;

pub struct ImuSocket {
    pub id: String,
    imu: Arc<dyn Imu + Send + Sync>,
    imu_data_port: u16,
    imu_data_socket: Arc<tokio::sync::Mutex<zeromq::PubSocket>>,
    is_running: Arc<AtomicBool>,
    update_task_handle: Arc<tokio::sync::Mutex<Option<task::JoinHandle<()>>>>,
    shutdown_notify: Arc<Notify>,
    on_imu_data: OnImuData,
}

#[derive(Serialize, Debug)]
struct ImuSocketInfo {
    device_type: String,
    status: String,
    sample_rate: u32,
    imu_data_port: u16,
    description: String,
}

impl ImuSocket {
    pub async fn new(
        imu: Box<dyn Imu + Send + Sync>,
        id: String,
        host: &str,
        on_imu_data: OnImuData,
    ) -> io::Result<Self> {
        debug!(target: &id, "creating imu socket");

        // Create ZMQ PUB socket
        let mut imu_data_socket = zeromq::PubSocket::new();
        let ep = imu_data_socket
            .bind(format!("tcp://{}:0", host).as_str())
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

        debug!(target: &id, "imu data socket bound to: tcp://{}:{}", host, imu_data_port);

        Ok(Self {
            id,
            imu: Arc::from(imu),
            imu_data_port,
            imu_data_socket: Arc::new(tokio::sync::Mutex::new(imu_data_socket)),
            is_running: Arc::new(AtomicBool::new(false)),
            update_task_handle: Arc::new(tokio::sync::Mutex::new(None)),
            shutdown_notify: Arc::new(Notify::new()),
            on_imu_data,
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
        let on_imu_data = self.on_imu_data.clone();

        task::spawn(async move {
            let sample_rate = imu.sample_rate();
            let interval_ms = 1000 / sample_rate as u64;
            let interval_s = 1.0 / sample_rate as f64;
            let mut interval = time::interval(Duration::from_millis(interval_ms));

            // Orientation estimation using Madgwick filter
            let mut ahrs = Madgwick::new(interval_s, 0.1);
            let mut update_orientation = |imu_data: &ImuData| -> ([f32; 4], [f32; 3]) {
                // radians/s
                let gyroscope = Vector3::new(
                    imu_data.gyro[2] as f64,
                    imu_data.gyro[1] as f64,
                    imu_data.gyro[0] as f64,
                );
                let accelerometer = Vector3::new(
                    imu_data.accel[2] as f64,
                    imu_data.accel[1] as f64,
                    imu_data.accel[0] as f64,
                );

                let quat = ahrs.update_imu(&gyroscope, &accelerometer).unwrap();
                let (roll, pitch, yaw) = quat.euler_angles();

                let quaternion = [
                    quat.coords[0] as f32,
                    quat.coords[1] as f32,
                    quat.coords[2] as f32,
                    quat.coords[3] as f32,
                ];
                let euler_angles = [yaw as f32, pitch as f32, roll as f32];

                (quaternion, euler_angles)
            };

            debug!("imu update task started for: {} in {}Hz", id, sample_rate);

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
                        let mut imu_data = imu.imu_data();

                        // Update orientation
                        let (quaternion, euler_angles) = update_orientation(&mut imu_data);
                        imu_data.quaternion = quaternion;
                        imu_data.euler_angles = euler_angles;

                        // Invoke on_imu_data callback for user's final adjustment
                        on_imu_data(&mut imu_data);

                        debug!("{} get imu data: {:#?}", id, imu_data);

                        // Convert to protobuf message
                        let proto_msg = ImuDataProto {
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_micros() as u64,
                            accel: imu_data.accel.to_vec(),
                            gyro: imu_data.gyro.to_vec(),
                            mag: imu_data.mag.to_vec(),
                            temp: imu_data.temp,
                            quaternion: imu_data.quaternion.to_vec(),
                            euler_angles: imu_data.euler_angles.to_vec(),
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
            device_type: self.imu.name(),
            status: if self.is_running() { "running" } else { "idle" }.to_string(),
            sample_rate: self.imu.sample_rate(),
            imu_data_port: self.imu_data_port,
            description: format!(
                "{} Subscribe to IMU data from <imu_data_port> using a ZMQ SUB socket. The data is published in Protobuf format, and its schema is available at /{}/schema.",
                Emoji::SUBSCRIBE,
                self.id
            ),
        };

        serde_json::to_string_pretty(&imu_socket_info).unwrap_or("wtf?🤡".to_string())
    }

    pub fn get_schema(&self) -> String {
        IMU_DATA_PROTO_SCHEMA.to_string()
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Acquire)
    }
}
