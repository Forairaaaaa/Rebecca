use log::{debug, error, info};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::io;
use zeromq::{Socket, SocketRecv, SubSocket};

include!(concat!(env!("OUT_DIR"), "/_.rs"));

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
    socket: SubSocket,
}

impl ImuSocket {
    pub async fn new(device_id: &str, host: &str, port: u16) -> io::Result<Self> {
        info!("create imu socket: {device_id}");

        let device_info = get_device_info(device_id, host, port).await?;

        let socket = create_socket(host, device_info.imu_data_port).await?;

        Ok(Self { socket })
    }

    pub async fn listen(&mut self) -> Option<()> {
        loop {
            match self.socket.recv().await {
                Ok(msg) => {
                    if let Some(data) = msg.get(0) {
                        if let Ok(imu_data) = ImuDataProto::decode(data.clone()) {
                            debug!("get imu data: {:#?}", imu_data);

                            // Output in json to stdout
                            if let Ok(json) = serde_json::to_string(&imu_data) {
                                println!("{}", json);
                            } else {
                                error!("imu data to json failed");
                            }
                        } else {
                            error!("imu data decode failed");
                        }
                    } else {
                        error!("ZMQ get msg error");
                    };
                }
                Err(e) => {
                    error!("ZMQ recv error: {:?}", e);
                }
            }
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
    info!("create zmq socket: {host}:{port}");

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
