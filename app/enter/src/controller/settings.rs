use crate::controller::AppState;
use app_kernel::{
    api::{
        repository::{
            account::Account, config::Config, kv::Theme,
            LocalStorage,
        },
        signaling::http_message::Response,
    },
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use app_kernel::{core_error,error::CoreResult};

#[tracing::instrument(skip( app_state))]
pub async fn settings_init(
    app_state: &AppState,
) -> CoreResult<()> {
    let config_dir: PathBuf = dirs_sys_next::home_dir().map(|h|h.join("workspace/rust/NextRiftXR_desktop")).unwrap();

    std::fs::create_dir_all(config_dir.clone())?;
    let storage_path = config_dir.join("desktop.db");

    tracing::info!(path = ?storage_path, "read repository");

    let storage = LocalStorage::new(storage_path)?;
    let account_count = storage.account().get_account_count()?;

    let mut storage_guard = app_state.storage.lock().await;
    *storage_guard = Some(storage);
    drop(storage_guard); // 释放锁

    if account_count == 0 {

    }

    Ok(())
}

// #[tracing::instrument(skip(app_state))]
// pub async fn settings_account_get(app_state: &AppState) -> CoreResult<Account> {
//     let Some(ref storage) = *app_state.storage.lock().await else {
//         return Err(core_error!("storage not initialize"));
//     };
//
//     storage.account().get_default_account()
// }


#[tracing::instrument(skip(app_state))]
pub async fn settings_account_get_by_name(
    app_state: &AppState,
    name: String,
) -> CoreResult<Account> {
    let Some(ref storage) = *app_state.storage.lock().await else {
        return Err(core_error!("storage not initialize"));
    };

    storage.account().get_account_by_name(name)
}


#[tracing::instrument(skip(app_state))]
pub async fn settings_account_create(
    app_state: &AppState,
    title:String,
    addr: String,
    is_default: bool,
    remarks: String,
) -> CoreResult<()> {

    let Some(ref storage) = *app_state.storage.lock().await else {
        return Err(core_error!("storage not initialize"));
    };

    let domain = "".to_string();
    let port = 2803;
    if storage.account().account_exist(&domain)? {
        return Err(core_error!("domain is exists"));
    }

    storage.account().add_account(Account {
        id: 0,
        title,
        remote_ip: addr,
        remote_port: port,
        is_default,
        device_id: 0,
        remote_os: "macos".to_string(),// 如果云电脑，通过解析获取
        auto_connect: false,
        remarks,
    })?;

    Ok(())
}

#[tracing::instrument(skip(app_state))]
pub async fn settings_account_delete(id: i64, app_state: AppState) -> CoreResult<()> {
    let Some(ref storage) = *app_state.storage.lock().await else {
        return Err(core_error!("storage not initialize"));
    };

    storage.account().delete_account(id)?;

    Ok(())
}

#[derive(Serialize)]
pub struct ConfigDomainListResponse {
    pub total: u32,
    pub accounts: Vec<Account>,
}


#[tracing::instrument(skip(app_state))]
pub async fn settings_account_list(
    page: u32,
    limit: u32,
    app_state: &AppState,
) -> CoreResult<ConfigDomainListResponse> {
    let Some(ref storage) = *app_state.storage.lock().await else {
        return Err(core_error!("storage not initialize"));
    };

    let (total, accounts) = storage.account().get_accounts(page, limit)?;

    Ok(ConfigDomainListResponse { total, accounts })
}


#[tracing::instrument(skip(app_state))]
pub async fn settings_language_get(app_state: AppState) -> CoreResult<String> {
    let Some(ref storage) = *app_state.storage.lock().await else {
        return Err(core_error!("storage not initialize"));
    };

    Ok(storage.kv().get_language()?.unwrap_or_default())
}

#[derive(Serialize, Clone)]
struct UpdateLanguageEvent {
    pub language: String,
}


// #[tracing::instrument(skip(app_state, app_handle))]
// pub async fn config_language_set(
//     app_state: AppState,
//     app_handle: AppHandle,
//     language: String,
// ) -> CoreResult<()> {
//     let Some(ref storage) = *app_state.storage.lock().await else {
//         return Err(core_error!("storage not initialize"));
//     };
//
//     storage.kv().set_language(&language)?;
//
//     app_handle
//         .emit_all(
//             "update_language",
//             UpdateLanguageEvent {
//                 language: language.clone(),
//             },
//         )
//         .map_err(|err| {
//             tracing::error!(?err, "emit event 'update_language' failed");
//             core_error!("emit event 'update_language' failed")
//         })?;
//
//     // update menu language
//
//     let (quit_text, show_text, hide_text, about_text) = match language.as_str() {
//         "en" => ("Quit", "Show", "Hide", "About"),
//         "zh" => ("退出", "显示", "隐藏", "关于"),
//         _ => return Ok(()),
//     };
//
//     let quit = CustomMenuItem::new("quit", quit_text);
//     let show = CustomMenuItem::new("show", show_text);
//     let hide = CustomMenuItem::new("hide", hide_text);
//     let about = CustomMenuItem::new("about", about_text);
//
//     let tray_menu = if cfg!(target_os = "macos") {
//         SystemTrayMenu::new()
//             .add_item(hide)
//             .add_item(show)
//             .add_native_item(SystemTrayMenuItem::Separator)
//             .add_item(quit)
//     } else {
//         SystemTrayMenu::new()
//             .add_item(hide)
//             .add_item(show)
//             .add_native_item(SystemTrayMenuItem::Separator)
//             .add_item(about)
//             .add_native_item(SystemTrayMenuItem::Separator)
//             .add_item(quit)
//     };
//
//     if let Err(err) = app_handle.tray_handle().set_menu(tray_menu) {
//         tracing::error!(?err, "set new tray menu failed");
//     }
//
//     #[cfg(target_os = "macos")]
//     {
//         let Some(window) = app_handle.get_window("main") else {
//             return Ok(());
//         };
//
//         let about_text = match language.as_str() {
//             "en" => "About",
//             "zh" => "关于",
//             _ => return Ok(()),
//         };
//
//         if let Err(err) = window
//             .menu_handle()
//             .get_item("about")
//             .set_title(format!("{about_text} MirrorX"))
//         {
//             tracing::error!(menu = "about", ?err, "set os menu failed");
//         }
//
//         if let Err(err) = window.menu_handle().get_item("quit").set_title(quit_text) {
//             tracing::error!(menu = "quit", ?err, "set os menu failed");
//         }
//     }
//
//     Ok(())
// }


#[tracing::instrument(skip(app_state))]
pub async fn settings_theme_get(app_state: AppState) -> CoreResult<Option<Theme>> {
    let Some(ref storage) = *app_state.storage.lock().await else {
        return Err(core_error!("storage not initialize"));
    };

    storage.kv().get_theme()
}

#[tracing::instrument(skip(app_state))]
pub async fn settings_theme_set(app_state: AppState, theme: Theme) -> CoreResult<()> {
    let Some(ref storage) = *app_state.storage.lock().await else {
        return Err(core_error!("storage not initialize"));
    };

    storage.kv().set_theme(theme)?;

    Ok(())
}

impl AppState{
    pub async fn settings_init(&self) -> CoreResult<()> {
        let config_dir: PathBuf = dirs_sys_next::home_dir().map(|h|h.join("workspace/rust/NextRiftXR_desktop")).unwrap();

        std::fs::create_dir_all(config_dir.clone())?;
        let storage_path = config_dir.join("desktop.db");

        tracing::info!(path = ?storage_path, "read repository");

        let storage = LocalStorage::new(storage_path)?;
        let account_count = storage.account().get_account_count()?;

        let mut storage_guard = self.storage.lock().await;
        *storage_guard = Some(storage);
        drop(storage_guard); // 释放锁

        if account_count == 0 {

        }

        Ok(())
    }

    pub async fn get_default_account(&self) -> CoreResult<Account> {
        let Some(ref storage) = *self.storage.lock().await else {
            return Err(core_error!("storage not initialize"));
        };

        storage.account().get_default_account()
    }

    pub async fn storage_get_config(&self,account_id:u16) -> CoreResult<Config> {
        let Some(ref storage) = *self.storage.lock().await else {
            return Err(core_error!("storage not initialize"));
        };

        storage.config().get_account_config(account_id)
    }

    // pub async fn storage_set_config<T>(&self, key:String,value:T) -> CoreResult<()> {
    //     let Some(ref storage) = *self.storage.lock().await else {
    //         return Err(core_error!("storage not initialize"));
    //     };
    //
    //     storage.config().set_config_by_key(1, key, value)?;
    //
    //     Ok(())
    // }

    pub async fn storage_update_config(&self, cfg: Config) -> CoreResult<()> {
        let Some(ref storage) = *self.storage.lock().await else {
            return Err(core_error!("storage not initialize"));
        };

        storage.config().update_config(cfg.clone()).expect("storage.config().update_config fail.");
        Ok(())
    }
}
