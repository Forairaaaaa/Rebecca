use crate::screen::CoverScreen;
use log::{debug, info};
use serde::Deserialize;
use std::io;
use zeromq::ReqSocket;
use zeromq::{Socket, SocketRecv, SocketSend};

#[derive(Debug, Deserialize)]
struct DeviceInfo {
    bits_per_pixel: u32,
    #[allow(dead_code)]
    description: String,
    #[allow(dead_code)]
    device_type: String,
    frame_buffer_port: u16,
    screen_size: [u32; 2],
}

pub struct SocketCoverScreen {
    device_info: DeviceInfo,
    socket: ReqSocket,
    frame_buffer: Vec<u8>,
}

#[derive(Deserialize, Debug)]
struct PushFrameResponse {
    status: u8,
    msg: String,
}

impl SocketCoverScreen {
    pub async fn list_screens(host: &str, port: u16) -> io::Result<Vec<String>> {
        let client = reqwest::Client::new();
        let url = format!("http://{host}:{port}/devices");

        let response = client.get(&url).send().await.map_err(|e| {
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

        devices.retain(|id| id.contains("screen"));

        Ok(devices)
    }

    pub async fn new(name: &str, host: &str, port: u16) -> io::Result<Self> {
        info!("create cover screen: {name}");

        let device_info = get_device_info(name, host, port).await?;

        let socket = create_socket(device_info.frame_buffer_port).await?;

        let frame_buffer = create_frame_buffer(&device_info);

        Ok(Self {
            device_info,
            socket,
            frame_buffer,
        })
    }
}

async fn get_device_info(name: &str, host: &str, port: u16) -> io::Result<DeviceInfo> {
    let client = reqwest::Client::new();
    let url = format!("http://{host}:{port}/{name}/info");

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

async fn create_socket(port: u16) -> io::Result<ReqSocket> {
    let mut socket = zeromq::ReqSocket::new();
    socket
        .connect(&format!("tcp://127.0.0.1:{port}"))
        .await
        .expect("failed to connect");
    info!("connected to socket port: {port}");

    Ok(socket)
}

fn create_frame_buffer(device_info: &DeviceInfo) -> Vec<u8> {
    let size_in_bytes =
        (device_info.screen_size[0] * device_info.screen_size[1] * device_info.bits_per_pixel / 8)
            as usize;

    info!(
        "create frame buffer: {} x {} {} bytes",
        device_info.screen_size[0], device_info.screen_size[1], size_in_bytes
    );

    vec![0u8; size_in_bytes]
}

impl CoverScreen for SocketCoverScreen {
    fn width(&self) -> u32 {
        self.device_info.screen_size[0]
    }

    fn height(&self) -> u32 {
        self.device_info.screen_size[1]
    }

    fn bpp(&self) -> u32 {
        self.device_info.bits_per_pixel
    }

    fn frame_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.frame_buffer
    }

    async fn push_frame(&mut self) -> io::Result<()> {
        debug!("push frame {} bytes", self.frame_buffer.len());

        // Send buffer
        self.socket
            .send(self.frame_buffer.clone().into()) // TODO: maybe avoid clone
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("send failed: {e}")))?;

        // Wait response
        let response = self
            .socket
            .recv()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("recv failed: {e}")))?;

        // Check response
        // debug!("response: {:?}", response);

        let response_json: PushFrameResponse = serde_json::from_slice(response.get(0).unwrap())
            .map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("parse response failed: {e}"))
            })?;

        debug!("response: {:?}", response_json);

        if response_json.status != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("push frame failed: {}", response_json.msg),
            ));
        }

        Ok(())
    }
}
