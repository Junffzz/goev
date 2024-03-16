#![allow(non_snake_case)]

use std::cell::RefCell;
use std::net::{IpAddr, Ipv4Addr};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::controller::AppState;
use lazy_static::lazy_static;
use tokio::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc::{Receiver as stdReceiver,Sender as stdSender};
use tokio::sync::RwLock;
use tokio::time::sleep;
use app_kernel::api::endpoint::client::EndPointClient;
use app_kernel::api::endpoint::id::EndPointID;
use app_kernel::component::video_decoder::decoder::VideoDecoder;
use app_kernel::DesktopDecodeFrame;
use app_kernel::error::CoreResult;
use app_utility::tasks::GRuntime;
use crate::controller;
use crate::model::{setting_bridge::{SettingBridge, SettingData}};
use crate::model::setting_bridge::{MultiType};

lazy_static! {
    pub static ref SERVICE: Service = Service::start();
}

pub struct Service {
    app_state: Arc<AppState>,
    setting_data_state: Arc<SettingData>,
}

pub async  fn reload_setting_data_state(account_id: u16,setting_data:Arc<SettingData>,app_state: Arc<AppState>) {
    let app_state_clone = Arc::clone(&app_state);
    let setting_data_state = setting_data.clone();

    let cfg = app_state_clone.storage_get_config(account_id).await.expect("app_state.storage_get_config fail.");
    {
        let mut setting_config_guard = setting_data_state.config.write();
        *setting_config_guard = cfg;
    }
}

impl Service {
    pub fn start() -> Self {
        let state = Arc::new(AppState::new());
        let state_clone = Arc::clone(&state);

        let setting_data_state = Arc::new(SettingData::default());
        let setting_data_state_clone = Arc::clone(&setting_data_state);
        let setting_data_state_clone2 = Arc::clone(&setting_data_state);
        // 配置初始化
        let future = async move {
            state_clone.settings_init().await.expect("controller::settings::settings_init fail.");
            // lan初始化
            controller::lan::lan_init(&state_clone, true).await.expect("controller::lan::lan_init fail.");
            // 初始化配置
            let default_account= match state_clone.get_default_account().await{
                Ok(account)=>account,
                Err(e)=>{
                    println!("get_default_account fail: {:?}",e);
                    return;
                }
            };
            let account_id = default_account.id;
            // 确保这个生命周期内锁能释放
            {
                let mut setting_account_id_guard = setting_data_state_clone.account_id.write();
                *setting_account_id_guard = account_id;
            }
            // 载入配置
            reload_setting_data_state(account_id, setting_data_state_clone2, state_clone.clone()).await;
        };
        GRuntime().blocking_task(future);

        let slf = Self {
            app_state: state,
            setting_data_state ,
        };

        // todo:必须让初始化任务进行完成
        // 加载配置同步
        slf.load_setting_storage_sync();

        // todo:暂时逻辑

        let addr: String = "127.0.0.1".to_string();
        let remote_ip: IpAddr = addr
            .parse()
            .unwrap();

        let endpoint_id = EndPointID::LANID {
            local_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED), // 0.0.0.0
            remote_ip,// 远端ip
        };
        let new_endpoint_id = Arc::new(Mutex::new(endpoint_id.clone()));
        slf.connect_client(new_endpoint_id.clone()).expect("connect_client fail.");

        slf
    }

    // 加载设置存储同步
    pub fn load_setting_storage_sync(&self) {
        let app_state = Arc::clone(&self.app_state);
        let setting_data_state = Arc::clone(&self.setting_data_state);
        let future = async move {
            let interval_secs = 5; // 每5秒同步一次
            let mut interval = Duration::from_secs(interval_secs);
            loop {
                let account_id = {
                    let account_id_guard = setting_data_state.account_id.read();
                    *account_id_guard
                };
                if account_id <= 0 {
                    interval = Duration::from_secs(interval_secs);
                    continue;
                }
                let cfg = {
                    let config_guard = setting_data_state.config.read();
                    (*config_guard).clone()
                };
                app_state.storage_update_config(cfg).await.expect("controller::settings::settings_sync fail.");
                sleep(interval).await;
                interval = Duration::from_secs(interval_secs);
            }
        };
        GRuntime().task(future);
    }

    // 初始化桥接ui设置
    pub fn init_bridge_setting_ui(&self) -> (stdSender<SettingBridge>,Arc<SettingData>) {
        let (bridge_tx, mut bridge_rx): (stdSender<SettingBridge>, stdReceiver<SettingBridge>) = std::sync::mpsc::channel();

        let app_state = self.app_state.clone();
        let setting_data_state = self.setting_data_state.clone();
        let future = async move {
            tracing::info!("settings ui bridge process");

            while let Ok(setting_data) = bridge_rx.recv() {
                println!("setting_data: {:?}", setting_data);
                if setting_data.cfg_key=="connect_account_id"{
                    if let MultiType::U16(a_id) = setting_data.cfg_val {
                        // reload_setting_data_state(a_id,setting_data_state.clone(),app_state.clone()).await;
                    }
                    continue
                }
                // 修改同步中的数据
                let mut theme_lock=setting_data_state.theme.write();
                *theme_lock="dark123456".to_string();
                println!("init_bridge_setting_ui theme: {}", *theme_lock);
                // app_state.settings_set_config(setting_data.cfg_key, setting_data.cfg_val).expect("app_state.settings_set_config fail.");
            }

            tracing::info!("settings ui bridge process exit");
        };
        GRuntime().task(future);

        (bridge_tx,self.setting_data_state.clone())
    }

    // 连接客户端
    pub fn connect_client(&self, endpoint_id: Arc<Mutex<EndPointID>>) -> CoreResult<(Arc<EndPointClient>, Receiver<DesktopDecodeFrame>)> {
        let endpoint_id_clone = endpoint_id.clone();
        let endpoint_id_clone2 = endpoint_id.clone();

        let connect_client = Arc::new(Mutex::new(EndPointClient::new(endpoint_id_clone.lock().unwrap().clone())));

        let connect_client_clone = Arc::clone(&connect_client);

        let app_state = Arc::clone(&self.app_state);
        let (render_frame_tx, render_frame_rx) = tokio::sync::mpsc::channel(180);

        let addr = "127.0.0.1".to_string(); // Convert &str to String
        let future = async move {
            // todo: 保证lan::init初始化完成
            sleep(Duration::from_secs(3)).await;

            let future = controller::lan::lan_connect("127.0.0.1".to_string(), render_frame_tx);
            let (endpoint_id1, client) = future.await.expect("controller::lan::lan_connect fail.");
            // println!("lan endpoint_id: {:?}", endpoint_id);
            let mut conn_endpoint_id_guard = endpoint_id_clone2.lock().unwrap();
            *conn_endpoint_id_guard = endpoint_id1;

            {
                let mut connect_client_clone_guard = connect_client.lock().unwrap();
                *connect_client_clone_guard = (*client).clone();
            }
        };
        GRuntime().task(future);

        let client_guard = connect_client_clone.lock().unwrap();
        let client = Arc::new(client_guard.clone());

        Ok((client, render_frame_rx))
    }
}

pub fn GService() -> &'static Service {
    &SERVICE
}