use clap::Subcommand;
use log::{debug, error, info};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::io;
use tokio::signal;
use zeromq::{Socket, SocketRecv, SubSocket};

include!(concat!(env!("OUT_DIR"), "/_.rs"));

#[derive(Subcommand, Debug)]
pub enum ImuCommand {
    /// Get IMU device information
    Info,
    /// Start IMU data publishing
    Start,
    /// Stop IMU data publishing
    Stop,
    /// Read IMU data
    Read,
}

/// Handle IMU subcommand
pub async fn handle_imu_command(
    device_id: Option<String>,
    command: Option<ImuCommand>,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        // If no subcommand provided, default behavior based on device_id
        None => {
            if device_id.is_none() {
                // rebecca-hal imu - list all IMUs
                let imus = list_imu(host, port).await?;
                let json = serde_json::to_string(&imus)?;
                println!("{}", json);
            } else {
                // rebecca-hal imu imu0 - default to info
                let device_id = device_id.unwrap();
                let device_info = get_device_info(&device_id, host, port).await?;
                let json = serde_json::to_string_pretty(&device_info)?;
                println!("{}", json);
            }
        }
        Some(ImuCommand::Info) => {
            if let Some(device_id) = device_id {
                let device_info = get_device_info(&device_id, host, port).await?;
                let json = serde_json::to_string_pretty(&device_info)?;
                println!("{}", json);
            } else {
                eprintln!("Error: device_id is required for info command");
                std::process::exit(1);
            }
        }
        Some(ImuCommand::Start) => {
            if let Some(device_id) = device_id {
                start_imu_data_publishing(&device_id, host, port).await?;
            } else {
                eprintln!("Error: device_id is required for start command");
                std::process::exit(1);
            }
        }
        Some(ImuCommand::Stop) => {
            if let Some(device_id) = device_id {
                stop_imu_data_publishing(&device_id, host, port).await?;
            } else {
                eprintln!("Error: device_id is required for stop command");
                std::process::exit(1);
            }
        }
        Some(ImuCommand::Read) => {
            if let Some(device_id) = device_id {
                // Create IMU socket for reading data
                let mut imu_socket = ImuSocket::new(&device_id, host, port).await?;

                // Wait for signal
                info!("start to listen imu data");

                tokio::select! {
                    _ = signal::ctrl_c() => {
                        info!("received SIGINT signal");
                    }
                    _ = imu_socket.listen() => {
                        error!("imu socket listen error");
                    }
                }
            } else {
                eprintln!("Error: device_id is required for read command");
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

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

/// Get device info
pub async fn get_device_info(device_id: &str, host: &str, port: u16) -> io::Result<DeviceInfo> {
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

/// Start imu data publishing
pub async fn start_imu_data_publishing(device_id: &str, host: &str, port: u16) -> io::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("http://{host}:{port}/{device_id}/start");

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

    let text = response.text().await.unwrap();

    info!("start imu data publishing: {:#?}", text);
    println!("{:#?}", text);

    Ok(())
}

/// Stop imu data publishing
pub async fn stop_imu_data_publishing(device_id: &str, host: &str, port: u16) -> io::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("http://{host}:{port}/{device_id}/stop");

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

    let text = response.text().await.unwrap();

    info!("stop imu data publishing: {:#?}", text);
    println!("{:#?}", text);

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
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
