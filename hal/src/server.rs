use crate::devices::{API_REGISTER, ApiRoute};
use hyper::service::service_fn;
use hyper::{Request, Response};
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
    let api_route = ApiRoute {
        path: req.uri().path().to_string(),
        method: req.method().clone(),
        description: "🔍".to_string(),
    };

    Ok(API_REGISTER.invoke_api(api_route, req).await)
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
