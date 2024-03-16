pub mod lan;
// pub mod signaling;
pub mod settings;
pub mod utility;

use app_kernel::{
    api::{repository::LocalStorage, endpoint::client::EndPointClient, signaling::SignalingClient},
    component::lan::LANProvider,
};
use moka::future::{Cache, CacheBuilder};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    storage: Mutex<Option<LocalStorage>>,
    signaling_client: Mutex<Option<(i64, SignalingClient)>>, // 信令客户端
    lan_provider: Mutex<Option<LANProvider>>,
    files_endpoints: Mutex<Cache<String, Arc<EndPointClient>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            storage: Mutex::new(None),
            signaling_client: Mutex::new(None),
            lan_provider: Mutex::new(None),
            files_endpoints: Mutex::new(CacheBuilder::new(64).build()),
        }
    }
}