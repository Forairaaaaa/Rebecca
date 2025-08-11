use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::io;
use zeromq::{Socket, SubSocket};

/// List all available IMUs
pub async fn list_imu(host: &str, port: u16) -> io::Result<Vec<String>> {
    let client = reqwest::Client::new();
    let url = format!("http://{host}:{port}/devices");

    let response =
        client.get(&url).send().await.map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("HTTP request failed: {}", e))
        })?;

    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("HTTP request failed with status: {}", response.status()),
        ));
    }

    let mut devices: Vec<String> = response.json().await.map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to parse JSON: {}", e))
    })?;

    devices.retain(|id| id.contains("imu"));

    Ok(devices)
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceInfo {
    device_type: String,
    status: String,
    sample_rate: u32,
    imu_data_port: u16,
    description: String,
}

pub struct ImuSocket {
    device_info: DeviceInfo,
    socket: SubSocket,
}

impl ImuSocket {
    pub async fn new(device_id: &str, host: &str, port: u16) -> io::Result<Self> {
        info!("create imu socket: {device_id}");

        let device_info = get_device_info(device_id, host, port).await?;

        let socket = create_socket(host, device_info.imu_data_port).await?;

        Ok(Self {
            device_info,
            socket,
        })
    }

    pub async fn listen(&mut self) -> Option<()> {
        loop {
            // ...
        }
    }
}

async fn get_device_info(device_id: &str, host: &str, port: u16) -> io::Result<DeviceInfo> {
    let client = reqwest::Client::new();
    let url = format!("http://{host}:{port}/{device_id}/info");

    let response =
        client.get(&url).send().await.map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("HTTP request failed: {}", e))
        })?;

    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("HTTP request failed with status: {}", response.status()),
        ));
    }

    let device_info: DeviceInfo = response.json().await.map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to parse JSON: {}", e))
    })?;

    info!("get device info: {:#?}", device_info);

    Ok(device_info)
}

async fn create_socket(host: &str, port: u16) -> io::Result<SubSocket> {
    let mut socket = SubSocket::new();
    let endpoint = format!("tcp://{}:{}", host, port);
    socket
        .connect(&endpoint)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("ZMQ connect failed: {}", e)))?;
    socket.subscribe("").await.map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("ZMQ subscribe failed: {}", e))
    })?;
    Ok(socket)
}
