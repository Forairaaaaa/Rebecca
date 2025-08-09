use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub info: String,
}
