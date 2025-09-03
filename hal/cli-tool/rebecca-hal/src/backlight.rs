use clap::Subcommand;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Subcommand, Debug)]
pub enum BacklightCommand {
    /// Get backlight device information
    Info,
    /// Get current brightness (0.0~1.0)
    Get,
    /// Set brightness (0.0~1.0)
    Set {
        /// Brightness value (0.0~1.0)
        brightness: f32,
    },
}

/// Handle backlight subcommand
pub async fn handle_backlight_command(
    device_id: Option<String>,
    command: Option<BacklightCommand>,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        // If no subcommand provided, default behavior based on device_id
        None => {
            if device_id.is_none() {
                // rebecca-hal backlight - list all backlights
                let backlights = list_backlight(host, port).await?;
                let json = serde_json::to_string(&backlights)?;
                println!("{}", json);
            } else {
                // rebecca-hal backlight backlight0 - default to get brightness
                let device_id = device_id.unwrap();
                let brightness = get_brightness(&device_id, host, port).await?;
                let json = serde_json::to_string(&brightness)?;
                println!("{}", json);
            }
        }
        Some(BacklightCommand::Info) => {
            if let Some(device_id) = device_id {
                let device_info = get_device_info(&device_id, host, port).await?;
                let json = serde_json::to_string_pretty(&device_info)?;
                println!("{}", json);
            } else {
                eprintln!("Error: device_id is required for info command");
                std::process::exit(1);
            }
        }
        Some(BacklightCommand::Get) => {
            if let Some(device_id) = device_id {
                let brightness = get_brightness(&device_id, host, port).await?;
                let json = serde_json::to_string(&brightness)?;
                println!("{}", json);
            } else {
                eprintln!("Error: device_id is required for get command");
                std::process::exit(1);
            }
        }
        Some(BacklightCommand::Set { brightness }) => {
            if let Some(device_id) = device_id {
                set_brightness(&device_id, brightness, host, port).await?;
            } else {
                eprintln!("Error: device_id is required for set command");
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

/// List all available backlight devices
pub async fn list_backlight(host: &str, port: u16) -> io::Result<Vec<String>> {
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

    devices.retain(|id| id.contains("backlight"));

    Ok(devices)
}

/// Get backlight device info
pub async fn get_device_info(device_id: &str, host: &str, port: u16) -> io::Result<BacklightInfo> {
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

    let device_info: BacklightInfo = response.json().await.map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to parse JSON: {}", e))
    })?;

    info!("get backlight device info: {:#?}", device_info);

    Ok(device_info)
}

/// Get current brightness
pub async fn get_brightness(device_id: &str, host: &str, port: u16) -> io::Result<BrightnessValue> {
    let client = reqwest::Client::new();
    let url = format!("http://{host}:{port}/{device_id}/get");

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

    let brightness: BrightnessValue = response.json().await.map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to parse JSON: {}", e))
    })?;

    debug!("get brightness: {:#?}", brightness);

    Ok(brightness)
}

/// Set brightness
pub async fn set_brightness(
    device_id: &str,
    brightness: f32,
    host: &str,
    port: u16,
) -> io::Result<()> {
    let client = reqwest::Client::new();
    let url = format!("http://{host}:{port}/{device_id}/set?brightness={brightness}");

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

    info!("set brightness: {:#?}", text);
    println!("{:#?}", text);

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BacklightInfo {
    device_type: String,
    description: String,
    current_brightness: f32,
    max_brightness: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrightnessValue {
    brightness: f32,
}
