use crate::common::Emoji;
use crate::devices::backlight::{Backlight, MockBacklight, SysfsBacklight};
use crate::devices::{API_REGISTER, ApiRoute};
use hyper::{Method, Response, StatusCode, header::CONTENT_TYPE};
use log::{error, info, warn};
use serde_json::json;
use std::io;
use std::sync::Arc;
use tokio::{sync::Notify, task};

// 批量克隆
macro_rules! arc_clones {
    ($arc_var:ident, $($name:ident),*) => {
        $( let $name = Arc::clone(&$arc_var); )*
    };
}

// 注册设备
async fn register_device(id: &str, backlight: &Arc<dyn Backlight + Send + Sync>) {
    arc_clones!(
        backlight,
        backlight_clone1,
        backlight_clone2,
        backlight_clone3
    );

    let success_response = || -> Response<String> {
        Response::builder()
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body("ok👍".to_string())
            .unwrap()
    };
    let error_response = |e: io::Error| {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(e.to_string())
            .unwrap()
    };

    // Add device to device list
    API_REGISTER.add_device(id.to_string()).await;

    // Get info
    if let Err(e) = API_REGISTER
        .add_api(
            ApiRoute {
                path: format!("/{}/info", id),
                method: Method::GET,
                description: format!("{} Get device info.", Emoji::INFO),
            },
            Box::new(move |_request| {
                let backlight = Arc::clone(&backlight_clone1);
                Box::pin(async move {
                    let info = json!({
                        "device_type": backlight.device_type(),
                        "max_brightness": backlight.max_brightness(),
                        "current_brightness": backlight.get_brightness().unwrap_or(0.0),
                        "description": format!("{} Control backlight brightness (0.0~1.0).", Emoji::LIGHT)
                    });

                    Response::builder()
                        .header(CONTENT_TYPE, "application/json; charset=utf-8")
                        .body(serde_json::to_string_pretty(&info).unwrap_or("wtf?🤡".to_string()))
                        .unwrap()
                })
            }),
        )
        .await
    {
        error!("failed to register backlight info api: {}", e);
    }

    // Get brightness
    if let Err(e) = API_REGISTER
        .add_api(
            ApiRoute {
                path: format!("/{}/get", id),
                method: Method::GET,
                description: format!("{} Get current brightness (0.0~1.0).", Emoji::LIGHT),
            },
            Box::new(move |_request| {
                let backlight = Arc::clone(&backlight_clone2);
                Box::pin(async move {
                    match backlight.get_brightness() {
                        Ok(brightness) => {
                            let response = json!({
                                "brightness": brightness
                            });
                            Response::builder()
                                .header(CONTENT_TYPE, "application/json; charset=utf-8")
                                .body(serde_json::to_string(&response).unwrap_or("wtf?🤡".to_string()))
                                .unwrap()
                        }
                        Err(e) => error_response(e),
                    }
                })
            }),
        )
        .await
    {
        error!("failed to register backlight get api: {}", e);
    }

    // Set brightness (using URL parameter instead of body for simplicity)
    if let Err(e) = API_REGISTER
        .add_api(
            ApiRoute {
                path: format!("/{}/set", id),
                method: Method::GET,
                description: format!("{} Set brightness (0.0~1.0). Use query parameter: /{}/set?brightness=0.5", Emoji::START, id),
            },
            Box::new(move |request| {
                let backlight = Arc::clone(&backlight_clone3);
                Box::pin(async move {
                    // Parse brightness from query parameter
                    let uri = request.uri();
                    let query = uri.query().unwrap_or("");
                    
                    let brightness: f32 = if let Some(brightness_str) = query
                        .split('&')
                        .find(|param| param.starts_with("brightness="))
                        .and_then(|param| param.split('=').nth(1))
                    {
                        match brightness_str.parse::<f32>() {
                            Ok(b) => b,
                            Err(_) => {
                                return Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                                    .body("invalid brightness value, must be a number between 0.0 and 1.0".to_string())
                                    .unwrap()
                            }
                        }
                    } else {
                        return Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                            .body("missing brightness parameter".to_string())
                            .unwrap()
                    };

                    match backlight.set_brightness(brightness) {
                        Ok(()) => success_response(),
                        Err(e) => error_response(e),
                    }
                })
            }),
        )
        .await
    {
        error!("failed to register backlight set api: {}", e);
    }
}

/// Start backlight service to handle backlight devices
/// # Arguments
/// * `host` - The host for ZMQ socket to bind to (not used for backlight, but kept for consistency)
/// * `shutdown_notify` - A notify clone for shutdown signal
/// * `mock_backlight` - Whether to create mock backlight for api test
/// # Returns
/// A `task::JoinHandle` that can be used to wait for the backlight service to shutdown
pub async fn start_backlight_service(
    _host: &str,
    shutdown_notify: Arc<Notify>,
    mock_backlight: bool,
) -> io::Result<task::JoinHandle<()>> {
    let mut backlights: Vec<Arc<dyn Backlight + Send + Sync>> = Vec::new();

    // Create sysfs backlights
    let sysfs_backlights = SysfsBacklight::get_all_devices();
    for backlight in sysfs_backlights {
        if let Err(e) = backlight.init() {
            warn!("failed to init backlight {}: {}", backlight.name(), e);
            continue;
        }
        info!("initialized backlight device: {}", backlight.name());
        backlights.push(Arc::new(backlight));
    }

    // Create mock backlights
    if mock_backlight {
        info!("create mock backlights");
        let mock = MockBacklight::new("mock", 2047);

        if let Err(e) = mock.init() {
            warn!("failed to init mock backlight {}: {}", mock.name(), e);
        } else {
            backlights.push(Arc::new(mock));
        }
    }

    if backlights.is_empty() {
        warn!("no backlight devices found");
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no backlight devices found",
        ));
    }

    // Register devices
    for (i,backlight) in backlights.iter().enumerate() {
        register_device(format!("backlight{}", i).as_str(), backlight).await;
    }

    info!("backlight service started with {} devices", backlights.len());

    // Start service task
    Ok(task::spawn(async move {
        // Wait for shutdown signal
        shutdown_notify.notified().await;
        info!("backlight service shutdown...");

        // Cleanup devices
        for backlight in backlights {
            if let Err(e) = backlight.deinit() {
                error!("failed to deinit backlight {}: {}", backlight.name(), e);
            }
        }

        info!("backlight service shutdown complete");
    }))
}
