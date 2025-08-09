use crate::devices::{DeviceInfo, screen::Screen};
use log::error;
use regex::Regex;
use serde::Serialize;
use serde_json::json;
use std::io;
use zeromq::{Socket, SocketRecv, SocketSend};

/// Screen socket
/// 监听一个 screen zmq rep socket 把接收数据推送到屏幕
pub struct ScreenSocket {
    pub id: String,
    screen: Box<dyn Screen>,
    frame_buffer_port: u16,
    frame_buffer_socket: zeromq::RepSocket,
}

#[derive(Serialize, Debug)]
struct ScreenSocketInfo {
    screen_size: (u32, u32),
    bits_per_pixel: u32,
    frame_buffer_port: u16,
    device_type: String,
    description: String,
}

impl ScreenSocket {
    pub async fn new(screen: Box<dyn Screen>, id: String) -> io::Result<Self> {
        // Create frame buffer zmq socket
        let mut frame_buffer_socket = zeromq::RepSocket::new();
        let ep = frame_buffer_socket
            .bind("tcp://127.0.0.1:0")
            .await
            .unwrap()
            .to_string();

        // Get port
        let frame_buffer_port = Regex::new(r":(\d+)$")
            .ok()
            .and_then(|re| re.captures(&ep))
            .and_then(|caps| caps.get(1))
            .and_then(|port_str| port_str.as_str().parse().ok());
        let frame_buffer_port = frame_buffer_port.unwrap_or_else(|| {
            error!("parse port from '{}' failed", ep);
            0
        });

        Ok(Self {
            id,
            screen,
            frame_buffer_port,
            frame_buffer_socket,
        })
    }

    pub async fn listen(&mut self) {
        match self.frame_buffer_socket.recv().await {
            Ok(msg) => {
                let data: &[u8] = msg.get(0).unwrap();
                let response: String;

                match self.screen.push_frame_buffer(data) {
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
                error!("ZMQ recv error: {:?}", e);
            }
        }
    }

    pub fn get_device_info(&self) -> DeviceInfo {
        let screen_socket_info = ScreenSocketInfo {
            screen_size: self.screen.size(),
            bits_per_pixel: self.screen.bpp(),
            frame_buffer_port: self.frame_buffer_port,
            device_type: self.screen.device_type(),
            description:
                "Render a frame by sending a raw buffer to <frame_buffer_port> via ZMQ REP socket."
                    .to_string(),
        };

        DeviceInfo {
            id: self.id.clone(),
            info: serde_json::to_string_pretty(&screen_socket_info).unwrap(),
        }
    }
}
