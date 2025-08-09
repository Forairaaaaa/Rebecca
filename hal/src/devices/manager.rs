use super::types::DeviceInfo;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type GlobalDeviceManager = Arc<DeviceManager>;

#[derive(Debug)]
pub struct DeviceManager {
    devices: RwLock<HashMap<String, DeviceInfo>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_device(&self, device: DeviceInfo) -> Result<(), String> {
        let mut devices = self.devices.write().await;

        if devices.contains_key(&device.id) {
            return Err(format!("device '{}' already exists", device.id));
        }

        devices.insert(device.id.clone(), device);
        Ok(())
    }

    pub async fn has_device(&self, device_id: &str) -> bool {
        let devices = self.devices.read().await;
        devices.contains_key(device_id)
    }

    pub async fn get_device(&self, device_id: &str) -> Option<DeviceInfo> {
        let devices = self.devices.read().await;
        devices.get(device_id).cloned()
    }

    pub async fn get_all_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    pub async fn remove_device(&self, device_id: &str) -> Option<DeviceInfo> {
        let mut devices = self.devices.write().await;
        devices.remove(device_id)
    }

    pub async fn device_count(&self) -> usize {
        let devices = self.devices.read().await;
        devices.len()
    }

    pub async fn update_device(&self, device: DeviceInfo) -> Result<(), String> {
        let mut devices = self.devices.write().await;

        if !devices.contains_key(&device.id) {
            return Err(format!("device '{}' not exist", device.id));
        }

        devices.insert(device.id.clone(), device);
        Ok(())
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

pub static DEVICE_MANAGER: Lazy<GlobalDeviceManager> = Lazy::new(|| Arc::new(DeviceManager::new()));
