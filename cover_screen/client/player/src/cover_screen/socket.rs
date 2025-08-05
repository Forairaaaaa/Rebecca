use crate::cover_screen::CoverScreen;
use log::{debug, info};
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use zeromq::ReqSocket;
use zeromq::{Socket, SocketRecv, SocketSend};

const SCREEN_INFO_DIR: &str = "/tmp/cover_screen";

#[derive(Debug, Deserialize)]
pub struct SocketInfoFile {
    pub name: String,
    pub screen_size: (u32, u32),
    pub bits_per_pixel: u32,
    pub frame_buffer_port: u16,
    pub created_at: Option<String>,
    pub device_path: Option<String>,
}

pub struct SocketCoverScreen {
    pub name: String,
    pub socket_info: SocketInfoFile,
    socket: ReqSocket,
    pub frame_buffer: Vec<u8>,
}

#[derive(Deserialize)]
struct PushFrameResponse {
    status: u8,
    msg: String,
}

impl SocketCoverScreen {
    pub async fn new(name: &str) -> io::Result<Self> {
        info!("create cover screen: {name}");

        let socket_info = get_socket_info(name)?;

        let socket = create_socket(socket_info.frame_buffer_port).await?;

        let frame_buffer = create_frame_buffer(&socket_info);

        Ok(Self {
            name: name.to_string(),
            socket_info,
            socket,
            frame_buffer,
        })
    }
}

fn get_socket_info(name: &str) -> io::Result<SocketInfoFile> {
    let info_path = Path::new(SCREEN_INFO_DIR).join(format!("{}.json", name));
    let content = fs::read_to_string(&info_path)?;
    let info: SocketInfoFile = serde_json::from_str(&content)?;

    info!("get socket info: {:#?}", info);

    Ok(info)
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

fn create_frame_buffer(socket_info: &SocketInfoFile) -> Vec<u8> {
    let size_in_bytes =
        (socket_info.screen_size.0 * socket_info.screen_size.1 * socket_info.bits_per_pixel / 8)
            as usize;

    info!(
        "create frame buffer: {} x {} {} bytes",
        socket_info.screen_size.0, socket_info.screen_size.1, size_in_bytes
    );

    vec![0u8; size_in_bytes]
}

impl CoverScreen for SocketCoverScreen {
    fn width(&self) -> u32 {
        self.socket_info.screen_size.0
    }

    fn height(&self) -> u32 {
        self.socket_info.screen_size.1
    }

    fn bpp(&self) -> u32 {
        self.socket_info.bits_per_pixel
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
        debug!("response: {:?}", response);

        let response_json: PushFrameResponse = serde_json::from_slice(response.get(0).unwrap())
            .map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("parse response failed: {e}"))
            })?;

        if response_json.status != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("push frame failed: {}", response_json.msg),
            ));
        }

        Ok(())
    }
}
