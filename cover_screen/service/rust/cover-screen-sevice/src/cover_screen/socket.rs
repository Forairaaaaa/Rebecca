use crate::cover_screen::screen::{ScreenInfo, push_frame_buffer};
use chrono::Utc;
use regex::Regex;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::io::Result as IoResult;
use std::path::PathBuf;
use zeromq::{Socket, SocketRecv, SocketSend};

pub struct ScreenSocket {
    screen: ScreenInfo,
    socket_name: String,
    frame_buffer_port: u16,
    frame_buffer_socket: zeromq::RepSocket,
}

#[derive(Serialize)]
struct SocketInfoFile {
    name: String,
    screen_size: (u32, u32),
    bits_per_pixel: u32,
    frame_buffer_port: u16,
    created_at: String,
    device_path: String,
}

const SOCKET_INFO_DIR: &str = "/tmp/cover_screen";

impl ScreenSocket {
    pub async fn new(
        target_screen: &ScreenInfo,
        socket_name: &String,
        base_port: Option<u16>,
    ) -> IoResult<Self> {
        let screen = target_screen.clone();
        let socket_name = socket_name.clone();
        let mut frame_buffer_port = base_port.unwrap_or(0);

        // Create frame buffer socket
        let mut frame_buffer_socket = zeromq::RepSocket::new();
        let endpoint: String;
        if frame_buffer_port == 0 {
            endpoint = "tcp://127.0.0.1:0".to_string();
        } else {
            endpoint = format!("tcp://127.0.0.1:{}", frame_buffer_port);
        }
        let ep = frame_buffer_socket
            .bind(&endpoint)
            .await
            .unwrap()
            .to_string();

        // Store actual port
        let re = Regex::new(r":(\d+)$").unwrap();
        let caps = re.captures(&ep).unwrap();
        frame_buffer_port = caps[1].parse().unwrap();

        // Create info file
        fs::create_dir_all(SOCKET_INFO_DIR)?;
        let info_path = format!("{}/{}.json", SOCKET_INFO_DIR, socket_name);
        let info_file = SocketInfoFile {
            name: socket_name.clone(),
            screen_size: screen.resolution,
            bits_per_pixel: screen.bpp,
            frame_buffer_port,
            created_at: Utc::now().to_rfc3339(),
            device_path: screen.device_path.to_string_lossy().to_string(),
        };
        let json = serde_json::to_string_pretty(&info_file).unwrap();
        fs::write(info_path, json)?;

        Ok(Self {
            screen,
            socket_name,
            frame_buffer_port,
            frame_buffer_socket,
        })
    }

    pub async fn listen(&mut self) {
        match self.frame_buffer_socket.recv().await {
            Ok(msg) => {
                let data: &[u8] = msg.get(0).unwrap();
                let response: String;

                match push_frame_buffer(&self.screen, data) {
                    Ok(()) => {
                        response = json!({"status": 0, "msg": "ok👌"}).to_string();
                    }
                    Err(e) => {
                        response = json!({"status": 1, "msg": e.to_string()}).to_string();
                    }
                }

                self.frame_buffer_socket
                    .send(response.into())
                    .await
                    .unwrap();
            }
            Err(e) => {
                eprintln!("ZMQ recv error: {:?}", e);
            }
        }
    }
}

pub fn cleanup() {
    let dir = PathBuf::from(SOCKET_INFO_DIR);
    if dir.exists() {
        fs::remove_dir_all(dir).unwrap();
    }
}
