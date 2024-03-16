use tokio::sync::RwLock;
use parking_lot::RwLock as plRwLock;
use app_kernel::api::repository::account::Account;
use app_kernel::api::repository::config::Config;

// Define an enum that can be String, int, or bool
#[derive(Clone, Debug)]
pub enum MultiType {
    String(String),
    Int(i32),
    U16(u16),
    F32(f32),
    Bool(bool),
}

#[derive(Clone, Debug)]
pub struct SettingBridge {
    pub cfg_key: String,
    pub cfg_val: MultiType,
}

// 和UI交互的数据
pub struct SettingData {
    pub account_id: plRwLock<u16>,
    pub theme: plRwLock<String>, // 主题：dark，light
    pub lang:plRwLock<String>,
    pub config: plRwLock<Config>,
    pub account: plRwLock<Account>,
}

impl Default for SettingData {
    fn default() -> Self {
        Self {
            account_id: plRwLock::new(0),
            theme: plRwLock::new("light".to_string()),
            lang: plRwLock::new("zh".to_string()),
            config: plRwLock::new(Config::default()),
            account: plRwLock::new(Account::default()),
        }
    }
}