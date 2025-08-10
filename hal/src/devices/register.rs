/// 全局单例 API 注册表，用于注册设备 API，以及给 server 提供路由和回调
use crate::common::Emoji;
use derivative::Derivative;
use hyper::{Method, Request, Response, StatusCode};
use indexmap::IndexMap;
use log::{debug, error};
use once_cell::sync::Lazy;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 设备 API 路由
#[derive(Derivative)]
#[derivative(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiRoute {
    pub path: String,
    pub method: Method,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub description: String, // 不参与 hash 和 eq 比较
}

impl Serialize for ApiRoute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ApiRoute", 2)?;
        s.serialize_field("path", &self.path)?;
        s.serialize_field("method", &self.method.as_str())?; // 用字符串序列化 method
        s.serialize_field("description", &self.description)?;
        s.end()
    }
}

/// 设备 API 回调
pub type ApiCallback = Box<
    dyn Fn(Request<hyper::body::Incoming>) -> Pin<Box<dyn Future<Output = Response<String>> + Send>>
        + Send
        + Sync,
>;

pub struct ApiRegister {
    api_map: RwLock<IndexMap<ApiRoute, ApiCallback>>,
    device_list: RwLock<Vec<String>>,
}

impl ApiRegister {
    pub fn new() -> Self {
        let mut api_map: IndexMap<ApiRoute, ApiCallback> = IndexMap::new();

        api_map.insert(
            ApiRoute {
                path: "/apis".to_string(),
                method: Method::GET,
                description: format!("{} List all available APIs", Emoji::APIS),
            },
            Box::new(move |_request| {
                Box::pin(async move {
                    let api_routes = API_REGISTER.get_all_api_routes().await;
                    let body =
                        serde_json::to_string_pretty(&api_routes).unwrap_or("wtf?🤡".to_string());
                    Response::new(body)
                })
            }),
        );

        api_map.insert(
            ApiRoute {
                path: "/devices".to_string(),
                method: Method::GET,
                description: format!("{} List all available devices", Emoji::DEVICES),
            },
            Box::new(move |_request| {
                Box::pin(async move {
                    let device_list = API_REGISTER.get_device_list().await;
                    let body =
                        serde_json::to_string_pretty(&device_list).unwrap_or("wtf?🤡".to_string());
                    Response::new(body)
                })
            }),
        );

        Self {
            api_map: RwLock::new(api_map),
            device_list: RwLock::new(vec![]),
        }
    }

    pub async fn add_api(&self, route: ApiRoute, callback: ApiCallback) -> Result<(), String> {
        let mut api_map = self.api_map.write().await;

        if api_map.contains_key(&route) {
            return Err(format!("api route: '{:?}' already exists", route));
        }

        debug!("add api route: {:?}", route);

        api_map.insert(route.clone(), callback);
        Ok(())
    }

    pub async fn get_all_api_routes(&self) -> Vec<ApiRoute> {
        let api_map = self.api_map.read().await;
        api_map.keys().cloned().collect()
    }

    pub async fn invoke_api(
        &self,
        route: ApiRoute,
        request: Request<hyper::body::Incoming>,
    ) -> Response<String> {
        let api_map = self.api_map.read().await;
        if let Some(callback) = api_map.get(&route) {
            // 调用回调
            debug!("invoke api route: {:?}", route);
            (callback)(request).await
        } else {
            // 找不到路由，返回 404
            debug!("api route: {:?} not found", route);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not Found".to_string())
                .unwrap()
        }
    }

    /// Add device to device list so the /devices can find it
    pub async fn add_device(&self, device_id: String) {
        let mut device_list = self.device_list.write().await;
        if device_list.contains(&device_id) {
            error!("device: {:?} already exists", device_id);
            return;
        }
        device_list.push(device_id);
    }

    pub async fn get_device_list(&self) -> Vec<String> {
        let device_list = self.device_list.read().await;
        device_list.clone()
    }
}

impl Default for ApiRegister {
    fn default() -> Self {
        Self::new()
    }
}

pub type GlobalApiRegister = Arc<ApiRegister>;
pub static API_REGISTER: Lazy<GlobalApiRegister> = Lazy::new(|| Arc::new(ApiRegister::new()));
