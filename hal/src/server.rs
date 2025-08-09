use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use log::{error, info};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::{sync::Notify, task};

async fn handle_request(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<String>, Infallible> {
    let path = req.uri().path();
    let method = req.method();

    match (method, path) {
        (&Method::GET, path) if path.starts_with("/get-device/") => {
            // 提取设备ID
            let device_id = &path[12..]; // 跳过 "/get-device/" 前缀

            if device_id.is_empty() {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body("empty device id".to_string())
                    .unwrap());
            }

            info!("get device info: {}", device_id);

            // TODO
            let response_body = serde_json::json!({
                "device_id": device_id,
                "status": "online",
                "message": format!("👍👍 {}", device_id)
            });

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(response_body.to_string())
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("not found".to_string())
            .unwrap()),
    }
}

/// Start a http server to handle hal request
/// # Arguments
/// * `port` - The port to listen on
/// * `device_infos` - A vector of available device infos
/// * `shutdown_notify` - A notify clone for shutdown signal
/// # Returns
/// A `task::JoinHandle` that can be used to wait for the server to shutdown
pub fn start_server(port: u16, shutdown_notify: Arc<Notify>) -> task::JoinHandle<()> {
    task::spawn(async move {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = match TcpListener::bind(addr).await {
            Ok(listener) => listener,
            Err(e) => {
                error!("failed to bind server: {}", e);
                return;
            }
        };

        info!("http server started at http://{}", addr);
        info!("use /get-device/<device_id> to get device info");

        loop {
            tokio::select! {
                // Check shutdown signal
                _ = shutdown_notify.notified() => {
                    info!("server shutdown...");
                    break;
                }

                // Accept new connection
                result = listener.accept() => {
                    let (stream, _) = match result {
                        Ok(conn) => conn,
                        Err(e) => {
                            error!("accept connection failed: {}", e);
                            continue;
                        }
                    };

                    tokio::task::spawn(async move {
                        let io = TokioIo::new(stream);
                        if let Err(err) = auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                            .serve_connection_with_upgrades(io, service_fn(move |req| {
                                handle_request(req)
                            }))
                        .await
                        {
                            error!("connection error: {}", err);
                        }
                    });
                }
            }
        }

        info!("server shutdown complete");
    })
}
